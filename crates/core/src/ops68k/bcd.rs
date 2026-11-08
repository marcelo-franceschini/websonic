//! Aritmética decimal codificada em binário (ABCD, SBCD, NBCD), só em byte.
//!
//! N e V são indefinidos no manual; seguem o oráculo.

use super::{Ccr, Resultado};

/// ABCD: `dst + src + X` em BCD.
pub fn abcd(src: u8, dst: u8, ccr: Ccr) -> Resultado {
    let x = u8::from(ccr.x);
    let (parcial, vai1) = src.overflowing_add(dst);
    let (soma, vai2) = parcial.overflowing_add(x);
    let vai_um_do_nibble = (src & 0x0F).wrapping_add(dst & 0x0F).wrapping_add(x) >= 0x10;
    let carry_bruto = vai1 || vai2;
    let mut ajuste = 0u8;
    if vai_um_do_nibble || soma & 0x0F > 9 {
        ajuste = ajuste.wrapping_add(0x06);
    }
    if soma > 0x99 || carry_bruto {
        ajuste = ajuste.wrapping_add(0x60);
    }
    let (corrigido, vai3) = soma.overflowing_add(ajuste);
    let vai_do_bit6 = (soma & 0x7F).wrapping_add(ajuste & 0x7F) >= 0x80;
    let carry = carry_bruto || vai3;
    Resultado {
        valor: u32::from(corrigido),
        ccr: Ccr {
            x: carry,
            n: corrigido & 0x80 != 0,
            z: ccr.z && corrigido == 0,
            v: vai_do_bit6 != vai3,
            c: carry,
        },
    }
}

fn subtrair_bcd(esq: u8, dir: u8, ccr: Ccr) -> Resultado {
    let x = u8::from(ccr.x);
    let (parcial, em1) = esq.overflowing_sub(dir);
    let (dif, em2) = parcial.overflowing_sub(x);
    let emprestimo_bruto = em1 || em2;
    let mut ajuste = 0u8;
    if (esq & 0x0F) < (dir & 0x0F).wrapping_add(x) {
        ajuste = ajuste.wrapping_add(0x06);
    }
    if emprestimo_bruto {
        ajuste = ajuste.wrapping_add(0x60);
    }
    let (corrigido, em3) = dif.overflowing_sub(ajuste);
    let emprestimo_do_bit6 = (dif & 0x7F) < (ajuste & 0x7F);
    let emprestimo = emprestimo_bruto || em3;
    Resultado {
        valor: u32::from(corrigido),
        ccr: Ccr {
            x: emprestimo,
            n: corrigido & 0x80 != 0,
            z: ccr.z && corrigido == 0,
            v: emprestimo_do_bit6 != em3,
            c: emprestimo,
        },
    }
}

/// SBCD: `dst - src - X` em BCD.
pub fn sbcd(src: u8, dst: u8, ccr: Ccr) -> Resultado {
    subtrair_bcd(dst, src, ccr)
}

/// NBCD: `0 - dst - X` em BCD.
pub fn nbcd(dst: u8, ccr: Ccr) -> Resultado {
    subtrair_bcd(0, dst, ccr)
}
