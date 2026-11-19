//! Arpões de Labyrinth, objeto 0x16 (REV01).
use super::super::boot::rom_b;
use super::super::ram::*;
use super::super::{ErroAbertura, rb, rw, wb, wl, ww};
use super::{animate_sprite, remember_state};
use crate::memoria::Memoria;

const TEMPO: u32 = 0x30;
const ANIMACAO: u32 = 0x12656;
const COLISOES: u32 = 0x12636;

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let mut rotina = rb(m, a.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        rotina = 2;
        wb(m, a.wrapping_add(OB_ROUTINE), rotina);
        wl(m, a.wrapping_add(OB_MAP), 0x1266e);
        ww(m, a.wrapping_add(OB_GFX), 0x03cc);
        let render = rb(m, a.wrapping_add(OB_RENDER)) | 4;
        wb(m, a.wrapping_add(OB_RENDER), render);
        wb(m, a.wrapping_add(OB_PRIORITY), 4);
        let subtipo = rb(m, a.wrapping_add(OB_SUBTYPE));
        wb(m, a.wrapping_add(OB_ANIM), subtipo);
        wb(m, a.wrapping_add(OB_ACTWID), 20);
        ww(m, a.wrapping_add(TEMPO), 60);
    }
    match rotina {
        2 => {
            animate_sprite(m, a, ANIMACAO)?;
            let quadro = rb(m, a.wrapping_add(OB_FRAME));
            let colisao = rom_b(m, COLISOES.wrapping_add(u32::from(quadro)));
            wb(m, a.wrapping_add(OB_COLTYPE), colisao);
        }
        4 => {
            let tempo = rw(m, a.wrapping_add(TEMPO)).wrapping_sub(1);
            ww(m, a.wrapping_add(TEMPO), tempo);
            if i16::from_be_bytes(tempo.to_be_bytes()) < 0 {
                ww(m, a.wrapping_add(TEMPO), 60);
                wb(m, a.wrapping_add(OB_ROUTINE), 2);
                let anim = rb(m, a.wrapping_add(OB_ANIM)) ^ 1;
                wb(m, a.wrapping_add(OB_ANIM), anim);
            }
        }
        _ => {
            return Err(ErroAbertura::NaoImplementado(
                "rotina do arpao de Labyrinth",
            ));
        }
    }
    remember_state(m, a);
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    fn memoria() -> Memoria {
        let mut rom = vec![0; 0x20000];
        rom[0x1265a..0x1265c].copy_from_slice(&8u16.to_be_bytes());
        rom[0x1265e..0x12661].copy_from_slice(&[5, 4, 0xff]);
        rom[0x1263a] = 0x9f;
        Memoria::nova(rom)
    }
    #[test]
    fn arpao_vertical_inicializa_e_usa_colisao_do_quadro() {
        let mut m = memoria();
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x16);
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 2);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_FRAME)), 4);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_COLTYPE)), 0x9f);
        assert_eq!(rw(&mut m, a.wrapping_add(TEMPO)), 60);
    }
    #[test]
    fn espera_alterna_entre_avanco_e_recuo() {
        let mut m = memoria();
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x16);
        wb(&mut m, a.wrapping_add(OB_ROUTINE), 4);
        wb(&mut m, a.wrapping_add(OB_ANIM), 2);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ANIM)), 3);
        assert_eq!(rw(&mut m, a.wrapping_add(TEMPO)), 60);
    }
}
