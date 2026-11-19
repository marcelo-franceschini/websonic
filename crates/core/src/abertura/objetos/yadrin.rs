//! Badnik Yadrin de Spring Yard, objeto 0x50 (REV01).
use super::super::fisica::{obj_floor_dist, obj_hit_wall, speed_to_pos};
use super::super::ram::*;
use super::super::{ErroAbertura, rb, rw, wb, wl, ww};
use super::{animate_sprite, remember_state};
use crate::memoria::Memoria;

const MAP_YAD: u32 = 0xFFBA;
const ANI_YAD: u32 = 0xFFA8;
const YAD_TIMEDELAY: u32 = 0x30;

fn s16(w: u16) -> i16 {
    i16::from_be_bytes(w.to_be_bytes())
}

fn u16_de(w: i16) -> u16 {
    u16::from_be_bytes(w.to_be_bytes())
}

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            wl(m, a.wrapping_add(OB_MAP), MAP_YAD);
            ww(m, a.wrapping_add(OB_GFX), 0x247B);
            wb(m, a.wrapping_add(OB_RENDER), 4);
            wb(m, a.wrapping_add(OB_PRIORITY), 4);
            wb(m, a.wrapping_add(OB_ACTWID), 20);
            wb(m, a.wrapping_add(OB_HEIGHT), 17);
            wb(m, a.wrapping_add(OB_WIDTH), 8);
            wb(m, a.wrapping_add(OB_COLTYPE), 0xCC);

            speed_to_pos(m, a);
            let vel = rw(m, a.wrapping_add(OB_VELY)).wrapping_add(0x38);
            ww(m, a.wrapping_add(OB_VELY), vel);
            let distancia = obj_floor_dist(m, a);
            if s16(distancia) < 0 {
                let y = rw(m, a.wrapping_add(OB_Y)).wrapping_add(distancia);
                ww(m, a.wrapping_add(OB_Y), y);
                ww(m, a.wrapping_add(OB_VELY), 0);
                wb(m, a.wrapping_add(OB_ROUTINE), 2);
                let status = rb(m, a.wrapping_add(OB_STATUS)) ^ 1;
                wb(m, a.wrapping_add(OB_STATUS), status);
            }
            Ok(())
        }
        2 => {
            match rb(m, a.wrapping_add(OB_ROUTINE_2)) {
                0 => {
                    let espera = rw(m, a.wrapping_add(YAD_TIMEDELAY)).wrapping_sub(1);
                    ww(m, a.wrapping_add(YAD_TIMEDELAY), espera);
                    if s16(espera) < 0 {
                        wb(m, a.wrapping_add(OB_ROUTINE_2), 2);
                        wb(m, a.wrapping_add(OB_ANIM), 1);
                        let status = rb(m, a.wrapping_add(OB_STATUS)) ^ 1;
                        wb(m, a.wrapping_add(OB_STATUS), status);
                        ww(
                            m,
                            a.wrapping_add(OB_VELX),
                            if status & 1 == 0 {
                                u16_de(-0x100)
                            } else {
                                0x100
                            },
                        );
                    }
                }
                2 => {
                    speed_to_pos(m, a);
                    let distancia = obj_floor_dist(m, a);
                    let d = s16(distancia);
                    if !(-8..12).contains(&d) {
                        wb(m, a.wrapping_add(OB_ROUTINE_2), 0);
                        ww(m, a.wrapping_add(YAD_TIMEDELAY), 59);
                        ww(m, a.wrapping_add(OB_VELX), 0);
                        wb(m, a.wrapping_add(OB_ANIM), 0);
                    } else {
                        let y = rw(m, a.wrapping_add(OB_Y)).wrapping_add(distancia);
                        ww(m, a.wrapping_add(OB_Y), y);

                        let idx = (a.wrapping_sub(V_OBJSPACE) / OBJECT_SIZE) as u16;
                        if (rw(m, V_FRAMECOUNT).wrapping_add(idx) & 3) == 0 {
                            let velx = s16(rw(m, a.wrapping_add(OB_VELX)));
                            let dist = if velx < 0 {
                                obj_hit_wall(m, a, 20, true)
                            } else {
                                obj_hit_wall(m, a, 20, false)
                            };
                            if s16(dist) < 0 {
                                wb(m, a.wrapping_add(OB_ROUTINE_2), 0);
                                ww(m, a.wrapping_add(YAD_TIMEDELAY), 59);
                                ww(m, a.wrapping_add(OB_VELX), 0);
                                wb(m, a.wrapping_add(OB_ANIM), 0);
                            }
                        }
                    }
                }
                _ => {
                    return Err(ErroAbertura::NaoImplementado("rotina secundaria do Yadrin"));
                }
            }
            animate_sprite(m, a, ANI_YAD)?;
            remember_state(m, a);
            Ok(())
        }
        _ => Err(ErroAbertura::NaoImplementado("rotina do Yadrin")),
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn inicializa_yadrin() {
        let mut m = Memoria::nova(vec![0; 0x1000]);
        let a0: u32 = 0xFF_D800;
        wb(&mut m, a0.wrapping_add(OB_ID), 0x50);
        wb(&mut m, a0.wrapping_add(OB_ROUTINE), 0);
        ww(&mut m, a0.wrapping_add(OB_X), 0x100);
        ww(&mut m, a0.wrapping_add(OB_Y), 0x100);

        let res = executar(&mut m, a0);
        assert!(res.is_ok());
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_COLTYPE)), 0xCC);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ACTWID)), 20);
    }
}
