//! Renderizador fiel em software: modelo da VDP do Mega Drive.
//!
//! Recebe só escritas nas portas da VDP (com a linha e a coluna em que ocorreram) e as
//! palavras dos DMAs. Não lê ROM nem arquivo, não interpreta instruções do 68000 ou do Z80.
//! A saída é um quadro de 320 por 224 pontos na cor de 9 bits.
//!
//! Regras do crate (as mesmas do núcleo, checadas pelo clippy): sem ponto flutuante, sem
//! estouro implícito de inteiros, sem casts que truncam e sem coleções não determinísticas.
#![forbid(unsafe_code)]

pub mod cor;
pub mod quadro;
