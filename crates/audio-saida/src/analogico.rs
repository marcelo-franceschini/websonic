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
        let dt = 1.0 / taxa_amostragem;
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
