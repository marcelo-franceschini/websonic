//! Deslocamentos e rotações (ASL, ASR, LSL, LSR, ROL, ROR, ROXL, ROXR).
//!
//! Feitos um bit por vez: a definição das flags de cada instrução é a de um passo, repetida
//! `contagem` vezes. A contagem em registrador vale módulo 64; a imediata vai de 1 a 8.

use super::{Ccr, Resultado, bit_alto, mascara};
use crate::memoria::Tamanho;

/// Qual deslocamento ou rotação.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Deslocamento {
    AslEsquerda,
    AsrDireita,
    LslEsquerda,
    LsrDireita,
    RolEsquerda,
    RorDireita,
    RoxlEsquerda,
    RoxrDireita,
}

/// Aplica o deslocamento `contagem` vezes (já módulo 64 se veio de registrador).
pub fn deslocar(tipo: Deslocamento, t: Tamanho, contagem: u32, dst: u32, ccr: Ccr) -> Resultado {
    let msb = bit_alto(t);
    let mask = mascara(t);
    let mut v = u64::from(dst) & mask;
    let mut x = ccr.x;
    let mut c = false;
    let mut mudou_sinal = false;
    // Com contagem 0, C vale 0 (e para ROX, C vale X); X só muda se houve deslocamento.
    let rox = matches!(tipo, Deslocamento::RoxlEsquerda | Deslocamento::RoxrDireita);
    if contagem == 0 && rox {
        c = ccr.x;
    }
    let esquerda = matches!(
        tipo,
        Deslocamento::AslEsquerda
            | Deslocamento::LslEsquerda
            | Deslocamento::RolEsquerda
            | Deslocamento::RoxlEsquerda
    );
    for _ in 0..contagem {
        if esquerda {
            let saiu = v & msb != 0;
            let entra = match tipo {
                Deslocamento::RolEsquerda => saiu,
                Deslocamento::RoxlEsquerda => x,
                _ => false,
            };
            v = ((v << 1) & mask) | u64::from(entra);
            if tipo == Deslocamento::AslEsquerda && (v & msb != 0) != saiu {
                mudou_sinal = true;
            }
            c = saiu;
        } else {
            let saiu = v & 1 != 0;
            let entra = match tipo {
                Deslocamento::AsrDireita => v & msb != 0,
                Deslocamento::RorDireita => saiu,
                Deslocamento::RoxrDireita => x,
                _ => false,
            };
            v >>= 1;
            if entra {
                v |= msb;
            }
            c = saiu;
        }
        // X acompanha C em tudo menos ROL e ROR.
        if !matches!(tipo, Deslocamento::RolEsquerda | Deslocamento::RorDireita) {
            x = c;
        }
    }
    Resultado {
        valor: u32::try_from(v & mask).unwrap_or(0),
        ccr: Ccr {
            x,
            n: v & msb != 0,
            z: v == 0,
            v: mudou_sinal,
            c,
        },
    }
}
