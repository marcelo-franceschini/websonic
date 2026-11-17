//! Sintetizador fiel unificado (Etapa 8).
//!
//! Reune o FM (YM2612) e o PSG (YM7101), agendando escritas e amostras no relogio mestre
//! NTSC (mclk = 53 693 175 Hz). Sem ponto flutuante, estritamente em inteiros.
//!
//! Regras temporais (docs/plano-etapa-8.md D8.5 e D8.6):
//! - FM: 1 ciclo interno = 42 mclk; 1 amostra = 24 ciclos internos (1008 mclk).
//! - PSG: 1 ciclo = 15 mclk; 1 amostra = 16 ciclos (240 mclk).
//! - Escrita com mclk `t` e aplicada no ciclo `t / 42` (FM) ou `t / 15` (PSG).

use crate::fm::Fm;
use crate::psg::Psg;

/// Ciclos do relogio mestre NTSC por ciclo interno do FM (53 693 175 / 7 * 6 = 42 mclk).
pub const MCLK_POR_CICLO_FM: u64 = 42;

/// Ciclos internos do FM por amostra completa.
pub const CICLOS_FM_POR_AMOSTRA: u32 = 24;

/// Ciclos do relogio mestre NTSC por amostra do FM (24 * 42 = 1008 mclk, cerca de 53 267 Hz).
pub const MCLK_POR_AMOSTRA_FM: u64 = 1008;

/// Ciclos do relogio mestre NTSC por ciclo do PSG (mestre/15).
pub const MCLK_POR_CICLO_PSG: u64 = 15;

/// Ciclos do PSG por amostra completa.
pub const CICLOS_PSG_POR_AMOSTRA: u32 = 16;

/// Ciclos do relogio mestre NTSC por amostra do PSG (16 * 15 = 240 mclk, cerca de 223 722 Hz).
pub const MCLK_POR_AMOSTRA_PSG: u64 = 240;

/// Ciclos do relogio mestre NTSC em um quadro nominal de 60 Hz (896 040 mclk).
pub const MCLK_POR_QUADRO_NTSC: u64 = 896_040;

/// Identificacao do chip alvo da escrita.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chip {
    Fm,
    Psg,
}

/// Escrita de barramento agendada com carimbo de tempo em mclk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EscritaSom {
    pub mclk: u64,
    pub chip: Chip,
    pub porta: u32,
    pub valor: u8,
}

/// Amostra de FM gerada a cada 24 ciclos internos (1008 mclk).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmostraFm {
    /// Soma dos 6 canais com panning (9 bits por canal com sinal, sem distorcao de repouso).
    pub misturado: (i16, i16),
    /// Soma dos 6 picos multiplexados nos pinos (com a curva de escada do YM2612).
    pub picos_pino: (i16, i16),
    /// Saida bruta dos pinos nos 24 ciclos internos (esquerda, direita).
    pub ciclos: [(i16, i16); 24],
}

/// Amostra de PSG gerada a cada 16 ciclos do PSG (240 mclk).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct AmostraPsg {
    /// Saida digital bruta de 16 bits (4 bits por canal, canal 0 nos bits altos).
    pub empacotado: u16,
    /// Atenuacao individual de cada um dos 4 canais (0 = maximo, 15 = mudo).
    pub canais: [u8; 4],
}

impl From<u16> for AmostraPsg {
    fn from(empacotado: u16) -> Self {
        let c0 = u8::try_from((empacotado >> 12) & 0x0F).unwrap_or(0);
        let c1 = u8::try_from((empacotado >> 8) & 0x0F).unwrap_or(0);
        let c2 = u8::try_from((empacotado >> 4) & 0x0F).unwrap_or(0);
        let c3 = u8::try_from(empacotado & 0x0F).unwrap_or(0);
        Self {
            empacotado,
            canais: [c0, c1, c2, c3],
        }
    }
}

impl From<AmostraPsg> for u16 {
    fn from(a: AmostraPsg) -> Self {
        a.empacotado
    }
}

/// Sintetizador completo do console.
#[derive(Clone, Debug)]
pub struct Sintetizador {
    fm: Fm,
    psg: Psg,
    mclk: u64,

    ciclos_fm: u64,
    ciclos_psg: u64,
    subciclo_fm: usize,
    subciclo_psg: usize,

    buffer_fm: [(i16, i16); 24],

    fila_fm: Vec<(u64, u32, u8)>,
    fila_psg: Vec<(u64, u8)>,
    pos_fm: usize,
    pos_psg: usize,
}

impl Default for Sintetizador {
    fn default() -> Self {
        Self::novo()
    }
}

impl Sintetizador {
    /// Inicializa o sintetizador com ambos os chips em estado de reset.
    pub fn novo() -> Self {
        Self {
            fm: Fm::novo(),
            psg: Psg::novo(),
            mclk: 0,
            ciclos_fm: 0,
            ciclos_psg: 0,
            subciclo_fm: 0,
            subciclo_psg: 0,
            buffer_fm: [(0, 0); 24],
            fila_fm: Vec::new(),
            fila_psg: Vec::new(),
            pos_fm: 0,
            pos_psg: 0,
        }
    }

    /// Reseta o sintetizador para o estado inicial no tempo mclk 0.
    pub fn reset(&mut self) {
        self.fm = Fm::novo();
        self.psg = Psg::novo();
        self.mclk = 0;
        self.ciclos_fm = 0;
        self.ciclos_psg = 0;
        self.subciclo_fm = 0;
        self.subciclo_psg = 0;
        self.buffer_fm = [(0, 0); 24];
        self.fila_fm.clear();
        self.fila_psg.clear();
        self.pos_fm = 0;
        self.pos_psg = 0;
    }

    /// Tempo atual em mclk avancado pelo sintetizador.
    pub fn mclk(&self) -> u64 {
        self.mclk
    }

    /// Referencia de leitura ao chip FM.
    pub fn fm(&self) -> &Fm {
        &self.fm
    }

    /// Referencia mutavel ao chip FM.
    pub fn fm_mut(&mut self) -> &mut Fm {
        &mut self.fm
    }

    /// Referencia de leitura ao chip PSG.
    pub fn psg(&self) -> &Psg {
        &self.psg
    }

    /// Referencia mutavel ao chip PSG.
    pub fn psg_mut(&mut self) -> &mut Psg {
        &mut self.psg
    }

    /// Agenda uma escrita para um instante em mclk.
    pub fn agendar(&mut self, mclk: u64, chip: Chip, porta: u32, valor: u8) {
        match chip {
            Chip::Fm => {
                let idx = self.fila_fm.partition_point(|&(m, _, _)| m <= mclk);
                self.fila_fm.insert(idx, (mclk, porta, valor));
            }
            Chip::Psg => {
                let idx = self.fila_psg.partition_point(|&(m, _)| m <= mclk);
                self.fila_psg.insert(idx, (mclk, valor));
            }
        }
    }

    /// Agenda uma escrita (apelido conveniente para `agendar`).
    pub fn escreve(&mut self, mclk: u64, chip: Chip, porta: u32, valor: u8) {
        self.agendar(mclk, chip, porta, valor);
    }

    /// Agenda um lote de escritas.
    pub fn agendar_lote(&mut self, escritas: &[EscritaSom]) {
        for e in escritas {
            self.agendar(e.mclk, e.chip, e.porta, e.valor);
        }
    }

    /// Avanca ambos os chips ate `mclk_alvo`, entregando as amostras geradas.
    pub fn avancar_ate(
        &mut self,
        mclk_alvo: u64,
        saida_fm: &mut Vec<AmostraFm>,
        saida_psg: &mut Vec<AmostraPsg>,
    ) {
        if mclk_alvo <= self.mclk {
            return;
        }

        // Avanco do FM.
        let ciclos_alvo_fm = mclk_alvo / MCLK_POR_CICLO_FM;
        let novos_fm =
            usize::try_from(ciclos_alvo_fm.saturating_sub(self.ciclos_fm) / 24).unwrap_or(0);
        saida_fm.reserve(novos_fm);
        while self.ciclos_fm < ciclos_alvo_fm {
            while self.pos_fm < self.fila_fm.len() {
                let (m, p, v) = self.fila_fm[self.pos_fm];
                let c_escrita = m / MCLK_POR_CICLO_FM;
                if c_escrita <= self.ciclos_fm {
                    self.fm.escreve(p, v);
                    self.pos_fm = self.pos_fm.wrapping_add(1);
                } else {
                    break;
                }
            }

            let proxima_escrita = if self.pos_fm < self.fila_fm.len() {
                self.fila_fm[self.pos_fm].0 / MCLK_POR_CICLO_FM
            } else {
                ciclos_alvo_fm
            };
            let limite = ciclos_alvo_fm.min(proxima_escrita);

            while self.ciclos_fm < limite {
                let lr = self.fm.ciclo();
                self.buffer_fm[self.subciclo_fm] = lr;
                self.subciclo_fm = self.subciclo_fm.wrapping_add(1);
                self.ciclos_fm = self.ciclos_fm.wrapping_add(1);
                if self.subciclo_fm >= 24 {
                    self.subciclo_fm = 0;
                    let misturado = self.fm.saida_misturada();
                    let mut pl = 0i16;
                    let mut pr = 0i16;
                    for &idx in &[3, 7, 11, 15, 19, 23] {
                        pl = pl.wrapping_add(self.buffer_fm[idx].0);
                        pr = pr.wrapping_add(self.buffer_fm[idx].1);
                    }
                    saida_fm.push(AmostraFm {
                        misturado,
                        picos_pino: (pl, pr),
                        ciclos: self.buffer_fm,
                    });
                }
            }
        }

        // Avanco do PSG.
        let ciclos_alvo_psg = mclk_alvo / MCLK_POR_CICLO_PSG;
        let novos_psg =
            usize::try_from(ciclos_alvo_psg.saturating_sub(self.ciclos_psg) / 16).unwrap_or(0);
        saida_psg.reserve(novos_psg);
        while self.ciclos_psg < ciclos_alvo_psg {
            while self.pos_psg < self.fila_psg.len() {
                let (m, v) = self.fila_psg[self.pos_psg];
                let c_escrita = m / MCLK_POR_CICLO_PSG;
                if c_escrita <= self.ciclos_psg {
                    self.psg.escreve(v);
                    self.pos_psg = self.pos_psg.wrapping_add(1);
                } else {
                    break;
                }
            }

            let proxima_escrita = if self.pos_psg < self.fila_psg.len() {
                self.fila_psg[self.pos_psg].0 / MCLK_POR_CICLO_PSG
            } else {
                ciclos_alvo_psg
            };
            let limite = ciclos_alvo_psg.min(proxima_escrita);

            while self.ciclos_psg < limite {
                self.psg.ciclo();
                self.subciclo_psg = self.subciclo_psg.wrapping_add(1);
                self.ciclos_psg = self.ciclos_psg.wrapping_add(1);
                if self.subciclo_psg >= 16 {
                    self.subciclo_psg = 0;
                    let emp = self.psg.saida_empacotada();
                    saida_psg.push(AmostraPsg::from(emp));
                }
            }
        }

        if self.pos_fm > 0 {
            self.fila_fm.drain(..self.pos_fm);
            self.pos_fm = 0;
        }
        if self.pos_psg > 0 {
            self.fila_psg.drain(..self.pos_psg);
            self.pos_psg = 0;
        }

        self.mclk = mclk_alvo;
    }

    /// Avanca ambos os chips ate `mclk_alvo`, devolvendo o PSG como `u16` bruto.
    pub fn avancar_ate_u16(
        &mut self,
        mclk_alvo: u64,
        saida_fm: &mut Vec<AmostraFm>,
        saida_psg: &mut Vec<u16>,
    ) {
        let mut psg_aux = Vec::new();
        self.avancar_ate(mclk_alvo, saida_fm, &mut psg_aux);
        saida_psg.extend(psg_aux.into_iter().map(u16::from));
    }
}
