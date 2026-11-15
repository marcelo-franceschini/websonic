//! Cartões GAME OVER e TIME OVER, objeto 0x39 (REV01).
use super::ram::*;
use super::{ErroAbertura, objetos, rb, rl, rw, wb, wl, ww};
use crate::memoria::Memoria;

const MAP_OVER: u32 = 0xD232;
const V_LAMP_TIME: u32 = 0xFF_FE38;

pub fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            if rl(m, V_PLC_BUFFER) != 0 {
                return Ok(());
            }
            wb(m, a.wrapping_add(OB_ROUTINE), 2);
            let frame = rb(m, a.wrapping_add(OB_FRAME));
            ww(
                m,
                a.wrapping_add(OB_X),
                if frame & 1 == 0 { 80 } else { 496 },
            );
            ww(m, a.wrapping_add(OB_SCREENY), 240);
            wl(m, a.wrapping_add(OB_MAP), MAP_OVER);
            ww(m, a.wrapping_add(OB_GFX), 0x855E);
            wb(m, a.wrapping_add(OB_RENDER), 0);
            wb(m, a.wrapping_add(OB_PRIORITY), 0);
            mover(m, a);
        }
        2 => mover(m, a),
        4 => esperar(m, a),
        _ => return Err(ErroAbertura::NaoImplementado("rotina do cartão game over")),
    }
    Ok(())
}

fn mover(m: &mut Memoria, a: u32) {
    let x = rw(m, a.wrapping_add(OB_X));
    if x == 288 {
        ww(m, a.wrapping_add(OB_TIME_FRAME), 720);
        wb(m, a.wrapping_add(OB_ROUTINE), 4);
        return;
    }
    let desloc = if x < 288 { 16 } else { 0u16.wrapping_sub(16) };
    ww(m, a.wrapping_add(OB_X), x.wrapping_add(desloc));
    objetos::display_sprite(m, a);
}

fn esperar(m: &mut Memoria, a: u32) {
    let pressionado = rb(m, V_JPADPRESS1) & 0x70 != 0;
    let frame = rb(m, a.wrapping_add(OB_FRAME));
    let tempo = rw(m, a.wrapping_add(OB_TIME_FRAME));
    if pressionado || (frame & 1 == 0 && tempo == 0) {
        if rb(m, F_TIMEOVER) != 0 {
            wl(m, V_LAMP_TIME, 0);
            ww(m, F_RESTART, 1);
        } else {
            let modo = if rb(m, V_CONTINUES) == 0 { 0 } else { 0x14 };
            wb(m, V_GAMEMODE, modo);
        }
    } else if frame & 1 == 0 {
        ww(m, a.wrapping_add(OB_TIME_FRAME), tempo.wrapping_sub(1));
    }
    objetos::display_sprite(m, a);
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn aguarda_arte_e_converge_ao_centro() {
        let mut m = Memoria::nova(vec![0; 0x10000]);
        let a = V_TITLECARD;
        wl(&mut m, V_PLC_BUFFER, 1);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE)), 0);
        wl(&mut m, V_PLC_BUFFER, 0);
        for _ in 0..14 {
            executar(&mut m, a).unwrap();
        }
        assert_eq!(rw(&mut m, a.wrapping_add(OB_X)), 288);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE)), 4);
        assert_eq!(rw(&mut m, a.wrapping_add(OB_TIME_FRAME)), 720);
    }

    #[test]
    fn escolhe_continue_sega_ou_reinicio_por_tempo() {
        let mut m = Memoria::nova(vec![0; 0x10000]);
        let a = V_TITLECARD;
        wb(&mut m, a.wrapping_add(OB_ROUTINE), 4);
        wb(&mut m, V_JPADPRESS1, 0x10);
        wb(&mut m, V_CONTINUES, 1);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, V_GAMEMODE), 0x14);
        wb(&mut m, V_CONTINUES, 0);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, V_GAMEMODE), 0);
        wb(&mut m, F_TIMEOVER, 1);
        wl(&mut m, V_LAMP_TIME, 0x1234);
        executar(&mut m, a).unwrap();
        assert_eq!(rw(&mut m, F_RESTART), 1);
        assert_eq!(rl(&mut m, V_LAMP_TIME), 0);
    }
}
