//! Orbinaut e bolas orbitais, objeto 0x60 (REV01).
use super::super::fisica::{calc_sine, speed_to_pos};
use super::super::ram::*;
use super::super::{ErroAbertura, rb, rl, rw, wb, wl, ww};
use super::{animate_sprite, delete_object, display_sprite, fora_da_janela};
use crate::memoria::Memoria;

const DIRECAO: u32 = 0x36;
const BOLAS: u32 = 0x37;
const PAI: u32 = 0x3c;
const ANI: u32 = 0x125aa;

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            iniciar(m, a);
            Ok(())
        }
        2 => {
            let x = rw(m, V_PLAYER.wrapping_add(OB_X)).abs_diff(rw(m, a.wrapping_add(OB_X)));
            let y = rw(m, V_PLAYER.wrapping_add(OB_Y)).abs_diff(rw(m, a.wrapping_add(OB_Y)));
            if x < 160 && y < 80 && rw(m, V_DEBUGUSE) == 0 {
                wb(m, a.wrapping_add(OB_ANIM), 1);
            }
            animate_sprite(m, a, ANI)?;
            exibir_pai(m, a);
            Ok(())
        }
        4 => {
            speed_to_pos(m, a);
            exibir_pai(m, a);
            Ok(())
        }
        6 => {
            orbitar(m, a);
            Ok(())
        }
        8 => {
            speed_to_pos(m, a);
            if rb(m, a.wrapping_add(OB_RENDER)) & 0x80 == 0 {
                delete_object(m, a);
            } else {
                display_sprite(m, a);
            }
            Ok(())
        }
        _ => Err(ErroAbertura::NaoImplementado("rotina do Orbinaut")),
    }
}

fn iniciar(m: &mut Memoria, a: u32) {
    wl(m, a.wrapping_add(OB_MAP), 0x125b8);
    let zona = rb(m, V_ZONE);
    let arte = if zona == ID_LZ {
        0x467
    } else if zona == 5 {
        0x429
    } else {
        0x2429
    };
    ww(m, a.wrapping_add(OB_GFX), arte);
    let render = rb(m, a.wrapping_add(OB_RENDER)) | 4;
    wb(m, a.wrapping_add(OB_RENDER), render);
    wb(m, a.wrapping_add(OB_PRIORITY), 4);
    wb(m, a.wrapping_add(OB_COLTYPE), 0x0b);
    wb(m, a.wrapping_add(OB_ACTWID), 12);
    let mut proximo = a.wrapping_add(OBJECT_SIZE);
    for i in 0..4u8 {
        while proximo < V_LVLOBJEND && rb(m, proximo.wrapping_add(OB_ID)) != 0 {
            proximo = proximo.wrapping_add(OBJECT_SIZE);
        }
        if proximo >= V_LVLOBJEND {
            break;
        }
        let conta = rb(m, a.wrapping_add(BOLAS)).wrapping_add(1);
        wb(m, a.wrapping_add(BOLAS), conta);
        let indice =
            u8::try_from((proximo.wrapping_sub(V_OBJSPACE) / OBJECT_SIZE) & 0x7f).unwrap_or(0);
        wb(
            m,
            a.wrapping_add(BOLAS).wrapping_add(u32::from(conta)),
            indice,
        );
        wb(m, proximo.wrapping_add(OB_ID), 0x60);
        wb(m, proximo.wrapping_add(OB_ROUTINE), 6);
        wl(m, proximo.wrapping_add(OB_MAP), 0x125b8);
        ww(m, proximo.wrapping_add(OB_GFX), arte);
        let render = rb(m, proximo.wrapping_add(OB_RENDER)) | 4;
        wb(m, proximo.wrapping_add(OB_RENDER), render);
        wb(m, proximo.wrapping_add(OB_PRIORITY), 4);
        wb(m, proximo.wrapping_add(OB_ACTWID), 8);
        wb(m, proximo.wrapping_add(OB_FRAME), 3);
        wb(m, proximo.wrapping_add(OB_COLTYPE), 0x98);
        wb(m, proximo.wrapping_add(OB_ANGLE), i.wrapping_mul(0x40));
        wl(m, proximo.wrapping_add(PAI), a);
        proximo = proximo.wrapping_add(OBJECT_SIZE);
    }
    let estado = rb(m, a.wrapping_add(OB_STATUS));
    wb(
        m,
        a.wrapping_add(DIRECAO),
        if estado & 1 == 0 { 1 } else { 0xff },
    );
    let subtipo = rb(m, a.wrapping_add(OB_SUBTYPE));
    wb(m, a.wrapping_add(OB_ROUTINE), subtipo.wrapping_add(2));
    ww(
        m,
        a.wrapping_add(OB_VELX),
        if estado & 1 == 0 { 0xffc0 } else { 0x40 },
    );
}

fn exibir_pai(m: &mut Memoria, a: u32) {
    let x = rw(m, a.wrapping_add(OB_X));
    if !fora_da_janela(m, x) {
        display_sprite(m, a);
        return;
    }
    let n = rb(m, a.wrapping_add(OB_RESPAWNNO));
    if n != 0 {
        let endereco = V_OBJSTATE.wrapping_add(2).wrapping_add(u32::from(n));
        let estado = rb(m, endereco) & 0x7f;
        wb(m, endereco, estado);
    }
    let total = rb(m, a.wrapping_add(BOLAS));
    for i in 1..=total {
        let n = rb(m, a.wrapping_add(BOLAS).wrapping_add(u32::from(i)));
        let filho = V_OBJSPACE.wrapping_add(u32::from(n).wrapping_mul(OBJECT_SIZE));
        delete_object(m, filho);
    }
    delete_object(m, a);
}

fn orbitar(m: &mut Memoria, a: u32) {
    let pai = rl(m, a.wrapping_add(PAI));
    if rb(m, pai.wrapping_add(OB_ID)) != 0x60 {
        delete_object(m, a);
        return;
    }
    let angulo = rb(m, a.wrapping_add(OB_ANGLE));
    if rb(m, pai.wrapping_add(OB_FRAME)) == 2 && angulo == 0x40 {
        wb(m, a.wrapping_add(OB_ROUTINE), 8);
        let restantes = rb(m, pai.wrapping_add(BOLAS)).wrapping_sub(1);
        wb(m, pai.wrapping_add(BOLAS), restantes);
        if restantes == 0 {
            let rotina = rb(m, pai.wrapping_add(OB_ROUTINE)).wrapping_add(2);
            wb(m, pai.wrapping_add(OB_ROUTINE), rotina);
        }
        let estado = rb(m, pai.wrapping_add(OB_STATUS));
        ww(
            m,
            a.wrapping_add(OB_VELX),
            if estado & 1 == 0 { 0xfe00 } else { 0x0200 },
        );
    } else {
        let (seno, cosseno) = calc_sine(m, angulo);
        let x = rw(m, pai.wrapping_add(OB_X))
            .wrapping_add(u16::from_be_bytes((cosseno >> 4).to_be_bytes()));
        let y = rw(m, pai.wrapping_add(OB_Y))
            .wrapping_add(u16::from_be_bytes((seno >> 4).to_be_bytes()));
        ww(m, a.wrapping_add(OB_X), x);
        ww(m, a.wrapping_add(OB_Y), y);
        let sentido = rb(m, pai.wrapping_add(DIRECAO));
        wb(m, a.wrapping_add(OB_ANGLE), angulo.wrapping_add(sentido));
    }
    display_sprite(m, a);
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn inicializa_quatro_bolas_e_pai_em_espera() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x60);
        wb(&mut m, V_ZONE, ID_LZ);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(BOLAS)), 4);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE)), 2);
        for i in 1..=4u32 {
            let filho = a.wrapping_add(i.wrapping_mul(OBJECT_SIZE));
            assert_eq!(rb(&mut m, filho.wrapping_add(OB_ROUTINE)), 6);
            assert_eq!(
                rb(&mut m, filho.wrapping_add(OB_ANGLE)),
                u8::try_from((i.wrapping_sub(1)).wrapping_mul(64)).unwrap()
            );
        }
    }
    #[test]
    fn ultima_bola_atirada_libera_movimento_do_pai() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let pai = V_LVLOBJSPACE;
        let filho = pai.wrapping_add(OBJECT_SIZE);
        wb(&mut m, pai.wrapping_add(OB_ID), 0x60);
        wb(&mut m, pai.wrapping_add(OB_ROUTINE), 2);
        wb(&mut m, pai.wrapping_add(OB_FRAME), 2);
        wb(&mut m, pai.wrapping_add(BOLAS), 1);
        wl(&mut m, filho.wrapping_add(PAI), pai);
        wb(&mut m, filho.wrapping_add(OB_ANGLE), 0x40);
        orbitar(&mut m, filho);
        assert_eq!(rb(&mut m, pai.wrapping_add(OB_ROUTINE)), 4);
        assert_eq!(rb(&mut m, filho.wrapping_add(OB_ROUTINE)), 8);
        assert_eq!(rw(&mut m, filho.wrapping_add(OB_VELX)), 0xfe00);
    }
}
