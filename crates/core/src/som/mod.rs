//! Som do Sonic 1 (Etapa 6): barramento e programa do Z80, e o driver do 68000.
//!
//! Nada aqui sintetiza áudio. O módulo produz as escritas nos chips de som, na ordem em que o
//! original as faz, e mantém o estado do driver na RAM espelhada.

pub mod z80;
