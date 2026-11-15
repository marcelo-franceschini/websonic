//! Fundo e eventos de Star Light Zone, REV01.
use super::ram::*;
use super::{ErroAbertura, rb, rl, rw, wb, wl, ww};
use crate::memoria::Memoria;

const BG1_YBLOCK: u32 = 0xFF_F74D;

/// `BgScroll_SLZ`: metade da altura inicial da câmera, com deslocamento fixo.
pub fn iniciar_fundo(m: &mut Memoria, camera_y: u16) {
    ww(m, V_BGSCREENPOSY, (camera_y >> 1).wrapping_add(0xC0));
    wl(m, V_BGSCREENPOSX, 0);
}

/// `DLE_SLZ`: prepara a arena e o chefe do terceiro ato.
pub fn eventos(m: &mut Memoria) -> Result<(), ErroAbertura> {
    let x = rw(m, V_SCREENPOSX);
    match rb(m, V_ACT) {
        0 | 1 => {}
        2 => match rb(m, V_DLE_ROUTINE) {
            0 if x >= 0x1E70 => {
                ww(m, V_LIMITBTM1, 0x210);
                wb(m, V_DLE_ROUTINE, 2);
            }
            2 if x >= 0x2000 => {
                if let Some(slot) = super::objetos::find_free_obj(m) {
                    wb(m, slot.wrapping_add(OB_ID), 0x7A);
                }
                wb(m, V_SOUNDQUEUE0, 0x8C);
                wb(m, F_LOCKSCREEN, 1);
                wb(m, V_DLE_ROUTINE, 4);
                super::plc::add_plc(m, 0x11)?;
            }
            4 => ww(m, V_LIMITLEFT2, x),
            0 | 2 => {}
            _ => return Err(ErroAbertura::NaoImplementado("rotina de eventos da SLZ3")),
        },
        _ => return Err(ErroAbertura::NaoImplementado("ato da SLZ")),
    }
    Ok(())
}

fn faixa(m: &mut Memoria, cursor: &mut u32, valor: u16, quantidade: u32) {
    for _ in 0..quantidade {
        ww(m, *cursor, valor);
        *cursor = cursor.wrapping_add(2);
    }
}

/// `Deform_SLZ`: fundo vertical e faixas de estrelas e prédios.
pub fn deformar(m: &mut Memoria) {
    let delta = i32::from(i16::from_be_bytes(rw(m, V_SCRSHIFTY).to_be_bytes())).wrapping_shl(7);
    let anterior = rl(m, V_BGSCREENPOSY);
    let novo = anterior.wrapping_add(u32::from_be_bytes(delta.to_be_bytes()));
    wl(m, V_BGSCREENPOSY, novo);
    let bloco = rb(m, BG1_YBLOCK);
    if u8::try_from((novo >> 16) & 0x10).unwrap_or(0) == bloco {
        wb(m, BG1_YBLOCK, bloco ^ 0x10);
        let flags = rb(m, V_BG1_SCROLL_FLAGS) | if delta < 0 { 0x10 } else { 0x20 };
        wb(m, V_BG1_SCROLL_FLAGS, flags);
    }
    let y = rw(m, V_BGSCREENPOSY);
    ww(m, V_BGSCRPOSY_VDP, y);
    let frente = 0u16.wrapping_sub(rw(m, V_SCREENPOSX));
    let frente_s = i16::from_be_bytes(frente.to_be_bytes());
    let oitavo = frente_s >> 3;
    let diferenca = oitavo.wrapping_sub(frente_s);
    let passo = (i32::from(diferenca).wrapping_mul(16) / 28).wrapping_shl(12);
    let mut acumulador = u32::from(frente);
    let mut cursor = V_BG_SCROLL_BUFFER;
    for _ in 0..28 {
        ww(m, cursor, u16::try_from(acumulador & 0xFFFF).unwrap_or(0));
        cursor = cursor.wrapping_add(2);
        acumulador = acumulador
            .rotate_left(16)
            .wrapping_add(u32::from_be_bytes(passo.to_be_bytes()))
            .rotate_left(16);
    }
    let distante = oitavo.wrapping_add(oitavo >> 1);
    faixa(
        m,
        &mut cursor,
        u16::from_be_bytes(distante.to_be_bytes()),
        5,
    );
    faixa(
        m,
        &mut cursor,
        u16::from_be_bytes((frente_s >> 2).to_be_bytes()),
        5,
    );
    faixa(
        m,
        &mut cursor,
        u16::from_be_bytes((frente_s >> 1).to_be_bytes()),
        30,
    );
    let mut fonte = V_BG_SCROLL_BUFFER.wrapping_add(u32::from((y.wrapping_sub(0xC0) & 0x3F0) >> 3));
    let mut destino = V_HSCROLLTABLEBUFFER;
    let mut linhas = 16u32.wrapping_sub(u32::from(y & 15));
    for _ in 0..15 {
        let fundo = rw(m, fonte);
        fonte = fonte.wrapping_add(2);
        for _ in 0..linhas {
            wl(m, destino, (u32::from(frente) << 16) | u32::from(fundo));
            destino = destino.wrapping_add(4);
        }
        linhas = 16;
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn arena_avanca_e_trava_camera() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        wb(&mut m, V_ACT, 2);
        ww(&mut m, V_SCREENPOSX, 0x1E6F);
        eventos(&mut m).unwrap();
        assert_eq!(rb(&mut m, V_DLE_ROUTINE), 0);
        ww(&mut m, V_SCREENPOSX, 0x1E70);
        eventos(&mut m).unwrap();
        assert_eq!(rw(&mut m, V_LIMITBTM1), 0x210);
        assert_eq!(rb(&mut m, V_DLE_ROUTINE), 2);
        ww(&mut m, V_SCREENPOSX, 0x2000);
        eventos(&mut m).unwrap();
        assert_eq!(rb(&mut m, V_LVLOBJSPACE), 0x7A);
        assert_eq!(rb(&mut m, F_LOCKSCREEN), 1);
        eventos(&mut m).unwrap();
        assert_eq!(rw(&mut m, V_LIMITLEFT2), 0x2000);
    }

    #[test]
    fn fundo_monta_estrelas_e_faixas_de_predios() {
        let mut m = Memoria::nova(vec![]);
        iniciar_fundo(&mut m, 0x100);
        assert_eq!(rw(&mut m, V_BGSCREENPOSY), 0x140);
        ww(&mut m, V_BGSCREENPOSY, 0xC0);
        ww(&mut m, V_SCREENPOSX, 0x80);
        deformar(&mut m);
        assert_eq!(rw(&mut m, V_BG_SCROLL_BUFFER), 0xFF80);
        assert_eq!(rw(&mut m, V_BG_SCROLL_BUFFER.wrapping_add(56)), 0xFFE8);
        assert_eq!(rw(&mut m, V_BG_SCROLL_BUFFER.wrapping_add(66)), 0xFFE0);
        assert_eq!(rw(&mut m, V_BG_SCROLL_BUFFER.wrapping_add(76)), 0xFFC0);
        assert_eq!(rl(&mut m, V_HSCROLLTABLEBUFFER), 0xFF80_FF80);
    }
}
