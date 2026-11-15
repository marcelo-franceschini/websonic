//! Cápsula de prisão após o chefe, objeto 0x3E (REV01).

use super::ram::*;
use super::{ErroAbertura, fisica, objetos, rb, rl, rw, signpost, wb, wl, ww};
use crate::memoria::Memoria;

const V_BOSSSTATUS: u32 = 0xFF_F7A7;
const MAP_PRI: u32 = 0x1_B52A;
const ANI_PRI: u32 = 0x1_B522;
const PRI_ORIG_Y: u32 = 0x30;
const ANIMAL_DELAY: u32 = 0x36;

fn soltar_sonic(m: &mut Memoria, a: u32) {
    wb(m, a.wrapping_add(OB_ROUTINE_2), 0);
    let status = rb(m, V_PLAYER.wrapping_add(OB_STATUS));
    wb(m, V_PLAYER.wrapping_add(OB_STATUS), (status & !8) | 2);
}

fn iniciar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let subtipo = rb(m, a.wrapping_add(OB_SUBTYPE));
    let (rotina, largura, prioridade, quadro) = match subtipo {
        0 => (2, 32, 4, 0),
        1 => (4, 12, 5, 1),
        2 => (6, 16, 4, 3),
        3 => (8, 16, 3, 5),
        _ => return Err(ErroAbertura::NaoImplementado("subtipo da cápsula")),
    };
    wl(m, a.wrapping_add(OB_MAP), MAP_PRI);
    ww(m, a.wrapping_add(OB_GFX), 0x49D);
    wb(m, a.wrapping_add(OB_RENDER), 4);
    let y = rw(m, a.wrapping_add(OB_Y));
    ww(m, a.wrapping_add(PRI_ORIG_Y), y);
    wb(m, a.wrapping_add(OB_ROUTINE), rotina);
    wb(m, a.wrapping_add(OB_ACTWID), largura);
    wb(m, a.wrapping_add(OB_PRIORITY), prioridade);
    wb(m, a.wrapping_add(OB_FRAME), quadro);
    if subtipo == 2 {
        wb(m, a.wrapping_add(OB_COLTYPE), 0x06);
        wb(m, a.wrapping_add(0x21), 8);
    }
    Ok(())
}

fn corpo(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    if rb(m, V_BOSSSTATUS) == 2 {
        if rb(m, a.wrapping_add(OB_ROUTINE_2)) != 0 {
            soltar_sonic(m, a);
        }
        wb(m, a.wrapping_add(OB_FRAME), 2);
    } else {
        objetos::solid_object(m, a, 43, 24, 24)?;
    }
    Ok(())
}

fn botao(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    objetos::solid_object(m, a, 23, 8, 8)?;
    objetos::animate_sprite(m, a, ANI_PRI)?;
    let y = rw(m, a.wrapping_add(PRI_ORIG_Y));
    ww(m, a.wrapping_add(OB_Y), y);
    if rb(m, a.wrapping_add(OB_ROUTINE_2)) != 0 {
        ww(m, a.wrapping_add(OB_Y), y.wrapping_add(8));
        wb(m, a.wrapping_add(OB_ROUTINE), 10);
        ww(m, a.wrapping_add(OB_TIME_FRAME), 60);
        wb(m, F_TIMECOUNT, 0);
        wb(m, F_LOCKSCREEN, 0);
        wb(m, F_LOCKCTRL, 1);
        ww(m, V_JPADHOLD2, 0x0800);
        soltar_sonic(m, a);
    }
    Ok(())
}

fn gerar_explosao(m: &mut Memoria, a: u32) {
    if rb(m, V_VBLANK_BYTE) & 7 != 0 {
        return;
    }
    if let Some(livre) = objetos::find_free_obj(m) {
        let x = rw(m, a.wrapping_add(OB_X));
        let y = rw(m, a.wrapping_add(OB_Y));
        let sorteio = fisica::random_number(m);
        let dx = (sorteio & 0xFF).wrapping_shr(2).wrapping_sub(32);
        let dy = sorteio.wrapping_shr(8).wrapping_shr(3);
        wb(m, livre.wrapping_add(OB_ID), 0x3F);
        ww(m, livre.wrapping_add(OB_X), x.wrapping_add(dx));
        ww(m, livre.wrapping_add(OB_Y), y.wrapping_add(dy));
    }
}

fn animal(m: &mut Memoria, a: u32, dx: u16, atraso: u16) -> bool {
    if let Some(livre) = objetos::find_free_obj(m) {
        let x = rw(m, a.wrapping_add(OB_X)).wrapping_add(dx);
        let y = rw(m, a.wrapping_add(OB_Y));
        wb(m, livre.wrapping_add(OB_ID), ID_ANIMALS);
        ww(m, livre.wrapping_add(OB_X), x);
        ww(m, livre.wrapping_add(OB_Y), y);
        ww(m, livre.wrapping_add(ANIMAL_DELAY), atraso);
        true
    } else {
        false
    }
}

fn explodir(m: &mut Memoria, a: u32) {
    gerar_explosao(m, a);
    let tempo = rw(m, a.wrapping_add(OB_TIME_FRAME)).wrapping_sub(1);
    ww(m, a.wrapping_add(OB_TIME_FRAME), tempo);
    if tempo != 0 {
        return;
    }
    wb(m, V_BOSSSTATUS, 2);
    wb(m, a.wrapping_add(OB_ROUTINE), 12);
    wb(m, a.wrapping_add(OB_FRAME), 6);
    ww(m, a.wrapping_add(OB_TIME_FRAME), 150);
    let y = rw(m, a.wrapping_add(OB_Y)).wrapping_add(32);
    ww(m, a.wrapping_add(OB_Y), y);
    let mut dx = 0u16.wrapping_sub(28);
    let mut atraso = 154u16;
    for _ in 0..8 {
        if !animal(m, a, dx, atraso) {
            break;
        }
        dx = dx.wrapping_add(7);
        atraso = atraso.wrapping_sub(8);
    }
}

fn libertar_animais(m: &mut Memoria, a: u32) {
    if rb(m, V_VBLANK_BYTE) & 7 == 0 && objetos::find_free_obj(m).is_some() {
        let sorteio = fisica::random_number(m);
        let mut dx = (sorteio & 0x1F).wrapping_sub(6);
        if rl(m, V_RANDOM) & 0x8000 != 0 {
            dx = 0u16.wrapping_sub(dx);
        }
        animal(m, a, dx, 12);
    }
    let tempo = rw(m, a.wrapping_add(OB_TIME_FRAME)).wrapping_sub(1);
    ww(m, a.wrapping_add(OB_TIME_FRAME), tempo);
    if tempo == 0 {
        wb(m, a.wrapping_add(OB_ROUTINE), 14);
        ww(m, a.wrapping_add(OB_TIME_FRAME), 180);
    }
}

fn finalizar(m: &mut Memoria, a: u32) {
    // FixBugs=0: o original examina somente 63 slots a partir do segundo.
    let mut slot = V_OBJSPACE.wrapping_add(OBJECT_SIZE);
    for _ in 0..63 {
        if rb(m, slot.wrapping_add(OB_ID)) == ID_ANIMALS {
            return;
        }
        slot = slot.wrapping_add(OBJECT_SIZE);
    }
    signpost::got_through_act(m);
    objetos::delete_object(m, a);
}

pub fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => iniciar(m, a)?,
        2 => corpo(m, a)?,
        4 => botao(m, a)?,
        6 | 8 | 10 => explodir(m, a),
        12 => libertar_animais(m, a),
        14 => finalizar(m, a),
        _ => return Err(ErroAbertura::NaoImplementado("rotina da cápsula")),
    }
    let x = rw(m, a.wrapping_add(OB_X));
    if objetos::fora_da_janela(m, x) {
        objetos::delete_object(m, a);
    } else {
        objetos::display_sprite(m, a);
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn inicializa_corpo_e_soltar_botao() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x3E);
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 1);
        ww(&mut m, a.wrapping_add(OB_Y), 0x200);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE)), 4);
        assert_eq!(rw(&mut m, a.wrapping_add(PRI_ORIG_Y)), 0x200);
        wb(&mut m, a.wrapping_add(OB_ROUTINE_2), 1);
        wb(&mut m, V_PLAYER.wrapping_add(OB_STATUS), 8);
        botao(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE)), 10);
        assert_eq!(rw(&mut m, a.wrapping_add(OB_TIME_FRAME)), 60);
        assert_eq!(rb(&mut m, V_PLAYER.wrapping_add(OB_STATUS)) & 0xA, 2);
        assert_eq!(rw(&mut m, V_JPADHOLD2), 0x800);
    }

    #[test]
    fn fim_das_explosoes_liberta_oito_animais_com_atrasos() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x3E);
        wb(&mut m, V_VBLANK_BYTE, 1);
        ww(&mut m, a.wrapping_add(OB_X), 0x400);
        ww(&mut m, a.wrapping_add(OB_Y), 0x200);
        ww(&mut m, a.wrapping_add(OB_TIME_FRAME), 1);
        explodir(&mut m, a);
        assert_eq!(rb(&mut m, V_BOSSSTATUS), 2);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE)), 12);
        for i in 0..8u32 {
            let slot = a.wrapping_add(OBJECT_SIZE.wrapping_mul(i.wrapping_add(1)));
            assert_eq!(rb(&mut m, slot.wrapping_add(OB_ID)), ID_ANIMALS);
            assert_eq!(
                rw(&mut m, slot.wrapping_add(ANIMAL_DELAY)),
                154u16.wrapping_sub(u16::try_from(i.wrapping_mul(8)).unwrap())
            );
        }
    }

    #[test]
    fn aguarda_animais_antes_do_cartao_de_fim() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        let outro = a.wrapping_add(OBJECT_SIZE);
        wb(&mut m, a.wrapping_add(OB_ID), 0x3E);
        wb(&mut m, outro.wrapping_add(OB_ID), ID_ANIMALS);
        finalizar(&mut m, a);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ID)), 0x3E);
        wb(&mut m, outro.wrapping_add(OB_ID), 0);
        finalizar(&mut m, a);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ID)), 0);
        assert_eq!(
            rb(&mut m, V_ENDCARD.wrapping_add(OB_ID)),
            ID_GOT_THROUGH_CARD
        );
    }
}
