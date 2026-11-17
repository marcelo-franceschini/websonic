//! Badnik Roller de Spring Yard, objeto 0x43 (REV01).
use super::super::fisica::{obj_floor_dist, speed_to_pos};
use super::super::ram::*;
use super::super::{ErroAbertura, rb, rw, wb, wl, ww};
use super::{animate_sprite, remember_state};
use crate::memoria::Memoria;

const MAP_ROLL: u32 = 0xE830;
const ANI_ROLL: u32 = 0xE818;
const ROLL_WAIT: u32 = 0x30;
const ROLL_FLAGS: u32 = 0x32;

fn s16(w: u16) -> i16 {
    i16::from_be_bytes(w.to_be_bytes())
}

fn u16_de(w: i16) -> u16 {
    u16::from_be_bytes(w.to_be_bytes())
}

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            wb(m, a.wrapping_add(OB_HEIGHT), 14);
            wb(m, a.wrapping_add(OB_WIDTH), 8);
            speed_to_pos(m, a);
            let vy = rw(m, a.wrapping_add(OB_VELY)).wrapping_add(0x38);
            ww(m, a.wrapping_add(OB_VELY), vy);
            let dist = obj_floor_dist(m, a);
            if s16(dist) < 0 {
                let y = rw(m, a.wrapping_add(OB_Y)).wrapping_add(dist);
                ww(m, a.wrapping_add(OB_Y), y);
                ww(m, a.wrapping_add(OB_VELY), 0);
                wb(m, a.wrapping_add(OB_ROUTINE), 2);
                wl(m, a.wrapping_add(OB_MAP), MAP_ROLL);
                ww(m, a.wrapping_add(OB_GFX), 0x4B8);
                wb(m, a.wrapping_add(OB_RENDER), 4);
                wb(m, a.wrapping_add(OB_PRIORITY), 4);
                wb(m, a.wrapping_add(OB_ACTWID), 16);
            }
            Ok(())
        }
        2 => {
            match rb(m, a.wrapping_add(OB_ROUTINE_2)) {
                0 => {
                    let sx = rw(m, V_PLAYER.wrapping_add(OB_X));
                    if sx < 256 {
                        return Ok(());
                    }
                    let rx = rw(m, a.wrapping_add(OB_X));
                    let diff = sx.wrapping_sub(256).wrapping_sub(rx);
                    if s16(diff) < 0 {
                        return Ok(());
                    }
                    wb(m, a.wrapping_add(OB_ROUTINE_2), 4);
                    wb(m, a.wrapping_add(OB_ANIM), 2);
                    ww(m, a.wrapping_add(OB_VELX), 0x700);
                    wb(m, a.wrapping_add(OB_COLTYPE), 0x8E);
                    return Ok(());
                }
                2 => {
                    if rb(m, a.wrapping_add(OB_ANIM)) == 2 {
                        wb(m, a.wrapping_add(OB_ROUTINE_2), 4);
                    } else {
                        let wait = rw(m, a.wrapping_add(ROLL_WAIT)).wrapping_sub(1);
                        ww(m, a.wrapping_add(ROLL_WAIT), wait);
                        if s16(wait) < 0 {
                            wb(m, a.wrapping_add(OB_ANIM), 1);
                            ww(m, a.wrapping_add(OB_VELX), 0x700);
                            wb(m, a.wrapping_add(OB_COLTYPE), 0x8E);
                        }
                    }
                }
                4 => {
                    let flags = rb(m, a.wrapping_add(ROLL_FLAGS));
                    if flags & 0x80 == 0 {
                        let sx = rw(m, V_PLAYER.wrapping_add(OB_X));
                        let rx = rw(m, a.wrapping_add(OB_X));
                        let diff = sx.wrapping_sub(48).wrapping_sub(rx);
                        if s16(diff) > 0 {
                            wb(m, a.wrapping_add(OB_ANIM), 0);
                            wb(m, a.wrapping_add(OB_COLTYPE), 0x0E);
                            ww(m, a.wrapping_add(OB_VELX), 0);
                            ww(m, a.wrapping_add(ROLL_WAIT), 120);
                            wb(m, a.wrapping_add(OB_ROUTINE_2), 2);
                            wb(m, a.wrapping_add(ROLL_FLAGS), flags | 0x80);
                        }
                    }
                    speed_to_pos(m, a);
                    let dist = obj_floor_dist(m, a);
                    let d = s16(dist);
                    if !(-8..12).contains(&d) {
                        wb(m, a.wrapping_add(OB_ROUTINE_2), 6);
                        let sf = rb(m, a.wrapping_add(ROLL_FLAGS));
                        wb(m, a.wrapping_add(ROLL_FLAGS), sf | 1);
                        if sf & 1 != 0 {
                            ww(m, a.wrapping_add(OB_VELY), u16_de(-0x600));
                        }
                    } else {
                        let y = rw(m, a.wrapping_add(OB_Y)).wrapping_add(dist);
                        ww(m, a.wrapping_add(OB_Y), y);
                    }
                }
                6 => {
                    speed_to_pos(m, a);
                    let vy = rw(m, a.wrapping_add(OB_VELY)).wrapping_add(0x38);
                    ww(m, a.wrapping_add(OB_VELY), vy);
                    if s16(vy) >= 0 {
                        let dist = obj_floor_dist(m, a);
                        if s16(dist) < 0 {
                            let y = rw(m, a.wrapping_add(OB_Y)).wrapping_add(dist);
                            ww(m, a.wrapping_add(OB_Y), y);
                            wb(m, a.wrapping_add(OB_ROUTINE_2), 4);
                            ww(m, a.wrapping_add(OB_VELY), 0);
                        }
                    }
                }
                _ => {
                    return Err(ErroAbertura::NaoImplementado("rotina secundaria do Roller"));
                }
            }
            animate_sprite(m, a, ANI_ROLL)?;
            remember_state(m, a);
            Ok(())
        }
        _ => Err(ErroAbertura::NaoImplementado("rotina do Roller")),
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn inicializacao_roller() {
        let mut m = Memoria::nova(vec![0; 0x1000]);
        let a0: u32 = 0xFF_D800;
        wb(&mut m, a0.wrapping_add(OB_ID), 0x43);
        wb(&mut m, a0.wrapping_add(OB_ROUTINE), 0);
        ww(&mut m, a0.wrapping_add(OB_X), 0x200);
        ww(&mut m, a0.wrapping_add(OB_Y), 0x200);

        let res = executar(&mut m, a0);
        assert!(res.is_ok());
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_WIDTH)), 8);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_HEIGHT)), 14);
    }
}
