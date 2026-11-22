//! Saida de audio: reamostrador polifasico, parte analogica e buffer de reproducao (Etapa 8).
//!
//! Converte a saida dos chips nativos (FM e PSG) para reproducao suave no navegador
//! via AudioWorklet. Sem cortes, sem estalidos e com ajuste fino de taxa.
#![forbid(unsafe_code)]

pub mod analogico;
pub mod misturador;
pub mod reamostrador;

pub use analogico::FiltroAnalogico;
pub use misturador::{MisturadorSaida, TAXA_FM_NATIVA, TAXA_PSG_NATIVA};
pub use reamostrador::{FASES_POLIFASICO, Reamostrador, TAPS_POLIFASICO};
