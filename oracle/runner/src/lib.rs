//! Oraculo do WebSonic: o jgenesis (modificado so para observar) rodando sem tela.
//!
//! Licenca GPLv3, como o jgenesis. Este crate nunca entra no build do site nem do `core`.

pub mod descompressao;
pub mod entradas;
pub mod oraculo;
pub mod rom_sintetica;

pub use entradas::{Botao, Entradas};
pub use oraculo::{CapturaTela, ChamadaRotina, MotivoParada, Oraculo, Parada, ResultadoRotina};
