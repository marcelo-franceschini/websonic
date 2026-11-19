//! Escada acionavel de Star Light, objeto 0x5B (REV01).
use super::super::ram::*;
use super::super::{ErroAbertura, rb, rl, rw, wb, wl, ww};
use super::{delete_object, display_sprite, find_free_obj, fora_da_janela, solid_object};
use crate::memoria::Memoria;

const MAP_STAIR: u32 = 0x11710;
const STAIR_ORIG_X: u32 = 0x30;
const STAIR_ORIG_Y: u32 = 0x32;
const STAIR_DELAY: u32 = 0x34;
const STAIR_TOUCH: u32 = 0x36;
const STAIR_PARENT_Y_INDEX: u32 = 0x37;
const STAIR_CHILDREN_Y: u32 = 0x38;
const STAIR_PARENT: u32 = 0x3C;

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let mut rotina = rb(m, a.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        rotina = 2;
        wb(m, a.wrapping_add(OB_ROUTINE), rotina);

        let xflip = rb(m, a.wrapping_add(OB_STATUS)) & 1 != 0;
        let mut y_idx = if xflip { 0x3Bu8 } else { 0x38u8 };
        let y_step: i8 = if xflip { -1 } else { 1 };

        let base_x = rw(m, a.wrapping_add(OB_X));
        let base_y = rw(m, a.wrapping_add(OB_Y));
        let sub = rb(m, a.wrapping_add(OB_SUBTYPE));

        setup_bloco(m, a, 2, (base_x, base_y), sub, y_idx, a);
        y_idx = y_idx.cast_signed().wrapping_add(y_step).cast_unsigned();

        for i in 1..=3u16 {
            let Some(filho) = find_free_obj(m) else { break };
            let bx = base_x.wrapping_add(i.wrapping_mul(32));
            setup_bloco(m, filho, 4, (bx, base_y), sub, y_idx, a);
            y_idx = y_idx.cast_signed().wrapping_add(y_step).cast_unsigned();
        }
    }

    if rotina == 2 {
        mover_pai(m, a);
    }

    let parent = if rotina == 2 {
        a
    } else {
        rl(m, a.wrapping_add(STAIR_PARENT))
    };
    let y_idx = u32::from(rb(m, a.wrapping_add(STAIR_PARENT_Y_INDEX)));
    let dy = u16::from(rb(m, parent.wrapping_add(y_idx)));
    let oy = rw(m, a.wrapping_add(STAIR_ORIG_Y));
    ww(m, a.wrapping_add(OB_Y), oy.wrapping_add(dy));

    let col = solid_object(m, a, 16, 16, 17)?;
    if col < 0 {
        wb(m, parent.wrapping_add(STAIR_TOUCH), col.cast_unsigned());
    }
    if rb(m, a.wrapping_add(OB_STATUS)) & STATUS_ON_OBJECT != 0 {
        wb(m, parent.wrapping_add(STAIR_TOUCH), 1);
    }

    let ox = rw(m, a.wrapping_add(STAIR_ORIG_X));
    if fora_da_janela(m, ox) {
        delete_object(m, a);
    } else {
        display_sprite(m, a);
    }
    Ok(())
}

fn setup_bloco(
    m: &mut Memoria,
    bloco: u32,
    rotina: u8,
    (x, y): (u16, u16),
    sub: u8,
    y_idx: u8,
    pai: u32,
) {
    wb(m, bloco.wrapping_add(OB_ID), 0x5B);
    wb(m, bloco.wrapping_add(OB_ROUTINE), rotina);
    wl(m, bloco.wrapping_add(OB_MAP), MAP_STAIR);
    ww(m, bloco.wrapping_add(OB_GFX), 0x4000);
    wb(m, bloco.wrapping_add(OB_RENDER), 4);
    wb(m, bloco.wrapping_add(OB_PRIORITY), 3);
    wb(m, bloco.wrapping_add(OB_ACTWID), 16);
    wb(m, bloco.wrapping_add(OB_SUBTYPE), sub);
    ww(m, bloco.wrapping_add(OB_X), x);
    ww(m, bloco.wrapping_add(OB_Y), y);
    let orig_x = rw(m, pai.wrapping_add(OB_X));
    ww(m, bloco.wrapping_add(STAIR_ORIG_X), orig_x);
    ww(m, bloco.wrapping_add(STAIR_ORIG_Y), y);
    wb(m, bloco.wrapping_add(STAIR_PARENT_Y_INDEX), y_idx);
    wl(m, bloco.wrapping_add(STAIR_PARENT), pai);
}

fn mover_pai(m: &mut Memoria, a: u32) {
    let tipo = rb(m, a.wrapping_add(OB_SUBTYPE)) & 7;
    match tipo {
        0 => {
            let delay = rw(m, a.wrapping_add(STAIR_DELAY));
            if delay != 0 {
                let d = delay.wrapping_sub(1);
                ww(m, a.wrapping_add(STAIR_DELAY), d);
                if d == 0 {
                    wb(m, a.wrapping_add(OB_SUBTYPE), 1);
                }
            } else if rb(m, a.wrapping_add(STAIR_TOUCH)) == 1 {
                ww(m, a.wrapping_add(STAIR_DELAY), 30);
            }
        }
        2 => {
            let delay = rw(m, a.wrapping_add(STAIR_DELAY));
            if delay != 0 {
                let d = delay.wrapping_sub(1);
                ww(m, a.wrapping_add(STAIR_DELAY), d);
                if d == 0 {
                    wb(m, a.wrapping_add(OB_SUBTYPE), 3);
                } else {
                    let seed = u8::try_from((d >> 2) & 1).unwrap_or(0);
                    wb(m, a.wrapping_add(STAIR_CHILDREN_Y), seed);
                    wb(
                        m,
                        a.wrapping_add(STAIR_CHILDREN_Y.wrapping_add(1)),
                        seed ^ 1,
                    );
                    wb(m, a.wrapping_add(STAIR_CHILDREN_Y.wrapping_add(2)), seed);
                    wb(
                        m,
                        a.wrapping_add(STAIR_CHILDREN_Y.wrapping_add(3)),
                        seed ^ 1,
                    );
                }
            } else if rb(m, a.wrapping_add(STAIR_TOUCH)).cast_signed() < 0 {
                ww(m, a.wrapping_add(STAIR_DELAY), 60);
            }
        }
        1 | 3 => {
            let mut y0 = rb(m, a.wrapping_add(STAIR_CHILDREN_Y));
            if y0 < 128 {
                y0 = y0.wrapping_add(1);
                wb(m, a.wrapping_add(STAIR_CHILDREN_Y), y0);
                let d1 = u16::from(y0);
                let y1 = u8::try_from(d1.wrapping_mul(3) / 4).unwrap_or(0);
                let y2 = u8::try_from(d1 / 2).unwrap_or(0);
                let y3 = u8::try_from(d1 / 4).unwrap_or(0);
                wb(m, a.wrapping_add(STAIR_CHILDREN_Y.wrapping_add(1)), y1);
                wb(m, a.wrapping_add(STAIR_CHILDREN_Y.wrapping_add(2)), y2);
                wb(m, a.wrapping_add(STAIR_CHILDREN_Y.wrapping_add(3)), y3);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn inicializacao_staircase_slz() {
        let mut m = Memoria::nova(vec![0; 0x1000]);
        let a0: u32 = 0xFF_D800;
        wb(&mut m, a0.wrapping_add(OB_ID), 0x5B);
        wb(&mut m, a0.wrapping_add(OB_ROUTINE), 0);
        ww(&mut m, a0.wrapping_add(OB_X), 0x100);
        ww(&mut m, a0.wrapping_add(OB_Y), 0x100);

        let res = executar(&mut m, a0);
        assert!(res.is_ok());
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ACTWID)), 16);
    }
}
