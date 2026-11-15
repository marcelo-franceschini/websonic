//! Tijolos fixos e móveis da Marble Zone, objeto 0x46.
use super::fisica::{obj_floor_block, obj_floor_dist, speed_to_pos};
use super::objetos::{delete_object, display_sprite, fora_da_janela, solid_object};
use super::ram::*;
use super::{ErroAbertura, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;
const ORIG_Y: u32 = 0x30;

pub fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            wb(m, a.wrapping_add(OB_ROUTINE), 2);
            wb(m, a.wrapping_add(OB_HEIGHT), 15);
            wb(m, a.wrapping_add(OB_WIDTH), 15);
            wl(m, a.wrapping_add(OB_MAP), 0xEF90);
            ww(m, a.wrapping_add(OB_GFX), 0x4000);
            wb(m, a.wrapping_add(OB_RENDER), 4);
            wb(m, a.wrapping_add(OB_PRIORITY), 3);
            wb(m, a.wrapping_add(OB_ACTWID), 16);
            let y = rw(m, a.wrapping_add(OB_Y));
            ww(m, a.wrapping_add(ORIG_Y), y);
            ww(m, a.wrapping_add(0x32), 0x5C0);
        }
        2 => {}
        _ => return Err(ErroAbertura::NaoImplementado("rotina do tijolo")),
    }
    if rb(m, a.wrapping_add(OB_RENDER)) & 0x80 != 0 {
        mover(m, a)?;
        solid_object(m, a, 27, 16, 17)?;
    }
    let x = rw(m, a.wrapping_add(OB_X));
    if fora_da_janela(m, x) {
        delete_object(m, a);
    } else {
        display_sprite(m, a);
    }
    Ok(())
}

fn mover(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let subtipo = rb(m, a.wrapping_add(OB_SUBTYPE));
    let mut tipo = subtipo & 7;
    if tipo == 2 {
        let px = rw(m, V_PLAYER.wrapping_add(OB_X));
        let x = rw(m, a.wrapping_add(OB_X));
        let (d, negativo) = px.overflowing_sub(x);
        let d = if negativo { d.wrapping_neg() } else { d };
        if d < 0x90 {
            wb(m, a.wrapping_add(OB_SUBTYPE), 3);
        }
        tipo = 1;
    }
    match tipo {
        0 => {}
        1 => {
            let v = u16::from(rb(m, V_OSCILLATE.wrapping_add(0x16)));
            let v = if subtipo & 8 != 0 {
                16u16.wrapping_sub(v)
            } else {
                v
            };
            let y = rw(m, a.wrapping_add(ORIG_Y)).wrapping_sub(v);
            ww(m, a.wrapping_add(OB_Y), y);
        }
        3 => {
            speed_to_pos(m, a);
            let vy = rw(m, a.wrapping_add(OB_VELY)).wrapping_add(0x18);
            ww(m, a.wrapping_add(OB_VELY), vy);
            let d = obj_floor_dist(m, a);
            if i16::from_be_bytes(d.to_be_bytes()) < 0 {
                let y = rw(m, a.wrapping_add(OB_Y)).wrapping_add(d);
                ww(m, a.wrapping_add(OB_Y), y);
                ww(m, a.wrapping_add(OB_VELY), 0);
                ww(m, a.wrapping_add(ORIG_Y), y);
                let bloco = obj_floor_block(m, a) & 0x3FF;
                wb(
                    m,
                    a.wrapping_add(OB_SUBTYPE),
                    if bloco >= 0x16A { 4 } else { 0 },
                );
            }
        }
        4 => {
            let v = u16::from(rb(m, V_OSCILLATE.wrapping_add(0x12))) >> 3;
            let y = rw(m, a.wrapping_add(ORIG_Y)).wrapping_sub(v);
            ww(m, a.wrapping_add(OB_Y), y);
        }
        _ => return Err(ErroAbertura::NaoImplementado("subtipo do tijolo")),
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn proximidade_dispara_queda_apos_quadro_de_oscilacao() {
        let mut m = Memoria::nova(vec![0; 0x10000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 2);
        ww(&mut m, a.wrapping_add(OB_X), 0x200);
        ww(&mut m, a.wrapping_add(ORIG_Y), 0x120);
        ww(&mut m, V_PLAYER.wrapping_add(OB_X), 0x240);
        wb(&mut m, V_OSCILLATE.wrapping_add(0x16), 4);
        mover(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_SUBTYPE)), 3);
        assert_eq!(rw(&mut m, a.wrapping_add(OB_Y)), 0x11C);
    }
}
