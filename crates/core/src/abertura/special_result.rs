//! Fades brancos e cartões de resultado da Special Stage.
use super::boot::rom_w;
use super::objetos::{delete_object, display_sprite};
use super::ram::*;
use super::{ErroAbertura, rb, rl, rw, wb, wl, ww};
use crate::memoria::Memoria;
use crate::rotinas;

const V_SSRESCARD: u32 = 0xFF_D5C0;
const V_SSRESCONTINUE: u32 = 0xFF_D6C0;
const V_SSRESEMERALDS: u32 = 0xFF_D800;
const V_EMLDLIST: u32 = 0xFF_FE58;
const SSR_ITEM_DATA: u32 = 0xCFBC;
const SSRC_POS_DATA: u32 = 0xCFF6;
const SSR_MAIN_X: u32 = 0x30;

/// Preenche a paleta ativa com branco para o aparecimento.
pub fn iniciar_branco(m: &mut Memoria) {
    ww(m, V_PFADE_START, 0x003F);
    for i in 0..64u32 {
        ww(m, V_PALETTE.wrapping_add(i.wrapping_mul(2)), 0xEEE);
    }
}

/// Avança um passo de fade para branco, inclusive na paleta submersa.
pub fn clarear(m: &mut Memoria) {
    let inicio = u32::from(rb(m, V_PFADE_START));
    let tamanho = u32::from(rb(m, V_PFADE_START.wrapping_add(1)));
    for base in [V_PALETTE, V_PALETTE_WATER] {
        for i in 0..=tamanho {
            let endereco = base.wrapping_add(inicio).wrapping_add(i.wrapping_mul(2));
            let cor = rw(m, endereco);
            let passo = if cor & 0x00E != 0x00E {
                2
            } else if cor & 0x0E0 != 0x0E0 {
                0x20
            } else if cor & 0xE00 != 0xE00 {
                0x200
            } else {
                0
            };
            ww(m, endereco, cor.wrapping_add(passo));
        }
    }
}

/// Avança um passo do branco para a paleta de destino.
pub fn escurecer(m: &mut Memoria) {
    let inicio = u32::from(rb(m, V_PFADE_START));
    let tamanho = u32::from(rb(m, V_PFADE_START.wrapping_add(1)));
    for (ativa, destino) in [
        (V_PALETTE, V_PALETTE_FADING),
        (V_PALETTE_WATER, V_PALETTE_WATER_FADING),
    ] {
        if ativa == V_PALETTE_WATER && rb(m, V_ZONE) != ID_LZ {
            continue;
        }
        for i in 0..=tamanho {
            let desloc = inicio.wrapping_add(i.wrapping_mul(2));
            let endereco = ativa.wrapping_add(desloc);
            let cor = rw(m, endereco);
            let alvo = rw(m, destino.wrapping_add(desloc));
            if cor == alvo {
                continue;
            }
            let nova = if cor.checked_sub(0x200).is_some_and(|v| v >= alvo) {
                cor.wrapping_sub(0x200)
            } else if cor.checked_sub(0x20).is_some_and(|v| v >= alvo) {
                cor.wrapping_sub(0x20)
            } else {
                cor.wrapping_sub(2)
            };
            ww(m, endereco, nova);
        }
    }
}

/// Prepara VRAM, paleta, fila de arte e objeto de resultados.
pub fn preparar(m: &mut Memoria) -> Result<(), ErroAbertura> {
    for reg in [0x8230, 0x8407, 0x9001] {
        ww(m, VDP_CONTROLE, reg);
    }
    super::sega::clear_screen(m)?;
    wl(
        m,
        VDP_CONTROLE,
        super::titulo::loc_vram(u32::from(ARTTILE_TITLE_CARD)),
    );
    rotinas::nem_dec(m, NEM_TITLECARD).map_err(ErroAbertura::Rotina)?;
    super::hud::hud_base(m);
    super::sega::pal_load(m, 0x11);
    super::plc::new_plc(m, PLCID_MAIN);
    super::plc::add_plc(m, 0x1B)?;
    wb(m, F_SCORECOUNT, 1);
    wb(m, F_ENDACTBONUS, 1);
    let bonus = rw(m, V_RINGS).wrapping_mul(10);
    ww(m, V_RINGBONUS, bonus);
    wb(m, V_SOUNDQUEUE1, 0x8E);
    super::titulo::zerar(m, V_OBJSPACE, V_OBJSPACE_END);
    wb(m, V_SSRESCARD, 0x7E);
    Ok(())
}

/// Objeto 0x7E, elementos dos cartões de resultado.
pub fn cartao(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            if rl(m, V_PLC_BUFFER) != 0 {
                return Ok(());
            }
            let quantidade: u32 = if rw(m, V_RINGS) >= 50 { 5 } else { 4 };
            for i in 0..quantidade {
                let slot = a.wrapping_add(i.wrapping_mul(OBJECT_SIZE));
                let entrada = SSR_ITEM_DATA.wrapping_add(i.wrapping_mul(8));
                wb(m, slot.wrapping_add(OB_ID), 0x7E);
                let x = rom_w(m, entrada);
                let alvo = rom_w(m, entrada.wrapping_add(2));
                let y = rom_w(m, entrada.wrapping_add(4));
                ww(m, slot.wrapping_add(OB_X), x);
                ww(m, slot.wrapping_add(SSR_MAIN_X), alvo);
                ww(m, slot.wrapping_add(OB_SCREENY), y);
                wb(
                    m,
                    slot.wrapping_add(OB_ROUTINE),
                    super::boot::rom_b(m, entrada.wrapping_add(6)),
                );
                wb(
                    m,
                    slot.wrapping_add(OB_FRAME),
                    super::boot::rom_b(m, entrada.wrapping_add(7)),
                );
                wl(m, slot.wrapping_add(OB_MAP), 0xD328);
                ww(m, slot.wrapping_add(OB_GFX), ARTTILE_TITLE_CARD | 0x8000);
                wb(m, slot.wrapping_add(OB_RENDER), 0);
            }
            let total = rb(m, V_EMERALDS);
            let frame = if total == 0 {
                7
            } else if total == 6 {
                8
            } else {
                0
            };
            wb(m, a.wrapping_add(OB_FRAME), frame);
            if total == 6 {
                ww(m, a.wrapping_add(OB_X), 0x18);
                ww(m, a.wrapping_add(SSR_MAIN_X), 0x118);
            }
            mover(m, a);
        }
        2 => mover(m, a),
        4 | 8 | 12 | 16 => {
            let tempo = rw(m, a.wrapping_add(OB_TIME_FRAME)).wrapping_sub(1);
            ww(m, a.wrapping_add(OB_TIME_FRAME), tempo);
            if tempo == 0 {
                let rotina = rb(m, a.wrapping_add(OB_ROUTINE)).wrapping_add(2);
                wb(m, a.wrapping_add(OB_ROUTINE), rotina);
            }
            display_sprite(m, a);
        }
        6 => {
            display_sprite(m, a);
            wb(m, F_ENDACTBONUS, 1);
            let bonus = rw(m, V_RINGBONUS);
            if bonus != 0 {
                ww(m, V_RINGBONUS, bonus.wrapping_sub(10));
                super::fisica::add_points(m, 10);
                if rb(m, V_VBLANK_BYTE) & 3 == 0 {
                    wb(m, V_SOUNDQUEUE1, SFX_SWITCH);
                }
            } else {
                wb(m, V_SOUNDQUEUE1, SFX_CASH);
                let continua = rw(m, V_RINGS) >= 50;
                wb(m, a.wrapping_add(OB_ROUTINE), if continua { 12 } else { 8 });
                ww(
                    m,
                    a.wrapping_add(OB_TIME_FRAME),
                    if continua { 60 } else { 180 },
                );
            }
        }
        10 | 18 => {
            ww(m, F_RESTART, 1);
            display_sprite(m, a);
        }
        14 => {
            wb(m, V_SSRESCONTINUE.wrapping_add(OB_FRAME), 4);
            wb(m, V_SSRESCONTINUE.wrapping_add(OB_ROUTINE), 20);
            wb(m, V_SOUNDQUEUE1, 0xBF);
            wb(m, a.wrapping_add(OB_ROUTINE), 16);
            ww(m, a.wrapping_add(OB_TIME_FRAME), 360);
            display_sprite(m, a);
        }
        20 => {
            if rb(m, V_VBLANK_BYTE) & 15 == 0 {
                let frame = rb(m, a.wrapping_add(OB_FRAME));
                wb(m, a.wrapping_add(OB_FRAME), frame ^ 1);
            }
            display_sprite(m, a);
        }
        _ => {
            return Err(ErroAbertura::NaoImplementado(
                "cartao de resultado da Special Stage",
            ));
        }
    }
    Ok(())
}

fn mover(m: &mut Memoria, a: u32) {
    let x = rw(m, a.wrapping_add(OB_X));
    let alvo = rw(m, a.wrapping_add(SSR_MAIN_X));
    let x = if x == alvo {
        if rb(m, a.wrapping_add(OB_FRAME)) == 2 {
            wb(m, a.wrapping_add(OB_ROUTINE), 4);
            ww(m, a.wrapping_add(OB_TIME_FRAME), 179);
            wb(m, V_SSRESEMERALDS, 0x7F);
        }
        x
    } else if i16::from_be_bytes(x.to_be_bytes()) < i16::from_be_bytes(alvo.to_be_bytes()) {
        x.wrapping_add(16)
    } else {
        x.wrapping_sub(16)
    };
    ww(m, a.wrapping_add(OB_X), x);
    if i16::from_be_bytes(x.to_be_bytes()) >= 0 && x < 0x200 {
        display_sprite(m, a);
    }
}

/// Objeto 0x7F, esmeraldas acumuladas exibidas no resultado.
pub fn esmeraldas(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            let total = rb(m, V_EMERALDS);
            if total == 0 {
                delete_object(m, a);
                return Ok(());
            }
            for i in 0..u32::from(total) {
                let slot = a.wrapping_add(i.wrapping_mul(OBJECT_SIZE));
                let x = rom_w(m, SSRC_POS_DATA.wrapping_add(i.wrapping_mul(2)));
                let frame = rb(m, V_EMLDLIST.wrapping_add(i));
                wb(m, slot.wrapping_add(OB_ID), 0x7F);
                ww(m, slot.wrapping_add(OB_X), x);
                ww(m, slot.wrapping_add(OB_SCREENY), 0xF0);
                wb(m, slot.wrapping_add(OB_FRAME), frame);
                wb(m, slot.wrapping_add(OB_ANIM), frame);
                wb(m, slot.wrapping_add(OB_ROUTINE), 2);
                wl(m, slot.wrapping_add(OB_MAP), 0xD482);
                ww(m, slot.wrapping_add(OB_GFX), 0x8541);
                wb(m, slot.wrapping_add(OB_RENDER), 0);
            }
        }
        2 => {}
        _ => {
            return Err(ErroAbertura::NaoImplementado(
                "esmeralda no resultado da Special Stage",
            ));
        }
    }
    let frame = rb(m, a.wrapping_add(OB_FRAME));
    let visivel = if frame == 6 {
        rb(m, a.wrapping_add(OB_ANIM))
    } else {
        6
    };
    wb(m, a.wrapping_add(OB_FRAME), visivel);
    display_sprite(m, a);
    Ok(())
}
