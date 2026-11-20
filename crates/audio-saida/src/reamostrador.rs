//! Reamostrador polifasico em fase com sinc janelada (Etapa 8, D8.4 e D8.6).
//!
//! Converte as taxas nativas (FM ~53 267 Hz, PSG ~223 722 Hz) para a taxa do navegador
//! (48 000 Hz ou 44 100 Hz) sem perda de fase, usando 256 fases e 32 coeficientes (taps).
//! Acompanha o relogio nominal de 60 Hz com um controlador proporcional de baixa banda (±0.5%).

use std::f32::consts::PI;

pub const FASES_POLIFASICO: usize = 256;
pub const TAPS_POLIFASICO: usize = 32;

/// Tabela de coeficientes do filtro polifasico [256 fases][32 coeficientes].
#[derive(Clone, Debug)]
pub struct TabelaFiltro {
    coefs: [[f32; TAPS_POLIFASICO]; FASES_POLIFASICO],
}

impl TabelaFiltro {
    /// Gera a tabela de sinc janelada por Blackman-Harris normalizada.
    pub fn gerar() -> Self {
        Self::gerar_com_corte(1.0)
    }

    /// O corte é relativo ao Nyquist da entrada. Reduzi-lo evita aliasing ao reduzir a taxa.
    fn gerar_com_corte(corte: f32) -> Self {
        let mut coefs = [[0.0f32; TAPS_POLIFASICO]; FASES_POLIFASICO];
        let metade_taps = (TAPS_POLIFASICO as f32) / 2.0;

        for (p, linha) in coefs.iter_mut().enumerate() {
            let delta = (p as f32) / (FASES_POLIFASICO as f32);
            let mut soma = 0.0f32;

            for (k, c) in linha.iter_mut().enumerate() {
                let x = (k as f32) - metade_taps + delta;
                let sinc = if x.abs() < 1e-6 {
                    corte
                } else {
                    let px = PI * x;
                    (px * corte).sin() / px
                };

                // Janela de Blackman-Harris de 4 termos
                let pos_w = ((k as f32) + delta) / (TAPS_POLIFASICO as f32);
                let w = if (0.0..=1.0).contains(&pos_w) {
                    let a0 = 0.35875;
                    let a1 = 0.48829;
                    let a2 = 0.14128;
                    let a3 = 0.01168;
                    a0 - a1 * (2.0 * PI * pos_w).cos() + a2 * (4.0 * PI * pos_w).cos()
                        - a3 * (6.0 * PI * pos_w).cos()
                } else {
                    0.0
                };

                let val = sinc * w;
                *c = val;
                soma += val;
            }

            // Normalizacao para ganho unitario em DC
            if soma.abs() > 1e-6 {
                for c in linha.iter_mut() {
                    *c /= soma;
                }
            }
        }

        Self { coefs }
    }
}

/// Reamostrador polifasico para um canal mono ou estereo.
#[derive(Clone, Debug)]
pub struct Reamostrador {
    tabela: TabelaFiltro,
    passo_base: f64,
    ajuste_fino: f64,
    pos_frac: f64,
    offset_base: usize,
    buffer_in_l: Vec<f32>,
    buffer_in_r: Vec<f32>,
}

impl Reamostrador {
    pub fn novo(taxa_entrada: f64, taxa_saida: f64) -> Self {
        let corte = (taxa_saida / taxa_entrada).min(1.0) as f32;
        let tabela = TabelaFiltro::gerar_com_corte(corte);
        let passo_base = taxa_entrada / taxa_saida;
        let mut buffer_in_l = Vec::with_capacity(8192);
        let mut buffer_in_r = Vec::with_capacity(8192);
        buffer_in_l.resize(TAPS_POLIFASICO, 0.0);
        buffer_in_r.resize(TAPS_POLIFASICO, 0.0);

        Self {
            tabela,
            passo_base,
            ajuste_fino: 0.0,
            pos_frac: 0.0,
            offset_base: 0,
            buffer_in_l,
            buffer_in_r,
        }
    }

    /// Cria um reamostrador apenas mono (nao aloca o buffer do canal direito).
    pub fn novo_mono(taxa_entrada: f64, taxa_saida: f64) -> Self {
        let corte = (taxa_saida / taxa_entrada).min(1.0) as f32;
        let tabela = TabelaFiltro::gerar_com_corte(corte);
        let passo_base = taxa_entrada / taxa_saida;
        let mut buffer_in_l = Vec::with_capacity(16_384);
        buffer_in_l.resize(TAPS_POLIFASICO, 0.0);

        Self {
            tabela,
            passo_base,
            ajuste_fino: 0.0,
            pos_frac: 0.0,
            offset_base: 0,
            buffer_in_l,
            buffer_in_r: Vec::new(),
        }
    }

    /// Ajuste fino da razao de reamostragem (limitado a ±0.5%).
    pub fn definir_ajuste_fino(&mut self, ajuste: f64) {
        self.ajuste_fino = ajuste.clamp(-0.005, 0.005);
    }

    /// Adiciona amostras de entrada estereo ao buffer de convolucao.
    pub fn alimentar_estereo(&mut self, amostras: &[(f32, f32)]) {
        self.buffer_in_l.reserve(amostras.len());
        self.buffer_in_r.reserve(amostras.len());
        for &(l, r) in amostras {
            self.buffer_in_l.push(l);
            self.buffer_in_r.push(r);
        }
    }

    /// Adiciona amostras de entrada mono ao buffer de convolucao (grava apenas em L).
    pub fn alimentar_mono(&mut self, amostras: &[f32]) {
        self.buffer_in_l.extend_from_slice(amostras);
    }

    /// Converte as amostras pendentes gerando saida estereo interpolada.
    pub fn processar(&mut self, saida: &mut Vec<(f32, f32)>) {
        let n_in = self.buffer_in_l.len();
        if n_in < TAPS_POLIFASICO {
            return;
        }

        let limite = n_in - TAPS_POLIFASICO;
        let passo = self.passo_base * (1.0 + self.ajuste_fino);
        let mono = self.buffer_in_r.len() < self.buffer_in_l.len();

        while (self.offset_base as f64) + self.pos_frac < (limite as f64) {
            let idx_base = self.offset_base;
            let fase =
                ((self.pos_frac * (FASES_POLIFASICO as f64)) as usize).min(FASES_POLIFASICO - 1);
            let coefs = &self.tabela.coefs[fase];

            let slice_l: &[f32; TAPS_POLIFASICO] = self.buffer_in_l
                [idx_base..idx_base + TAPS_POLIFASICO]
                .try_into()
                .expect("a fatia tem exatamente TAPS_POLIFASICO amostras");
            let slice_r: &[f32; TAPS_POLIFASICO] = if mono {
                slice_l
            } else {
                self.buffer_in_r[idx_base..idx_base + TAPS_POLIFASICO]
                    .try_into()
                    .expect("a fatia tem exatamente TAPS_POLIFASICO amostras")
            };

            let mut out_l = 0.0f32;
            let mut out_r = 0.0f32;

            for k in 0..TAPS_POLIFASICO {
                out_l += slice_l[k] * coefs[k];
                out_r += slice_r[k] * coefs[k];
            }

            saida.push((out_l, out_r));
            self.pos_frac += passo;
            let inteiros = self.pos_frac as usize;
            if inteiros > 0 {
                self.offset_base += inteiros;
                self.pos_frac -= inteiros as f64;
            }
        }

        // Compacta o buffer apenas quando o offset_base acumula mais de 2048 amostras
        if self.offset_base > 2048 {
            self.buffer_in_l.drain(..self.offset_base);
            if self.buffer_in_r.len() >= self.offset_base {
                self.buffer_in_r.drain(..self.offset_base);
            }
            self.offset_base = 0;
        }
    }

    /// Converte as amostras pendentes gerando saida mono interpolada.
    pub fn processar_mono(&mut self, saida: &mut Vec<f32>) {
        let n_in = self.buffer_in_l.len();
        if n_in < TAPS_POLIFASICO {
            return;
        }

        let limite = n_in - TAPS_POLIFASICO;
        let passo = self.passo_base * (1.0 + self.ajuste_fino);

        while (self.offset_base as f64) + self.pos_frac < (limite as f64) {
            let idx_base = self.offset_base;
            let fase =
                ((self.pos_frac * (FASES_POLIFASICO as f64)) as usize).min(FASES_POLIFASICO - 1);
            let coefs = &self.tabela.coefs[fase];

            let slice_l: &[f32; TAPS_POLIFASICO] = self.buffer_in_l
                [idx_base..idx_base + TAPS_POLIFASICO]
                .try_into()
                .expect("a fatia tem exatamente TAPS_POLIFASICO amostras");

            let mut out = 0.0f32;

            for k in 0..TAPS_POLIFASICO {
                out += slice_l[k] * coefs[k];
            }

            saida.push(out);
            self.pos_frac += passo;
            let inteiros = self.pos_frac as usize;
            if inteiros > 0 {
                self.offset_base += inteiros;
                self.pos_frac -= inteiros as f64;
            }
        }

        // PSG gera ~3733 amostras por quadro: compactar acima de 8192 evita drenar todo quadro.
        if self.offset_base > 8192 {
            self.buffer_in_l.drain(..self.offset_base);
            self.offset_base = 0;
        }
    }

    /// Reseta o historico e a fase.
    pub fn reset(&mut self) {
        self.pos_frac = 0.0;
        self.offset_base = 0;
        self.ajuste_fino = 0.0;
        self.buffer_in_l.clear();
        self.buffer_in_l.resize(TAPS_POLIFASICO, 0.0);
        if !self.buffer_in_r.is_empty() {
            self.buffer_in_r.clear();
            self.buffer_in_r.resize(TAPS_POLIFASICO, 0.0);
        }
    }
}
