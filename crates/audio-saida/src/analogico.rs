//! Parte analogica da saida de audio do Mega Drive Modelo 1.
//!
//! APROXIMADO: esta secao e uma aproximacao da cadeia analogica pos-DAC (filtro
//! passa-baixas de primeira ordem e ganho do amplificador). Nao entra no veredito
//! amostra a amostra dos chips digitais.

use std::f32::consts::PI;

/// Filtro analogico de saida de audio (marcado explicitamente como APROXIMADO).
#[derive(Clone, Debug)]
pub struct FiltroAnalogico {
    taxa_amostragem: f32,
    frequencia_corte: f32,
    ganho: f32,
    ligado: bool,
    alpha: f32,
    estado_l: f32,
    estado_r: f32,
}

impl FiltroAnalogico {
    /// Cria o filtro passa-baixas com corte tipico do Mega Drive Modelo 1 (~3386 Hz).
    /// Marcado como APROXIMADO (nao verificado contra silicio).
    pub fn novo(taxa_amostragem: f32) -> Self {
        let corte_padrao = 3386.0;
        let mut s = Self {
            taxa_amostragem,
            frequencia_corte: corte_padrao,
            ganho: 1.0,
            ligado: true,
            alpha: 1.0,
            estado_l: 0.0,
            estado_r: 0.0,
        };
        s.recalcular();
        s
    }

    fn recalcular(&mut self) {
        let rc = 1.0 / (2.0 * PI * self.frequencia_corte);
        let dt = 1.0 / self.taxa_amostragem;
        self.alpha = dt / (rc + dt);
    }

    /// Processa uma amostra estereo (esquerda, direita) em f32.
    pub fn processar(&mut self, l: f32, r: f32) -> (f32, f32) {
        if !self.ligado {
            return (l * self.ganho, r * self.ganho);
        }
        self.estado_l += self.alpha * (l - self.estado_l);
        self.estado_r += self.alpha * (r - self.estado_r);
        (self.estado_l * self.ganho, self.estado_r * self.ganho)
    }

    /// Reseta os estados internos do filtro.
    pub fn reset(&mut self) {
        self.estado_l = 0.0;
        self.estado_r = 0.0;
    }

    /// Mede a resposta em frequencia em dB para senoides sinteticas nas 8 frequencias tipicas.
    pub fn medir_resposta_db(&self) -> Vec<(u32, f32)> {
        let freqs = [100, 250, 500, 1000, 2000, 3386, 5000, 10000];
        let mut resp = Vec::new();

        for &f in &freqs {
            let f_f32 = f as f32;
            let omega = 2.0 * PI * f_f32;
            let rc = 1.0 / (2.0 * PI * self.frequencia_corte);
            // Resposta em amplitude de 1º ordem H(jw) = 1 / sqrt(1 + (w*RC)^2)
            let mag = self.ganho / (1.0 + (omega * rc).powi(2)).sqrt();
            let db = 20.0 * mag.log10();
            resp.push((f, db));
        }

        resp
    }
}
