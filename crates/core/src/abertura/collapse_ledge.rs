//! Objeto 1A: Plataforma que desmorona em Green Hill Zone (`1A, 53 Collapsing Ledges and Floors.asm`).

use super::boot::{rom_b, rom_w};
use super::objetos::{delete_object, display_sprite, find_free_obj, remember_state};
use super::ram::*;
use super::{ErroAbertura, rb, rl, rw, wb, wl, ww};
use crate::memoria::Memoria;

pub const MAP_LEDGE: u32 = 0x008C1E;
pub const ARTTILE_LEDGE: u16 = 0x4000;

const COLLAPSE_DATA_GHZ_LEDGE: u32 = 0x8B7E;
const LEDGE_SLOPE_DATA: u32 = 0x8BEE;

/// Executa o objeto `CollapseLedge` (`0x1A`).
pub fn executar(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    match rotina {
        0 => ledge_main(m, a0),
        2 => ledge_chk_touch(m, a0),
        4 => ledge_on_platform(m, a0),
        6 => ledge_fragment_piece(m, a0),
        8 => {
            delete_object(m, a0);
            Ok(())
        }
        10 => ledge_walk_off_routine(m, a0),
        _ => Ok(()),
    }
}

fn ledge_main(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    wb(m, a0.wrapping_add(OB_ROUTINE), 2);
    wl(m, a0.wrapping_add(OB_MAP), MAP_LEDGE);
    ww(m, a0.wrapping_add(OB_GFX), ARTTILE_LEDGE);
    let render = rb(m, a0.wrapping_add(OB_RENDER)) | RENDER_CAM_FIELD | 0x10;
    wb(m, a0.wrapping_add(OB_RENDER), render);
    wb(m, a0.wrapping_add(OB_PRIORITY), 4);
    wb(m, a0.wrapping_add(0x38), 7); // collapsible_timedelay
    wb(m, a0.wrapping_add(OB_ACTWID), 100); // 200 / 2
    let subtype = rb(m, a0.wrapping_add(OB_SUBTYPE));
    wb(m, a0.wrapping_add(OB_FRAME), subtype);
    wb(m, a0.wrapping_add(OB_HEIGHT), 56); // 112 / 2
    ledge_chk_touch(m, a0)
}

fn ledge_chk_touch(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    if rb(m, a0.wrapping_add(0x3A)) != 0 {
        let delay = rb(m, a0.wrapping_add(0x38));
        if delay == 0 {
            fragmentate_ghz_ledge(m, a0, true);
            return Ok(());
        }
        wb(m, a0.wrapping_add(0x38), delay.wrapping_sub(1));
    }

    slope_object(m, a0, 48)?;
    remember_state(m, a0);
    Ok(())
}

fn ledge_on_platform(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let delay = rb(m, a0.wrapping_add(0x38));
    if delay == 0 {
        fragmentate_ghz_ledge(m, a0, false);
        return Ok(());
    }
    wb(m, a0.wrapping_add(0x3A), 1);
    wb(m, a0.wrapping_add(0x38), delay.wrapping_sub(1));

    ledge_walk_off(m, a0);
    remember_state(m, a0);
    Ok(())
}

fn ledge_walk_off_routine(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    ledge_walk_off(m, a0);
    remember_state(m, a0);
    Ok(())
}

fn ledge_walk_off(m: &mut Memoria, a0: u32) {
    exit_platform(m, a0, 48);
    slope_object_assume_stood_on(m, a0, 48);
}

fn exit_platform(m: &mut Memoria, a0: u32, d1: u16) {
    let d2 = d1.wrapping_mul(2);
    let player_status = rb(m, V_PLAYER.wrapping_add(OB_STATUS));
    if player_status & STATUS_IN_AIR != 0 {
        let ps = player_status & !STATUS_ON_OBJECT;
        wb(m, V_PLAYER.wrapping_add(OB_STATUS), ps);
        wb(m, a0.wrapping_add(OB_ROUTINE), 2);
        let os = rb(m, a0.wrapping_add(OB_STATUS)) & !STATUS_ON_OBJECT;
        wb(m, a0.wrapping_add(OB_STATUS), os);
        return;
    }

    let px = rw(m, V_PLAYER.wrapping_add(OB_X));
    let ox = rw(m, a0.wrapping_add(OB_X));
    let d0 = px.wrapping_sub(ox).wrapping_add(d1);
    if i16::from_be_bytes(d0.to_be_bytes()) < 0 || d0 >= d2 {
        let ps = player_status & !STATUS_ON_OBJECT;
        wb(m, V_PLAYER.wrapping_add(OB_STATUS), ps);
        wb(m, a0.wrapping_add(OB_ROUTINE), 2);
        let os = rb(m, a0.wrapping_add(OB_STATUS)) & !STATUS_ON_OBJECT;
        wb(m, a0.wrapping_add(OB_STATUS), os);
    }
}

fn slope_object_assume_stood_on(m: &mut Memoria, a0: u32, d1: u16) {
    if rb(m, V_PLAYER.wrapping_add(OB_STATUS)) & STATUS_ON_OBJECT == 0 {
        return;
    }
    let px = rw(m, V_PLAYER.wrapping_add(OB_X));
    let ox = rw(m, a0.wrapping_add(OB_X));
    let mut d0 = px.wrapping_sub(ox).wrapping_add(d1);
    d0 >>= 1;
    let render = rb(m, a0.wrapping_add(OB_RENDER));
    if render & 1 != 0 {
        d0 = (!d0).wrapping_add(d1);
    }
    let idx = usize::from(d0 & 0x7FFF).min(47);
    let slope_val = u16::from(rom_b(
        m,
        LEDGE_SLOPE_DATA.wrapping_add(u32::try_from(idx).unwrap_or(0)),
    ));
    let oy = rw(m, a0.wrapping_add(OB_Y));
    let ph = u16::from(rb(m, V_PLAYER.wrapping_add(OB_HEIGHT)));
    let new_py = oy.wrapping_sub(slope_val).wrapping_sub(ph);
    ww(m, V_PLAYER.wrapping_add(OB_Y), new_py);
}

fn slope_object(m: &mut Memoria, a0: u32, d1: u16) -> Result<(), ErroAbertura> {
    let p_vy = i16::from_be_bytes(rw(m, V_PLAYER.wrapping_add(OB_VELY)).to_be_bytes());
    if p_vy < 0 {
        return Ok(());
    }

    let px = rw(m, V_PLAYER.wrapping_add(OB_X));
    let ox = rw(m, a0.wrapping_add(OB_X));
    let mut d0 = px.wrapping_sub(ox).wrapping_add(d1);
    if i16::from_be_bytes(d0.to_be_bytes()) < 0 {
        return Ok(());
    }
    let d1_double = d1.wrapping_mul(2);
    if d0 >= d1_double {
        return Ok(());
    }

    let render = rb(m, a0.wrapping_add(OB_RENDER));
    if render & 1 != 0 {
        d0 = (!d0).wrapping_add(d1_double);
    }
    d0 >>= 1;
    let idx = usize::from(d0 & 0x7FFF).min(47);
    let slope_val = rom_b(
        m,
        LEDGE_SLOPE_DATA.wrapping_add(u32::try_from(idx).unwrap_or(0)),
    );

    let oy = rw(m, a0.wrapping_add(OB_Y));
    let surface_y = oy.wrapping_sub(u16::from(slope_val));

    let py = rw(m, V_PLAYER.wrapping_add(OB_Y));
    let ph = u16::from(rb(m, V_PLAYER.wrapping_add(OB_HEIGHT)));
    let p_bottom = py.wrapping_add(ph).wrapping_add(4);
    let dist = i16::from_be_bytes(surface_y.wrapping_sub(p_bottom).to_be_bytes());
    if !(-16..=-1).contains(&dist) {
        return Ok(());
    }

    if i8::from_be_bytes([rb(m, F_PLAYERCTRL)]) < 0 {
        return Ok(());
    }
    if rb(m, V_PLAYER.wrapping_add(OB_ROUTINE)) >= 6 {
        return Ok(());
    }

    let dist_u16 = u16::from_be_bytes(dist.to_be_bytes());
    let new_py = py.wrapping_add(dist_u16).wrapping_add(3);
    ww(m, V_PLAYER.wrapping_add(OB_Y), new_py);
    let rout = rb(m, a0.wrapping_add(OB_ROUTINE)).wrapping_add(2);
    wb(m, a0.wrapping_add(OB_ROUTINE), rout);

    let player_status = rb(m, V_PLAYER.wrapping_add(OB_STATUS));
    if player_status & STATUS_ON_OBJECT != 0 {
        let prev_idx = u32::from(rb(m, V_PLAYER.wrapping_add(SON_STANDONOBJECT)));
        let prev_obj = V_OBJSPACE.wrapping_add(prev_idx.wrapping_mul(OBJECT_SIZE));
        let ps = rb(m, prev_obj.wrapping_add(OB_STATUS)) & !STATUS_ON_OBJECT;
        wb(m, prev_obj.wrapping_add(OB_STATUS), ps);
        wb(m, prev_obj.wrapping_add(OB_ROUTINE_2), 0);
        if rb(m, prev_obj.wrapping_add(OB_ROUTINE)) == 4 {
            wb(m, prev_obj.wrapping_add(OB_ROUTINE), 2);
        }
    }

    let current_idx = u8::try_from((a0.wrapping_sub(V_OBJSPACE) / OBJECT_SIZE) & 0x7F).unwrap_or(0);
    wb(m, V_PLAYER.wrapping_add(SON_STANDONOBJECT), current_idx);
    wb(m, V_PLAYER.wrapping_add(OB_ANGLE), 0);
    ww(m, V_PLAYER.wrapping_add(OB_VELY), 0);
    let vx = rw(m, V_PLAYER.wrapping_add(OB_VELX));
    ww(m, V_PLAYER.wrapping_add(OB_INERTIA), vx);

    if player_status & STATUS_IN_AIR != 0 {
        super::fisica::sonic_reset_on_floor(m, V_PLAYER);
    }

    let ps = rb(m, V_PLAYER.wrapping_add(OB_STATUS)) | STATUS_ON_OBJECT;
    wb(m, V_PLAYER.wrapping_add(OB_STATUS), ps);
    let os = rb(m, a0.wrapping_add(OB_STATUS)) | STATUS_ON_OBJECT;
    wb(m, a0.wrapping_add(OB_STATUS), os);

    Ok(())
}

fn fragmentate_ghz_ledge(m: &mut Memoria, a0: u32, reset_flag: bool) {
    if reset_flag {
        wb(m, a0.wrapping_add(0x3A), 0);
    }

    let frame = rb(m, a0.wrapping_add(OB_FRAME)).wrapping_add(2);
    wb(m, a0.wrapping_add(OB_FRAME), frame);

    let map_base = rl(m, a0.wrapping_add(OB_MAP));
    let frame_off = u32::from(rom_w(
        m,
        map_base.wrapping_add(u32::from(frame).wrapping_mul(2)),
    ));
    let mut a3 = map_base.wrapping_add(frame_off).wrapping_add(1);

    let render = rb(m, a0.wrapping_add(OB_RENDER)) | 0x20;
    wb(m, a0.wrapping_add(OB_RENDER), render);

    let obj_id = rb(m, a0.wrapping_add(OB_ID));
    let x = rw(m, a0.wrapping_add(OB_X));
    let y = rw(m, a0.wrapping_add(OB_Y));
    let gfx = rw(m, a0.wrapping_add(OB_GFX));
    let prio = rb(m, a0.wrapping_add(OB_PRIORITY));
    let actwid = rb(m, a0.wrapping_add(OB_ACTWID));

    wb(m, a0.wrapping_add(OB_ROUTINE), 6);
    wl(m, a0.wrapping_add(OB_MAP), a3);
    let delay = rom_b(m, COLLAPSE_DATA_GHZ_LEDGE);
    wb(m, a0.wrapping_add(0x38), delay);

    for indice in 1..25u32 {
        let delay = rom_b(m, COLLAPSE_DATA_GHZ_LEDGE.wrapping_add(indice));
        if let Some(a1) = find_free_obj(m) {
            a3 = a3.wrapping_add(5);
            wb(m, a1.wrapping_add(OB_ROUTINE), 6);
            wb(m, a1.wrapping_add(OB_ID), obj_id);
            wl(m, a1.wrapping_add(OB_MAP), a3);
            wb(m, a1.wrapping_add(OB_RENDER), render);
            ww(m, a1.wrapping_add(OB_X), x);
            ww(m, a1.wrapping_add(OB_Y), y);
            ww(m, a1.wrapping_add(OB_GFX), gfx);
            wb(m, a1.wrapping_add(OB_PRIORITY), prio);
            wb(m, a1.wrapping_add(OB_ACTWID), actwid);
            wb(m, a1.wrapping_add(0x38), delay);

            if a1 < a0 {
                display_sprite(m, a1);
            }
        } else {
            break;
        }
    }

    display_sprite(m, a0);
    wb(m, V_SOUNDQUEUE1, 0xB9);
}

fn ledge_fragment_piece(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let delay = rb(m, a0.wrapping_add(0x38));
    if delay == 0 {
        super::fisica::object_fall(m, a0);
        display_sprite(m, a0);
        if rb(m, a0.wrapping_add(OB_RENDER)) & 0x80 == 0 {
            delete_object(m, a0);
        }
        return Ok(());
    }

    let flag = rb(m, a0.wrapping_add(0x3A));
    if flag != 0 {
        let n_delay = delay.wrapping_sub(1);
        wb(m, a0.wrapping_add(0x38), n_delay);

        ledge_walk_off(m, a0);

        let ps = rb(m, V_PLAYER.wrapping_add(OB_STATUS));
        if ps & STATUS_ON_OBJECT == 0 {
            wb(m, a0.wrapping_add(0x3A), 0);
            wb(m, a0.wrapping_add(OB_ROUTINE), 6);
            return Ok(());
        }

        if n_delay != 0 {
            return Ok(());
        }

        let nps = ps & !(STATUS_ON_OBJECT | STATUS_PUSHING);
        wb(m, V_PLAYER.wrapping_add(OB_STATUS), nps);
        wb(m, V_PLAYER.wrapping_add(OB_PREV_ANI), 1); // id_Run = 1
        wb(m, a0.wrapping_add(0x3A), 0);
        wb(m, a0.wrapping_add(OB_ROUTINE), 6);
        return Ok(());
    }

    wb(m, a0.wrapping_add(0x38), delay.wrapping_sub(1));
    display_sprite(m, a0);
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::abertura::ram::{V_LVLOBJSPACE, V_SCREENPOSX};

    fn memoria() -> Memoria {
        Memoria::nova(vec![0; 0x10000])
    }

    #[test]
    fn ledge_inicializa_com_rotina_2_e_constantes() {
        let mut m = memoria();
        let a = V_LVLOBJSPACE;
        wb(&mut m, a + OB_ID, 0x1A);
        wb(&mut m, a + OB_SUBTYPE, 0x00);
        ww(&mut m, a + OB_X, 300);
        ww(&mut m, a + OB_Y, 200);
        ww(&mut m, V_SCREENPOSX, 300);

        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a + OB_ROUTINE), 2);
        assert_eq!(rl(&mut m, a + OB_MAP), MAP_LEDGE);
        assert_eq!(rw(&mut m, a + OB_GFX), ARTTILE_LEDGE);
        assert_eq!(rb(&mut m, a + OB_FRAME), 0);
        assert_eq!(rb(&mut m, a + OB_ACTWID), 100);
        assert_eq!(rb(&mut m, a + OB_HEIGHT), 56);
        assert_eq!(rb(&mut m, a + 0x38), 7);
    }
}
