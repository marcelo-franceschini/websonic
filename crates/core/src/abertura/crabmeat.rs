//! Badnik Crabmeat (GHZ, SYZ): patrulha com detecção de terreno/borda e disparos de fogo.
use super::fisica::{obj_floor_dist_and_angle, object_fall, speed_to_pos};
use super::objetos::{
    animate_sprite, delete_object, display_sprite, find_free_obj, remember_state,
};
use super::ram::*;
use super::{ErroAbertura, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;

pub const MAP_CRAB: u32 = 0x009DCE;
pub const ANI_CRAB: u32 = 0x009D98;
pub const ART_TILE_CRABMEAT: u16 = 0x0400;

const CRAB_TIMEDELAY: u32 = 0x30;
const CRAB_FLAGS: u32 = 0x32;

fn u16_de(i: i16) -> u16 {
    u16::from_be_bytes(i.to_be_bytes())
}

fn s16(w: u16) -> i16 {
    i16::from_be_bytes(w.to_be_bytes())
}

pub fn executar(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    match rb(m, a0.wrapping_add(OB_ROUTINE)) {
        0 => crab_main(m, a0),
        2 => crab_action(m, a0),
        4 => {
            delete_object(m, a0);
            Ok(())
        }
        6 => crab_ball_main(m, a0),
        8 => crab_ball_move(m, a0),
        _ => Err(ErroAbertura::NaoImplementado("rotina de Crabmeat")),
    }
}

fn crab_main(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    wb(m, a0.wrapping_add(OB_HEIGHT), 16);
    wb(m, a0.wrapping_add(OB_WIDTH), 8);
    wl(m, a0.wrapping_add(OB_MAP), MAP_CRAB);
    ww(m, a0.wrapping_add(OB_GFX), ART_TILE_CRABMEAT);
    wb(m, a0.wrapping_add(OB_RENDER), 4);
    wb(m, a0.wrapping_add(OB_PRIORITY), 3);
    wb(m, a0.wrapping_add(OB_COLTYPE), 6);
    wb(m, a0.wrapping_add(OB_ACTWID), 21);

    object_fall(m, a0);
    let x = rw(m, a0.wrapping_add(OB_X));
    let (d1, angulo) = obj_floor_dist_and_angle(m, a0, x);
    if d1 < 0 {
        let y = rw(m, a0.wrapping_add(OB_Y)).wrapping_add(u16_de(d1));
        ww(m, a0.wrapping_add(OB_Y), y);
        wb(m, a0.wrapping_add(OB_ANGLE), angulo);
        ww(m, a0.wrapping_add(OB_VELY), 0);
        wb(m, a0.wrapping_add(OB_ROUTINE), 2);
    }
    Ok(())
}

fn crab_action(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    match rb(m, a0.wrapping_add(OB_ROUTINE_2)) {
        0 => crab_action_wait_fire(m, a0),
        2 => crab_action_scuttle(m, a0),
        _ => {
            return Err(ErroAbertura::NaoImplementado(
                "rotina secundaria de Crabmeat",
            ));
        }
    }

    animate_sprite(m, a0, ANI_CRAB)?;
    remember_state(m, a0);
    Ok(())
}

fn crab_action_wait_fire(m: &mut Memoria, a0: u32) {
    let delay = s16(rw(m, a0.wrapping_add(CRAB_TIMEDELAY))).wrapping_sub(1);
    ww(m, a0.wrapping_add(CRAB_TIMEDELAY), u16_de(delay));
    if delay >= 0 {
        return;
    }
    let render = rb(m, a0.wrapping_add(OB_RENDER));
    if render & 0x80 == 0 {
        start_moving(m, a0);
        return;
    }
    let flags = rb(m, a0.wrapping_add(CRAB_FLAGS));
    wb(m, a0.wrapping_add(CRAB_FLAGS), flags ^ 2);
    if flags & 2 != 0 {
        action_fire(m, a0);
    } else {
        start_moving(m, a0);
    }
}

fn start_moving(m: &mut Memoria, a0: u32) {
    wb(m, a0.wrapping_add(OB_ROUTINE_2), 2);
    ww(m, a0.wrapping_add(CRAB_TIMEDELAY), 127);
    ww(m, a0.wrapping_add(OB_VELX), 0x80);
    let angle = rb(m, a0.wrapping_add(OB_ANGLE));
    let status = rb(m, a0.wrapping_add(OB_STATUS));
    let anim = crab_set_ani(angle, status).wrapping_add(3);
    wb(m, a0.wrapping_add(OB_ANIM), anim);
    let new_status = status ^ 1;
    wb(m, a0.wrapping_add(OB_STATUS), new_status);
    if status & 1 == 0 {
        ww(m, a0.wrapping_add(OB_VELX), u16_de(-0x80));
    }
}

fn action_fire(m: &mut Memoria, a0: u32) {
    ww(m, a0.wrapping_add(CRAB_TIMEDELAY), 59);
    wb(m, a0.wrapping_add(OB_ANIM), 6);
    let x = rw(m, a0.wrapping_add(OB_X));
    let y = rw(m, a0.wrapping_add(OB_Y));

    if let Some(a1) = find_free_obj(m) {
        wb(m, a1.wrapping_add(OB_ID), ID_CRABMEAT);
        wb(m, a1.wrapping_add(OB_ROUTINE), 6);
        ww(m, a1.wrapping_add(OB_X), x.wrapping_sub(0x10));
        ww(m, a1.wrapping_add(OB_Y), y);
        ww(m, a1.wrapping_add(OB_VELX), u16_de(-0x100));
    }

    if let Some(a1) = find_free_obj(m) {
        wb(m, a1.wrapping_add(OB_ID), ID_CRABMEAT);
        wb(m, a1.wrapping_add(OB_ROUTINE), 6);
        ww(m, a1.wrapping_add(OB_X), x.wrapping_add(0x10));
        ww(m, a1.wrapping_add(OB_Y), y);
        ww(m, a1.wrapping_add(OB_VELX), 0x100);
    }
}

fn crab_action_scuttle(m: &mut Memoria, a0: u32) {
    let delay = s16(rw(m, a0.wrapping_add(CRAB_TIMEDELAY))).wrapping_sub(1);
    ww(m, a0.wrapping_add(CRAB_TIMEDELAY), u16_de(delay));
    if delay < 0 {
        init_fire(m, a0);
        return;
    }

    speed_to_pos(m, a0);
    let flags = rb(m, a0.wrapping_add(CRAB_FLAGS));
    wb(m, a0.wrapping_add(CRAB_FLAGS), flags ^ 1);
    if flags & 1 != 0 {
        let x = rw(m, a0.wrapping_add(OB_X));
        let (d1, angulo) = obj_floor_dist_and_angle(m, a0, x);
        let y = rw(m, a0.wrapping_add(OB_Y)).wrapping_add(u16_de(d1));
        ww(m, a0.wrapping_add(OB_Y), y);
        wb(m, a0.wrapping_add(OB_ANGLE), angulo);
        let status = rb(m, a0.wrapping_add(OB_STATUS));
        let anim = crab_set_ani(angulo, status).wrapping_add(3);
        wb(m, a0.wrapping_add(OB_ANIM), anim);
    } else {
        let x = rw(m, a0.wrapping_add(OB_X));
        let status = rb(m, a0.wrapping_add(OB_STATUS));
        let check_x = if status & 1 != 0 {
            x.wrapping_sub(16)
        } else {
            x.wrapping_add(16)
        };
        let (d1, _) = obj_floor_dist_and_angle(m, a0, check_x);
        if !(-8..12).contains(&d1) {
            init_fire(m, a0);
        }
    }
}

fn init_fire(m: &mut Memoria, a0: u32) {
    wb(m, a0.wrapping_add(OB_ROUTINE_2), 0);
    ww(m, a0.wrapping_add(CRAB_TIMEDELAY), 59);
    ww(m, a0.wrapping_add(OB_VELX), 0);
    let angle = rb(m, a0.wrapping_add(OB_ANGLE));
    let status = rb(m, a0.wrapping_add(OB_STATUS));
    let anim = crab_set_ani(angle, status);
    wb(m, a0.wrapping_add(OB_ANIM), anim);
}

fn crab_set_ani(angle: u8, status: u8) -> u8 {
    let facing_left = status & 1 != 0;
    if angle < 0x80 {
        if angle < 6 {
            0
        } else if facing_left {
            1
        } else {
            2
        }
    } else if angle > 0xFA {
        0
    } else if facing_left {
        2
    } else {
        1
    }
}

fn crab_ball_main(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    wb(m, a0.wrapping_add(OB_ROUTINE), 8);
    wl(m, a0.wrapping_add(OB_MAP), MAP_CRAB);
    ww(m, a0.wrapping_add(OB_GFX), ART_TILE_CRABMEAT);
    wb(m, a0.wrapping_add(OB_RENDER), 4);
    wb(m, a0.wrapping_add(OB_PRIORITY), 3);
    wb(m, a0.wrapping_add(OB_COLTYPE), 0x87);
    wb(m, a0.wrapping_add(OB_ACTWID), 8);
    ww(m, a0.wrapping_add(OB_VELY), u16_de(-0x400));
    wb(m, a0.wrapping_add(OB_ANIM), 7);

    crab_ball_move(m, a0)
}

fn crab_ball_move(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    animate_sprite(m, a0, ANI_CRAB)?;
    object_fall(m, a0);
    display_sprite(m, a0);
    let limit = rw(m, V_LIMITBTM2).wrapping_add(224);
    let y = rw(m, a0.wrapping_add(OB_Y));
    if limit < y {
        delete_object(m, a0);
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::abertura::rl;

    fn memoria_teste() -> Memoria {
        let mut rom = vec![0u8; 0x10000];
        // Ani_Crab script table at ANI_CRAB (0x9D98): 8 word offsets to animations
        // 0: stand, 1: standslope, 2: standsloperev, 3: walk, 4: walkslope, 5: walksloperev, 6: firing, 7: ball
        // Let's place animation data right after table (0x9D98 + 16 = 0x9DA8)
        let base_ani = usize::try_from(ANI_CRAB).unwrap_or(0);
        let data_ani = base_ani.wrapping_add(16);
        let offset = 16u16;
        let b0 = u8::try_from(offset >> 8).unwrap_or(0);
        let b1 = u8::try_from(offset & 0xFF).unwrap_or(0);
        for i in 0..8usize {
            let idx = base_ani.wrapping_add(i.wrapping_mul(2));
            rom[idx] = b0;
            rom[idx.wrapping_add(1)] = b1;
        }
        rom[data_ani] = 15;
        rom[data_ani.wrapping_add(1)] = 0;
        rom[data_ani.wrapping_add(2)] = 0xFF;

        Memoria::nova(rom)
    }

    #[test]
    fn crabmeat_inicializa_propriedades() {
        let mut m = memoria_teste();
        let a0 = V_LVLOBJSPACE;
        wb(&mut m, a0.wrapping_add(OB_ID), ID_CRABMEAT);
        wb(&mut m, a0.wrapping_add(OB_ROUTINE), 0);
        ww(&mut m, a0.wrapping_add(OB_X), 0x200);
        ww(&mut m, a0.wrapping_add(OB_Y), 0x100);

        executar(&mut m, a0).expect("executar");

        assert_eq!(rb(&mut m, a0.wrapping_add(OB_HEIGHT)), 16);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_WIDTH)), 8);
        assert_eq!(rl(&mut m, a0.wrapping_add(OB_MAP)), MAP_CRAB);
        assert_eq!(rw(&mut m, a0.wrapping_add(OB_GFX)), ART_TILE_CRABMEAT);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_COLTYPE)), 6);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ACTWID)), 21);
    }

    #[test]
    fn crabmeat_set_ani_angulos() {
        // Flat (angle 0..=5 and 252..=255)
        assert_eq!(crab_set_ani(0, 0), 0);
        assert_eq!(crab_set_ani(5, 0), 0);
        assert_eq!(crab_set_ani(0xFF, 0), 0);
        assert_eq!(crab_set_ani(0xFB, 0), 0);

        // Descending steep (angle >= 6): facing left = 1, facing right = 2
        assert_eq!(crab_set_ani(10, 1), 1);
        assert_eq!(crab_set_ani(10, 0), 2);

        // Ascending steep (angle <= 0xFA): facing left = 2, facing right = 1
        assert_eq!(crab_set_ani(0xF0, 1), 2);
        assert_eq!(crab_set_ani(0xF0, 0), 1);
    }

    #[test]
    fn crabmeat_projeteis_criacao() {
        let mut m = memoria_teste();
        let a0 = V_LVLOBJSPACE;
        wb(&mut m, a0.wrapping_add(OB_ID), ID_CRABMEAT);
        wb(&mut m, a0.wrapping_add(OB_ROUTINE), 2);
        wb(&mut m, a0.wrapping_add(OB_ROUTINE_2), 0);
        wb(&mut m, a0.wrapping_add(OB_RENDER), 0x84);
        wb(&mut m, a0.wrapping_add(CRAB_FLAGS), 2); // fire flag set
        ww(&mut m, a0.wrapping_add(CRAB_TIMEDELAY), 0);
        ww(&mut m, a0.wrapping_add(OB_X), 0x200);
        ww(&mut m, a0.wrapping_add(OB_Y), 0x150);

        executar(&mut m, a0).expect("executar");

        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ANIM)), 6);
        // Left fireball in slot 1
        let a1 = V_LVLOBJSPACE.wrapping_add(OBJECT_SIZE);
        assert_eq!(rb(&mut m, a1.wrapping_add(OB_ID)), ID_CRABMEAT);
        assert_eq!(rb(&mut m, a1.wrapping_add(OB_ROUTINE)), 6);
        assert_eq!(rw(&mut m, a1.wrapping_add(OB_X)), 0x200 - 0x10);
        assert_eq!(rw(&mut m, a1.wrapping_add(OB_VELX)), u16_de(-0x100));

        // Right fireball in slot 2
        let a2 = a1.wrapping_add(OBJECT_SIZE);
        assert_eq!(rb(&mut m, a2.wrapping_add(OB_ID)), ID_CRABMEAT);
        assert_eq!(rb(&mut m, a2.wrapping_add(OB_ROUTINE)), 6);
        assert_eq!(rw(&mut m, a2.wrapping_add(OB_X)), 0x200 + 0x10);
        assert_eq!(rw(&mut m, a2.wrapping_add(OB_VELX)), 0x100);
    }
}
