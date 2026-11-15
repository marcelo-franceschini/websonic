//! Recompilador antecipado do 68000 e suporte de execução.
//!
//! O recurso padrão `gerador` fornece a ferramenta de análise. O runtime pode ser usado
//! em WebAssembly com `default-features = false`, sem incluir o gerador ou a validação da ROM.
#![forbid(unsafe_code)]

pub mod runtime;

#[cfg(feature = "gerador")]
mod gerador;
#[cfg(feature = "gerador")]
pub use gerador::{Erro, Programa, descobrir, descobrir_fragmento};
