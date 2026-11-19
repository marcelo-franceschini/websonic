//! Ponte entre o núcleo e o renderizador (Etapa 7).
//!
//! O núcleo manda escritas na VDP. O renderizador precisa da linha e da coluna de cada escrita
//! e, nos DMAs da memória para a VDP, das palavras que o DMA leu. Esta ponte junta as duas coisas.
//! No navegador a posição é nominal (sem tempo gravado, D7.5). No comparador a posição vem do
//! oráculo (D7.3).

use std::cell::RefCell;
use std::rc::Rc;

use sonic1_audio::{Chip, MCLK_POR_QUADRO_NTSC, Sintetizador};
use sonic1_audio_saida::MisturadorSaida;
use sonic1_core::abertura::{Abertura, Espera, EventoQuadro, Marca, MomentoSaida};
use sonic1_core::memoria::{Memoria, PortaVdp, Saida, Tamanho};
use sonic1_video::quadro::{ALTURA, LARGURA};
use sonic1_video::renderizador::{Escrita, Renderizador};

/// Quadros que cada trecho espera no tempo nominal, medidos no oráculo (bloco 1 e rodada 3 do
/// bloco 2). São provisórios até o modelo do nível 2 (Etapa 15).
fn quadros_de_espera(m: Marca) -> u32 {
    match m {
        Marca::GameProgram => 3,
        Marca::ChecksumOk => 49,
        Marca::VdpSetupGame => 3,
        Marca::DacDriverLoad => 1,
        Marca::JoypadInit => 5,
        Marca::MainGameLoop | Marca::GmSega => 0,
        Marca::SegaWaitPal => 0,
        Marca::SegaWaitEnd | Marca::GmTitle => 0,
        Marca::PaletteFadeIn => 5,
        // A parte B entra logo depois da última VBlank do aparecimento (sem espera própria).
        Marca::PaletteFadeOut => 0,
        Marca::TitMainLoop => 0,
        // Vem junto com o VBlank (ver `eventos_nominais`); o console não espera por ela.
        Marca::WaitForVBlank => 0,
        // `GotoDemo` não é espera do núcleo; `GM_Level` vem logo depois do `rts` do `GotoDemo`.
        Marca::GotoDemo | Marca::GmLevel => 0,
        // Arte dos cartoes e limpeza da RAM: medido no oraculo (2 quadros de lag).
        Marca::LevelTtlCardLoop => 2,
        // Carga final da fase (descompressoes Kosinski/Enigma e blocos): 25 quadros de lag no oraculo.
        Marca::LevelDelay => 25,
        Marca::LevelStartGame => 0,
    }
}

/// Quadros que o tratador da voz leva até devolver o controle (134 no oráculo).
const QUADROS_DA_VOZ: u32 = 134;

/// Uma escrita da VDP já acompanhada pela ponte, com o DMA da memória que ela dispara, se houver.
#[derive(Clone, Debug)]
pub struct Capturada {
    porta: PortaVdp,
    valor: u16,
    tamanho: Tamanho,
    dma: Option<Vec<u16>>,
}

/// Escrita da VDP com a faixa de palavras de DMA lidas na memória neste quadro, se houver.
#[derive(Clone, Copy, Debug)]
struct CapturadaInterna {
    porta: PortaVdp,
    valor: u16,
    tamanho: Tamanho,
    faixa_dma: Option<(usize, usize)>,
}

/// A ponte: aplica as escritas do núcleo ao renderizador.
pub struct Ponte {
    rend: Renderizador,
    /// Primeira palavra de um comando de duas palavras, à espera da segunda.
    primeira: Option<u16>,
    /// Cópia dos registradores da VDP que a ponte precisa para o DMA. O renderizador guarda as
    /// escritas atrás de um DMA em andamento, então os registradores dele podem estar atrasados.
    regs: [u8; 24],
}

impl Default for Ponte {
    fn default() -> Self {
        Self::nova()
    }
}

impl Ponte {
    pub fn nova() -> Self {
        Self {
            rend: Renderizador::novo(),
            primeira: None,
            regs: [0; 24],
        }
    }

    pub fn renderizador(&self) -> &Renderizador {
        &self.rend
    }

    /// Liga ou desliga a emulação dos pontos de CRAM analógicos do hardware original.
    pub fn definir_pontos_cram(&mut self, ativos: bool) {
        self.rend.definir_pontos_cram(ativos);
    }

    /// Informa se a emulação dos pontos de CRAM analógicos está ativa.
    pub fn pontos_cram_ativos(&self) -> bool {
        self.rend.pontos_cram_ativos()
    }

    /// Lê as palavras de um DMA da memória para a VDP, do endereço dado pelos registradores.
    fn coletar_palavras_dma(&self, mem: &mut Memoria, buffer: &mut Vec<u16>) -> (usize, usize) {
        let inicio = buffer.len();
        let reg = |i: usize| self.regs.get(i).copied().unwrap_or(0);
        let base =
            u32::from(reg(21)) | (u32::from(reg(22)) << 8) | (u32::from(reg(23) & 0x7F) << 16);
        let mut a = base.wrapping_mul(2);
        let n = match u32::from(reg(19)) | (u32::from(reg(20)) << 8) {
            0 => 0x1_0000,
            n => n,
        };
        buffer.reserve(usize::try_from(n).unwrap_or(0));
        for _ in 0..n {
            let w = mem.ler(a, Tamanho::Word).unwrap_or(0);
            buffer.push(u16::try_from(w & 0xFFFF).unwrap_or(0));
            a = a.wrapping_add(2);
        }
        (inicio, buffer.len())
    }

    /// O DMA da memória para a VDP está permitido e pedido, pelos registradores copiados?
    fn dma_de_memoria(&self) -> bool {
        let r1 = self.regs.get(1).copied().unwrap_or(0);
        let r23 = self.regs.get(23).copied().unwrap_or(0);
        r1 & 0x10 != 0 && r23 >> 6 < 2
    }

    /// Primeira fase de `aplicar`: acompanha os registradores e, se a saída pede um DMA da
    /// memória, lê as palavras agora (a memória muda depois). Devolve `None` para o que não é VDP.
    pub fn capturar(&mut self, mem: &mut Memoria, saida: &Saida) -> Option<Capturada> {
        let mut palavras = Vec::new();
        let c = self.capturar_interna(mem, saida, &mut palavras)?;
        let dma = c.faixa_dma.map(|_| palavras);
        Some(Capturada {
            porta: c.porta,
            valor: c.valor,
            tamanho: c.tamanho,
            dma,
        })
    }

    /// Variante interna que grava palavras de DMA em buffer reutilizado por quadro.
    fn capturar_interna(
        &mut self,
        mem: &mut Memoria,
        saida: &Saida,
        palavras_dma: &mut Vec<u16>,
    ) -> Option<CapturadaInterna> {
        let Saida::Vdp {
            porta,
            valor,
            tamanho,
        } = *saida
        else {
            return None;
        };
        let mut faixa_dma = None;
        if porta == PortaVdp::Controle {
            let e_registrador = valor & 0xC000 == 0x8000;
            if e_registrador {
                self.primeira = None;
                let n = usize::from((valor >> 8) & 0x1F);
                if let Some(r) = self.regs.get_mut(n) {
                    *r = valor.to_le_bytes()[0];
                }
            } else if let Some(_primeira) = self.primeira.take() {
                if valor & 0x0080 != 0 && self.dma_de_memoria() {
                    faixa_dma = Some(self.coletar_palavras_dma(mem, palavras_dma));
                }
            } else {
                self.primeira = Some(valor);
            }
        }
        Some(CapturadaInterna {
            porta,
            valor,
            tamanho,
            faixa_dma,
        })
    }

    /// Aplica uma escrita capturada internamente, fatiando o buffer de palavras de DMA.
    fn aplicar_capturada_interna(
        &mut self,
        c: &CapturadaInterna,
        palavras_dma: &[u16],
        linha: u16,
        coluna: u64,
    ) {
        match c.porta {
            PortaVdp::Dados => {
                let t = if c.tamanho == Tamanho::Byte { 1 } else { 2 };
                self.rend.aplicar(
                    linha,
                    coluna,
                    Escrita::Dados {
                        valor: c.valor,
                        tamanho: t,
                    },
                );
            }
            PortaVdp::Controle => {
                if let Some((inicio, fim)) = c.faixa_dma {
                    if let Some(palavras) = palavras_dma.get(inicio..fim) {
                        self.rend.entregar_dma(palavras);
                    }
                }
                self.rend.aplicar(linha, coluna, Escrita::Controle(c.valor));
            }
        }
    }

    /// Segunda fase: entrega o DMA lido e aplica a escrita na posição dada.
    pub fn aplicar_capturada(&mut self, c: &Capturada, linha: u16, coluna: u64) {
        let palavras = c.dma.as_deref().unwrap_or(&[]);
        let faixa_dma = c.dma.as_ref().map(|v| (0, v.len()));
        let interna = CapturadaInterna {
            porta: c.porta,
            valor: c.valor,
            tamanho: c.tamanho,
            faixa_dma,
        };
        self.aplicar_capturada_interna(&interna, palavras, linha, coluna);
    }

    /// Aplica uma saída do núcleo na posição dada. Só as escritas da VDP interessam aqui.
    pub fn aplicar(&mut self, mem: &mut Memoria, saida: &Saida, linha: u16, coluna: u64) {
        if let Some(c) = self.capturar(mem, saida) {
            self.aplicar_capturada(&c, linha, coluna);
        }
    }

    /// Fecha o quadro na coluna dada da linha 232 (como o oráculo fecha) e guarda a imagem.
    pub fn fechar_quadro_em(&mut self, coluna: u64) {
        self.rend.fechar_quadro_em(coluna);
    }

    /// Fecha o quadro e devolve a imagem em RGBA (320 por 224).
    pub fn fechar_quadro(&mut self) -> Vec<u8> {
        let q = self.rend.fechar_quadro();
        let mut rgba = Vec::with_capacity(LARGURA.saturating_mul(ALTURA).saturating_mul(4));
        for y in 0..ALTURA {
            for x in 0..LARGURA {
                let c = q.ponto(x, y).unwrap_or(0);
                let (b, g, r) = ((c >> 6) & 7, (c >> 3) & 7, c & 7);
                // 3 bits para 8: replica os bits altos.
                let exp = |v: u16| -> u8 {
                    let v = u8::try_from(v).unwrap_or(0);
                    (v << 5) | (v << 2) | (v >> 1)
                };
                rgba.extend([exp(r), exp(g), exp(b), 255]);
            }
        }
        rgba
    }

    /// Hash simples (FNV-1a de 64 bits) da imagem de 9 bits, para testes.
    pub fn hash_da_imagem(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for y in 0..ALTURA {
            for x in 0..LARGURA {
                let c = self.rend.quadro().ponto(x, y).unwrap_or(0);
                for b in c.to_le_bytes() {
                    h ^= u64::from(b);
                    h = h.wrapping_mul(0x0000_0100_0000_01B3);
                }
            }
        }
        h
    }
}

/// Captura DMA antes de o laço principal alterar a memória de origem.
struct CapturasEventos {
    ponte: Ponte,
    saidas: Vec<(Saida, Option<CapturadaInterna>, MomentoSaida, Option<u8>)>,
    palavras_dma: Vec<u16>,
    linha_hblank: Option<u8>,
}

impl CapturasEventos {
    fn nova() -> Self {
        Self {
            ponte: Ponte::nova(),
            saidas: Vec::new(),
            palavras_dma: Vec::new(),
            linha_hblank: None,
        }
    }

    fn recolher(&mut self, mem: &mut Memoria, momento: MomentoSaida, novas: &[Saida]) {
        for &saida in novas {
            let capturada = self
                .ponte
                .capturar_interna(mem, &saida, &mut self.palavras_dma);
            let linha = if momento == MomentoSaida::HBlank {
                self.linha_hblank
            } else {
                None
            };
            self.saidas.push((saida, capturada, momento, linha));
        }
        if momento == MomentoSaida::VBlank {
            self.linha_hblank = Some(self.ponte.regs[10]);
        }
    }
}

/// O console completo no navegador: núcleo com tempo nominal mais a ponte e o sintetizador de som.
pub struct Console {
    pub ab: Abertura,
    ponte: Ponte,
    capturas: Rc<RefCell<CapturasEventos>>,
    espera_atual: Option<Espera>,
    quadros_na_espera: u32,
    escritas_no_quadro: usize,
    sint: Sintetizador,
    misturador: MisturadorSaida,
    quadro_num: u64,
    buffer_audio_quadro: Vec<f32>,
    amostras_fm: Vec<(i16, i16)>,
    amostras_psg: Vec<u16>,
    quadros_lag_pendentes: u32,
    eventos_quadro: Vec<EventoQuadro>,
    saidas_anteriores: Vec<Saida>,
    saidas_reserva: Vec<(Saida, Option<CapturadaInterna>, MomentoSaida, Option<u8>)>,
}

impl Console {
    pub fn novo(rom: Vec<u8>) -> Self {
        Self::com_taxa_audio(rom, 48_000.0)
    }

    /// Cria o console usando a taxa efetiva do dispositivo de áudio.
    pub fn com_taxa_audio(rom: Vec<u8>, taxa_audio: f64) -> Self {
        let capturas = Rc::new(RefCell::new(CapturasEventos::nova()));
        let observadas = Rc::clone(&capturas);
        let mut ab = Abertura::nova_nominal(rom);
        ab.definir_observador_temporal(Box::new(move |mem, momento, novas| {
            observadas.borrow_mut().recolher(mem, momento, novas)
        }));
        Self {
            ab,
            ponte: Ponte::nova(),
            capturas,
            espera_atual: None,
            quadros_na_espera: 0,
            escritas_no_quadro: 0,
            sint: Sintetizador::novo(),
            misturador: MisturadorSaida::novo(taxa_audio),
            quadro_num: 0,
            buffer_audio_quadro: Vec::new(),
            amostras_fm: Vec::new(),
            amostras_psg: Vec::new(),
            quadros_lag_pendentes: 0,
            eventos_quadro: Vec::new(),
            saidas_anteriores: Vec::new(),
            saidas_reserva: Vec::new(),
        }
    }

    /// Preenche os eventos nominais do quadro no vetor reutilizado.
    fn preencher_eventos_nominais(&mut self) {
        self.eventos_quadro.clear();
        if self.quadros_lag_pendentes > 0 {
            self.quadros_lag_pendentes = self.quadros_lag_pendentes.saturating_sub(1);
            self.eventos_quadro
                .push(EventoQuadro::Interrupcao { nivel: 6, pc: 0 });
            return;
        }
        let espera = self.ab.espera();
        if self.espera_atual != Some(espera) {
            self.espera_atual = Some(espera);
            self.quadros_na_espera = 0;
        }
        let ja = self.quadros_na_espera;
        self.quadros_na_espera = self.quadros_na_espera.saturating_add(1);
        match espera {
            Espera::Marca(m) if ja >= quadros_de_espera(m) => {
                self.eventos_quadro.push(EventoQuadro::Marca(m));
            }
            // No tempo nominal não há lag: o laço chega à espera antes de cada VBlank (D7.25).
            Espera::VBlank => {
                self.eventos_quadro
                    .push(EventoQuadro::Marca(Marca::WaitForVBlank));
                self.eventos_quadro
                    .push(EventoQuadro::Interrupcao { nivel: 6, pc: 0 });
            }
            Espera::VozAcabando if ja >= QUADROS_DA_VOZ => {
                self.eventos_quadro
                    .push(EventoQuadro::Interrupcao { nivel: 6, pc: 0 });
            }
            _ => {}
        }
    }

    /// Botões apertados (bits: cima, baixo, esquerda, direita, B, C, A, Start).
    pub fn definir_botoes(&mut self, mascara: u8) {
        self.ab.mem.definir_botoes(mascara);
    }

    /// Imagem de 9 bits do quadro gerada pelo renderizador fiel da CPU (320 x 224 pontos).
    pub fn imagem_9bits(&self) -> &[u16] {
        self.ponte.renderizador().quadro().pontos()
    }

    /// Informa a ocupação da fila de áudio do navegador em pares estéreo.
    pub fn definir_ocupacao_audio(&mut self, pares: usize) {
        self.misturador.definir_ocupacao_externa(pares);
    }

    /// Liga ou desliga a emulação dos pontos de CRAM analógicos do hardware original.
    pub fn definir_pontos_cram(&mut self, ativos: bool) {
        self.ponte.definir_pontos_cram(ativos);
    }

    /// Informa se a emulação dos pontos de CRAM analógicos está ativa.
    pub fn pontos_cram_ativos(&self) -> bool {
        self.ponte.pontos_cram_ativos()
    }

    /// Roda um quadro e devolve a imagem em RGBA (320 por 224).
    pub fn executar_quadro(&mut self) -> Vec<u8> {
        self.executar_quadro_interno(true)
    }

    /// Avança a simulação sem construir uma imagem RGBA na CPU.
    pub fn simular_quadro(&mut self) {
        self.executar_quadro_interno(false);
    }

    fn executar_quadro_interno(&mut self, imagem: bool) -> Vec<u8> {
        self.capturas.borrow_mut().palavras_dma.clear();
        // Saídas externas podem ser inseridas antes de qualquer evento do núcleo.
        self.ab.mem.drenar_saidas_em(&mut self.saidas_anteriores);
        self.capturas.borrow_mut().recolher(
            &mut self.ab.mem,
            MomentoSaida::Principal,
            &self.saidas_anteriores,
        );
        self.preencher_eventos_nominais();
        let teve_vblank = self
            .eventos_quadro
            .iter()
            .any(|e| matches!(e, EventoQuadro::Interrupcao { .. }));
        self.ab.executar_quadro(&self.eventos_quadro);

        self.quadro_num = self.quadro_num.wrapping_add(1);
        let mclk_inicio = self
            .quadro_num
            .wrapping_sub(1)
            .wrapping_mul(MCLK_POR_QUADRO_NTSC);
        let mclk_fim = self.quadro_num.wrapping_mul(MCLK_POR_QUADRO_NTSC);

        // Posicao nominal: o tratador de VBlank escreve na linha 224, o resto na linha 0.
        let linha = if teve_vblank { 224 } else { 0 };
        self.ab.mem.descartar_saidas();
        std::mem::swap(
            &mut self.capturas.borrow_mut().saidas,
            &mut self.saidas_reserva,
        );
        self.escritas_no_quadro = self.saidas_reserva.len();

        let n_saidas = self.saidas_reserva.len();
        {
            let cap = self.capturas.borrow();
            let dma_palavras = &cap.palavras_dma;
            for (i, (s, capturada, _momento, _linha_hblank)) in
                self.saidas_reserva.iter().enumerate()
            {
                let delta = if n_saidas > 0 {
                    (MCLK_POR_QUADRO_NTSC.wrapping_mul(i as u64)) / (n_saidas as u64)
                } else {
                    0
                };
                let mclk_esc = mclk_inicio.wrapping_add(delta);

                match s {
                    Saida::Vdp { .. } => {
                        if let Some(c) = capturada {
                            self.ponte
                                .aplicar_capturada_interna(c, dma_palavras, linha, 0);
                        }
                    }
                    Saida::Ym { porta, valor } | Saida::YmZ80 { porta, valor } => {
                        self.sint
                            .agendar(mclk_esc, Chip::Fm, u32::from(*porta), *valor);
                    }
                    Saida::Psg(valor) => {
                        self.sint.agendar(mclk_esc, Chip::Psg, 0, *valor);
                    }
                    _ => {}
                }
            }
        }
        self.saidas_reserva.clear();

        // Sintetiza o áudio gerado neste quadro e alimenta o misturador
        self.amostras_fm.clear();
        self.amostras_psg.clear();
        self.sint
            .avancar_ate_compacto(mclk_fim, &mut self.amostras_fm, &mut self.amostras_psg);
        self.misturador
            .alimentar_quadro_compacto(&self.amostras_fm, &self.amostras_psg);

        // Extrai as amostras reamostradas para reprodução no navegador
        self.buffer_audio_quadro.clear();
        let disp = self.misturador.ocupacao_buffer();
        if disp > 0 {
            let total = disp.saturating_mul(2);
            self.buffer_audio_quadro.resize(total, 0.0);
            self.misturador
                .extrair_bloco_worklet(&mut self.buffer_audio_quadro);
        }

        let resultado = if imagem {
            self.ponte.fechar_quadro()
        } else {
            self.ponte.rend.fechar_quadro();
            Vec::new()
        };

        resultado
    }

    /// Modo atual de jogo (0 = Sega, 4 = Titulo, 8 = Demo, 0x0C = Level, etc).
    pub fn modo_jogo(&self) -> u8 {
        self.ab.mem.ram()[0xF600]
    }

    /// Devolve as amostras de áudio estereo geradas no último quadro (intercaladas: L, R, L, R...).
    pub fn extrair_audio(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.buffer_audio_quadro)
    }

    /// Fatia das amostras de áudio estereo geradas no último quadro.
    pub fn audio_slice(&self) -> &[f32] {
        &self.buffer_audio_quadro
    }

    /// Amostras nativas do último quadro: FM misturado `(esquerda, direita)` e PSG empacotado.
    /// Servem à guarda `guarda_sintese` (a saída de cada chip, antes de reamostrar).
    pub fn amostras_nativas(&self) -> (&[(i16, i16)], &[u16]) {
        (&self.amostras_fm, &self.amostras_psg)
    }

    pub fn hash_da_imagem(&self) -> u64 {
        self.ponte.hash_da_imagem()
    }

    /// Números para achar defeitos: o que o laço espera, quantas escritas o quadro teve, quantas
    /// cores e bytes de VRAM não são zero e se a imagem está ligada. Nenhum valor de dado.
    pub fn diagnostico(&self) -> String {
        let v = self.ponte.renderizador().vdp();
        let cram = v.cram().iter().filter(|c| **c != 0).count();
        let vram = v.vram().iter().filter(|b| **b != 0).count();
        format!(
            "espera={:?} escritas={} cram_nao_zero={cram} vram_nao_zero={vram} imagem_ligada={} avisos={}",
            self.ab.espera(),
            self.escritas_no_quadro,
            v.imagem_ligada(),
            v.avisos.total()
        )
    }

    /// Informa se o núcleo encontrou algum erro e parou.
    pub fn tem_erro(&self) -> bool {
        self.ab.erro.is_some()
    }

    pub fn erro(&self) -> Option<String> {
        self.ab.erro.map(|e| format!("{e:?}"))
    }
}

#[cfg(test)]
mod testes_captura_eventos {
    use super::*;

    #[test]
    fn observadores_veem_as_mesmas_saidas_em_eventos_consecutivos() {
        let mut abertura = Abertura::nova_nominal(vec![0; 0x10000]);
        let temporal = Rc::new(RefCell::new(Vec::new()));
        let legado = Rc::new(RefCell::new(Vec::new()));
        let t = Rc::clone(&temporal);
        let l = Rc::clone(&legado);
        abertura.definir_observador_temporal(Box::new(move |_, _, novas| {
            t.borrow_mut().extend_from_slice(novas);
        }));
        abertura.definir_observador(Box::new(move |mem| {
            l.borrow_mut().extend(mem.tirar_saidas());
        }));
        for _ in 0..2 {
            abertura.mem.escrever(0xFF_F644, Tamanho::Word, 1).unwrap();
            abertura.executar_quadro(&[EventoQuadro::Interrupcao { nivel: 4, pc: 0 }]);
        }
        assert!(!temporal.borrow().is_empty());
        assert_eq!(*temporal.borrow(), *legado.borrow());
        assert_eq!(temporal.borrow().len(), 134);
    }

    #[test]
    fn evento_hblank_e_identificado_pelo_observador_do_nucleo() {
        let capturas = Rc::new(RefCell::new(CapturasEventos::nova()));
        let observadas = Rc::clone(&capturas);
        let mut abertura = Abertura::nova_nominal(vec![0; 0x10000]);
        abertura.definir_observador_temporal(Box::new(move |mem, momento, novas| {
            observadas.borrow_mut().recolher(mem, momento, novas);
        }));
        abertura.mem.escrever(0xFF_F644, Tamanho::Word, 1).unwrap();
        abertura.executar_quadro(&[EventoQuadro::Interrupcao { nivel: 4, pc: 0 }]);
        assert!(abertura.erro.is_none());
        assert!(
            capturas
                .borrow()
                .saidas
                .iter()
                .any(|saida| saida.2 == MomentoSaida::HBlank)
        );
    }

    #[test]
    fn identifica_hblank_pela_linha_programada_no_vblank() {
        let mut mem = Memoria::nova(vec![0; 0x100]);
        let mut capturas = CapturasEventos::nova();
        mem.escrever(0xC0_0004, Tamanho::Word, 0x8A37).unwrap();
        let saidas = mem.tirar_saidas();
        capturas.recolher(&mut mem, MomentoSaida::VBlank, &saidas);
        mem.escrever(0xC0_0004, Tamanho::Word, 0x8ADF).unwrap();
        let saidas = mem.tirar_saidas();
        capturas.recolher(&mut mem, MomentoSaida::HBlank, &saidas);
        assert_eq!(capturas.saidas[1].2, MomentoSaida::HBlank);
        assert_eq!(capturas.saidas[1].3, Some(0x37));
    }

    #[test]
    fn captura_dma_antes_de_alterar_a_origem() {
        let mut mem = Memoria::nova(vec![0; 0x100]);
        mem.ram_mut()[0xFB02..0xFB04].copy_from_slice(&0x0EEEu16.to_be_bytes());
        for comando in [
            0x8174, 0x8F02, 0x9340, 0x9400, 0x96FD, 0x9580, 0x977F, 0xC000, 0x0080,
        ] {
            mem.escrever(0xC0_0004, Tamanho::Word, comando).unwrap();
        }
        let mut capturas = CapturasEventos::nova();
        let saidas = mem.tirar_saidas();
        capturas.recolher(&mut mem, MomentoSaida::VBlank, &saidas);
        mem.ram_mut()[0xFB02..0xFB04].copy_from_slice(&0x000Eu16.to_be_bytes());
        let mut ponte = Ponte::nova();
        for (_, captura, _, _) in &capturas.saidas {
            if let Some(c) = captura {
                ponte.aplicar_capturada_interna(c, &capturas.palavras_dma, 224, 0);
            }
        }
        ponte.fechar_quadro();
        assert_eq!(ponte.renderizador().vdp().cram()[1], 0x0EEE);
    }
}
