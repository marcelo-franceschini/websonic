//! Sequencia de final do jogo, creditos e telas de Try Again (objetos 0x87, 0x88, 0x89, 0x8B, 0x8C).
use super::ram::*;
use super::{ErroAbertura, fisica, objetos, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;

const MAP_ESON: u32 = 0x00568E;
const ANI_ESON: u32 = 0x005522;
const MAP_ECHA: u32 = 0x005788;
const MAP_ESTH: u32 = 0x0057C0;
const MAP_EEGG: u32 = 0x005BCA;
const ANI_EEGG: u32 = 0x005AB0;

const ARTTILE_ENDING_SONIC: u16 = 0x03E1;
const ARTTILE_ENDING_EMERALDS: u16 = 0x03C5;
const ARTTILE_ENDING_STH: u16 = 0x05C5;
const ARTTILE_TRY_AGAIN_EGGMAN: u16 = 0x03E1;
const ARTTILE_TRY_AGAIN_EMERALDS: u16 = 0x03C5;
const MAP_CRED: u32 = 0x017A8E;
const ARTTILE_CREDITS_FONT: u16 = 0x05A0;
const ARTTILE_SONIC_TEAM_FONT: u16 = 0x00A6;

/// Sub-rotina que controla o Sonic na sequencia final (`End_MoveSonic`).
pub fn end_move_sonic(m: &mut Memoria) {
    let r = rb(m, V_SONICEND);
    match r {
        0 => {
            let px = rw(m, V_PLAYER.wrapping_add(OB_X));
            if px < 144 {
                wb(m, V_SONICEND, 2);
                wb(m, F_LOCKCTRL, 1);
                ww(m, V_JPADHOLD2, 0x0800); // segura direita para derrapar
            }
        }
        2 => {
            let px = rw(m, V_PLAYER.wrapping_add(OB_X));
            if px >= 160 {
                wb(m, V_SONICEND, 4);
                wb(m, F_LOCKCTRL, 0);
                ww(m, V_JPADHOLD2, 0);
                ww(m, V_PLAYER.wrapping_add(OB_INERTIA), 0);
                wb(m, F_PLAYERCTRL, 0x81);
                wb(m, V_PLAYER.wrapping_add(OB_FRAME), 0x22); // espera/parado
                ww(m, V_PLAYER.wrapping_add(OB_ANIM), 0x0505);
                wb(m, V_PLAYER.wrapping_add(OB_TIMEFRAME), 3);
            }
        }
        4 => {
            wb(m, V_SONICEND, 6);
            ww(m, V_PLAYER.wrapping_add(OB_X), 160);
            wb(m, V_PLAYER.wrapping_add(OB_ID), 0x87); // substitui pelo fake Sonic do final
            ww(m, V_PLAYER.wrapping_add(OB_ROUTINE), 0);
        }
        _ => {}
    }
}

/// Objeto 0x87: Sonic na sequencia de final (`EndSonic`).
pub fn end_sonic(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let r2 = rb(m, a0.wrapping_add(OB_2NDROUT));
    match r2 {
        0 => eson_main(m, a0),
        2 => eson_make_emeralds(m, a0),
        4 | 0x0A | 0x0E | 0x12 => eson_animate(m, a0)?,
        6 => eson_look_up(m, a0),
        8 => eson_delete_emeralds(m, a0),
        0x0C => eson_make_logo(m, a0),
        0x10 => eson_bad_ending(m, a0),
        _ => {}
    }
    objetos::display_sprite(m, a0);
    Ok(())
}

fn eson_main(m: &mut Memoria, a0: u32) {
    if rb(m, V_EMERALDS) >= 6 {
        wb(m, a0.wrapping_add(OB_2NDROUT), 2);
        wl(m, a0.wrapping_add(OB_MAP), MAP_ESON);
        ww(m, a0.wrapping_add(OB_GFX), ARTTILE_ENDING_SONIC);
        wb(m, a0.wrapping_add(OB_RENDER), 4);
        wb(m, a0.wrapping_add(OB_STATUS), 0);
        wb(m, a0.wrapping_add(OB_PRIORITY), 2);
        wb(m, a0.wrapping_add(OB_FRAME), 0);
        ww(m, a0.wrapping_add(0x30), 80);
    } else {
        wb(m, a0.wrapping_add(OB_2NDROUT), 0x10);
        ww(m, a0.wrapping_add(0x30), 216);
    }
}

fn eson_make_emeralds(m: &mut Memoria, a0: u32) {
    let t = rw(m, a0.wrapping_add(0x30)).wrapping_sub(1);
    ww(m, a0.wrapping_add(0x30), t);
    if t == 0 {
        wb(m, a0.wrapping_add(OB_2NDROUT), 4);
        ww(m, a0.wrapping_add(OB_ANIM), 1);
        wb(m, V_ENDEMERALDS.wrapping_add(OB_ID), 0x88);
    }
}

fn eson_look_up(m: &mut Memoria, a0: u32) {
    let radius = rw(m, V_ENDEMERALDS.wrapping_add(0x3C));
    if radius >= 0x2000 {
        ww(m, F_RESTART, 1);
        ww(m, a0.wrapping_add(0x30), 90);
        wb(m, a0.wrapping_add(OB_2NDROUT), 8);
    }
}

fn eson_delete_emeralds(m: &mut Memoria, a0: u32) {
    let t = rw(m, a0.wrapping_add(0x30)).wrapping_sub(1);
    ww(m, a0.wrapping_add(0x30), t);
    if t == 0 {
        // Limpa os slots de esmeraldas
        for i in 0..16u32 {
            let slot = V_ENDEMERALDS.wrapping_add(i.wrapping_mul(OBJECT_SIZE));
            wb(m, slot.wrapping_add(OB_ID), 0);
        }
        ww(m, F_RESTART, 1);
        wb(m, a0.wrapping_add(OB_2NDROUT), 0x0A);
        wb(m, a0.wrapping_add(OB_ANIM), 1);
        ww(m, a0.wrapping_add(0x30), 60);
    }
}

fn eson_make_logo(m: &mut Memoria, a0: u32) {
    let t = rw(m, a0.wrapping_add(0x30)).wrapping_sub(1);
    ww(m, a0.wrapping_add(0x30), t);
    if t == 0 {
        wb(m, a0.wrapping_add(OB_2NDROUT), 0x0E);
        ww(m, a0.wrapping_add(0x30), 180);
        wb(m, a0.wrapping_add(OB_ANIM), 2);
        wb(m, V_ENDLOGO.wrapping_add(OB_ID), 0x89);
    }
}

fn eson_bad_ending(m: &mut Memoria, a0: u32) {
    let t = rw(m, a0.wrapping_add(0x30)).wrapping_sub(1);
    ww(m, a0.wrapping_add(0x30), t);
    if t == 0 {
        wb(m, a0.wrapping_add(OB_2NDROUT), 0x12);
        wl(m, a0.wrapping_add(OB_MAP), MAP_ESON);
        ww(m, a0.wrapping_add(OB_GFX), ARTTILE_ENDING_SONIC);
        wb(m, a0.wrapping_add(OB_RENDER), 4);
        wb(m, a0.wrapping_add(OB_STATUS), 0);
        wb(m, a0.wrapping_add(OB_PRIORITY), 2);
        wb(m, a0.wrapping_add(OB_FRAME), 5);
        wb(m, a0.wrapping_add(OB_ANIM), 2);
        wb(m, V_ENDLOGO.wrapping_add(OB_ID), 0x89);
    }
}

fn eson_animate(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    objetos::animate_sprite(m, a0, ANI_ESON)
}

/// Objeto 0x88: Esmeraldas do Caos girando no bom final (`EndChaos`).
pub fn end_chaos(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        if rb(m, V_PLAYER.wrapping_add(OB_FRAME)) != 2 {
            return Ok(());
        }
        let son_x = rw(m, V_PLAYER.wrapping_add(OB_X));
        let son_y = rw(m, V_PLAYER.wrapping_add(OB_Y));

        let mut angle = 0u8;
        for i in 0..6u32 {
            let slot = a0.wrapping_add(i.wrapping_mul(OBJECT_SIZE));
            wb(m, slot.wrapping_add(OB_ID), 0x88);
            wb(m, slot.wrapping_add(OB_ROUTINE), 2);
            wl(m, slot.wrapping_add(OB_MAP), MAP_ECHA);
            ww(m, slot.wrapping_add(OB_GFX), ARTTILE_ENDING_EMERALDS);
            wb(m, slot.wrapping_add(OB_RENDER), 4);
            wb(m, slot.wrapping_add(OB_PRIORITY), 1);
            ww(m, slot.wrapping_add(0x38), son_x); // origX
            ww(m, slot.wrapping_add(0x3A), son_y); // origY
            wb(m, slot.wrapping_add(OB_FRAME), (i + 1) as u8);
            wb(m, slot.wrapping_add(OB_ANGLE), angle);
            angle = angle.wrapping_add((256 / 6) as u8);
        }
    }

    let angle_inc = rw(m, a0.wrapping_add(0x3E));
    let cur_angle = rb(m, a0.wrapping_add(OB_ANGLE)).wrapping_add((angle_inc >> 8) as u8);
    wb(m, a0.wrapping_add(OB_ANGLE), cur_angle);

    let (s, c) = fisica::calc_sine(m, cur_angle);
    let radius = rw(m, a0.wrapping_add(0x3C)) >> 8;
    let orig_x = rw(m, a0.wrapping_add(0x38));
    let orig_y = rw(m, a0.wrapping_add(0x3A));

    let off_x = (i32::from(c).wrapping_mul(i32::from(radius)) >> 8) as i16;
    let off_y = (i32::from(s).wrapping_mul(i32::from(radius)) >> 8) as i16;
    ww(m, a0.wrapping_add(OB_X), orig_x.wrapping_add(off_x as u16));
    ww(m, a0.wrapping_add(OB_Y), orig_y.wrapping_add(off_y as u16));

    let rad = rw(m, a0.wrapping_add(0x3C));
    if rad < 0x2000 {
        ww(m, a0.wrapping_add(0x3C), rad.wrapping_add(0x20));
    }
    if angle_inc < 0x2000 {
        ww(m, a0.wrapping_add(0x3E), angle_inc.wrapping_add(0x20));
    }
    if orig_y > 0x140 {
        ww(m, a0.wrapping_add(0x3A), orig_y.wrapping_sub(1));
    }

    objetos::display_sprite(m, a0);
    Ok(())
}

/// Objeto 0x89: Texto "SONIC THE HEDGEHOG" deslizando no final (`EndSTH`).
pub fn end_sth(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    match rotina {
        0 => {
            wb(m, a0.wrapping_add(OB_ROUTINE), 2);
            ww(m, a0.wrapping_add(OB_X), 0xFFE0);
            ww(m, a0.wrapping_add(OB_SCREENY), 0x00D8);
            wl(m, a0.wrapping_add(OB_MAP), MAP_ESTH);
            ww(m, a0.wrapping_add(OB_GFX), ARTTILE_ENDING_STH);
            wb(m, a0.wrapping_add(OB_RENDER), 0);
            wb(m, a0.wrapping_add(OB_PRIORITY), 0);
        }
        2 => {
            let cur_x = rw(m, a0.wrapping_add(OB_X));
            if cur_x >= 0x00C0 {
                wb(m, a0.wrapping_add(OB_ROUTINE), 4);
                ww(m, a0.wrapping_add(0x30), 300); // 5 segundos no REV01
            } else {
                ww(m, a0.wrapping_add(OB_X), cur_x.wrapping_add(0x10));
            }
        }
        _ => {
            let t = s16(rw(m, a0.wrapping_add(0x30))).wrapping_sub(1);
            ww(m, a0.wrapping_add(0x30), as_u16(t));
            if t < 0 {
                // Avanca para tela de creditos
                wb(m, V_GAMEMODE, GM_CREDITS_ID);
            }
        }
    }
    objetos::display_sprite(m, a0);
    Ok(())
}

/// Objeto 0x8B: Eggman nas telas "TRY AGAIN" e "END" (`EndEggman`).
pub fn end_eggman(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    match rotina {
        0 => {
            wb(m, a0.wrapping_add(OB_ROUTINE), 2);
            ww(m, a0.wrapping_add(OB_X), 0x120);
            ww(m, a0.wrapping_add(OB_SCREENY), 0x00F4);
            wl(m, a0.wrapping_add(OB_MAP), MAP_EEGG);
            ww(m, a0.wrapping_add(OB_GFX), ARTTILE_TRY_AGAIN_EGGMAN);
            wb(m, a0.wrapping_add(OB_RENDER), 0);
            wb(m, a0.wrapping_add(OB_PRIORITY), 2);
            wb(m, a0.wrapping_add(OB_ANIM), 2); // bom final (birra)

            if rb(m, V_EMERALDS) < 6 {
                // Mau final (malabarismo)
                wb(m, V_TRYAGAIN.wrapping_add(OB_ID), 0x8A); // CreditsText
                ww(m, V_CREDITSNUM, 9); // "TRY AGAIN"
                wb(m, V_EGGMANCHAOS.wrapping_add(OB_ID), 0x8C); // TryChaos
                wb(m, a0.wrapping_add(OB_ANIM), 0);
            }
        }
        2 => {
            let _ = objetos::animate_sprite(m, a0, ANI_EEGG);
        }
        4 => {
            wb(m, a0.wrapping_add(OB_ROUTINE), 6);
            ww(m, a0.wrapping_add(0x30), 112);
        }
        _ => {
            let t = s16(rw(m, a0.wrapping_add(0x30))).wrapping_sub(1);
            ww(m, a0.wrapping_add(0x30), as_u16(t));
            if t < 0 {
                let anim = rb(m, a0.wrapping_add(OB_ANIM)) ^ 1;
                wb(m, a0.wrapping_add(OB_ANIM), anim);
                wb(m, a0.wrapping_add(OB_ROUTINE), 2);
            }
        }
    }
    objetos::display_sprite(m, a0);
    Ok(())
}

/// Objeto 0x8C: Esmeraldas que faltam no "TRY AGAIN" (`TryChaos`).
pub fn try_chaos(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        wb(m, a0.wrapping_add(OB_ROUTINE), 2);
        wl(m, a0.wrapping_add(OB_MAP), MAP_ECHA);
        ww(m, a0.wrapping_add(OB_GFX), ARTTILE_TRY_AGAIN_EMERALDS);
        wb(m, a0.wrapping_add(OB_RENDER), 0);
        wb(m, a0.wrapping_add(OB_PRIORITY), 1);
        ww(m, a0.wrapping_add(OB_X), 0x104);
        ww(m, a0.wrapping_add(0x38), 0x120); // origX
        ww(m, a0.wrapping_add(OB_SCREENY), 0x00EC);
        ww(m, a0.wrapping_add(0x3A), 0x00EC); // origY
        wb(m, a0.wrapping_add(0x3C), 0x1C); // raio
        wb(m, a0.wrapping_add(OB_FRAME), 1);
    }

    let ang = rb(m, a0.wrapping_add(OB_ANGLE)).wrapping_add(2);
    wb(m, a0.wrapping_add(OB_ANGLE), ang);

    let (s, c) = fisica::calc_sine(m, ang);
    let orig_x = rw(m, a0.wrapping_add(0x38));
    let orig_y = rw(m, a0.wrapping_add(0x3A));
    let off_x = (i32::from(c).wrapping_mul(0x1C) >> 8) as i16;
    let off_y = (i32::from(s).wrapping_mul(0x1C) >> 8) as i16;
    ww(m, a0.wrapping_add(OB_X), orig_x.wrapping_add(off_x as u16));
    ww(
        m,
        a0.wrapping_add(OB_SCREENY),
        orig_y.wrapping_add(off_y as u16),
    );

    objetos::display_sprite(m, a0);
    Ok(())
}

fn s16(w: u16) -> i16 {
    i16::from_be_bytes(w.to_be_bytes())
}

fn as_u16(i: i16) -> u16 {
    u16::from_be_bytes(i.to_be_bytes())
}

/// Objeto 0x8A: Textos dos creditos e "SONIC TEAM PRESENTS" (`CreditsText`).
pub fn credits_text(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        wb(m, a0.wrapping_add(OB_ROUTINE), 2);
        ww(m, a0.wrapping_add(OB_X), 0x120);
        ww(m, a0.wrapping_add(OB_SCREENY), 0x00F0);
        wl(m, a0.wrapping_add(OB_MAP), MAP_CRED);
        ww(m, a0.wrapping_add(OB_GFX), ARTTILE_CREDITS_FONT);
        let d0 = rw(m, V_CREDITSNUM);
        wb(m, a0.wrapping_add(OB_FRAME), d0 as u8);
        wb(m, a0.wrapping_add(OB_RENDER), 0);
        wb(m, a0.wrapping_add(OB_PRIORITY), 0);
        if rb(m, V_GAMEMODE) == GM_TITLE_ID {
            ww(m, a0.wrapping_add(OB_GFX), ARTTILE_SONIC_TEAM_FONT);
            wb(m, a0.wrapping_add(OB_FRAME), 0x0A);
        }
    }
    objetos::display_sprite(m, a0);
    Ok(())
}

/// Inicializa os dados da sequencia de final (`GM_Ending`).
pub fn preparo(m: &mut Memoria) -> Result<(), ErroAbertura> {
    super::titulo::zerar(m, V_OBJSPACE, V_OBJSPACE_END);
    super::titulo::zerar(m, V_MISC_VARIABLES, V_MISC_VARIABLES_END);
    super::titulo::zerar(m, V_LEVELVARIABLES, V_LEVELVARIABLES_END);
    super::titulo::zerar(
        m,
        V_TIMINGANDSCREENVARIABLES,
        V_TIMINGANDSCREENVARIABLES_END,
    );

    super::sega::clear_screen(m)?;
    for w in [0x8B03u16, 0x8230, 0x8407, 0x857C, 0x9001, 0x8004, 0x8720] {
        ww(m, VDP_CONTROLE, w);
    }
    ww(m, V_HBLANK_HREG, 0x8ADF);
    let h = rw(m, V_HBLANK_HREG);
    ww(m, VDP_CONTROLE, h);
    ww(m, V_AIR, 30);

    let tem_todas = rb(m, V_EMERALDS) >= 6;
    let zone_act = if tem_todas { ID_ENDZ_GOOD } else { ID_ENDZ_BAD };
    ww(m, V_ZONE_ACT, zone_act);

    super::plc::quick_plc(m, PLCID_ENDING)?;
    super::hud::hud_base(m);
    super::cenario::level_size_load(m)?;
    super::cenario::deform_layers(m)?;
    let flags = rb(m, V_FG_SCROLL_FLAGS) | 4;
    wb(m, V_FG_SCROLL_FLAGS, flags);
    super::cenario::level_layout_load(m)?;

    let chunk_dest = V_256X256.wrapping_add(0x4A * 0x200);
    crate::rotinas::kos_dec(m, 0x061822, chunk_dest).map_err(ErroAbertura::Rotina)?;

    super::sega::pal_load_para(m, PALID_SONIC, 0x80);
    wb(m, V_SOUNDQUEUE0, BGM_ENDING);

    wb(m, V_PLAYER.wrapping_add(OB_ID), ID_SONIC);
    wb(m, V_PLAYER.wrapping_add(OB_STATUS), 1);
    wb(m, F_LOCKCTRL, 1);
    ww(m, V_JPADHOLD2, 0x0400);
    ww(m, V_PLAYER.wrapping_add(OB_INERTIA), 0xF800);
    wb(m, V_HUD.wrapping_add(OB_ID), ID_HUD);

    super::objpos::obj_pos_load(m)?;
    objetos::execute_objects(m)?;
    objetos::build_sprites(m)?;

    ww(m, V_RINGS, 0);
    wl(m, V_TIME, 0);
    wb(m, V_LIFECOUNT, 0);
    wb(m, V_SHIELD, 0);
    wb(m, V_INVINC, 0);
    wb(m, V_SHOES, 0);
    ww(m, V_DEBUGUSE, 0);
    ww(m, F_RESTART, 0);
    ww(m, V_FRAMECOUNT, 0);
    super::fase::oscillate_num_init(m);
    wb(m, F_SCORECOUNT, 1);
    wb(m, F_RINGCOUNT, 1);
    wb(m, F_TIMECOUNT, 0);

    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DepoisEnding {
    Continua,
    Clarear,
    Creditos,
}

pub fn passo(m: &mut Memoria) -> Result<DepoisEnding, ErroAbertura> {
    super::fase::pause_game(m);
    let fc = rw(m, V_FRAMECOUNT).wrapping_add(1);
    ww(m, V_FRAMECOUNT, fc);

    end_move_sonic(m);
    objetos::execute_objects(m)?;
    super::cenario::deform_layers(m)?;
    objetos::build_sprites(m)?;
    super::objpos::obj_pos_load(m)?;
    super::paleta::palette_cycle(m)?;
    super::fase::oscillate_num_do(m);
    super::fase::synchro_animate(m);

    if rb(m, V_GAMEMODE) != GM_ENDING_ID {
        return Ok(DepoisEnding::Creditos);
    }
    if rw(m, F_RESTART) != 0 {
        return Ok(DepoisEnding::Clarear);
    }
    Ok(DepoisEnding::Continua)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DepoisClarear {
    Continua,
    Terminou,
}

pub fn passo_clarear(m: &mut Memoria) -> Result<DepoisClarear, ErroAbertura> {
    super::fase::pause_game(m);
    let fc = rw(m, V_FRAMECOUNT).wrapping_add(1);
    ww(m, V_FRAMECOUNT, fc);

    end_move_sonic(m);
    objetos::execute_objects(m)?;
    super::cenario::deform_layers(m)?;
    objetos::build_sprites(m)?;
    super::objpos::obj_pos_load(m)?;
    super::fase::oscillate_num_do(m);
    super::fase::synchro_animate(m);

    let t = rw(m, V_PALCHGSPEED).wrapping_sub(1);
    ww(m, V_PALCHGSPEED, t);
    if i16::from_be_bytes(t.to_be_bytes()) < 0 {
        ww(m, V_PALCHGSPEED, 2);
        super::special_result::clarear(m);
    }

    if rw(m, F_RESTART) != 0 {
        ww(m, F_RESTART, 0);
        ww(m, V_LVLLAYOUT_FG, 0x2E2F);
        super::sega::pal_load_para(m, PALID_ENDING, 0x80);
        return Ok(DepoisClarear::Terminou);
    }
    Ok(DepoisClarear::Continua)
}

const END_DEMO_LEVELS: [u16; 8] = [
    0x0000, 0x0201, 0x0402, 0x0102, 0x0302, 0x0500, 0x0501, 0x0000,
];

pub fn preparar_pagina(m: &mut Memoria) -> Result<(), ErroAbertura> {
    super::plc::clear_plc(m);
    super::titulo::zerar(m, V_OBJSPACE, V_OBJSPACE_END);
    super::sega::clear_screen(m)?;
    for w in [0x8B03u16, 0x8230, 0x8407, 0x857C, 0x9001, 0x8004, 0x8720] {
        ww(m, VDP_CONTROLE, w);
    }
    wb(m, F_WTR_STATE, 0);

    wl(
        m,
        VDP_CONTROLE,
        super::titulo::loc_vram(u32::from(ARTTILE_CREDITS_FONT)),
    );
    crate::rotinas::nem_dec(m, NEM_CREDITTEXT).map_err(ErroAbertura::Rotina)?;

    super::titulo::zerar(m, V_PALETTE_FADING, V_PALETTE_FADING_END);
    super::sega::pal_load_para(m, PALID_SONIC, 0x80);

    wb(m, V_CREDITS.wrapping_add(OB_ID), 0x8A);
    objetos::execute_objects(m)?;
    objetos::build_sprites(m)?;

    ending_demo_load(m)?;

    ww(m, V_GENERICTIMER, 120);
    super::titulo::palette_fade_in_inicio(m);
    Ok(())
}

pub fn ending_demo_load(m: &mut Memoria) -> Result<(), ErroAbertura> {
    let num = rw(m, V_CREDITSNUM) as usize;
    let zone_act = END_DEMO_LEVELS[num.min(7)];
    ww(m, V_ZONE_ACT, zone_act);

    let next_num = (num as u16).wrapping_add(1);
    ww(m, V_CREDITSNUM, next_num);

    if next_num < 9 {
        ww(m, F_DEMO, 0x8001);
        wb(m, V_GAMEMODE, 8);
        wb(m, V_LIVES, 3);
        ww(m, V_RINGS, 0);
        wl(m, V_TIME, 0);
        wl(m, V_SCORE, 0);
        wb(m, V_LASTLAMP, 0);

        if next_num == 4 {
            wb(m, V_LASTLAMP, 1);
            wb(m, V_LASTLAMP.wrapping_add(1), 1);
            ww(m, V_LASTLAMP.wrapping_add(2), 0x0A00);
            ww(m, V_LASTLAMP.wrapping_add(4), 0x062C);
            ww(m, V_LASTLAMP.wrapping_add(6), 13);
            wl(m, V_LASTLAMP.wrapping_add(8), 0);
            wb(m, V_LASTLAMP.wrapping_add(12), 0);
            wb(m, V_LASTLAMP.wrapping_add(13), 0);
            ww(m, V_LASTLAMP.wrapping_add(14), 0x0800);
            ww(m, V_LASTLAMP.wrapping_add(16), 0x0957);
            ww(m, V_LASTLAMP.wrapping_add(18), 0x05CC);
            ww(m, V_LASTLAMP.wrapping_add(20), 0x04AB);
            ww(m, V_LASTLAMP.wrapping_add(22), 0x03A6);
            ww(m, V_LASTLAMP.wrapping_add(24), 0);
            ww(m, V_LASTLAMP.wrapping_add(26), 0x028C);
            ww(m, V_LASTLAMP.wrapping_add(28), 0);
            ww(m, V_LASTLAMP.wrapping_add(30), 0);
            ww(m, V_LASTLAMP.wrapping_add(32), 0x0308);
            wb(m, V_LASTLAMP.wrapping_add(34), 1);
            wb(m, V_LASTLAMP.wrapping_add(35), 1);
        }

        let zona = rb(m, V_ZONE);
        let cabecalho = LEVEL_HEADERS.wrapping_add(u32::from(zona).wrapping_mul(0x10));
        let primeira = super::boot::rom_b(m, cabecalho);
        if primeira != 0 {
            super::plc::add_plc(m, u16::from(primeira))?;
        }
        super::plc::add_plc(m, PLCID_MAIN2)?;
    }
    Ok(())
}

pub fn try_again_preparar(m: &mut Memoria) -> Result<(), ErroAbertura> {
    super::plc::clear_plc(m);
    super::titulo::zerar(m, V_OBJSPACE, V_OBJSPACE_END);
    super::sega::clear_screen(m)?;
    for w in [0x8B03u16, 0x8230, 0x8407, 0x857C, 0x9001, 0x8004, 0x8720] {
        ww(m, VDP_CONTROLE, w);
    }
    wb(m, F_WTR_STATE, 0);

    super::plc::quick_plc(m, PLCID_TRYAGAIN)?;
    super::titulo::zerar(m, V_PALETTE_FADING, V_PALETTE_FADING_END);
    super::sega::pal_load_para(m, PALID_ENDING, 0x80);
    ww(m, V_PALETTE_FADING_LINE_3, 0);

    wb(m, V_ENDEGGMAN.wrapping_add(OB_ID), 0x8B);
    objetos::execute_objects(m)?;
    objetos::build_sprites(m)?;

    ww(m, V_GENERICTIMER, 1800);
    super::titulo::palette_fade_in_inicio(m);
    Ok(())
}

pub fn try_again_passo(m: &mut Memoria) -> Result<bool, ErroAbertura> {
    super::fase::pause_game(m);
    objetos::execute_objects(m)?;
    objetos::build_sprites(m)?;

    let t = rw(m, V_GENERICTIMER);
    let start_pressed = rb(m, V_JPADPRESS1) & BTN_START != 0;
    if t == 0 || start_pressed {
        return Ok(true);
    }
    Ok(false)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn end_sonic_com_seis_esmeraldas() {
        let mut m = Memoria::nova(vec![0; 0x1000]);
        let a0: u32 = 0xFF_D800;
        wb(&mut m, a0.wrapping_add(OB_ID), 0x87);
        wb(&mut m, a0.wrapping_add(OB_2NDROUT), 0);
        wb(&mut m, V_EMERALDS, 6);

        let res = end_sonic(&mut m, a0);
        assert!(res.is_ok());
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_2NDROUT)), 2);
        assert_eq!(rw(&mut m, a0.wrapping_add(0x30)), 80);
    }

    #[test]
    fn end_sonic_sem_esmeraldas() {
        let mut m = Memoria::nova(vec![0; 0x1000]);
        let a0: u32 = 0xFF_D800;
        wb(&mut m, a0.wrapping_add(OB_ID), 0x87);
        wb(&mut m, a0.wrapping_add(OB_2NDROUT), 0);
        wb(&mut m, V_EMERALDS, 3);

        let res = end_sonic(&mut m, a0);
        assert!(res.is_ok());
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_2NDROUT)), 0x10);
        assert_eq!(rw(&mut m, a0.wrapping_add(0x30)), 216);
    }
}
