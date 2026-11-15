//! Barreiras invisíveis, objeto 0x71.
use super::objetos::{delete_object, display_sprite, fora_da_janela, solid_object};
use super::ram::*;
use super::{ErroAbertura, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;

pub fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            wb(m, a.wrapping_add(OB_ROUTINE), 2);
            wl(m, a.wrapping_add(OB_MAP), 0x11B8A);
            ww(m, a.wrapping_add(OB_GFX), 0x8680);
            let render = rb(m, a.wrapping_add(OB_RENDER)) | 4;
            wb(m, a.wrapping_add(OB_RENDER), render);
            let subtipo = rb(m, a.wrapping_add(OB_SUBTYPE));
            wb(
                m,
                a.wrapping_add(OB_ACTWID),
                u8::try_from(u16::from(subtipo & 0xF0).wrapping_add(0x10) >> 1).unwrap_or(0),
            );
            wb(
                m,
                a.wrapping_add(OB_HEIGHT),
                (subtipo & 15).wrapping_add(1).wrapping_mul(8),
            );
        }
        2 => {}
        _ => {
            return Err(ErroAbertura::NaoImplementado(
                "rotina da barreira invisível",
            ));
        }
    }
    let x = rw(m, a.wrapping_add(OB_X));
    let y = rw(m, a.wrapping_add(OB_Y));
    let dx = x.wrapping_sub(rw(m, V_SCREENPOSX));
    let dy = y.wrapping_sub(rw(m, V_SCREENPOSY));
    if i16::from_be_bytes(dx.to_be_bytes()) >= 0
        && dx < 320
        && i16::from_be_bytes(dy.to_be_bytes()) >= 0
        && dy < 224
    {
        let largura = u16::from(rb(m, a.wrapping_add(OB_ACTWID))).wrapping_add(11);
        let altura = u16::from(rb(m, a.wrapping_add(OB_HEIGHT)));
        // SolidObject_NoRenderChk usa a mesma colisão sem exigir o bit de visibilidade.
        let render = rb(m, a.wrapping_add(OB_RENDER));
        wb(m, a.wrapping_add(OB_RENDER), render | 0x80);
        let resultado = solid_object(m, a, largura, altura, altura.wrapping_add(1));
        wb(m, a.wrapping_add(OB_RENDER), render);
        resultado?;
    }
    if fora_da_janela(m, x) {
        delete_object(m, a);
    } else if rw(m, V_DEBUGUSE) != 0 {
        display_sprite(m, a);
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn inicializa_dimensoes_e_permanece_invisivel() {
        let mut m = Memoria::nova(vec![0; 0x10000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x71);
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 0x23);
        ww(&mut m, a.wrapping_add(OB_X), 100);
        ww(&mut m, a.wrapping_add(OB_Y), 100);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ACTWID)), 24);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_HEIGHT)), 32);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_RENDER)), 4);
        assert_eq!(rw(&mut m, V_SPRITEQUEUE), 0);
    }
}
