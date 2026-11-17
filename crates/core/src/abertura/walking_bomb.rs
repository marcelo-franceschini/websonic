//! Badnik Walking Bomb de Star Light e Scrap Brain, objeto 0x5F (REV01).
use super::super::fisica::speed_to_pos;
use super::super::ram::*;
use super::super::{rb, rw, wb, wl, ww, ErroAbertura};
use super::{animate_sprite, delete_object, display_sprite, find_free_obj, remember_state};
use crate::memoria::Memoria;

const MAP_BOMB: u32 = 0x122FC;
const ANI_BOMB: u32 = 0x122DC;
const BOM_TIME: u32 = 0x30;
const BOM_ORIG_Y: u32 = 0x34;

fn s16(w: u16) -> i16 {
    i16::from_be_bytes(w.to_be_bytes())
}

fn u16_de(w: i16) -> u16 {
    u16::from_be_bytes(w.to_be_bytes())
}

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            wl(m, a.wrapping_add(OB_MAP), MAP_BOMB);
            ww(m, a.wrapping_add(OB_GFX), 0x400);
            wb(m, a.wrapping_add(OB_RENDER), 4);
            wb(m, a.wrapping_add(OB_PRIORITY), 3);
            wb(m, a.wrapping_add(OB_ACTWID), 12);

            let sub = rb(m, a.wrapping_add(OB_SUBTYPE));
            if sub != 0 {
                wb(m, a.wrapping_add(OB_ROUTINE), sub);
                return Ok(());
            }
            wb(m, a.wrapping_add(OB_ROUTINE), 2);
            wb(m, a.wrapping_add(OB_COLTYPE), 0x9A);
            let st = rb(m, a.wrapping_add(OB_STATUS)) ^ 1;
            wb(m, a.wrapping_add(OB_STATUS), st);
            Ok(())
        }
        2 => {
            match rb(m, a.wrapping_add(OB_ROUTINE_2)) {
                0 => {
                    checar_pavio(m, a);
                    let tempo = s16(rw(m, a.wrapping_add(BOM_TIME))).wrapping_sub(1);
                    ww(m, a.wrapping_add(BOM_TIME), tempo as u16);
                    if tempo < 0 {
                        wb(m, a.wrapping_add(OB_ROUTINE_2), 2);
                        ww(m, a.wrapping_add(BOM_TIME), 25 * 60 + 35);
                        wb(m, a.wrapping_add(OB_ANIM), 1);
                        let st = rb(m, a.wrapping_add(OB_STATUS)) ^ 1;
                        wb(m, a.wrapping_add(OB_STATUS), st);
                        ww(
                            m,
                            a.wrapping_add(OB_VELX),
                            if st & 1 != 0 { u16_de(-0x10) } else { 0x10 },
                        );
                    }
                }
                2 => {
                    checar_pavio(m, a);
                    let tempo = s16(rw(m, a.wrapping_add(BOM_TIME))).wrapping_sub(1);
                    ww(m, a.wrapping_add(BOM_TIME), tempo as u16);
                    if tempo < 0 {
                        wb(m, a.wrapping_add(OB_ROUTINE_2), 0);
                        ww(m, a.wrapping_add(BOM_TIME), 3 * 60 - 1);
                        ww(m, a.wrapping_add(OB_VELX), 0);
                        wb(m, a.wrapping_add(OB_ANIM), 0);
                    } else {
                        speed_to_pos(m, a);
                    }
                }
                4 => {
                    let tempo = s16(rw(m, a.wrapping_add(BOM_TIME))).wrapping_sub(1);
                    ww(m, a.wrapping_add(BOM_TIME), tempo as u16);
                    if tempo < 0 {
                        wb(m, a.wrapping_add(OB_ID), ID_EXPLOSION_ITEM);
                        wb(m, a.wrapping_add(OB_ROUTINE), 0);
                        return Ok(());
                    }
                }
                _ => return Err(ErroAbertura::NaoImplementado("rotina secundaria da Bomba")),
            }
            animate_sprite(m, a, ANI_BOMB)?;
            remember_state(m, a);
            Ok(())
        }
        4 => {
            let tempo = s16(rw(m, a.wrapping_add(BOM_TIME))).wrapping_sub(1);
            ww(m, a.wrapping_add(BOM_TIME), tempo as u16);
            if tempo < 0 {
                let speeds: [(i16, i16); 4] = [
                    (-0x200, -0x300),
                    (-0x100, -0x200),
                    (0x200, -0x300),
                    (0x100, -0x200),
                ];
                let bx = rw(m, a.wrapping_add(OB_X));
                let by = rw(m, a.wrapping_add(BOM_ORIG_Y));
                for i in 1..4 {
                    if let Some(s) = find_free_obj(m) {
                        setup_estilhaco(m, s, bx, by, speeds[i].0, speeds[i].1);
                    }
                }
                setup_estilhaco(m, a, bx, by, speeds[0].0, speeds[0].1);
            } else {
                speed_to_pos(m, a);
                animate_sprite(m, a, ANI_BOMB)?;
                remember_state(m, a);
            }
            Ok(())
        }
        6 => {
            speed_to_pos(m, a);
            let vy = rw(m, a.wrapping_add(OB_VELY)).wrapping_add(0x18);
            ww(m, a.wrapping_add(OB_VELY), vy);
            animate_sprite(m, a, ANI_BOMB)?;
            if rb(m, a.wrapping_add(OB_RENDER)) & 0x80 == 0 {
                delete_object(m, a);
            } else {
                display_sprite(m, a);
            }
            Ok(())
        }
        _ => Err(ErroAbertura::NaoImplementado("rotina da Bomba")),
    }
}

fn checar_pavio(m: &mut Memoria, a: u32) {
    let dx = s16(rw(m, V_PLAYER.wrapping_add(OB_X)).wrapping_sub(rw(m, a.wrapping_add(OB_X))))
        .unsigned_abs();
    let dy = s16(rw(m, V_PLAYER.wrapping_add(OB_Y)).wrapping_sub(rw(m, a.wrapping_add(OB_Y))))
        .unsigned_abs();
    if dx < 96 && dy < 96 && rw(m, V_DEBUGUSE) == 0 {
        wb(m, a.wrapping_add(OB_ROUTINE_2), 4);
        ww(m, a.wrapping_add(BOM_TIME), 2 * 60 + 23);
        ww(m, a.wrapping_add(OB_VELX), 0);
        wb(m, a.wrapping_add(OB_ANIM), 2);

        if let Some(f) = find_free_obj(m) {
            let bx = rw(m, a.wrapping_add(OB_X));
            let by = rw(m, a.wrapping_add(OB_Y));
            let bst = rb(m, a.wrapping_add(OB_STATUS));
            wb(m, f.wrapping_add(OB_ID), 0x5F);
            ww(m, f.wrapping_add(OB_X), bx);
            ww(m, f.wrapping_add(OB_Y), by);
            ww(m, f.wrapping_add(BOM_ORIG_Y), by);
            wb(m, f.wrapping_add(OB_STATUS), bst);
            wb(m, f.wrapping_add(OB_SUBTYPE), 4);
            wb(m, f.wrapping_add(OB_ANIM), 3);
            let vy = if bst & 2 != 0 {
                u16_de(-0x10)
            } else {
                0x10
            };
            ww(m, f.wrapping_add(OB_VELY), vy);
            ww(m, f.wrapping_add(BOM_TIME), 2 * 60 + 23);
        }
    }
}

fn setup_estilhaco(m: &mut Memoria, obj: u32, x: u16, y: u16, vx: i16, vy: i16) {
    wb(m, obj.wrapping_add(OB_ID), 0x5F);
    wb(m, obj.wrapping_add(OB_ROUTINE), 6);
    ww(m, obj.wrapping_add(OB_X), x);
    ww(m, obj.wrapping_add(OB_Y), y);
    wb(m, obj.wrapping_add(OB_SUBTYPE), 6);
    wb(m, obj.wrapping_add(OB_ANIM), 4);
    ww(m, obj.wrapping_add(OB_VELX), u16_de(vx));
    ww(m, obj.wrapping_add(OB_VELY), u16_de(vy));
    wb(m, obj.wrapping_add(OB_COLTYPE), 0x98); // col_8x8 | col_hurt = 0x18 | 0x80 = 0x98
    let r = rb(m, obj.wrapping_add(OB_RENDER)) | 0x80;
    wb(m, obj.wrapping_add(OB_RENDER), r);
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn inicializacao_walking_bomb() {
        let mut m = Memoria::nova(vec![0; 0x1000]);
        let a0: u32 = 0xFF_D800;
        wb(&mut m, a0.wrapping_add(OB_ID), 0x5F);
        wb(&mut m, a0.wrapping_add(OB_ROUTINE), 0);
        wb(&mut m, a0.wrapping_add(OB_SUBTYPE), 0);
        ww(&mut m, a0.wrapping_add(OB_X), 0x100);
        ww(&mut m, a0.wrapping_add(OB_Y), 0x100);

        let res = executar(&mut m, a0);
        assert!(res.is_ok());
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ACTWID)), 12);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_COLTYPE)), 0x9A);
    }
}

