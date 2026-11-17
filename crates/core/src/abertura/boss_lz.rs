//! Nave do chefe da Labyrinth Zone (objeto 0x77, REV01).
use super::ram::*;
use super::{ErroAbertura, fisica, objetos, rb, rl, rw, wb, wl, ww};
use crate::memoria::Memoria;

const MAP_EGGMAN: u32 = 0x1_84B8;
const ANI_EGGMAN: u32 = 0x1_8468;

const BOSS_LZ_X: u16 = 0x1DE0;
const BOSS_LZ_Y: u16 = 0x00C0;
const BOSS_LZ_END: u16 = 0x1DE0 + 0x250;

const BOSS_X: u32 = 0x30;
const BOSS_Y: u32 = 0x38;
const PARENT: u32 = 0x34;
const TIMER: u32 = 0x3C;
const EARLY_DEFEAT: u32 = 0x3D;
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
        _ => Ok(()),
    }
}

fn preparar(m: &mut Memoria, a0: u32) {
    let x = BOSS_LZ_X.wrapping_add(0x30);
    let y = BOSS_LZ_Y.wrapping_add(0x500);
    ww(m, a0.wrapping_add(OB_X), x);
    ww(m, a0.wrapping_add(OB_Y), y);
    ww(m, a0.wrapping_add(BOSS_X), x);
    ww(m, a0.wrapping_add(BOSS_Y), y);
    wb(m, a0.wrapping_add(OB_COLTYPE), 0x0F);
    wb(m, a0.wrapping_add(0x21), 8); // 8 vidas

    let mut slot = a0;
    for (rot, anim) in [(2, 0), (4, 1), (6, 7)] {
        wb(m, slot.wrapping_add(OB_ID), 0x77);
        wb(m, slot.wrapping_add(OB_ROUTINE), rot);
        wb(m, slot.wrapping_add(OB_ROUTINE_2), 0);
        wb(m, slot.wrapping_add(OB_ANIM), anim);
        wb(m, slot.wrapping_add(OB_PRIORITY), 4);
        wl(m, slot.wrapping_add(OB_MAP), MAP_EGGMAN);
        ww(m, slot.wrapping_add(OB_GFX), 0x400);
        wb(m, slot.wrapping_add(OB_RENDER), 4);
        wb(m, slot.wrapping_add(OB_ACTWID), 32);
        ww(m, slot.wrapping_add(OB_X), x);
        ww(m, slot.wrapping_add(OB_Y), y);
        wl(m, slot.wrapping_add(PARENT), a0);

        if rot == 6 {
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
        2 => ship_move1(m, a),
        4 => ship_move2(m, a),
        6 => ship_move3(m, a),
        8 => ship_at_top(m, a),
        0x0A => ship_wait(m, a),
        0x0C => escape1(m, a),
        0x0E => escape2(m, a),
        _ => {}
    }
    ship_update(m, a);
    renderizar(m, a, ANI_EGGMAN)
}

fn ship_start(m: &mut Memoria, a: u32) {
    let sonic_x = rw(m, V_PLAYER.wrapping_add(OB_X));
    if sonic_x >= BOSS_LZ_X.wrapping_sub(0x40) {
        ww(m, a.wrapping_add(OB_VELY), as_u16(-0x180));
        ww(m, a.wrapping_add(OB_VELX), 0x60);
        wb(m, a.wrapping_add(OB_ROUTINE_2), 2);
    }
    move_boss(m, a);
}

fn move_boss(m: &mut Memoria, a: u32) {
    boss_move(m, a);
    let bx = rw(m, a.wrapping_add(BOSS_X));
    ww(m, a.wrapping_add(OB_X), bx);
    let by = rw(m, a.wrapping_add(BOSS_Y));
    ww(m, a.wrapping_add(OB_Y), by);
}

fn ship_move1(m: &mut Memoria, a: u32) {
    let mut condicoes = 0u8;
    let alvo_x = BOSS_LZ_X.wrapping_add(0x68);
    let cur_x = rw(m, a.wrapping_add(BOSS_X));
    if cur_x >= alvo_x {
        ww(m, a.wrapping_add(BOSS_X), alvo_x);
        ww(m, a.wrapping_add(OB_VELX), 0);
        condicoes = condicoes.wrapping_add(1);
    }

    let alvo_y = BOSS_LZ_Y.wrapping_add(0x440);
    let cur_y = rw(m, a.wrapping_add(BOSS_Y));
    if cur_y <= alvo_y {
        ww(m, a.wrapping_add(BOSS_Y), alvo_y);
        ww(m, a.wrapping_add(OB_VELY), 0);
        condicoes = condicoes.wrapping_add(1);
    }

    if condicoes == 2 {
        ww(m, a.wrapping_add(OB_VELX), 0x140);
        ww(m, a.wrapping_add(OB_VELY), as_u16(-0x200));
        wb(m, a.wrapping_add(OB_ROUTINE_2), 4);
    }
    move_boss(m, a);
}

fn ship_move2(m: &mut Memoria, a: u32) {
    let mut condicoes = 0u8;
    let alvo_x = BOSS_LZ_X.wrapping_add(0x90);
    let cur_x = rw(m, a.wrapping_add(BOSS_X));
    if cur_x >= alvo_x {
        ww(m, a.wrapping_add(BOSS_X), alvo_x);
        ww(m, a.wrapping_add(OB_VELX), 0);
        condicoes = condicoes.wrapping_add(1);
    }

    let alvo_y = BOSS_LZ_Y.wrapping_add(0x400);
    let cur_y = rw(m, a.wrapping_add(BOSS_Y));
    if cur_y <= alvo_y {
        ww(m, a.wrapping_add(BOSS_Y), alvo_y);
        ww(m, a.wrapping_add(OB_VELY), 0);
        condicoes = condicoes.wrapping_add(1);
    }

    if condicoes == 2 {
        ww(m, a.wrapping_add(OB_VELY), as_u16(-0x180));
        ww(m, a.wrapping_add(OB_VELX), 0);
        wb(m, a.wrapping_add(OB_ROUTINE_2), 6);
        wb(m, a.wrapping_add(SINE_COUNTER), 0);
    }
    move_boss(m, a);
}

fn ship_move3(m: &mut Memoria, a: u32) {
    let cur_y = rw(m, a.wrapping_add(BOSS_Y));
    let topo = BOSS_LZ_Y.wrapping_add(0x40);
    if cur_y <= topo {
        ww(m, a.wrapping_add(BOSS_Y), topo);
        let mut vx = 0x140i16;
        let mut vy = -0x80i16;
        if rb(m, a.wrapping_add(EARLY_DEFEAT)) != 0 {
            vx = vx.wrapping_mul(2);
            vy = vy.wrapping_mul(2);
        }
        ww(m, a.wrapping_add(OB_VELX), as_u16(vx));
        ww(m, a.wrapping_add(OB_VELY), as_u16(vy));
        wb(m, a.wrapping_add(OB_ROUTINE_2), 8);
        move_boss(m, a);
        return;
    }

    let sc = rb(m, a.wrapping_add(SINE_COUNTER)).wrapping_add(2);
    wb(m, a.wrapping_add(SINE_COUNTER), sc);
    let (seno, cosseno) = fisica::calc_sine(m, sc);

    let st = rb(m, a.wrapping_add(OB_STATUS));
    if cosseno >= 0 {
        wb(m, a.wrapping_add(OB_STATUS), st | 1); // Face right
    } else {
        wb(m, a.wrapping_add(OB_STATUS), st & !1); // Face left
    }

    let osc = s16(rw(m, a.wrapping_add(BOSS_X))).wrapping_add(seno >> 4);
    ww(m, a.wrapping_add(OB_X), as_u16(osc));

    let mut vy = s16(rw(m, a.wrapping_add(OB_VELY)));
    let sonic_y = s16(rw(m, V_PLAYER.wrapping_add(OB_Y)));
    let eggman_y = s16(rw(m, a.wrapping_add(OB_Y)));
    let mut dist = sonic_y.wrapping_sub(eggman_y);

    if dist > 0 {
        dist = dist.wrapping_sub(72);
        if dist > 0 {
            vy = vy >> 1;
            dist = dist.wrapping_sub(40);
            if dist > 0 {
                vy = vy >> 1;
                dist = dist.wrapping_sub(40);
                if dist > 0 {
                    vy = 0;
                }
            }
        }
    }

    let mut passo = i32::from(vy).wrapping_shl(8);
    if rb(m, a.wrapping_add(EARLY_DEFEAT)) != 0 {
        passo = passo.wrapping_mul(2);
    }
    let novo_by = rl(m, a.wrapping_add(BOSS_Y)).wrapping_add(signed_long(passo));
    wl(m, a.wrapping_add(BOSS_Y), novo_by);
    let by = rw(m, a.wrapping_add(BOSS_Y));
    ww(m, a.wrapping_add(OB_Y), by);
}

fn ship_at_top(m: &mut Memoria, a: u32) {
    let mut condicoes = 0u8;
    let alvo_x = BOSS_LZ_X.wrapping_add(0x16C);
    let cur_x = rw(m, a.wrapping_add(BOSS_X));
    if cur_x >= alvo_x {
        ww(m, a.wrapping_add(BOSS_X), alvo_x);
        ww(m, a.wrapping_add(OB_VELX), 0);
        condicoes = condicoes.wrapping_add(1);
    }

    let alvo_y = BOSS_LZ_Y;
    let cur_y = rw(m, a.wrapping_add(BOSS_Y));
    if cur_y <= alvo_y {
        ww(m, a.wrapping_add(BOSS_Y), alvo_y);
        ww(m, a.wrapping_add(OB_VELY), 0);
        condicoes = condicoes.wrapping_add(1);
    }

    if condicoes == 2 {
        wb(m, a.wrapping_add(OB_ROUTINE_2), 0x0A);
        let st = rb(m, a.wrapping_add(OB_STATUS));
        wb(m, a.wrapping_add(OB_STATUS), st & !1); // Face left
    }
    move_boss(m, a);
}

fn ship_wait(m: &mut Memoria, a: u32) {
    if rb(m, a.wrapping_add(EARLY_DEFEAT)) != 0 {
        start_escape(m, a);
    } else {
        let sx = rw(m, V_PLAYER.wrapping_add(OB_X));
        let sy = rw(m, V_PLAYER.wrapping_add(OB_Y));
        if sx >= BOSS_LZ_X.wrapping_add(0xE8) && sy <= BOSS_LZ_Y.wrapping_add(0x30) {
            wb(m, a.wrapping_add(TIMER), 50);
            start_escape(m, a);
        }
    }
    move_boss(m, a);
}

fn start_escape(m: &mut Memoria, a: u32) {
    wb(m, V_SOUNDQUEUE0, 0x81); // bgm_LZ
    wb(m, F_LOCKSCREEN, 0);
    let st = rb(m, a.wrapping_add(OB_STATUS));
    wb(m, a.wrapping_add(OB_STATUS), st | 1); // Face right
    wb(m, a.wrapping_add(OB_ROUTINE_2), 0x0C);
}

fn escape1(m: &mut Memoria, a: u32) {
    if rb(m, a.wrapping_add(EARLY_DEFEAT)) != 0 {
        escape_now(m, a);
    } else {
        let t = rb(m, a.wrapping_add(TIMER));
        let (novo_t, zero) = t.overflowing_sub(1);
        wb(m, a.wrapping_add(TIMER), novo_t);
        if zero || novo_t == 0 {
            escape_now(m, a);
        }
    }
    move_boss(m, a);
}

fn escape_now(m: &mut Memoria, a: u32) {
    wb(m, a.wrapping_add(TIMER), 0);
    ww(m, a.wrapping_add(OB_VELX), 0x400);
    ww(m, a.wrapping_add(OB_VELY), 0xFFC0); // -0x40
    wb(m, a.wrapping_add(EARLY_DEFEAT), 0);
    wb(m, a.wrapping_add(OB_ROUTINE_2), 0x0E);
}

fn escape2(m: &mut Memoria, a: u32) {
    let borda = rw(m, V_LIMITRIGHT2);
    if borda < BOSS_LZ_END {
        ww(m, V_LIMITRIGHT2, borda.wrapping_add(2));
    } else if rw(m, a.wrapping_add(OB_X)) > BOSS_LZ_END.wrapping_add(0x80) {
        ww(m, V_PALETTE.wrapping_add(0x22), 0);
        objetos::delete_object(m, a);
        return;
    }
    move_boss(m, a);
}

fn ship_update(m: &mut Memoria, a: u32) {
    if rb(m, a.wrapping_add(EARLY_DEFEAT)) != 0 {
        super::boss_ghz::explosao(m, a);
        return;
    }
    if rb(m, a.wrapping_add(OB_STATUS)) & 0x80 != 0 {
        fisica::add_points(m, 100);
        wb(m, a.wrapping_add(EARLY_DEFEAT), 0xFF);
        return;
    }
    if rb(m, a.wrapping_add(OB_COLTYPE)) != 0 {
        return;
    }
    let mut flash = rb(m, a.wrapping_add(FLASH));
    if flash == 0 {
        flash = 0x20;
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
    if rb(m, pai.wrapping_add(OB_ID)) != 0x77 {
        objetos::delete_object(m, a);
        return Ok(());
    }
    let px = rw(m, pai.wrapping_add(OB_X));
    ww(m, a.wrapping_add(OB_X), px);
    let py = rw(m, pai.wrapping_add(OB_Y));
    ww(m, a.wrapping_add(OB_Y), py);
    let st = rb(m, pai.wrapping_add(OB_STATUS));
    wb(m, a.wrapping_add(OB_STATUS), st);

    let mut anim = 1;
    if rb(m, pai.wrapping_add(EARLY_DEFEAT)) != 0 {
        anim = 0x0A; // Defeated
    } else if rb(m, pai.wrapping_add(OB_COLTYPE)) == 0 {
        anim = 5; // Hit
    } else if rb(m, pai.wrapping_add(OB_ROUTINE_2)) >= 0x0E {
        anim = 6; // Panic
        if rw(m, a.wrapping_add(OB_X)) > BOSS_LZ_END.wrapping_add(0x80) {
            objetos::delete_object(m, a);
            return Ok(());
        }
    }
    wb(m, a.wrapping_add(OB_ANIM), anim);
    renderizar(m, a, ANI_EGGMAN)
}

fn chama(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let pai = rl(m, a.wrapping_add(PARENT));
    if rb(m, pai.wrapping_add(OB_ID)) != 0x77 {
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
    let mut anim = 7;
    if r2 >= 0x0E {
        anim = 0x0B; // Thruster
        if rw(m, a.wrapping_add(OB_X)) > BOSS_LZ_END.wrapping_add(0x80) {
            objetos::delete_object(m, a);
            return Ok(());
        }
    } else if rw(m, pai.wrapping_add(OB_VELX)) != 0 || rw(m, pai.wrapping_add(OB_VELY)) != 0 {
        anim = 8;
    }
    wb(m, a.wrapping_add(OB_ANIM), anim);
    renderizar(m, a, ANI_EGGMAN)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn inicializacao_lz_cria_subpartes() {
        let mut m = Memoria::nova(vec![0; 0x1000]);
        let a0: u32 = 0xFF_D800;
        wb(&mut m, a0.wrapping_add(OB_ID), 0x77);
        wb(&mut m, a0.wrapping_add(OB_ROUTINE), 0);

        let res = executar(&mut m, a0);
        assert!(res.is_ok());
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(rb(&mut m, a0.wrapping_add(0x21)), 8);
        assert_eq!(rw(&mut m, a0.wrapping_add(OB_X)), BOSS_LZ_X.wrapping_add(0x30));
        assert_eq!(rw(&mut m, a0.wrapping_add(OB_Y)), BOSS_LZ_Y.wrapping_add(0x500));

        let mut partes = 0;
        let mut slot = a0.wrapping_add(OBJECT_SIZE);
        while slot < a0.wrapping_add(OBJECT_SIZE.wrapping_mul(4)) {
            if rb(&mut m, slot.wrapping_add(OB_ID)) == 0x77 {
                partes += 1;
            }
            slot = slot.wrapping_add(OBJECT_SIZE);
        }
        assert!(partes >= 2);
    }
}
