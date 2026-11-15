//! Pilares de vidro da Marble Zone, objeto 0x30.
use super::boot::rom_b;
use super::objetos::{delete_object, display_sprite, fora_da_janela, solid_object};
use super::ram::*;
use super::{ErroAbertura, rb, rl, rw, wb, wl, ww};
use crate::memoria::Memoria;
const ORIG_Y: u32 = 0x30;
const DISTANCIA: u32 = 0x32;
const FLAGS: u32 = 0x34;
const PRIMEIRO: u32 = 0x35;
const RESTANTE: u32 = 0x36;
const ATRASO: u32 = 0x38;
const PAI: u32 = 0x3C;

pub fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    if rb(m, a.wrapping_add(OB_ROUTINE)) == 0 {
        iniciar(m, a);
    }
    let rotina = rb(m, a.wrapping_add(OB_ROUTINE));
    match rotina {
        2 | 6 => {
            mover(m, a)?;
            let altura = if rotina == 2 { 72 } else { 56 };
            solid_object(m, a, 43, altura, altura.wrapping_add(1))?;
        }
        4 | 8 => {
            let pai = rl(m, a.wrapping_add(PAI));
            let distancia = rw(m, pai.wrapping_add(DISTANCIA));
            ww(m, a.wrapping_add(DISTANCIA), distancia);
            if rotina == 8 {
                let y = rw(m, pai.wrapping_add(OB_Y));
                ww(m, a.wrapping_add(ORIG_Y), y);
            }
            mover(m, a)?;
        }
        _ => return Err(ErroAbertura::NaoImplementado("rotina do pilar de vidro")),
    }
    let x = rw(m, a.wrapping_add(OB_X));
    if fora_da_janela(m, x) {
        delete_object(m, a);
    } else {
        display_sprite(m, a);
    }
    Ok(())
}

fn iniciar(m: &mut Memoria, a: u32) {
    let subtipo = rb(m, a.wrapping_add(OB_SUBTYPE));
    wb(
        m,
        a.wrapping_add(OB_HEIGHT),
        if subtipo < 3 { 72 } else { 56 },
    );
    let tabela = if subtipo < 3 { 0xBA66u32 } else { 0xBA6C };
    let x = rw(m, a.wrapping_add(OB_X));
    let y = rw(m, a.wrapping_add(OB_Y));
    let mut slot = a;
    for i in 0..2u32 {
        if i != 0 {
            let mut proximo = a.wrapping_add(OBJECT_SIZE);
            while proximo < V_LVLOBJEND && rb(m, proximo.wrapping_add(OB_ID)) != 0 {
                proximo = proximo.wrapping_add(OBJECT_SIZE);
            }
            if proximo >= V_LVLOBJEND {
                break;
            }
            slot = proximo;
        }
        let entrada = tabela.wrapping_add(i.wrapping_mul(3));
        wb(m, slot.wrapping_add(OB_ROUTINE), rom_b(m, entrada));
        wb(m, slot.wrapping_add(OB_ID), 0x30);
        ww(m, slot.wrapping_add(OB_X), x);
        let desloc = i16::from(i8::from_be_bytes([rom_b(m, entrada.wrapping_add(1))]));
        ww(
            m,
            slot.wrapping_add(OB_Y),
            y.wrapping_add(u16::from_be_bytes(desloc.to_be_bytes())),
        );
        wl(m, slot.wrapping_add(OB_MAP), 0xBC84);
        ww(m, slot.wrapping_add(OB_GFX), 0xC38E);
        wb(m, slot.wrapping_add(OB_RENDER), 4);
        let inicio = rw(m, slot.wrapping_add(OB_Y));
        ww(m, slot.wrapping_add(ORIG_Y), inicio);
        wb(m, slot.wrapping_add(OB_SUBTYPE), subtipo);
        wb(m, slot.wrapping_add(OB_ACTWID), 32);
        wb(m, slot.wrapping_add(OB_PRIORITY), 4);
        wb(
            m,
            slot.wrapping_add(OB_FRAME),
            rom_b(m, entrada.wrapping_add(2)),
        );
        wl(m, slot.wrapping_add(PAI), a);
        if i == 1 {
            wb(m, slot.wrapping_add(OB_ACTWID), 16);
            wb(m, slot.wrapping_add(OB_PRIORITY), 3);
            wb(
                m,
                slot.wrapping_add(OB_SUBTYPE),
                subtipo.wrapping_add(8) & 15,
            );
        }
    }
    ww(m, a.wrapping_add(DISTANCIA), 144);
    let render = rb(m, a.wrapping_add(OB_RENDER)) | 0x10;
    wb(m, a.wrapping_add(OB_RENDER), render);
}

fn mover(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let subtipo = rb(m, a.wrapping_add(OB_SUBTYPE));
    let brilho = subtipo & 8 != 0;
    let deslocamento = match subtipo & 7 {
        0 => return Ok(()),
        1 | 2 => {
            let osc = u16::from(rb(m, V_OSCILLATE.wrapping_add(0x12)));
            let mut d = if subtipo & 7 == 1 {
                osc
            } else {
                64u16.wrapping_sub(osc)
            };
            if brilho {
                d = 64u16.wrapping_sub(d) >> 1;
                d = d.wrapping_add(32);
            }
            d
        }
        3 if brilho => u16::from(rb(m, V_OSCILLATE.wrapping_add(0x12))).wrapping_sub(16),
        3 => {
            esmagar(m, a);
            rw(m, a.wrapping_add(DISTANCIA))
        }
        4 if brilho => u16::from(rb(m, V_OSCILLATE.wrapping_add(0x12))).wrapping_sub(16),
        4 => {
            if rb(m, a.wrapping_add(FLAGS)) == 0
                && rb(m, 0xFF_F7E0u32.wrapping_add(u32::from(subtipo >> 4))) != 0
            {
                wb(m, a.wrapping_add(FLAGS), 1);
            }
            let d = rw(m, a.wrapping_add(DISTANCIA));
            if rb(m, a.wrapping_add(FLAGS)) != 0 && d != 0 {
                ww(m, a.wrapping_add(DISTANCIA), d.wrapping_sub(2));
            }
            rw(m, a.wrapping_add(DISTANCIA))
        }
        _ => return Err(ErroAbertura::NaoImplementado("subtipo do pilar de vidro")),
    };
    let y = rw(m, a.wrapping_add(ORIG_Y)).wrapping_sub(deslocamento);
    ww(m, a.wrapping_add(OB_Y), y);
    Ok(())
}

fn esmagar(m: &mut Memoria, a: u32) {
    let mut flags = rb(m, a.wrapping_add(FLAGS));
    if rb(m, a.wrapping_add(OB_STATUS)) & 8 == 0 {
        flags &= !1;
    } else if flags == 0 {
        flags = 1;
        let anterior = rb(m, a.wrapping_add(PRIMEIRO));
        wb(m, a.wrapping_add(PRIMEIRO), anterior | 1);
        if anterior & 1 != 0 {
            flags |= 0x80;
            let d = if rw(m, a.wrapping_add(DISTANCIA)) == 64 {
                64
            } else {
                16
            };
            ww(m, a.wrapping_add(RESTANTE), d);
            wb(m, a.wrapping_add(ATRASO), 10);
        }
    }
    if flags & 0x80 != 0 {
        let atraso = rb(m, a.wrapping_add(ATRASO));
        let agora = atraso.saturating_sub(1);
        wb(m, a.wrapping_add(ATRASO), agora);
        if agora == 0 {
            let distancia = rw(m, a.wrapping_add(DISTANCIA));
            if distancia == 0 {
                flags &= !0x80;
            } else {
                ww(m, a.wrapping_add(DISTANCIA), distancia.wrapping_sub(1));
                let resta = rw(m, a.wrapping_add(RESTANTE)).wrapping_sub(1);
                ww(m, a.wrapping_add(RESTANTE), resta);
                if resta == 0 {
                    flags &= !0x80;
                }
            }
        }
    }
    wb(m, a.wrapping_add(FLAGS), flags);
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn pilar_de_chave_desce_ate_limite() {
        let mut m = Memoria::nova(vec![0; 0x10000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 0x24);
        ww(&mut m, a.wrapping_add(ORIG_Y), 300);
        ww(&mut m, a.wrapping_add(DISTANCIA), 4);
        wb(&mut m, 0xFF_F7E2, 1);
        mover(&mut m, a).unwrap();
        assert_eq!(rw(&mut m, a.wrapping_add(DISTANCIA)), 2);
        assert_eq!(rw(&mut m, a.wrapping_add(OB_Y)), 298);
        mover(&mut m, a).unwrap();
        assert_eq!(rw(&mut m, a.wrapping_add(OB_Y)), 300);
    }
}
