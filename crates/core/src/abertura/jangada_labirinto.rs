//! Bloco movel e jangada secreta de Labyrinth, objeto 0x52 (REV01).
use super::super::fisica::{obj_floor_dist, obj_hit_wall, speed_to_pos};
use super::super::plataforma_oscilante::{detectar_altura, mover_sonic_altura, sair};
use super::super::ram::*;
use super::super::{ErroAbertura, rb, rw, wb, wl, ww};
use super::{delete_object, display_sprite, fora_da_janela};
use crate::memoria::Memoria;

const ORIG_X: u32 = 0x30;
const ORIG_Y: u32 = 0x32;
const SLIDE_WAIT: u32 = 0x34;
const SLIDE_GOBACK: u32 = 0x36;
const SWITCH_2: u32 = 0xFF_F7E2;

// MBlock_Var: (largura_solida, frame)
const MBLOCK_VAR: [(u8, u8); 5] = [
    (16, 0), // $0x: MZ bloco unico / LZ jangada pequena
    (32, 1), // $1x: MZ bloco duplo
    (32, 2), // $2x: SBZ curto (listras amarelas/pretas)
    (64, 3), // $3x: SBZ longo (piso deslizante vermelho)
    (48, 4), // $4x: MZ bloco triplo
];

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let mut rotina = rb(m, a.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        rotina = 2;
        wb(m, a.wrapping_add(OB_ROUTINE), rotina);
        wl(m, a.wrapping_add(OB_MAP), 0x106D6);
        ww(m, a.wrapping_add(OB_GFX), 0x42B8);

        let zona = rb(m, V_ZONE);
        let subtipo_bruto = rb(m, a.wrapping_add(OB_SUBTYPE));

        if zona == ID_LZ {
            wl(m, a.wrapping_add(OB_MAP), 0x1072C);
            ww(m, a.wrapping_add(OB_GFX), 0x43BC);
            wb(m, a.wrapping_add(OB_HEIGHT), 7);
        } else if zona == ID_SBZ {
            if subtipo_bruto == 0x28 {
                ww(m, a.wrapping_add(OB_GFX), 0x22C0);
            } else {
                ww(m, a.wrapping_add(OB_GFX), 0x4460);
            }
        }

        wb(m, a.wrapping_add(OB_RENDER), 4);

        let var_idx = usize::from((subtipo_bruto >> 4) & 7);
        let (largura, frame) = if var_idx < MBLOCK_VAR.len() {
            MBLOCK_VAR[var_idx]
        } else {
            MBLOCK_VAR[0]
        };
        wb(m, a.wrapping_add(OB_ACTWID), largura);
        wb(m, a.wrapping_add(OB_FRAME), frame);

        wb(m, a.wrapping_add(OB_PRIORITY), 4);
        let x = rw(m, a.wrapping_add(OB_X));
        let y = rw(m, a.wrapping_add(OB_Y));
        ww(m, a.wrapping_add(ORIG_X), x);
        ww(m, a.wrapping_add(ORIG_Y), y);
        let tipo = subtipo_bruto & 15;
        wb(m, a.wrapping_add(OB_SUBTYPE), tipo);
    }
    match rotina {
        2 => {
            if !mover(m, a)? {
                return Ok(());
            }
            detectar_altura(m, a, 8);
        }
        4 => {
            sair(m, a);
            let anterior = rw(m, a.wrapping_add(OB_X));
            if !mover(m, a)? {
                return Ok(());
            }
            mover_sonic_altura(m, a, anterior, 9);
        }
        _ => {
            return Err(ErroAbertura::NaoImplementado(
                "rotina do bloco movel",
            ));
        }
    }
    let x = rw(m, a.wrapping_add(ORIG_X));
    if fora_da_janela(m, x) {
        delete_object(m, a);
    } else {
        display_sprite(m, a);
    }
    Ok(())
}

// Devolve falso quando a jangada secreta deve permanecer invisivel neste quadro.
fn mover(m: &mut Memoria, a: u32) -> Result<bool, ErroAbertura> {
    let tipo = rb(m, a.wrapping_add(OB_SUBTYPE)) & 15;
    match tipo {
        0 => {}
        1 | 8 => {
            let horizontal = tipo == 1;
            let (oscilador, limite, origem, campo) = if horizontal {
                (V_OSCILLATE.wrapping_add(0x0e), 0x60u16, ORIG_X, OB_X)
            } else {
                (V_OSCILLATE.wrapping_add(0x1e), 0x80u16, ORIG_Y, OB_Y)
            };
            let mut desloc = u16::from(rb(m, oscilador));
            if rb(m, a.wrapping_add(OB_STATUS)) & 1 != 0 {
                desloc = limite.wrapping_sub(desloc);
            }
            let pos = rw(m, a.wrapping_add(origem)).wrapping_sub(desloc);
            ww(m, a.wrapping_add(campo), pos);
        }
        2 | 4 | 9 => {
            if rb(m, a.wrapping_add(OB_ROUTINE)) == 4 {
                wb(m, a.wrapping_add(OB_SUBTYPE), tipo.wrapping_add(1));
            }
        }
        3 | 5 => {
            let largura = u16::from(rb(m, a.wrapping_add(OB_ACTWID)));
            let distancia = obj_hit_wall(m, a, largura, false);
            if i16::from_be_bytes(distancia.to_be_bytes()) < 0 {
                wb(m, a.wrapping_add(OB_SUBTYPE), if tipo == 3 { 0 } else { 6 });
            } else {
                let x = rw(m, a.wrapping_add(OB_X)).wrapping_add(1);
                ww(m, a.wrapping_add(OB_X), x);
                ww(m, a.wrapping_add(ORIG_X), x);
            }
        }
        6 => {
            speed_to_pos(m, a);
            let vel = rw(m, a.wrapping_add(OB_VELY)).wrapping_add(0x18);
            ww(m, a.wrapping_add(OB_VELY), vel);
            let distancia = obj_floor_dist(m, a);
            if i16::from_be_bytes(distancia.to_be_bytes()) < 0 {
                let y = rw(m, a.wrapping_add(OB_Y)).wrapping_add(distancia);
                ww(m, a.wrapping_add(OB_Y), y);
                ww(m, a.wrapping_add(OB_VELY), 0);
                wb(m, a.wrapping_add(OB_SUBTYPE), 0);
            }
        }
        7 => {
            if rb(m, SWITCH_2) != 0 {
                wb(m, a.wrapping_add(OB_SUBTYPE), 4);
            }
            let x = rw(m, a.wrapping_add(ORIG_X));
            if fora_da_janela(m, x) {
                delete_object(m, a);
            }
            return Ok(false);
        }
        10 => {
            let largura = u16::from(rb(m, a.wrapping_add(OB_ACTWID)));
            let total_dist = largura.wrapping_mul(2);
            let xflip = rb(m, a.wrapping_add(OB_STATUS)) & 1 != 0;
            let (passo, alvo_diff) = if xflip {
                (0xFFF8u16, (0u16).wrapping_sub(total_dist))
            } else {
                (8u16, total_dist)
            };
            let voltando = rw(m, a.wrapping_add(SLIDE_GOBACK)) != 0;
            let x = rw(m, a.wrapping_add(OB_X));
            let orig_x = rw(m, a.wrapping_add(ORIG_X));
            let diff = x.wrapping_sub(orig_x);
            if voltando {
                if diff == 0 {
                    ww(m, a.wrapping_add(SLIDE_GOBACK), 0);
                    let sub = rb(m, a.wrapping_add(OB_SUBTYPE)).wrapping_sub(1);
                    wb(m, a.wrapping_add(OB_SUBTYPE), sub);
                } else {
                    ww(m, a.wrapping_add(OB_X), x.wrapping_sub(passo));
                }
            } else if diff == alvo_diff {
                let espera = rw(m, a.wrapping_add(SLIDE_WAIT)).wrapping_sub(1);
                ww(m, a.wrapping_add(SLIDE_WAIT), espera);
                if espera == 0 {
                    ww(m, a.wrapping_add(SLIDE_GOBACK), 1);
                }
            } else {
                ww(m, a.wrapping_add(OB_X), x.wrapping_add(passo));
                ww(m, a.wrapping_add(SLIDE_WAIT), 5 * 60);
            }
        }
        _ => {
            return Err(ErroAbertura::NaoImplementado(
                "tipo de bloco movel",
            ));
        }
    }
    Ok(true)
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn jangada_secreta_aparece_no_quadro_apos_chave() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, V_ZONE, ID_LZ);
        wb(&mut m, a.wrapping_add(OB_ID), 0x52);
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 7);
        ww(&mut m, a.wrapping_add(OB_X), 0x200);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_SUBTYPE)), 7);
        assert_eq!(rw(&mut m, V_SPRITEQUEUE.wrapping_add(4 * 0x80)), 0);
        wb(&mut m, SWITCH_2, 1);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_SUBTYPE)), 4);
        assert_eq!(rw(&mut m, V_SPRITEQUEUE.wrapping_add(4 * 0x80)), 0);
        executar(&mut m, a).unwrap();
        assert_ne!(rw(&mut m, V_SPRITEQUEUE.wrapping_add(4 * 0x80)), 0);
    }
    #[test]
    fn jangada_armada_comeca_a_mover_quando_sonic_sobe() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 4);
        wb(&mut m, a.wrapping_add(OB_ROUTINE), 4);
        assert!(mover(&mut m, a).unwrap());
        assert_eq!(rb(&mut m, a.wrapping_add(OB_SUBTYPE)), 5);
    }
}
