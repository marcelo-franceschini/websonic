//! Buzz Bomber e seu projétil.
use super::fisica::speed_to_pos;
use super::objetos::{animate_sprite, delete_object, display_sprite, fora_da_janela};
use super::ram::*;
use super::{ErroAbertura, rb, rl, rw, wb, wl, ww};
use crate::memoria::Memoria;

const TEMPO: u32 = 0x32;
const ESTADO: u32 = 0x34;
const PAI: u32 = 0x3C;
const ANI_BUZZ: u32 = 0xA096;
const ANI_MISSIL: u32 = 0xA0A8;

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a.wrapping_add(OB_ROUTINE));
    if rotina == 4 {
        delete_object(m, a);
        return Ok(());
    }
    if rotina == 0 {
        wb(m, a.wrapping_add(OB_ROUTINE), 2);
        wl(m, a.wrapping_add(OB_MAP), 0xA0B4);
        ww(m, a.wrapping_add(OB_GFX), 0x444);
        wb(m, a.wrapping_add(OB_RENDER), 4);
        wb(m, a.wrapping_add(OB_PRIORITY), 3);
        wb(m, a.wrapping_add(OB_COLTYPE), 8);
        wb(m, a.wrapping_add(OB_ACTWID), 24);
    } else if rotina != 2 {
        return Err(ErroAbertura::NaoImplementado("rotina do Buzz Bomber"));
    }
    let tempo = rw(m, a.wrapping_add(TEMPO)).wrapping_sub(1);
    ww(m, a.wrapping_add(TEMPO), tempo);
    match rb(m, a.wrapping_add(OB_ROUTINE_2)) {
        0 if i16::from_be_bytes(tempo.to_be_bytes()) < 0 => {
            if rb(m, a.wrapping_add(ESTADO)) & 2 != 0 {
                disparar(m, a);
            } else {
                wb(m, a.wrapping_add(OB_ROUTINE_2), 2);
                ww(m, a.wrapping_add(TEMPO), 127);
                let status = rb(m, a.wrapping_add(OB_STATUS));
                ww(
                    m,
                    a.wrapping_add(OB_VELX),
                    if status & 1 != 0 { 0x400 } else { 0xFC00 },
                );
                wb(m, a.wrapping_add(OB_ANIM), 1);
            }
        }
        0 => {}
        2 if i16::from_be_bytes(tempo.to_be_bytes()) < 0 => {
            wb(m, a.wrapping_add(ESTADO), 0);
            let status = rb(m, a.wrapping_add(OB_STATUS)) ^ 1;
            wb(m, a.wrapping_add(OB_STATUS), status);
            ww(m, a.wrapping_add(TEMPO), 59);
            parar(m, a);
        }
        2 => {
            speed_to_pos(m, a);
            if rb(m, a.wrapping_add(ESTADO)) == 0 {
                let sonic = rw(m, V_PLAYER.wrapping_add(OB_X));
                let x = rw(m, a.wrapping_add(OB_X));
                let distancia =
                    i16::from_be_bytes(sonic.wrapping_sub(x).to_be_bytes()).unsigned_abs();
                if distancia < 96 && rb(m, a.wrapping_add(OB_RENDER)) & 0x80 != 0 {
                    wb(m, a.wrapping_add(ESTADO), 2);
                    ww(m, a.wrapping_add(TEMPO), 29);
                    parar(m, a);
                }
            }
        }
        _ => {
            return Err(ErroAbertura::NaoImplementado(
                "rotina secundaria do Buzz Bomber",
            ));
        }
    }
    animate_sprite(m, a, ANI_BUZZ)?;
    let x = rw(m, a.wrapping_add(OB_X));
    if fora_da_janela(m, x) {
        let n = rb(m, a.wrapping_add(OB_RESPAWNNO));
        if n != 0 {
            let marca = V_OBJSTATE.wrapping_add(2).wrapping_add(u32::from(n));
            let valor = rb(m, marca) & 0x7F;
            wb(m, marca, valor);
        }
        delete_object(m, a);
    } else {
        display_sprite(m, a);
    }
    Ok(())
}

fn parar(m: &mut Memoria, a: u32) {
    wb(m, a.wrapping_add(OB_ROUTINE_2), 0);
    ww(m, a.wrapping_add(OB_VELX), 0);
    wb(m, a.wrapping_add(OB_ANIM), 0);
}

fn disparar(m: &mut Memoria, a: u32) {
    let mut livre = V_LVLOBJSPACE;
    while livre < V_LVLOBJEND && rb(m, livre.wrapping_add(OB_ID)) != 0 {
        livre = livre.wrapping_add(OBJECT_SIZE);
    }
    if livre >= V_LVLOBJEND {
        return;
    }
    let status = rb(m, a.wrapping_add(OB_STATUS));
    let x = rw(m, a.wrapping_add(OB_X));
    let y = rw(m, a.wrapping_add(OB_Y));
    wb(m, livre.wrapping_add(OB_ID), 0x23);
    ww(
        m,
        livre.wrapping_add(OB_X),
        if status & 1 != 0 {
            x.wrapping_add(24)
        } else {
            x.wrapping_sub(24)
        },
    );
    ww(m, livre.wrapping_add(OB_Y), y.wrapping_add(28));
    ww(m, livre.wrapping_add(OB_VELY), 0x200);
    ww(
        m,
        livre.wrapping_add(OB_VELX),
        if status & 1 != 0 { 0x200 } else { 0xFE00 },
    );
    wb(m, livre.wrapping_add(OB_STATUS), status);
    ww(m, livre.wrapping_add(TEMPO), 14);
    wl(m, livre.wrapping_add(PAI), a);
    wb(m, a.wrapping_add(ESTADO), 1);
    ww(m, a.wrapping_add(TEMPO), 59);
    wb(m, a.wrapping_add(OB_ANIM), 2);
}

pub(super) fn missil(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let mut rotina = rb(m, a.wrapping_add(OB_ROUTINE));
    let mut iniciou = false;
    if rotina == 0 {
        let t = rw(m, a.wrapping_add(TEMPO)).wrapping_sub(1);
        ww(m, a.wrapping_add(TEMPO), t);
        if i16::from_be_bytes(t.to_be_bytes()) >= 0 {
            cancelar_se_pai_destruido(m, a);
            return Ok(());
        }
        rotina = if rb(m, a.wrapping_add(OB_SUBTYPE)) != 0 {
            8
        } else {
            2
        };
        iniciou = true;
        wb(m, a.wrapping_add(OB_ROUTINE), rotina);
        wl(m, a.wrapping_add(OB_MAP), 0xA184);
        ww(m, a.wrapping_add(OB_GFX), 0x2444);
        wb(m, a.wrapping_add(OB_RENDER), 4);
        wb(m, a.wrapping_add(OB_PRIORITY), 3);
        wb(m, a.wrapping_add(OB_ACTWID), 8);
        let status = rb(m, a.wrapping_add(OB_STATUS)) & 3;
        wb(m, a.wrapping_add(OB_STATUS), status);
    }
    match rotina {
        2 => {
            cancelar_se_pai_destruido(m, a);
        }
        4 => {
            wb(m, a.wrapping_add(OB_COLTYPE), 0x87);
            wb(m, a.wrapping_add(OB_ANIM), 1);
            speed_to_pos(m, a);
        }
        6 => {
            delete_object(m, a);
            return Ok(());
        }
        8 => {
            if !iniciou && rb(m, a.wrapping_add(OB_RENDER)) & 0x80 == 0 {
                delete_object(m, a);
                return Ok(());
            }
            wb(m, a.wrapping_add(OB_COLTYPE), 0x87);
            wb(m, a.wrapping_add(OB_ANIM), 1);
            speed_to_pos(m, a);
        }
        _ => return Err(ErroAbertura::NaoImplementado("rotina do missil")),
    }
    animate_sprite(m, a, ANI_MISSIL)?;
    display_sprite(m, a);
    if rotina == 4 {
        let limite = rw(m, V_LIMITBTM2).wrapping_add(224);
        if limite < rw(m, a.wrapping_add(OB_Y)) {
            delete_object(m, a);
        }
    }
    Ok(())
}

fn cancelar_se_pai_destruido(m: &mut Memoria, a: u32) {
    let pai = rl(m, a.wrapping_add(PAI));
    if rb(m, pai.wrapping_add(OB_ID)) == 0x27 {
        delete_object(m, a);
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn inicia_movimento_e_atira_com_direcao_do_pai() {
        let mut m = Memoria::nova(vec![0; 0xA200]);
        let pai = V_LVLOBJSPACE;
        wb(&mut m, pai.wrapping_add(OB_ID), 0x22);
        ww(&mut m, pai.wrapping_add(OB_X), 0x200);
        ww(&mut m, pai.wrapping_add(OB_Y), 0x100);
        executar(&mut m, pai).unwrap();
        assert_eq!(rb(&mut m, pai.wrapping_add(OB_ROUTINE_2)), 2);
        assert_eq!(rw(&mut m, pai.wrapping_add(OB_VELX)), 0xFC00);

        wb(&mut m, pai.wrapping_add(OB_ROUTINE_2), 0);
        wb(&mut m, pai.wrapping_add(ESTADO), 2);
        ww(&mut m, pai.wrapping_add(TEMPO), 0);
        executar(&mut m, pai).unwrap();
        let proj = pai.wrapping_add(OBJECT_SIZE);
        assert_eq!(rb(&mut m, proj.wrapping_add(OB_ID)), 0x23);
        assert_eq!(rw(&mut m, proj.wrapping_add(OB_VELX)), 0xFE00);
        assert_eq!(rl(&mut m, proj.wrapping_add(PAI)), pai);
    }
}
