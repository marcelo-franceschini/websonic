//! Plataforma circular de Star Light, objeto 0x5A (REV01).
use super::super::plataforma_oscilante::{detectar_altura, mover_sonic_altura, sair};
use super::super::ram::*;
use super::super::{ErroAbertura, rb, rw, wb, wl, ww};
use super::{delete_object, display_sprite, fora_da_janela};
use crate::memoria::Memoria;

const MAP_CIRC: u32 = 0x11556;
const ORIG_Y: u32 = 0x30;
const ORIG_X: u32 = 0x32;

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let mut rotina = rb(m, a.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        rotina = 2;
        wb(m, a.wrapping_add(OB_ROUTINE), rotina);
        wl(m, a.wrapping_add(OB_MAP), MAP_CIRC);
        ww(m, a.wrapping_add(OB_GFX), 0x4000);
        wb(m, a.wrapping_add(OB_RENDER), 4);
        wb(m, a.wrapping_add(OB_PRIORITY), 4);
        wb(m, a.wrapping_add(OB_ACTWID), 24);
        let ox = rw(m, a.wrapping_add(OB_X));
        let oy = rw(m, a.wrapping_add(OB_Y));
        ww(m, a.wrapping_add(ORIG_X), ox);
        ww(m, a.wrapping_add(ORIG_Y), oy);
    }
    match rotina {
        2 => {
            mover(m, a);
            detectar_altura(m, a, 8);
        }
        4 => {
            sair(m, a);
            let anterior = rw(m, a.wrapping_add(OB_X));
            mover(m, a);
            mover_sonic_altura(m, a, anterior, 9);
        }
        _ => {
            return Err(ErroAbertura::NaoImplementado(
                "rotina da plataforma circular",
            ));
        }
    }
    let ox = rw(m, a.wrapping_add(ORIG_X));
    if fora_da_janela(m, ox) {
        delete_object(m, a);
    } else {
        display_sprite(m, a);
    }
    Ok(())
}

fn mover(m: &mut Memoria, a: u32) {
    let mut d1 = i16::from(rb(m, V_OSCILLATE.wrapping_add(0x22))).wrapping_sub(0x50);
    let mut d2 = i16::from(rb(m, V_OSCILLATE.wrapping_add(0x26))).wrapping_sub(0x50);
    let sub = rb(m, a.wrapping_add(OB_SUBTYPE));
    if sub & 1 != 0 {
        d1 = d1.wrapping_neg();
        d2 = d2.wrapping_neg();
    }
    if sub & 2 != 0 {
        d1 = d1.wrapping_neg();
        std::mem::swap(&mut d1, &mut d2);
    }
    if sub & 4 != 0 {
        d1 = d1.wrapping_neg();
    }
    let ox = rw(m, a.wrapping_add(ORIG_X));
    let oy = rw(m, a.wrapping_add(ORIG_Y));
    ww(m, a.wrapping_add(OB_X), ox.wrapping_add(d1.cast_unsigned()));
    ww(m, a.wrapping_add(OB_Y), oy.wrapping_add(d2.cast_unsigned()));
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn inicializacao_circling_platform() {
        let mut m = Memoria::nova(vec![0; 0x1000]);
        let a0: u32 = 0xFF_D800;
        wb(&mut m, a0.wrapping_add(OB_ID), 0x5A);
        wb(&mut m, a0.wrapping_add(OB_ROUTINE), 0);
        ww(&mut m, a0.wrapping_add(OB_X), 0x100);
        ww(&mut m, a0.wrapping_add(OB_Y), 0x100);

        let res = executar(&mut m, a0);
        assert!(res.is_ok());
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ACTWID)), 24);
        assert_eq!(rw(&mut m, a0.wrapping_add(ORIG_X)), 0x100);
        assert_eq!(rw(&mut m, a0.wrapping_add(ORIG_Y)), 0x100);
    }
}
