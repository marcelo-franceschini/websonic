//! Esfera gigante de espinhos de Spring Yard, objeto 0x58 (REV01).
use super::super::fisica::calc_sine;
use super::super::ram::*;
use super::super::{rb, rw, wb, wl, ww, ErroAbertura};
use super::{delete_object, display_sprite, fora_da_janela};
use crate::memoria::Memoria;

const MAP_BBALL: u32 = 0x11174;
const ORIG_Y: u32 = 0x38;
const ORIG_X: u32 = 0x3A;
const RAIO: u32 = 0x3C;
const VELOCIDADE: u32 = 0x3E;

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let mut rotina = rb(m, a.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        rotina = 2;
        wb(m, a.wrapping_add(OB_ROUTINE), rotina);
        wl(m, a.wrapping_add(OB_MAP), MAP_BBALL);
        ww(m, a.wrapping_add(OB_GFX), 0x0396);
        wb(m, a.wrapping_add(OB_RENDER), 4);
        wb(m, a.wrapping_add(OB_PRIORITY), 4);
        wb(m, a.wrapping_add(OB_ACTWID), 24);

        let x = rw(m, a.wrapping_add(OB_X));
        let y = rw(m, a.wrapping_add(OB_Y));
        ww(m, a.wrapping_add(ORIG_X), x);
        ww(m, a.wrapping_add(ORIG_Y), y);
        wb(m, a.wrapping_add(OB_COLTYPE), 0x86);

        let sub = rb(m, a.wrapping_add(OB_SUBTYPE));
        let speed = (i16::from((sub & 0xF0) as i8) << 3) as u16;
        ww(m, a.wrapping_add(VELOCIDADE), speed);

        let status = rb(m, a.wrapping_add(OB_STATUS));
        let angulo = ((status & 1) << 6) | ((status & 2) << 6);
        wb(m, a.wrapping_add(OB_ANGLE), angulo);
        wb(m, a.wrapping_add(RAIO), 0x50);
    }
    if rotina == 2 {
        let sub = rb(m, a.wrapping_add(OB_SUBTYPE)) & 7;
        match sub {
            0 => {}
            1 => {
                let xflip = rb(m, a.wrapping_add(OB_STATUS)) & 1 != 0;
                let mut d0 = u16::from(rb(m, V_OSCILLATE.wrapping_add(0x0e)));
                if xflip {
                    d0 = (0x60u16).wrapping_sub(d0);
                }
                let orig_x = rw(m, a.wrapping_add(ORIG_X));
                ww(m, a.wrapping_add(OB_X), orig_x.wrapping_sub(d0));
            }
            2 => {
                let xflip = rb(m, a.wrapping_add(OB_STATUS)) & 1 != 0;
                let mut d0 = u16::from(rb(m, V_OSCILLATE.wrapping_add(0x0e)));
                if xflip {
                    d0 = (0x80u16).wrapping_sub(d0);
                }
                let orig_y = rw(m, a.wrapping_add(ORIG_Y));
                ww(m, a.wrapping_add(OB_Y), orig_y.wrapping_sub(d0));
            }
            3 => {
                let speed = rw(m, a.wrapping_add(VELOCIDADE));
                let ang_w = rw(m, a.wrapping_add(OB_ANGLE)).wrapping_add(speed);
                ww(m, a.wrapping_add(OB_ANGLE), ang_w);
                let angulo = (ang_w >> 8) as u8;
                let (seno, cosseno) = calc_sine(m, angulo);
                let raio = i32::from(rb(m, a.wrapping_add(RAIO)));
                let dy = raio.wrapping_mul(i32::from(seno)) >> 8;
                let dx = raio.wrapping_mul(i32::from(cosseno)) >> 8;
                let orig_y = rw(m, a.wrapping_add(ORIG_Y));
                let orig_x = rw(m, a.wrapping_add(ORIG_X));
                ww(m, a.wrapping_add(OB_Y), orig_y.wrapping_add(dy as u16));
                ww(m, a.wrapping_add(OB_X), orig_x.wrapping_add(dx as u16));
            }
            _ => {}
        }
        let orig_x = rw(m, a.wrapping_add(ORIG_X));
        if fora_da_janela(m, orig_x) {
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
    fn inicializacao_big_spike_ball() {
        let mut m = Memoria::nova(vec![0; 0x1000]);
        let a0: u32 = 0xFF_D800;
        wb(&mut m, a0.wrapping_add(OB_ID), 0x58);
        wb(&mut m, a0.wrapping_add(OB_ROUTINE), 0);
        wb(&mut m, a0.wrapping_add(OB_SUBTYPE), 0);
        ww(&mut m, a0.wrapping_add(OB_X), 0x100);
        ww(&mut m, a0.wrapping_add(OB_Y), 0x100);

        let res = executar(&mut m, a0);
        assert!(res.is_ok());
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ACTWID)), 24);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_COLTYPE)), 0x86);
        assert_eq!(rw(&mut m, a0.wrapping_add(ORIG_X)), 0x100);
        assert_eq!(rw(&mut m, a0.wrapping_add(ORIG_Y)), 0x100);
    }
}

