//! Blocos de Labyrinth, objeto 61 (`61 LZ Blocks.asm`).
use super::super::fisica::{calc_sine, obj_floor_dist, obj_hit_ceiling, speed_to_pos};
use super::super::ram::*;
use super::super::{ErroAbertura, rb, rw, wb, wl, ww};
use super::{delete_object, display_sprite, fora_da_janela, solid_object};
use crate::memoria::Memoria;

const ORIG_Y: u32 = 0x30;
const ORIG_X: u32 = 0x34;
const TIMER: u32 = 0x36;
const UNTOUCHED: u32 = 0x38;
const NUDGE: u32 = 0x3e;
const TOUCH: u32 = 0x3f;
const WATERPOS: u32 = 0xFF_F646;

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            wb(m, a.wrapping_add(OB_ROUTINE), 2);
            wl(m, a.wrapping_add(OB_MAP), 0x12896);
            ww(m, a.wrapping_add(OB_GFX), 0x43e6);
            wb(m, a.wrapping_add(OB_RENDER), 4);
            wb(m, a.wrapping_add(OB_PRIORITY), 3);
            let subtipo = rb(m, a.wrapping_add(OB_SUBTYPE));
            let forma = (subtipo >> 4) & 7;
            let (largura, altura) = match forma {
                0 | 2 | 3 => (16, 16),
                1 => (32, 12),
                _ => return Err(ErroAbertura::NaoImplementado("forma do bloco de Labyrinth")),
            };
            wb(m, a.wrapping_add(OB_ACTWID), largura);
            wb(m, a.wrapping_add(OB_HEIGHT), altura);
            wb(m, a.wrapping_add(OB_FRAME), forma);
            let x = rw(m, a.wrapping_add(OB_X));
            let y = rw(m, a.wrapping_add(OB_Y));
            ww(m, a.wrapping_add(ORIG_X), x);
            ww(m, a.wrapping_add(ORIG_Y), y);
            if subtipo & 15 != 0 && subtipo & 15 != 7 {
                wb(m, a.wrapping_add(UNTOUCHED), 1);
            }
        }
        2 => {}
        _ => {
            return Err(ErroAbertura::NaoImplementado(
                "rotina do bloco de Labyrinth",
            ));
        }
    }
    match rb(m, a.wrapping_add(OB_SUBTYPE)) & 15 {
        0 => {}
        1 | 3 => {
            let tempo = rw(m, a.wrapping_add(TIMER));
            if tempo != 0 {
                let proximo = tempo.wrapping_sub(1);
                ww(m, a.wrapping_add(TIMER), proximo);
                if proximo == 0 {
                    avancar(m, a);
                }
            } else if rb(m, a.wrapping_add(OB_STATUS)) & 8 != 0 {
                ww(m, a.wrapping_add(TIMER), 30);
            }
        }
        2 | 6 => {
            speed_to_pos(m, a);
            let vel = rw(m, a.wrapping_add(OB_VELY)).wrapping_add(8);
            ww(m, a.wrapping_add(OB_VELY), vel);
            let distancia = obj_floor_dist(m, a);
            if i16::from_be_bytes(distancia.to_be_bytes()) < 0 {
                let y = rw(m, a.wrapping_add(OB_Y))
                    .wrapping_add(distancia)
                    .wrapping_add(1);
                ww(m, a.wrapping_add(OB_Y), y);
                ww(m, a.wrapping_add(OB_VELY), 0);
                wb(m, a.wrapping_add(OB_SUBTYPE), 0);
            }
        }
        4 => {
            speed_to_pos(m, a);
            let vel = rw(m, a.wrapping_add(OB_VELY)).wrapping_sub(8);
            ww(m, a.wrapping_add(OB_VELY), vel);
            let distancia = obj_hit_ceiling(m, a);
            if i16::from_be_bytes(distancia.to_be_bytes()) < 0 {
                let y = rw(m, a.wrapping_add(OB_Y)).wrapping_sub(distancia);
                ww(m, a.wrapping_add(OB_Y), y);
                ww(m, a.wrapping_add(OB_VELY), 0);
                wb(m, a.wrapping_add(OB_SUBTYPE), 0);
            }
        }
        5 => {
            if rb(m, a.wrapping_add(TOUCH)) == 1 {
                avancar(m, a);
            }
        }
        7 => {
            let agua = rw(m, WATERPOS);
            let y = rw(m, a.wrapping_add(OB_Y));
            if agua != y {
                let descendo = agua >= y;
                let delta = i16::from_be_bytes(agua.wrapping_sub(y).to_be_bytes());
                let passo = if descendo {
                    delta.min(2)
                } else {
                    delta.max(-2)
                };
                ww(
                    m,
                    a.wrapping_add(OB_Y),
                    y.wrapping_add(u16::from_be_bytes(passo.to_be_bytes())),
                );
                let distancia = if descendo {
                    obj_floor_dist(m, a)
                } else {
                    obj_hit_ceiling(m, a)
                };
                if i16::from_be_bytes(distancia.to_be_bytes()) < 0 {
                    let y = rw(m, a.wrapping_add(OB_Y));
                    ww(
                        m,
                        a.wrapping_add(OB_Y),
                        if descendo {
                            y.wrapping_add(distancia).wrapping_add(1)
                        } else {
                            y.wrapping_sub(distancia)
                        },
                    );
                }
            }
        }
        _ => return Err(ErroAbertura::NaoImplementado("tipo do bloco de Labyrinth")),
    }
    if rb(m, a.wrapping_add(OB_RENDER)) & 0x80 != 0 {
        let largura = u16::from(rb(m, a.wrapping_add(OB_ACTWID))).wrapping_add(11);
        let altura = u16::from(rb(m, a.wrapping_add(OB_HEIGHT)));
        let toque = solid_object(m, a, largura, altura, altura.wrapping_add(1))?;
        wb(m, a.wrapping_add(TOUCH), toque.to_be_bytes()[0]);
        nudge(m, a);
    }
    let x = rw(m, a.wrapping_add(ORIG_X));
    if fora_da_janela(m, x) {
        delete_object(m, a);
    } else {
        display_sprite(m, a);
    }
    Ok(())
}

fn avancar(m: &mut Memoria, a: u32) {
    let subtipo = rb(m, a.wrapping_add(OB_SUBTYPE));
    wb(m, a.wrapping_add(OB_SUBTYPE), subtipo.wrapping_add(1));
    wb(m, a.wrapping_add(UNTOUCHED), 0);
}

fn nudge(m: &mut Memoria, a: u32) {
    if rb(m, a.wrapping_add(UNTOUCHED)) == 0 {
        return;
    }
    let atual = rb(m, a.wrapping_add(NUDGE));
    let novo = if rb(m, a.wrapping_add(OB_STATUS)) & 8 != 0 {
        if atual == 0x40 {
            return;
        }
        atual.wrapping_add(4)
    } else {
        if atual == 0 {
            return;
        }
        atual.wrapping_sub(4)
    };
    wb(m, a.wrapping_add(NUDGE), novo);
    let (seno, _) = calc_sine(m, novo);
    let desloc = i32::from(seno).wrapping_mul(0x400) >> 16;
    let y = rw(m, a.wrapping_add(ORIG_Y));
    let bytes = desloc.to_be_bytes();
    ww(
        m,
        a.wrapping_add(OB_Y),
        y.wrapping_add(u16::from_be_bytes([bytes[2], bytes[3]])),
    );
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn primeiro_bloco_da_lz1_inicializa_como_solido_estacionario() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a + OB_ID, 0x61);
        wb(&mut m, a + OB_SUBTYPE, 0x30);
        ww(&mut m, a + OB_X, 0x70);
        ww(&mut m, a + OB_Y, 0x200);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a + OB_ROUTINE), 2);
        assert_eq!(rb(&mut m, a + OB_FRAME), 3);
        assert_eq!(rb(&mut m, a + OB_ACTWID), 16);
        assert_eq!(rw(&mut m, a + OB_Y), 0x200);
    }
    #[test]
    fn plataforma_espera_trinta_quadros_antes_de_subir() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a + OB_ID, 0x61);
        wb(&mut m, a + OB_SUBTYPE, 0x13);
        wb(&mut m, a + OB_STATUS, 8);
        executar(&mut m, a).unwrap();
        assert_eq!(rw(&mut m, a + TIMER), 30);
        for _ in 0..30 {
            executar(&mut m, a).unwrap();
        }
        assert_eq!(rb(&mut m, a + OB_SUBTYPE), 0x14);
        assert_eq!(rb(&mut m, a + UNTOUCHED), 0);
    }
}
