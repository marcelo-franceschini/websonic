//! Bolhas de ar da Labyrinth, objeto 0x64 (REV01).
use super::super::boot::rom_b;
use super::super::fisica::{random_number, speed_to_pos};
use super::super::ram::*;
use super::super::{ErroAbertura, rb, rl, rw, wb, wl, ww};
use super::{animate_sprite, delete_object, display_sprite, find_free_obj, fora_da_janela};
use crate::memoria::Memoria;

const RESPIRAVEL: u32 = 0x2e;
const ORIG_X: u32 = 0x30;
const TEMPO: u32 = 0x32;
const BASE: u32 = 0x33;
const CONTAGEM: u32 = 0x34;
const FLAGS: u32 = 0x36;
const ATRASO: u32 = 0x38;
const LISTA: u32 = 0x3c;
const AGUA: u32 = 0xFF_F646;
const ANI: u32 = 0x13070;
const TIPOS: u32 = 0x13020;
const OSCILACAO: u32 = 0x144f0;

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let mut rotina = rb(m, a.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        wb(m, a.wrapping_add(OB_ROUTINE), 2);
        wl(m, a.wrapping_add(OB_MAP), 0x130a0);
        ww(m, a.wrapping_add(OB_GFX), 0x8348);
        wb(m, a.wrapping_add(OB_RENDER), 0x84);
        wb(m, a.wrapping_add(OB_ACTWID), 16);
        wb(m, a.wrapping_add(OB_PRIORITY), 1);
        let subtipo = rb(m, a.wrapping_add(OB_SUBTYPE));
        if subtipo & 0x80 != 0 {
            rotina = 10;
            wb(m, a.wrapping_add(OB_ROUTINE), rotina);
            wb(m, a.wrapping_add(TEMPO), subtipo & 0x7f);
            wb(m, a.wrapping_add(BASE), subtipo & 0x7f);
            wb(m, a.wrapping_add(OB_ANIM), 6);
        } else {
            rotina = 2;
            wb(m, a.wrapping_add(OB_ANIM), subtipo);
            let x = rw(m, a.wrapping_add(OB_X));
            ww(m, a.wrapping_add(ORIG_X), x);
            ww(m, a.wrapping_add(OB_VELY), 0xff78);
            let sorteio = random_number(m).to_be_bytes()[1];
            wb(m, a.wrapping_add(OB_ANGLE), sorteio);
        }
    }
    match rotina {
        2 => {
            animate_sprite(m, a, ANI)?;
            if rb(m, a.wrapping_add(OB_FRAME)) == 6 {
                wb(m, a.wrapping_add(RESPIRAVEL), 1);
            }
            verificar_agua(m, a)
        }
        4 => verificar_agua(m, a),
        6 => estourando(m, a),
        8 => {
            delete_object(m, a);
            Ok(())
        }
        10 => gerador(m, a),
        _ => Err(ErroAbertura::NaoImplementado(
            "rotina de bolha da Labyrinth",
        )),
    }
}

fn verificar_agua(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let agua = rw(m, AGUA);
    let y = rw(m, a.wrapping_add(OB_Y));
    if agua >= y {
        return estourar(m, a);
    }
    let angulo = rb(m, a.wrapping_add(OB_ANGLE));
    wb(m, a.wrapping_add(OB_ANGLE), angulo.wrapping_add(1));
    let desloc = i16::from(i8::from_be_bytes([rom_b(
        m,
        OSCILACAO.wrapping_add(u32::from(angulo & 0x7f)),
    )]));
    let x = rw(m, a.wrapping_add(ORIG_X)).wrapping_add(u16::from_be_bytes(desloc.to_be_bytes()));
    ww(m, a.wrapping_add(OB_X), x);
    if rb(m, a.wrapping_add(RESPIRAVEL)) != 0 && tocou(m, a) {
        super::super::sonic_agua::retomar_musica(m);
        wb(m, V_SOUNDQUEUE1, 0xad);
        for campo in [OB_VELX, OB_VELY, OB_INERTIA] {
            ww(m, V_PLAYER.wrapping_add(campo), 0);
        }
        wb(m, V_PLAYER.wrapping_add(OB_ANIM), 0x15);
        ww(m, V_PLAYER.wrapping_add(SON_LOCKTIME), 35);
        wb(m, V_PLAYER.wrapping_add(SON_JUMPING), 0);
        let status = rb(m, V_PLAYER.wrapping_add(OB_STATUS));
        wb(m, V_PLAYER.wrapping_add(OB_STATUS), status & !0x34);
        if status & 4 != 0 {
            wb(m, V_PLAYER.wrapping_add(OB_HEIGHT), 19);
            wb(m, V_PLAYER.wrapping_add(OB_WIDTH), 9);
            let y = rw(m, V_PLAYER.wrapping_add(OB_Y)).wrapping_sub(5);
            ww(m, V_PLAYER.wrapping_add(OB_Y), y);
        }
        return estourar(m, a);
    }
    speed_to_pos(m, a);
    exibir_ou_apagar(m, a);
    Ok(())
}

fn tocou(m: &mut Memoria, a: u32) -> bool {
    if rb(m, F_PLAYERCTRL) & 0x80 != 0 {
        return false;
    }
    let x = rw(m, a.wrapping_add(OB_X));
    let y = rw(m, a.wrapping_add(OB_Y));
    let sx = rw(m, V_PLAYER.wrapping_add(OB_X));
    let sy = rw(m, V_PLAYER.wrapping_add(OB_Y));
    x.wrapping_sub(16) < sx && x.wrapping_add(16) >= sx && y < sy && y.wrapping_add(16) >= sy
}

fn estourar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    wb(m, a.wrapping_add(OB_ROUTINE), 6);
    let anim = rb(m, a.wrapping_add(OB_ANIM)).wrapping_add(3);
    wb(m, a.wrapping_add(OB_ANIM), anim);
    estourando(m, a)
}
fn estourando(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    animate_sprite(m, a, ANI)?;
    exibir_ou_apagar(m, a);
    Ok(())
}
fn exibir_ou_apagar(m: &mut Memoria, a: u32) {
    if rb(m, a.wrapping_add(OB_RENDER)) & 0x80 == 0 {
        delete_object(m, a);
    } else {
        display_sprite(m, a);
    }
}

fn gerador(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let flags = rw(m, a.wrapping_add(FLAGS));
    if flags == 0 {
        let agua = rw(m, AGUA);
        let y = rw(m, a.wrapping_add(OB_Y));
        if agua >= y || rb(m, a.wrapping_add(OB_RENDER)) & 0x80 == 0 {
            return exibir_gerador(m, a);
        }
    }
    let tempo = rw(m, a.wrapping_add(ATRASO)).wrapping_sub(1);
    ww(m, a.wrapping_add(ATRASO), tempo);
    if i16::from_be_bytes(tempo.to_be_bytes()) >= 0 {
        return animar_gerador(m, a);
    }
    if flags == 0 {
        ww(m, a.wrapping_add(FLAGS), 1);
        let sorteio = loop {
            let n = random_number(m);
            if n & 7 < 6 {
                break n;
            }
        };
        wb(m, a.wrapping_add(CONTAGEM), sorteio.to_be_bytes()[1] & 7);
        wl(
            m,
            a.wrapping_add(LISTA),
            TIPOS.wrapping_add(u32::from(sorteio & 12)),
        );
        let tempo = rb(m, a.wrapping_add(TEMPO)).wrapping_sub(1);
        wb(m, a.wrapping_add(TEMPO), tempo);
        if tempo & 0x80 != 0 {
            let base = rb(m, a.wrapping_add(BASE));
            wb(m, a.wrapping_add(TEMPO), base);
            wb(m, a.wrapping_add(FLAGS), 0x80);
        }
    }
    gerar_bolha(m, a);
    animar_gerador(m, a)
}

fn gerar_bolha(m: &mut Memoria, a: u32) {
    let intervalo = random_number(m) & 0x1f;
    ww(m, a.wrapping_add(ATRASO), intervalo);
    if let Some(filho) = find_free_obj(m) {
        wb(m, filho.wrapping_add(OB_ID), 0x64);
        let x = rw(m, a.wrapping_add(OB_X));
        let dx = (random_number(m) & 15).wrapping_sub(8);
        ww(m, filho.wrapping_add(OB_X), x.wrapping_add(dx));
        let y = rw(m, a.wrapping_add(OB_Y));
        ww(m, filho.wrapping_add(OB_Y), y);
        let contador = rb(m, a.wrapping_add(CONTAGEM));
        let lista = rl(m, a.wrapping_add(LISTA));
        let tipo = rom_b(m, lista.wrapping_add(u32::from(contador)));
        wb(m, filho.wrapping_add(OB_SUBTYPE), tipo);
        let flags = rb(m, a.wrapping_add(FLAGS));
        if flags & 0x80 != 0 {
            let sorte = random_number(m) & 3;
            if (sorte == 0 || contador == 0) && flags & 0x40 == 0 {
                wb(m, a.wrapping_add(FLAGS), flags | 0x40);
                wb(m, filho.wrapping_add(OB_SUBTYPE), 2);
            }
        }
    }
    let restante = rb(m, a.wrapping_add(CONTAGEM)).wrapping_sub(1);
    wb(m, a.wrapping_add(CONTAGEM), restante);
    if restante & 0x80 != 0 {
        let atraso = rw(m, a.wrapping_add(ATRASO))
            .wrapping_add((random_number(m) & 0x7f).wrapping_add(0x80));
        ww(m, a.wrapping_add(ATRASO), atraso);
        ww(m, a.wrapping_add(FLAGS), 0);
    }
}
fn animar_gerador(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    animate_sprite(m, a, ANI)?;
    exibir_gerador(m, a)
}
fn exibir_gerador(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let x = rw(m, a.wrapping_add(OB_X));
    if fora_da_janela(m, x) {
        delete_object(m, a);
    } else if rw(m, AGUA) < rw(m, a.wrapping_add(OB_Y)) {
        display_sprite(m, a);
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn bolha_grande_restaura_ar_e_desfaz_rolamento() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x64);
        wb(&mut m, a.wrapping_add(OB_RENDER), 0x84);
        wb(&mut m, a.wrapping_add(RESPIRAVEL), 1);
        ww(&mut m, a.wrapping_add(ORIG_X), 200);
        ww(&mut m, a.wrapping_add(OB_Y), 300);
        ww(&mut m, AGUA, 100);
        ww(&mut m, V_PLAYER.wrapping_add(OB_X), 200);
        ww(&mut m, V_PLAYER.wrapping_add(OB_Y), 308);
        wb(&mut m, V_PLAYER.wrapping_add(OB_STATUS), 4);
        ww(&mut m, V_AIR, 10);
        verificar_agua(&mut m, a).unwrap();
        assert_eq!(rw(&mut m, V_AIR), 30);
        assert_eq!(rw(&mut m, V_PLAYER.wrapping_add(SON_LOCKTIME)), 35);
        assert_eq!(rw(&mut m, V_PLAYER.wrapping_add(OB_Y)), 303);
        assert_eq!(rb(&mut m, V_SOUNDQUEUE1), 0xad);
    }
    #[test]
    fn ultima_bolha_garante_ar_quando_necessario() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x64);
        ww(&mut m, a.wrapping_add(FLAGS), 0x8001);
        wl(&mut m, a.wrapping_add(LISTA), TIPOS);
        gerar_bolha(&mut m, a);
        let filha = a.wrapping_add(OBJECT_SIZE);
        assert_eq!(rb(&mut m, filha.wrapping_add(OB_ID)), 0x64);
        assert_eq!(rb(&mut m, filha.wrapping_add(OB_SUBTYPE)), 2);
        assert_eq!(rw(&mut m, a.wrapping_add(FLAGS)), 0);
    }
    #[test]
    fn gerador_fora_da_agua_nao_produz_bolhas() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x64);
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 0x80);
        ww(&mut m, a.wrapping_add(OB_Y), 100);
        ww(&mut m, AGUA, 100);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE)), 10);
        assert_eq!(
            rb(&mut m, a.wrapping_add(OBJECT_SIZE).wrapping_add(OB_ID)),
            0
        );
    }
}
