//! Ponte entre o núcleo e o renderizador (Etapa 7).
//!
//! O núcleo manda escritas na VDP. O renderizador precisa da linha e da coluna de cada escrita
//! e, nos DMAs da memória para a VDP, das palavras que o DMA leu. Esta ponte junta as duas coisas.
//! No navegador a posição é nominal (sem tempo gravado, D7.5). No comparador a posição vem do
//! oráculo (D7.3).

use sonic1_audio::{AmostraFm, AmostraPsg, Chip, MCLK_POR_QUADRO_NTSC, Sintetizador};
use sonic1_audio_saida::MisturadorSaida;
use sonic1_core::abertura::{Abertura, Espera, EventoQuadro, Marca};
use sonic1_core::memoria::{Memoria, PortaVdp, Saida, Tamanho};
use sonic1_video::quadro::{ALTURA, LARGURA};
use sonic1_video::renderizador::{Escrita, Renderizador};

/// Linha do começo do VBlank (primeira linha fora da imagem).
const LINHA_VBLANK: u16 = 224;

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
        // Arte dos cartões e limpeza da RAM: estimado pela rodada de 970 quadros (as
        // interrupções voltam uns 7 quadros depois do fim do desvanecimento).
        Marca::LevelTtlCardLoop => 7,
        // Estimativa até a rodada com a ROM medir os dois trechos da carga final.
        Marca::LevelDelay => 2,
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

    /// Lê as palavras de um DMA da memória para a VDP, do endereço dado pelos registradores.
    fn palavras_do_dma(&self, mem: &mut Memoria) -> Vec<u16> {
        let reg = |i: usize| self.regs.get(i).copied().unwrap_or(0);
        let base =
            u32::from(reg(21)) | (u32::from(reg(22)) << 8) | (u32::from(reg(23) & 0x7F) << 16);
        let mut a = base.wrapping_mul(2);
        let n = match u32::from(reg(19)) | (u32::from(reg(20)) << 8) {
            0 => 0x1_0000,
            n => n,
        };
        let mut v = Vec::new();
        for _ in 0..n {
            let w = mem.ler(a, Tamanho::Word).unwrap_or(0);
            v.push(u16::try_from(w & 0xFFFF).unwrap_or(0));
            a = a.wrapping_add(2);
        }
        v
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
        let Saida::Vdp {
            porta,
            valor,
            tamanho,
        } = *saida
        else {
            return None;
        };
        let mut dma = None;
        if porta == PortaVdp::Controle {
            // Registrador (010x xxxx...): palavra única. Senão é metade de um comando.
            let e_registrador = valor & 0xC000 == 0x8000;
            if e_registrador {
                self.primeira = None;
                let n = usize::from((valor >> 8) & 0x1F);
                if let Some(r) = self.regs.get_mut(n) {
                    *r = valor.to_le_bytes()[0];
                }
            } else if let Some(_primeira) = self.primeira.take() {
                // Segunda palavra: o bit 7 pede DMA. Se for da memória, lê as palavras.
                if valor & 0x0080 != 0 && self.dma_de_memoria() {
                    dma = Some(self.palavras_do_dma(mem));
                }
            } else {
                self.primeira = Some(valor);
            }
        }
        Some(Capturada {
            porta,
            valor,
            tamanho,
            dma,
        })
    }

    /// Segunda fase: entrega o DMA lido e aplica a escrita na posição dada.
    pub fn aplicar_capturada(&mut self, c: &Capturada, linha: u16, coluna: u64) {
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
                if let Some(palavras) = &c.dma {
                    self.rend.entregar_dma(palavras);
                }
                self.rend.aplicar(linha, coluna, Escrita::Controle(c.valor));
            }
        }
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

/// O console completo no navegador: núcleo com tempo nominal mais a ponte e o sintetizador de som.
pub struct Console {
    pub ab: Abertura,
    ponte: Ponte,
    espera_atual: Option<Espera>,
    quadros_na_espera: u32,
    escritas_no_quadro: usize,
    sint: Sintetizador,
    misturador: MisturadorSaida,
    quadro_num: u64,
    buffer_audio_quadro: Vec<f32>,
    amostras_fm: Vec<AmostraFm>,
    amostras_psg: Vec<AmostraPsg>,
}

impl Console {
    pub fn novo(rom: Vec<u8>) -> Self {
        Self {
            ab: Abertura::nova_nominal(rom),
            ponte: Ponte::nova(),
            espera_atual: None,
            quadros_na_espera: 0,
            escritas_no_quadro: 0,
            sint: Sintetizador::novo(),
            misturador: MisturadorSaida::novo(48_000.0),
            quadro_num: 0,
            buffer_audio_quadro: Vec::new(),
            amostras_fm: Vec::new(),
            amostras_psg: Vec::new(),
        }
    }

    /// Eventos nominais do quadro, pelo que o laço principal espera.
    fn eventos_nominais(&mut self) -> Vec<EventoQuadro> {
        let espera = self.ab.espera();
        if self.espera_atual != Some(espera) {
            self.espera_atual = Some(espera);
            self.quadros_na_espera = 0;
        }
        let ja = self.quadros_na_espera;
        self.quadros_na_espera = self.quadros_na_espera.saturating_add(1);
        match espera {
            Espera::Marca(m) if ja >= quadros_de_espera(m) => vec![EventoQuadro::Marca(m)],
            // No tempo nominal não há lag: o laço chega à espera antes de cada VBlank (D7.25).
            Espera::VBlank => vec![
                EventoQuadro::Marca(Marca::WaitForVBlank),
                EventoQuadro::Interrupcao { nivel: 6, pc: 0 },
            ],
            Espera::VozAcabando if ja >= QUADROS_DA_VOZ => {
                vec![EventoQuadro::Interrupcao { nivel: 6, pc: 0 }]
            }
            _ => Vec::new(),
        }
    }

    /// Botões apertados (bits: cima, baixo, esquerda, direita, B, C, A, Start).
    pub fn definir_botoes(&mut self, mascara: u8) {
        self.ab.mem.definir_botoes(mascara);
    }

    /// Roda um quadro e devolve a imagem em RGBA (320 por 224).
    pub fn executar_quadro(&mut self) -> Vec<u8> {
        let eventos = self.eventos_nominais();
        let teve_vblank = eventos
            .iter()
            .any(|e| matches!(e, EventoQuadro::Interrupcao { .. }));
        self.ab.executar_quadro(&eventos);

        self.quadro_num = self.quadro_num.wrapping_add(1);
        let mclk_inicio = self
            .quadro_num
            .wrapping_sub(1)
            .wrapping_mul(MCLK_POR_QUADRO_NTSC);
        let mclk_fim = self.quadro_num.wrapping_mul(MCLK_POR_QUADRO_NTSC);

        // Posição nominal: o tratador de VBlank escreve na linha 224; o resto, na linha 0.
        let linha = if teve_vblank { LINHA_VBLANK } else { 0 };
        let saidas = self.ab.mem.tirar_saidas();
        self.escritas_no_quadro = saidas.len();

        let n_saidas = saidas.len();
        for (i, s) in saidas.iter().enumerate() {
            let delta = if n_saidas > 0 {
                (MCLK_POR_QUADRO_NTSC.wrapping_mul(i as u64)) / (n_saidas as u64)
            } else {
                0
            };
            let mclk_esc = mclk_inicio.wrapping_add(delta);

            match s {
                Saida::Vdp { .. } => {
                    self.ponte.aplicar(&mut self.ab.mem, s, linha, 0);
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

        // Sintetiza o áudio gerado neste quadro e alimenta o misturador
        self.amostras_fm.clear();
        self.amostras_psg.clear();
        self.sint
            .avancar_ate(mclk_fim, &mut self.amostras_fm, &mut self.amostras_psg);
        self.misturador
            .alimentar_quadro(&self.amostras_fm, &self.amostras_psg);

        // Extrai as amostras reamostradas para reprodução no navegador
        self.buffer_audio_quadro.clear();
        let disp = self.misturador.ocupacao_buffer();
        if disp > 0 {
            let mut bloco = vec![0.0f32; disp.wrapping_mul(2)];
            self.misturador.extrair_bloco_worklet(&mut bloco);
            self.buffer_audio_quadro = bloco;
        }

        self.ponte.fechar_quadro()
    }

    /// Devolve as amostras de áudio estereo geradas no último quadro (intercaladas: L, R, L, R...).
    pub fn extrair_audio(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.buffer_audio_quadro)
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

    pub fn erro(&self) -> Option<String> {
        self.ab.erro.map(|e| format!("{e:?}"))
    }
}
