//! Botões e contato com o bloco da Marble Zone, objeto 0x32.
use super::objetos::{delete_object, display_sprite, fora_da_janela, solid_object};
use super::ram::*;
use super::{ErroAbertura, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;

const MAP_BOTAO: u32 = 0xC52E;
const F_SWITCH: u32 = 0xFF_F7E0;

pub fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            wb(m, a.wrapping_add(OB_ROUTINE), 2);
            wl(m, a.wrapping_add(OB_MAP), MAP_BOTAO);
            let arte = if rb(m, V_ZONE) == ID_MZ {
                0x4513
            } else {
                0x0513
            };
            ww(m, a.wrapping_add(OB_GFX), arte);
            wb(m, a.wrapping_add(OB_RENDER), 4);
            wb(m, a.wrapping_add(OB_ACTWID), 16);
            wb(m, a.wrapping_add(OB_PRIORITY), 4);
            let y = rw(m, a.wrapping_add(OB_Y)).wrapping_add(3);
            ww(m, a.wrapping_add(OB_Y), y);
        }
        2 => {}
        _ => return Err(ErroAbertura::NaoImplementado("rotina do botão")),
    }
    if rb(m, a.wrapping_add(OB_RENDER)) & 0x80 != 0 {
        solid_object(m, a, 27, 5, 5)?;
        let subtipo = rb(m, a.wrapping_add(OB_SUBTYPE));
        let chave = F_SWITCH.wrapping_add(u32::from(subtipo & 0x0F));
        let mascara = if subtipo & 0x40 != 0 { 0x80 } else { 1 };
        let pressionado = (subtipo & 0x80 != 0 && bloco_sobre_botao(m, a))
            || rb(m, a.wrapping_add(OB_ROUTINE_2)) != 0;
        let estado = rb(m, chave);
        let mut quadro = rb(m, a.wrapping_add(OB_FRAME)) & !1;
        if pressionado {
            if estado == 0 {
                wb(m, V_SOUNDQUEUE1, SFX_SWITCH);
            }
            wb(m, chave, estado | mascara);
            quadro |= 1;
        } else {
            wb(m, chave, estado & !mascara);
        }
        if subtipo & 0x20 != 0 {
            let tempo = rb(m, a.wrapping_add(OB_TIME_FRAME)).wrapping_sub(1);
            wb(m, a.wrapping_add(OB_TIME_FRAME), tempo);
            if i8::from_be_bytes([tempo]) < 0 {
                wb(m, a.wrapping_add(OB_TIME_FRAME), 7);
                quadro ^= 2;
            }
        }
        wb(m, a.wrapping_add(OB_FRAME), quadro);
    }
    display_sprite(m, a);
    let x = rw(m, a.wrapping_add(OB_X));
    if fora_da_janela(m, x) {
        delete_object(m, a);
    }
    Ok(())
}

fn sobrepoe(origem: u16, borda: u16, comprimento: u16) -> bool {
    let (distancia, negativo) = origem.wrapping_sub(16).overflowing_sub(borda);
    if negativo {
        distancia.overflowing_add(32).1
    } else {
        distancia <= comprimento
    }
}

fn bloco_sobre_botao(m: &mut Memoria, a: u32) -> bool {
    let x = rw(m, a.wrapping_add(OB_X)).wrapping_sub(16);
    let y = rw(m, a.wrapping_add(OB_Y)).wrapping_sub(8);
    let mut slot = V_LVLOBJSPACE;
    while slot < V_LVLOBJEND {
        if rb(m, slot.wrapping_add(OB_RENDER)) & 0x80 != 0
            && rb(m, slot.wrapping_add(OB_ID)) == 0x33
            && sobrepoe(rw(m, slot.wrapping_add(OB_X)), x, 32)
            && sobrepoe(rw(m, slot.wrapping_add(OB_Y)), y, 16)
        {
            return true;
        }
        slot = slot.wrapping_add(OBJECT_SIZE);
    }
    false
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn bloco_aciona_e_libera_a_chave() {
        let mut m = Memoria::nova(vec![0; 0x10000]);
        let botao = V_LVLOBJSPACE;
        let bloco = botao.wrapping_add(OBJECT_SIZE);
        wb(&mut m, botao.wrapping_add(OB_ID), 0x32);
        wb(&mut m, botao.wrapping_add(OB_ROUTINE), 2);
        wb(&mut m, botao.wrapping_add(OB_RENDER), 0x84);
        wb(&mut m, botao.wrapping_add(OB_SUBTYPE), 0x82);
        ww(&mut m, botao.wrapping_add(OB_X), 0x100);
        ww(&mut m, botao.wrapping_add(OB_Y), 0x100);
        wb(&mut m, bloco.wrapping_add(OB_ID), 0x33);
        wb(&mut m, bloco.wrapping_add(OB_RENDER), 0x84);
        ww(&mut m, bloco.wrapping_add(OB_X), 0x100);
        ww(&mut m, bloco.wrapping_add(OB_Y), 0xE8);
        executar(&mut m, botao).unwrap();
        assert_eq!(rb(&mut m, F_SWITCH.wrapping_add(2)), 1);
        assert_eq!(rb(&mut m, V_SOUNDQUEUE1), SFX_SWITCH);
        ww(&mut m, bloco.wrapping_add(OB_X), 0x180);
        executar(&mut m, botao).unwrap();
        assert_eq!(rb(&mut m, F_SWITCH.wrapping_add(2)), 0);
    }
}
