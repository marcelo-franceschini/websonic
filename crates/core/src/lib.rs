//! Núcleo fiel do jogo.
//!
//! O núcleo espelha memória e rotinas do original com lógica nativa em Rust.
//! Não emula nem interpreta instruções do 68000 ou do Z80.
//! Regras do crate (checadas pelo clippy): sem ponto flutuante, sem estouro
//! implícito de inteiros, sem casts que truncam e sem coleções não determinísticas.
#![forbid(unsafe_code)]

pub mod aritmetica;
