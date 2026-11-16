//! Jaws, inimigo aquático da Labyrinth, objeto 0x2C (REV01).
use super::super::fisica::speed_to_pos;
use super::super::ram::*;
use super::super::{ErroAbertura, rb, rw, wb, wl, ww};
use super::{animate_sprite, remember_state};
use crate::memoria::Memoria;

const TEMPO: u32 = 0x30;
const TEMPO_BASE: u32 = 0x32;
const ANI_JAWS: u32 = 0xB2F2;

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            wb(m, a.wrapping_add(OB_ROUTINE), 2);
            wl(m, a.wrapping_add(OB_MAP), 0xB2FA);
            ww(m, a.wrapping_add(OB_GFX), 0x2486);
            let render = rb(m, a.wrapping_add(OB_RENDER)) | 4;
            wb(m, a.wrapping_add(OB_RENDER), render);
            wb(m, a.wrapping_add(OB_COLTYPE), 10);
            wb(m, a.wrapping_add(OB_PRIORITY), 4);
            wb(m, a.wrapping_add(OB_ACTWID), 16);
            let subtipo = u16::from(rb(m, a.wrapping_add(OB_SUBTYPE)));
            let tempo = subtipo.wrapping_mul(64).wrapping_sub(1);
            ww(m, a.wrapping_add(TEMPO), tempo);
            ww(m, a.wrapping_add(TEMPO_BASE), tempo);
            let estado = rb(m, a.wrapping_add(OB_STATUS));
            ww(
                m,
                a.wrapping_add(OB_VELX),
                if estado & 1 == 0 { 0xffc0 } else { 0x0040 },
            );
        }
        2 => {}
        _ => return Err(ErroAbertura::NaoImplementado("rotina do Jaws")),
    }
    let tempo = rw(m, a.wrapping_add(TEMPO)).wrapping_sub(1);
    ww(m, a.wrapping_add(TEMPO), tempo);
    if i16::from_be_bytes(tempo.to_be_bytes()) < 0 {
        let base = rw(m, a.wrapping_add(TEMPO_BASE));
        ww(m, a.wrapping_add(TEMPO), base);
        let velocidade = rw(m, a.wrapping_add(OB_VELX)).wrapping_neg();
        ww(m, a.wrapping_add(OB_VELX), velocidade);
        let estado = rb(m, a.wrapping_add(OB_STATUS)) ^ 1;
        wb(m, a.wrapping_add(OB_STATUS), estado);
        wb(m, a.wrapping_add(OB_PREV_ANI), 1);
    }
    animate_sprite(m, a, ANI_JAWS)?;
    speed_to_pos(m, a);
    remember_state(m, a);
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    fn memoria() -> Memoria {
        let mut rom = vec![0; 0x20000];
        rom[0xb2f2..0xb2f4].copy_from_slice(&2u16.to_be_bytes());
        rom[0xb2f4..0xb2f7].copy_from_slice(&[5, 0, 0xff]);
        Memoria::nova(rom)
    }
    #[test]
    fn inicializacao_define_periodo_e_movimento() {
        let mut m = memoria();
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x2c);
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 6);
        ww(&mut m, a.wrapping_add(OB_X), 0x200);
        executar(&mut m, a).unwrap();
        assert_eq!(rw(&mut m, a.wrapping_add(TEMPO_BASE)), 383);
        assert_eq!(rw(&mut m, a.wrapping_add(TEMPO)), 382);
        assert_eq!(rw(&mut m, a.wrapping_add(OB_VELX)), 0xffc0);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_COLTYPE)), 10);
    }
    #[test]
    fn tempo_esgotado_inverte_velocidade_e_reinicia_contagem() {
        let mut m = memoria();
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x2c);
        wb(&mut m, a.wrapping_add(OB_ROUTINE), 2);
        ww(&mut m, a.wrapping_add(OB_X), 0x200);
        ww(&mut m, a.wrapping_add(OB_VELX), 0xffc0);
        ww(&mut m, a.wrapping_add(TEMPO_BASE), 383);
        executar(&mut m, a).unwrap();
        assert_eq!(rw(&mut m, a.wrapping_add(OB_VELX)), 0x40);
        assert_eq!(rw(&mut m, a.wrapping_add(TEMPO)), 383);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_STATUS)) & 1, 1);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_PREV_ANI)), 0);
    }
}
