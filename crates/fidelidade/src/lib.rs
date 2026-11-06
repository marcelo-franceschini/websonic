//! Formato do trace da fidelidade (Etapa 1).
//!
//! O trace e o registro, quadro a quadro, do que o oraculo produziu para uma entrada.
//! Este crate define o formato binario, o escritor, o leitor e a comparacao.
//! Ele nao depende do oraculo (GPLv3) e nao contem dado da ROM.
//!
//! Regras de determinismo do formato:
//! - tamanho fixo por registro, ordem de bytes little-endian;
//! - sem relogio de parede, sem ponteiros, sem ordem de hash;
//! - eventos ordenados por (mclk, seq) dentro do quadro.

pub mod comparar;
pub mod evento;
pub mod formato;
pub mod hash;

pub use comparar::{Divergencia, Onde, comparar_leitores, comparar_quadros};
pub use evento::{Evento, Origem, TAMANHO_EVENTO, Tipo};
pub use formato::{Cabecalho, Categoria, Erro, Escritor, Leitor, Quadro, VERSAO};
