//! Gárgulas e bolas de fogo da Labyrinth, objeto 0x62 (REV01).
use super::super::boot::rom_b;
use super::super::fisica::{obj_hit_wall, speed_to_pos};
use super::super::ram::*;
use super::super::{ErroAbertura, rb, rw, wb, wl, ww};
use super::{delete_object, find_free_obj, remember_state};
use crate::memoria::Memoria;

const TAXAS: u32 = 0x128d6;

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let mut rotina = rb(m, a.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        rotina = 2;
        wb(m, a.wrapping_add(OB_ROUTINE), rotina);
        wl(m, a.wrapping_add(OB_MAP), 0x129ec);
        ww(m, a.wrapping_add(OB_GFX), 0x42e9);
        let render = rb(m, a.wrapping_add(OB_RENDER)) | 4;
        wb(m, a.wrapping_add(OB_RENDER), render);
        wb(m, a.wrapping_add(OB_PRIORITY), 3);
        wb(m, a.wrapping_add(OB_ACTWID), 16);
        let subtipo = rb(m, a.wrapping_add(OB_SUBTYPE)) & 15;
        let taxa = rom_b(m, TAXAS.wrapping_add(u32::from(subtipo)));
        wb(m, a.wrapping_add(OB_DELAY_ANI), taxa);
        wb(m, a.wrapping_add(OB_TIME_FRAME), taxa);
        wb(m, a.wrapping_add(OB_SUBTYPE), subtipo);
    } else if rotina == 4 {
        iniciar_fogo(m, a);
        rotina = 6;
    }
    match rotina {
        2 => gerar(m, a),
        6 => mover_fogo(m, a),
        _ => {
            return Err(ErroAbertura::NaoImplementado(
                "rotina da gargula de Labyrinth",
            ));
        }
    }
    remember_state(m, a);
    Ok(())
}

fn gerar(m: &mut Memoria, a: u32) {
    let tempo = rb(m, a.wrapping_add(OB_TIME_FRAME)).wrapping_sub(1);
    wb(m, a.wrapping_add(OB_TIME_FRAME), tempo);
    if tempo != 0 {
        return;
    }
    let taxa = rb(m, a.wrapping_add(OB_DELAY_ANI));
    wb(m, a.wrapping_add(OB_TIME_FRAME), taxa);
    let x = rw(m, a.wrapping_add(OB_X)).wrapping_sub(rw(m, V_SCREENPOSX));
    let y = rw(m, a.wrapping_add(OB_Y)).wrapping_sub(rw(m, V_SCREENPOSY));
    if i16::from_be_bytes(x.to_be_bytes()) < 0
        || x >= 320
        || i16::from_be_bytes(y.to_be_bytes()) < 0
        || y >= 224
    {
        return;
    }
    if let Some(filho) = find_free_obj(m) {
        wb(m, filho.wrapping_add(OB_ID), 0x62);
        let rotina = rb(m, filho.wrapping_add(OB_ROUTINE)).wrapping_add(4);
        wb(m, filho.wrapping_add(OB_ROUTINE), rotina);
        let x = rw(m, a.wrapping_add(OB_X));
        let y = rw(m, a.wrapping_add(OB_Y));
        ww(m, filho.wrapping_add(OB_X), x);
        ww(m, filho.wrapping_add(OB_Y), y);
        let render = rb(m, a.wrapping_add(OB_RENDER));
        let status = rb(m, a.wrapping_add(OB_STATUS));
        wb(m, filho.wrapping_add(OB_RENDER), render);
        wb(m, filho.wrapping_add(OB_STATUS), status);
    }
}

fn iniciar_fogo(m: &mut Memoria, a: u32) {
    wb(m, a.wrapping_add(OB_ROUTINE), 6);
    wb(m, a.wrapping_add(OB_HEIGHT), 8);
    wb(m, a.wrapping_add(OB_WIDTH), 8);
    wl(m, a.wrapping_add(OB_MAP), 0x129ec);
    ww(m, a.wrapping_add(OB_GFX), 0x02e9);
    let render = rb(m, a.wrapping_add(OB_RENDER)) | 4;
    wb(m, a.wrapping_add(OB_RENDER), render);
    wb(m, a.wrapping_add(OB_PRIORITY), 4);
    wb(m, a.wrapping_add(OB_COLTYPE), 0x98);
    wb(m, a.wrapping_add(OB_ACTWID), 8);
    wb(m, a.wrapping_add(OB_FRAME), 2);
    let y = rw(m, a.wrapping_add(OB_Y)).wrapping_add(8);
    ww(m, a.wrapping_add(OB_Y), y);
    let status = rb(m, a.wrapping_add(OB_STATUS));
    ww(
        m,
        a.wrapping_add(OB_VELX),
        if status & 1 == 0 { 0xfe00 } else { 0x0200 },
    );
    wb(m, V_SOUNDQUEUE1, 0xae);
}

fn mover_fogo(m: &mut Memoria, a: u32) {
    if rb(m, V_FRAMEBYTE) & 7 == 0 {
        let quadro = rb(m, a.wrapping_add(OB_FRAME)) ^ 1;
        wb(m, a.wrapping_add(OB_FRAME), quadro);
    }
    speed_to_pos(m, a);
    let esquerda = rb(m, a.wrapping_add(OB_STATUS)) & 1 == 0;
    let distancia = obj_hit_wall(m, a, if esquerda { 0xfff8 } else { 8 }, esquerda);
    if i16::from_be_bytes(distancia.to_be_bytes()) < 0 {
        delete_object(m, a);
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn inicializacao_carrega_taxa_da_tabela() {
        let mut rom = vec![0; 0x20000];
        rom[0x128d7] = 60;
        let mut m = Memoria::nova(rom);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x62);
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 1);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_DELAY_ANI)), 60);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_TIME_FRAME)), 59);
    }
    #[test]
    fn gera_fogo_apenas_quando_visivel_e_recarrega_tempo() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x62);
        wb(&mut m, a.wrapping_add(OB_TIME_FRAME), 1);
        wb(&mut m, a.wrapping_add(OB_DELAY_ANI), 60);
        ww(&mut m, a.wrapping_add(OB_X), 100);
        ww(&mut m, a.wrapping_add(OB_Y), 100);
        gerar(&mut m, a);
        let filho = a.wrapping_add(OBJECT_SIZE);
        assert_eq!(rb(&mut m, filho.wrapping_add(OB_ID)), 0x62);
        assert_eq!(rb(&mut m, filho.wrapping_add(OB_ROUTINE)), 4);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_TIME_FRAME)), 60);
    }
    #[test]
    fn fogo_sai_na_direcao_da_gargula() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_STATUS), 1);
        ww(&mut m, a.wrapping_add(OB_Y), 100);
        iniciar_fogo(&mut m, a);
        assert_eq!(rw(&mut m, a.wrapping_add(OB_VELX)), 0x200);
        assert_eq!(rw(&mut m, a.wrapping_add(OB_Y)), 108);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_COLTYPE)), 0x98);
        assert_eq!(rb(&mut m, V_SOUNDQUEUE1), 0xae);
    }
}
