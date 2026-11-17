//! Rebatedor de pinball de Spring Yard, objeto 0x47 (REV01).
use super::super::fisica::{add_points, calc_angle, calc_sine};
use super::super::ram::*;
use super::super::{ErroAbertura, rb, rw, wb, wl, ww};
use super::{animate_sprite, find_free_obj, remember_state};
use crate::memoria::Memoria;

const MAP_BUMP: u32 = 0xF18C;
const ANI_BUMP: u32 = 0xF17C;
const SFX_BUMPER: u8 = 0xB4;

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let mut rotina = rb(m, a.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        rotina = 2;
        wb(m, a.wrapping_add(OB_ROUTINE), rotina);
        wl(m, a.wrapping_add(OB_MAP), MAP_BUMP);
        ww(m, a.wrapping_add(OB_GFX), 0x0380);
        wb(m, a.wrapping_add(OB_RENDER), 4);
        wb(m, a.wrapping_add(OB_ACTWID), 16);
        wb(m, a.wrapping_add(OB_PRIORITY), 1);
        wb(m, a.wrapping_add(OB_COLTYPE), 0xD7);
    }
    if rotina == 2 {
        if rb(m, a.wrapping_add(OB_COLPROP)) != 0 {
            wb(m, a.wrapping_add(OB_COLPROP), 0);

            let bx = rw(m, a.wrapping_add(OB_X));
            let by = rw(m, a.wrapping_add(OB_Y));
            let sx = rw(m, V_PLAYER.wrapping_add(OB_X));
            let sy = rw(m, V_PLAYER.wrapping_add(OB_Y));
            let dx = bx.wrapping_sub(sx);
            let dy = by.wrapping_sub(sy);

            let angulo = calc_angle(m, dx, dy);
            let (seno, cosseno) = calc_sine(m, angulo);

            let vx = (i32::from(cosseno).wrapping_mul(-0x700)) >> 8;
            let vy = (i32::from(seno).wrapping_mul(-0x700)) >> 8;

            ww(m, V_PLAYER.wrapping_add(OB_VELX), vx as u16);
            ww(m, V_PLAYER.wrapping_add(OB_VELY), vy as u16);

            let mut st = rb(m, V_PLAYER.wrapping_add(OB_STATUS));
            st |= STATUS_IN_AIR;
            st &= !0x10;
            st &= !STATUS_PUSHING;
            wb(m, V_PLAYER.wrapping_add(OB_STATUS), st);
            wb(m, V_PLAYER.wrapping_add(SON_JUMPING), 0);

            wb(m, a.wrapping_add(OB_ANIM), 1);
            wb(m, V_SOUNDQUEUE1, SFX_BUMPER);

            let respawn = rb(m, a.wrapping_add(OB_RESPAWNNO));
            let mut pode_pontuar = true;
            if respawn != 0 {
                let marca = V_OBJSTATE.wrapping_add(2).wrapping_add(u32::from(respawn));
                let hits = rb(m, marca);
                if hits >= 10 + 0x80 {
                    pode_pontuar = false;
                } else {
                    wb(m, marca, hits.wrapping_add(1));
                }
            }
            if pode_pontuar {
                add_points(m, 10);
                if let Some(a1) = find_free_obj(m) {
                    wb(m, a1.wrapping_add(OB_ID), ID_POINTS);
                    ww(m, a1.wrapping_add(OB_X), bx);
                    ww(m, a1.wrapping_add(OB_Y), by);
                    wb(m, a1.wrapping_add(OB_FRAME), 4);
                }
            }
        }
        animate_sprite(m, a, ANI_BUMP)?;
        remember_state(m, a);
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn inicializa_bumper() {
        let mut m = Memoria::nova(vec![0; 0x1000]);
        let a0: u32 = 0xFF_D800;
        wb(&mut m, a0.wrapping_add(OB_ID), 0x47);
        wb(&mut m, a0.wrapping_add(OB_ROUTINE), 0);
        ww(&mut m, a0.wrapping_add(OB_X), 0x100);
        ww(&mut m, a0.wrapping_add(OB_Y), 0x100);

        let res = executar(&mut m, a0);
        assert!(res.is_ok());
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_COLTYPE)), 0xD7);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_PRIORITY)), 1);
    }
}
