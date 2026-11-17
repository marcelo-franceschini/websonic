//! Chefe da Final Zone (Eggman, cilindros e bolas de plasma, objetos 0x85, 0x84 e 0x86).
use super::ram::*;
use super::{ErroAbertura, fisica, objetos, rb, rl, rw, wb, wl, ww};
use crate::memoria::Memoria;

const BOSS_FZ_X: u16 = 0x2450;
const BOSS_FZ_Y: u16 = 0x0510;
const BOSS_FZ_END: u16 = BOSS_FZ_X + 0x2B0; // 0x2700

const MAP_SEGG: u32 = 0x1_A1E4;
const ANI_SEGG: u32 = 0x1_A1B2;
const MAP_EGGCYL: u32 = 0x1_AE2C;
const MAP_FZLEGS: u32 = 0x1_ABD2;
const MAP_EGGMAN: u32 = 0x1_84B8;
const ANI_EGGMAN: u32 = 0x1_8468;
const MAP_PLASMA: u32 = 0x1_B25C;
const ANI_PLASMA: u32 = 0x1_B22C;

const ARTTILE_FZ_BOSS: u16 = 0x0300;
const ARTTILE_FZ_EGGMAN_FLEEING: u16 = 0x03A0;
const ARTTILE_FZ_EGGMAN_NO_VEHICLE: u16 = 0x0470;
const ARTTILE_EGGMAN: u16 = 0x0400;

const CHILD_CMD: u32 = 0x29;
const ATTACK_STATE: u32 = 0x30;
const ESCAPE_TIMER: u32 = 0x30;
const CHILD_COUNTER: u32 = 0x32;
const PARENT_OBJ: u32 = 0x34;
const HIT_FLASH: u32 = 0x35;
const PLASMA_PTR: u32 = 0x36;
const CYLINDER_PTR: u32 = 0x38;

const HAS_EGGMAN: u32 = 0x30;
const BASE_Y: u32 = 0x38;
const DISPLACEMENT: u32 = 0x3C;

const TARGET_X: u32 = 0x30;
const BALLS_ALIVE: u32 = 0x34;

const SFX_HIT_BOSS: u8 = 0xAC;
const SFX_RUMBLING: u8 = 0xB4;
const SFX_ELECTRIC: u8 = 0xC9;

/// Objeto 0x85: Chefe da Final Zone.
pub fn boss_final(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    match rotina {
        0 => boss_final_main(m, a0),
        2 => boss_final_eggman(m, a0),
        4 => boss_final_panel(m, a0),
        6 => boss_final_legs(m, a0),
        8 => boss_final_cockpit(m, a0),
        10 => boss_final_empty_ship(m, a0),
        12 => boss_final_flame(m, a0),
        _ => Ok(()),
    }
}

fn boss_final_main(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    // Configura o objeto principal Eggman (a0)
    wb(m, a0.wrapping_add(OB_ROUTINE), 2);
    wb(m, a0.wrapping_add(OB_BOSSHITS), 8);
    ww(m, a0.wrapping_add(ATTACK_STATE), 0xFFFF);
    ww(m, a0.wrapping_add(PARENT_OBJ), 0);
    ww(m, a0.wrapping_add(OB_X), 0x100);
    ww(m, a0.wrapping_add(OB_Y), 0x100);
    ww(m, a0.wrapping_add(OB_GFX), ARTTILE_FZ_EGGMAN_NO_VEHICLE);
    wl(m, a0.wrapping_add(OB_MAP), MAP_SEGG);
    wb(m, a0.wrapping_add(OB_PRIORITY), 4);
    wb(m, a0.wrapping_add(OB_ACTWID), 32);
    wb(m, a0.wrapping_add(OB_HEIGHT), 25);
    wb(m, a0.wrapping_add(OB_RENDER), 0x84);

    // Paineis e partes da nave em fuga
    let subobjs = [
        (
            4u8,
            BOSS_FZ_X.wrapping_add(0x160),
            BOSS_FZ_Y.wrapping_add(0x80),
            ARTTILE_FZ_BOSS,
            MAP_EGGCYL,
            1u8,
            18u8,
            8u8,
        ),
        (
            6u8,
            BOSS_FZ_X.wrapping_add(0x290),
            BOSS_FZ_Y.wrapping_add(0x86),
            ARTTILE_FZ_EGGMAN_FLEEING,
            MAP_FZLEGS,
            3u8,
            20u8,
            12u8,
        ),
        (
            8u8,
            BOSS_FZ_X.wrapping_add(0x290),
            BOSS_FZ_Y.wrapping_add(0x86),
            ARTTILE_FZ_EGGMAN_NO_VEHICLE,
            MAP_SEGG,
            3u8,
            32u8,
            28u8,
        ),
        (
            10u8,
            BOSS_FZ_X.wrapping_add(0x290),
            BOSS_FZ_Y.wrapping_add(0x86),
            ARTTILE_EGGMAN,
            MAP_EGGMAN,
            3u8,
            32u8,
            32u8,
        ),
        (
            12u8,
            BOSS_FZ_X.wrapping_add(0x290),
            BOSS_FZ_Y.wrapping_add(0x86),
            ARTTILE_EGGMAN,
            MAP_EGGMAN,
            3u8,
            8u8,
            3u8,
        ),
    ];

    for (rout, x, y, gfx, map, prio, wid, hgt) in subobjs {
        if let Some(a1) = objetos::find_free_obj(m) {
            wb(m, a1.wrapping_add(OB_ID), 0x85);
            wb(m, a1.wrapping_add(OB_ROUTINE), rout);
            ww(m, a1.wrapping_add(OB_X), x);
            ww(m, a1.wrapping_add(OB_Y), y);
            ww(m, a1.wrapping_add(OB_GFX), gfx);
            wl(m, a1.wrapping_add(OB_MAP), map);
            wb(m, a1.wrapping_add(OB_PRIORITY), prio);
            wb(m, a1.wrapping_add(OB_ACTWID), wid);
            wb(m, a1.wrapping_add(OB_HEIGHT), hgt);
            wb(m, a1.wrapping_add(OB_RENDER), 0x84);
            wl(m, a1.wrapping_add(PARENT_OBJ), a0);
        }
    }

    // Gerador de plasma (objeto 0x86)
    if let Some(a1) = objetos::find_free_obj(m) {
        wb(m, a1.wrapping_add(OB_ID), 0x86);
        ww(m, a0.wrapping_add(PLASMA_PTR), a1 as u16);
        wl(m, a1.wrapping_add(PARENT_OBJ), a0);
    }

    // 4 cilindros esmagadores (objeto 0x84)
    for i in 0..4u32 {
        if let Some(a1) = objetos::find_free_obj(m) {
            let off = i.wrapping_mul(2);
            ww(
                m,
                a0.wrapping_add(CYLINDER_PTR).wrapping_add(off),
                a1 as u16,
            );
            wb(m, a1.wrapping_add(OB_ID), 0x84);
            wl(m, a1.wrapping_add(PARENT_OBJ), a0);
            wb(m, a1.wrapping_add(OB_SUBTYPE), (i * 2) as u8);
        }
    }

    Ok(())
}

fn boss_final_eggman(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let subrout = rb(m, a0.wrapping_add(PARENT_OBJ));
    match subrout {
        0 => eggman_wait(m, a0),
        2 => eggman_crush(m, a0)?,
        4 => eggman_plasma(m, a0),
        6 => eggman_fall(m, a0),
        8 => eggman_run(m, a0),
        10 => eggman_jump(m, a0),
        12 => eggman_ship(m, a0),
        _ => eggman_escape(m, a0)?,
    }
    objetos::display_sprite(m, a0);
    Ok(())
}

fn eggman_wait(m: &mut Memoria, a0: u32) {
    if rw(m, V_SCREENPOSX) >= BOSS_FZ_X {
        wb(m, a0.wrapping_add(PARENT_OBJ), 2);
    }
}

const CYLINDER_PAIRS: [(usize, usize); 4] = [
    (0, 1), // top-left (0), top-right (1)
    (1, 2), // top-right (1), bottom-left (2)
    (2, 3), // bottom-left (2), bottom-right (3)
    (3, 0), // bottom-right (3), top-left (0)
];

fn eggman_crush(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let state = rw(m, a0.wrapping_add(ATTACK_STATE));
    if state == 0xFFFF {
        // Escolhe um par aleatorio de cilindros
        let rnd = rw(m, V_RANDOM);
        let pair_idx = ((rnd >> 2) & 3) as usize;
        let (mut c_egg, mut c_decoy) = CYLINDER_PAIRS[pair_idx];
        if rnd & 1 != 0 {
            core::mem::swap(&mut c_egg, &mut c_decoy);
        }

        let ptr_egg = ext(rw(
            m,
            a0.wrapping_add(CYLINDER_PTR)
                .wrapping_add((c_egg * 2) as u32),
        ));
        let ptr_decoy = ext(rw(
            m,
            a0.wrapping_add(CYLINDER_PTR)
                .wrapping_add((c_decoy * 2) as u32),
        ));

        wb(m, ptr_egg.wrapping_add(CHILD_CMD), 0xFF);
        ww(m, ptr_egg.wrapping_add(HAS_EGGMAN), 0xFFFF);

        wb(m, ptr_decoy.wrapping_add(CHILD_CMD), 1);
        ww(m, ptr_decoy.wrapping_add(HAS_EGGMAN), 0);

        ww(m, a0.wrapping_add(ATTACK_STATE), c_egg as u16);
        ww(m, a0.wrapping_add(CHILD_COUNTER), 1);
        wb(m, a0.wrapping_add(HIT_FLASH), 0);
        wb(m, V_SOUNDQUEUE1, SFX_RUMBLING);
    }

    let child_counter = s16(rw(m, a0.wrapping_add(CHILD_COUNTER)));
    if child_counter < 0 {
        // Cilindros terminaram de retrair
        if rb(m, a0.wrapping_add(OB_BOSSHITS)) == 0 {
            // Derrotado!
            wb(m, a0.wrapping_add(PARENT_OBJ), 6); // eggman_fall
            ww(m, a0.wrapping_add(OB_X), BOSS_FZ_X.wrapping_add(0x170));
            ww(m, a0.wrapping_add(OB_Y), BOSS_FZ_Y.wrapping_add(0x2C));
            wb(m, a0.wrapping_add(OB_HEIGHT), 20);
            return Ok(());
        }
        wb(m, a0.wrapping_add(PARENT_OBJ), 4); // plasma
        ww(m, a0.wrapping_add(ATTACK_STATE), 0xFFFF);
        ww(m, a0.wrapping_add(CHILD_COUNTER), 0);
        return Ok(());
    }

    // Olha para Sonic
    let px = rw(m, V_PLAYER.wrapping_add(OB_X));
    let egg_x = rw(m, a0.wrapping_add(OB_X));
    let st = rb(m, a0.wrapping_add(OB_STATUS));
    if px >= egg_x {
        wb(m, a0.wrapping_add(OB_STATUS), st | STATUS_XFLIP);
    } else {
        wb(m, a0.wrapping_add(OB_STATUS), st & !STATUS_XFLIP);
    }

    // Colisao solida do pistao do Eggman
    let d1 = 32u16.wrapping_add(SONIC_SOLID_WIDTH);
    let _ = objetos::solid_object(m, a0, d1, 20, 20);

    // Verifica se Sonic atacou
    let flash = rb(m, a0.wrapping_add(HIT_FLASH));
    if flash == 0 {
        if rb(m, V_PLAYER.wrapping_add(OB_ANIM)) == ANIM_ROLL {
            let p_vx = if rb(m, a0.wrapping_add(OB_STATUS)) & STATUS_XFLIP != 0 {
                0x0300
            } else {
                u16_de(-0x300i16)
            };
            ww(m, V_PLAYER.wrapping_add(OB_VELX), p_vx);
            let hits = rb(m, a0.wrapping_add(OB_BOSSHITS));
            if hits > 0 {
                let novos_hits = hits.wrapping_sub(1);
                wb(m, a0.wrapping_add(OB_BOSSHITS), novos_hits);
                wb(m, a0.wrapping_add(HIT_FLASH), 100);
                wb(m, V_SOUNDQUEUE1, SFX_HIT_BOSS);
            }
        }
    } else {
        let novo_flash = flash.wrapping_sub(1);
        wb(m, a0.wrapping_add(HIT_FLASH), novo_flash);
        if novo_flash > 0 {
            wb(m, a0.wrapping_add(OB_ANIM), 3); // dano
        } else {
            wb(m, a0.wrapping_add(OB_ANIM), 1); // riso
        }
    }

    objetos::animate_sprite(m, a0, ANI_SEGG)?;
    Ok(())
}

fn eggman_plasma(m: &mut Memoria, a0: u32) {
    let state = rw(m, a0.wrapping_add(ATTACK_STATE));
    if state == 0xFFFF {
        let plasma_addr = ext(rw(m, a0.wrapping_add(PLASMA_PTR)));
        wb(m, plasma_addr.wrapping_add(CHILD_CMD), 0xFF);
        wb(m, V_SOUNDQUEUE1, SFX_ELECTRIC);
        ww(m, a0.wrapping_add(ATTACK_STATE), 0);
    }

    if rw(m, V_VBLANK_WORD) & 0xF == 0 {
        wb(m, V_SOUNDQUEUE1, SFX_ELECTRIC);
    }

    if s16(rw(m, a0.wrapping_add(CHILD_COUNTER))) != 0 {
        wb(m, a0.wrapping_add(PARENT_OBJ), 2); // volta para esmagamento
        ww(m, a0.wrapping_add(ATTACK_STATE), 0xFFFF);
        ww(m, a0.wrapping_add(CHILD_COUNTER), 0);
    }
}

fn eggman_fall(m: &mut Memoria, a0: u32) {
    wb(m, a0.wrapping_add(OB_ACTWID), 48);
    let st = rb(m, a0.wrapping_add(OB_STATUS));
    wb(m, a0.wrapping_add(OB_STATUS), st | STATUS_XFLIP);
    fisica::speed_to_pos(m, a0);
    wb(m, a0.wrapping_add(OB_FRAME), 6);
    let vy = rw(m, a0.wrapping_add(OB_VELY)).wrapping_add(0x10);
    ww(m, a0.wrapping_add(OB_VELY), vy);

    let cur_y = rw(m, a0.wrapping_add(OB_Y));
    if cur_y >= BOSS_FZ_Y.wrapping_add(0x8C) {
        ww(m, a0.wrapping_add(OB_Y), BOSS_FZ_Y.wrapping_add(0x8C));
        wb(m, a0.wrapping_add(PARENT_OBJ), 8); // eggman_run
        wb(m, a0.wrapping_add(OB_ACTWID), 32);
        ww(m, a0.wrapping_add(OB_VELX), 0x100);
        ww(m, a0.wrapping_add(OB_VELY), u16_de(-0x100i16));
        let dle = rb(m, V_DLE_ROUTINE).wrapping_add(2);
        wb(m, V_DLE_ROUTINE, dle);
    }
    eggman_screen_scroll(m, a0);
}

fn eggman_run(m: &mut Memoria, a0: u32) {
    let st = rb(m, a0.wrapping_add(OB_STATUS));
    wb(m, a0.wrapping_add(OB_STATUS), st | STATUS_XFLIP);
    wb(m, a0.wrapping_add(OB_ANIM), 4);
    fisica::speed_to_pos(m, a0);
    let vy = rw(m, a0.wrapping_add(OB_VELY)).wrapping_add(0x10);
    ww(m, a0.wrapping_add(OB_VELY), vy);

    let cur_y = rw(m, a0.wrapping_add(OB_Y));
    if cur_y >= BOSS_FZ_Y.wrapping_add(0x93) {
        ww(m, a0.wrapping_add(OB_VELY), u16_de(-0x40i16));
    }

    let mut vx = 0x400u16;
    let cur_x = rw(m, a0.wrapping_add(OB_X));
    let player_x = rw(m, V_PLAYER.wrapping_add(OB_X));
    if cur_x < player_x {
        vx = 0x500;
    } else {
        let diff = cur_x.wrapping_sub(player_x);
        if diff >= 0x70 {
            vx = vx.saturating_sub(0x100);
        }
        if diff >= 0x78 {
            vx = vx.saturating_sub(0x100);
        }
        if diff >= 0x80 {
            vx = vx.saturating_sub(0x80);
        }
        if diff >= 0x88 {
            vx = vx.saturating_sub(0x80);
        }
        if diff >= 0x90 {
            vx = vx.saturating_sub(0x80);
        }
        if diff >= 0xC8 {
            vx = 0;
        }
    }
    ww(m, a0.wrapping_add(OB_VELX), vx);

    if cur_x >= BOSS_FZ_X.wrapping_add(0x250) {
        ww(m, a0.wrapping_add(OB_X), BOSS_FZ_X.wrapping_add(0x250));
        ww(m, a0.wrapping_add(OB_VELX), 0x240);
        ww(m, a0.wrapping_add(OB_VELY), u16_de(-0x4C0i16));
        wb(m, a0.wrapping_add(PARENT_OBJ), 10); // jump
    }
    eggman_animate(m, a0);
    eggman_screen_scroll(m, a0);
}

fn eggman_jump(m: &mut Memoria, a0: u32) {
    fisica::speed_to_pos(m, a0);
    let cur_x = rw(m, a0.wrapping_add(OB_X));
    if cur_x >= BOSS_FZ_X.wrapping_add(0x290) {
        ww(m, a0.wrapping_add(OB_VELX), 0);
    }
    let vy = rw(m, a0.wrapping_add(OB_VELY)).wrapping_add(0x34);
    ww(m, a0.wrapping_add(OB_VELY), vy);
    if s16(vy) >= 0 {
        let cur_y = rw(m, a0.wrapping_add(OB_Y));
        if cur_y >= BOSS_FZ_Y.wrapping_add(0x82) {
            ww(m, a0.wrapping_add(OB_Y), BOSS_FZ_Y.wrapping_add(0x82));
            ww(m, a0.wrapping_add(OB_VELY), 0);
        }
    }
    let vx = rw(m, a0.wrapping_add(OB_VELX));
    let cur_vy = rw(m, a0.wrapping_add(OB_VELY));
    if vx == 0 && cur_vy == 0 {
        wb(m, a0.wrapping_add(PARENT_OBJ), 12); // ship
        ww(m, a0.wrapping_add(OB_VELY), u16_de(-0x180i16));
        wb(m, a0.wrapping_add(OB_BOSSHITS), 1);
    }
    eggman_animate(m, a0);
    eggman_screen_scroll(m, a0);
}

fn eggman_ship(m: &mut Memoria, a0: u32) {
    wl(m, a0.wrapping_add(OB_MAP), MAP_EGGMAN);
    ww(m, a0.wrapping_add(OB_GFX), ARTTILE_EGGMAN);
    wb(m, a0.wrapping_add(OB_ANIM), 0);
    let st = rb(m, a0.wrapping_add(OB_STATUS));
    wb(m, a0.wrapping_add(OB_STATUS), st | STATUS_XFLIP);
    fisica::speed_to_pos(m, a0);

    let cur_y = rw(m, a0.wrapping_add(OB_Y));
    if cur_y < BOSS_FZ_Y.wrapping_add(0x34) {
        ww(m, a0.wrapping_add(OB_VELX), 0x180);
        ww(m, a0.wrapping_add(OB_VELY), u16_de(-0x18i16));
        wb(m, a0.wrapping_add(OB_COLTYPE), 0x48 | 0x80);
        wb(m, a0.wrapping_add(PARENT_OBJ), 14); // escape
    }
    eggman_animate(m, a0);
}

fn eggman_escape(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let st = rb(m, a0.wrapping_add(OB_STATUS));
    wb(m, a0.wrapping_add(OB_STATUS), st | STATUS_XFLIP);
    fisica::speed_to_pos(m, a0);

    let t = rw(m, a0.wrapping_add(ESCAPE_TIMER));
    if t == 0 {
        if rb(m, a0.wrapping_add(OB_COLTYPE)) == 0 {
            ww(m, a0.wrapping_add(ESCAPE_TIMER), 30);
            wb(m, V_SOUNDQUEUE1, SFX_HIT_BOSS);
        }
    } else {
        let novo_t = t.wrapping_sub(1);
        ww(m, a0.wrapping_add(ESCAPE_TIMER), novo_t);
        if novo_t == 0 {
            if rb(m, a0.wrapping_add(OB_STATUS)) & 0x80 == 0 {
                wb(m, a0.wrapping_add(OB_COLTYPE), 0x48 | 0x80);
            } else {
                ww(m, a0.wrapping_add(OB_VELY), 0x60);
            }
        }
    }

    let px = rw(m, V_PLAYER.wrapping_add(OB_X));
    if px >= BOSS_FZ_END.wrapping_add(0x90) {
        wb(m, F_LOCKCTRL, 1);
        ww(m, V_JPADHOLD2, 0);
        ww(m, V_PLAYER.wrapping_add(OB_INERTIA), 0);
        if s16(rw(m, a0.wrapping_add(OB_VELY))) < 0 {
            ww(m, V_JPADHOLD2, 0x0100); // olha para cima
        }
    }
    if px >= BOSS_FZ_END.wrapping_add(0xE0) {
        ww(
            m,
            V_PLAYER.wrapping_add(OB_X),
            BOSS_FZ_END.wrapping_add(0xE0),
        );
    }

    let cur_x = rw(m, a0.wrapping_add(OB_X));
    if cur_x >= BOSS_FZ_END.wrapping_add(0x200) && rb(m, a0.wrapping_add(OB_RENDER)) & 0x80 == 0 {
        // Modo do jogo muda para Ending ($18)!
        wb(m, V_GAMEMODE, GM_ENDING_ID);
        objetos::delete_object(m, a0);
        return Ok(());
    }

    eggman_animate(m, a0);
    Ok(())
}

fn eggman_animate(m: &mut Memoria, a0: u32) {
    let _ = objetos::animate_sprite(m, a0, ANI_SEGG);
}

fn eggman_screen_scroll(m: &mut Memoria, a0: u32) {
    let r2 = rw(m, V_LIMITRIGHT2);
    if r2 < BOSS_FZ_END {
        ww(m, V_LIMITRIGHT2, r2.wrapping_add(2));
    }
    if rb(m, a0.wrapping_add(PARENT_OBJ)) < 12 {
        let _x = rw(m, a0.wrapping_add(OB_X));
        let d1 = 16u16.wrapping_add(SONIC_SOLID_WIDTH);
        let _ = objetos::solid_object(m, a0, d1, 112, 113);
    }
}

fn boss_final_panel(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    wb(m, a0.wrapping_add(OB_FRAME), 5);
    let px = rw(m, V_PLAYER.wrapping_add(OB_X));
    let cur_x = rw(m, a0.wrapping_add(OB_X));
    if px >= cur_x && rb(m, a0.wrapping_add(OB_RENDER)) & 0x80 == 0 {
        objetos::delete_object(m, a0);
        return Ok(());
    }
    objetos::display_sprite(m, a0);
    Ok(())
}

fn boss_final_legs(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let tf = rb(m, a0.wrapping_add(OB_TIMEFRAME));
    let (novo_tf, estouro) = tf.overflowing_sub(1);
    wb(m, a0.wrapping_add(OB_TIMEFRAME), novo_tf);
    if estouro {
        let fr = rb(m, a0.wrapping_add(OB_FRAME)).wrapping_add(1);
        wb(m, a0.wrapping_add(OB_FRAME), fr);
        if fr > 2 {
            objetos::delete_object(m, a0);
            return Ok(());
        }
    }
    boss_final_flame_pos(m, a0);
    objetos::display_sprite(m, a0);
    Ok(())
}

fn boss_final_cockpit(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let parent = ext(rw(m, a0.wrapping_add(PARENT_OBJ)));
    if rb(m, parent.wrapping_add(OB_ID)) != 0x85 {
        objetos::delete_object(m, a0);
        return Ok(());
    }
    let st = rb(m, a0.wrapping_add(OB_STATUS));
    wb(m, a0.wrapping_add(OB_STATUS), st | STATUS_XFLIP);
    let r = rb(m, parent.wrapping_add(PARENT_OBJ));
    if r == 10 {
        wb(m, a0.wrapping_add(OB_FRAME), 0);
    } else if r > 10 {
        objetos::delete_object(m, a0);
        return Ok(());
    } else {
        wb(m, a0.wrapping_add(OB_FRAME), 6);
    }
    boss_final_flame_pos(m, a0);
    objetos::display_sprite(m, a0);
    Ok(())
}

fn boss_final_empty_ship(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    wb(m, a0.wrapping_add(OB_FRAME), 0);
    let st = rb(m, a0.wrapping_add(OB_STATUS));
    wb(m, a0.wrapping_add(OB_STATUS), st | STATUS_XFLIP);
    let parent = ext(rw(m, a0.wrapping_add(PARENT_OBJ)));
    if rb(m, parent.wrapping_add(PARENT_OBJ)) >= 12 {
        objetos::delete_object(m, a0);
        return Ok(());
    }
    boss_final_flame_pos(m, a0);
    objetos::display_sprite(m, a0);
    Ok(())
}

fn boss_final_flame(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let parent = ext(rw(m, a0.wrapping_add(PARENT_OBJ)));
    if rb(m, parent.wrapping_add(OB_ID)) != 0x85 {
        objetos::delete_object(m, a0);
        return Ok(());
    }
    let r = rb(m, parent.wrapping_add(PARENT_OBJ));
    if r >= 12 && rw(m, parent.wrapping_add(OB_VELX)) != 0 {
        wb(m, a0.wrapping_add(OB_ANIM), 0x0B);
    } else {
        wb(m, a0.wrapping_add(OB_ANIM), 7);
    }
    let _ = objetos::animate_sprite(m, a0, ANI_EGGMAN);
    boss_final_flame_pos(m, a0);
    objetos::display_sprite(m, a0);
    Ok(())
}

fn boss_final_flame_pos(m: &mut Memoria, a0: u32) {
    let parent = ext(rw(m, a0.wrapping_add(PARENT_OBJ)));
    let px = rw(m, parent.wrapping_add(OB_X));
    let py = rw(m, parent.wrapping_add(OB_Y));
    ww(m, a0.wrapping_add(OB_X), px);
    ww(m, a0.wrapping_add(OB_Y), py);
}

/// Objeto 0x84: Cilindro em que Eggman se esconde e ataca (FZ).
pub fn eggman_cylinder(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    match rotina {
        0 => cylinder_main(m, a0),
        2 => cylinder_action(m, a0),
        4 => cylinder_move(m, a0),
        _ => Ok(()),
    }
}

const CYLINDER_POS_DATA: [(u16, u16); 4] = [
    (BOSS_FZ_X + 0x80, BOSS_FZ_Y + 0x110),
    (BOSS_FZ_X + 0x100, BOSS_FZ_Y + 0x110),
    (BOSS_FZ_X + 0x40, BOSS_FZ_Y - 0x50),
    (BOSS_FZ_X + 0xC0, BOSS_FZ_Y - 0x50),
];

fn cylinder_main(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let sub = (rb(m, a0.wrapping_add(OB_SUBTYPE)) / 2) as usize;
    let (x, y) = CYLINDER_POS_DATA[sub.min(3)];

    wb(m, a0.wrapping_add(OB_RENDER), 0x84);
    ww(m, a0.wrapping_add(OB_GFX), ARTTILE_FZ_BOSS);
    wl(m, a0.wrapping_add(OB_MAP), MAP_EGGCYL);
    ww(m, a0.wrapping_add(OB_X), x);
    ww(m, a0.wrapping_add(OB_Y), y);
    ww(m, a0.wrapping_add(BASE_Y), y);
    wb(m, a0.wrapping_add(OB_ACTWID), 32);
    wb(m, a0.wrapping_add(OB_HEIGHT), 96);
    wb(m, a0.wrapping_add(OB_PRIORITY), 3);
    wb(m, a0.wrapping_add(OB_ROUTINE), 2);
    cylinder_action(m, a0)
}

fn cylinder_action(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    if rb(m, a0.wrapping_add(OB_SUBTYPE)) > 2 {
        let rnd = rb(m, a0.wrapping_add(OB_RENDER));
        wb(m, a0.wrapping_add(OB_RENDER), rnd | 2); // yflip
    }
    wl(m, a0.wrapping_add(DISPLACEMENT), 0);
    if rb(m, a0.wrapping_add(CHILD_CMD)) != 0 {
        wb(m, a0.wrapping_add(OB_ROUTINE), 4);
    }
    cylinder_update_pos(m, a0)
}

fn cylinder_move(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let sub = rb(m, a0.wrapping_add(OB_SUBTYPE));
    let mut disp = rl(m, a0.wrapping_add(DISPLACEMENT));

    if sub <= 2 {
        // Cilindro do chao
        if rb(m, a0.wrapping_add(CHILD_CMD)) != 0 {
            // Estendendo
            let d_w = (disp >> 16) as u16;
            if s16(d_w) < -16 {
                disp = disp.wrapping_sub(0x2_8000);
            }
            disp = disp.wrapping_sub(0x8000);
            if s16((disp >> 16) as u16) <= -0xA0 {
                disp = u32_de(-0xA0i32) << 16;
                wb(m, a0.wrapping_add(CHILD_CMD), 0);
            }
        } else {
            // Retraindo
            let parent = ext(rw(m, a0.wrapping_add(PARENT_OBJ)));
            let speed = if rb(m, parent.wrapping_add(OB_BOSSHITS)) == 0 {
                0x1_0000
            } else {
                0x2_0000
            };
            disp = disp.wrapping_add(speed);
            if s16((disp >> 16) as u16) >= 0 {
                disp = 0;
                let c = s16(rw(m, parent.wrapping_add(CHILD_COUNTER))).wrapping_sub(1);
                ww(m, parent.wrapping_add(CHILD_COUNTER), as_u16(c));
                ww(m, parent.wrapping_add(ATTACK_STATE), 0);
                wb(m, a0.wrapping_add(OB_ROUTINE), 2);
            }
        }
    } else {
        // Cilindro do teto
        let rnd = rb(m, a0.wrapping_add(OB_RENDER));
        wb(m, a0.wrapping_add(OB_RENDER), rnd | 2);

        if rb(m, a0.wrapping_add(CHILD_CMD)) != 0 {
            let d_w = (disp >> 16) as u16;
            if s16(d_w) > 16 {
                disp = disp.wrapping_add(0x2_8000);
            }
            disp = disp.wrapping_add(0x8000);
            if s16((disp >> 16) as u16) >= 0xA0 {
                disp = 0x00A0_0000;
                wb(m, a0.wrapping_add(CHILD_CMD), 0);
            }
        } else {
            let parent = ext(rw(m, a0.wrapping_add(PARENT_OBJ)));
            let speed = if rb(m, parent.wrapping_add(OB_BOSSHITS)) == 0 {
                0x1_0000
            } else {
                0x2_0000
            };
            disp = disp.wrapping_sub(speed);
            if s16((disp >> 16) as u16) <= 0 {
                disp = 0;
                let c = s16(rw(m, parent.wrapping_add(CHILD_COUNTER))).wrapping_sub(1);
                ww(m, parent.wrapping_add(CHILD_COUNTER), as_u16(c));
                ww(m, parent.wrapping_add(ATTACK_STATE), 0);
                wb(m, a0.wrapping_add(OB_ROUTINE), 2);
            }
        }
    }
    wl(m, a0.wrapping_add(DISPLACEMENT), disp);
    cylinder_update_pos(m, a0)
}

fn cylinder_update_pos(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let disp = rl(m, a0.wrapping_add(DISPLACEMENT));
    let base_y = rw(m, a0.wrapping_add(BASE_Y));
    let pixel_disp = (disp >> 16) as u16;
    let cur_y = base_y.wrapping_add(pixel_disp);
    ww(m, a0.wrapping_add(OB_Y), cur_y);

    // Se este cilindro contem Eggman, atualiza a posicao do Eggman
    if rb(m, a0.wrapping_add(OB_ROUTINE)) == 4 && rw(m, a0.wrapping_add(HAS_EGGMAN)) == 0xFFFF {
        let parent = ext(rw(m, a0.wrapping_add(PARENT_OBJ)));
        let egg_y = if rb(m, a0.wrapping_add(OB_SUBTYPE)) <= 2 {
            cur_y.wrapping_sub(10)
        } else {
            cur_y.wrapping_add(14)
        };
        ww(m, parent.wrapping_add(OB_Y), egg_y);
        let cur_x = rw(m, a0.wrapping_add(OB_X));
        ww(m, parent.wrapping_add(OB_X), cur_x);
    }

    // Colisao solida
    let cur_x = rw(m, a0.wrapping_add(OB_X));
    let d1 = 32u16.wrapping_add(SONIC_SOLID_WIDTH);
    let _ = objetos::solid_object(m, a0, d1, 96, 97);

    // Calcula frame do cilindro conforme extensao
    let mut frame = 0u8;
    if rb(m, a0.wrapping_add(OB_SUBTYPE)) <= 2 {
        let mut d1 = s16(pixel_disp).abs() as u16;
        if d1 >= 8 {
            frame = 1;
            d1 = d1.wrapping_sub(8);
            frame = frame.wrapping_add((d1 >> 4) as u8);
        }
    } else {
        let mut d1 = pixel_disp;
        if d1 >= 0x27 {
            frame = 1;
            d1 = d1.wrapping_sub(0x27);
            frame = frame.wrapping_add((d1 >> 4) as u8);
        }
    }
    wb(m, a0.wrapping_add(OB_FRAME), frame.min(4));

    // Remocao quando fora da tela apos a derrota
    let px = rw(m, V_PLAYER.wrapping_add(OB_X));
    if px.wrapping_sub(cur_x) > 0x140 && rb(m, a0.wrapping_add(OB_RENDER)) & 0x80 == 0 {
        objetos::delete_object(m, a0);
        return Ok(());
    }

    objetos::display_sprite(m, a0);
    Ok(())
}

/// Objeto 0x86: Bolas de plasma e lancador (FZ).
pub fn boss_plasma(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    match rotina {
        0 => plasma_main(m, a0),
        2 | 4 => plasma_make_balls(m, a0),
        6 => plasma_finish(m, a0),
        8 => plasma_balls(m, a0),
        _ => Ok(()),
    }
}

fn plasma_main(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    ww(m, a0.wrapping_add(OB_X), BOSS_FZ_X.wrapping_add(0x160));
    ww(m, a0.wrapping_add(OB_Y), BOSS_FZ_Y.wrapping_add(0x2C));
    wb(m, a0.wrapping_add(OB_RENDER), 0x84);
    ww(m, a0.wrapping_add(OB_GFX), ARTTILE_FZ_BOSS | 0x4000);
    wl(m, a0.wrapping_add(OB_MAP), MAP_PLASMA);
    wb(m, a0.wrapping_add(OB_PRIORITY), 3);
    wb(m, a0.wrapping_add(OB_ROUTINE), 4);
    plasma_make_balls(m, a0)
}

fn plasma_make_balls(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    if rb(m, a0.wrapping_add(CHILD_CMD)) != 0 {
        wb(m, a0.wrapping_add(CHILD_CMD), 0);
        let spread = rw(m, a0.wrapping_add(TARGET_X)).wrapping_add(4);
        ww(m, a0.wrapping_add(TARGET_X), spread);
        ww(m, a0.wrapping_add(CHILD_COUNTER), 0);

        for i in 0..4u32 {
            if let Some(a1) = objetos::find_free_obj(m) {
                let px = rw(m, a0.wrapping_add(OB_X));
                wb(m, a1.wrapping_add(OB_ID), 0x86);
                ww(m, a1.wrapping_add(OB_X), px);
                ww(m, a1.wrapping_add(OB_Y), BOSS_FZ_Y.wrapping_add(0x2C));
                wb(m, a1.wrapping_add(OB_ROUTINE), 8);
                ww(m, a1.wrapping_add(OB_GFX), ARTTILE_FZ_BOSS | 0x4000);
                wl(m, a1.wrapping_add(OB_MAP), MAP_PLASMA);
                wb(m, a1.wrapping_add(OB_HEIGHT), 12);
                wb(m, a1.wrapping_add(OB_WIDTH), 12);
                wb(m, a1.wrapping_add(OB_PRIORITY), 3);
                ww(m, a1.wrapping_add(OB_SUBTYPE), 62);
                wb(m, a1.wrapping_add(OB_RENDER), 0x84);
                wl(m, a1.wrapping_add(PARENT_OBJ), a0);

                let target = BOSS_FZ_X.wrapping_add(0x128).wrapping_sub(i as u16 * 0x4F);
                ww(m, a1.wrapping_add(TARGET_X), target);
                let c = rw(m, a0.wrapping_add(CHILD_COUNTER)).wrapping_add(1);
                ww(m, a0.wrapping_add(CHILD_COUNTER), c);
                ww(m, a0.wrapping_add(BALLS_ALIVE), c);
            }
        }
    }

    if rw(m, a0.wrapping_add(CHILD_COUNTER)) == 0 {
        wb(m, a0.wrapping_add(OB_ROUTINE), 6);
    }
    objetos::display_sprite(m, a0);
    Ok(())
}

fn plasma_finish(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    wb(m, a0.wrapping_add(OB_ANIM), 2);
    if rw(m, a0.wrapping_add(BALLS_ALIVE)) == 0 {
        wb(m, a0.wrapping_add(OB_ROUTINE), 2);
        let parent = ext(rw(m, a0.wrapping_add(PARENT_OBJ)));
        ww(m, parent.wrapping_add(CHILD_COUNTER), 0xFFFF);
    }
    objetos::display_sprite(m, a0);
    Ok(())
}

fn plasma_balls(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let r2 = rb(m, a0.wrapping_add(OB_2NDROUT));
    match r2 {
        0 => {
            let tx = rw(m, a0.wrapping_add(TARGET_X));
            let cur_x = rw(m, a0.wrapping_add(OB_X));
            let vx = tx.wrapping_sub(cur_x).wrapping_shl(4);
            ww(m, a0.wrapping_add(OB_VELX), vx);
            ww(m, a0.wrapping_add(OB_SUBTYPE), 180);
            wb(m, a0.wrapping_add(OB_2NDROUT), 2);
        }
        2 => {
            if rw(m, a0.wrapping_add(OB_VELX)) != 0 {
                fisica::speed_to_pos(m, a0);
                let tx = rw(m, a0.wrapping_add(TARGET_X));
                let cur_x = rw(m, a0.wrapping_add(OB_X));
                if (cur_x as i16 - tx as i16).abs() <= 8 {
                    ww(m, a0.wrapping_add(OB_X), tx);
                    ww(m, a0.wrapping_add(OB_VELX), 0);
                    let parent = ext(rw(m, a0.wrapping_add(PARENT_OBJ)));
                    let c = rw(m, parent.wrapping_add(CHILD_COUNTER)).wrapping_sub(1);
                    ww(m, parent.wrapping_add(CHILD_COUNTER), c);
                }
            } else {
                let t = rw(m, a0.wrapping_add(OB_SUBTYPE)).wrapping_sub(1);
                ww(m, a0.wrapping_add(OB_SUBTYPE), t);
                if t == 0 {
                    ww(m, a0.wrapping_add(OB_VELY), 0x200);
                    wb(m, a0.wrapping_add(OB_COLTYPE), 0x8B);
                    wb(m, a0.wrapping_add(OB_2NDROUT), 4);
                }
            }
        }
        _ => {
            fisica::speed_to_pos(m, a0);
            let cur_y = rw(m, a0.wrapping_add(OB_Y));
            if cur_y >= BOSS_FZ_Y.wrapping_add(0x100) {
                let parent = ext(rw(m, a0.wrapping_add(PARENT_OBJ)));
                let alive = rw(m, parent.wrapping_add(BALLS_ALIVE)).wrapping_sub(1);
                ww(m, parent.wrapping_add(BALLS_ALIVE), alive);
                objetos::delete_object(m, a0);
                return Ok(());
            }
        }
    }
    let _ = objetos::animate_sprite(m, a0, ANI_PLASMA);
    objetos::display_sprite(m, a0);
    Ok(())
}

fn s16(w: u16) -> i16 {
    i16::from_be_bytes(w.to_be_bytes())
}

fn as_u16(i: i16) -> u16 {
    u16::from_be_bytes(i.to_be_bytes())
}

fn u16_de(i: i16) -> u16 {
    u16::from_be_bytes(i.to_be_bytes())
}

fn u32_de(i: i32) -> u32 {
    u32::from_be_bytes(i.to_be_bytes())
}

fn ext(w: u16) -> u32 {
    0xFF_0000 | u32::from(w)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn inicializacao_fz_boss_e_partes() {
        let mut m = Memoria::nova(vec![0; 0x1000]);
        let a0: u32 = 0xFF_D800;
        wb(&mut m, a0.wrapping_add(OB_ID), 0x85);
        wb(&mut m, a0.wrapping_add(OB_ROUTINE), 0);

        let res = boss_final(&mut m, a0);
        assert!(res.is_ok());
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_BOSSHITS)), 8);
        assert_eq!(rw(&mut m, a0.wrapping_add(ATTACK_STATE)), 0xFFFF);

        // Verifica criacao das subpartes da nave e plasma
        let mut partes_fz = 0;
        let mut plasma = 0;
        let mut slot = a0.wrapping_add(OBJECT_SIZE);
        while slot < a0.wrapping_add(OBJECT_SIZE.wrapping_mul(8)) {
            let id = rb(&mut m, slot.wrapping_add(OB_ID));
            if id == 0x85 {
                partes_fz += 1;
            } else if id == 0x86 {
                plasma += 1;
            }
            slot = slot.wrapping_add(OBJECT_SIZE);
        }
        assert!(partes_fz >= 4);
        assert_eq!(plasma, 1);
    }
}
