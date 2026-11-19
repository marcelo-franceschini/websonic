//! Nave e fogo do chefe de Marble Zone (objeto 0x73 e 0x74, REV01).
use super::ram::*;
use super::{ErroAbertura, fisica, objetos, rb, rl, rw, wb, wl, ww};
use crate::memoria::Memoria;

const MAP_EGGMAN: u32 = 0x1_84B8;
const ANI_EGGMAN: u32 = 0x1_8468;
const MAP_BOSS_ITEMS: u32 = 0x1_8580;
const MAP_FIRE: u32 = 0x00B9F8;
const ANI_FIRE: u32 = 0x00EB52;

const BOSS_MZ_X: u16 = 0x1800;
const BOSS_MZ_Y: u16 = 0x0210;
const BOSS_MZ_END: u16 = 0x1800 + 0x160;

const BOSS_X: u32 = 0x30;
const BOSS_Y: u32 = 0x38;
const PARENT: u32 = 0x34;
const LAVA_TIMER: u32 = 0x34;
const SPREAD_X: u32 = 0x32;
const TIMER: u32 = 0x3C;
const FLASH: u32 = 0x3E;
const SINE_COUNTER: u32 = 0x3F;

fn s16(v: u16) -> i16 {
    i16::from_be_bytes(v.to_be_bytes())
}

fn as_u16(v: i16) -> u16 {
    u16::from_be_bytes(v.to_be_bytes())
}

fn signed_long(v: i32) -> u32 {
    u32::from_be_bytes(v.to_be_bytes())
}

fn next_free(m: &mut Memoria, a: u32) -> Option<u32> {
    let mut slot = a.wrapping_add(OBJECT_SIZE);
    while slot < V_LVLOBJEND {
        if rb(m, slot.wrapping_add(OB_ID)) == 0 {
            return Some(slot);
        }
        slot = slot.wrapping_add(OBJECT_SIZE);
    }
    None
}

fn boss_move(m: &mut Memoria, a: u32) {
    for (pos, vel) in [(BOSS_X, OB_VELX), (BOSS_Y, OB_VELY)] {
        let passo = i32::from(s16(rw(m, a.wrapping_add(vel)))).wrapping_shl(8);
        let valor = rl(m, a.wrapping_add(pos)).wrapping_add(signed_long(passo));
        wl(m, a.wrapping_add(pos), valor);
    }
}

pub fn executar(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    match rotina {
        0 => {
            preparar(m, a0);
            nave(m, a0)
        }
        2 => nave(m, a0),
        4 => rosto(m, a0),
        6 => chama(m, a0),
        8 => tubo(m, a0),
        _ => Ok(()),
    }
}

fn preparar(m: &mut Memoria, a0: u32) {
    let x = rw(m, a0.wrapping_add(OB_X));
    let y = rw(m, a0.wrapping_add(OB_Y));
    ww(m, a0.wrapping_add(BOSS_X), x);
    ww(m, a0.wrapping_add(BOSS_Y), y);
    wb(m, a0.wrapping_add(OB_COLTYPE), 0x0F);
    wb(m, a0.wrapping_add(0x21), 8); // 8 vidas do chefe

    let mut slot = a0;
    for (rot, anim, prio) in [(2, 0, 4), (4, 1, 4), (6, 7, 4), (8, 0, 3)] {
        wb(m, slot.wrapping_add(OB_ID), 0x73);
        wb(m, slot.wrapping_add(OB_ROUTINE), rot);
        wb(m, slot.wrapping_add(OB_ROUTINE_2), 0);
        wb(m, slot.wrapping_add(OB_ANIM), anim);
        wb(m, slot.wrapping_add(OB_PRIORITY), prio);
        wl(m, slot.wrapping_add(OB_MAP), MAP_EGGMAN);
        ww(m, slot.wrapping_add(OB_GFX), 0x400);
        wb(m, slot.wrapping_add(OB_RENDER), 4);
        wb(m, slot.wrapping_add(OB_ACTWID), 32);
        ww(m, slot.wrapping_add(OB_X), x);
        ww(m, slot.wrapping_add(OB_Y), y);
        wl(m, slot.wrapping_add(PARENT), a0);

        if rot == 8 {
            break;
        }
        let Some(prox) = next_free(m, slot) else {
            break;
        };
        slot = prox;
    }
    let st = rb(m, a0.wrapping_add(OB_STATUS));
    wb(m, a0.wrapping_add(OB_STATUS), st & !1);
}

fn nave(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let rot2 = rb(m, a.wrapping_add(OB_ROUTINE_2));
    match rot2 {
        0 => ship_start(m, a),
        2 => ship_move(m, a)?,
        4 => explode(m, a),
        6 => recover(m, a),
        8 => escape(m, a),
        _ => {}
    }
    ship_update(m, a);
    renderizar(m, a, ANI_EGGMAN)
}

fn ship_start(m: &mut Memoria, a: u32) {
    let sc = rb(m, a.wrapping_add(SINE_COUNTER)).wrapping_add(2);
    wb(m, a.wrapping_add(SINE_COUNTER), sc);
    let (seno, _) = fisica::calc_sine(m, sc);
    ww(m, a.wrapping_add(OB_VELY), as_u16(seno >> 2));
    ww(m, a.wrapping_add(OB_VELX), 0xFF00); // -0x100 para esquerda
    boss_move(m, a);

    if rw(m, a.wrapping_add(BOSS_X)) <= BOSS_MZ_X.wrapping_add(0x110) {
        wb(m, a.wrapping_add(OB_ROUTINE_2), 2);
        wb(m, a.wrapping_add(OB_SUBTYPE), 0);
        ww(m, a.wrapping_add(OB_VELX), 0);
        let rnd = fisica::random_number(m);
        wb(m, a.wrapping_add(LAVA_TIMER), (rnd & 0xFF) as u8);
    }
}

fn ship_move(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let sub = rb(m, a.wrapping_add(OB_SUBTYPE)) & 6;
    match sub {
        0 | 4 => chg_dir(m, a),
        2 | 6 => drop_fire(m, a),
        _ => {}
    }
    let novo_sub = rb(m, a.wrapping_add(OB_SUBTYPE)) & 6;
    wb(m, a.wrapping_add(OB_SUBTYPE), novo_sub);
    Ok(())
}

fn chg_dir(m: &mut Memoria, a: u32) {
    if rw(m, a.wrapping_add(OB_VELX)) == 0 {
        let alvo_y = BOSS_MZ_Y.wrapping_add(0x1C);
        let cur_y = rw(m, a.wrapping_add(BOSS_Y));
        if cur_y == alvo_y {
            // Swoop
            let vx: u16 = if rb(m, a.wrapping_add(OB_STATUS)) & 1 != 0 {
                0x200
            } else {
                0xFE00 // -0x200
            };
            ww(m, a.wrapping_add(OB_VELX), vx);
            ww(m, a.wrapping_add(OB_VELY), 0x100);
        } else {
            let vy: u16 = if cur_y < alvo_y { 0x40 } else { 0xFFC0 };
            ww(m, a.wrapping_add(OB_VELY), vy);
            boss_move(m, a);
            return;
        }
    }

    if rb(m, a.wrapping_add(FLASH)) < 0x18 {
        boss_move(m, a);
        let vy = rw(m, a.wrapping_add(OB_VELY)).wrapping_sub(4);
        ww(m, a.wrapping_add(OB_VELY), vy);
    }

    // Gerador de lava
    let lt = rb(m, a.wrapping_add(LAVA_TIMER));
    let (novo_lt, expira) = lt.overflowing_sub(1);
    wb(m, a.wrapping_add(LAVA_TIMER), novo_lt);
    if expira {
        if let Some(livre) = objetos::find_free_obj(m) {
            wb(m, livre.wrapping_add(OB_ID), 0x14); // LavaBall
            ww(m, livre.wrapping_add(OB_Y), BOSS_MZ_Y.wrapping_add(0xD8));
            let rnd = fisica::random_number(m);
            let rem = rnd % 0x50;
            let spawn_x = BOSS_MZ_X.wrapping_add(0x78).wrapping_add(rem);
            ww(m, livre.wrapping_add(OB_X), spawn_x);
            ww(m, livre.wrapping_add(OB_SUBTYPE), 0x00FF);
        }
        let rnd = fisica::random_number(m);
        let timer = ((rnd & 0x1F) as u8).wrapping_add(0x40);
        wb(m, a.wrapping_add(LAVA_TIMER), timer);
    }

    // Limites horizontais
    if rb(m, a.wrapping_add(OB_STATUS)) & 1 != 0 {
        let x = rw(m, a.wrapping_add(BOSS_X));
        if x >= BOSS_MZ_X.wrapping_add(0x110) {
            ww(m, a.wrapping_add(BOSS_X), BOSS_MZ_X.wrapping_add(0x110));
            ww(m, a.wrapping_add(OB_VELX), 0);
            ww(m, a.wrapping_add(OB_VELY), 0xFE80); // -0x180 (sobe)
            let s = rb(m, a.wrapping_add(OB_SUBTYPE)).wrapping_add(2);
            wb(m, a.wrapping_add(OB_SUBTYPE), s);
        }
    } else {
        let x = rw(m, a.wrapping_add(BOSS_X));
        if x <= BOSS_MZ_X.wrapping_add(0x30) {
            ww(m, a.wrapping_add(BOSS_X), BOSS_MZ_X.wrapping_add(0x30));
            ww(m, a.wrapping_add(OB_VELX), 0);
            ww(m, a.wrapping_add(OB_VELY), 0xFE80); // -0x180 (sobe)
            let s = rb(m, a.wrapping_add(OB_SUBTYPE)).wrapping_add(2);
            wb(m, a.wrapping_add(OB_SUBTYPE), s);
        }
    }
}

fn drop_fire(m: &mut Memoria, a: u32) {
    boss_move(m, a);
    let cur_y = rw(m, a.wrapping_add(BOSS_Y));
    let topo = BOSS_MZ_Y.wrapping_add(0x1C);
    if cur_y <= topo {
        ww(m, a.wrapping_add(BOSS_Y), topo);
        if rw(m, a.wrapping_add(OB_VELY)) != 0 {
            ww(m, a.wrapping_add(OB_VELY), 0);
            ww(m, a.wrapping_add(TIMER), 80);
            let st = rb(m, a.wrapping_add(OB_STATUS));
            wb(m, a.wrapping_add(OB_STATUS), st ^ 1);
            let bx = rw(m, a.wrapping_add(BOSS_X));
            let by = rw(m, a.wrapping_add(BOSS_Y));
            if let Some(livre) = objetos::find_free_obj(m) {
                ww(m, livre.wrapping_add(OB_X), bx);
                ww(m, livre.wrapping_add(OB_Y), by.wrapping_add(0x18));
                wb(m, livre.wrapping_add(OB_ID), 0x74);
                wb(m, livre.wrapping_add(OB_SUBTYPE), 1);
            }
        }
    }
    let t = rw(m, a.wrapping_add(TIMER));
    if t != 0 {
        let t_novo = t.wrapping_sub(1);
        ww(m, a.wrapping_add(TIMER), t_novo);
        if t_novo == 0 {
            let s = rb(m, a.wrapping_add(OB_SUBTYPE)).wrapping_add(2);
            wb(m, a.wrapping_add(OB_SUBTYPE), s);
        }
    }
}

fn explode(m: &mut Memoria, a: u32) {
    let t = s16(rw(m, a.wrapping_add(TIMER))).wrapping_sub(1);
    ww(m, a.wrapping_add(TIMER), as_u16(t));
    if t < 0 {
        let st = rb(m, a.wrapping_add(OB_STATUS));
        wb(m, a.wrapping_add(OB_STATUS), (st | 1) & !0x80);
        ww(m, a.wrapping_add(OB_VELX), 0);
        wb(m, a.wrapping_add(OB_ROUTINE_2), 6);
        ww(m, a.wrapping_add(TIMER), as_u16(-38));
        if rb(m, 0xFF_F7A7) == 0 {
            wb(m, 0xFF_F7A7, 1);
            ww(m, a.wrapping_add(OB_VELY), 0);
        }
        return;
    }
    super::boss_ghz::explosao(m, a);
}

fn recover(m: &mut Memoria, a: u32) {
    let t = s16(rw(m, a.wrapping_add(TIMER))).wrapping_add(1);
    ww(m, a.wrapping_add(TIMER), as_u16(t));
    if t == 0 {
        ww(m, a.wrapping_add(OB_VELY), 0);
        ww(m, a.wrapping_add(TIMER), 0);
    } else if t < 0 {
        let by = rw(m, a.wrapping_add(BOSS_Y));
        if by >= BOSS_MZ_Y.wrapping_add(0x60) {
            ww(m, a.wrapping_add(OB_VELY), 0);
            ww(m, a.wrapping_add(TIMER), 0);
        } else {
            let vy = rw(m, a.wrapping_add(OB_VELY)).wrapping_add(0x18);
            ww(m, a.wrapping_add(OB_VELY), vy);
        }
    } else if t < 48 {
        let vy = rw(m, a.wrapping_add(OB_VELY)).wrapping_sub(8);
        ww(m, a.wrapping_add(OB_VELY), vy);
    } else if t == 48 {
        ww(m, a.wrapping_add(OB_VELY), 0);
        wb(m, V_SOUNDQUEUE0, 0x83); // bgm_MZ
    } else if t >= 56 {
        wb(m, a.wrapping_add(OB_ROUTINE_2), 8);
    }
    boss_move(m, a);
}

fn escape(m: &mut Memoria, a: u32) {
    ww(m, a.wrapping_add(OB_VELX), 0x500);
    ww(m, a.wrapping_add(OB_VELY), 0xFFC0); // -0x40
    let borda = rw(m, V_LIMITRIGHT2);
    if borda < BOSS_MZ_END {
        ww(m, V_LIMITRIGHT2, borda.wrapping_add(2));
    } else if rw(m, a.wrapping_add(OB_X)) > BOSS_MZ_END.wrapping_add(0x80) {
        objetos::delete_object(m, a);
        return;
    }
    boss_move(m, a);
}

fn ship_update(m: &mut Memoria, a: u32) {
    let bx = rw(m, a.wrapping_add(BOSS_X));
    let by = rw(m, a.wrapping_add(BOSS_Y));
    ww(m, a.wrapping_add(OB_X), bx);
    ww(m, a.wrapping_add(OB_Y), by);

    if rb(m, a.wrapping_add(OB_ROUTINE_2)) >= 4 {
        return;
    }
    if rb(m, a.wrapping_add(OB_STATUS)) & 0x80 != 0 {
        fisica::add_points(m, 100);
        wb(m, a.wrapping_add(OB_ROUTINE_2), 4);
        ww(m, a.wrapping_add(TIMER), 180);
        ww(m, a.wrapping_add(OB_VELX), 0);
        return;
    }
    if rb(m, a.wrapping_add(OB_COLTYPE)) != 0 {
        return;
    }
    let mut flash = rb(m, a.wrapping_add(FLASH));
    if flash == 0 {
        flash = 0x28;
        wb(m, V_SOUNDQUEUE1, 0xAC);
    }
    let cor = if rw(m, V_PALETTE.wrapping_add(0x22)) == 0 {
        0xEEE
    } else {
        0
    };
    ww(m, V_PALETTE.wrapping_add(0x22), cor);
    flash = flash.wrapping_sub(1);
    wb(m, a.wrapping_add(FLASH), flash);
    if flash == 0 {
        wb(m, a.wrapping_add(OB_COLTYPE), 0x0F);
    }
}

fn renderizar(m: &mut Memoria, a: u32, ani: u32) -> Result<(), ErroAbertura> {
    objetos::animate_sprite(m, a, ani)?;
    let render = (rb(m, a.wrapping_add(OB_RENDER)) & !3) | (rb(m, a.wrapping_add(OB_STATUS)) & 3);
    wb(m, a.wrapping_add(OB_RENDER), render);
    objetos::display_sprite(m, a);
    Ok(())
}

fn rosto(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let pai = rl(m, a.wrapping_add(PARENT));
    if rb(m, pai.wrapping_add(OB_ID)) != 0x73 {
        objetos::delete_object(m, a);
        return Ok(());
    }
    let px = rw(m, pai.wrapping_add(OB_X));
    ww(m, a.wrapping_add(OB_X), px);
    let py = rw(m, pai.wrapping_add(OB_Y));
    ww(m, a.wrapping_add(OB_Y), py);
    let st = rb(m, pai.wrapping_add(OB_STATUS));
    wb(m, a.wrapping_add(OB_STATUS), st);

    let r2 = rb(m, pai.wrapping_add(OB_ROUTINE_2));
    let mut anim = 1;
    if r2 == 2 {
        let sub = rb(m, pai.wrapping_add(OB_SUBTYPE)) & 6;
        if (sub == 2 || sub == 6) && rw(m, pai.wrapping_add(OB_VELY)) == 0 {
            anim = 4; // Laugh
        }
    } else if (4..8).contains(&r2) {
        anim = 0x0A; // Defeated
    } else if r2 >= 8 {
        anim = 6; // Panic
        if rw(m, a.wrapping_add(OB_X)) > BOSS_MZ_END.wrapping_add(0x80) {
            objetos::delete_object(m, a);
            return Ok(());
        }
    }

    if rb(m, pai.wrapping_add(OB_COLTYPE)) == 0 {
        anim = 5; // Hit
    } else if rb(m, V_PLAYER.wrapping_add(OB_ROUTINE)) >= 4 {
        anim = 4; // Laugh if Sonic hurt
    }
    wb(m, a.wrapping_add(OB_ANIM), anim);
    renderizar(m, a, ANI_EGGMAN)
}

fn chama(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let pai = rl(m, a.wrapping_add(PARENT));
    if rb(m, pai.wrapping_add(OB_ID)) != 0x73 {
        objetos::delete_object(m, a);
        return Ok(());
    }
    let px = rw(m, pai.wrapping_add(OB_X));
    ww(m, a.wrapping_add(OB_X), px);
    let py = rw(m, pai.wrapping_add(OB_Y));
    ww(m, a.wrapping_add(OB_Y), py);
    let st = rb(m, pai.wrapping_add(OB_STATUS));
    wb(m, a.wrapping_add(OB_STATUS), st);

    let r2 = rb(m, pai.wrapping_add(OB_ROUTINE_2));
    let mut anim = 7; // Invisivel
    if r2 >= 8 {
        anim = 0x0B; // Thruster
        if rw(m, a.wrapping_add(OB_X)) > BOSS_MZ_END.wrapping_add(0x80) {
            objetos::delete_object(m, a);
            return Ok(());
        }
    } else if rw(m, pai.wrapping_add(OB_VELX)) != 0 {
        anim = 8;
    }
    wb(m, a.wrapping_add(OB_ANIM), anim);
    renderizar(m, a, ANI_EGGMAN)
}

fn tubo(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let pai = rl(m, a.wrapping_add(PARENT));
    if rb(m, pai.wrapping_add(OB_ID)) != 0x73 {
        objetos::delete_object(m, a);
        return Ok(());
    }
    if rb(m, pai.wrapping_add(OB_ROUTINE_2)) >= 8
        && rw(m, a.wrapping_add(OB_X)) > BOSS_MZ_END.wrapping_add(0x80)
    {
        objetos::delete_object(m, a);
        return Ok(());
    }
    let px = rw(m, pai.wrapping_add(OB_X));
    ww(m, a.wrapping_add(OB_X), px);
    let py = rw(m, pai.wrapping_add(OB_Y));
    ww(m, a.wrapping_add(OB_Y), py);
    let st = rb(m, pai.wrapping_add(OB_STATUS));
    wb(m, a.wrapping_add(OB_STATUS), st);
    wl(m, a.wrapping_add(OB_MAP), MAP_BOSS_ITEMS);
    ww(m, a.wrapping_add(OB_GFX), 0x2400); // Tile_Pal2
    wb(m, a.wrapping_add(OB_FRAME), 4);
    let render = (rb(m, a.wrapping_add(OB_RENDER)) & !3) | (rb(m, a.wrapping_add(OB_STATUS)) & 3);
    wb(m, a.wrapping_add(OB_RENDER), render);
    objetos::display_sprite(m, a);
    Ok(())
}

// -----------------------------------------------------------------------------
// Objeto 0x74: BossFire (fogo que Eggman solta e chamas no chao)
// -----------------------------------------------------------------------------

pub fn fogo(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rot = rb(m, a0.wrapping_add(OB_ROUTINE));
    match rot {
        0 => fire_init(m, a0),
        2 => fire_action(m, a0),
        4 => fire_temp(m, a0),
        6 => {
            objetos::delete_object(m, a0);
            Ok(())
        }
        _ => Ok(()),
    }
}

fn fire_init(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    wb(m, a.wrapping_add(OB_HEIGHT), 8);
    wb(m, a.wrapping_add(OB_WIDTH), 8);
    wl(m, a.wrapping_add(OB_MAP), MAP_FIRE);
    ww(m, a.wrapping_add(OB_GFX), 0x345);
    wb(m, a.wrapping_add(OB_RENDER), 4);
    wb(m, a.wrapping_add(OB_PRIORITY), 5);
    let y = rw(m, a.wrapping_add(OB_Y));
    ww(m, a.wrapping_add(BOSS_Y), y);
    wb(m, a.wrapping_add(OB_ACTWID), 8);
    wb(m, a.wrapping_add(OB_ROUTINE), 2);

    if rb(m, a.wrapping_add(OB_SUBTYPE)) == 0 {
        wb(m, a.wrapping_add(OB_COLTYPE), 0x8B); // 16x16 hurt
        wb(m, a.wrapping_add(OB_ROUTINE), 4);
        return fire_temp(m, a);
    }
    wb(m, a.wrapping_add(TIMER), 30);
    wb(m, V_SOUNDQUEUE1, 0xBA); // sfx_Fireball
    fire_action(m, a)
}

fn fire_action(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let r2 = rb(m, a.wrapping_add(OB_ROUTINE_2));
    match r2 {
        0 => fire_drop(m, a),
        2 => fire_make_flame(m, a),
        4 => fire_duplicate(m, a),
        6 => fire_fall_edge(m, a),
        _ => {}
    }
    fisica::speed_to_pos(m, a);
    objetos::animate_sprite(m, a, ANI_FIRE)?;
    if rw(m, a.wrapping_add(OB_Y)) > BOSS_MZ_Y.wrapping_add(0xD8) {
        objetos::delete_object(m, a);
        return Ok(());
    }
    objetos::display_sprite(m, a);
    Ok(())
}

fn fire_drop(m: &mut Memoria, a: u32) {
    let st = rb(m, a.wrapping_add(OB_STATUS));
    wb(m, a.wrapping_add(OB_STATUS), st | 2); // yflip
    let t = rb(m, a.wrapping_add(TIMER));
    let (novo_t, expira) = t.overflowing_sub(1);
    wb(m, a.wrapping_add(TIMER), novo_t);
    if !expira {
        return;
    }
    wb(m, a.wrapping_add(OB_COLTYPE), 0x8B);
    wb(m, a.wrapping_add(OB_SUBTYPE), 0);
    let vy = rw(m, a.wrapping_add(OB_VELY)).wrapping_add(0x18);
    ww(m, a.wrapping_add(OB_VELY), vy);
    wb(m, a.wrapping_add(OB_STATUS), st & !2);

    let cur_x = rw(m, a.wrapping_add(OB_X));
    let (dist, _) = fisica::obj_floor_dist_and_angle(m, a, cur_x);
    if dist < 0 {
        wb(m, a.wrapping_add(OB_ROUTINE_2), 2);
    }
}

fn fire_make_flame(m: &mut Memoria, a: u32) {
    let y = rw(m, a.wrapping_add(OB_Y)).wrapping_sub(2);
    ww(m, a.wrapping_add(OB_Y), y);
    let gfx = rw(m, a.wrapping_add(OB_GFX));
    ww(m, a.wrapping_add(OB_GFX), gfx | 0x8000);
    ww(m, a.wrapping_add(OB_VELX), 0xA0);
    ww(m, a.wrapping_add(OB_VELY), 0);
    let x = rw(m, a.wrapping_add(OB_X));
    ww(m, a.wrapping_add(BOSS_X), x);
    ww(m, a.wrapping_add(BOSS_Y), y);
    wb(m, a.wrapping_add(TIMER), 3);

    if let Some(clone) = next_free(m, a) {
        for off in 0..OBJECT_SIZE {
            let val = rb(m, a.wrapping_add(off));
            wb(m, clone.wrapping_add(off), val);
        }
        ww(m, clone.wrapping_add(OB_VELX), as_u16(-0xA0));
        wb(m, clone.wrapping_add(OB_ROUTINE_2), 4);
    }
    wb(m, a.wrapping_add(OB_ROUTINE_2), 4);
}

fn fire_duplicate(m: &mut Memoria, a: u32) {
    let cur_x = rw(m, a.wrapping_add(OB_X));
    let (dist, _) = fisica::obj_floor_dist_and_angle(m, a, cur_x);
    if dist >= 0 {
        wb(m, a.wrapping_add(OB_ROUTINE_2), 6);
        return;
    }
    if cur_x > BOSS_MZ_X.wrapping_add(0x140) {
        wb(m, a.wrapping_add(OB_ROUTINE), 4);
        return;
    }
    let last_x = rw(m, a.wrapping_add(BOSS_X));
    if (cur_x & 0x10) != (last_x & 0x10) {
        let cur_y = rw(m, a.wrapping_add(OB_Y));
        if let Some(filho) = next_free(m, a) {
            ww(m, filho.wrapping_add(OB_X), cur_x);
            ww(m, filho.wrapping_add(OB_Y), cur_y);
            wb(m, filho.wrapping_add(OB_ID), 0x74);
            wb(m, filho.wrapping_add(OB_SUBTYPE), 0);
            wb(m, filho.wrapping_add(TIMER), 103);
        }
        ww(m, a.wrapping_add(SPREAD_X), cur_x);
    }
    ww(m, a.wrapping_add(BOSS_X), cur_x);
}

fn fire_fall_edge(m: &mut Memoria, a: u32) {
    let st = rb(m, a.wrapping_add(OB_STATUS));
    wb(m, a.wrapping_add(OB_STATUS), st & !2);
    let vy = rw(m, a.wrapping_add(OB_VELY)).wrapping_add(0x24);
    ww(m, a.wrapping_add(OB_VELY), vy);

    let cur_x = rw(m, a.wrapping_add(OB_X));
    let sp_x = rw(m, a.wrapping_add(SPREAD_X));
    let delta = s16(cur_x).wrapping_sub(s16(sp_x)).abs();
    if delta == 18 {
        let gfx = rw(m, a.wrapping_add(OB_GFX));
        ww(m, a.wrapping_add(OB_GFX), gfx & !0x8000);
    }

    let (dist, _) = fisica::obj_floor_dist_and_angle(m, a, cur_x);
    if dist < 0 {
        let t = rb(m, a.wrapping_add(TIMER));
        let (novo_t, zero) = t.overflowing_sub(1);
        wb(m, a.wrapping_add(TIMER), novo_t);
        if zero || novo_t == 0 {
            objetos::delete_object(m, a);
            return;
        }
        ww(m, a.wrapping_add(OB_VELY), 0);
        ww(m, a.wrapping_add(OB_X), sp_x);
        let by = rw(m, a.wrapping_add(BOSS_Y));
        ww(m, a.wrapping_add(OB_Y), by);
        let gfx = rw(m, a.wrapping_add(OB_GFX));
        ww(m, a.wrapping_add(OB_GFX), gfx | 0x8000);
        wb(m, a.wrapping_add(OB_ROUTINE_2), 4);
    }
}

fn fire_temp(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let gfx = rw(m, a.wrapping_add(OB_GFX));
    ww(m, a.wrapping_add(OB_GFX), gfx | 0x8000);
    let t = rb(m, a.wrapping_add(TIMER));
    let (novo_t, expira) = t.overflowing_sub(1);
    wb(m, a.wrapping_add(TIMER), novo_t);
    if expira || novo_t == 0 {
        wb(m, a.wrapping_add(OB_ANIM), 1); // Flame out anim
        let y = rw(m, a.wrapping_add(OB_Y)).wrapping_sub(4);
        ww(m, a.wrapping_add(OB_Y), y);
        wb(m, a.wrapping_add(OB_COLTYPE), 0);
    }
    objetos::animate_sprite(m, a, ANI_FIRE)?;
    objetos::display_sprite(m, a);
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn inicializacao_cria_partes_e_vidas() {
        let mut m = Memoria::nova(vec![0; 0x1000]);
        let a0: u32 = 0xFF_D800;
        ww(&mut m, a0.wrapping_add(OB_X), BOSS_MZ_X);
        ww(&mut m, a0.wrapping_add(OB_Y), BOSS_MZ_Y);
        wb(&mut m, a0.wrapping_add(OB_ID), 0x73);
        wb(&mut m, a0.wrapping_add(OB_ROUTINE), 0);

        let res = executar(&mut m, a0);
        assert!(res.is_ok());
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(rb(&mut m, a0.wrapping_add(0x21)), 8);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_COLTYPE)), 0x0F);

        // Verifica criacao das partes filhas nos slots seguintes
        let mut partes_encontradas = 0;
        let mut slot = a0.wrapping_add(OBJECT_SIZE);
        while slot < a0.wrapping_add(OBJECT_SIZE.wrapping_mul(5)) {
            if rb(&mut m, slot.wrapping_add(OB_ID)) == 0x73 {
                partes_encontradas += 1;
            }
            slot = slot.wrapping_add(OBJECT_SIZE);
        }
        assert!(partes_encontradas >= 3);
    }

    #[test]
    fn fogo_inicializa_parametros() {
        let mut m = Memoria::nova(vec![0; 0x1000]);
        let a0: u32 = 0xFF_D800;
        wb(&mut m, a0.wrapping_add(OB_ID), 0x74);
        wb(&mut m, a0.wrapping_add(OB_SUBTYPE), 0);
        wb(&mut m, a0.wrapping_add(OB_ROUTINE), 0);
        ww(&mut m, a0.wrapping_add(OB_X), BOSS_MZ_X);
        ww(&mut m, a0.wrapping_add(OB_Y), BOSS_MZ_Y);

        wb(&mut m, a0.wrapping_add(TIMER), 60);
        let res = fogo(&mut m, a0);
        assert!(res.is_ok());
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 4);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_COLTYPE)), 0x8B);

        // Subtipo != 0 e a bola que cai
        let a1: u32 = 0xFF_D840;
        wb(&mut m, a1.wrapping_add(OB_ID), 0x74);
        wb(&mut m, a1.wrapping_add(OB_SUBTYPE), 1);
        wb(&mut m, a1.wrapping_add(OB_ROUTINE), 0);
        ww(&mut m, a1.wrapping_add(OB_X), BOSS_MZ_X);
        ww(&mut m, a1.wrapping_add(OB_Y), BOSS_MZ_Y);

        let res = fogo(&mut m, a1);
        assert!(res.is_ok());
        assert_eq!(rb(&mut m, a1.wrapping_add(OB_ROUTINE)), 2);
    }
}
