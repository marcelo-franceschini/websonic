//! Entrada e saída de Sonic na água de Labyrinth Zone, REV01.
use super::ram::*;
use super::{rb, rw, wb, ww};
use crate::memoria::Memoria;

const WATER1: u32 = 0xFF_F646;
const SONIC_BUBBLES: u32 = 0xFF_D340;
const SPLASH: u32 = 0xFF_D300;
const BUBBLE_TIME: u32 = 0x32;

/// Restaura o ar e seleciona a música após o aviso de afogamento.
pub fn retomar_musica(m: &mut Memoria) {
    if rw(m, V_AIR) <= 12 {
        let mut musica = if rw(m, V_ZONE_ACT) == ID_LZ_ACT4 {
            0x86
        } else {
            0x82
        };
        if rb(m, V_INVINC) != 0 {
            musica = 0x87;
        }
        if rb(m, F_LOCKSCREEN) != 0 {
            musica = 0x8C;
        }
        wb(m, V_SOUNDQUEUE0, musica);
    }
    ww(m, V_AIR, 30);
    wb(m, SONIC_BUBBLES.wrapping_add(BUBBLE_TIME), 0);
}

/// Atualiza o estado e a velocidade apenas no momento em que Sonic cruza a superfície.
pub fn atualizar(m: &mut Memoria, sonic: u32) {
    if rb(m, V_ZONE) != ID_LZ {
        return;
    }
    let agua = i16::from_be_bytes(rw(m, WATER1).to_be_bytes());
    let y = i16::from_be_bytes(rw(m, sonic.wrapping_add(OB_Y)).to_be_bytes());
    let status = rb(m, sonic.wrapping_add(OB_STATUS));
    if agua < y {
        if status & STATUS_UNDERWATER != 0 {
            return;
        }
        wb(m, sonic.wrapping_add(OB_STATUS), status | STATUS_UNDERWATER);
        retomar_musica(m);
        wb(m, SONIC_BUBBLES.wrapping_add(OB_ID), 0x0A);
        wb(m, SONIC_BUBBLES.wrapping_add(OB_SUBTYPE), 0x81);
        ww(m, V_SONSPEEDMAX, 0x300);
        ww(m, V_SONSPEEDACC, 6);
        ww(m, V_SONSPEEDDEC, 0x40);
        let x = i16::from_be_bytes(rw(m, sonic.wrapping_add(OB_VELX)).to_be_bytes()) >> 1;
        let y = i16::from_be_bytes(rw(m, sonic.wrapping_add(OB_VELY)).to_be_bytes()) >> 2;
        ww(
            m,
            sonic.wrapping_add(OB_VELX),
            u16::from_be_bytes(x.to_be_bytes()),
        );
        ww(
            m,
            sonic.wrapping_add(OB_VELY),
            u16::from_be_bytes(y.to_be_bytes()),
        );
        if y != 0 {
            criar_splash(m);
        }
    } else {
        if status & STATUS_UNDERWATER == 0 {
            return;
        }
        wb(
            m,
            sonic.wrapping_add(OB_STATUS),
            status & !STATUS_UNDERWATER,
        );
        retomar_musica(m);
        ww(m, V_SONSPEEDMAX, 0x600);
        ww(m, V_SONSPEEDACC, 12);
        ww(m, V_SONSPEEDDEC, 0x80);
        let y = rw(m, sonic.wrapping_add(OB_VELY)).wrapping_shl(1);
        ww(m, sonic.wrapping_add(OB_VELY), y);
        if y == 0 {
            return;
        }
        if i16::from_be_bytes(y.to_be_bytes()) <= -0x1000 {
            ww(m, sonic.wrapping_add(OB_VELY), 0xF000);
        }
        criar_splash(m);
    }
}

fn criar_splash(m: &mut Memoria) {
    wb(m, SPLASH.wrapping_add(OB_ID), 8);
    wb(m, V_SOUNDQUEUE1, 0xAA);
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn cruzar_superficie_altera_velocidade_uma_vez() {
        let mut m = Memoria::nova(vec![0; 0x10000]);
        wb(&mut m, V_ZONE, ID_LZ);
        ww(&mut m, WATER1, 100);
        ww(&mut m, V_PLAYER.wrapping_add(OB_Y), 101);
        ww(&mut m, V_PLAYER.wrapping_add(OB_VELX), 0xFF00);
        ww(&mut m, V_PLAYER.wrapping_add(OB_VELY), 0x0400);
        atualizar(&mut m, V_PLAYER);
        assert_eq!(rw(&mut m, V_PLAYER.wrapping_add(OB_VELX)), 0xFF80);
        assert_eq!(rw(&mut m, V_PLAYER.wrapping_add(OB_VELY)), 0x0100);
        assert_eq!(rw(&mut m, V_SONSPEEDMAX), 0x300);
        assert_eq!(rb(&mut m, SONIC_BUBBLES.wrapping_add(OB_ID)), 0x0A);
        atualizar(&mut m, V_PLAYER);
        assert_eq!(rw(&mut m, V_PLAYER.wrapping_add(OB_VELY)), 0x0100);
        ww(&mut m, V_PLAYER.wrapping_add(OB_Y), 100);
        ww(&mut m, V_PLAYER.wrapping_add(OB_VELY), 0xF700);
        atualizar(&mut m, V_PLAYER);
        assert_eq!(rw(&mut m, V_PLAYER.wrapping_add(OB_VELY)), 0xF000);
        assert_eq!(rw(&mut m, V_SONSPEEDMAX), 0x600);
        assert_eq!(rb(&mut m, SPLASH.wrapping_add(OB_ID)), 8);
    }
    #[test]
    fn musica_do_chefe_tem_prioridade_ao_recuperar_ar() {
        let mut m = Memoria::nova(vec![0; 0x10000]);
        ww(&mut m, V_AIR, 12);
        wb(&mut m, V_INVINC, 1);
        wb(&mut m, F_LOCKSCREEN, 1);
        retomar_musica(&mut m);
        assert_eq!(rb(&mut m, V_SOUNDQUEUE0), 0x8C);
        assert_eq!(rw(&mut m, V_AIR), 30);
    }
}
