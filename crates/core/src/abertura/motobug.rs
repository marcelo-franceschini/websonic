//! MotoBug da Green Hill: preparação, patrulha e fumaça.
use super::fisica::{obj_floor_dist, speed_to_pos};
use super::objetos::{animate_sprite, delete_object, display_sprite, fora_da_janela};
use super::ram::*;
use super::{ErroAbertura, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;

const MAP_MOTO: u32 = 0xFE2C;
const ANI_MOTO: u32 = 0xFE0E;
const MOTO_ESPERA: u32 = 0x30;
const MOTO_FUMACA: u32 = 0x33;

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            wl(m, a.wrapping_add(OB_MAP), MAP_MOTO);
            ww(m, a.wrapping_add(OB_GFX), 0x4F0);
            wb(m, a.wrapping_add(OB_RENDER), 4);
            wb(m, a.wrapping_add(OB_PRIORITY), 4);
            wb(m, a.wrapping_add(OB_ACTWID), 20);
            if rb(m, a.wrapping_add(OB_ANIM)) != 0 {
                wb(m, a.wrapping_add(OB_ROUTINE), 4);
                return fumaca(m, a);
            }
            wb(m, a.wrapping_add(OB_HEIGHT), 14);
            wb(m, a.wrapping_add(OB_WIDTH), 8);
            wb(m, a.wrapping_add(OB_COLTYPE), 12);
            speed_to_pos(m, a);
            let vel = rw(m, a.wrapping_add(OB_VELY)).wrapping_add(0x38);
            ww(m, a.wrapping_add(OB_VELY), vel);
            let distancia = obj_floor_dist(m, a);
            if i16::from_be_bytes(distancia.to_be_bytes()) < 0 {
                let y = rw(m, a.wrapping_add(OB_Y)).wrapping_add(distancia);
                ww(m, a.wrapping_add(OB_Y), y);
                ww(m, a.wrapping_add(OB_VELY), 0);
                wb(m, a.wrapping_add(OB_ROUTINE), 2);
                let status = rb(m, a.wrapping_add(OB_STATUS)) ^ 1;
                wb(m, a.wrapping_add(OB_STATUS), status);
            }
            Ok(())
        }
        2 => {
            match rb(m, a.wrapping_add(OB_ROUTINE_2)) {
                0 => {
                    let espera = rw(m, a.wrapping_add(MOTO_ESPERA)).wrapping_sub(1);
                    ww(m, a.wrapping_add(MOTO_ESPERA), espera);
                    if i16::from_be_bytes(espera.to_be_bytes()) < 0 {
                        wb(m, a.wrapping_add(OB_ROUTINE_2), 2);
                        wb(m, a.wrapping_add(OB_ANIM), 1);
                        let status = rb(m, a.wrapping_add(OB_STATUS)) ^ 1;
                        wb(m, a.wrapping_add(OB_STATUS), status);
                        ww(
                            m,
                            a.wrapping_add(OB_VELX),
                            if status & 1 == 0 { 0xFF00 } else { 0x100 },
                        );
                    }
                }
                2 => {
                    speed_to_pos(m, a);
                    let distancia = obj_floor_dist(m, a);
                    let d = i16::from_be_bytes(distancia.to_be_bytes());
                    if !(-8..12).contains(&d) {
                        wb(m, a.wrapping_add(OB_ROUTINE_2), 0);
                        ww(m, a.wrapping_add(MOTO_ESPERA), 59);
                        ww(m, a.wrapping_add(OB_VELX), 0);
                        wb(m, a.wrapping_add(OB_ANIM), 0);
                    } else {
                        let y = rw(m, a.wrapping_add(OB_Y)).wrapping_add(distancia);
                        ww(m, a.wrapping_add(OB_Y), y);
                        let espera = rb(m, a.wrapping_add(MOTO_FUMACA)).wrapping_sub(1);
                        wb(m, a.wrapping_add(MOTO_FUMACA), espera);
                        if espera & 0x80 != 0 {
                            wb(m, a.wrapping_add(MOTO_FUMACA), 15);
                            criar_fumaca(m, a);
                        }
                    }
                }
                _ => {
                    return Err(ErroAbertura::NaoImplementado(
                        "rotina secundaria do MotoBug",
                    ));
                }
            }
            animate_sprite(m, a, ANI_MOTO)?;
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
        4 => fumaca(m, a),
        6 => {
            delete_object(m, a);
            Ok(())
        }
        _ => Err(ErroAbertura::NaoImplementado("rotina do MotoBug")),
    }
}

fn fumaca(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    animate_sprite(m, a, ANI_MOTO)?;
    display_sprite(m, a);
    Ok(())
}

fn criar_fumaca(m: &mut Memoria, origem: u32) {
    let mut destino = V_LVLOBJSPACE;
    while destino < V_LVLOBJEND {
        if rb(m, destino.wrapping_add(OB_ID)) == 0 {
            wb(m, destino.wrapping_add(OB_ID), 0x40);
            let x = rw(m, origem.wrapping_add(OB_X));
            let y = rw(m, origem.wrapping_add(OB_Y));
            let status = rb(m, origem.wrapping_add(OB_STATUS));
            ww(m, destino.wrapping_add(OB_X), x);
            ww(m, destino.wrapping_add(OB_Y), y);
            wb(m, destino.wrapping_add(OB_STATUS), status);
            wb(m, destino.wrapping_add(OB_ANIM), 2);
            break;
        }
        destino = destino.wrapping_add(OBJECT_SIZE);
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn espera_terminada_inverte_direcao_e_inicia_patrulha() {
        for (status, velocidade) in [(1, 0xFF00), (0, 0x0100)] {
            let mut m = Memoria::nova(vec![0; 0x10000]);
            let a = V_LVLOBJSPACE;
            wb(&mut m, a.wrapping_add(OB_ROUTINE), 2);
            wb(&mut m, a.wrapping_add(OB_STATUS), status);
            executar(&mut m, a).unwrap();
            assert_eq!(rw(&mut m, a.wrapping_add(OB_VELX)), velocidade);
            assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE_2)), 2);
        }
    }
}
