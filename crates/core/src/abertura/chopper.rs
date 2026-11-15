//! Chopper (piranha da Green Hill Zone): salto vertical sobre as pontes e cachoeiras.
use super::fisica::speed_to_pos;
use super::objetos::{animate_sprite, delete_object, display_sprite, fora_da_janela};
use super::ram::*;
use super::{ErroAbertura, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;

pub const MAP_CHOP: u32 = 0xB254;
pub const ANI_CHOP: u32 = 0xB242;
pub const ART_TILE_CHOPPER: u16 = 0x047B;
pub const CHOP_ORIG_Y: u32 = 0x30;

fn u16_de(i: i16) -> u16 {
    u16::from_be_bytes(i.to_be_bytes())
}

fn s16(w: u16) -> i16 {
    i16::from_be_bytes(w.to_be_bytes())
}

pub fn executar(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    match rb(m, a0.wrapping_add(OB_ROUTINE)) {
        0 => {
            wb(m, a0.wrapping_add(OB_ROUTINE), 2);
            wl(m, a0.wrapping_add(OB_MAP), MAP_CHOP);
            ww(m, a0.wrapping_add(OB_GFX), ART_TILE_CHOPPER);
            wb(m, a0.wrapping_add(OB_RENDER), 4);
            wb(m, a0.wrapping_add(OB_PRIORITY), 4);
            wb(m, a0.wrapping_add(OB_COLTYPE), 9);
            wb(m, a0.wrapping_add(OB_ACTWID), 16);
            ww(m, a0.wrapping_add(OB_VELY), u16_de(-0x700i16));
            let y = rw(m, a0.wrapping_add(OB_Y));
            ww(m, a0.wrapping_add(CHOP_ORIG_Y), y);

            chg_speed(m, a0)?;
        }
        2 => {
            chg_speed(m, a0)?;
        }
        _ => return Err(ErroAbertura::NaoImplementado("rotina de Chopper")),
    }

    remember_state(m, a0);
    Ok(())
}

fn chg_speed(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    animate_sprite(m, a0, ANI_CHOP)?;
    speed_to_pos(m, a0);
    let vy = s16(rw(m, a0.wrapping_add(OB_VELY))).wrapping_add(0x18);
    ww(m, a0.wrapping_add(OB_VELY), u16_de(vy));

    let mut d0 = rw(m, a0.wrapping_add(CHOP_ORIG_Y));
    let obj_y = rw(m, a0.wrapping_add(OB_Y));
    if d0 < obj_y {
        ww(m, a0.wrapping_add(OB_Y), d0);
        ww(m, a0.wrapping_add(OB_VELY), u16_de(-0x700i16));
    }

    wb(m, a0.wrapping_add(OB_ANIM), 1);
    d0 = d0.wrapping_sub(0xC0);
    if d0 < obj_y {
        wb(m, a0.wrapping_add(OB_ANIM), 0);
        if s16(rw(m, a0.wrapping_add(OB_VELY))) >= 0 {
            wb(m, a0.wrapping_add(OB_ANIM), 2);
        }
    }
    Ok(())
}

fn remember_state(m: &mut Memoria, a0: u32) {
    let x = rw(m, a0.wrapping_add(OB_X));
    if fora_da_janela(m, x) {
        let n = rb(m, a0.wrapping_add(OB_RESPAWNNO));
        if n != 0 {
            let marca = V_OBJSTATE.wrapping_add(2).wrapping_add(u32::from(n));
            let valor = rb(m, marca) & 0x7F;
            wb(m, marca, valor);
        }
        delete_object(m, a0);
    } else {
        display_sprite(m, a0);
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::abertura::rl;

    fn memoria_teste() -> Memoria {
        let mut rom = vec![0u8; 0x10000];
        // Animações sintéticas, distintas dos dados do jogo.
        for indice in 0..3usize {
            let offset = u16::try_from(6usize.saturating_add(indice.saturating_mul(3))).unwrap();
            let local = 0xB242usize.saturating_add(indice.saturating_mul(2));
            rom[local..local.saturating_add(2)].copy_from_slice(&offset.to_be_bytes());
            let inicio = 0xB242usize.saturating_add(usize::from(offset));
            rom[inicio..inicio.saturating_add(3)].copy_from_slice(&[5, 3, 0xFF]);
        }
        Memoria::nova(rom)
    }

    #[test]
    fn chopper_inicializa_e_sobe() {
        let mut m = memoria_teste();
        let a0 = V_LVLOBJSPACE;
        wb(&mut m, a0.wrapping_add(OB_ID), ID_CHOPPER);
        ww(&mut m, a0.wrapping_add(OB_X), 0x200);
        ww(&mut m, a0.wrapping_add(OB_Y), 0x300);
        ww(&mut m, V_SCREENPOSX, 0x180);
        ww(&mut m, V_SCREENPOSY, 0x280);

        executar(&mut m, a0).expect("chopper executa");

        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(rl(&mut m, a0.wrapping_add(OB_MAP)), MAP_CHOP);
        assert_eq!(rw(&mut m, a0.wrapping_add(OB_GFX)), ART_TILE_CHOPPER);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_COLTYPE)), 9);
        assert_eq!(rw(&mut m, a0.wrapping_add(CHOP_ORIG_Y)), 0x300);
        // Pos-speed_to_pos com velocidade inicial -0x700:
        assert!(rw(&mut m, a0.wrapping_add(OB_Y)) < 0x300);
    }

    #[test]
    fn chopper_atinge_orig_y_e_reinicia_salto() {
        let mut m = memoria_teste();
        let a0 = V_LVLOBJSPACE;
        wb(&mut m, a0.wrapping_add(OB_ID), ID_CHOPPER);
        wb(&mut m, a0.wrapping_add(OB_ROUTINE), 2);
        ww(&mut m, a0.wrapping_add(OB_X), 0x200);
        ww(&mut m, a0.wrapping_add(OB_Y), 0x300);
        ww(&mut m, a0.wrapping_add(CHOP_ORIG_Y), 0x300);
        ww(&mut m, a0.wrapping_add(OB_VELY), 0x200); // Caindo
        ww(&mut m, V_SCREENPOSX, 0x180);
        ww(&mut m, V_SCREENPOSY, 0x280);

        executar(&mut m, a0).expect("chopper executa");

        // Como caiu alem de origY, foi reiniciado para origY e velY -0x700
        assert_eq!(rw(&mut m, a0.wrapping_add(OB_Y)), 0x300);
        assert_eq!(rw(&mut m, a0.wrapping_add(OB_VELY)), u16_de(-0x700i16));
    }

    #[test]
    fn chopper_fora_da_janela_e_removido() {
        let mut m = memoria_teste();
        let a0 = V_LVLOBJSPACE;
        wb(&mut m, a0.wrapping_add(OB_ID), ID_CHOPPER);
        wb(&mut m, a0.wrapping_add(OB_ROUTINE), 2);
        ww(&mut m, a0.wrapping_add(OB_X), 0x1000); // Longe da tela
        ww(&mut m, a0.wrapping_add(OB_Y), 0x300);
        ww(&mut m, a0.wrapping_add(CHOP_ORIG_Y), 0x300);
        wb(&mut m, a0.wrapping_add(OB_RESPAWNNO), 5);
        wb(&mut m, V_OBJSTATE.wrapping_add(2).wrapping_add(5), 0x80);

        executar(&mut m, a0).expect("chopper executa");

        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ID)), 0);
        assert_eq!(rb(&mut m, V_OBJSTATE.wrapping_add(2).wrapping_add(5)), 0);
    }
}
