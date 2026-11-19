//! Luz giratoria de Spring Yard, objeto 0x12 (REV01).
use super::super::ram::*;
use super::super::{ErroAbertura, rb, rw, wb, wl, ww};
use super::{delete_object, display_sprite, fora_da_janela};
use crate::memoria::Memoria;

const MAP_LIGHT: u32 = 0xF00E;

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let mut rotina = rb(m, a.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        rotina = 2;
        wb(m, a.wrapping_add(OB_ROUTINE), rotina);
        wl(m, a.wrapping_add(OB_MAP), MAP_LIGHT);
        ww(m, a.wrapping_add(OB_GFX), 0);
        wb(m, a.wrapping_add(OB_RENDER), 4);
        wb(m, a.wrapping_add(OB_ACTWID), 16);
        wb(m, a.wrapping_add(OB_PRIORITY), 6);
    }
    if rotina == 2 {
        let tempo = (rb(m, a.wrapping_add(OB_TIMEFRAME)) as i8).wrapping_sub(1);
        wb(m, a.wrapping_add(OB_TIMEFRAME), tempo as u8);
        if tempo < 0 {
            wb(m, a.wrapping_add(OB_TIMEFRAME), 7);
            let mut frame = rb(m, a.wrapping_add(OB_FRAME)).wrapping_add(1);
            if frame >= 6 {
                frame = 0;
            }
            wb(m, a.wrapping_add(OB_FRAME), frame);
        }
        let x = rw(m, a.wrapping_add(OB_X));
        if fora_da_janela(m, x) {
            delete_object(m, a);
        } else {
            display_sprite(m, a);
        }
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn inicializacao_spinning_light() {
        let mut m = Memoria::nova(vec![0; 0x1000]);
        let a0: u32 = 0xFF_D800;
        wb(&mut m, a0.wrapping_add(OB_ID), 0x12);
        wb(&mut m, a0.wrapping_add(OB_ROUTINE), 0);
        ww(&mut m, a0.wrapping_add(OB_X), 0x100);
        ww(&mut m, a0.wrapping_add(OB_Y), 0x100);

        let res = executar(&mut m, a0);
        assert!(res.is_ok());
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ACTWID)), 16);
    }
}
