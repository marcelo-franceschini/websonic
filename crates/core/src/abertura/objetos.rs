//! Motor de objetos mínimo (sub-etapa 7.5): `ExecuteObjects`, `DisplaySprite`, `BuildSprites`
//! e o objeto `CreditsText`. Os objetos do original são código na ROM (tabela `Obj_Index`);
//! aqui cada identificador chama uma função nativa, citando o rótulo que reproduz.

use super::boot::rom_w;
use super::ram::*;
use super::{ErroAbertura, rb, rl, rw, wb, wl, ww};
use crate::memoria::Memoria;

const SPRITES_MAX: u8 = 80;
const SPRITELAYER_NUM: u32 = 8;
const SPRITELAYER_SIZE: u32 = 0x80;

/// Estende o sinal de uma palavra, como `movea.w` e `adda.w` fazem.
fn ext(a: u16) -> u32 {
    u32::from_be_bytes(i32::from(i16::from_be_bytes(a.to_be_bytes())).to_be_bytes())
}

/// `DeleteObject`: zera as 16 longas do objeto.
pub fn delete_object(m: &mut Memoria, a0: u32) {
    for i in 0..16u32 {
        wl(m, a0.wrapping_add(i.wrapping_mul(4)), 0);
    }
}

/// `DisplaySprite`: põe o objeto na fila de sprites da sua prioridade.
pub fn display_sprite(m: &mut Memoria, a0: u32) {
    let prio = rw(m, a0.wrapping_add(OB_PRIORITY));
    let d0 = u32::from(prio >> 1) & (0x80 * (SPRITELAYER_NUM - 1));
    let camada = V_SPRITEQUEUE.wrapping_add(d0);
    let n = rw(m, camada);
    if u32::from(n) >= SPRITELAYER_SIZE - 2 {
        return;
    }
    let n = n.wrapping_add(2);
    ww(m, camada, n);
    ww(
        m,
        camada.wrapping_add(u32::from(n)),
        u16::try_from(a0 & 0xFFFF).unwrap_or(0),
    );
}

/// `ExecuteObjects`: roda cada objeto da memória de objetos.
pub fn execute_objects(m: &mut Memoria) -> Result<(), ErroAbertura> {
    // O Sonic morto (rotina 6 ou mais) é tratado na fase; fora dela o objeto 0 está vazio.
    if rb(m, V_PLAYER.wrapping_add(OB_ROUTINE)) >= 6 {
        return Err(ErroAbertura::NaoImplementado(
            "ExecuteObjects com o Sonic morto",
        ));
    }
    let mut a0 = V_OBJSPACE;
    while a0 < V_OBJSPACE_END {
        let id = rb(m, a0.wrapping_add(OB_ID));
        if id != 0 {
            rodar_objeto(m, a0, id)?;
        }
        a0 = a0.wrapping_add(OBJECT_SIZE);
    }
    Ok(())
}

fn rodar_objeto(m: &mut Memoria, a0: u32, id: u8) -> Result<(), ErroAbertura> {
    match id {
        ID_CREDITS_TEXT => {
            credits_text(m, a0);
            Ok(())
        }
        _ => Err(ErroAbertura::NaoImplementado("objeto")),
    }
}

/// Objeto `CreditsText` (`8A`): "SONIC TEAM PRESENTS" e os créditos.
fn credits_text(m: &mut Memoria, a0: u32) {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        // Cred_Main
        wb(m, a0.wrapping_add(OB_ROUTINE), 2);
        ww(m, a0.wrapping_add(OB_X), 160 + 0x80);
        ww(m, a0.wrapping_add(OB_SCREENY), 112 + 0x80);
        wl(m, a0.wrapping_add(OB_MAP), MAP_CRED);
        ww(m, a0.wrapping_add(OB_GFX), ARTTILE_CREDITS_FONT);
        let n = rw(m, V_CREDITSNUM);
        wb(m, a0.wrapping_add(OB_FRAME), n.to_be_bytes()[1]);
        wb(m, a0.wrapping_add(OB_RENDER), 0);
        wb(m, a0.wrapping_add(OB_PRIORITY), 0);
        if rb(m, V_GAMEMODE) == GM_TITLE_ID {
            ww(m, a0.wrapping_add(OB_GFX), ARTTILE_SONIC_TEAM_FONT);
            wb(m, a0.wrapping_add(OB_FRAME), 0xA);
            if rb(m, F_CREDITSCHEAT) != 0 && rb(m, V_JPADHOLD1) == 0x72 {
                ww(m, V_PALETTE_FADING_LINE_3, 0x0EEE);
                ww(m, V_PALETTE_FADING_LINE_3.wrapping_add(2), 0x0880);
                delete_object(m, a0);
                return;
            }
        }
    }
    // Cred_Display
    display_sprite(m, a0);
}

fn sext8(b: u8) -> i16 {
    i16::from(i8::from_be_bytes([b]))
}

/// `BuildSprites`: monta a tabela de sprites a partir das filas de prioridade.
pub fn build_sprites(m: &mut Memoria) -> Result<(), ErroAbertura> {
    let mut a2 = V_SPRITETABLEBUFFER;
    let mut d5: u8 = 0;
    let mut a4 = V_SPRITEQUEUE;
    for _ in 0..SPRITELAYER_NUM {
        if rw(m, a4) != 0 {
            let mut d6: u32 = 2;
            loop {
                let a0 = ext(rw(m, a4.wrapping_add(d6)));
                if rb(m, a0.wrapping_add(OB_ID)) != 0 {
                    desenhar_objeto(m, a0, &mut a2, &mut d5)?;
                }
                d6 = d6.wrapping_add(2);
                let n = rw(m, a4).wrapping_sub(2);
                ww(m, a4, n);
                if n == 0 {
                    break;
                }
            }
        }
        a4 = a4.wrapping_add(SPRITELAYER_SIZE);
    }
    wb(m, V_SPRITECOUNT, d5);
    if d5 == SPRITES_MAX {
        wb(m, a2.wrapping_sub(5), 0);
    } else {
        wl(m, a2, 0);
    }
    Ok(())
}

fn desenhar_objeto(
    m: &mut Memoria,
    a0: u32,
    a2: &mut u32,
    d5: &mut u8,
) -> Result<(), ErroAbertura> {
    // bclr #7: desmarca "renderizado".
    let render = rb(m, a0.wrapping_add(OB_RENDER)) & 0x7F;
    wb(m, a0.wrapping_add(OB_RENDER), render);
    let d4 = render;
    let cam = d4 & 0x0C;
    let (d2, d3): (i16, i16);
    if cam == 0 {
        d2 = i16::from_be_bytes(rw(m, a0.wrapping_add(OB_SCREENY)).to_be_bytes());
        d3 = i16::from_be_bytes(rw(m, a0.wrapping_add(OB_X)).to_be_bytes());
    } else {
        let a1 = match cam {
            0x04 => V_SCREENPOSX,
            0x08 => V_BGSCREENPOSX,
            _ => V_BG3SCREENPOSX,
        };
        let largura = i16::from(rb(m, a0.wrapping_add(OB_ACTWID)));
        let x = i16::from_be_bytes(rw(m, a0.wrapping_add(OB_X)).to_be_bytes());
        let cx = i16::from_be_bytes(rw(m, a1).to_be_bytes());
        let mut d3v = x.wrapping_sub(cx);
        if d3v.wrapping_add(largura) < 0 {
            return Ok(());
        }
        if d3v.wrapping_sub(largura) >= 320 {
            return Ok(());
        }
        d3v = d3v.wrapping_add(0x80);
        let y = i16::from_be_bytes(rw(m, a0.wrapping_add(OB_Y)).to_be_bytes());
        let cy = i16::from_be_bytes(rw(m, a1.wrapping_add(4)).to_be_bytes());
        let mut d2v = y.wrapping_sub(cy);
        if d4 & 0x10 != 0 {
            let h = i16::from(rb(m, a0.wrapping_add(OB_HEIGHT)));
            if d2v.wrapping_add(h) < 0 || d2v.wrapping_sub(h) >= 224 {
                return Ok(());
            }
            d2v = d2v.wrapping_add(0x80);
        } else {
            d2v = d2v.wrapping_add(0x80);
            // Altura suposta de 32 pontos.
            if !(0x80 - 32..0x80 + 224 + 32).contains(&d2v) {
                return Ok(());
            }
        }
        d2 = d2v;
        d3 = d3v;
    }
    let mut a1 = rl(m, a0.wrapping_add(OB_MAP));
    let mut d1: i32;
    if d4 & 0x20 != 0 {
        d1 = 0;
    } else {
        let f = rb(m, a0.wrapping_add(OB_FRAME)).wrapping_add(rb(m, a0.wrapping_add(OB_FRAME)));
        let off = rom_w(m, a1.wrapping_add(u32::from(f)));
        a1 = a1.wrapping_add(ext(off));
        let n = rom_byte(m, a1);
        a1 = a1.wrapping_add(1);
        d1 = i32::from(n).wrapping_sub(1);
        if d1 < 0 {
            let r = rb(m, a0.wrapping_add(OB_RENDER)) | 0x80;
            wb(m, a0.wrapping_add(OB_RENDER), r);
            return Ok(());
        }
    }
    let a3 = rw(m, a0.wrapping_add(OB_GFX));
    let xflip = d4 & 1 != 0;
    let yflip = d4 & 2 != 0;
    while d1 >= 0 {
        if *d5 == SPRITES_MAX {
            break;
        }
        let y = sext8(rom_byte(m, a1));
        let tam = rom_byte(m, a1.wrapping_add(1));
        let mut d0 = y;
        if yflip {
            let d4b = i16::from((tam << 3) & 0x18).wrapping_add(8);
            d0 = (0i16).wrapping_sub(d0).wrapping_sub(d4b);
        }
        d0 = d0.wrapping_add(d2);
        ww(m, *a2, u16::from_be_bytes(d0.to_be_bytes()));
        *a2 = a2.wrapping_add(2);
        wb(m, *a2, tam);
        *a2 = a2.wrapping_add(1);
        *d5 = d5.wrapping_add(1);
        wb(m, *a2, *d5);
        *a2 = a2.wrapping_add(1);
        let tile = u16::from_be_bytes([
            rom_byte(m, a1.wrapping_add(2)),
            rom_byte(m, a1.wrapping_add(3)),
        ]);
        let mut t = tile.wrapping_add(a3);
        if xflip || yflip {
            t ^= (u16::from(xflip) << 11) | (u16::from(yflip) << 12);
        }
        ww(m, *a2, t);
        *a2 = a2.wrapping_add(2);
        let mut x = sext8(rom_byte(m, a1.wrapping_add(4)));
        if xflip {
            let d4b = i16::from(tam.wrapping_add(tam) & 0x18).wrapping_add(8);
            x = (0i16).wrapping_sub(x).wrapping_sub(d4b);
        }
        x = x.wrapping_add(d3);
        let mut xw = u16::from_be_bytes(x.to_be_bytes()) & 0x1FF;
        if xw == 0 {
            xw = 1;
        }
        ww(m, *a2, xw);
        *a2 = a2.wrapping_add(2);
        a1 = a1.wrapping_add(5);
        d1 = d1.wrapping_sub(1);
    }
    let r = rb(m, a0.wrapping_add(OB_RENDER)) | 0x80;
    wb(m, a0.wrapping_add(OB_RENDER), r);
    Ok(())
}

fn rom_byte(m: &Memoria, a: u32) -> u8 {
    super::boot::rom_b(m, a)
}
