//! Burrobot da Labyrinth, objeto 0x2D.
use super::super::fisica::{obj_floor_dist, obj_floor_dist_and_angle, speed_to_pos};
use super::super::ram::*;
use super::super::{ErroAbertura, rb, rw, wb, wl, ww};
use super::{animate_sprite, remember_state};
use crate::memoria::Memoria;

const TEMPO: u32 = 0x30;
const VERIFICACAO: u32 = 0x32;
const ANI_BURRO: u32 = 0xB4CE;

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            wb(m, a.wrapping_add(OB_ROUTINE), 2);
            wb(m, a.wrapping_add(OB_HEIGHT), 19);
            wb(m, a.wrapping_add(OB_WIDTH), 8);
            wl(m, a.wrapping_add(OB_MAP), 0xB4E6);
            ww(m, a.wrapping_add(OB_GFX), 0x04A6);
            let render = rb(m, a.wrapping_add(OB_RENDER)) | 4;
            wb(m, a.wrapping_add(OB_RENDER), render);
            wb(m, a.wrapping_add(OB_PRIORITY), 4);
            wb(m, a.wrapping_add(OB_COLTYPE), 5);
            wb(m, a.wrapping_add(OB_ACTWID), 12);
            let rotina = rb(m, a.wrapping_add(OB_ROUTINE_2)).wrapping_add(6);
            wb(m, a.wrapping_add(OB_ROUTINE_2), rotina);
            wb(m, a.wrapping_add(OB_ANIM), 2);
        }
        2 => {}
        _ => return Err(ErroAbertura::NaoImplementado("rotina do Burrobot")),
    }
    match rb(m, a.wrapping_add(OB_ROUTINE_2)) {
        0 => virar(m, a),
        2 => mover(m, a),
        4 => saltar(m, a),
        6 => verificar_sonic(m, a),
        _ => {
            return Err(ErroAbertura::NaoImplementado(
                "rotina secundaria do Burrobot",
            ));
        }
    }
    animate_sprite(m, a, ANI_BURRO)?;
    remember_state(m, a);
    Ok(())
}

fn virar(m: &mut Memoria, a: u32) {
    let tempo = rw(m, a.wrapping_add(TEMPO)).wrapping_sub(1);
    ww(m, a.wrapping_add(TEMPO), tempo);
    if i16::from_be_bytes(tempo.to_be_bytes()) >= 0 {
        return;
    }
    wb(m, a.wrapping_add(OB_ROUTINE_2), 2);
    ww(m, a.wrapping_add(TEMPO), 255);
    wb(m, a.wrapping_add(OB_ANIM), 1);
    let estado = rb(m, a.wrapping_add(OB_STATUS));
    wb(m, a.wrapping_add(OB_STATUS), estado ^ 1);
    ww(
        m,
        a.wrapping_add(OB_VELX),
        if estado & 1 == 0 { 0x80 } else { 0xff80 },
    );
}

fn mover(m: &mut Memoria, a: u32) {
    let tempo = rw(m, a.wrapping_add(TEMPO)).wrapping_sub(1);
    ww(m, a.wrapping_add(TEMPO), tempo);
    if i16::from_be_bytes(tempo.to_be_bytes()) < 0 {
        proxima_acao(m, a);
        return;
    }
    speed_to_pos(m, a);
    let tipo = rb(m, a.wrapping_add(VERIFICACAO));
    wb(m, a.wrapping_add(VERIFICACAO), tipo ^ 1);
    if tipo & 1 != 0 {
        let distancia = obj_floor_dist(m, a);
        let y = rw(m, a.wrapping_add(OB_Y)).wrapping_add(distancia);
        ww(m, a.wrapping_add(OB_Y), y);
    } else {
        let x = rw(m, a.wrapping_add(OB_X));
        let estado = rb(m, a.wrapping_add(OB_STATUS));
        let x = if estado & 1 != 0 {
            x.wrapping_add(12)
        } else {
            x.wrapping_sub(12)
        };
        let (distancia, _) = obj_floor_dist_and_angle(m, a, x);
        if distancia >= 12 {
            proxima_acao(m, a);
        }
    }
}

fn proxima_acao(m: &mut Memoria, a: u32) {
    if rb(m, V_VBLANK_BYTE) & 4 != 0 {
        wb(m, a.wrapping_add(OB_ROUTINE_2), 0);
        ww(m, a.wrapping_add(TEMPO), 59);
        ww(m, a.wrapping_add(OB_VELX), 0);
        wb(m, a.wrapping_add(OB_ANIM), 0);
    } else {
        wb(m, a.wrapping_add(OB_ROUTINE_2), 4);
        ww(m, a.wrapping_add(OB_VELY), 0xfc00);
        wb(m, a.wrapping_add(OB_ANIM), 2);
    }
}

fn saltar(m: &mut Memoria, a: u32) {
    speed_to_pos(m, a);
    let velocidade = rw(m, a.wrapping_add(OB_VELY)).wrapping_add(0x18);
    ww(m, a.wrapping_add(OB_VELY), velocidade);
    if i16::from_be_bytes(velocidade.to_be_bytes()) < 0 {
        return;
    }
    wb(m, a.wrapping_add(OB_ANIM), 3);
    let distancia = obj_floor_dist(m, a);
    if i16::from_be_bytes(distancia.to_be_bytes()) >= 0 {
        return;
    }
    let y = rw(m, a.wrapping_add(OB_Y)).wrapping_add(distancia);
    ww(m, a.wrapping_add(OB_Y), y);
    ww(m, a.wrapping_add(OB_VELY), 0);
    wb(m, a.wrapping_add(OB_ANIM), 1);
    ww(m, a.wrapping_add(TEMPO), 255);
    wb(m, a.wrapping_add(OB_ROUTINE_2), 2);
    virar_para_sonic(m, a);
}

fn virar_para_sonic(m: &mut Memoria, a: u32) -> (u16, u16) {
    let x = rw(m, a.wrapping_add(OB_X));
    let sonic = rw(m, V_PLAYER.wrapping_add(OB_X));
    let estado = rb(m, a.wrapping_add(OB_STATUS));
    if sonic >= x {
        wb(m, a.wrapping_add(OB_STATUS), estado | 1);
        (sonic.wrapping_sub(x), 0x80)
    } else {
        wb(m, a.wrapping_add(OB_STATUS), estado & !1);
        (x.wrapping_sub(sonic), 0xff80)
    }
}

fn verificar_sonic(m: &mut Memoria, a: u32) {
    let (distancia, velocidade) = virar_para_sonic(m, a);
    if distancia >= 96 {
        return;
    }
    let sonic_y = rw(m, V_PLAYER.wrapping_add(OB_Y));
    let y = rw(m, a.wrapping_add(OB_Y));
    let delta = sonic_y.wrapping_sub(y);
    if sonic_y >= y || delta < 0xff80 || rw(m, V_DEBUGUSE) != 0 {
        return;
    }
    wb(m, a.wrapping_add(OB_ROUTINE_2), 4);
    ww(m, a.wrapping_add(OB_VELX), velocidade);
    ww(m, a.wrapping_add(OB_VELY), 0xfc00);
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn inicializa_com_hitbox_de_burrobot() {
        let mut rom = vec![0; 0x20000];
        rom[0xb4d2..0xb4d4].copy_from_slice(&8u16.to_be_bytes());
        rom[0xb4d6..0xb4d9].copy_from_slice(&[5, 0, 0xff]);
        let mut m = Memoria::nova(rom);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x2d);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_COLTYPE)), 5);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE_2)), 6);
    }
    #[test]
    fn detecta_sonic_acima_e_salta_na_direcao_dele() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        ww(&mut m, a.wrapping_add(OB_X), 0x300);
        ww(&mut m, a.wrapping_add(OB_Y), 0x400);
        ww(&mut m, V_PLAYER.wrapping_add(OB_X), 0x2d0);
        ww(&mut m, V_PLAYER.wrapping_add(OB_Y), 0x380);
        verificar_sonic(&mut m, a);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE_2)), 4);
        assert_eq!(rw(&mut m, a.wrapping_add(OB_VELX)), 0xff80);
        assert_eq!(rw(&mut m, a.wrapping_add(OB_VELY)), 0xfc00);
    }
    #[test]
    fn nao_salta_se_sonic_esta_abaixo_ou_em_debug() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        ww(&mut m, a.wrapping_add(OB_Y), 0x400);
        ww(&mut m, V_PLAYER.wrapping_add(OB_Y), 0x401);
        verificar_sonic(&mut m, a);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE_2)), 0);
        ww(&mut m, V_PLAYER.wrapping_add(OB_Y), 0x3ff);
        ww(&mut m, V_DEBUGUSE, 1);
        verificar_sonic(&mut m, a);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE_2)), 0);
    }
    #[test]
    fn fim_da_patrulha_escolhe_salto_ou_pausa_pelo_vblank() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        proxima_acao(&mut m, a);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE_2)), 4);
        wb(&mut m, V_VBLANK_BYTE, 4);
        proxima_acao(&mut m, a);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE_2)), 0);
        assert_eq!(rw(&mut m, a.wrapping_add(TEMPO)), 59);
    }
}
