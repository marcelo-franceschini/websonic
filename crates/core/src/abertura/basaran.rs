//! Basaran, inimigo voador da Marble Zone (objeto 0x55).
use super::fisica::{obj_hit_ceiling, speed_to_pos};
use super::objetos::{animate_sprite, delete_object, remember_state};
use super::ram::*;
use super::{ErroAbertura, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;

const MAP_BASARAN: u32 = 0x108CA;
const ANI_BASARAN: u32 = 0x108B6;
const SONIC_Y: u32 = 0x36;

pub fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            wb(m, a.wrapping_add(OB_ROUTINE), 2);
            wl(m, a.wrapping_add(OB_MAP), MAP_BASARAN);
            ww(m, a.wrapping_add(OB_GFX), 0x84B8);
            wb(m, a.wrapping_add(OB_RENDER), 4);
            wb(m, a.wrapping_add(OB_HEIGHT), 12);
            wb(m, a.wrapping_add(OB_PRIORITY), 2);
            wb(m, a.wrapping_add(OB_COLTYPE), 0x0B);
            wb(m, a.wrapping_add(OB_ACTWID), 16);
        }
        2 => {}
        _ => return Err(ErroAbertura::NaoImplementado("rotina do Basaran")),
    }
    match rb(m, a.wrapping_add(OB_ROUTINE_2)) {
        0 => {
            let (distancia, _) = orientar(m, a);
            if distancia < 128 {
                let y_sonic = rw(m, V_PLAYER.wrapping_add(OB_Y));
                ww(m, a.wrapping_add(SONIC_Y), y_sonic);
                let y = rw(m, a.wrapping_add(OB_Y));
                if y_sonic >= y
                    && y_sonic.wrapping_sub(y) < 128
                    && rw(m, V_DEBUGUSE) == 0
                    && quadro_ativacao(m, a)
                {
                    wb(m, a.wrapping_add(OB_ANIM), 1);
                    wb(m, a.wrapping_add(OB_ROUTINE_2), 2);
                }
            }
        }
        2 => {
            speed_to_pos(m, a);
            let velocidade = rw(m, a.wrapping_add(OB_VELY)).wrapping_add(0x18);
            ww(m, a.wrapping_add(OB_VELY), velocidade);
            let (_, vx) = orientar(m, a);
            let alvo = rw(m, a.wrapping_add(SONIC_Y));
            let y = rw(m, a.wrapping_add(OB_Y));
            if alvo < y {
                if rb(m, a.wrapping_add(OB_RENDER)) & 0x80 == 0 {
                    delete_object(m, a);
                }
            } else if alvo.wrapping_sub(y) < 16 {
                ww(m, a.wrapping_add(OB_VELX), vx);
                ww(m, a.wrapping_add(OB_VELY), 0);
                wb(m, a.wrapping_add(OB_ANIM), 2);
                wb(m, a.wrapping_add(OB_ROUTINE_2), 4);
            }
        }
        4 => {
            if rb(m, V_VBLANK_BYTE) & 0x0F == 0 {
                wb(m, V_SOUNDQUEUE1, 0xC0);
            }
            speed_to_pos(m, a);
            let (distancia, _) = distancia(m, a);
            if distancia >= 128 && quadro_ativacao(m, a) {
                wb(m, a.wrapping_add(OB_ROUTINE_2), 6);
            }
        }
        6 => {
            speed_to_pos(m, a);
            let vy = rw(m, a.wrapping_add(OB_VELY)).wrapping_sub(0x18);
            ww(m, a.wrapping_add(OB_VELY), vy);
            let d = obj_hit_ceiling(m, a);
            if i16::from_be_bytes(d.to_be_bytes()) < 0 {
                let y = rw(m, a.wrapping_add(OB_Y)).wrapping_sub(d);
                let x = rw(m, a.wrapping_add(OB_X)) & 0xFFF8;
                ww(m, a.wrapping_add(OB_Y), y);
                ww(m, a.wrapping_add(OB_X), x);
                ww(m, a.wrapping_add(OB_VELX), 0);
                ww(m, a.wrapping_add(OB_VELY), 0);
                wb(m, a.wrapping_add(OB_ANIM), 0);
                wb(m, a.wrapping_add(OB_ROUTINE_2), 0);
            }
        }
        _ => return Err(ErroAbertura::NaoImplementado("ação do Basaran")),
    }
    animate_sprite(m, a, ANI_BASARAN)?;
    remember_state(m, a);
    Ok(())
}

fn distancia(m: &mut Memoria, a: u32) -> (u16, u16) {
    let sonic_x = rw(m, V_PLAYER.wrapping_add(OB_X));
    let x = rw(m, a.wrapping_add(OB_X));
    let (delta, esquerda) = sonic_x.overflowing_sub(x);
    if esquerda {
        (delta.wrapping_neg(), 0xFF00)
    } else {
        (delta, 0x100)
    }
}
fn orientar(m: &mut Memoria, a: u32) -> (u16, u16) {
    let (d, velocidade) = distancia(m, a);
    let estado = rb(m, a.wrapping_add(OB_STATUS));
    wb(
        m,
        a.wrapping_add(OB_STATUS),
        if velocidade == 0x100 {
            estado | 1
        } else {
            estado & !1
        },
    );
    (d, velocidade)
}
fn quadro_ativacao(m: &mut Memoria, a: u32) -> bool {
    let indice = a.wrapping_sub(V_OBJSPACE) / OBJECT_SIZE;
    let d7 = 127u32.wrapping_sub(indice);
    let d7 = u8::try_from(d7 & 0xFF).unwrap_or(0);
    rb(m, V_VBLANK_BYTE).wrapping_add(d7) & 7 == 0
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn detecta_sonic_e_inicia_queda_no_quadro_apropriado() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x55);
        ww(&mut m, a.wrapping_add(OB_X), 0x100);
        ww(&mut m, a.wrapping_add(OB_Y), 0x100);
        ww(&mut m, V_PLAYER.wrapping_add(OB_X), 0x140);
        ww(&mut m, V_PLAYER.wrapping_add(OB_Y), 0x160);
        wb(&mut m, V_VBLANK_BYTE, 1);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE_2)), 2);
        assert_eq!(rw(&mut m, a.wrapping_add(SONIC_Y)), 0x160);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_STATUS)) & 1, 1);
    }
}
