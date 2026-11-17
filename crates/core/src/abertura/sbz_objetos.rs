//! Objetos interativos de Scrap Brain Zone (SBZ1, SBZ2 e SBZ3).
use super::ram::*;
use super::{ErroAbertura, fisica, objetos, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;

const MAP_ADOOR: u32 = 0x00906A;
const ANI_ADOOR: u32 = 0x00904E;
const MAP_JUN: u32 = 0x0159FA;
const MAP_DISC: u32 = 0x015DEE;
const MAP_SPIN: u32 = 0x0160A2;
const MAP_TRAP: u32 = 0x016048;
const ANI_SPIN: u32 = 0x01600C;
const MAP_SAW: u32 = 0x016300;
const MAP_STOMP: u32 = 0x0166D0;
const MAP_VANP: u32 = 0x016892;
const MAP_ELEC: u32 = 0x016948;
const ANI_ELEC: u32 = 0x016930;
const MAP_FLAME: u32 = 0x00EC82;
const ANI_FLAME: u32 = 0x00EC4A;
const MAP_GIRD: u32 = 0x016D90;

const ARTTILE_SBZ_DOOR: u16 = 0x02E8;
const ARTTILE_SBZ_GIRDER: u16 = 0x02F0;
const ARTTILE_SBZ_DISC: u16 = 0x0344;
const ARTTILE_SBZ_JUNCTION: u16 = 0x0348;
const ARTTILE_SBZ_SAW: u16 = 0x03B5;
const ARTTILE_SBZ_FLAMETHROWER: u16 = 0x03D9;
const ARTTILE_SBZ_ELECTRIC_ORB: u16 = 0x047E;
const ARTTILE_SBZ_TRAP_DOOR: u16 = 0x0492;
const ARTTILE_SBZ_VANISHING_BLOCK: u16 = 0x04C3;
const ARTTILE_SBZ_SPINNING_PLATFORM: u16 = 0x04DF;
const ARTTILE_SBZ_MOVING_BLOCK_SHORT: u16 = 0x02C0;

const SFX_ROLL: u8 = 0xBE;

/// Objeto 0x2A: Porta vertical pequena automatica (SBZ).
pub fn auto_door(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        wb(m, a0.wrapping_add(OB_ROUTINE), 2);
        wl(m, a0.wrapping_add(OB_MAP), MAP_ADOOR);
        ww(m, a0.wrapping_add(OB_GFX), ARTTILE_SBZ_DOOR | 0x6000); // Pal 3
        wb(m, a0.wrapping_add(OB_RENDER), 4);
        wb(m, a0.wrapping_add(OB_ACTWID), 8);
        wb(m, a0.wrapping_add(OB_PRIORITY), 4);
    }

    let px = rw(m, V_PLAYER.wrapping_add(OB_X));
    let door_x = rw(m, a0.wrapping_add(OB_X));
    let xflip = rb(m, a0.wrapping_add(OB_STATUS)) & STATUS_XFLIP != 0;

    wb(m, a0.wrapping_add(OB_ANIM), 0); // Fechando por padrao
    let dist = (px as i16).wrapping_sub(door_x as i16);
    if dist.abs() < 64 {
        if (xflip && dist < 0) || (!xflip && dist >= 0) {
            wb(m, a0.wrapping_add(OB_ANIM), 1); // Abrindo
        }
    }

    let _ = objetos::animate_sprite(m, a0, ANI_ADOOR);
    if rb(m, a0.wrapping_add(OB_FRAME)) == 0 {
        let d1 = 6u16.wrapping_add(SONIC_SOLID_WIDTH);
        let _ = objetos::solid_object(m, a0, d1, 32, 33);
    }
    objetos::display_sprite(m, a0);
    Ok(())
}

/// Objeto 0x66: Juncao circular giratoria que captura Sonic (SBZ).
pub fn junction(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        wb(m, a0.wrapping_add(OB_ROUTINE), 2);
        wl(m, a0.wrapping_add(OB_MAP), MAP_JUN);
        ww(m, a0.wrapping_add(OB_GFX), ARTTILE_SBZ_JUNCTION | 0x6000);
        wb(m, a0.wrapping_add(OB_RENDER), 4);
        wb(m, a0.wrapping_add(OB_ACTWID), 48);
        wb(m, a0.wrapping_add(OB_PRIORITY), 4);
        wb(m, a0.wrapping_add(0x34), 1); // direcao 1 (horario)
    }

    // Rotaciona a juncao
    let frame = rb(m, a0.wrapping_add(OB_FRAME));
    let dir = rb(m, a0.wrapping_add(0x34));
    if rw(m, V_FRAMECOUNT) & 3 == 0 {
        let novo_frame = if dir == 1 {
            (frame + 1) & 0x0F
        } else {
            (frame.wrapping_sub(1)) & 0x0F
        };
        wb(m, a0.wrapping_add(OB_FRAME), novo_frame);
    }

    let junc_x = rw(m, a0.wrapping_add(OB_X));
    let d1 = 37u16.wrapping_add(SONIC_SOLID_WIDTH);
    let _ = objetos::solid_object(m, a0, d1, 37, 38);

    // Se Sonic estiver empurrando contra a entrada
    if rb(m, a0.wrapping_add(OB_STATUS)) & STATUS_PUSHING != 0 {
        let cur_frame = rb(m, a0.wrapping_add(OB_FRAME));
        let px = rw(m, V_PLAYER.wrapping_add(OB_X));
        let alvo_frame = if px < junc_x { 0x0E } else { 7 };
        if cur_frame == alvo_frame {
            wb(m, F_PLAYERCTRL, 1);
            wb(m, V_PLAYER.wrapping_add(OB_ANIM), ANIM_ROLL);
            ww(m, V_PLAYER.wrapping_add(OB_INERTIA), 0x800);
            ww(m, V_PLAYER.wrapping_add(OB_VELX), 0);
            ww(m, V_PLAYER.wrapping_add(OB_VELY), 0);
            let pst = rb(m, V_PLAYER.wrapping_add(OB_STATUS)) | STATUS_IN_AIR;
            wb(m, V_PLAYER.wrapping_add(OB_STATUS), pst & !STATUS_PUSHING);
            let ost = rb(m, a0.wrapping_add(OB_STATUS));
            wb(m, a0.wrapping_add(OB_STATUS), ost & !STATUS_PUSHING);
        }
    }

    objetos::display_sprite(m, a0);
    Ok(())
}

/// Objeto 0x67: Disco giratorio de corrida (SBZ).
pub fn running_disc(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        wb(m, a0.wrapping_add(OB_ROUTINE), 2);
        wl(m, a0.wrapping_add(OB_MAP), MAP_DISC);
        ww(m, a0.wrapping_add(OB_GFX), ARTTILE_SBZ_DISC | 0x6000);
        wb(m, a0.wrapping_add(OB_RENDER), 4);
        wb(m, a0.wrapping_add(OB_PRIORITY), 4);
        wb(m, a0.wrapping_add(OB_ACTWID), 8);
        let ox = rw(m, a0.wrapping_add(OB_X));
        let oy = rw(m, a0.wrapping_add(OB_Y));
        ww(m, a0.wrapping_add(0x32), ox);
        ww(m, a0.wrapping_add(0x30), oy);
        wb(m, a0.wrapping_add(0x34), 0x18);
        wb(m, a0.wrapping_add(0x38), 0x48);
    }

    // Gira o ponto visual dentro da engrenagem
    let ang = rb(m, a0.wrapping_add(OB_ANGLE)).wrapping_add(4);
    wb(m, a0.wrapping_add(OB_ANGLE), ang);

    let (s, c) = fisica::calc_sine(m, ang);
    let orig_x = rw(m, a0.wrapping_add(0x32));
    let orig_y = rw(m, a0.wrapping_add(0x30));
    let off_x = (i32::from(c).wrapping_mul(0x18) >> 8) as i16;
    let off_y = (i32::from(s).wrapping_mul(0x18) >> 8) as i16;
    ww(m, a0.wrapping_add(OB_X), orig_x.wrapping_add(off_x as u16));
    ww(m, a0.wrapping_add(OB_Y), orig_y.wrapping_add(off_y as u16));

    objetos::display_sprite(m, a0);
    Ok(())
}

/// Objeto 0x68: Esteiras rolantes horizontais da SBZ.
pub fn conveyor(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        wb(m, a0.wrapping_add(OB_ROUTINE), 2);
        let sub = rb(m, a0.wrapping_add(OB_SUBTYPE));
        let width = if sub & 0x0F == 0 { 128 } else { 56 };
        wb(m, a0.wrapping_add(0x38), width);
        let spd = ((sub as i8) >> 4) as i16;
        ww(m, a0.wrapping_add(0x36), u16_de(spd));
    }

    // Move Sonic se ele estiver pisando sobre a esteira
    let px = rw(m, V_PLAYER.wrapping_add(OB_X));
    let py = rw(m, V_PLAYER.wrapping_add(OB_Y));
    let cx = rw(m, a0.wrapping_add(OB_X));
    let cy = rw(m, a0.wrapping_add(OB_Y));
    let wid = u16::from(rb(m, a0.wrapping_add(0x38)));

    let dx = (px as i16).wrapping_sub(cx as i16).abs() as u16;
    let dy = (py as i16).wrapping_sub(cy as i16);
    if dx < wid
        && (0..=32).contains(&dy)
        && rb(m, V_PLAYER.wrapping_add(OB_STATUS)) & STATUS_IN_AIR == 0
    {
        let spd = s16(rw(m, a0.wrapping_add(0x36)));
        let novo_px = px.wrapping_add(spd as u16);
        ww(m, V_PLAYER.wrapping_add(OB_X), novo_px);
    }

    Ok(())
}

/// Objeto 0x69: Plataformas giratorias e alcapoes (SBZ).
pub fn spin_platform(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        wb(m, a0.wrapping_add(OB_ROUTINE), 2);
        let sub = rb(m, a0.wrapping_add(OB_SUBTYPE));
        if sub & 0x40 == 0 {
            // Alcapao
            wl(m, a0.wrapping_add(OB_MAP), MAP_TRAP);
            ww(m, a0.wrapping_add(OB_GFX), ARTTILE_SBZ_TRAP_DOOR | 0x6000);
            wb(m, a0.wrapping_add(OB_ACTWID), 64);
        } else {
            // Plataforma giratoria
            wl(m, a0.wrapping_add(OB_MAP), MAP_SPIN);
            ww(
                m,
                a0.wrapping_add(OB_GFX),
                ARTTILE_SBZ_SPINNING_PLATFORM | 0x6000,
            );
            wb(m, a0.wrapping_add(OB_ACTWID), 32);
        }
        wb(m, a0.wrapping_add(OB_RENDER), 4);
        wb(m, a0.wrapping_add(OB_PRIORITY), 4);
        let sec = u16::from(sub & 0x0F);
        ww(m, a0.wrapping_add(0x32), sec.wrapping_mul(60));
    }

    let t = rw(m, a0.wrapping_add(0x30));
    if t == 0 {
        let base_t = rw(m, a0.wrapping_add(0x32));
        ww(m, a0.wrapping_add(0x30), base_t);
        let anim = rb(m, a0.wrapping_add(OB_ANIM)) ^ 1;
        wb(m, a0.wrapping_add(OB_ANIM), anim);
    } else {
        ww(m, a0.wrapping_add(0x30), t.wrapping_sub(1));
    }

    let _ = objetos::animate_sprite(m, a0, ANI_SPIN);
    if rb(m, a0.wrapping_add(OB_FRAME)) == 0 {
        let wid = u16::from(rb(m, a0.wrapping_add(OB_ACTWID)));
        let d1 = wid.wrapping_add(SONIC_SOLID_WIDTH);
        let _ = objetos::solid_object(m, a0, d1, 8, 9);
    }
    objetos::display_sprite(m, a0);
    Ok(())
}

/// Objeto 0x6A: Serras e cortadores de pizza (SBZ).
pub fn saws(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        wb(m, a0.wrapping_add(OB_ROUTINE), 2);
        wl(m, a0.wrapping_add(OB_MAP), MAP_SAW);
        ww(m, a0.wrapping_add(OB_GFX), ARTTILE_SBZ_SAW | 0x6000);
        wb(m, a0.wrapping_add(OB_RENDER), 4);
        wb(m, a0.wrapping_add(OB_PRIORITY), 4);
        wb(m, a0.wrapping_add(OB_ACTWID), 32);
        let ox = rw(m, a0.wrapping_add(OB_X));
        let oy = rw(m, a0.wrapping_add(OB_Y));
        ww(m, a0.wrapping_add(0x3A), ox);
        ww(m, a0.wrapping_add(0x38), oy);

        if rb(m, a0.wrapping_add(OB_SUBTYPE)) < 3 {
            wb(m, a0.wrapping_add(OB_COLTYPE), 0x8C); // dano 48x48
        }
    }

    let sub = rb(m, a0.wrapping_add(OB_SUBTYPE));
    match sub {
        1 => {
            // Move esquerda/direita oscilando
            let ang = (rw(m, V_FRAMECOUNT) & 0xFF) as u8;
            let (_, c) = fisica::calc_sine(m, ang);
            let orig_x = rw(m, a0.wrapping_add(0x3A));
            let off = (i32::from(c).wrapping_mul(0x40) >> 8) as i16;
            ww(m, a0.wrapping_add(OB_X), orig_x.wrapping_add(off as u16));
        }
        2 => {
            // Move cima/baixo oscilando
            let ang = (rw(m, V_FRAMECOUNT) & 0xFF) as u8;
            let (s, _) = fisica::calc_sine(m, ang);
            let orig_y = rw(m, a0.wrapping_add(0x38));
            let off = (i32::from(s).wrapping_mul(0x40) >> 8) as i16;
            ww(m, a0.wrapping_add(OB_Y), orig_y.wrapping_add(off as u16));
        }
        3..=4 => {
            // Serra veloz em linha reta
            fisica::speed_to_pos(m, a0);
        }
        _ => {}
    }

    let fr = (rb(m, a0.wrapping_add(OB_FRAME)) + 1) & 1;
    wb(m, a0.wrapping_add(OB_FRAME), fr);
    objetos::display_sprite(m, a0);
    Ok(())
}

/// Objeto 0x6B: Prensa e portas de correr (SBZ).
pub fn scrap_stomp(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        wb(m, a0.wrapping_add(OB_ROUTINE), 2);
        wl(m, a0.wrapping_add(OB_MAP), MAP_STOMP);
        ww(
            m,
            a0.wrapping_add(OB_GFX),
            ARTTILE_SBZ_MOVING_BLOCK_SHORT | 0x4000,
        );
        wb(m, a0.wrapping_add(OB_RENDER), 4);
        wb(m, a0.wrapping_add(OB_PRIORITY), 4);
        wb(m, a0.wrapping_add(OB_ACTWID), 32);
        wb(m, a0.wrapping_add(OB_HEIGHT), 32);
    }

    let d1 = 32u16.wrapping_add(SONIC_SOLID_WIDTH);
    let _ = objetos::solid_object(m, a0, d1, 32, 33);
    objetos::display_sprite(m, a0);
    Ok(())
}

/// Objeto 0x6C: Plataformas evanescentes (SBZ).
pub fn vanish_platform(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        wb(m, a0.wrapping_add(OB_ROUTINE), 2);
        wl(m, a0.wrapping_add(OB_MAP), MAP_VANP);
        ww(
            m,
            a0.wrapping_add(OB_GFX),
            ARTTILE_SBZ_VANISHING_BLOCK | 0x6000,
        );
        wb(m, a0.wrapping_add(OB_RENDER), 4);
        wb(m, a0.wrapping_add(OB_PRIORITY), 4);
        wb(m, a0.wrapping_add(OB_ACTWID), 16);
        let sub = u16::from(rb(m, a0.wrapping_add(OB_SUBTYPE)) & 0x0F);
        let delay = sub.wrapping_add(1).wrapping_shl(7); // * 128
        ww(m, a0.wrapping_add(0x30), delay);
        ww(m, a0.wrapping_add(0x32), delay);
    }

    let t = rw(m, a0.wrapping_add(0x30));
    if t == 0 {
        let base_t = rw(m, a0.wrapping_add(0x32));
        ww(m, a0.wrapping_add(0x30), base_t);
        let vis = rb(m, a0.wrapping_add(OB_FRAME)) ^ 1;
        wb(m, a0.wrapping_add(OB_FRAME), vis);
    } else {
        ww(m, a0.wrapping_add(0x30), t.wrapping_sub(1));
    }

    if rb(m, a0.wrapping_add(OB_FRAME)) == 0 {
        let d1 = 16u16.wrapping_add(SONIC_SOLID_WIDTH);
        let _ = objetos::solid_object(m, a0, d1, 16, 17);
        objetos::display_sprite(m, a0);
    }
    Ok(())
}

/// Objeto 0x6D: Lancador de chamas (SBZ).
pub fn flamethrower(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        wb(m, a0.wrapping_add(OB_ROUTINE), 2);
        wl(m, a0.wrapping_add(OB_MAP), MAP_FLAME);
        ww(
            m,
            a0.wrapping_add(OB_GFX),
            ARTTILE_SBZ_FLAMETHROWER | 0x8000,
        );
        wb(m, a0.wrapping_add(OB_RENDER), 4);
        wb(m, a0.wrapping_add(OB_PRIORITY), 1);
        wb(m, a0.wrapping_add(OB_ACTWID), 12);
        let sub = rb(m, a0.wrapping_add(OB_SUBTYPE));
        let ftime = u16::from(sub & 0xF0).wrapping_mul(2);
        let ptime = u16::from(sub & 0x0F).wrapping_shl(5);
        ww(m, a0.wrapping_add(0x30), ftime);
        ww(m, a0.wrapping_add(0x32), ftime);
        ww(m, a0.wrapping_add(0x34), ptime);
    }

    let t = rw(m, a0.wrapping_add(0x30));
    if t == 0 {
        if rb(m, a0.wrapping_add(OB_COLTYPE)) == 0 {
            // Comeca a queimar
            let ftime = rw(m, a0.wrapping_add(0x32));
            ww(m, a0.wrapping_add(0x30), ftime);
            wb(m, a0.wrapping_add(OB_COLTYPE), 0x8B);
            wb(m, a0.wrapping_add(OB_ANIM), 1);
        } else {
            // Pausa
            let ptime = rw(m, a0.wrapping_add(0x34));
            ww(m, a0.wrapping_add(0x30), ptime);
            wb(m, a0.wrapping_add(OB_COLTYPE), 0);
            wb(m, a0.wrapping_add(OB_ANIM), 0);
        }
    } else {
        ww(m, a0.wrapping_add(0x30), t.wrapping_sub(1));
    }

    let _ = objetos::animate_sprite(m, a0, ANI_FLAME);
    objetos::display_sprite(m, a0);
    Ok(())
}

/// Objeto 0x6E: Orbe de eletrocussao (SBZ).
pub fn electro(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        wb(m, a0.wrapping_add(OB_ROUTINE), 2);
        wl(m, a0.wrapping_add(OB_MAP), MAP_ELEC);
        ww(m, a0.wrapping_add(OB_GFX), ARTTILE_SBZ_ELECTRIC_ORB);
        wb(m, a0.wrapping_add(OB_RENDER), 4);
        wb(m, a0.wrapping_add(OB_ACTWID), 40);
        let sub = rb(m, a0.wrapping_add(OB_SUBTYPE));
        let freq = (u16::from(sub).wrapping_shl(4)).wrapping_sub(1);
        ww(m, a0.wrapping_add(0x34), freq);
    }

    let freq = rw(m, a0.wrapping_add(0x34));
    if rw(m, V_FRAMECOUNT) & freq == 0 {
        wb(m, a0.wrapping_add(OB_ANIM), 1); // Disparar choque
        wb(m, a0.wrapping_add(OB_COLTYPE), 0x8A);
    }

    let _ = objetos::animate_sprite(m, a0, ANI_ELEC);
    if rb(m, a0.wrapping_add(OB_FRAME)) == 0 {
        wb(m, a0.wrapping_add(OB_COLTYPE), 0);
    }
    objetos::display_sprite(m, a0);
    Ok(())
}

/// Objeto 0x6F: Plataformas giratorias em esteira (SBZ).
pub fn spin_convey(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        wb(m, a0.wrapping_add(OB_ROUTINE), 2);
        wl(m, a0.wrapping_add(OB_MAP), MAP_SPIN);
        ww(
            m,
            a0.wrapping_add(OB_GFX),
            ARTTILE_SBZ_SPINNING_PLATFORM | 0x6000,
        );
        wb(m, a0.wrapping_add(OB_RENDER), 4);
        wb(m, a0.wrapping_add(OB_PRIORITY), 4);
        wb(m, a0.wrapping_add(OB_ACTWID), 32);
    }

    let d1 = 32u16.wrapping_add(SONIC_SOLID_WIDTH);
    let _ = objetos::solid_object(m, a0, d1, 8, 9);
    objetos::display_sprite(m, a0);
    Ok(())
}

/// Objeto 0x70: Bloco de viga deslizante (SBZ).
pub fn girder(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        wb(m, a0.wrapping_add(OB_ROUTINE), 2);
        wl(m, a0.wrapping_add(OB_MAP), MAP_GIRD);
        ww(m, a0.wrapping_add(OB_GFX), ARTTILE_SBZ_GIRDER | 0x6000);
        wb(m, a0.wrapping_add(OB_RENDER), 4);
        wb(m, a0.wrapping_add(OB_PRIORITY), 4);
        wb(m, a0.wrapping_add(OB_ACTWID), 96);
        wb(m, a0.wrapping_add(OB_HEIGHT), 24);
        let ox = rw(m, a0.wrapping_add(OB_X));
        let oy = rw(m, a0.wrapping_add(OB_Y));
        ww(m, a0.wrapping_add(0x32), ox);
        ww(m, a0.wrapping_add(0x30), oy);
    }

    let d1 = 96u16.wrapping_add(SONIC_SOLID_WIDTH);
    let _ = objetos::solid_object(m, a0, d1, 24, 25);
    objetos::display_sprite(m, a0);
    Ok(())
}

/// Objeto 0x72: Tubos de teletransporte (SBZ2).
pub fn teleport(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    match rotina {
        0 => tele_main(m, a0),
        2 => tele_action(m, a0),
        4 => tele_prebump(m, a0),
        _ => tele_teleporting(m, a0),
    }
}

// Destinos dos teletransportadores por subtipo (X, Y)
const TELE_DESTS: [[(u16, u16); 7]; 8] = [
    [
        (0x794, 0x98C),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
    ],
    [
        (0x94, 0x38C),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
    ],
    [
        (0x794, 0x2E8),
        (0x7A4, 0x2C0),
        (0x7D0, 0x2AC),
        (0x858, 0x2AC),
        (0x884, 0x298),
        (0x894, 0x270),
        (0x894, 0x190),
    ],
    [
        (0x894, 0x690),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
    ],
    [
        (0x1194, 0x470),
        (0x1184, 0x498),
        (0x1158, 0x4AC),
        (0xFD0, 0x4AC),
        (0xFA4, 0x4C0),
        (0xF94, 0x4E8),
        (0xF94, 0x590),
    ],
    [
        (0x1294, 0x490),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
    ],
    [
        (0x1594, 0xFFE8),
        (0x1584, 0xFFC0),
        (0x1560, 0xFFAC),
        (0x14D0, 0xFFAC),
        (0x14A4, 0xFF98),
        (0x1494, 0xFF70),
        (0x1494, 0xFD90),
    ],
    [
        (0x894, 0x90),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
    ],
];

const TELE_COUNTS: [usize; 8] = [1, 1, 7, 1, 7, 1, 7, 1];

fn tele_main(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    wb(m, a0.wrapping_add(OB_ROUTINE), 2);
    wb(m, a0.wrapping_add(0x3A), 0); // tele_current
    let sub = (rb(m, a0.wrapping_add(OB_SUBTYPE)) & 7) as usize;
    let (tx, ty) = TELE_DESTS[sub][0];
    ww(m, a0.wrapping_add(0x36), tx);
    ww(m, a0.wrapping_add(0x38), ty);
    tele_action(m, a0)
}

fn tele_action(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let px = rw(m, V_PLAYER.wrapping_add(OB_X));
    let py = rw(m, V_PLAYER.wrapping_add(OB_Y));
    let tx = rw(m, a0.wrapping_add(OB_X));
    let ty = rw(m, a0.wrapping_add(OB_Y));

    let dx = (px as i16).wrapping_sub(tx as i16).abs();
    let dy = (py as i16).wrapping_sub(ty as i16).abs();

    if dx < 16 && dy < 32 && rb(m, F_PLAYERCTRL) == 0 {
        let sub = rb(m, a0.wrapping_add(OB_SUBTYPE));
        if sub == 7 && rw(m, V_RINGS) < 50 {
            return Ok(());
        }

        wb(m, a0.wrapping_add(OB_ROUTINE), 4);
        wb(m, F_PLAYERCTRL, 0x81);
        wb(m, V_PLAYER.wrapping_add(OB_ANIM), ANIM_ROLL);
        ww(m, V_PLAYER.wrapping_add(OB_INERTIA), 0x800);
        ww(m, V_PLAYER.wrapping_add(OB_VELX), 0);
        ww(m, V_PLAYER.wrapping_add(OB_VELY), 0);
        let pst = rb(m, V_PLAYER.wrapping_add(OB_STATUS)) | STATUS_IN_AIR;
        wb(m, V_PLAYER.wrapping_add(OB_STATUS), pst & !STATUS_PUSHING);
        ww(m, V_PLAYER.wrapping_add(OB_X), tx);
        ww(m, V_PLAYER.wrapping_add(OB_Y), ty);
        wb(m, a0.wrapping_add(0x32), 0); // prebump
        wb(m, V_SOUNDQUEUE1, SFX_ROLL);
    }
    Ok(())
}

fn tele_prebump(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let bump = rb(m, a0.wrapping_add(0x32)).wrapping_add(2);
    wb(m, a0.wrapping_add(0x32), bump);
    if bump >= 0x80 {
        wb(m, a0.wrapping_add(OB_ROUTINE), 6);
        tele_next_direction(m, a0);
    }
    Ok(())
}

fn tele_teleporting(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let t = rw(m, a0.wrapping_add(0x2E));
    if t == 0 {
        let curr = (rb(m, a0.wrapping_add(0x3A)) + 1) as usize;
        let sub = (rb(m, a0.wrapping_add(OB_SUBTYPE)) & 7) as usize;
        if curr >= TELE_COUNTS[sub] {
            // Sai do teletransporte
            let py = rw(m, V_PLAYER.wrapping_add(OB_Y)) & 0x7FF;
            ww(m, V_PLAYER.wrapping_add(OB_Y), py);
            wb(m, a0.wrapping_add(OB_ROUTINE), 0);
            wb(m, F_PLAYERCTRL, 0);
            ww(m, V_PLAYER.wrapping_add(OB_VELX), 0);
            ww(m, V_PLAYER.wrapping_add(OB_VELY), 0x200);
            return Ok(());
        }
        wb(m, a0.wrapping_add(0x3A), curr as u8);
        let (tx, ty) = TELE_DESTS[sub][curr];
        ww(m, a0.wrapping_add(0x36), tx);
        ww(m, a0.wrapping_add(0x38), ty);
        tele_next_direction(m, a0);
    } else {
        ww(m, a0.wrapping_add(0x2E), t.wrapping_sub(1));
    }

    fisica::speed_to_pos(m, V_PLAYER);
    Ok(())
}

fn tele_next_direction(m: &mut Memoria, a0: u32) {
    let px = rw(m, V_PLAYER.wrapping_add(OB_X));
    let py = rw(m, V_PLAYER.wrapping_add(OB_Y));
    let tx = rw(m, a0.wrapping_add(0x36));
    let ty = rw(m, a0.wrapping_add(0x38));

    let dx = (tx as i16).wrapping_sub(px as i16);
    let dy = (ty as i16).wrapping_sub(py as i16);

    let abs_dx = dx.abs() as u16;
    let abs_dy = dy.abs() as u16;

    if abs_dx >= abs_dy {
        let vx = if dx >= 0 { 0x1000i16 } else { -0x1000i16 };
        let steps = abs_dx.max(1);
        let vy = ((i32::from(dy) << 12) / i32::from(steps)) as i16;
        ww(m, V_PLAYER.wrapping_add(OB_VELX), u16_de(vx));
        ww(m, V_PLAYER.wrapping_add(OB_VELY), u16_de(vy));
        ww(m, a0.wrapping_add(0x2E), steps >> 4);
    } else {
        let vy = if dy >= 0 { 0x1000i16 } else { -0x1000i16 };
        let steps = abs_dy.max(1);
        let vx = ((i32::from(dx) << 12) / i32::from(steps)) as i16;
        ww(m, V_PLAYER.wrapping_add(OB_VELX), u16_de(vx));
        ww(m, V_PLAYER.wrapping_add(OB_VELY), u16_de(vy));
        ww(m, a0.wrapping_add(0x2E), steps >> 4);
    }
}

fn s16(w: u16) -> i16 {
    i16::from_be_bytes(w.to_be_bytes())
}

fn u16_de(i: i16) -> u16 {
    u16::from_be_bytes(i.to_be_bytes())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn auto_door_inicializacao() {
        let mut m = Memoria::nova(vec![0; 0x1000]);
        let a0: u32 = 0xFF_D800;
        wb(&mut m, a0.wrapping_add(OB_ID), 0x2A);
        wb(&mut m, a0.wrapping_add(OB_ROUTINE), 0);
        ww(&mut m, a0.wrapping_add(OB_X), 0x100);
        ww(&mut m, a0.wrapping_add(OB_Y), 0x100);

        let res = auto_door(&mut m, a0);
        assert!(res.is_ok());
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ACTWID)), 8);
    }

    #[test]
    fn junction_inicializacao() {
        let mut m = Memoria::nova(vec![0; 0x1000]);
        let a0: u32 = 0xFF_D800;
        wb(&mut m, a0.wrapping_add(OB_ID), 0x66);
        wb(&mut m, a0.wrapping_add(OB_ROUTINE), 0);
        ww(&mut m, a0.wrapping_add(OB_X), 0x100);
        ww(&mut m, a0.wrapping_add(OB_Y), 0x100);

        let res = junction(&mut m, a0);
        assert!(res.is_ok());
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ACTWID)), 48);
    }
}
