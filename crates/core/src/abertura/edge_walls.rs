//! Objeto 44: Paredes de borda da Green Hill Zone (`44 GHZ Edge Walls.asm`).

use super::objetos::{delete_object, display_sprite, fora_da_janela};
use super::ram::*;
use super::{ErroAbertura, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;

pub const MAP_EDGE: u32 = 0x00E8DE;
pub const ART_TILE_GHZ_EDGE_WALL: u16 = 0x034C;
pub const TILE_PAL3: u16 = 2 << 13;
pub const ARTTILE_EDGE_WALL: u16 = ART_TILE_GHZ_EDGE_WALL | TILE_PAL3; // 0x434C

fn s16(v: u16) -> i16 {
    i16::from_be_bytes(v.to_be_bytes())
}

fn u16_de(i: i16) -> u16 {
    u16::from_be_bytes(i.to_be_bytes())
}

/// Executa o objeto `EdgeWalls` (`0x44`).
pub fn executar(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let mut rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        wb(m, a0.wrapping_add(OB_ROUTINE), 2);
        wl(m, a0.wrapping_add(OB_MAP), MAP_EDGE);
        ww(m, a0.wrapping_add(OB_GFX), ARTTILE_EDGE_WALL);
        let render = rb(m, a0.wrapping_add(OB_RENDER)) | RENDER_CAM_FIELD;
        wb(m, a0.wrapping_add(OB_RENDER), render);
        wb(m, a0.wrapping_add(OB_ACTWID), 8); // 16 / 2
        wb(m, a0.wrapping_add(OB_PRIORITY), 6);

        let subtype = rb(m, a0.wrapping_add(OB_SUBTYPE));
        let frame = subtype & !0x10;
        wb(m, a0.wrapping_add(OB_FRAME), frame);
        if subtype & 0x10 != 0 {
            wb(m, a0.wrapping_add(OB_ROUTINE), 4);
            rotina = 4;
        } else {
            rotina = 2;
        }
    }

    if rotina == 2 {
        edge_wall_solid_wall(m, a0, 19, 40);
    }

    display_sprite(m, a0);
    let x = rw(m, a0.wrapping_add(OB_X));
    if fora_da_janela(m, x) {
        delete_object(m, a0);
    }

    Ok(())
}

fn edge_wall_solid_wall(m: &mut Memoria, a0: u32, d1: i16, d2: i16) {
    let a1 = V_PLAYER;
    let sonic_x = s16(rw(m, a1.wrapping_add(OB_X)));
    let obj_x = s16(rw(m, a0.wrapping_add(OB_X)));
    let mut d0 = sonic_x.wrapping_sub(obj_x).wrapping_add(d1);
    let d1_u = u16_de(d1);
    if d0 < 0 || u16_de(d0) > d1_u.wrapping_mul(2) {
        edge_no_collision(m, a0, a1);
        return;
    }

    let sonic_h = s16(u16::from(rb(m, a1.wrapping_add(OB_HEIGHT))));
    let d2_tot = d2.wrapping_add(sonic_h);
    let d2_u = u16_de(d2_tot);
    let sonic_y = s16(rw(m, a1.wrapping_add(OB_Y)));
    let obj_y = s16(rw(m, a0.wrapping_add(OB_Y)));
    let mut d3 = sonic_y.wrapping_sub(obj_y).wrapping_add(d2_tot);
    if d3 < 0 || u16_de(d3) >= d2_u.wrapping_mul(2) {
        edge_no_collision(m, a0, a1);
        return;
    }

    if rb(m, F_PLAYERCTRL) & 0x80 != 0
        || rb(m, a1.wrapping_add(OB_ROUTINE)) >= 6
        || rw(m, V_DEBUGUSE) != 0
    {
        edge_no_collision(m, a0, a1);
        return;
    }

    let mut d5 = d0;
    if u16_de(d0) > d1_u {
        d0 = d0.wrapping_sub(d1.wrapping_mul(2));
        d5 = d0.wrapping_neg();
    }

    let mut d1_side = d3;
    if u16_de(d3) > d2_u {
        d3 = d3.wrapping_sub(d2_tot.wrapping_mul(2));
        d1_side = d3.wrapping_neg();
    }

    if u16_de(d5) > u16_de(d1_side) {
        // Top/bottom collision
        let vy = s16(rw(m, a1.wrapping_add(OB_VELY)));
        if vy < 0 && d3 < 0 {
            let y = rw(m, a1.wrapping_add(OB_Y));
            ww(m, a1.wrapping_add(OB_Y), y.wrapping_sub(u16_de(d3)));
            ww(m, a1.wrapping_add(OB_VELY), 0);
        }
        return;
    }

    // Side collision
    let vx = s16(rw(m, a1.wrapping_add(OB_VELX)));
    if d0 != 0 {
        if d0 < 0 {
            if vx < 0 {
                let x = rw(m, a1.wrapping_add(OB_X));
                ww(m, a1.wrapping_add(OB_X), x.wrapping_sub(u16_de(d0)));
                ww(m, a1.wrapping_add(OB_INERTIA), 0);
                ww(m, a1.wrapping_add(OB_VELX), 0);
            }
        } else if vx >= 0 {
            let x = rw(m, a1.wrapping_add(OB_X));
            ww(m, a1.wrapping_add(OB_X), x.wrapping_sub(u16_de(d0)));
            ww(m, a1.wrapping_add(OB_INERTIA), 0);
            ww(m, a1.wrapping_add(OB_VELX), 0);
        }
    }

    // .centre
    if rb(m, a1.wrapping_add(OB_STATUS)) & STATUS_IN_AIR == 0 {
        let s1 = rb(m, a1.wrapping_add(OB_STATUS)) | STATUS_PUSHING;
        wb(m, a1.wrapping_add(OB_STATUS), s1);
        let s0 = rb(m, a0.wrapping_add(OB_STATUS)) | STATUS_PUSHING;
        wb(m, a0.wrapping_add(OB_STATUS), s0);
    } else {
        let s0 = rb(m, a0.wrapping_add(OB_STATUS)) & !STATUS_PUSHING;
        wb(m, a0.wrapping_add(OB_STATUS), s0);
        let s1 = rb(m, a1.wrapping_add(OB_STATUS)) & !STATUS_PUSHING;
        wb(m, a1.wrapping_add(OB_STATUS), s1);
    }
}

fn edge_no_collision(m: &mut Memoria, a0: u32, a1: u32) {
    if rb(m, a0.wrapping_add(OB_STATUS)) & STATUS_PUSHING != 0 {
        ww(m, a1.wrapping_add(OB_ANIM), u16::from(ANIM_RUN));
    }
    let s0 = rb(m, a0.wrapping_add(OB_STATUS)) & !STATUS_PUSHING;
    wb(m, a0.wrapping_add(OB_STATUS), s0);
    let s1 = rb(m, a1.wrapping_add(OB_STATUS)) & !STATUS_PUSHING;
    wb(m, a1.wrapping_add(OB_STATUS), s1);
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::abertura::ram::{V_LVLOBJSPACE, V_SCREENPOSX};
    use crate::abertura::rl;

    fn memoria() -> Memoria {
        Memoria::nova(vec![0; 0x10000])
    }

    #[test]
    fn edge_walls_inicializa_e_e_solida() {
        let mut m = memoria();
        let a = V_LVLOBJSPACE;
        wb(&mut m, a + OB_ID, 0x44);
        wb(&mut m, a + OB_SUBTYPE, 0x01); // solida, frame 1
        ww(&mut m, a + OB_X, 200);
        ww(&mut m, a + OB_Y, 200);
        ww(&mut m, V_SCREENPOSX, 200);

        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a + OB_ROUTINE), 2);
        assert_eq!(rl(&mut m, a + OB_MAP), MAP_EDGE);
        assert_eq!(rw(&mut m, a + OB_GFX), ARTTILE_EDGE_WALL);
        assert_eq!(rb(&mut m, a + OB_FRAME), 1);
        assert_eq!(rb(&mut m, a + OB_ACTWID), 8);
        assert_eq!(rb(&mut m, a + OB_PRIORITY), 6);

        // Subtipo não-sólido (com bit 4 ligado)
        wb(&mut m, a + OB_ROUTINE, 0);
        wb(&mut m, a + OB_SUBTYPE, 0x12);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a + OB_ROUTINE), 4);
        assert_eq!(rb(&mut m, a + OB_FRAME), 2);
    }
}
