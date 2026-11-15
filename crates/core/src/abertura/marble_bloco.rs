//! Bloco empurrável usado pela Marble Zone (objeto 33).
use super::boot::rom_b;
use super::fisica::{obj_floor_dist, obj_hit_wall, speed_to_pos};
use super::objetos::{delete_object, display_sprite, fora_da_janela, solid_object};
use super::ram::*;
use super::{ErroAbertura, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;
const ORIG_X: u32 = 0x34;
const ORIG_Y: u32 = 0x36;
const NA_LAVA: u32 = 0x32;
const VEL_LAVA: u32 = 0x30;

pub fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => iniciar(m, a)?,
        2 => acao(m, a)?,
        4 => verificar_visibilidade(m, a),
        _ => return Err(ErroAbertura::NaoImplementado("rotina do bloco empurrável")),
    }
    Ok(())
}
fn iniciar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    wb(m, a.wrapping_add(OB_ROUTINE), 2);
    wb(m, a.wrapping_add(OB_HEIGHT), 15);
    wb(m, a.wrapping_add(OB_WIDTH), 15);
    wl(m, a.wrapping_add(OB_MAP), 0xC970);
    let tipo = rb(m, a.wrapping_add(OB_SUBTYPE));
    ww(
        m,
        a.wrapping_add(OB_GFX),
        if tipo == 0 { 0x42B8 } else { 0xC2B8 },
    );
    wb(m, a.wrapping_add(OB_RENDER), 4);
    wb(m, a.wrapping_add(OB_PRIORITY), 3);
    let x = rw(m, a.wrapping_add(OB_X));
    let y = rw(m, a.wrapping_add(OB_Y));
    ww(m, a.wrapping_add(ORIG_X), x);
    ww(m, a.wrapping_add(ORIG_Y), y);
    let tabela = 0xC570u32.wrapping_add(u32::from(tipo & 7).wrapping_mul(2));
    wb(m, a.wrapping_add(OB_ACTWID), rom_b(m, tabela));
    wb(
        m,
        a.wrapping_add(OB_FRAME),
        rom_b(m, tabela.wrapping_add(1)),
    );
    let n = rb(m, a.wrapping_add(OB_RESPAWNNO));
    if n != 0 {
        let marca = V_OBJSTATE.wrapping_add(2).wrapping_add(u32::from(n));
        let estado = rb(m, marca);
        wb(m, marca, (estado & 0x7F) | 1);
        if estado & 1 != 0 {
            delete_object(m, a);
            return Ok(());
        }
    }
    acao(m, a)
}
fn acao(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    if rb(m, a.wrapping_add(NA_LAVA)) != 0 {
        return Err(ErroAbertura::NaoImplementado("bloco empurrável na lava"));
    }
    let estado = rb(m, a.wrapping_add(OB_ROUTINE_2));
    if estado == 4 {
        speed_to_pos(m, a);
        let velocidade = rw(m, a.wrapping_add(OB_VELY)).wrapping_add(0x18);
        ww(m, a.wrapping_add(OB_VELY), velocidade);
        let d = obj_floor_dist(m, a);
        if i16::from_be_bytes(d.to_be_bytes()) < 0 {
            let y = rw(m, a.wrapping_add(OB_Y)).wrapping_add(d);
            ww(m, a.wrapping_add(OB_Y), y);
            ww(m, a.wrapping_add(OB_VELY), 0);
            wb(m, a.wrapping_add(OB_ROUTINE_2), 0);
        }
    } else if estado == 6 {
        speed_to_pos(m, a);
        let x = rw(m, a.wrapping_add(OB_X));
        if x & 12 == 0 {
            ww(m, a.wrapping_add(OB_X), x & 0xFFF0);
            let v = rw(m, a.wrapping_add(OB_VELX));
            ww(m, a.wrapping_add(VEL_LAVA), v);
            ww(m, a.wrapping_add(OB_VELX), 0);
            wb(m, a.wrapping_add(OB_ROUTINE_2), 4);
        }
    } else {
        let largura = u16::from(rb(m, a.wrapping_add(OB_ACTWID))).wrapping_add(11);
        let x = rw(m, a.wrapping_add(OB_X));
        let px = rw(m, V_PLAYER.wrapping_add(OB_X));
        let colisao = solid_object(m, a, largura, 16, 17)?;
        if estado == 0 && colisao > 0 {
            empurrar(m, a, px, x);
        }
    }
    if rw(m, V_ZONE_ACT) == 0x200 {
        let subtipo = rb(m, a.wrapping_add(OB_SUBTYPE)) & 0x7F;
        wb(m, a.wrapping_add(OB_SUBTYPE), subtipo);
        let x = rw(m, a.wrapping_add(OB_X));
        if (0xA20..0xAA1).contains(&x) {
            let y = rw(m, 0xFF_F7A4).wrapping_sub(0x1C);
            ww(m, a.wrapping_add(OB_Y), y);
            let b = rb(m, 0xFF_F7A4) | 0x80;
            wb(m, 0xFF_F7A4, b);
            wb(m, a.wrapping_add(OB_SUBTYPE), subtipo | 0x80);
        }
    }
    apresentar(m, a);
    Ok(())
}
fn empurrar(m: &mut Memoria, a: u32, px: u16, x: u16) {
    let direita = px < x;
    let status = rb(m, V_PLAYER.wrapping_add(OB_STATUS));
    if (status & 1 != 0) == direita {
        return;
    }
    let largura = u16::from(rb(m, a.wrapping_add(OB_ACTWID)));
    let desloc = if direita { largura } else { !largura };
    if i16::from_be_bytes(obj_hit_wall(m, a, desloc, !direita).to_be_bytes()) < 0 {
        return;
    }
    let passo = if direita { 1u16 } else { 0xFFFF };
    ww(m, a.wrapping_add(OB_X), x.wrapping_add(passo));
    let px = rw(m, V_PLAYER.wrapping_add(OB_X)).wrapping_add(passo);
    ww(m, V_PLAYER.wrapping_add(OB_X), px);
    ww(
        m,
        V_PLAYER.wrapping_add(OB_INERTIA),
        if direita { 0x40 } else { 0xFFC0 },
    );
    ww(m, V_PLAYER.wrapping_add(OB_VELX), 0);
    wb(m, V_SOUNDQUEUE1, 0xA7);
    if rb(m, a.wrapping_add(OB_SUBTYPE)) & 0x80 == 0 {
        let d = obj_floor_dist(m, a);
        if i16::from_be_bytes(d.to_be_bytes()) > 4 {
            ww(
                m,
                a.wrapping_add(OB_VELX),
                if direita { 0x400 } else { 0xFC00 },
            );
            wb(m, a.wrapping_add(OB_ROUTINE_2), 6);
        } else {
            let y = rw(m, a.wrapping_add(OB_Y)).wrapping_add(d);
            ww(m, a.wrapping_add(OB_Y), y);
        }
    }
}
fn apresentar(m: &mut Memoria, a: u32) {
    let x = rw(m, a.wrapping_add(OB_X));
    if !fora_da_janela(m, x) {
        display_sprite(m, a);
        return;
    }
    let origem = rw(m, a.wrapping_add(ORIG_X));
    if fora_da_janela(m, origem) {
        let n = rb(m, a.wrapping_add(OB_RESPAWNNO));
        if n != 0 {
            let marca = V_OBJSTATE.wrapping_add(2).wrapping_add(u32::from(n));
            let valor = rb(m, marca) & !1;
            wb(m, marca, valor);
        }
        delete_object(m, a);
    } else {
        ww(m, a.wrapping_add(OB_X), origem);
        let y = rw(m, a.wrapping_add(ORIG_Y));
        ww(m, a.wrapping_add(OB_Y), y);
        wb(m, a.wrapping_add(OB_ROUTINE), 4);
        verificar_visibilidade(m, a);
    }
}
fn verificar_visibilidade(m: &mut Memoria, a: u32) {
    let largura = u16::from(rb(m, a.wrapping_add(OB_ACTWID)));
    let x = rw(m, a.wrapping_add(OB_X))
        .wrapping_sub(rw(m, V_SCREENPOSX))
        .wrapping_add(largura);
    let y = rw(m, a.wrapping_add(OB_Y)).wrapping_sub(rw(m, V_SCREENPOSY));
    let direita = i16::from_be_bytes(x.wrapping_sub(largura.wrapping_mul(2)).to_be_bytes());
    if i16::from_be_bytes(x.to_be_bytes()) >= 0 && direita < 320 && y < 224 {
        return;
    }
    wb(m, a.wrapping_add(OB_ROUTINE), 2);
    wb(m, a.wrapping_add(NA_LAVA), 0);
    ww(m, a.wrapping_add(OB_VELX), 0);
    ww(m, a.wrapping_add(OB_VELY), 0);
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn bloco_posicionado_no_esmagador_atualiza_o_limite() {
        let mut rom = vec![0; 0x10000];
        rom[0xC570..0xC572].copy_from_slice(&[16, 0]);
        let mut m = Memoria::nova(rom);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x33);
        ww(&mut m, V_ZONE_ACT, 0x200);
        ww(&mut m, a.wrapping_add(OB_X), 0xA40);
        ww(&mut m, a.wrapping_add(OB_Y), 0x300);
        ww(&mut m, V_SCREENPOSX, 0x900);
        ww(&mut m, 0xFF_F7A4, 0x300);
        executar(&mut m, a).unwrap();
        assert_eq!(rw(&mut m, a.wrapping_add(OB_Y)), 0x2E4);
        assert_ne!(rb(&mut m, a.wrapping_add(OB_SUBTYPE)) & 0x80, 0);
        assert_ne!(rb(&mut m, 0xFF_F7A4) & 0x80, 0);
    }
}
