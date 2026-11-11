//! Sintetizador fiel do Mega Drive (Etapa 8).
//!
//! Dois chips, cada um na sua taxa: o PSG (mestre/15, uma amostra a cada 16 ciclos) e o FM
//! (YM2612, mestre/42 por ciclo interno, uma amostra a cada 24 ciclos). A saída é digital,
//! inteira, antes de qualquer reamostragem ou filtro analógico.
//! Regras do crate (checadas pelo clippy): sem float, sem estouro implícito, sem casts que truncam.
#![forbid(unsafe_code)]

pub mod fm;
pub mod psg;
pub mod sintetizador;
mod tabelas;

pub use sintetizador::{
    AmostraFm, AmostraPsg, CICLOS_FM_POR_AMOSTRA, CICLOS_PSG_POR_AMOSTRA, Chip, EscritaSom,
    MCLK_POR_AMOSTRA_FM, MCLK_POR_AMOSTRA_PSG, MCLK_POR_CICLO_FM, MCLK_POR_CICLO_PSG,
    MCLK_POR_QUADRO_NTSC, Sintetizador,
};
