//! Camada fina entre o JavaScript e os crates do projeto. Sem lógica própria.
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

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn recusa_arquivo_qualquer_e_mostra_o_hash() {
        let r = identificar(b"abc");
        assert!(!r.valida());
        assert_eq!(r.sha1(), "a9993e364706816aba3e25717850c26c9cd0d89d");
        assert_eq!(r.tamanho(), 3);
        assert!(r.motivo().contains("tamanho"));
    }
}
