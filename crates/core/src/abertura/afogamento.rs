//! Contagem de ar e bolhas de Sonic em Labyrinth Zone (objeto 0x0A).
use super::fisica::{random_number, sonic_reset_on_floor, speed_to_pos};
use super::objetos::{animate_sprite, delete_object, display_sprite, find_free_obj};
use super::ram::*;
use super::{ErroAbertura, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;

const WATER1: u32 = 0xFF_F646;
const MAP_BUB: u32 = 0x130A0;
const MAP_UNUSED_FACE: u32 = 0x148B8;
const ANI_DROWN: u32 = 0x14820;
const WOBBLE: u32 = 0x144F0;
const RESTART: u32 = 0x2C;
const ORIG_X: u32 = 0x30;
const DISPLAY_TIME: u32 = 0x32;
const TIPO: u32 = 0x33;
const EXTRAS: u32 = 0x34;
const FLAGS: u32 = 0x36;
const NUM_TIME: u32 = 0x38;
const DELAY: u32 = 0x3A;

pub fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            wb(m, a.wrapping_add(OB_ROUTINE), 2);
            wl(m, a.wrapping_add(OB_MAP), MAP_BUB);
            ww(m, a.wrapping_add(OB_GFX), 0x8348);
            wb(m, a.wrapping_add(OB_RENDER), 0x84);
            wb(m, a.wrapping_add(OB_ACTWID), 16);
            wb(m, a.wrapping_add(OB_PRIORITY), 1);
            let subtipo = rb(m, a.wrapping_add(OB_SUBTYPE));
            if subtipo & 0x80 != 0 {
                wb(m, a.wrapping_add(OB_ROUTINE), 10);
                wl(m, a.wrapping_add(OB_MAP), MAP_UNUSED_FACE);
                ww(m, a.wrapping_add(OB_GFX), 0x440);
                wb(m, a.wrapping_add(TIPO), subtipo & 0x7F);
                return contagem(m, a);
            }
            wb(m, a.wrapping_add(OB_ANIM), subtipo);
            let x = rw(m, a.wrapping_add(OB_X));
            ww(m, a.wrapping_add(ORIG_X), x);
            ww(m, a.wrapping_add(OB_VELY), 0xFF78);
            animar(m, a)?;
            verificar_agua(m, a)
        }
        2 => {
            animar(m, a)?;
            verificar_agua(m, a)
        }
        4 => verificar_agua(m, a),
        6 | 14 => {
            mostrar_numero(m, a);
            animar(m, a)?;
            display_sprite(m, a);
            Ok(())
        }
        8 | 16 => {
            delete_object(m, a);
            Ok(())
        }
        10 => contagem(m, a),
        12 => ar_restante(m, a),
        _ => Err(ErroAbertura::NaoImplementado("rotina da contagem de ar")),
    }
}
fn animar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    animate_sprite(m, a, ANI_DROWN)
}
fn verificar_agua(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    if rw(m, WATER1) >= rw(m, a.wrapping_add(OB_Y)) {
        wb(m, a.wrapping_add(OB_ROUTINE), 6);
        let anim = rb(m, a.wrapping_add(OB_ANIM)).wrapping_add(7);
        wb(m, a.wrapping_add(OB_ANIM), anim);
        mostrar_numero(m, a);
        animar(m, a)?;
        display_sprite(m, a);
        return Ok(());
    }
    if rb(m, F_WTUNNELMODE) != 0 {
        let x = rw(m, a.wrapping_add(ORIG_X)).wrapping_add(4);
        ww(m, a.wrapping_add(ORIG_X), x);
    }
    let angulo = rb(m, a.wrapping_add(OB_ANGLE));
    wb(m, a.wrapping_add(OB_ANGLE), angulo.wrapping_add(1));
    let onda = super::boot::rom_b(m, WOBBLE.wrapping_add(u32::from(angulo & 0x7F)));
    let desloc = i16::from(i8::from_be_bytes([onda]));
    let x = rw(m, a.wrapping_add(ORIG_X)).wrapping_add(u16::from_be_bytes(desloc.to_be_bytes()));
    ww(m, a.wrapping_add(OB_X), x);
    mostrar_numero(m, a);
    speed_to_pos(m, a);
    if rb(m, a.wrapping_add(OB_RENDER)) & 0x80 == 0 {
        delete_object(m, a);
    } else {
        display_sprite(m, a);
    }
    Ok(())
}
fn mostrar_numero(m: &mut Memoria, a: u32) {
    let tempo = rw(m, a.wrapping_add(NUM_TIME));
    if tempo == 0 {
        return;
    }
    let tempo = tempo.wrapping_sub(1);
    ww(m, a.wrapping_add(NUM_TIME), tempo);
    if tempo != 0 || rb(m, a.wrapping_add(OB_ANIM)) >= 7 {
        return;
    }
    ww(m, a.wrapping_add(NUM_TIME), 15);
    ww(m, a.wrapping_add(OB_VELY), 0);
    wb(m, a.wrapping_add(OB_RENDER), 0x80);
    let x = rw(m, a.wrapping_add(OB_X))
        .wrapping_sub(rw(m, V_SCREENPOSX))
        .wrapping_add(0x80);
    let y = rw(m, a.wrapping_add(OB_Y))
        .wrapping_sub(rw(m, V_SCREENPOSY))
        .wrapping_add(0x80);
    ww(m, a.wrapping_add(OB_X), x);
    ww(m, a.wrapping_add(OB_SCREENY), y);
    wb(m, a.wrapping_add(OB_ROUTINE), 12);
}
fn ar_restante(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    if rw(m, V_AIR) > 12 {
        delete_object(m, a);
        return Ok(());
    }
    let tempo = rw(m, a.wrapping_add(NUM_TIME)).wrapping_sub(1);
    ww(m, a.wrapping_add(NUM_TIME), tempo);
    if tempo == 0 {
        wb(m, a.wrapping_add(OB_ROUTINE), 14);
        let anim = rb(m, a.wrapping_add(OB_ANIM)).wrapping_add(7);
        wb(m, a.wrapping_add(OB_ANIM), anim);
        mostrar_numero(m, a);
        animar(m, a)?;
        display_sprite(m, a);
    } else {
        animar(m, a)?;
        if rb(m, a.wrapping_add(OB_RENDER)) & 0x80 == 0 {
            delete_object(m, a);
        } else {
            display_sprite(m, a);
        }
    }
    Ok(())
}
fn contagem(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let reinicio = rw(m, a.wrapping_add(RESTART));
    if reinicio != 0 {
        let restante = reinicio.wrapping_sub(1);
        ww(m, a.wrapping_add(RESTART), restante);
        if restante == 0 {
            wb(m, V_PLAYER.wrapping_add(OB_ROUTINE), 6);
            return Ok(());
        }
        speed_to_pos(m, V_PLAYER);
        let vy = rw(m, V_PLAYER.wrapping_add(OB_VELY)).wrapping_add(0x10);
        ww(m, V_PLAYER.wrapping_add(OB_VELY), vy);
    } else {
        if rb(m, V_PLAYER.wrapping_add(OB_ROUTINE)) >= 6
            || rb(m, V_PLAYER.wrapping_add(OB_STATUS)) & STATUS_UNDERWATER == 0
        {
            return Ok(());
        }
        let tempo = rw(m, a.wrapping_add(NUM_TIME)).wrapping_sub(1);
        ww(m, a.wrapping_add(NUM_TIME), tempo);
        if tempo & 0x8000 != 0 {
            ww(m, a.wrapping_add(NUM_TIME), 59);
            ww(m, a.wrapping_add(FLAGS), 1);
            let extras = u8::try_from(random_number(m) & 1).unwrap_or(0);
            wb(m, a.wrapping_add(EXTRAS), extras);
            let ar = rw(m, V_AIR);
            if [25, 20, 15].contains(&ar) {
                wb(m, V_SOUNDQUEUE1, 0xC2);
            } else if ar <= 12 {
                if ar == 12 {
                    wb(m, V_SOUNDQUEUE0, 0x92);
                }
                let n = rb(m, a.wrapping_add(DISPLAY_TIME)).wrapping_sub(1);
                wb(m, a.wrapping_add(DISPLAY_TIME), n);
                if n & 0x80 != 0 {
                    let tipo = rb(m, a.wrapping_add(TIPO));
                    wb(m, a.wrapping_add(DISPLAY_TIME), tipo);
                    let flags = rb(m, a.wrapping_add(FLAGS)) | 0x80;
                    wb(m, a.wrapping_add(FLAGS), flags);
                }
            }
            let ar_novo = ar.wrapping_sub(1);
            ww(m, V_AIR, ar_novo);
            if ar == 0 {
                super::sonic_agua::retomar_musica(m);
                wb(m, F_PLAYERCTRL, 0x81);
                wb(m, V_SOUNDQUEUE1, 0xB2);
                wb(m, a.wrapping_add(EXTRAS), 10);
                ww(m, a.wrapping_add(FLAGS), 1);
                ww(m, a.wrapping_add(RESTART), 120);
                sonic_reset_on_floor(m, V_PLAYER);
                wb(m, V_PLAYER.wrapping_add(OB_ANIM), 0x17);
                let status = rb(m, V_PLAYER.wrapping_add(OB_STATUS)) | 2;
                wb(m, V_PLAYER.wrapping_add(OB_STATUS), status);
                let gfx = rb(m, V_PLAYER.wrapping_add(OB_GFX)) | 0x80;
                wb(m, V_PLAYER.wrapping_add(OB_GFX), gfx);
                for campo in [OB_VELX, OB_VELY, OB_INERTIA] {
                    ww(m, V_PLAYER.wrapping_add(campo), 0);
                }
                wb(m, F_NOBGSCROLL, 1);
                return Ok(());
            }
            return gerar_bolha(m, a);
        }
    }
    if rw(m, a.wrapping_add(FLAGS)) == 0 {
        return Ok(());
    }
    let delay = rw(m, a.wrapping_add(DELAY)).wrapping_sub(1);
    ww(m, a.wrapping_add(DELAY), delay);
    if delay & 0x8000 == 0 {
        return Ok(());
    }
    gerar_bolha(m, a)
}
fn gerar_bolha(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let delay = random_number(m) & 15;
    ww(m, a.wrapping_add(DELAY), delay);
    let Some(b) = find_free_obj(m) else {
        return Ok(());
    };
    wb(m, b.wrapping_add(OB_ID), 0x0A);
    let status = rb(m, V_PLAYER.wrapping_add(OB_STATUS));
    let x = rw(m, V_PLAYER.wrapping_add(OB_X));
    ww(
        m,
        b.wrapping_add(OB_X),
        if status & 1 == 0 {
            x.wrapping_add(6)
        } else {
            x.wrapping_sub(6)
        },
    );
    if status & 1 != 0 {
        wb(m, b.wrapping_add(OB_ANGLE), 0x40);
    }
    let y = rw(m, V_PLAYER.wrapping_add(OB_Y));
    ww(m, b.wrapping_add(OB_Y), y);
    wb(m, b.wrapping_add(OB_SUBTYPE), 6);
    if rw(m, a.wrapping_add(RESTART)) != 0 {
        ww(m, a.wrapping_add(DELAY), delay & 7);
        ww(m, b.wrapping_add(OB_Y), y.wrapping_sub(12));
        let sorteio = random_number(m).to_be_bytes()[1];
        wb(m, b.wrapping_add(OB_ANGLE), sorteio);
        if rw(m, V_FRAMECOUNT) & 3 == 0 {
            wb(m, b.wrapping_add(OB_SUBTYPE), 14);
        }
    } else if rb(m, a.wrapping_add(FLAGS)) & 0x80 != 0 {
        let numero = u8::try_from((rw(m, V_AIR) >> 1) & 0xFF).unwrap_or(0);
        let sorteio = random_number(m) & 3;
        let extras = rb(m, a.wrapping_add(EXTRAS));
        let flags = rb(m, a.wrapping_add(FLAGS));
        if (sorteio == 0 || extras == 0) && flags & 0x40 == 0 {
            wb(m, a.wrapping_add(FLAGS), flags | 0x40);
            wb(m, b.wrapping_add(OB_SUBTYPE), numero);
            ww(m, b.wrapping_add(NUM_TIME), 28);
        }
    }
    let restante = rb(m, a.wrapping_add(EXTRAS)).wrapping_sub(1);
    wb(m, a.wrapping_add(EXTRAS), restante);
    if restante & 0x80 != 0 {
        ww(m, a.wrapping_add(FLAGS), 0);
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn segundos_de_ar_produzem_aviso_e_musica() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x0A);
        wb(&mut m, a.wrapping_add(OB_ROUTINE), 10);
        wb(&mut m, V_PLAYER.wrapping_add(OB_STATUS), STATUS_UNDERWATER);
        ww(&mut m, V_AIR, 25);
        executar(&mut m, a).unwrap();
        assert_eq!(rw(&mut m, V_AIR), 24);
        assert_eq!(rb(&mut m, V_SOUNDQUEUE1), 0xC2);
        ww(&mut m, V_AIR, 12);
        ww(&mut m, a.wrapping_add(NUM_TIME), 0);
        executar(&mut m, a).unwrap();
        assert_eq!(rw(&mut m, V_AIR), 11);
        assert_eq!(rb(&mut m, V_SOUNDQUEUE0), 0x92);
    }
    #[test]
    fn sem_ar_inicia_morte_apos_o_atraso() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x0A);
        wb(&mut m, a.wrapping_add(OB_ROUTINE), 10);
        wb(&mut m, V_PLAYER.wrapping_add(OB_STATUS), STATUS_UNDERWATER);
        ww(&mut m, V_AIR, 0);
        executar(&mut m, a).unwrap();
        assert_eq!(rw(&mut m, a.wrapping_add(RESTART)), 120);
        assert_eq!(rb(&mut m, F_PLAYERCTRL), 0x81);
        ww(&mut m, a.wrapping_add(RESTART), 1);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, V_PLAYER.wrapping_add(OB_ROUTINE)), 6);
    }
}
