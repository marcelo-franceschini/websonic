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
    taxa_saida: f64,

    ganho_fm: f32,
    ganho_psg: f32,

    buffer_audio: Vec<(f32, f32)>,
    pendentes_fm: VecDeque<(f32, f32)>,
    pendentes_psg: VecDeque<(f32, f32)>,
    ocupacao_externa: Option<usize>,
    tamanho_alvo_buffer: usize,

    cont_fm_in: u64,
    cont_psg_in: u64,
    cont_saida: u64,
    falhas: u64,

    fm_float: Vec<(f32, f32)>,
    psg_float: Vec<f32>,
    saida_fm: Vec<(f32, f32)>,
    saida_psg: Vec<f32>,
}

impl MisturadorSaida {
    /// Cria o misturador configurado para a taxa de saida do navegador (ex: 48000.0 ou 44100.0).
    pub fn novo(taxa_saida: f64) -> Self {
        let reamostrador_fm = Reamostrador::novo(TAXA_FM_NATIVA, taxa_saida);
        let reamostrador_psg = Reamostrador::novo(TAXA_PSG_NATIVA, taxa_saida);
        let filtro_analogico = FiltroAnalogico::novo(taxa_saida as f32);

        // Alvo de buffer: 3 quadros nominais de 60 Hz (~50 ms)
        let tamanho_alvo_buffer = ((taxa_saida / 60.0) * 3.0) as usize;

        Self {
            reamostrador_fm,
            reamostrador_psg,
            filtro_analogico,
            taxa_saida,
            ganho_fm: 0.75,
            ganho_psg: 0.50,
            buffer_audio: Vec::with_capacity(tamanho_alvo_buffer * 4),
            pendentes_fm: VecDeque::new(),
            pendentes_psg: VecDeque::new(),
            ocupacao_externa: None,
            tamanho_alvo_buffer,
            cont_fm_in: 0,
            cont_psg_in: 0,
            cont_saida: 0,
            falhas: 0,
            fm_float: Vec::new(),
            psg_float: Vec::new(),
            saida_fm: Vec::new(),
            saida_psg: Vec::new(),
        }
    }

    /// Altera a frequencia de corte do filtro analogico (APROXIMADO).
    pub fn definir_corte_analogico(&mut self, corte_hz: f32) {
        self.filtro_analogico.definir_corte(corte_hz);
    }

    /// Taxa de saida configurada (ex: 48000.0 ou 44100.0).
    pub fn taxa_saida(&self) -> f64 {
        self.taxa_saida
    }

    /// Ativa ou desativa o filtro analogico pos-DAC (APROXIMADO).
    pub fn definir_filtro_ligado(&mut self, ligado: bool) {
        self.filtro_analogico.definir_ligado(ligado);
    }

    /// Altera os ganhos relativos de FM e PSG.
    pub fn definir_ganhos(&mut self, ganho_fm: f32, ganho_psg: f32) {
        self.ganho_fm = ganho_fm;
        self.ganho_psg = ganho_psg;
    }

    /// Alimenta um lote de amostras nativas de FM e PSG geradas por um quadro.
    pub fn alimentar_quadro(&mut self, amostras_fm: &[AmostraFm], amostras_psg: &[AmostraPsg]) {
        self.alimentar_com(
            amostras_fm.iter().map(|a| a.misturado),
            amostras_psg.iter().map(|p| p.canais),
        );
    }

    /// Alimenta o quadro com amostras de PSG em formato bruto `u16`.
    pub fn alimentar_quadro_u16(&mut self, amostras_fm: &[AmostraFm], amostras_psg: &[u16]) {
        self.alimentar_com(
            amostras_fm.iter().map(|a| a.misturado),
            amostras_psg.iter().map(|&e| AmostraPsg::from(e).canais),
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

        self.fm_float.clear();
        self.psg_float.clear();
        self.saida_fm.clear();
        self.saida_psg.clear();

        // Converte FM para f32 [-1.0, 1.0]
        for (fl, fr) in fm {
            let l = (fl as f32) / 2048.0;
            let r = (fr as f32) / 2048.0;
            self.fm_float.push((l * self.ganho_fm, r * self.ganho_fm));
        }
        self.reamostrador_fm.alimentar_estereo(&self.fm_float);

        // Converte PSG para f32 mono centralizado
        for canais in psg {
            let v0 = PSG_VOL_TABLE[usize::from(canais[0] & 15)];
            let v1 = PSG_VOL_TABLE[usize::from(canais[1] & 15)];
            let v2 = PSG_VOL_TABLE[usize::from(canais[2] & 15)];
            let v3 = PSG_VOL_TABLE[usize::from(canais[3] & 15)];
            let soma = (v0 + v1 + v2 + v3) * 0.25 * self.ganho_psg;
            self.psg_float.push(soma);
        }
        self.reamostrador_psg.alimentar_mono(&self.psg_float);

        // Processa reamostragem de ambos
        self.reamostrador_fm.processar(&mut self.saida_fm);
        self.reamostrador_psg.processar_mono(&mut self.saida_psg);

        // Combina as saidas interpoladas e passa pelo filtro analogico
        self.pendentes_fm.extend(self.saida_fm.iter().copied());
        self.pendentes_psg
            .extend(self.saida_psg.iter().copied().map(|p| (p, p)));
        while !self.pendentes_fm.is_empty() && !self.pendentes_psg.is_empty() {
            let (fml, fmr) = self.pendentes_fm.pop_front().expect("FM disponível");
            let (psgl, psgr) = self.pendentes_psg.pop_front().expect("PSG disponível");

            let mist_l = (fml + psgl).clamp(-1.0, 1.0);
            let mist_r = (fmr + psgr).clamp(-1.0, 1.0);

            let filtrado = self.filtro_analogico.processar(mist_l, mist_r);
            self.buffer_audio
                .push((filtrado.0.clamp(-1.0, 1.0), filtrado.1.clamp(-1.0, 1.0)));
        }

        // Controlador proporcional de baixa banda para ajustar a taxa de amostragem
        self.atualizar_controlador_taxa();
    }

    fn atualizar_controlador_taxa(&mut self) {
        let ocupacao = self.ocupacao_externa.unwrap_or(self.buffer_audio.len());
        let erro = (ocupacao as f64) - (self.tamanho_alvo_buffer as f64);

        // Ganho proporcional suave: ~0.1% a cada 500 amostras de desvio
        let kp = 0.000002;
        let ajuste = (erro * kp).clamp(-0.005, 0.005);

        // Se o buffer esta crescendo, aceleramos a saida (ou aumentamos razao)
        self.reamostrador_fm.definir_ajuste_fino(ajuste);
        self.reamostrador_psg.definir_ajuste_fino(ajuste);
    }

    /// Atualiza a ocupação da fila consumida pelo dispositivo, em pares estéreo.
    pub fn definir_ocupacao_externa(&mut self, pares: usize) {
        self.ocupacao_externa = Some(pares);
    }

    /// Retira um bloco de amostras estereo intercaladas (L, R, L, R...) para o AudioWorklet.
    /// Se faltar amostras no buffer, repete a ultima amostra com decaimento suave e conta a falha.
    pub fn extrair_bloco_worklet(&mut self, destino_intercalado: &mut [f32]) -> usize {
        let pares_pedidos = destino_intercalado.len() / 2;
        let pares_disponiveis = self.buffer_audio.len().min(pares_pedidos);

        let mut ultimo_l = 0.0f32;
        let mut ultimo_r = 0.0f32;

        for (i, &(l, r)) in self.buffer_audio[..pares_disponiveis].iter().enumerate() {
            destino_intercalado[i * 2] = l;
            destino_intercalado[i * 2 + 1] = r;
            ultimo_l = l;
            ultimo_r = r;
        }

        self.buffer_audio.drain(..pares_disponiveis);
        self.cont_saida = self.cont_saida.wrapping_add(pares_disponiveis as u64);

        // Se faltou dado, preenche com decaimento curto e registra a falha
        if pares_disponiveis < pares_pedidos {
            self.falhas = self.falhas.wrapping_add(1);
            let mut decay = 0.95f32;
            for i in pares_disponiveis..pares_pedidos {
                ultimo_l *= decay;
                ultimo_r *= decay;
                destino_intercalado[i * 2] = ultimo_l;
                destino_intercalado[i * 2 + 1] = ultimo_r;
                decay *= 0.95;
            }
        }

        pares_disponiveis
    }

    /// Numero de amostras estereo prontas no buffer.
    pub fn ocupacao_buffer(&self) -> usize {
        self.buffer_audio.len()
    }

    /// Contador de falhas (quantas chamadas do worklet encontraram buffer vazio).
    pub fn falhas(&self) -> u64 {
        self.falhas
    }

    /// Contadores acumulados: (amostras FM de entrada, amostras PSG de entrada, amostras de saida).
    pub fn contadores(&self) -> (u64, u64, u64) {
        (self.cont_fm_in, self.cont_psg_in, self.cont_saida)
    }

    /// Reseta o misturador e esvazia o buffer.
    pub fn reset(&mut self) {
        self.reamostrador_fm.reset();
        self.reamostrador_psg.reset();
        self.filtro_analogico.reset();
        self.buffer_audio.clear();
        self.pendentes_fm.clear();
        self.pendentes_psg.clear();
        self.fm_float.clear();
        self.psg_float.clear();
        self.saida_fm.clear();
        self.saida_psg.clear();
        self.ocupacao_externa = None;
        self.cont_fm_in = 0;
        self.cont_psg_in = 0;
        self.cont_saida = 0;
        self.falhas = 0;
    }
}
