//! Hastes de espinhos giratórios e bônus ocultos de Green Hill.
use super::boot::rom_w;
use super::fisica::add_points;
use super::objetos::{delete_object, display_sprite, find_free_obj, fora_da_janela};
use super::ram::*;
use super::{ErroAbertura, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;

pub(super) fn helix(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            wb(m, a.wrapping_add(OB_ROUTINE), 2);
            wl(m, a.wrapping_add(OB_MAP), 0x8476);
            ww(m, a.wrapping_add(OB_GFX), 0x4398);
            wb(m, a.wrapping_add(OB_STATUS), 7);
            wb(m, a.wrapping_add(OB_RENDER), 4);
            wb(m, a.wrapping_add(OB_PRIORITY), 3);
            wb(m, a.wrapping_add(OB_ACTWID), 8);
            let y = rw(m, a.wrapping_add(OB_Y));
            let origem = rw(m, a.wrapping_add(OB_X));
            let quantidade = rb(m, a.wrapping_add(OB_SUBTYPE));
            wb(m, a.wrapping_add(OB_SUBTYPE), 0);
            let mut x = origem.wrapping_sub(u16::from(quantidade >> 1).wrapping_mul(16));
            let mut frame = 0u8;
            if quantidade >= 2 {
                let mut indice_lista = 1u32;
                for _ in 0..quantidade.wrapping_sub(1) {
                    let Some(b) = find_free_obj(m) else { break };
                    let count = rb(m, a.wrapping_add(OB_SUBTYPE)).wrapping_add(1);
                    wb(m, a.wrapping_add(OB_SUBTYPE), count);
                    let indice = u8::try_from((b.wrapping_sub(V_OBJSPACE) / OBJECT_SIZE) & 0x7F)
                        .unwrap_or(0);
                    wb(
                        m,
                        a.wrapping_add(OB_SUBTYPE).wrapping_add(indice_lista),
                        indice,
                    );
                    indice_lista = indice_lista.wrapping_add(1);
                    wb(m, b.wrapping_add(OB_ROUTINE), 8);
                    wb(m, b.wrapping_add(OB_ID), 0x17);
                    ww(m, b.wrapping_add(OB_Y), y);
                    ww(m, b.wrapping_add(OB_X), x);
                    wl(m, b.wrapping_add(OB_MAP), 0x8476);
                    ww(m, b.wrapping_add(OB_GFX), 0x4398);
                    wb(m, b.wrapping_add(OB_RENDER), 4);
                    wb(m, b.wrapping_add(OB_PRIORITY), 3);
                    wb(m, b.wrapping_add(OB_ACTWID), 8);
                    wb(m, b.wrapping_add(0x3E), frame);
                    frame = frame.wrapping_add(1) & 7;
                    x = x.wrapping_add(16);
                    if x == origem {
                        wb(m, a.wrapping_add(0x3E), frame);
                        frame = frame.wrapping_add(1) & 7;
                        x = x.wrapping_add(16);
                        let count = rb(m, a.wrapping_add(OB_SUBTYPE)).wrapping_add(1);
                        wb(m, a.wrapping_add(OB_SUBTYPE), count);
                    }
                }
            }
            girar(m, a);
            display_sprite(m, a);
            verificar_helix(m, a);
        }
        2 | 4 => {
            girar(m, a);
            display_sprite(m, a);
            verificar_helix(m, a);
        }
        6 => delete_object(m, a),
        8 => {
            girar(m, a);
            display_sprite(m, a);
        }
        _ => return Err(ErroAbertura::NaoImplementado("rotina da helice")),
    }
    Ok(())
}

fn girar(m: &mut Memoria, a: u32) {
    let frame = rb(m, 0xFF_FEC1).wrapping_add(rb(m, a.wrapping_add(0x3E))) & 7;
    wb(m, a.wrapping_add(OB_FRAME), frame);
    wb(
        m,
        a.wrapping_add(OB_COLTYPE),
        if frame == 0 { 0x84 } else { 0 },
    );
}

fn verificar_helix(m: &mut Memoria, a: u32) {
    let x = rw(m, a.wrapping_add(OB_X));
    if !fora_da_janela(m, x) {
        return;
    }
    let n = rb(m, a.wrapping_add(OB_SUBTYPE));
    if n >= 2 {
        for i in 0..u32::from(n.wrapping_sub(1)) {
            let indice = rb(
                m,
                a.wrapping_add(OB_SUBTYPE).wrapping_add(1).wrapping_add(i),
            );
            delete_object(
                m,
                V_OBJSPACE.wrapping_add(u32::from(indice).wrapping_mul(OBJECT_SIZE)),
            );
        }
    }
    delete_object(m, a);
}

pub(super) fn bonus(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            let px = rw(m, V_PLAYER.wrapping_add(OB_X));
            let py = rw(m, V_PLAYER.wrapping_add(OB_Y));
            let x = rw(m, a.wrapping_add(OB_X));
            let y = rw(m, a.wrapping_add(OB_Y));
            if px.wrapping_sub(x).wrapping_add(16) < 32
                && py.wrapping_sub(y).wrapping_add(16) < 32
                && rw(m, V_DEBUGUSE) == 0
                && rb(m, 0xFF_F7CD) == 0
            {
                wb(m, a.wrapping_add(OB_ROUTINE), 2);
                wl(m, a.wrapping_add(OB_MAP), 0x179F4);
                ww(m, a.wrapping_add(OB_GFX), 0x84B6);
                let render = rb(m, a.wrapping_add(OB_RENDER)) | 4;
                wb(m, a.wrapping_add(OB_RENDER), render);
                wb(m, a.wrapping_add(OB_PRIORITY), 0);
                wb(m, a.wrapping_add(OB_ACTWID), 16);
                let subtipo = rb(m, a.wrapping_add(OB_SUBTYPE));
                wb(m, a.wrapping_add(OB_FRAME), subtipo);
                ww(m, a.wrapping_add(0x30), 119);
                wb(m, V_SOUNDQUEUE1, 0xC9);
                let pontos = rom_w(
                    m,
                    0x179BEu32.wrapping_add(u32::from(subtipo).wrapping_mul(2)),
                );
                add_points(m, u32::from(pontos));
            }
        }
        2 => {
            let tempo = rw(m, a.wrapping_add(0x30)).wrapping_sub(1);
            ww(m, a.wrapping_add(0x30), tempo);
            if i16::from_be_bytes(tempo.to_be_bytes()) < 0 {
                delete_object(m, a);
                return Ok(());
            }
            let x = rw(m, a.wrapping_add(OB_X));
            if fora_da_janela(m, x) {
                delete_object(m, a);
            } else {
                display_sprite(m, a);
            }
            return Ok(());
        }
        _ => return Err(ErroAbertura::NaoImplementado("rotina do bonus oculto")),
    }
    let x = rw(m, a.wrapping_add(OB_X));
    if fora_da_janela(m, x) {
        delete_object(m, a);
    }
    Ok(())
}

pub(super) fn anel_gigante(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            wl(m, a.wrapping_add(OB_MAP), 0xA654);
            ww(m, a.wrapping_add(OB_GFX), 0x2400);
            let render = rb(m, a.wrapping_add(OB_RENDER)) | 4;
            wb(m, a.wrapping_add(OB_RENDER), render);
            wb(m, a.wrapping_add(OB_ACTWID), 64);
            if render & 0x80 != 0 {
                if rb(m, V_EMERALDS) == 6 {
                    delete_object(m, a);
                    return Ok(());
                }
                if rw(m, V_RINGS) < 50 {
                    return Ok(());
                }
                wb(m, a.wrapping_add(OB_ROUTINE), 2);
                wb(m, a.wrapping_add(OB_PRIORITY), 2);
                wb(m, a.wrapping_add(OB_COLTYPE), 0x52);
                ww(m, V_GFXBIGRING, 0xC40);
            }
        }
        2 => {}
        4 => {
            wb(m, a.wrapping_add(OB_ROUTINE), 2);
            wb(m, a.wrapping_add(OB_COLTYPE), 0);
            if let Some(b) = find_free_obj(m) {
                let x = rw(m, a.wrapping_add(OB_X));
                let y = rw(m, a.wrapping_add(OB_Y));
                let sonic_x = rw(m, V_PLAYER.wrapping_add(OB_X));
                wb(m, b.wrapping_add(OB_ID), 0x7C);
                ww(m, b.wrapping_add(OB_X), x);
                ww(m, b.wrapping_add(OB_Y), y);
                wl(m, b.wrapping_add(0x3C), a);
                if sonic_x >= x {
                    let render = rb(m, b.wrapping_add(OB_RENDER));
                    wb(m, b.wrapping_add(OB_RENDER), render | 1);
                }
            }
            wb(m, V_SOUNDQUEUE1, 0xC3);
        }
        6 => {
            delete_object(m, a);
            return Ok(());
        }
        _ => return Err(ErroAbertura::NaoImplementado("rotina do anel gigante")),
    }
    let frame = rb(m, V_ANI1_FRAME);
    wb(m, a.wrapping_add(OB_FRAME), frame);
    let x = rw(m, a.wrapping_add(OB_X));
    if fora_da_janela(m, x) {
        delete_object(m, a);
    } else {
        display_sprite(m, a);
    }
    Ok(())
}

pub(super) fn brilho_anel(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            wb(m, a.wrapping_add(OB_ROUTINE), 2);
            wl(m, a.wrapping_add(OB_MAP), 0xA6F6);
            ww(m, a.wrapping_add(OB_GFX), 0x2462);
            let render = rb(m, a.wrapping_add(OB_RENDER)) | 4;
            wb(m, a.wrapping_add(OB_RENDER), render);
            wb(m, a.wrapping_add(OB_PRIORITY), 0);
            wb(m, a.wrapping_add(OB_ACTWID), 32);
            wb(m, a.wrapping_add(OB_FRAME), 0xFF);
        }
        2 => {}
        4 => {
            delete_object(m, a);
            return Ok(());
        }
        _ => return Err(ErroAbertura::NaoImplementado("rotina do brilho do anel")),
    }
    let tempo = rb(m, a.wrapping_add(OB_TIME_FRAME)).wrapping_sub(1);
    wb(m, a.wrapping_add(OB_TIME_FRAME), tempo);
    if i8::from_be_bytes([tempo]) < 0 {
        wb(m, a.wrapping_add(OB_TIME_FRAME), 1);
        let frame = rb(m, a.wrapping_add(OB_FRAME)).wrapping_add(1);
        wb(m, a.wrapping_add(OB_FRAME), frame);
        if frame >= 8 {
            wb(m, a.wrapping_add(OB_ROUTINE), 4);
            ww(m, V_PLAYER, 0);
            return Ok(());
        }
        if frame == 3 {
            let pai = super::rl(m, a.wrapping_add(0x3C));
            wb(m, pai.wrapping_add(OB_ROUTINE), 6);
            wb(m, V_PLAYER.wrapping_add(OB_ANIM), 0x1C);
            wb(m, 0xFF_F7CD, 1);
            wb(m, V_INVINC, 0);
            wb(m, V_SHIELD, 0);
        }
    }
    let x = rw(m, a.wrapping_add(OB_X));
    if fora_da_janela(m, x) {
        delete_object(m, a);
    } else {
        display_sprite(m, a);
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn anel_exige_50_aneis_e_cria_brilho() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x4B);
        wb(&mut m, a.wrapping_add(OB_RENDER), 0x80);
        ww(&mut m, V_RINGS, 49);
        anel_gigante(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE)), 0);
        ww(&mut m, V_RINGS, 50);
        anel_gigante(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_COLTYPE)), 0x52);
        assert_eq!(rw(&mut m, V_GFXBIGRING), 0xC40);
        wb(&mut m, a.wrapping_add(OB_ROUTINE), 4);
        anel_gigante(&mut m, a).unwrap();
        let brilho = a.wrapping_add(OBJECT_SIZE);
        assert_eq!(rb(&mut m, brilho.wrapping_add(OB_ID)), 0x7C);
        assert_eq!(super::super::rl(&mut m, brilho.wrapping_add(0x3C)), a);
    }

    #[test]
    fn helix_cria_spikes_e_alterna_perigo_pelo_quadro() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x17);
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 16);
        ww(&mut m, a.wrapping_add(OB_X), 0x100);
        ww(&mut m, a.wrapping_add(OB_Y), 0x100);
        helix(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_SUBTYPE)), 16);
        let b = a.wrapping_add(OBJECT_SIZE);
        assert_eq!(rb(&mut m, b.wrapping_add(OB_ID)), 0x17);
        assert_eq!(rw(&mut m, b.wrapping_add(OB_X)), 0x80);
        helix(&mut m, b).unwrap();
        assert_eq!(rb(&mut m, b.wrapping_add(OB_COLTYPE)), 0x84);
        wb(&mut m, 0xFF_FEC1, 1);
        helix(&mut m, b).unwrap();
        assert_eq!(rb(&mut m, b.wrapping_add(OB_COLTYPE)), 0);
    }

    #[test]
    fn bonus_nao_ativa_durante_debug() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x7D);
        ww(&mut m, V_DEBUGUSE, 1);
        bonus(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE)), 0);
        ww(&mut m, V_DEBUGUSE, 0);
        bonus(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(rb(&mut m, V_SOUNDQUEUE1), 0xC9);
    }
}
