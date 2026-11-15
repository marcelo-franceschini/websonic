//! Parede quebrável e fragmentos de GHZ e SLZ.
use super::boot::rom_w;
use super::fisica::speed_to_pos;
use super::objetos::{delete_object, display_sprite, find_free_obj, remember_state, solid_object};
use super::ram::*;
use super::{ErroAbertura, rb, rl, rw, wb, wl, ww};
use crate::memoria::Memoria;

const MAPA: u32 = 0xD944;
const VELOCIDADE: u32 = 0x30;

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            wb(m, a.wrapping_add(OB_ROUTINE), 2);
            wl(m, a.wrapping_add(OB_MAP), MAPA);
            ww(m, a.wrapping_add(OB_GFX), 0x450F);
            wb(m, a.wrapping_add(OB_RENDER), 4);
            wb(m, a.wrapping_add(OB_ACTWID), 16);
            wb(m, a.wrapping_add(OB_PRIORITY), 4);
            let subtipo = rb(m, a.wrapping_add(OB_SUBTYPE));
            wb(m, a.wrapping_add(OB_FRAME), subtipo);
            solida(m, a)?;
        }
        2 => solida(m, a)?,
        4 => fragmento(m, a),
        _ => return Err(ErroAbertura::NaoImplementado("rotina da parede quebravel")),
    }
    remember_state(m, a);
    Ok(())
}

fn solida(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let velocidade = rw(m, V_PLAYER.wrapping_add(OB_VELX));
    ww(m, a.wrapping_add(VELOCIDADE), velocidade);
    solid_object(m, a, 27, 32, 32)?;
    if rb(m, a.wrapping_add(OB_STATUS)) & STATUS_PUSHING == 0
        || rb(m, V_PLAYER.wrapping_add(OB_ANIM)) != 2
        || i16::from_be_bytes(velocidade.to_be_bytes()).unsigned_abs() < 0x480
    {
        return Ok(());
    }
    ww(m, V_PLAYER.wrapping_add(OB_VELX), velocidade);
    let sonic_x = rw(m, V_PLAYER.wrapping_add(OB_X)).wrapping_add(4);
    ww(m, V_PLAYER.wrapping_add(OB_X), sonic_x);
    let direita = rw(m, a.wrapping_add(OB_X)) < sonic_x;
    if !direita {
        ww(m, V_PLAYER.wrapping_add(OB_X), sonic_x.wrapping_sub(8));
    }
    ww(m, V_PLAYER.wrapping_add(OB_INERTIA), velocidade);
    let status = rb(m, a.wrapping_add(OB_STATUS)) & !STATUS_PUSHING;
    wb(m, a.wrapping_add(OB_STATUS), status);
    let status = rb(m, V_PLAYER.wrapping_add(OB_STATUS)) & !STATUS_PUSHING;
    wb(m, V_PLAYER.wrapping_add(OB_STATUS), status);
    quebrar(m, a, if direita { 0xD904 } else { 0xD924 });
    fragmento(m, a);
    Ok(())
}

fn quebrar(m: &mut Memoria, a: u32, tabela: u32) {
    let mapa = rl(m, a.wrapping_add(OB_MAP));
    let frame = u32::from(rb(m, a.wrapping_add(OB_FRAME)));
    let offset = rom_w(m, mapa.wrapping_add(frame.wrapping_mul(2)));
    let mut peca = mapa.wrapping_add(u32::from(offset)).wrapping_add(1);
    let render = rb(m, a.wrapping_add(OB_RENDER)) | 0x20;
    wb(m, a.wrapping_add(OB_RENDER), render);
    let id = rb(m, a.wrapping_add(OB_ID));
    let x = rw(m, a.wrapping_add(OB_X));
    let y = rw(m, a.wrapping_add(OB_Y));
    let gfx = rw(m, a.wrapping_add(OB_GFX));
    let prioridade = rb(m, a.wrapping_add(OB_PRIORITY));
    let largura = rb(m, a.wrapping_add(OB_ACTWID));
    for i in 0..8u32 {
        let b = if i == 0 {
            a
        } else if let Some(livre) = find_free_obj(m) {
            livre
        } else {
            break;
        };
        if i != 0 {
            peca = peca.wrapping_add(5);
        }
        wb(m, b.wrapping_add(OB_ROUTINE), 4);
        wb(m, b.wrapping_add(OB_ID), id);
        wl(m, b.wrapping_add(OB_MAP), peca);
        wb(m, b.wrapping_add(OB_RENDER), render);
        ww(m, b.wrapping_add(OB_X), x);
        ww(m, b.wrapping_add(OB_Y), y);
        ww(m, b.wrapping_add(OB_GFX), gfx);
        wb(m, b.wrapping_add(OB_PRIORITY), prioridade);
        wb(m, b.wrapping_add(OB_ACTWID), largura);
        let vx = rom_w(m, tabela.wrapping_add(i.wrapping_mul(4)));
        let vy = rom_w(m, tabela.wrapping_add(i.wrapping_mul(4)).wrapping_add(2));
        ww(m, b.wrapping_add(OB_VELX), vx);
        ww(m, b.wrapping_add(OB_VELY), vy);
        if b < a {
            speed_to_pos(m, b);
            let vy = rw(m, b.wrapping_add(OB_VELY)).wrapping_add(0x70);
            ww(m, b.wrapping_add(OB_VELY), vy);
            display_sprite(m, b);
        }
    }
    wb(m, V_SOUNDQUEUE1, 0xCB);
}

fn fragmento(m: &mut Memoria, a: u32) {
    speed_to_pos(m, a);
    let vy = rw(m, a.wrapping_add(OB_VELY)).wrapping_add(0x70);
    ww(m, a.wrapping_add(OB_VELY), vy);
    display_sprite(m, a);
    if rb(m, a.wrapping_add(OB_RENDER)) & 0x80 == 0 {
        delete_object(m, a);
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn fragmentos_usam_tabela_da_rom_e_som() {
        let mut rom = vec![0; 0x10000];
        let map = usize::try_from(MAPA).unwrap();
        rom[map..map.wrapping_add(2)].copy_from_slice(&6u16.to_be_bytes());
        let tabela = 0xD904usize;
        rom[tabela..tabela.wrapping_add(2)].copy_from_slice(&0x400u16.to_be_bytes());
        rom[tabela.wrapping_add(2)..tabela.wrapping_add(4)]
            .copy_from_slice(&0xFB00u16.to_be_bytes());
        let mut m = Memoria::nova(rom);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x3C);
        wl(&mut m, a.wrapping_add(OB_MAP), MAPA);
        quebrar(&mut m, a, 0xD904);
        assert_eq!(rw(&mut m, a.wrapping_add(OB_VELX)), 0x400);
        assert_eq!(rw(&mut m, a.wrapping_add(OB_VELY)), 0xFB00);
        assert_eq!(rl(&mut m, a.wrapping_add(OB_MAP)), MAPA.wrapping_add(7));
        assert_eq!(rb(&mut m, V_SOUNDQUEUE1), 0xCB);
        assert_eq!(
            rb(&mut m, a.wrapping_add(OBJECT_SIZE).wrapping_add(OB_ID)),
            0x3C
        );
    }
}
