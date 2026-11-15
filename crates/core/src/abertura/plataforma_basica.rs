//! Plataformas móveis básicas, objeto 18.
use super::fisica::calc_sine;
use super::objetos::{delete_object, display_sprite, fora_da_janela};
use super::plataforma_oscilante::{detectar_altura, mover_sonic_altura, sair};
use super::ram::*;
use super::{ErroAbertura, rb, rl, rw, wb, wl, ww};
use crate::memoria::Memoria;

const Y_BRUTO: u32 = 0x2C;
const X_ORIGEM: u32 = 0x32;
const Y_ORIGEM: u32 = 0x34;
const PRESSAO: u32 = 0x38;
const TEMPO: u32 = 0x3A;

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a.wrapping_add(OB_ROUTINE));
    if rotina == 6 {
        delete_object(m, a);
        return Ok(());
    }
    if rotina == 0 {
        wb(m, a.wrapping_add(OB_ROUTINE), 2);
        ww(m, a.wrapping_add(OB_GFX), 0x4000);
        let zona = rb(m, V_ZONE);
        let mapa = if zona == 4 {
            0x8806
        } else if zona == 3 {
            0x8818
        } else {
            0x87BA
        };
        wl(m, a.wrapping_add(OB_MAP), mapa);
        wb(m, a.wrapping_add(OB_ACTWID), 32);
        if zona == 3 {
            wb(m, a.wrapping_add(OB_SUBTYPE), 3);
        }
        wb(m, a.wrapping_add(OB_RENDER), 4);
        wb(m, a.wrapping_add(OB_PRIORITY), 4);
        let y = rw(m, a.wrapping_add(OB_Y));
        let x = rw(m, a.wrapping_add(OB_X));
        ww(m, a.wrapping_add(Y_BRUTO), y);
        ww(m, a.wrapping_add(Y_ORIGEM), y);
        ww(m, a.wrapping_add(X_ORIGEM), x);
        ww(m, a.wrapping_add(OB_ANGLE), 0x80);
        let frame = u8::from(rb(m, a.wrapping_add(OB_SUBTYPE)) == 10);
        wb(m, a.wrapping_add(OB_FRAME), frame);
    }
    let rotina = rb(m, a.wrapping_add(OB_ROUTINE));
    match rotina {
        2 => {
            let pressao = rb(m, a.wrapping_add(PRESSAO));
            if pressao != 0 {
                wb(m, a.wrapping_add(PRESSAO), pressao.wrapping_sub(4));
            }
            detectar_altura(m, a, 8);
        }
        4 => {
            let pressao = rb(m, a.wrapping_add(PRESSAO));
            if pressao != 0x40 {
                wb(m, a.wrapping_add(PRESSAO), pressao.wrapping_add(4));
            }
            sair(m, a);
        }
        8 => {}
        _ => return Err(ErroAbertura::NaoImplementado("rotina da plataforma basica")),
    }
    let anterior = rw(m, a.wrapping_add(OB_X));
    mover(m, a)?;
    let angulo = rb(m, a.wrapping_add(PRESSAO));
    let (seno, _) = calc_sine(m, angulo);
    let deslocamento = i32::from(seno).wrapping_mul(0x400) >> 16;
    let deslocamento =
        u16::try_from(u32::from_be_bytes(deslocamento.to_be_bytes()) & 0xFFFF).unwrap_or(0);
    let y = rw(m, a.wrapping_add(Y_BRUTO)).wrapping_add(deslocamento);
    ww(m, a.wrapping_add(OB_Y), y);
    if rotina == 4 {
        mover_sonic_altura(m, a, anterior, 9);
    }
    display_sprite(m, a);
    let origem = rw(m, a.wrapping_add(X_ORIGEM));
    if fora_da_janela(m, origem) {
        delete_object(m, a);
    }
    Ok(())
}

fn mover(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let tipo = rb(m, a.wrapping_add(OB_SUBTYPE)) & 15;
    match tipo {
        0 | 9 => {}
        1 | 2 | 5 | 6 | 10 | 11 | 12 => {
            let horizontal = tipo == 1 || tipo == 5;
            let reverso = tipo == 5 || tipo == 6 || tipo == 12;
            let lento = tipo == 11 || tipo == 12;
            let valor = if lento {
                rb(m, 0xFF_FE6C)
            } else {
                rb(m, a.wrapping_add(OB_ANGLE))
            };
            let centro = if lento { 0x30u8 } else { 0x40u8 };
            let delta = if reverso {
                centro.wrapping_sub(valor)
            } else {
                valor.wrapping_sub(centro)
            };
            let mut delta = i16::from(i8::from_be_bytes([delta]));
            if tipo == 10 {
                delta >>= 1;
            }
            let origem = if horizontal { X_ORIGEM } else { Y_ORIGEM };
            let alvo = if horizontal { OB_X } else { Y_BRUTO };
            let pos =
                rw(m, a.wrapping_add(origem)).wrapping_add(u16::from_be_bytes(delta.to_be_bytes()));
            ww(m, a.wrapping_add(alvo), pos);
            let oscilacao = rb(m, 0xFF_FE78);
            wb(m, a.wrapping_add(OB_ANGLE), oscilacao);
        }
        3 => {
            let tempo = rw(m, a.wrapping_add(TEMPO));
            if tempo == 0 {
                if rb(m, a.wrapping_add(OB_STATUS)) & STATUS_ON_OBJECT != 0 {
                    ww(m, a.wrapping_add(TEMPO), 30);
                }
            } else {
                let tempo = tempo.wrapping_sub(1);
                ww(m, a.wrapping_add(TEMPO), tempo);
                if tempo == 0 {
                    ww(m, a.wrapping_add(TEMPO), 32);
                    let subtipo = rb(m, a.wrapping_add(OB_SUBTYPE)).wrapping_add(1);
                    wb(m, a.wrapping_add(OB_SUBTYPE), subtipo);
                }
            }
        }
        4 => {
            let tempo = rw(m, a.wrapping_add(TEMPO));
            if tempo != 0 {
                let tempo = tempo.wrapping_sub(1);
                ww(m, a.wrapping_add(TEMPO), tempo);
                if tempo == 0 {
                    let status = rb(m, a.wrapping_add(OB_STATUS));
                    if status & STATUS_ON_OBJECT != 0 {
                        let ps = (rb(m, V_PLAYER.wrapping_add(OB_STATUS)) | STATUS_IN_AIR)
                            & !STATUS_ON_OBJECT;
                        wb(m, V_PLAYER.wrapping_add(OB_STATUS), ps);
                        wb(m, V_PLAYER.wrapping_add(OB_ROUTINE), 2);
                        wb(m, a.wrapping_add(OB_STATUS), status & !STATUS_ON_OBJECT);
                        wb(m, a.wrapping_add(0x25), 0);
                        let vy = rw(m, a.wrapping_add(OB_VELY));
                        ww(m, V_PLAYER.wrapping_add(OB_VELY), vy);
                    }
                    wb(m, a.wrapping_add(OB_ROUTINE), 8);
                }
            }
            let velocidade = i32::from(i16::from_be_bytes(
                rw(m, a.wrapping_add(OB_VELY)).to_be_bytes(),
            ))
            .wrapping_shl(8);
            let y = rl(m, a.wrapping_add(Y_BRUTO))
                .wrapping_add(u32::from_be_bytes(velocidade.to_be_bytes()));
            wl(m, a.wrapping_add(Y_BRUTO), y);
            let vy = rw(m, a.wrapping_add(OB_VELY)).wrapping_add(0x38);
            ww(m, a.wrapping_add(OB_VELY), vy);
            if rw(m, a.wrapping_add(Y_BRUTO)) > rw(m, V_LIMITBTM2).wrapping_add(224) {
                wb(m, a.wrapping_add(OB_ROUTINE), 6);
            }
        }
        7 => {
            let tempo = rw(m, a.wrapping_add(TEMPO));
            if tempo == 0 {
                let subtipo = rb(m, a.wrapping_add(OB_SUBTYPE));
                if rb(m, 0xFF_F7E0u32.wrapping_add(u32::from(subtipo >> 4))) != 0 {
                    ww(m, a.wrapping_add(TEMPO), 60);
                }
            } else {
                let tempo = tempo.wrapping_sub(1);
                ww(m, a.wrapping_add(TEMPO), tempo);
                if tempo == 0 {
                    let subtipo = rb(m, a.wrapping_add(OB_SUBTYPE)).wrapping_add(1);
                    wb(m, a.wrapping_add(OB_SUBTYPE), subtipo);
                }
            }
        }
        8 => {
            let y = rw(m, a.wrapping_add(Y_BRUTO)).wrapping_sub(2);
            ww(m, a.wrapping_add(Y_BRUTO), y);
            if y == rw(m, a.wrapping_add(Y_ORIGEM)).wrapping_sub(0x200) {
                wb(m, a.wrapping_add(OB_SUBTYPE), 0);
            }
        }
        _ => return Err(ErroAbertura::NaoImplementado("tipo da plataforma basica")),
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn desloca_horizontal_e_atualiza_movimento_no_proximo_quadro() {
        let mut m = Memoria::nova(vec![0; 0x10000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x18);
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 1);
        ww(&mut m, a.wrapping_add(OB_X), 0x100);
        ww(&mut m, a.wrapping_add(OB_Y), 0x100);
        wb(&mut m, 0xFF_FE78, 0x30);
        executar(&mut m, a).unwrap();
        assert_eq!(rw(&mut m, a.wrapping_add(OB_X)), 0xC0);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ANGLE)), 0x30);
        executar(&mut m, a).unwrap();
        assert_eq!(rw(&mut m, a.wrapping_add(OB_X)), 0xF0);
    }

    #[test]
    fn temporizador_inicia_queda_depois_de_ser_pisada() {
        let mut m = Memoria::nova(vec![0; 0x10000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 3);
        wb(&mut m, a.wrapping_add(OB_STATUS), STATUS_ON_OBJECT);
        mover(&mut m, a).unwrap();
        assert_eq!(rw(&mut m, a.wrapping_add(TEMPO)), 30);
        for _ in 0..30 {
            mover(&mut m, a).unwrap();
        }
        assert_eq!(rb(&mut m, a.wrapping_add(OB_SUBTYPE)), 4);
        assert_eq!(rw(&mut m, a.wrapping_add(TEMPO)), 32);
    }
}
