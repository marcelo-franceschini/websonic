//! Elevador e plataformas acionadas de Star Light, objeto 0x59 (REV01).
use super::super::plataforma_oscilante::{detectar_altura, mover_sonic_altura, sair};
use super::super::ram::*;
use super::super::{ErroAbertura, rb, rl, rw, wb, wl, ww};
use super::{delete_object, display_sprite, find_free_obj, fora_da_janela};
use crate::memoria::Memoria;

const MAP_ELEV: u32 = 0x1141C;
const ORIG_Y: u32 = 0x30;
const ORIG_X: u32 = 0x32;
const MOVED_DIST: u32 = 0x34;
const ACCEL: u32 = 0x38;
const SLOWING: u32 = 0x3A;
const HALF_DIST: u32 = 0x3C;
const SPAWN_DELAY: u32 = 0x3C;
const SPAWN_DELAY_BASE: u32 = 0x3E;

const ELEV_VAR2: [(u8, u8); 15] = [
    (0x10, 1),
    (0x20, 1),
    (0x34, 1),
    (0x10, 3),
    (0x20, 3),
    (0x34, 3),
    (0x14, 1),
    (0x24, 1),
    (0x2C, 1),
    (0x14, 3),
    (0x24, 3),
    (0x2C, 3),
    (0x20, 5),
    (0x20, 7),
    (0x30, 9),
];

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let mut rotina = rb(m, a.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        let sub = rb(m, a.wrapping_add(OB_SUBTYPE));
        if sub & 0x80 != 0 {
            wb(m, a.wrapping_add(OB_ROUTINE), 6);
            let interval = u16::from(sub & 0x7F).wrapping_mul(6);
            ww(m, a.wrapping_add(SPAWN_DELAY), interval);
            ww(m, a.wrapping_add(SPAWN_DELAY_BASE), interval);
            return Ok(());
        }
        rotina = 2;
        wb(m, a.wrapping_add(OB_ROUTINE), rotina);
        let idx = usize::from(sub & 0x0F);
        let (half_base, act_type) = if idx < ELEV_VAR2.len() {
            ELEV_VAR2[idx]
        } else {
            ELEV_VAR2[0]
        };
        let half_dist = u16::from(half_base).wrapping_mul(4);
        ww(m, a.wrapping_add(HALF_DIST), half_dist);
        wb(m, a.wrapping_add(OB_SUBTYPE), act_type);

        wb(m, a.wrapping_add(OB_ACTWID), 40);
        wb(m, a.wrapping_add(OB_FRAME), 0);
        wl(m, a.wrapping_add(OB_MAP), MAP_ELEV);
        ww(m, a.wrapping_add(OB_GFX), 0x4000);
        wb(m, a.wrapping_add(OB_RENDER), 4);
        wb(m, a.wrapping_add(OB_PRIORITY), 4);
        let ox = rw(m, a.wrapping_add(OB_X));
        let oy = rw(m, a.wrapping_add(OB_Y));
        ww(m, a.wrapping_add(ORIG_X), ox);
        ww(m, a.wrapping_add(ORIG_Y), oy);
    }
    match rotina {
        2 => {
            mover(m, a);
            detectar_altura(m, a, 8);
        }
        4 => {
            sair(m, a);
            let anterior = rw(m, a.wrapping_add(OB_X));
            mover(m, a);
            if rb(m, a.wrapping_add(OB_ID)) != 0 {
                mover_sonic_altura(m, a, anterior, 9);
            }
        }
        6 => {
            let delay = rw(m, a.wrapping_add(SPAWN_DELAY)).wrapping_sub(1);
            if delay == 0 {
                let base = rw(m, a.wrapping_add(SPAWN_DELAY_BASE));
                ww(m, a.wrapping_add(SPAWN_DELAY), base);
                if let Some(a1) = find_free_obj(m) {
                    let sx = rw(m, a.wrapping_add(OB_X));
                    let sy = rw(m, a.wrapping_add(OB_Y));
                    wb(m, a1.wrapping_add(OB_ID), 0x59);
                    ww(m, a1.wrapping_add(OB_X), sx);
                    ww(m, a1.wrapping_add(OB_Y), sy);
                    wb(m, a1.wrapping_add(OB_SUBTYPE), 0x0E);
                }
            } else {
                ww(m, a.wrapping_add(SPAWN_DELAY), delay);
            }
            let ox = rw(m, a.wrapping_add(ORIG_X));
            if fora_da_janela(m, ox) {
                delete_object(m, a);
            }
            return Ok(());
        }
        _ => return Err(ErroAbertura::NaoImplementado("rotina do elevador")),
    }
    if rb(m, a.wrapping_add(OB_ID)) != 0 {
        let ox = rw(m, a.wrapping_add(ORIG_X));
        if fora_da_janela(m, ox) {
            delete_object(m, a);
        } else {
            display_sprite(m, a);
        }
    }
    Ok(())
}

fn mover(m: &mut Memoria, a: u32) {
    let act = rb(m, a.wrapping_add(OB_SUBTYPE)) & 0x0F;
    match act {
        0 => {}
        1 | 3 | 5 | 7 => {
            if rb(m, a.wrapping_add(OB_ROUTINE)) == 4 {
                wb(m, a.wrapping_add(OB_SUBTYPE), act.wrapping_add(1));
            }
        }
        2 => {
            avancar(m, a);
            let dist = rw(m, a.wrapping_add(MOVED_DIST));
            let oy = rw(m, a.wrapping_add(ORIG_Y));
            ww(m, a.wrapping_add(OB_Y), oy.wrapping_sub(dist));
        }
        4 => {
            avancar(m, a);
            let dist = rw(m, a.wrapping_add(MOVED_DIST));
            let oy = rw(m, a.wrapping_add(ORIG_Y));
            ww(m, a.wrapping_add(OB_Y), oy.wrapping_add(dist));
        }
        6 => {
            avancar(m, a);
            let dist = rw(m, a.wrapping_add(MOVED_DIST));
            let oy = rw(m, a.wrapping_add(ORIG_Y));
            let ox = rw(m, a.wrapping_add(ORIG_X));
            ww(m, a.wrapping_add(OB_Y), oy.wrapping_sub(dist >> 1));
            ww(m, a.wrapping_add(OB_X), ox.wrapping_add(dist));
        }
        8 => {
            avancar(m, a);
            let dist = rw(m, a.wrapping_add(MOVED_DIST));
            let oy = rw(m, a.wrapping_add(ORIG_Y));
            let ox = rw(m, a.wrapping_add(ORIG_X));
            ww(m, a.wrapping_add(OB_Y), oy.wrapping_sub(dist >> 1));
            ww(m, a.wrapping_add(OB_X), ox.wrapping_sub(dist));
        }
        9 => {
            avancar(m, a);
            let dist = rw(m, a.wrapping_add(MOVED_DIST));
            let oy = rw(m, a.wrapping_add(ORIG_Y));
            ww(m, a.wrapping_add(OB_Y), oy.wrapping_sub(dist));
            if rb(m, a.wrapping_add(OB_SUBTYPE)) == 0 {
                if rb(m, a.wrapping_add(OB_STATUS)) & STATUS_ON_OBJECT != 0 {
                    let mut st = rb(m, V_PLAYER.wrapping_add(OB_STATUS));
                    st |= STATUS_IN_AIR;
                    st &= !STATUS_ON_OBJECT;
                    wb(m, V_PLAYER.wrapping_add(OB_STATUS), st);
                    wb(m, V_PLAYER.wrapping_add(OB_ROUTINE), 2);
                }
                delete_object(m, a);
            }
        }
        _ => {}
    }
}

fn avancar(m: &mut Memoria, a: u32) {
    let mut accel = rw(m, a.wrapping_add(ACCEL));
    let slowing = rb(m, a.wrapping_add(SLOWING)) != 0;
    if slowing {
        if accel != 0 {
            accel = accel.saturating_sub(0x10);
        }
    } else if accel < 0x800 {
        accel = accel.saturating_add(0x10);
    }
    ww(m, a.wrapping_add(ACCEL), accel);
    let shift = u32::from(accel) << 8;
    let dist_long = rl(m, a.wrapping_add(MOVED_DIST)).wrapping_add(shift);
    wl(m, a.wrapping_add(MOVED_DIST), dist_long);
    let int_dist = (dist_long >> 16) as u16;
    let half = rw(m, a.wrapping_add(HALF_DIST));
    if int_dist > half {
        wb(m, a.wrapping_add(SLOWING), 1);
    }
    if int_dist >= half.wrapping_mul(2) {
        wb(m, a.wrapping_add(OB_SUBTYPE), 0);
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn inicializacao_elevador_slz() {
        let mut m = Memoria::nova(vec![0; 0x1000]);
        let a0: u32 = 0xFF_D800;
        wb(&mut m, a0.wrapping_add(OB_ID), 0x59);
        wb(&mut m, a0.wrapping_add(OB_ROUTINE), 0);
        wb(&mut m, a0.wrapping_add(OB_SUBTYPE), 0);
        ww(&mut m, a0.wrapping_add(OB_X), 0x100);
        ww(&mut m, a0.wrapping_add(OB_Y), 0x100);

        let res = executar(&mut m, a0);
        assert!(res.is_ok());
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ACTWID)), 40);
        assert_eq!(rw(&mut m, a0.wrapping_add(ORIG_X)), 0x100);
        assert_eq!(rw(&mut m, a0.wrapping_add(ORIG_Y)), 0x100);
    }
}
