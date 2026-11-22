//! Misturador e buffer de reproducao para o navegador (Etapa 8).
//!
//! Integra os reamostradores de FM e PSG com o filtro analogico aproximado e
//! uma fila circular para alimentacao do AudioWorklet.

use crate::analogico::FiltroAnalogico;
use crate::reamostrador::Reamostrador;
use sonic1_audio::{AmostraFm, AmostraPsg};
use std::collections::VecDeque;

/// Frequencias de amostragem nativas do Mega Drive NTSC (mclk = 53 693 175 Hz).
pub const TAXA_FM_NATIVA: f64 = 53_693_175.0 / 1008.0; // ~53 267.0387 Hz
pub const TAXA_PSG_NATIVA: f64 = 53_693_175.0 / 240.0; // ~223 721.5625 Hz

/// Tabela de atenuacao de 2 dB por passo do PSG (SN76489 / YM7101).
const PSG_VOL_TABLE: [f32; 16] = [
    1.000000, 0.794328, 0.630957, 0.501187, 0.398107, 0.316228, 0.251189, 0.199526, 0.158489,
    0.125893, 0.100000, 0.079433, 0.063096, 0.050119, 0.039811, 0.000000,
];

/// Misturador completo com controle de fluxo para o AudioWorklet.
#[derive(Clone, Debug)]
pub struct MisturadorSaida {
    reamostrador_fm: Reamostrador,
    reamostrador_psg: Reamostrador,
    filtro_analogico: FiltroAnalogico,

    ganho_fm: f32,
    ganho_psg: f32,
    ganho_psg_canais: [f32; 4],
    psg_ganhos_padrao: bool,

    buffer_audio: VecDeque<(f32, f32)>,
    pendentes_fm: VecDeque<(f32, f32)>,
    pendentes_psg: VecDeque<f32>,
    ocupacao_externa: Option<usize>,
    ocupacao_externa_atualizada: bool,
    tamanho_alvo_buffer: usize,

    cont_fm_in: u64,
    cont_psg_in: u64,
    cont_saida: u64,
    falhas: u64,

    saida_fm: Vec<(f32, f32)>,
    saida_psg: Vec<f32>,
}

impl MisturadorSaida {
    /// Cria o misturador configurado para a taxa de saida do navegador (ex: 48000.0 ou 44100.0).
    pub fn novo(taxa_saida: f64) -> Self {
        let reamostrador_fm = Reamostrador::novo(TAXA_FM_NATIVA, taxa_saida);
        let reamostrador_psg = Reamostrador::novo_mono(TAXA_PSG_NATIVA, taxa_saida);
        let filtro_analogico = FiltroAnalogico::novo(taxa_saida as f32);

        // Alvo de buffer: 3 quadros nominais de 60 Hz (~50 ms)
        let tamanho_alvo_buffer = ((taxa_saida / 60.0) * 3.0) as usize;

        Self {
            reamostrador_fm,
            reamostrador_psg,
            filtro_analogico,
            ganho_fm: 0.75,
            ganho_psg: 0.50,
            ganho_psg_canais: [1.0; 4],
            psg_ganhos_padrao: true,
            buffer_audio: VecDeque::with_capacity(tamanho_alvo_buffer * 4),
            pendentes_fm: VecDeque::with_capacity(1024),
            pendentes_psg: VecDeque::with_capacity(1024),
            ocupacao_externa: None,
            ocupacao_externa_atualizada: false,
            tamanho_alvo_buffer,
            cont_fm_in: 0,
            cont_psg_in: 0,
            cont_saida: 0,
            falhas: 0,
            saida_fm: Vec::with_capacity(1024),
            saida_psg: Vec::with_capacity(1024),
        }
    }

    /// Define o ganho individual dos 4 canais do PSG (0.0 a 1.0) para separar musica e FX.
    pub fn definir_ganhos_psg_canais(&mut self, ganhos: [f32; 4]) {
        let mut padrao = true;
        for (dst, &src) in self.ganho_psg_canais.iter_mut().zip(ganhos.iter()) {
            let v = if src.is_finite() {
                src.clamp(0.0, 1.0)
            } else {
                1.0
            };
            *dst = v;
            if (v - 1.0).abs() > f32::EPSILON {
                padrao = false;
            }
        }
        self.psg_ganhos_padrao = padrao;
    }

    /// Alimenta um lote de amostras nativas de FM e PSG geradas por um quadro.
    pub fn alimentar_quadro(&mut self, amostras_fm: &[AmostraFm], amostras_psg: &[AmostraPsg]) {
        self.alimentar_com(
            amostras_fm.iter().map(|a| a.misturado),
            amostras_psg.iter().map(|p| p.canais),
        );
    }

    /// Alimenta um lote compacto de amostras nativas de FM `(i16, i16)` e PSG `u16`.
    pub fn alimentar_quadro_compacto(&mut self, amostras_fm: &[(i16, i16)], amostras_psg: &[u16]) {
        self.alimentar_com(
            amostras_fm.iter().copied(),
            amostras_psg.iter().map(|&e| AmostraPsg::from(e).canais),
        );
    }

    /// Caminho único das três entradas: converte, reamostra, mistura e filtra.
    /// `fm` traz a saída misturada de cada amostra de FM; `psg`, a atenuação dos 4 canais.
    /// Assim o site e os testes de fidelidade passam pelas mesmas contas.
    fn alimentar_com(
        &mut self,
        fm: impl ExactSizeIterator<Item = (i16, i16)>,
        psg: impl ExactSizeIterator<Item = [u8; 4]>,
    ) {
        self.cont_fm_in = self.cont_fm_in.wrapping_add(fm.len() as u64);
        self.cont_psg_in = self.cont_psg_in.wrapping_add(psg.len() as u64);

        self.saida_fm.clear();
        self.saida_psg.clear();

        // Converte FM e PSG em passagem unica direto para os buffers internos dos reamostradores.
        let escala_fm = self.ganho_fm * (1.0 / 2048.0);
        self.reamostrador_fm.alimentar_estereo_iter(
            fm.map(|(fl, fr)| ((fl as f32) * escala_fm, (fr as f32) * escala_fm)),
        );

        let escala_psg = 0.25 * self.ganho_psg;
        let ganhos_psg = self.ganho_psg_canais;
        let psg_padrao = self.psg_ganhos_padrao;
        self.reamostrador_psg.alimentar_mono_iter(psg.map(|canais| {
            let v0 = PSG_VOL_TABLE[usize::from(canais[0] & 15)];
            let v1 = PSG_VOL_TABLE[usize::from(canais[1] & 15)];
            let v2 = PSG_VOL_TABLE[usize::from(canais[2] & 15)];
            let v3 = PSG_VOL_TABLE[usize::from(canais[3] & 15)];
            if psg_padrao {
                (v0 + v1 + v2 + v3) * escala_psg
            } else {
                (v0 * ganhos_psg[0]
                    + v1 * ganhos_psg[1]
                    + v2 * ganhos_psg[2]
                    + v3 * ganhos_psg[3])
                    * escala_psg
            }
        }));

        // Processa reamostragem de ambos
        self.reamostrador_fm.processar(&mut self.saida_fm);
        self.reamostrador_psg.processar_mono(&mut self.saida_psg);

        // Combina FM e PSG diretamente sem copiar todo o quadro para as filas de pendentes:
        // apenas o excedente (normalmente 0..2 amostras por quadro) entra em `pendentes_*`.
        let total_fm = self.pendentes_fm.len() + self.saida_fm.len();
        let total_psg = self.pendentes_psg.len() + self.saida_psg.len();
        let pares_totais = total_fm.min(total_psg);
        self.buffer_audio.reserve(pares_totais);

        let mut idx_fm = 0usize;
        let mut idx_psg = 0usize;

        if !self.pendentes_fm.is_empty() && !self.saida_psg.is_empty() {
            let k = self.pendentes_fm.len().min(self.saida_psg.len());
            for ((fml, fmr), &psg) in self.pendentes_fm.drain(..k).zip(&self.saida_psg[..k]) {
                let mist_l = (fml + psg).clamp(-1.0, 1.0);
                let mist_r = (fmr + psg).clamp(-1.0, 1.0);
                self.buffer_audio
                    .push_back(self.filtro_analogico.processar(mist_l, mist_r));
            }
            idx_psg = k;
        } else if !self.pendentes_psg.is_empty() && !self.saida_fm.is_empty() {
            let k = self.pendentes_psg.len().min(self.saida_fm.len());
            for (&(fml, fmr), psg) in self.saida_fm[..k].iter().zip(self.pendentes_psg.drain(..k)) {
                let mist_l = (fml + psg).clamp(-1.0, 1.0);
                let mist_r = (fmr + psg).clamp(-1.0, 1.0);
                self.buffer_audio
                    .push_back(self.filtro_analogico.processar(mist_l, mist_r));
            }
            idx_fm = k;
        }

        if self.pendentes_fm.is_empty() && self.pendentes_psg.is_empty() {
            let diretos = (self.saida_fm.len() - idx_fm).min(self.saida_psg.len() - idx_psg);
            for (&(fml, fmr), &psg) in self.saida_fm[idx_fm..idx_fm + diretos]
                .iter()
                .zip(&self.saida_psg[idx_psg..idx_psg + diretos])
            {
                let mist_l = (fml + psg).clamp(-1.0, 1.0);
                let mist_r = (fmr + psg).clamp(-1.0, 1.0);
                self.buffer_audio
                    .push_back(self.filtro_analogico.processar(mist_l, mist_r));
            }
            idx_fm += diretos;
            idx_psg += diretos;
        }

        if idx_fm < self.saida_fm.len() {
            self.pendentes_fm
                .extend(self.saida_fm[idx_fm..].iter().copied());
        }
        if idx_psg < self.saida_psg.len() {
            self.pendentes_psg
                .extend(self.saida_psg[idx_psg..].iter().copied());
        }

        // Controlador proporcional de baixa banda para ajustar a taxa de amostragem
        self.atualizar_controlador_taxa();
    }

    fn atualizar_controlador_taxa(&mut self) {
        let ocupacao = match self.ocupacao_externa {
            Some(pares) => {
                if !self.ocupacao_externa_atualizada {
                    return;
                }
                self.ocupacao_externa_atualizada = false;
                pares
            }
            None => self.buffer_audio.len(),
        };
        let erro = (ocupacao as f64) - (self.tamanho_alvo_buffer as f64);

        // Ganho proporcional suave: ~0.1% a cada 500 amostras de desvio
        let kp = 0.000002;
        let ajuste = (erro * kp).clamp(-0.005, 0.005);

        // Se o buffer esta crescendo (erro > 0), aumenta o passo de entrada para gerar menos amostras.
        self.reamostrador_fm.definir_ajuste_fino(ajuste);
        self.reamostrador_psg.definir_ajuste_fino(ajuste);
    }

    /// Atualiza a ocupação da fila consumida pelo dispositivo, em pares estéreo.
    pub fn definir_ocupacao_externa(&mut self, pares: usize) {
        self.ocupacao_externa = Some(pares);
        self.ocupacao_externa_atualizada = true;
    }

    /// Retira um bloco de amostras estereo intercaladas (L, R, L, R...) para o AudioWorklet.
    /// Se faltar amostras no buffer, repete a ultima amostra com decaimento suave e conta a falha.
    pub fn extrair_bloco_worklet(&mut self, destino_intercalado: &mut [f32]) -> usize {
        let pares_pedidos = destino_intercalado.len() / 2;
        let pares_disponiveis = self.buffer_audio.len().min(pares_pedidos);

        let mut ultimo_l = 0.0f32;
        let mut ultimo_r = 0.0f32;

        for (par_dst, (l, r)) in destino_intercalado
            .as_chunks_mut::<2>()
            .0
            .iter_mut()
            .take(pares_disponiveis)
            .zip(self.buffer_audio.drain(..pares_disponiveis))
        {
            par_dst[0] = l;
            par_dst[1] = r;
            ultimo_l = l;
            ultimo_r = r;
        }

        self.cont_saida = self.cont_saida.wrapping_add(pares_disponiveis as u64);

        // Se faltou dado, preenche com decaimento curto e registra a falha
        if pares_disponiveis < pares_pedidos {
            self.falhas = self.falhas.wrapping_add(1);
            let mut decay = 0.95f32;
            for par_dst in destino_intercalado
                .as_chunks_mut::<2>()
                .0
                .iter_mut()
                .skip(pares_disponiveis)
                .take(pares_pedidos - pares_disponiveis)
            {
                ultimo_l *= decay;
                ultimo_r *= decay;
                par_dst[0] = ultimo_l;
                par_dst[1] = ultimo_r;
                decay *= 0.95;
            }
        }

        pares_disponiveis
    }

    /// Retira todas as amostras estereo prontas no buffer diretamente para `destino`
    /// (intercaladas L, R, L, R...), sem exigir preenchimento previo com zeros.
    pub fn extrair_todos_intercalados(&mut self, destino: &mut Vec<f32>) -> usize {
        destino.clear();
        let pares = self.buffer_audio.len();
        if pares == 0 {
            return 0;
        }
        destino.reserve(pares.saturating_mul(2));
        for (l, r) in self.buffer_audio.drain(..) {
            destino.push(l);
            destino.push(r);
        }
        self.cont_saida = self.cont_saida.wrapping_add(pares as u64);
        pares
    }

    /// Numero de amostras estereo prontas no buffer.
    pub fn ocupacao_buffer(&self) -> usize {
        self.buffer_audio.len()
    }

    /// Contador de falhas (quantas chamadas do worklet encontraram buffer vazio).
    pub fn falhas(&self) -> u64 {
        self.falhas
    }

    /// Reseta o misturador e esvazia o buffer.
    pub fn reset(&mut self) {
        self.reamostrador_fm.reset();
        self.reamostrador_psg.reset();
        self.filtro_analogico.reset();
        self.buffer_audio.clear();
        self.pendentes_fm.clear();
        self.pendentes_psg.clear();
        self.saida_fm.clear();
        self.saida_psg.clear();
        self.ocupacao_externa = None;
        self.ocupacao_externa_atualizada = false;
        self.cont_fm_in = 0;
        self.cont_psg_in = 0;
        self.cont_saida = 0;
        self.falhas = 0;
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn extracao_parcial_em_fila_o1_preserva_ordem_e_conta_falha() {
        let mut m = MisturadorSaida::novo(48_000.0);
        let fm = vec![(1024i16, -512i16); 890];
        let psg = vec![0xFFFFu16; 3733];
        m.alimentar_quadro_compacto(&fm, &psg);
        let disponiveis = m.ocupacao_buffer();
        assert!(disponiveis > 256);

        let mut bloco1 = vec![0.0f32; 256];
        let lidos1 = m.extrair_bloco_worklet(&mut bloco1);
        assert_eq!(lidos1, 128);
        assert_eq!(m.falhas(), 0);
        assert_eq!(m.ocupacao_buffer(), disponiveis - 128);

        let mut bloco2 = vec![0.0f32; (disponiveis + 64) * 2];
        let lidos2 = m.extrair_bloco_worklet(&mut bloco2);
        assert_eq!(lidos2, disponiveis - 128);
        assert_eq!(m.falhas(), 1);
        assert_eq!(m.ocupacao_buffer(), 0);
    }

    #[test]
    fn extrair_todos_intercalados_equivale_a_extrair_bloco() {
        let mut m1 = MisturadorSaida::novo(48_000.0);
        let mut m2 = MisturadorSaida::novo(48_000.0);
        let fm = vec![(900i16, -700i16); 890];
        let psg = vec![0x00FFu16; 3733];
        m1.alimentar_quadro_compacto(&fm, &psg);
        m2.alimentar_quadro_compacto(&fm, &psg);

        let disp = m1.ocupacao_buffer();
        let mut via_bloco = vec![0.0f32; disp * 2];
        assert_eq!(m1.extrair_bloco_worklet(&mut via_bloco), disp);

        let mut via_todos = Vec::new();
        assert_eq!(m2.extrair_todos_intercalados(&mut via_todos), disp);
        assert_eq!(via_todos, via_bloco);
        assert_eq!(m2.ocupacao_buffer(), 0);
    }

    #[test]
    fn mistura_direta_preserva_alinhamento_em_lotes_assimetricos() {
        let mut continuo = MisturadorSaida::novo(48_000.0);
        let mut fracionado = MisturadorSaida::novo(48_000.0);
        // Mantem ajuste_fino = 0.0 entre os lotes para comparar alinhamento exato amostra a amostra.
        continuo.definir_ocupacao_externa(continuo.tamanho_alvo_buffer);
        fracionado.definir_ocupacao_externa(fracionado.tamanho_alvo_buffer);

        let fm_total: Vec<(i16, i16)> = (0..1800)
            .map(|i| {
                (
                    ((i * 19) % 1024) as i16 - 512,
                    512 - ((i * 13) % 1024) as i16,
                )
            })
            .collect();
        let psg_total: Vec<u16> = (0..7500)
            .map(|i| if (i / 17) % 2 == 0 { 0x05AF } else { 0xFFFF })
            .collect();

        continuo.alimentar_quadro_compacto(&fm_total, &psg_total);
        // Alimenta em lotes de tamanhos desiguais para exercitar pendentes_fm e pendentes_psg.
        fracionado.alimentar_quadro_compacto(&fm_total[..1100], &psg_total[..2000]);
        fracionado.alimentar_quadro_compacto(&fm_total[1100..], &psg_total[2000..]);

        let mut a = Vec::new();
        let mut b = Vec::new();
        continuo.extrair_todos_intercalados(&mut a);
        fracionado.extrair_todos_intercalados(&mut b);
        assert!(!a.is_empty());
        assert_eq!(a, b);
        assert!(fracionado.pendentes_fm.is_empty() || fracionado.pendentes_psg.is_empty());
    }
}
