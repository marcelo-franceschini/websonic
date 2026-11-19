//! Ventilador de Star Light, objeto 0x5D (REV01).
use super::super::ram::*;
use super::super::{ErroAbertura, rb, rw, wb, wl, ww};
use super::{delete_object, display_sprite, fora_da_janela};
use crate::memoria::Memoria;

const MAP_FAN: u32 = 0x11CE4;
const FAN_TIME: u32 = 0x30;
const FAN_SWITCH: u32 = 0x32;

fn s16(w: u16) -> i16 {
    i16::from_be_bytes(w.to_be_bytes())
}

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let mut rotina = rb(m, a.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        rotina = 2;
        wb(m, a.wrapping_add(OB_ROUTINE), rotina);
        wl(m, a.wrapping_add(OB_MAP), MAP_FAN);
        ww(m, a.wrapping_add(OB_GFX), 0x43A0);
        wb(m, a.wrapping_add(OB_RENDER), 4);
        wb(m, a.wrapping_add(OB_ACTWID), 16);
        wb(m, a.wrapping_add(OB_PRIORITY), 4);
    }

    let sub = rb(m, a.wrapping_add(OB_SUBTYPE));
    if sub & 2 == 0 {
        let mut time = s16(rw(m, a.wrapping_add(FAN_TIME))).wrapping_sub(1);
        if time < 0 {
            let sw = rb(m, a.wrapping_add(FAN_SWITCH)) ^ 1;
            wb(m, a.wrapping_add(FAN_SWITCH), sw);
            time = if sw == 0 { 2 * 60 } else { 3 * 60 };
        }
        ww(m, a.wrapping_add(FAN_TIME), time.cast_unsigned());
    }

    if rb(m, a.wrapping_add(FAN_SWITCH)) == 0 && rw(m, V_DEBUGUSE) == 0 {
        let mut d0 = rw(m, V_PLAYER.wrapping_add(OB_X)).wrapping_sub(rw(m, a.wrapping_add(OB_X)));
        if rb(m, a.wrapping_add(OB_STATUS)) & 1 == 0 {
            d0 = d0.wrapping_neg();
        }
        d0 = d0.wrapping_add(0x50);
        if d0 < 0xA0 + 0x50 {
            let d1 = rw(m, V_PLAYER.wrapping_add(OB_Y))
                .wrapping_add(0x60)
                .wrapping_sub(rw(m, a.wrapping_add(OB_Y)));
            if d1 < 0x10 + 0x60 {
                let mut force = d0.wrapping_sub(0x50);
                if s16(force) < 0 {
                    force = (!force).wrapping_mul(2);
                }
                force = force.wrapping_add(0x60);
                if rb(m, a.wrapping_add(OB_STATUS)) & 1 == 0 {
                    force = force.wrapping_neg();
                }
                let low = u8::try_from(force & 0xff).unwrap_or(0).wrapping_neg();
                let f_signed = ((force & 0xFF00) | u16::from(low)).cast_signed();
                let mut shift = (f_signed >> 4).cast_unsigned();
                if sub & 1 != 0 {
                    shift = shift.wrapping_neg();
                }
                let px = rw(m, V_PLAYER.wrapping_add(OB_X)).wrapping_add(shift);
                ww(m, V_PLAYER.wrapping_add(OB_X), px);
            }
        }
    }

    let time = rb(m, a.wrapping_add(OB_TIMEFRAME))
        .cast_signed()
        .wrapping_sub(1);
    if time < 0 {
        wb(m, a.wrapping_add(OB_TIMEFRAME), 0);
        let mut ani = rb(m, a.wrapping_add(OB_ANI_FRAME)).wrapping_add(1);
        if ani >= 3 {
            ani = 0;
        }
        wb(m, a.wrapping_add(OB_ANI_FRAME), ani);
        let base: u8 = if sub & 1 != 0 { 2 } else { 0 };
        wb(m, a.wrapping_add(OB_FRAME), base.wrapping_add(ani));
    } else {
        wb(m, a.wrapping_add(OB_TIMEFRAME), time.cast_unsigned());
    }

    let x = rw(m, a.wrapping_add(OB_X));
    if fora_da_janela(m, x) {
        delete_object(m, a);
    } else {
        display_sprite(m, a);
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn inicializacao_fan_slz() {
        let mut m = Memoria::nova(vec![0; 0x1000]);
        let a0: u32 = 0xFF_D800;
        wb(&mut m, a0.wrapping_add(OB_ID), 0x5D);
        wb(&mut m, a0.wrapping_add(OB_ROUTINE), 0);
        ww(&mut m, a0.wrapping_add(OB_X), 0x100);
        ww(&mut m, a0.wrapping_add(OB_Y), 0x100);

        let res = executar(&mut m, a0);
        assert!(res.is_ok());
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ACTWID)), 16);
    }
}
