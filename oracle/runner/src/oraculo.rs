//! O oraculo: o emulador sem tela, com entradas, estados, paradas, eventos e rotina isolada.

use crate::entradas::{Entradas, entradas_da_mascara};
use bincode::config::standard as config_bincode;
use genesis_config::{GenesisEmulatorConfig, GenesisRegion};
use genesis_core::api::oracle::Regs68;
use genesis_core::{GenesisEmulator, GenesisInputs};
use jgenesis_common::frontend::{
    AudioOutput, Color, EmulatorTrait, FrameSize, InputPoller, RenderFrameOptions, Renderer,
    SaveWriter, TickEffect, TimingMode,
};
use sonic1_fidelidade::hash::{em_hex, sha1};
use sonic1_fidelidade::{Cabecalho, Categoria, Evento, Origem, Quadro, Tipo};
use std::convert::Infallible;
use std::fmt;

/// Identificacao do oraculo no cabecalho do trace.
/// `patches/SHA1` guarda o hash dos patches aplicados sobre o commit fixado.
pub const ID_ORACULO: &str = concat!(
    "websonic-oracle jgenesis@386db855ec3aaa307df3321bb85df8b5cc20ac95 patches:",
    include_str!("../../patches/SHA1")
);

/// Limite de seguranca de passos por quadro (uma instrucao ou espera por passo).
const LIMITE_PASSOS_POR_QUADRO: u64 = 5_000_000;
const MAGIA_ESTADO: &[u8; 8] = b"S1ESTADO";

// ---- Sinks nulos: o oraculo nao tem tela, nem som, nem arquivo de save. ----

/// Tabela de niveis de cor do jgenesis na configuracao do oraculo (escala nao linear, sem
/// escurecimento): o nivel de 3 bits do canal e a posicao do valor de 8 bits nesta lista.
const NIVEIS_NORMAL: [u8; 8] = [0, 52, 87, 116, 144, 172, 206, 255];

/// Converte um canal de 8 bits no nivel de 3 bits. `None` se o valor nao esta na tabela
/// (por exemplo, um ponto de sombra ou realce).
fn nivel(valor: u8) -> Option<u16> {
    NIVEIS_NORMAL
        .iter()
        .position(|n| *n == valor)
        .and_then(|i| u16::try_from(i).ok())
}

/// Cor de 9 bits (`BBBGGGRRR`, 0 a 511) de um ponto do jgenesis.
pub fn cor9(c: &Color) -> Option<u16> {
    Some((nivel(c.b)? << 6) | (nivel(c.g)? << 3) | nivel(c.r)?)
}

/// Captura a imagem de cada quadro renderizado, ja na cor de 9 bits (pedido P6).
#[derive(Default)]
pub struct CapturaTela {
    /// Pontos do ultimo quadro, linha apos linha.
    pub pontos: Vec<u16>,
    pub largura: u32,
    pub altura: u32,
    /// Primeiro ponto cuja cor nao esta na tabela de 9 bits.
    pub erro: Option<String>,
}

impl Renderer for CapturaTela {
    type Err = Infallible;
    fn render_frame(
        &mut self,
        quadro: &[Color],
        tamanho: FrameSize,
        _fps: f64,
        _opcoes: RenderFrameOptions,
    ) -> Result<(), Infallible> {
        self.largura = tamanho.width;
        self.altura = tamanho.height;
        self.pontos.clear();
        self.erro = None;
        // O vetor do jgenesis tem tamanho fixo (maximo); so os primeiros largura x altura pontos valem.
        let total = usize::try_from(tamanho.width.saturating_mul(tamanho.height)).unwrap_or(0);
        for c in quadro.iter().take(total) {
            match cor9(c) {
                Some(v) => self.pontos.push(v),
                None => {
                    if self.erro.is_none() {
                        self.erro = Some("ponto com cor fora da tabela de 9 bits".into());
                    }
                    self.pontos.push(0);
                }
            }
        }
        Ok(())
    }
}

struct SemSom;
impl AudioOutput for SemSom {
    type Err = Infallible;
    fn push_sample(&mut self, _l: f64, _r: f64) -> Result<(), Infallible> {
        Ok(())
    }
}

#[derive(Debug)]
struct SemSave;
impl fmt::Display for SemSave {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "o oraculo nao usa saves")
    }
}

struct SemArquivos;
impl SaveWriter for SemArquivos {
    type Err = SemSave;
    fn load_bytes(&mut self, _ext: &str) -> Result<Vec<u8>, SemSave> {
        Err(SemSave)
    }
    fn persist_bytes(&mut self, _ext: &str, _bytes: &[u8]) -> Result<(), SemSave> {
        Ok(())
    }
    fn load_serialized<D: bincode::Decode<()>>(&mut self, _ext: &str) -> Result<D, SemSave> {
        Err(SemSave)
    }
    fn persist_serialized<E: bincode::Encode>(&mut self, _ext: &str, _d: E) -> Result<(), SemSave> {
        Ok(())
    }
}

struct Leitor<'a>(&'a GenesisInputs);
impl InputPoller<GenesisInputs> for Leitor<'_> {
    fn poll(&mut self) -> &GenesisInputs {
        self.0
    }
}

/// Configuracao fixa do console de referencia: Mega Drive japones, modelo 1, NTSC, 3 botoes.
pub fn config_console() -> GenesisEmulatorConfig {
    GenesisEmulatorConfig {
        forced_region: Some(GenesisRegion::Japan),
        forced_timing_mode: Some(TimingMode::Ntsc),
        ..GenesisEmulatorConfig::default()
    }
}

/// Condicoes de parada.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Parada {
    /// Chegou a um endereco: a instrucao nesse endereco ainda nao executou.
    Pc68(u32),
    /// Idem, para o Z80.
    Pc80(u16),
    /// O quadro de numero dado vai comecar (todos os anteriores foram fechados).
    InicioDoQuadro(u32),
    /// O relogio mestre chegou a pelo menos este valor.
    Mclk(u64),
}

/// Por que `executar_ate` parou.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotivoParada {
    Parada(Parada),
    LimiteDePassos,
}

/// Pedido de chamada de rotina isolada.
#[derive(Clone, Debug)]
pub struct ChamadaRotina {
    /// Registradores de entrada. `pc` e o endereco da rotina.
    pub regs: Regs68,
    /// Bytes a escrever na RAM do 68000 antes da chamada: (endereco, valor).
    pub pokes_ram68: Vec<(u32, u8)>,
    /// Endereco de retorno falso. A rotina para quando o PC chega a ele.
    pub sentinela: u32,
    /// Se verdadeiro (padrao), o SR entra com a mascara de interrupcoes em 7.
    pub mascarar_interrupcoes: bool,
    pub max_passos: u64,
}

/// Resultado da chamada de rotina isolada.
#[derive(Clone, Debug)]
pub struct ResultadoRotina {
    /// Registradores no retorno (PC igual a sentinela).
    pub regs: Regs68,
    pub passos: u64,
    /// Relogio mestre gasto pela rotina.
    pub mclk_gasto: u64,
    /// Bytes da RAM do 68000 que mudaram: (endereco, antes, depois).
    pub mudancas_ram68: Vec<(u32, u8, u8)>,
    /// Eventos produzidos durante a rotina.
    pub eventos: Vec<Evento>,
}

/// O oraculo.
pub struct Oraculo {
    emu: GenesisEmulator,
    inputs: GenesisInputs,
    entradas: Entradas,
    cabecalho: Cabecalho,
    quadro: u32,
    seq: u32,
    eventos: Vec<Evento>,
    brutos: Vec<oracle_trace::Raw>,
    // Copias do estado do YM2612 e do PSG, montadas so com as escritas observadas.
    ym: [u8; 512],
    ym_endereco: [u8; 2],
    psg: [u16; 8],
    psg_ultimo: u8,
    passos: u64,
    tela: CapturaTela,
    dma_quadro: Vec<u16>,
}

impl Oraculo {
    /// Cria o oraculo com a ROM dada (so os bytes: o oraculo nao le arquivos).
    pub fn novo(rom: Vec<u8>, entradas: Entradas) -> Result<Self, String> {
        let rom_sha1 = sha1(&rom);
        let config = config_console();
        let config_sha1 = sha1(format!("{config:?}|p1=3botoes|p2=nenhum").as_bytes());
        let cabecalho = Cabecalho {
            oraculo: ID_ORACULO.trim_end().to_owned(),
            rom_sha1,
            config_sha1,
            entradas_sha1: entradas.sha1(),
            modo: 0,
        };
        oracle_trace::set_enabled(true);
        oracle_trace::clear();
        oracle_trace::clear_dma();
        let emu = GenesisEmulator::create(
            genesis_core::api::GenesisHardware::Standalone,
            Some(rom),
            None,
            None,
            config,
            &mut SemArquivos,
        )
        .map_err(|e| format!("falha ao criar o emulador: {e}"))?;
        let inputs = entradas_da_mascara(entradas.do_quadro(0));
        let mut oraculo = Self {
            emu,
            inputs,
            entradas,
            cabecalho,
            quadro: 0,
            seq: 0,
            eventos: Vec::new(),
            brutos: Vec::new(),
            ym: [0; 512],
            ym_endereco: [0; 2],
            psg: [0; 8],
            psg_ultimo: 0,
            passos: 0,
            tela: CapturaTela::default(),
            dma_quadro: Vec::new(),
        };
        // Eventos gerados na criacao (reset) nao pertencem a nenhum quadro.
        oracle_trace::clear();
        oracle_trace::clear_dma();
        oraculo.brutos.clear();
        Ok(oraculo)
    }

    /// Imagem do ultimo quadro renderizado, na cor de 9 bits (pedido P6).
    pub fn imagem(&self) -> &CapturaTela {
        &self.tela
    }

    /// Palavras que o DMA de memoria para VDP leu no ultimo quadro fechado (pedido P7).
    pub fn dma_do_quadro(&self) -> &[u16] {
        &self.dma_quadro
    }

    pub fn cabecalho(&self) -> &Cabecalho {
        &self.cabecalho
    }

    /// Numero do quadro em andamento.
    pub fn quadro_atual(&self) -> u32 {
        self.quadro
    }

    pub fn passos(&self) -> u64 {
        self.passos
    }

    pub fn emulador(&self) -> &GenesisEmulator {
        &self.emu
    }

    pub fn emulador_mut(&mut self) -> &mut GenesisEmulator {
        &mut self.emu
    }

    /// Define os botoes da porta 1 a partir de agora, ate o proximo quadro.
    pub fn definir_botoes(&mut self, mascara: u8) {
        self.inputs = entradas_da_mascara(mascara);
    }

    /// Executa um passo: uma instrucao do 68000 (ou uma espera) e o que o acompanha.
    /// Devolve o quadro fechado, se este passo terminou um quadro.
    pub fn passo(&mut self) -> Result<Option<Quadro>, String> {
        let efeito = self
            .emu
            .tick(
                &mut self.tela,
                &mut SemSom,
                &mut Leitor(&self.inputs),
                &mut SemArquivos,
            )
            .map_err(|e| format!("erro no emulador: {e}"))?;
        self.passos += 1;
        self.coletar();
        if efeito == TickEffect::FrameRendered {
            self.dma_quadro = oracle_trace::take_dma();
            let quadro = self.fechar_quadro();
            // Os botoes do proximo quadro valem a partir daqui.
            self.inputs = entradas_da_mascara(self.entradas.do_quadro(self.quadro));
            Ok(Some(quadro))
        } else {
            Ok(None)
        }
    }

    /// Roda ate fechar o quadro em andamento.
    pub fn executar_quadro(&mut self) -> Result<Quadro, String> {
        for _ in 0..LIMITE_PASSOS_POR_QUADRO {
            if let Some(q) = self.passo()? {
                return Ok(q);
            }
        }
        Err("o quadro nao terminou dentro do limite de passos".into())
    }

    /// Roda ate uma das paradas, entregando cada quadro fechado a `ao_fechar`.
    /// Sempre executa pelo menos um passo. Para no momento em que a condicao passa a valer.
    pub fn executar_ate(
        &mut self,
        paradas: &[Parada],
        limite_passos: u64,
        ao_fechar: &mut dyn FnMut(Quadro),
    ) -> Result<MotivoParada, String> {
        for _ in 0..limite_passos {
            let pc68_antes = self.emu.oracle_pc68();
            let pc80_antes = self.emu.oracle_pc80();
            if let Some(q) = self.passo()? {
                ao_fechar(q);
            }
            for &parada in paradas {
                let atingida = match parada {
                    Parada::Pc68(a) => self.emu.oracle_pc68() == a && pc68_antes != a,
                    Parada::Pc80(a) => self.emu.oracle_pc80() == a && pc80_antes != a,
                    Parada::InicioDoQuadro(n) => self.quadro >= n,
                    Parada::Mclk(m) => self.emu.oracle_mclk() >= m,
                };
                if atingida {
                    return Ok(MotivoParada::Parada(parada));
                }
            }
        }
        Ok(MotivoParada::LimiteDePassos)
    }

    // ---- Eventos ----

    fn coletar(&mut self) {
        oracle_trace::drain(&mut self.brutos);
        let brutos = std::mem::take(&mut self.brutos);
        for raw in &brutos {
            let (Some(tipo), Some(origem)) = (Tipo::de_u8(raw.kind), Origem::de_u8(raw.src)) else {
                continue;
            };
            let mut evento = Evento {
                mclk: raw.mclk,
                seq: self.seq,
                tipo,
                origem,
                linha: raw.line,
                coluna: raw.col,
                a: raw.a,
                b: raw.b,
                c: raw.c,
                d: raw.d,
            };
            match tipo {
                Tipo::Ym => self.observar_ym(&mut evento),
                Tipo::Psg => self.observar_psg(raw.a as u8),
                _ => {}
            }
            self.seq = self.seq.wrapping_add(1);
            self.eventos.push(evento);
        }
        self.brutos = brutos;
        self.brutos.clear();
    }

    /// Acompanha o YM2612: guarda o registrador escolhido e o valor escrito. Para escritas de dado,
    /// `c` recebe o registrador e `d` vale 1 se for o DAC (registrador $2A).
    fn observar_ym(&mut self, evento: &mut Evento) {
        let porta = (evento.a & 3) as usize;
        let parte = porta >> 1;
        let valor = evento.b as u8;
        if porta & 1 == 0 {
            self.ym_endereco[parte] = valor;
        } else {
            let registrador = self.ym_endereco[parte];
            self.ym[parte * 256 + usize::from(registrador)] = valor;
            evento.c = registrador.into();
            evento.d = u32::from(parte == 0 && registrador == 0x2A);
        }
    }

    /// Acompanha o PSG: 4 canais de tom e volume, mais o ruido.
    fn observar_psg(&mut self, byte: u8) {
        if byte & 0x80 != 0 {
            self.psg_ultimo = (byte >> 4) & 7;
            let reg = usize::from(self.psg_ultimo);
            self.psg[reg] = (self.psg[reg] & !0x0F) | u16::from(byte & 0x0F);
        } else {
            let reg = usize::from(self.psg_ultimo);
            // Dado de tom: so os registradores de tom (0, 2, 4) usam os 6 bits altos.
            if reg & 1 == 0 && reg < 6 {
                self.psg[reg] = (self.psg[reg] & 0x0F) | (u16::from(byte & 0x3F) << 4);
            } else {
                self.psg[reg] = (self.psg[reg] & !0x0F) | u16::from(byte & 0x0F);
            }
        }
    }

    /// Fecha o quadro em andamento, com os hashes de cada categoria.
    fn fechar_quadro(&mut self) -> Quadro {
        let mut hashes = [[0u8; 20]; Categoria::TOTAL];
        hashes[Categoria::Ram68k as usize] = sha1(&self.ram68k());
        hashes[Categoria::RamZ80 as usize] = sha1(&self.ram_z80());
        hashes[Categoria::Vram as usize] = sha1(self.emu.oracle_vram());
        hashes[Categoria::Cram as usize] = sha1(&palavras_em_bytes(self.emu.oracle_cram()));
        hashes[Categoria::Vsram as usize] = sha1(&palavras_em_bytes(self.emu.oracle_vsram()));
        hashes[Categoria::RegsVdp as usize] = sha1(self.emu.oracle_registradores_vdp().as_bytes());
        hashes[Categoria::Ym as usize] = sha1(&self.ym);
        hashes[Categoria::Psg as usize] = sha1(&palavras_em_bytes(&self.psg));
        let mut quadro = Quadro {
            numero: self.quadro,
            mclk_fim: self.emu.oracle_mclk(),
            eventos: std::mem::take(&mut self.eventos),
            hashes,
        };
        quadro.fechar();
        self.quadro = self.quadro.wrapping_add(1);
        self.seq = 0;
        quadro
    }

    // ---- Memoria ----

    /// RAM do 68000 inteira (64 KiB, $FF0000 a $FFFFFF).
    pub fn ram68k(&self) -> Vec<u8> {
        (0..0x1_0000u32)
            .map(|i| self.emu.oracle_ram68k_byte(0xFF_0000 + i))
            .collect()
    }

    /// RAM do Z80 inteira (8 KiB).
    pub fn ram_z80(&self) -> Vec<u8> {
        (0..0x2000u16)
            .map(|i| self.emu.oracle_ram_z80_byte(i))
            .collect()
    }

    pub fn vram(&self) -> &[u8] {
        self.emu.oracle_vram()
    }

    pub fn cram(&self) -> &[u16] {
        self.emu.oracle_cram()
    }

    pub fn vsram(&self) -> &[u16] {
        self.emu.oracle_vsram()
    }

    /// Le um byte da RAM do 68000 (endereco entre $FF0000 e $FFFFFF).
    pub fn ler_byte_ram68k(&self, endereco: u32) -> u8 {
        self.emu.oracle_ram68k_byte(endereco)
    }

    /// Le uma palavra da RAM do 68000, big-endian.
    pub fn ler_palavra_ram68k(&self, endereco: u32) -> u16 {
        u16::from_be_bytes([
            self.ler_byte_ram68k(endereco),
            self.ler_byte_ram68k(endereco + 1),
        ])
    }

    // ---- Estados ----

    /// Serializa o estado completo: emulador, copias do YM e do PSG, contadores e eventos
    /// do quadro em andamento. Nao inclui a ROM.
    pub fn salvar_estado(&self) -> Result<Vec<u8>, String> {
        let emu_bytes = bincode::encode_to_vec(self.emu.to_save_state(), config_bincode())
            .map_err(|e| format!("falha ao serializar o emulador: {e}"))?;
        let mut saida = Vec::with_capacity(emu_bytes.len() + 4096);
        saida.extend_from_slice(MAGIA_ESTADO);
        saida.extend_from_slice(&self.quadro.to_le_bytes());
        saida.extend_from_slice(&self.seq.to_le_bytes());
        saida.extend_from_slice(&self.passos.to_le_bytes());
        saida.extend_from_slice(&self.ym);
        saida.extend_from_slice(&self.ym_endereco);
        for p in &self.psg {
            saida.extend_from_slice(&p.to_le_bytes());
        }
        saida.push(self.psg_ultimo);
        saida.push(self.inputs_mascara_atual());
        let n = u32::try_from(self.eventos.len()).map_err(|_| "eventos demais")?;
        saida.extend_from_slice(&n.to_le_bytes());
        for e in &self.eventos {
            saida.extend_from_slice(&e.para_bytes());
        }
        saida.extend_from_slice(
            &u64::try_from(emu_bytes.len())
                .map_err(|_| "estado grande")?
                .to_le_bytes(),
        );
        saida.extend_from_slice(&emu_bytes);
        Ok(saida)
    }

    fn inputs_mascara_atual(&self) -> u8 {
        self.entradas.do_quadro(self.quadro)
    }

    /// Carrega um estado salvo por `salvar_estado`. A ROM e as entradas ficam as do oraculo atual.
    pub fn carregar_estado(&mut self, bytes: &[u8]) -> Result<(), String> {
        let mut p = Cursor {
            dados: bytes,
            pos: 0,
        };
        if p.pegar(8)? != MAGIA_ESTADO {
            return Err("nao e um estado do oraculo".into());
        }
        let quadro = p.u32()?;
        let seq = p.u32()?;
        let passos = p.u64()?;
        let ym: [u8; 512] = p.pegar(512)?.try_into().map_err(|_| "ym")?;
        let ym_endereco: [u8; 2] = p.pegar(2)?.try_into().map_err(|_| "ym_endereco")?;
        let mut psg = [0u16; 8];
        for r in &mut psg {
            *r = u16::from_le_bytes(p.pegar(2)?.try_into().map_err(|_| "psg")?);
        }
        let psg_ultimo = p.pegar(1)?[0];
        let mascara = p.pegar(1)?[0];
        let n = p.u32()?;
        let mut eventos = Vec::new();
        for _ in 0..n {
            let b: [u8; sonic1_fidelidade::TAMANHO_EVENTO] = p
                .pegar(sonic1_fidelidade::TAMANHO_EVENTO)?
                .try_into()
                .map_err(|_| "evento")?;
            eventos.push(Evento::de_bytes(&b).map_err(|e| e.to_string())?);
        }
        let tam = usize::try_from(p.u64()?).map_err(|_| "tamanho")?;
        let emu_bytes = p.pegar(tam)?;
        let (estado, _): (GenesisEmulator, usize) =
            bincode::decode_from_slice(emu_bytes, config_bincode())
                .map_err(|e| format!("falha ao ler o emulador: {e}"))?;
        self.emu.load_state(estado);
        self.quadro = quadro;
        self.seq = seq;
        self.passos = passos;
        self.ym = ym;
        self.ym_endereco = ym_endereco;
        self.psg = psg;
        self.psg_ultimo = psg_ultimo;
        self.eventos = eventos;
        self.inputs = entradas_da_mascara(mascara);
        self.brutos.clear();
        oracle_trace::clear();
        oracle_trace::clear_dma();
        Ok(())
    }

    // ---- Rotina isolada ----

    /// Chama uma rotina do 68000 a partir do estado atual e dos registradores dados.
    /// O oraculo empilha `sentinela` como endereco de retorno, roda ate o PC chegar a ele e
    /// devolve registradores, mudancas na RAM e eventos.
    pub fn chamar_rotina(&mut self, chamada: &ChamadaRotina) -> Result<ResultadoRotina, String> {
        for &(endereco, valor) in &chamada.pokes_ram68 {
            self.emu.oracle_escrever_ram68k_byte(endereco, valor);
        }
        let mut regs = chamada.regs;
        if chamada.mascarar_interrupcoes {
            regs.sr |= 0x0700;
        }
        // A pilha em uso depende do modo (supervisor ou usuario).
        let supervisor = regs.sr & 0x2000 != 0;
        let sp = if supervisor { regs.ssp } else { regs.usp };
        let novo_sp = sp.wrapping_sub(4);
        for (i, byte) in chamada.sentinela.to_be_bytes().into_iter().enumerate() {
            let deslocamento = u32::try_from(i).map_err(|_| "indice")?;
            self.emu
                .oracle_escrever_ram68k_byte(novo_sp.wrapping_add(deslocamento), byte);
        }
        if supervisor {
            regs.ssp = novo_sp;
        } else {
            regs.usp = novo_sp;
        }
        self.emu.oracle_definir_regs68(&regs);

        let antes = self.ram68k();
        let mclk_inicio = self.emu.oracle_mclk();
        let passos_inicio = self.passos;
        let eventos_inicio = self.eventos.len();
        let mut quadros_fechados = 0u32;
        let motivo = self.executar_ate(
            &[Parada::Pc68(chamada.sentinela)],
            chamada.max_passos,
            &mut |_| quadros_fechados += 1,
        )?;
        if motivo != MotivoParada::Parada(Parada::Pc68(chamada.sentinela)) {
            return Err("a rotina nao retornou dentro do limite de passos".into());
        }
        let depois = self.ram68k();
        let mudancas_ram68 = antes
            .iter()
            .zip(&depois)
            .enumerate()
            .filter(|(_, (a, d))| a != d)
            .map(|(i, (a, d))| (0xFF_0000 + u32::try_from(i).unwrap_or(0), *a, *d))
            .collect();
        // Se a rotina atravessou o fim de um quadro, os eventos anteriores ja foram entregues.
        let eventos = if quadros_fechados == 0 {
            self.eventos[eventos_inicio..].to_vec()
        } else {
            self.eventos.clone()
        };
        Ok(ResultadoRotina {
            regs: self.emu.oracle_regs68(),
            passos: self.passos - passos_inicio,
            mclk_gasto: self.emu.oracle_mclk() - mclk_inicio,
            mudancas_ram68,
            eventos,
        })
    }

    /// Texto curto para relatorios: so hashes, sem dado de memoria.
    pub fn resumo_hashes(&self) -> String {
        format!(
            "quadro={} ram68k={} vram={}",
            self.quadro,
            em_hex(&sha1(&self.ram68k())),
            em_hex(&sha1(self.vram()))
        )
    }
}

fn palavras_em_bytes(palavras: &[u16]) -> Vec<u8> {
    palavras.iter().flat_map(|p| p.to_le_bytes()).collect()
}

struct Cursor<'a> {
    dados: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn pegar(&mut self, n: usize) -> Result<&'a [u8], String> {
        let fim = self.pos.checked_add(n).ok_or("estado truncado")?;
        let fatia = self.dados.get(self.pos..fim).ok_or("estado truncado")?;
        self.pos = fim;
        Ok(fatia)
    }

    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(
            self.pegar(4)?.try_into().map_err(|_| "u32")?,
        ))
    }

    fn u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(
            self.pegar(8)?.try_into().map_err(|_| "u64")?,
        ))
    }
}
