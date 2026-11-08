//! Gabarito independente das operações do 68000, pelas fórmulas booleanas do manual.
//!
//! Fonte: Motorola, M68000 Family Programmer's Reference Manual, seções de ADD, SUB, NEG e CMP
//! (colunas "V" e "C" da descrição dos códigos de condição). Sm, Dm e Rm são o bit mais alto
//! da fonte, do destino e do resultado. A implementação do núcleo usa aritmética em tipo mais
//! largo e não estas fórmulas, então uma concordância entre as duas é prova de verdade.

use sonic1_core::memoria::Tamanho;
use sonic1_core::ops68k::Ccr;

pub fn mascara(t: Tamanho) -> u32 {
    match t {
        Tamanho::Byte => 0xFF,
        Tamanho::Word => 0xFFFF,
        Tamanho::Long => 0xFFFF_FFFF,
    }
}

fn msb(v: u32, t: Tamanho) -> bool {
    v & (mascara(t) ^ (mascara(t) >> 1)) != 0
}

/// ADD/ADDX: V = Sm Dm ~Rm + ~Sm ~Dm Rm; C = Sm Dm + ~Rm Dm + Sm ~Rm.
pub fn gabarito_add(
    t: Tamanho,
    s: u32,
    d: u32,
    x: bool,
    com_x: bool,
    z_in: bool,
    acumula_z: bool,
) -> (u32, Ccr) {
    let m = mascara(t);
    let r = s.wrapping_add(d).wrapping_add(u32::from(com_x && x)) & m;
    let (sm, dm, rm) = (msb(s, t), msb(d, t), msb(r, t));
    let v = (sm && dm && !rm) || (!sm && !dm && rm);
    let c = (sm && dm) || (!rm && dm) || (sm && !rm);
    let z = if acumula_z { z_in && r == 0 } else { r == 0 };
    (
        r,
        Ccr {
            x: c,
            n: rm,
            z,
            v,
            c,
        },
    )
}

/// SUB/SUBX/CMP: V = ~Sm Dm ~Rm + Sm ~Dm Rm; C = Sm ~Dm + Rm ~Dm + Sm Rm.
pub fn gabarito_sub(
    t: Tamanho,
    s: u32,
    d: u32,
    x: bool,
    com_x: bool,
    z_in: bool,
    acumula_z: bool,
) -> (u32, Ccr) {
    let m = mascara(t);
    let r = d.wrapping_sub(s).wrapping_sub(u32::from(com_x && x)) & m;
    let (sm, dm, rm) = (msb(s, t), msb(d, t), msb(r, t));
    let v = (!sm && dm && !rm) || (sm && !dm && rm);
    let c = (sm && !dm) || (rm && !dm) || (sm && rm);
    let z = if acumula_z { z_in && r == 0 } else { r == 0 };
    (
        r,
        Ccr {
            x: c,
            n: rm,
            z,
            v,
            c,
        },
    )
}

/// NEG: V = Dm Rm; C = Dm + Rm.
pub fn gabarito_neg(t: Tamanho, d: u32) -> (u32, Ccr) {
    let r = 0u32.wrapping_sub(d) & mascara(t);
    let (dm, rm) = (msb(d, t), msb(r, t));
    let c = dm || rm;
    (
        r,
        Ccr {
            x: c,
            n: rm,
            z: r == 0,
            v: dm && rm,
            c,
        },
    )
}
