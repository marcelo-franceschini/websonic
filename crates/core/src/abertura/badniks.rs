//! Newtron, Ball Hog e seus projéteis, conforme as rotinas da REV01.
use super::fisica::{obj_floor_dist_and_angle, object_fall, speed_to_pos};
use super::objetos::{
    animate_sprite, delete_object, display_sprite, find_free_obj, remember_state,
};
use super::ram::*;
use super::{ErroAbertura, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;

const NEWT_MAP: u32 = 0xE5D0;
const NEWT_ANI: u32 = 0xE5A8;
const HOG_MAP: u32 = 0x94E0;
const HOG_ANI: u32 = 0x94C6;
const FLAG: u32 = 0x32;
const TEMPO: u32 = 0x30;

fn s16(x: u16) -> i16 {
    i16::from_be_bytes(x.to_be_bytes())
}
fn bits(x: i16) -> u16 {
    u16::from_be_bytes(x.to_be_bytes())
}

fn orientar(m: &mut Memoria, a: u32) -> u16 {
    let sonic = rw(m, V_PLAYER.wrapping_add(OB_X));
    let x = rw(m, a.wrapping_add(OB_X));
    let distancia = sonic.wrapping_sub(x);
    let status = rb(m, a.wrapping_add(OB_STATUS));
    if sonic < x {
        wb(m, a.wrapping_add(OB_STATUS), status & !1);
        distancia.wrapping_neg()
    } else {
        wb(m, a.wrapping_add(OB_STATUS), status | 1);
        distancia
    }
}

pub(super) fn newtron(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            wb(m, a.wrapping_add(OB_ROUTINE), 2);
            wl(m, a.wrapping_add(OB_MAP), NEWT_MAP);
            ww(m, a.wrapping_add(OB_GFX), 0x49B);
            wb(m, a.wrapping_add(OB_RENDER), 4);
            wb(m, a.wrapping_add(OB_PRIORITY), 4);
            wb(m, a.wrapping_add(OB_ACTWID), 20);
            wb(m, a.wrapping_add(OB_HEIGHT), 16);
            wb(m, a.wrapping_add(OB_WIDTH), 8);
        }
        2 => {}
        4 => {
            delete_object(m, a);
            return Ok(());
        }
        _ => return Err(ErroAbertura::NaoImplementado("rotina de Newtron")),
    }
    match rb(m, a.wrapping_add(OB_ROUTINE_2)) {
        0 => {
            if orientar(m, a) < 128 {
                wb(m, a.wrapping_add(OB_ROUTINE_2), 2);
                wb(m, a.wrapping_add(OB_ANIM), 1);
                if rb(m, a.wrapping_add(OB_SUBTYPE)) != 0 {
                    ww(m, a.wrapping_add(OB_GFX), 0x249B);
                    wb(m, a.wrapping_add(OB_ROUTINE_2), 8);
                    wb(m, a.wrapping_add(OB_ANIM), 4);
                }
            }
        }
        2 => {
            if rb(m, a.wrapping_add(OB_FRAME)) < 4 {
                orientar(m, a);
            } else {
                object_fall(m, a);
                let x = rw(m, a.wrapping_add(OB_X));
                let (dist, _) = obj_floor_dist_and_angle(m, a, x);
                if dist < 0 {
                    let y = rw(m, a.wrapping_add(OB_Y)).wrapping_add(bits(dist));
                    ww(m, a.wrapping_add(OB_Y), y);
                    ww(m, a.wrapping_add(OB_VELY), 0);
                    wb(m, a.wrapping_add(OB_ROUTINE_2), 4);
                    wb(m, a.wrapping_add(OB_ANIM), 2);
                    wb(m, a.wrapping_add(OB_COLTYPE), 0x0D);
                    let status = rb(m, a.wrapping_add(OB_STATUS));
                    ww(
                        m,
                        a.wrapping_add(OB_VELX),
                        if status & 1 != 0 { 0x200 } else { 0xFE00 },
                    );
                }
            }
        }
        4 => {
            speed_to_pos(m, a);
            let x = rw(m, a.wrapping_add(OB_X));
            let (dist, _) = obj_floor_dist_and_angle(m, a, x);
            if !(-8..12).contains(&dist) {
                wb(m, a.wrapping_add(OB_ROUTINE_2), 6);
            } else {
                let y = rw(m, a.wrapping_add(OB_Y)).wrapping_add(bits(dist));
                ww(m, a.wrapping_add(OB_Y), y);
            }
        }
        6 => speed_to_pos(m, a),
        8 => {
            let frame = rb(m, a.wrapping_add(OB_FRAME));
            if frame == 1 {
                wb(m, a.wrapping_add(OB_COLTYPE), 0x0C);
            }
            if frame == 2 && rb(m, a.wrapping_add(FLAG)) == 0 {
                wb(m, a.wrapping_add(FLAG), 1);
                if let Some(b) = find_free_obj(m) {
                    let status = rb(m, a.wrapping_add(OB_STATUS));
                    let x = rw(m, a.wrapping_add(OB_X));
                    let y = rw(m, a.wrapping_add(OB_Y));
                    wb(m, b.wrapping_add(OB_ID), 0x23);
                    ww(
                        m,
                        b.wrapping_add(OB_X),
                        if status & 1 != 0 {
                            x.wrapping_add(20)
                        } else {
                            x.wrapping_sub(20)
                        },
                    );
                    ww(m, b.wrapping_add(OB_Y), y.wrapping_sub(8));
                    ww(
                        m,
                        b.wrapping_add(OB_VELX),
                        if status & 1 != 0 { 0x200 } else { 0xFE00 },
                    );
                    wb(m, b.wrapping_add(OB_STATUS), status);
                    wb(m, b.wrapping_add(OB_SUBTYPE), 1);
                }
            }
        }
        _ => {
            return Err(ErroAbertura::NaoImplementado(
                "rotina secundaria de Newtron",
            ));
        }
    }
    animate_sprite(m, a, NEWT_ANI)?;
    remember_state(m, a);
    Ok(())
}

pub(super) fn ball_hog(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            wb(m, a.wrapping_add(OB_HEIGHT), 19);
            wb(m, a.wrapping_add(OB_WIDTH), 8);
            wl(m, a.wrapping_add(OB_MAP), HOG_MAP);
            ww(m, a.wrapping_add(OB_GFX), 0x2302);
            wb(m, a.wrapping_add(OB_RENDER), 4);
            wb(m, a.wrapping_add(OB_PRIORITY), 4);
            wb(m, a.wrapping_add(OB_COLTYPE), 5);
            wb(m, a.wrapping_add(OB_ACTWID), 12);
            object_fall(m, a);
            let x = rw(m, a.wrapping_add(OB_X));
            let (dist, _) = obj_floor_dist_and_angle(m, a, x);
            if dist < 0 {
                let y = rw(m, a.wrapping_add(OB_Y)).wrapping_add(bits(dist));
                ww(m, a.wrapping_add(OB_Y), y);
                ww(m, a.wrapping_add(OB_VELY), 0);
                wb(m, a.wrapping_add(OB_ROUTINE), 2);
            }
        }
        2 => {
            animate_sprite(m, a, HOG_ANI)?;
            if rb(m, a.wrapping_add(OB_FRAME)) != 1 {
                wb(m, a.wrapping_add(FLAG), 0);
            } else if rb(m, a.wrapping_add(FLAG)) == 0 {
                wb(m, a.wrapping_add(FLAG), 1);
                if let Some(b) = find_free_obj(m) {
                    let status = rb(m, a.wrapping_add(OB_STATUS));
                    let x = rw(m, a.wrapping_add(OB_X));
                    let y = rw(m, a.wrapping_add(OB_Y));
                    let subtype = rb(m, a.wrapping_add(OB_SUBTYPE));
                    wb(m, b.wrapping_add(OB_ID), 0x20);
                    ww(
                        m,
                        b.wrapping_add(OB_X),
                        if status & 1 != 0 {
                            x.wrapping_add(4)
                        } else {
                            x.wrapping_sub(4)
                        },
                    );
                    ww(m, b.wrapping_add(OB_Y), y.wrapping_add(12));
                    ww(
                        m,
                        b.wrapping_add(OB_VELX),
                        if status & 1 != 0 { 0x100 } else { 0xFF00 },
                    );
                    ww(m, b.wrapping_add(OB_VELY), 0);
                    wb(m, b.wrapping_add(OB_SUBTYPE), subtype);
                }
            }
            remember_state(m, a);
        }
        _ => return Err(ErroAbertura::NaoImplementado("rotina de Ball Hog")),
    }
    Ok(())
}

pub(super) fn cannonball(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            wb(m, a.wrapping_add(OB_ROUTINE), 2);
            wb(m, a.wrapping_add(OB_HEIGHT), 7);
            wl(m, a.wrapping_add(OB_MAP), HOG_MAP);
            ww(m, a.wrapping_add(OB_GFX), 0x2302);
            wb(m, a.wrapping_add(OB_RENDER), 4);
            wb(m, a.wrapping_add(OB_PRIORITY), 3);
            wb(m, a.wrapping_add(OB_COLTYPE), 0x87);
            wb(m, a.wrapping_add(OB_ACTWID), 8);
            let tempo = u16::from(rb(m, a.wrapping_add(OB_SUBTYPE))).wrapping_mul(60);
            ww(m, a.wrapping_add(TEMPO), tempo);
            wb(m, a.wrapping_add(OB_FRAME), 4);
        }
        2 => {}
        _ => return Err(ErroAbertura::NaoImplementado("rotina de Cannonball")),
    }
    object_fall(m, a);
    if s16(rw(m, a.wrapping_add(OB_VELY))) >= 0 {
        let x = rw(m, a.wrapping_add(OB_X));
        let (dist, angulo) = obj_floor_dist_and_angle(m, a, x);
        if dist < 0 {
            let y = rw(m, a.wrapping_add(OB_Y)).wrapping_add(bits(dist));
            ww(m, a.wrapping_add(OB_Y), y);
            ww(m, a.wrapping_add(OB_VELY), 0xFD00);
            let vx = rw(m, a.wrapping_add(OB_VELX));
            if (angulo != 0 && angulo < 0x80 && s16(vx) < 0) || (angulo >= 0x80 && s16(vx) >= 0) {
                ww(m, a.wrapping_add(OB_VELX), vx.wrapping_neg());
            }
        }
    }
    let tempo = rw(m, a.wrapping_add(TEMPO)).wrapping_sub(1);
    ww(m, a.wrapping_add(TEMPO), tempo);
    if s16(tempo) < 0 {
        wb(m, a.wrapping_add(OB_ID), 0x3F);
        wb(m, a.wrapping_add(OB_ROUTINE), 0);
        return explosao(m, a);
    }
    let tempo_frame = rb(m, a.wrapping_add(OB_TIME_FRAME)).wrapping_sub(1);
    wb(m, a.wrapping_add(OB_TIME_FRAME), tempo_frame);
    if i8::from_be_bytes([tempo_frame]) < 0 {
        wb(m, a.wrapping_add(OB_TIME_FRAME), 5);
        let frame = rb(m, a.wrapping_add(OB_FRAME));
        wb(m, a.wrapping_add(OB_FRAME), frame ^ 1);
    }
    display_sprite(m, a);
    if rw(m, V_LIMITBTM2).wrapping_add(224) < rw(m, a.wrapping_add(OB_Y)) {
        delete_object(m, a);
    }
    Ok(())
}

pub(super) fn explosao(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            wb(m, a.wrapping_add(OB_ROUTINE), 2);
            wl(m, a.wrapping_add(OB_MAP), 0x958A);
            ww(m, a.wrapping_add(OB_GFX), 0x5A0);
            wb(m, a.wrapping_add(OB_RENDER), 4);
            wb(m, a.wrapping_add(OB_PRIORITY), 1);
            wb(m, a.wrapping_add(OB_COLTYPE), 0);
            wb(m, a.wrapping_add(OB_ACTWID), 12);
            wb(m, a.wrapping_add(OB_TIME_FRAME), 7);
            wb(m, a.wrapping_add(OB_FRAME), 0);
            wb(m, V_SOUNDQUEUE1, 0xC4);
        }
        2 => {
            let tempo = rb(m, a.wrapping_add(OB_TIME_FRAME)).wrapping_sub(1);
            wb(m, a.wrapping_add(OB_TIME_FRAME), tempo);
            if i8::from_be_bytes([tempo]) < 0 {
                wb(m, a.wrapping_add(OB_TIME_FRAME), 7);
                let frame = rb(m, a.wrapping_add(OB_FRAME)).wrapping_add(1);
                wb(m, a.wrapping_add(OB_FRAME), frame);
                if frame >= 5 {
                    delete_object(m, a);
                    return Ok(());
                }
            }
            display_sprite(m, a);
        }
        _ => return Err(ErroAbertura::NaoImplementado("rotina de Explosion")),
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    fn memoria() -> Memoria {
        let mut rom = vec![0; 0x10000];
        for (base, n) in [(NEWT_ANI, 5u32), (HOG_ANI, 1)] {
            let script = base.wrapping_add(n.wrapping_mul(2));
            let desloc = u16::try_from(n.wrapping_mul(2)).unwrap();
            for i in 0..n {
                let pos = usize::try_from(base.wrapping_add(i.wrapping_mul(2))).unwrap();
                rom[pos..pos.wrapping_add(2)].copy_from_slice(&desloc.to_be_bytes());
            }
            let pos = usize::try_from(script).unwrap();
            rom[pos] = 15;
            rom[pos.wrapping_add(1)] = 0;
            rom[pos.wrapping_add(2)] = 0xFF;
        }
        Memoria::nova(rom)
    }

    #[test]
    fn newtron_verde_dispara_uma_vez_na_direcao_do_jogador() {
        let mut m = memoria();
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x42);
        ww(&mut m, a.wrapping_add(OB_X), 0x200);
        ww(&mut m, a.wrapping_add(OB_Y), 0x100);
        ww(&mut m, V_PLAYER.wrapping_add(OB_X), 0x1F0);
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 1);
        newtron(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE_2)), 8);
        wb(&mut m, a.wrapping_add(OB_FRAME), 2);
        newtron(&mut m, a).unwrap();
        let b = a.wrapping_add(OBJECT_SIZE);
        assert_eq!(rb(&mut m, b.wrapping_add(OB_ID)), 0x23);
        assert_eq!(rw(&mut m, b.wrapping_add(OB_X)), 0x1EC);
        assert_eq!(rw(&mut m, b.wrapping_add(OB_VELX)), 0xFE00);
        assert_eq!(rb(&mut m, b.wrapping_add(OB_SUBTYPE)), 1);
        wb(&mut m, a.wrapping_add(OB_FRAME), 2);
        newtron(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, b.wrapping_add(OBJECT_SIZE)), 0);
    }

    #[test]
    fn cannonball_expira_e_inicia_explosao() {
        let mut m = memoria();
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x20);
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 0);
        cannonball(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ID)), 0x3F);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(rb(&mut m, V_SOUNDQUEUE1), 0xC4);
    }
}
