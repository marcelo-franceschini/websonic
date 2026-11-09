//! Sintetizador fiel do Mega Drive (Etapa 8).
//!
//! Dois chips, cada um na sua taxa: o PSG (mestre/15, uma amostra a cada 16 ciclos) e o FM
//! (YM2612, mestre/42 por ciclo interno, uma amostra a cada 24 ciclos). A saída é digital,
//! inteira, antes de qualquer reamostragem ou filtro analógico.
//! Regras do crate (checadas pelo clippy): sem float, sem estouro implícito, sem casts que truncam.
#![forbid(unsafe_code)]

pub mod psg;

/// Ciclos do relógio mestre NTSC por ciclo do PSG (mestre/15).
pub const MCLK_POR_CICLO_PSG: u64 = 15;
/// Ciclos do PSG por amostra de saída.
pub const CICLOS_PSG_POR_AMOSTRA: u32 = 16;
