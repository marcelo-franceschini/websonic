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

                // Janela de Blackman-Harris de 4 termos via recorrencia de Chebyshev
                // (cos(2t) = 2c^2 - 1, cos(3t) = c(4c^2 - 3)), economizando 2 cos() por tap.
                let pos_w = ((k as f32) + delta) / (TAPS_POLIFASICO as f32);
                let w = if (0.0..=1.0).contains(&pos_w) {
                    let a0 = 0.35875;
                    let a1 = 0.48829;
                    let a2 = 0.14128;
                    let a3 = 0.01168;
                    let c1 = (2.0 * PI * pos_w).cos();
                    let c2 = 2.0 * c1 * c1 - 1.0;
                    let c3 = c1 * (4.0 * c1 * c1 - 3.0);
                    a0 - a1 * c1 + a2 * c2 - a3 * c3
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
        self.alimentar_estereo_iter(amostras.iter().copied());
    }

    /// Adiciona amostras de entrada estereo diretamente de um iterador (sem buffer intermediario).
    pub fn alimentar_estereo_iter(&mut self, amostras: impl ExactSizeIterator<Item = (f32, f32)>) {
        let n = amostras.len();
        self.buffer_in_l.reserve(n);
        self.buffer_in_r.reserve(n);
        for (l, r) in amostras {
            self.buffer_in_l.push(l);
            self.buffer_in_r.push(r);
        }
    }

    /// Adiciona amostras de entrada mono ao buffer de convolucao (grava apenas em L).
    pub fn alimentar_mono(&mut self, amostras: &[f32]) {
        self.buffer_in_l.extend_from_slice(amostras);
    }

    /// Adiciona amostras de entrada mono diretamente de um iterador (sem buffer intermediario).
    pub fn alimentar_mono_iter(&mut self, amostras: impl ExactSizeIterator<Item = f32>) {
        self.buffer_in_l.reserve(amostras.len());
        self.buffer_in_l.extend(amostras);
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
        let mut offset = self.offset_base;
        let mut pos_frac = self.pos_frac;

        if offset < limite && passo > 0.0 {
            let estimativa = ((limite - offset) as f64 / passo) as usize + 2;
            saida.reserve(estimativa);
        }

        // Como `pos_frac` permanece estritamente em [0.0, 1.0), `offset + pos_frac < limite`
        // equivale exatamente a `offset < limite`, evitando conversoes usize -> f64 por amostra.
        let coefs_tabela = &self.tabela.coefs;
        let buf_l = &self.buffer_in_l;
        if mono {
            while offset < limite {
                let fase =
                    ((pos_frac * (FASES_POLIFASICO as f64)) as usize).min(FASES_POLIFASICO - 1);
                let coefs = &coefs_tabela[fase];

                let slice_l: &[f32; TAPS_POLIFASICO] = buf_l[offset..offset + TAPS_POLIFASICO]
                    .try_into()
                    .expect("a fatia tem exatamente TAPS_POLIFASICO amostras");
                let v = convolucao_mono_4vias(slice_l, coefs);
                saida.push((v, v));

                pos_frac += passo;
                let inteiros = pos_frac as usize;
                if inteiros > 0 {
                    offset += inteiros;
                    pos_frac -= inteiros as f64;
                }
            }
        } else {
            let buf_r = &self.buffer_in_r;
            while offset < limite {
                let fase =
                    ((pos_frac * (FASES_POLIFASICO as f64)) as usize).min(FASES_POLIFASICO - 1);
                let coefs = &coefs_tabela[fase];

                let slice_l: &[f32; TAPS_POLIFASICO] = buf_l[offset..offset + TAPS_POLIFASICO]
                    .try_into()
                    .expect("a fatia tem exatamente TAPS_POLIFASICO amostras");
                let slice_r: &[f32; TAPS_POLIFASICO] = buf_r[offset..offset + TAPS_POLIFASICO]
                    .try_into()
                    .expect("a fatia tem exatamente TAPS_POLIFASICO amostras");
                saida.push(convolucao_estereo_4vias(slice_l, slice_r, coefs));

                pos_frac += passo;
                let inteiros = pos_frac as usize;
                if inteiros > 0 {
                    offset += inteiros;
                    pos_frac -= inteiros as f64;
                }
            }
        }
        self.offset_base = offset;
        self.pos_frac = pos_frac;

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
        let mut offset = self.offset_base;
        let mut pos_frac = self.pos_frac;

        if offset < limite && passo > 0.0 {
            let estimativa = ((limite - offset) as f64 / passo) as usize + 2;
            saida.reserve(estimativa);
        }

        let coefs_tabela = &self.tabela.coefs;
        let buf_l = &self.buffer_in_l;
        while offset < limite {
            let fase =
                ((pos_frac * (FASES_POLIFASICO as f64)) as usize).min(FASES_POLIFASICO - 1);
            let coefs = &coefs_tabela[fase];

            let slice_l: &[f32; TAPS_POLIFASICO] = buf_l[offset..offset + TAPS_POLIFASICO]
                .try_into()
                .expect("a fatia tem exatamente TAPS_POLIFASICO amostras");

            saida.push(convolucao_mono_4vias(slice_l, coefs));
            pos_frac += passo;
            let inteiros = pos_frac as usize;
            if inteiros > 0 {
                offset += inteiros;
                pos_frac -= inteiros as f64;
            }
        }
        self.offset_base = offset;
        self.pos_frac = pos_frac;

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

/// Produto escalar FIR de 32 taps em 4 acumuladores independentes (quebra dependencia serial
/// de ponto flutuante e habilita autovetorizacao f32x4 em SIMD128/SSE/NEON sem unsafe).
#[inline]
fn convolucao_mono_4vias(
    amostras: &[f32; TAPS_POLIFASICO],
    coefs: &[f32; TAPS_POLIFASICO],
) -> f32 {
    let mut a0 = 0.0f32;
    let mut a1 = 0.0f32;
    let mut a2 = 0.0f32;
    let mut a3 = 0.0f32;

    for (s, c) in amostras.chunks_exact(4).zip(coefs.chunks_exact(4)) {
        a0 += s[0] * c[0];
        a1 += s[1] * c[1];
        a2 += s[2] * c[2];
        a3 += s[3] * c[3];
    }

    (a0 + a1) + (a2 + a3)
}

#[inline]
fn convolucao_estereo_4vias(
    amostras_l: &[f32; TAPS_POLIFASICO],
    amostras_r: &[f32; TAPS_POLIFASICO],
    coefs: &[f32; TAPS_POLIFASICO],
) -> (f32, f32) {
    let mut l0 = 0.0f32;
    let mut l1 = 0.0f32;
    let mut l2 = 0.0f32;
    let mut l3 = 0.0f32;
    let mut r0 = 0.0f32;
    let mut r1 = 0.0f32;
    let mut r2 = 0.0f32;
    let mut r3 = 0.0f32;

    for ((sl, sr), c) in amostras_l
        .chunks_exact(4)
        .zip(amostras_r.chunks_exact(4))
        .zip(coefs.chunks_exact(4))
    {
        let c0 = c[0];
        let c1 = c[1];
        let c2 = c[2];
        let c3 = c[3];
        l0 += sl[0] * c0;
        l1 += sl[1] * c1;
        l2 += sl[2] * c2;
        l3 += sl[3] * c3;
        r0 += sr[0] * c0;
        r1 += sr[1] * c1;
        r2 += sr[2] * c2;
        r3 += sr[3] * c3;
    }

    ((l0 + l1) + (l2 + l3), (r0 + r1) + (r2 + r3))
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn convolucao_4vias_equivale_ao_produto_escalar_referencia() {
        let tabela = TabelaFiltro::gerar();
        let coefs = &tabela.coefs[127];
        let mut l = [0.0f32; TAPS_POLIFASICO];
        let mut r = [0.0f32; TAPS_POLIFASICO];
        for i in 0..TAPS_POLIFASICO {
            l[i] = ((i as f32) * 0.37).sin();
            r[i] = ((i as f32) * 0.21).cos();
        }

        let mut ref_l = 0.0f32;
        let mut ref_r = 0.0f32;
        for k in 0..TAPS_POLIFASICO {
            ref_l += l[k] * coefs[k];
            ref_r += r[k] * coefs[k];
        }

        let mono = convolucao_mono_4vias(&l, coefs);
        let (el, er) = convolucao_estereo_4vias(&l, &r, coefs);
        assert!((mono - ref_l).abs() < 1e-6);
        assert!((el - ref_l).abs() < 1e-6);
        assert!((er - ref_r).abs() < 1e-6);
    }

    #[test]
    fn alimentar_iter_equivale_a_fatia() {
        let mut r_fatia = Reamostrador::novo(53_267.0, 48_000.0);
        let mut r_iter = Reamostrador::novo(53_267.0, 48_000.0);
        let entrada: Vec<(f32, f32)> = (0..512)
            .map(|i| (((i as f32) * 0.13).sin(), ((i as f32) * 0.09).cos()))
            .collect();

        r_fatia.alimentar_estereo(&entrada);
        r_iter.alimentar_estereo_iter(entrada.iter().copied());

        let mut s1 = Vec::new();
        let mut s2 = Vec::new();
        r_fatia.processar(&mut s1);
        r_iter.processar(&mut s2);
        assert_eq!(s1, s2);

        let mut m_fatia = Reamostrador::novo_mono(223_722.0, 48_000.0);
        let mut m_iter = Reamostrador::novo_mono(223_722.0, 48_000.0);
        let mono_in: Vec<f32> = (0..1024).map(|i| ((i as f32) * 0.07).sin()).collect();
        m_fatia.alimentar_mono(&mono_in);
        m_iter.alimentar_mono_iter(mono_in.iter().copied());

        let mut ms1 = Vec::new();
        let mut ms2 = Vec::new();
        m_fatia.processar_mono(&mut ms1);
        m_iter.processar_mono(&mut ms2);
        assert_eq!(ms1, ms2);
    }
}
