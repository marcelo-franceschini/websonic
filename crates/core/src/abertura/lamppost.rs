//! Lamppost (objeto 0x79) e dados de retorno ao ponto de controle.
use super::fisica::calc_sine;
use super::objetos::{find_free_obj, remember_state};
use super::ram::*;
use super::{ErroAbertura, rb, rl, rw, wb, wl, ww};
use crate::memoria::Memoria;

const MAP_LAMP: u32 = 0x178A4;
const LAMP_X: u32 = 0x30;
const LAMP_Y: u32 = 0x32;
const LAMP_TIMER: u32 = 0x36;
const LAMP_DATA: u32 = 0xFF_FE32;
const LIFECOUNT: u32 = 0xFF_FE1B;
const WATERPOS: u32 = 0xFF_F648;
const WATER_ROUTINE: u32 = 0xFF_F64D;
const WATER_STATE: u32 = 0xFF_F64E;

fn marca(m: &mut Memoria, a: u32) -> u32 {
    V_OBJSTATE
        .wrapping_add(2)
        .wrapping_add(u32::from(rb(m, a.wrapping_add(OB_RESPAWNNO))))
}

pub fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a.wrapping_add(OB_ROUTINE));
    match rotina {
        0 => {
            wb(m, a.wrapping_add(OB_ROUTINE), 2);
            wl(m, a.wrapping_add(OB_MAP), MAP_LAMP);
            ww(m, a.wrapping_add(OB_GFX), 0x7A0);
            wb(m, a.wrapping_add(OB_RENDER), 4);
            wb(m, a.wrapping_add(OB_ACTWID), 8);
            wb(m, a.wrapping_add(OB_PRIORITY), 5);
            let estado = marca(m, a);
            let anterior = rb(m, estado) & 0x7F;
            wb(m, estado, anterior);
            let ultimo = rb(m, V_LASTLAMP) & 0x7F;
            let numero = rb(m, a.wrapping_add(OB_SUBTYPE)) & 0x7F;
            if anterior & 1 != 0 || ultimo >= numero {
                wb(m, estado, anterior | 1);
                wb(m, a.wrapping_add(OB_ROUTINE), 4);
                wb(m, a.wrapping_add(OB_FRAME), 3);
            } else {
                azul(m, a);
            }
        }
        2 => azul(m, a),
        4 => {}
        6 => girar(m, a),
        _ => return Err(ErroAbertura::NaoImplementado("rotina do lamppost")),
    }
    remember_state(m, a);
    Ok(())
}

fn azul(m: &mut Memoria, a: u32) {
    if rw(m, V_DEBUGUSE) != 0 || rb(m, F_PLAYERCTRL) & 0x80 != 0 {
        return;
    }
    let ultimo = rb(m, V_LASTLAMP) & 0x7F;
    let numero = rb(m, a.wrapping_add(OB_SUBTYPE)) & 0x7F;
    if ultimo >= numero {
        let estado = marca(m, a);
        let valor = rb(m, estado) | 1;
        wb(m, estado, valor);
        wb(m, a.wrapping_add(OB_ROUTINE), 4);
        wb(m, a.wrapping_add(OB_FRAME), 3);
        return;
    }
    let dx = rw(m, V_PLAYER.wrapping_add(OB_X))
        .wrapping_sub(rw(m, a.wrapping_add(OB_X)))
        .wrapping_add(8);
    let dy = rw(m, V_PLAYER.wrapping_add(OB_Y))
        .wrapping_sub(rw(m, a.wrapping_add(OB_Y)))
        .wrapping_add(0x40);
    if dx >= 16 || dy >= 0x68 {
        return;
    }
    wb(m, V_SOUNDQUEUE1, 0xA1);
    wb(m, a.wrapping_add(OB_ROUTINE), 4);
    if let Some(filho) = find_free_obj(m) {
        wb(m, filho.wrapping_add(OB_ID), 0x79);
        wb(m, filho.wrapping_add(OB_ROUTINE), 6);
        let x = rw(m, a.wrapping_add(OB_X));
        let y = rw(m, a.wrapping_add(OB_Y)).wrapping_sub(0x18);
        ww(m, filho.wrapping_add(LAMP_X), x);
        ww(m, filho.wrapping_add(LAMP_Y), y);
        wl(m, filho.wrapping_add(OB_MAP), MAP_LAMP);
        ww(m, filho.wrapping_add(OB_GFX), 0x7A0);
        wb(m, filho.wrapping_add(OB_RENDER), 4);
        wb(m, filho.wrapping_add(OB_ACTWID), 8);
        wb(m, filho.wrapping_add(OB_PRIORITY), 4);
        wb(m, filho.wrapping_add(OB_FRAME), 2);
        ww(m, filho.wrapping_add(LAMP_TIMER), 32);
    }
    wb(m, a.wrapping_add(OB_FRAME), 1);
    guardar(m, a);
    let estado = marca(m, a);
    let valor = rb(m, estado) | 1;
    wb(m, estado, valor);
}

fn girar(m: &mut Memoria, a: u32) {
    let tempo = rw(m, a.wrapping_add(LAMP_TIMER)).wrapping_sub(1);
    ww(m, a.wrapping_add(LAMP_TIMER), tempo);
    if tempo & 0x8000 != 0 {
        wb(m, a.wrapping_add(OB_ROUTINE), 4);
    }
    let angulo = rb(m, a.wrapping_add(OB_ANGLE));
    wb(m, a.wrapping_add(OB_ANGLE), angulo.wrapping_sub(0x10));
    let (seno, cosseno) = calc_sine(m, angulo.wrapping_sub(0x40));
    let desloc = |valor: i16| -> u16 {
        let produto = i32::from(valor).wrapping_mul(0xC00) >> 16;
        let bytes = produto.to_be_bytes();
        u16::from_be_bytes([bytes[2], bytes[3]])
    };
    let x = rw(m, a.wrapping_add(LAMP_X)).wrapping_add(desloc(cosseno));
    let y = rw(m, a.wrapping_add(LAMP_Y)).wrapping_add(desloc(seno));
    ww(m, a.wrapping_add(OB_X), x);
    ww(m, a.wrapping_add(OB_Y), y);
}

fn guardar(m: &mut Memoria, a: u32) {
    let numero = rb(m, a.wrapping_add(OB_SUBTYPE));
    wb(m, V_LASTLAMP, numero);
    wb(m, V_LASTLAMP.wrapping_add(1), numero);
    for (origem, destino) in [
        (a.wrapping_add(OB_X), 0),
        (a.wrapping_add(OB_Y), 2),
        (V_RINGS, 4),
        (V_LIMITBTM2, 12),
        (V_SCREENPOSX, 14),
        (V_SCREENPOSY, 16),
        (V_BGSCREENPOSX, 18),
        (V_BGSCREENPOSY, 20),
        (V_BG2SCREENPOSX, 22),
        (V_BG2SCREENPOSY, 24),
        (V_BG3SCREENPOSX, 26),
        (V_BG3SCREENPOSY, 28),
        (WATERPOS, 30),
    ] {
        let valor = rw(m, origem);
        ww(m, LAMP_DATA.wrapping_add(destino), valor);
    }
    let tempo = rl(m, V_TIME);
    wl(m, LAMP_DATA.wrapping_add(6), tempo);
    for (origem, destino) in [
        (V_DLE_ROUTINE, 10),
        (WATER_ROUTINE, 32),
        (WATER_STATE, 33),
        (LIFECOUNT, 34),
    ] {
        let valor = rb(m, origem);
        wb(m, LAMP_DATA.wrapping_add(destino), valor);
    }
}

/// Restaura a posição e a câmera preservadas na ativação do poste.
pub fn carregar(m: &mut Memoria) {
    let numero = rb(m, V_LASTLAMP.wrapping_add(1));
    wb(m, V_LASTLAMP, numero);
    for (destino, origem) in [
        (V_PLAYER.wrapping_add(OB_X), 0),
        (V_PLAYER.wrapping_add(OB_Y), 2),
        (V_LIMITBTM1, 12),
        (V_LIMITBTM2, 12),
        (V_SCREENPOSX, 14),
        (V_SCREENPOSY, 16),
        (V_BGSCREENPOSX, 18),
        (V_BGSCREENPOSY, 20),
        (V_BG2SCREENPOSX, 22),
        (V_BG2SCREENPOSY, 24),
        (V_BG3SCREENPOSX, 26),
        (V_BG3SCREENPOSY, 28),
    ] {
        let valor = rw(m, LAMP_DATA.wrapping_add(origem));
        ww(m, destino, valor);
    }
    ww(m, V_RINGS, 0);
    wb(m, LIFECOUNT, 0);
    let tempo = rl(m, LAMP_DATA.wrapping_add(6));
    wl(m, V_TIME, tempo);
    wb(m, V_TIMECENT, 59);
    let segundos = rb(m, V_TIMESEC).wrapping_sub(1);
    wb(m, V_TIMESEC, segundos);
    let rotina = rb(m, LAMP_DATA.wrapping_add(10));
    wb(m, V_DLE_ROUTINE, rotina);
    let agua = rb(m, LAMP_DATA.wrapping_add(32));
    wb(m, WATER_ROUTINE, agua);
    if rb(m, V_ZONE) == 1 {
        let pos = rw(m, LAMP_DATA.wrapping_add(30));
        ww(m, WATERPOS, pos);
        let estado = rb(m, LAMP_DATA.wrapping_add(33));
        wb(m, WATER_STATE, estado);
    }
    if numero & 0x80 != 0 {
        let x = rw(m, V_PLAYER.wrapping_add(OB_X)).wrapping_sub(160);
        ww(m, V_LIMITLEFT2, x);
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn ativacao_guarda_estado_e_retorno_restaura_camera() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let poste = V_LVLOBJSPACE;
        wb(&mut m, poste.wrapping_add(OB_ID), 0x79);
        wb(&mut m, poste.wrapping_add(OB_SUBTYPE), 1);
        ww(&mut m, poste.wrapping_add(OB_X), 0x180);
        ww(&mut m, poste.wrapping_add(OB_Y), 0x200);
        ww(&mut m, V_PLAYER.wrapping_add(OB_X), 0x180);
        ww(&mut m, V_PLAYER.wrapping_add(OB_Y), 0x200);
        ww(&mut m, V_SCREENPOSX, 0x100);
        ww(&mut m, V_LIMITBTM2, 0x300);
        wl(&mut m, V_TIME, 0x00010203);
        executar(&mut m, poste).unwrap();
        assert_eq!(rb(&mut m, V_LASTLAMP), 1);
        assert_eq!(rb(&mut m, poste.wrapping_add(OB_FRAME)), 1);
        assert_eq!(
            rb(
                &mut m,
                poste.wrapping_add(OBJECT_SIZE).wrapping_add(OB_ROUTINE)
            ),
            6
        );
        ww(&mut m, V_SCREENPOSX, 0);
        ww(&mut m, V_RINGS, 42);
        carregar(&mut m);
        assert_eq!(rw(&mut m, V_SCREENPOSX), 0x100);
        assert_eq!(rw(&mut m, V_LIMITBTM1), 0x300);
        assert_eq!(rw(&mut m, V_RINGS), 0);
        assert_eq!(rb(&mut m, V_TIMESEC), 1);
        assert_eq!(rb(&mut m, V_TIMECENT), 59);
    }
}
