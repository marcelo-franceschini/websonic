//! Camada fina entre o JavaScript e os crates do projeto. Sem lógica própria.
use sonic1_console::Console;
use sonic1_rom::{Veredito, identificar as identificar_rom};
use wasm_bindgen::prelude::*;

/// O que o JavaScript recebe de volta.
#[wasm_bindgen]
pub struct Resultado {
    sha1: String,
    tamanho: usize,
    valida: bool,
    motivo: String,
}

#[wasm_bindgen]
impl Resultado {
    /// SHA-1 em hexadecimal.
    #[wasm_bindgen(getter)]
    pub fn sha1(&self) -> String {
        self.sha1.clone()
    }

    /// Tamanho em bytes.
    #[wasm_bindgen(getter)]
    pub fn tamanho(&self) -> usize {
        self.tamanho
    }

    /// Verdadeiro só para a ROM REV01.
    #[wasm_bindgen(getter)]
    pub fn valida(&self) -> bool {
        self.valida
    }

    /// Motivo da recusa. Vazio quando a ROM é válida.
    #[wasm_bindgen(getter)]
    pub fn motivo(&self) -> String {
        self.motivo.clone()
    }
}

/// Identifica o arquivo escolhido. Os bytes não saem do navegador.
#[wasm_bindgen]
pub fn identificar(dados: &[u8]) -> Resultado {
    let id = identificar_rom(dados);
    let (valida, motivo) = match &id.veredito {
        Veredito::Rev01 => (true, String::new()),
        Veredito::Recusada(m) => (false, m.descricao()),
    };
    Resultado {
        sha1: id.sha1,
        tamanho: id.tamanho,
        valida,
        motivo,
    }
}

/// O jogo rodando: o núcleo com tempo nominal, a ponte de vídeo e a imagem do último quadro.
/// A síntese de som é a Etapa 8: aqui não há som.
#[wasm_bindgen]
pub struct Jogo {
    console: Console,
}

impl Jogo {
    /// Cria o jogo sem conferir a ROM (para testes com ROM sintética). A página só chama
    /// [`Jogo::criar`].
    pub fn sem_validar(rom: Vec<u8>) -> Self {
        Self {
            console: Console::novo(rom),
        }
    }

    pub fn console(&self) -> &Console {
        &self.console
    }
}

#[wasm_bindgen]
impl Jogo {
    /// Cria o jogo. Só aceita a REV01: qualquer outro arquivo é recusado.
    pub fn criar(dados: &[u8]) -> Result<Jogo, String> {
        let limpos = if dados.len() == 524_288 + 512 { &dados[512..] } else { dados };
        let id = identificar_rom(limpos);
        match id.veredito {
            Veredito::Rev01 => Ok(Self::sem_validar(limpos.to_vec())),
            Veredito::Recusada(m) => Err(m.descricao()),
        }
    }

    /// Cria o jogo com a taxa de amostragem do AudioContext.
    pub fn criar_com_audio(dados: &[u8], taxa_audio: f64) -> Result<Jogo, String> {
        if !taxa_audio.is_finite() || !(3_000.0..=768_000.0).contains(&taxa_audio) {
            return Err("Taxa de áudio inválida".into());
        }
        let limpos = if dados.len() == 524_288 + 512 { &dados[512..] } else { dados };
        let id = identificar_rom(limpos);
        match id.veredito {
            Veredito::Rev01 => Ok(Self {
                console: Console::com_taxa_audio(limpos.to_vec(), taxa_audio),
            }),
            Veredito::Recusada(m) => Err(m.descricao()),
        }
    }

    /// Botões apertados: bits 0 a 7 = cima, baixo, esquerda, direita, B, C, A, Start.
    pub fn botoes(&mut self, mascara: u8) {
        self.console.definir_botoes(mascara);
    }

    /// Coleta os candidatos de vídeo para o compositor WebGL2.
    pub fn ativar_gpu(&mut self) {
        self.console.ativar_gpu();
    }

    /// Ativa o compositor gráfico com dimensões configuradas.
    pub fn ativar_gpu_com_dimensoes(&mut self, largura: usize, altura: usize) {
        self.console.ativar_gpu_com_dimensoes(largura, altura);
    }

    /// Ativa o compositor gráfico em modo widescreen 16:9 (426x224).
    pub fn ativar_gpu_widescreen(&mut self) {
        self.console.ativar_gpu_widescreen();
    }

    /// Usa somente a composição gráfica, sem rasterizar uma imagem CPU.
    pub fn ativar_gpu_sem_referencia(&mut self) {
        self.console.ativar_gpu_sem_referencia();
    }

    pub fn largura_gpu(&self) -> usize {
        self.console
            .quadro_gpu()
            .map(|q| q.largura())
            .unwrap_or(320)
    }

    pub fn altura_gpu(&self) -> usize {
        self.console.quadro_gpu().map(|q| q.altura()).unwrap_or(224)
    }

    pub fn desativar_gpu(&mut self) {
        self.console.desativar_gpu();
    }

    /// Avança simulação e áudio; a apresentação usa as camadas separadamente.
    pub fn simular_quadro(&mut self) {
        self.console.simular_quadro();
    }

    pub fn camadas_gpu(&self) -> Vec<u8> {
        self.console
            .quadro_gpu()
            .map(|q| q.camadas().to_vec())
            .unwrap_or_default()
    }

    pub fn camadas_gpu_ptr(&self) -> *const u8 {
        self.console
            .quadro_gpu()
            .map(|q| q.camadas().as_ptr())
            .unwrap_or(std::ptr::null())
    }

    pub fn camadas_gpu_len(&self) -> usize {
        self.console
            .quadro_gpu()
            .map(|q| q.camadas().len())
            .unwrap_or(0)
    }

    pub fn seletores_gpu(&self) -> Vec<u16> {
        self.console
            .quadro_gpu()
            .map(|q| q.seletores().to_vec())
            .unwrap_or_default()
    }

    pub fn seletores_gpu_ptr(&self) -> *const u16 {
        self.console
            .quadro_gpu()
            .map(|q| q.seletores().as_ptr())
            .unwrap_or(std::ptr::null())
    }

    pub fn seletores_gpu_len(&self) -> usize {
        self.console
            .quadro_gpu()
            .map(|q| q.seletores().len())
            .unwrap_or(0)
    }

    pub fn sprites_gpu(&self) -> Vec<u8> {
        self.console.sprites_gpu_slice().to_vec()
    }

    pub fn sprites_gpu_ptr(&self) -> *const u8 {
        self.console.sprites_gpu_slice().as_ptr()
    }

    pub fn sprites_gpu_len(&self) -> usize {
        self.console.sprites_gpu_slice().len()
    }

    pub fn fundos_gpu(&self) -> Vec<u8> {
        self.console
            .quadro_gpu()
            .map(|q| q.fundos().to_vec())
            .unwrap_or_default()
    }

    pub fn fundos_gpu_ptr(&self) -> *const u8 {
        self.console
            .quadro_gpu()
            .map(|q| q.fundos().as_ptr())
            .unwrap_or(std::ptr::null())
    }

    pub fn fundos_gpu_len(&self) -> usize {
        self.console
            .quadro_gpu()
            .map(|q| q.fundos().len())
            .unwrap_or(0)
    }

    pub fn paletas_gpu(&self) -> Vec<u16> {
        self.console
            .quadro_gpu()
            .map(|q| q.paletas().to_vec())
            .unwrap_or_default()
    }

    pub fn paletas_gpu_ptr(&self) -> *const u16 {
        self.console
            .quadro_gpu()
            .map(|q| q.paletas().as_ptr())
            .unwrap_or(std::ptr::null())
    }

    pub fn paletas_gpu_len(&self) -> usize {
        self.console
            .quadro_gpu()
            .map(|q| q.paletas().len())
            .unwrap_or(0)
    }

    /// Informa quantos pares estéreo ainda aguardam reprodução.
    pub fn ocupacao_audio(&mut self, pares: usize) {
        self.console.definir_ocupacao_audio(pares);
    }

    /// Roda um quadro e devolve a imagem de 320 por 224 em RGBA.
    pub fn quadro(&mut self) -> Vec<u8> {
        self.console.executar_quadro()
    }

    /// Devolve as amostras de áudio estereo geradas no último quadro (Float32Array intercalado).
    pub fn audio(&mut self) -> Vec<f32> {
        self.console.extrair_audio()
    }

    /// Ponteiro direto para o buffer de áudio na memória Wasm (zero-copy).
    pub fn audio_ptr(&self) -> *const f32 {
        self.console.audio_slice().as_ptr()
    }

    /// Comprimento do buffer de áudio (número de floats).
    pub fn audio_len(&self) -> usize {
        self.console.audio_slice().len()
    }

    /// Modo atual de jogo (0 = Sega, 4 = Titulo, 8 = Demo, 0x0C = Level, etc).
    pub fn modo_jogo(&self) -> u8 {
        self.console.modo_jogo()
    }

    /// Hash da imagem de 9 bits do último quadro, em hexadecimal (para conferir com o relatório).
    pub fn hash_da_imagem(&self) -> String {
        format!("{:016x}", self.console.hash_da_imagem())
    }

    /// Erro do núcleo, se parou. Vazio se está tudo bem.
    pub fn erro(&self) -> String {
        self.console.erro().unwrap_or_default()
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn jogo_recusa_arquivo_que_nao_e_a_rev01() {
        assert!(Jogo::criar(b"abc").is_err());
    }

    #[test]
    fn jogo_com_rom_sintetica_roda_quadros_deterministicos_com_imagem_do_tamanho_certo() {
        // ROM de zeros: o núcleo para na verificação da soma, sem pânico. O quadro tem o tamanho certo.
        let rodar = || {
            let mut j = Jogo::sem_validar(vec![0u8; 0x4000]);
            let mut ultimo = Vec::new();
            for _ in 0..70 {
                ultimo = j.quadro();
            }
            (ultimo.len(), j.hash_da_imagem(), j.erro())
        };
        let (n, h, e) = rodar();
        assert_eq!(n, 320 * 224 * 4);
        assert_eq!((n, h.clone(), e.clone()), {
            let (a, b, c) = rodar();
            (a, b, c)
        });
    }

    #[test]
    fn recusa_arquivo_qualquer_e_mostra_o_hash() {
        let r = identificar(b"abc");
        assert!(!r.valida());
        assert_eq!(r.sha1(), "a9993e364706816aba3e25717850c26c9cd0d89d");
        assert_eq!(r.tamanho(), 3);
        assert!(r.motivo().contains("tamanho"));
    }
}
