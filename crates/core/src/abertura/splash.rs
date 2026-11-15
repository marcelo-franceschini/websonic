//! Objeto 0x08, respingo da superfície em Labyrinth Zone.
use super::objetos::{animate_sprite, delete_object, display_sprite};
use super::ram::*;
use super::{ErroAbertura, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;

const WATER1: u32 = 0xFF_F646;
const MAP_SPLASH: u32 = 0x14D34;
const ANI_SPLASH: u32 = 0x14D2C;

/// Anima o respingo e o remove ao fim da sequência da ROM.
pub fn executar(m: &mut Memoria, objeto: u32) -> Result<(), ErroAbertura> {
    match rb(m, objeto.wrapping_add(OB_ROUTINE)) {
        0 => {
            wb(m, objeto.wrapping_add(OB_ROUTINE), 2);
            wl(m, objeto.wrapping_add(OB_MAP), MAP_SPLASH);
            let render = rb(m, objeto.wrapping_add(OB_RENDER)) | 4;
            wb(m, objeto.wrapping_add(OB_RENDER), render);
            wb(m, objeto.wrapping_add(OB_PRIORITY), 1);
            wb(m, objeto.wrapping_add(OB_ACTWID), 16);
            ww(m, objeto.wrapping_add(OB_GFX), 0x4259);
            let x = rw(m, V_PLAYER.wrapping_add(OB_X));
            ww(m, objeto.wrapping_add(OB_X), x);
        }
        2 => {}
        4 => {
            delete_object(m, objeto);
            return Ok(());
        }
        _ => return Err(ErroAbertura::NaoImplementado("rotina do respingo")),
    }
    let y = rw(m, WATER1);
    ww(m, objeto.wrapping_add(OB_Y), y);
    animate_sprite(m, objeto, ANI_SPLASH)?;
    display_sprite(m, objeto);
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn inicializa_e_acompanha_altura_da_agua() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 8);
        ww(&mut m, V_PLAYER.wrapping_add(OB_X), 300);
        ww(&mut m, WATER1, 200);
        executar(&mut m, a).unwrap();
        assert_eq!(rw(&mut m, a.wrapping_add(OB_X)), 300);
        assert_eq!(rw(&mut m, a.wrapping_add(OB_Y)), 200);
        assert_eq!(rw(&mut m, a.wrapping_add(OB_GFX)), 0x4259);
        wb(&mut m, a.wrapping_add(OB_ROUTINE), 4);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ID)), 0);
    }
}
