//! Parte analogica da saida de audio do Mega Drive Modelo 1.
//!
//! APROXIMADO: esta secao e uma aproximacao da cadeia analogica pos-DAC (filtro
//! passa-baixas de primeira ordem e ganho do amplificador). Nao entra no veredito
//! amostra a amostra dos chips digitais.

use std::f32::consts::PI;

/// Filtro analogico de saida de audio (marcado explicitamente como APROXIMADO).
#[derive(Clone, Debug)]
pub struct FiltroAnalogico {
    frequencia_corte: f32,
    alpha: f32,
    estado_l: f32,
    estado_r: f32,
}

impl FiltroAnalogico {
    /// Cria o filtro passa-baixas com corte tipico do Mega Drive Modelo 1 (~3386 Hz).
    /// Marcado como APROXIMADO (nao verificado contra silicio).
    pub fn novo(taxa_amostragem: f32) -> Self {
        let frequencia_corte = 3386.0;
        let rc = 1.0 / (2.0 * PI * frequencia_corte);
        let dt = 1.0 / taxa_amostragem.max(1.0);
        let alpha = dt / (rc + dt);
        Self {
            frequencia_corte,
            alpha,
            estado_l: 0.0,
            estado_r: 0.0,
        }
    }

    /// Processa uma amostra estereo (esquerda, direita) em f32.
    #[inline]
    pub fn processar(&mut self, l: f32, r: f32) -> (f32, f32) {
        self.estado_l += self.alpha * (l - self.estado_l);
        self.estado_r += self.alpha * (r - self.estado_r);
        (self.estado_l, self.estado_r)
    }

    /// Processa um bloco estereo intercalado `[L0, R0, L1, R1, ...]` in-place.
    #[inline]
    pub fn processar_bloco_intercalado(&mut self, intercalado: &mut [f32]) {
        let alpha = self.alpha;
        let mut el = self.estado_l;
        let mut er = self.estado_r;
        for par in intercalado.chunks_exact_mut(2) {
            el += alpha * (par[0] - el);
            er += alpha * (par[1] - er);
            par[0] = el;
            par[1] = er;
        }
        self.estado_l = el;
        self.estado_r = er;
    }

    /// Reseta os estados internos do filtro.
    pub fn reset(&mut self) {
        self.estado_l = 0.0;
        self.estado_r = 0.0;
    }

    /// Mede a resposta em frequencia em dB para senoides sinteticas nas 8 frequencias tipicas.
    pub fn medir_resposta_db(&self) -> Vec<(u32, f32)> {
        let freqs = [100, 250, 500, 1000, 2000, 3386, 5000, 10000];
        let rc = 1.0 / (2.0 * PI * self.frequencia_corte);
        let mut resp = Vec::with_capacity(freqs.len());

        for &f in &freqs {
            let f_f32 = f as f32;
            let omega = 2.0 * PI * f_f32;
            // Resposta em amplitude de 1a ordem H(jw) = 1 / sqrt(1 + (w*RC)^2)
            let mag = 1.0 / (1.0 + (omega * rc).powi(2)).sqrt();
            let db = 20.0 * mag.log10();
            resp.push((f, db));
        }

        resp
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn bloco_intercalado_equivale_a_processar_amostra_a_amostra() {
        let mut f1 = FiltroAnalogico::novo(48_000.0);
        let mut f2 = FiltroAnalogico::novo(48_000.0);
        let entrada: Vec<f32> = (0..64)
            .map(|i| ((i as f32) * 0.23).sin() * 0.75)
            .collect();
        let mut bloco = entrada.clone();
        f1.processar_bloco_intercalado(&mut bloco);
        for (i, par) in entrada.chunks_exact(2).enumerate() {
            let (l, r) = f2.processar(par[0], par[1]);
            assert!((bloco[i * 2] - l).abs() < 1e-6);
            assert!((bloco[i * 2 + 1] - r).abs() < 1e-6);
        }
    }

    #[test]
    fn dc_estabiliza_em_ganho_unitario_e_reset_zera_estado() {
        let mut f = FiltroAnalogico::novo(48_000.0);
        let mut ultimo = (0.0, 0.0);
        for _ in 0..2000 {
            ultimo = f.processar(1.0, -0.5);
        }
        assert!((ultimo.0 - 1.0).abs() < 1e-4);
        assert!((ultimo.1 + 0.5).abs() < 1e-4);
        f.reset();
        let apos_reset = f.processar(0.0, 0.0);
        assert_eq!(apos_reset, (0.0, 0.0));
    }
}
