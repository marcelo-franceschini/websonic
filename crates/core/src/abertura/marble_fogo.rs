//! Geradores e bolas de lava (objetos 13 e 14).
use super::boot::{rom_b, rom_w};
use super::fisica::speed_to_pos;
use super::objetos::{
    animate_sprite, delete_object, display_sprite, find_free_obj, fora_da_janela,
};
use super::ram::*;
use super::{ErroAbertura, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;

const INTERVALO: u32 = 0x1F;
const TEMPO: u32 = 0x1E;
const ORIG_Y: u32 = 0x30;

/// Gerador invisível com período e velocidade lidos da ROM.
pub fn gerador(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    if rb(m, a.wrapping_add(OB_ROUTINE)) == 0 {
        wb(m, a.wrapping_add(OB_ROUTINE), 2);
        let tipo = rb(m, a.wrapping_add(OB_SUBTYPE));
        let intervalo = rom_b(m, 0xE93Au32.wrapping_add(u32::from(tipo >> 4)));
        wb(m, a.wrapping_add(INTERVALO), intervalo);
        wb(m, a.wrapping_add(TEMPO), intervalo);
        wb(m, a.wrapping_add(OB_SUBTYPE), tipo & 15);
    }
    let tempo = rb(m, a.wrapping_add(TEMPO)).wrapping_sub(1);
    wb(m, a.wrapping_add(TEMPO), tempo);
    if tempo == 0 {
        let intervalo = rb(m, a.wrapping_add(INTERVALO));
        wb(m, a.wrapping_add(TEMPO), intervalo);
        let x = rw(m, a.wrapping_add(OB_X));
        let y = rw(m, a.wrapping_add(OB_Y));
        let dx = x.wrapping_sub(rw(m, V_SCREENPOSX));
        let dy = y.wrapping_sub(rw(m, V_SCREENPOSY));
        if dx < 320
            && dy < 224
            && let Some(b) = find_free_obj(m)
        {
            wb(m, b.wrapping_add(OB_ID), 0x14);
            ww(m, b.wrapping_add(OB_X), x);
            ww(m, b.wrapping_add(OB_Y), y);
            let tipo = rb(m, a.wrapping_add(OB_SUBTYPE));
            wb(m, b.wrapping_add(OB_SUBTYPE), tipo);
        }
    }
    let x = rw(m, a.wrapping_add(OB_X));
    if fora_da_janela(m, x) {
        delete_object(m, a);
    }
    Ok(())
}

/// Bola de lava vertical, incluindo os lançamentos de MZ1.
pub fn bola(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    if rb(m, a.wrapping_add(OB_ROUTINE)) == 4 {
        delete_object(m, a);
        display_sprite(m, a);
        return Ok(());
    }
    if rb(m, a.wrapping_add(OB_ROUTINE)) == 0 {
        wb(m, a.wrapping_add(OB_ROUTINE), 2);
        wb(m, a.wrapping_add(OB_HEIGHT), 8);
        wb(m, a.wrapping_add(OB_WIDTH), 8);
        wl(m, a.wrapping_add(OB_MAP), 0xB9F8);
        let gfx = if rb(m, V_ZONE) == 3 { 0x480 } else { 0x345 };
        ww(m, a.wrapping_add(OB_GFX), gfx);
        wb(m, a.wrapping_add(OB_RENDER), 4);
        let prioridade = if rb(m, a.wrapping_add(0x29)) == 0 {
            3
        } else {
            5
        };
        wb(m, a.wrapping_add(OB_PRIORITY), prioridade);
        wb(m, a.wrapping_add(OB_COLTYPE), 0x8B);
        let y = rw(m, a.wrapping_add(OB_Y));
        ww(m, a.wrapping_add(ORIG_Y), y);
        let tipo = rb(m, a.wrapping_add(OB_SUBTYPE));
        let velocidade = rom_w(m, 0xE9AAu32.wrapping_add(u32::from(tipo).wrapping_mul(2)));
        ww(m, a.wrapping_add(OB_VELY), velocidade);
        wb(m, a.wrapping_add(OB_ACTWID), 8);
        if tipo >= 6 {
            wb(m, a.wrapping_add(OB_ACTWID), 16);
            wb(m, a.wrapping_add(OB_ANIM), 2);
            ww(m, a.wrapping_add(OB_VELX), velocidade);
            ww(m, a.wrapping_add(OB_VELY), 0);
        }
        wb(m, V_SOUNDQUEUE1, 0xAE);
    }
    let tipo = rb(m, a.wrapping_add(OB_SUBTYPE));
    match tipo {
        0..=3 => {
            let velocidade = rw(m, a.wrapping_add(OB_VELY)).wrapping_add(0x18);
            ww(m, a.wrapping_add(OB_VELY), velocidade);
            if rw(m, a.wrapping_add(ORIG_Y)) < rw(m, a.wrapping_add(OB_Y)) {
                wb(m, a.wrapping_add(OB_ROUTINE), 4);
            }
            let status = rb(m, a.wrapping_add(OB_STATUS)) & !2;
            wb(
                m,
                a.wrapping_add(OB_STATUS),
                if velocidade & 0x8000 != 0 {
                    status | 2
                } else {
                    status
                },
            );
        }
        8 => {}
        _ => {
            return Err(ErroAbertura::NaoImplementado(
                "colisão de bola de lava com parede ou teto",
            ));
        }
    }
    speed_to_pos(m, a);
    animate_sprite(m, a, 0xEB52)?;
    let x = rw(m, a.wrapping_add(OB_X));
    if fora_da_janela(m, x) {
        delete_object(m, a);
    }
    display_sprite(m, a);
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn gerador_temporiza_e_cria_bola_quando_visivel() {
        let mut rom = vec![0; 0x10000];
        rom[0xE93D] = 120;
        let mut m = Memoria::nova(rom);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x13);
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 0x30);
        ww(&mut m, a.wrapping_add(OB_X), 100);
        ww(&mut m, a.wrapping_add(OB_Y), 100);
        gerador(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(TEMPO)), 119);
        wb(&mut m, a.wrapping_add(TEMPO), 1);
        gerador(&mut m, a).unwrap();
        let b = a.wrapping_add(OBJECT_SIZE);
        assert_eq!(rb(&mut m, b.wrapping_add(OB_ID)), 0x14);
        assert_eq!(rw(&mut m, b.wrapping_add(OB_Y)), 100);
    }
}
