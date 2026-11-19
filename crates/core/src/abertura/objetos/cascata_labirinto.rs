//! Cachoeiras decorativas da Labyrinth, objeto 0x65 (REV01).
use super::super::ram::*;
use super::super::{ErroAbertura, rb, rw, wb, wl, ww};
use super::{animate_sprite, remember_state};
use crate::memoria::Memoria;

const ANI: u32 = 0x13220;
const AGUA: u32 = 0xFF_F646;
const CHUNK_ESCORREGADOR: u32 = V_LVLLAYOUT_FG.wrapping_add(0x106);

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let mut rotina = rb(m, a.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        rotina = 4;
        wl(m, a.wrapping_add(OB_MAP), 0x13228);
        let subtipo = rb(m, a.wrapping_add(OB_SUBTYPE));
        ww(
            m,
            a.wrapping_add(OB_GFX),
            if subtipo & 0x80 != 0 { 0xc259 } else { 0x4259 },
        );
        let render = rb(m, a.wrapping_add(OB_RENDER)) | 4;
        wb(m, a.wrapping_add(OB_RENDER), render);
        wb(m, a.wrapping_add(OB_ACTWID), 24);
        wb(m, a.wrapping_add(OB_PRIORITY), 1);
        let quadro = subtipo & 15;
        wb(m, a.wrapping_add(OB_FRAME), quadro);
        if quadro == 9 {
            wb(m, a.wrapping_add(OB_PRIORITY), 0);
            rotina = if subtipo & 0x20 != 0 {
                8
            } else if subtipo & 0x40 != 0 {
                6
            } else {
                2
            };
        }
        wb(m, a.wrapping_add(OB_ROUTINE), rotina);
        // A inicialização cai diretamente na animação para os subtipos de respingo.
        // Os ajustes de água e prioridade só ocorrem na execução seguinte.
        if quadro == 9 {
            animate_sprite(m, a, ANI)?;
            remember_state(m, a);
            return Ok(());
        }
    }
    match rotina {
        2 => animate_sprite(m, a, ANI)?,
        4 => {}
        6 => {
            let y = rw(m, AGUA).wrapping_sub(16);
            ww(m, a.wrapping_add(OB_Y), y);
            animate_sprite(m, a, ANI)?;
        }
        8 => {
            let prioridade = rb(m, CHUNK_ESCORREGADOR) == 7;
            let gfx = rw(m, a.wrapping_add(OB_GFX)) & 0x7fff;
            ww(
                m,
                a.wrapping_add(OB_GFX),
                if prioridade { gfx | 0x8000 } else { gfx },
            );
            animate_sprite(m, a, ANI)?;
        }
        _ => {
            return Err(ErroAbertura::NaoImplementado(
                "rotina da cascata de Labyrinth",
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
        rom[0x13220..0x13222].copy_from_slice(&2u16.to_be_bytes());
        rom[0x13222..0x13225].copy_from_slice(&[5, 9, 0xff]);
        Memoria::nova(rom)
    }
    #[test]
    fn cascata_estatica_define_quadro_e_prioridade_de_tile() {
        let mut m = memoria();
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x65);
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 0x87);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE)), 4);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_FRAME)), 7);
        assert_eq!(rw(&mut m, a.wrapping_add(OB_GFX)), 0xc259);
    }
    #[test]
    fn respingo_alinha_na_superficie_a_partir_do_segundo_quadro() {
        let mut m = memoria();
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x65);
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 0x49);
        ww(&mut m, a.wrapping_add(OB_Y), 0x300);
        ww(&mut m, AGUA, 0x200);
        executar(&mut m, a).unwrap();
        assert_eq!(rw(&mut m, a.wrapping_add(OB_Y)), 0x300);
        executar(&mut m, a).unwrap();
        assert_eq!(rw(&mut m, a.wrapping_add(OB_Y)), 0x1f0);
    }
    #[test]
    fn respingo_oculto_acompanha_mudanca_do_layout() {
        let mut m = memoria();
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x65);
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 0xa9);
        executar(&mut m, a).unwrap();
        executar(&mut m, a).unwrap();
        assert_eq!(rw(&mut m, a.wrapping_add(OB_GFX)) & 0x8000, 0);
        wb(&mut m, CHUNK_ESCORREGADOR, 7);
        executar(&mut m, a).unwrap();
        assert_eq!(rw(&mut m, a.wrapping_add(OB_GFX)) & 0x8000, 0x8000);
    }
}
