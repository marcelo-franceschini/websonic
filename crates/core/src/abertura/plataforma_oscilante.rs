//! Plataforma pendular e elos, objeto 15.
use super::fisica::{calc_sine, sonic_reset_on_floor};
use super::objetos::{delete_object, display_sprite, find_free_obj, fora_da_janela};
use super::ram::*;
use super::{ErroAbertura, rb, rl, rw, wb, wl, ww};
use crate::memoria::Memoria;

const ORIG_Y: u32 = 0x38;
const ORIG_X: u32 = 0x3A;
const RAIO: u32 = 0x3C;
const OSCILACAO: u32 = 0xFF_FE78;

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => iniciar(m, a)?,
        2 => {
            detectar(m, a);
            oscilar(m, a);
            exibir_e_remover(m, a);
        }
        4 => {
            sair(m, a);
            let anterior = rw(m, a.wrapping_add(OB_X));
            oscilar(m, a);
            mover_sonic(m, a, anterior);
            exibir_e_remover(m, a);
        }
        6 | 8 => delete_object(m, a),
        10 => display_sprite(m, a),
        12 => {
            oscilar(m, a);
            exibir_e_remover(m, a);
        }
        _ => {
            return Err(ErroAbertura::NaoImplementado(
                "rotina da plataforma oscilante",
            ));
        }
    }
    Ok(())
}

fn iniciar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    wb(m, a.wrapping_add(OB_ROUTINE), 2);
    wl(m, a.wrapping_add(OB_MAP), 0x82C6);
    ww(m, a.wrapping_add(OB_GFX), 0x4380);
    wb(m, a.wrapping_add(OB_RENDER), 4);
    wb(m, a.wrapping_add(OB_PRIORITY), 3);
    wb(m, a.wrapping_add(OB_ACTWID), 24);
    wb(m, a.wrapping_add(OB_HEIGHT), 8);
    let y = rw(m, a.wrapping_add(OB_Y));
    let x = rw(m, a.wrapping_add(OB_X));
    ww(m, a.wrapping_add(ORIG_Y), y);
    ww(m, a.wrapping_add(ORIG_X), x);
    let zona = rb(m, V_ZONE);
    if zona == 3 {
        wl(m, a.wrapping_add(OB_MAP), 0x82E4);
        ww(m, a.wrapping_add(OB_GFX), 0x43DC);
        wb(m, a.wrapping_add(OB_ACTWID), 32);
        wb(m, a.wrapping_add(OB_HEIGHT), 16);
        wb(m, a.wrapping_add(OB_COLTYPE), 0x99);
    } else if zona == 5 {
        wl(m, a.wrapping_add(OB_MAP), 0x11174);
        ww(m, a.wrapping_add(OB_GFX), 0x391);
        wb(m, a.wrapping_add(OB_HEIGHT), 24);
        wb(m, a.wrapping_add(OB_COLTYPE), 0x86);
        wb(m, a.wrapping_add(OB_ROUTINE), 12);
    }
    let subtipo = rb(m, a.wrapping_add(OB_SUBTYPE));
    let n = subtipo & 15;
    wb(m, a.wrapping_add(OB_SUBTYPE), 0);
    let mut raio = n.wrapping_mul(16).wrapping_add(8);
    wb(m, a.wrapping_add(RAIO), raio);
    raio = raio.wrapping_sub(8);
    let mut quantidade = u16::from(n).wrapping_add(1);
    if rb(m, a.wrapping_add(OB_FRAME)) != 0 {
        raio = raio.wrapping_add(8);
        quantidade = quantidade.wrapping_sub(1);
    }
    for _ in 0..quantidade {
        let Some(b) = find_free_obj(m) else { break };
        let count = rb(m, a.wrapping_add(OB_SUBTYPE)).wrapping_add(1);
        wb(m, a.wrapping_add(OB_SUBTYPE), count);
        let indice = u8::try_from((b.wrapping_sub(V_OBJSPACE) / OBJECT_SIZE) & 0x7F).unwrap_or(0);
        wb(
            m,
            a.wrapping_add(OB_SUBTYPE).wrapping_add(u32::from(count)),
            indice,
        );
        let id = rb(m, a.wrapping_add(OB_ID));
        let mapa = rl(m, a.wrapping_add(OB_MAP));
        let gfx = rw(m, a.wrapping_add(OB_GFX));
        wb(m, b.wrapping_add(OB_ROUTINE), 10);
        wb(m, b.wrapping_add(OB_ID), id);
        wl(m, b.wrapping_add(OB_MAP), mapa);
        ww(m, b.wrapping_add(OB_GFX), gfx & !0x4000);
        wb(m, b.wrapping_add(OB_RENDER), 4);
        wb(m, b.wrapping_add(OB_PRIORITY), 4);
        wb(m, b.wrapping_add(OB_ACTWID), 8);
        wb(m, b.wrapping_add(OB_FRAME), 1);
        wb(m, b.wrapping_add(RAIO), raio);
        let anterior = raio;
        raio = raio.wrapping_sub(16);
        if anterior < 16 {
            wb(m, b.wrapping_add(OB_FRAME), 2);
            wb(m, b.wrapping_add(OB_PRIORITY), 3);
            ww(m, b.wrapping_add(OB_GFX), gfx | 0x4000);
        }
    }
    let count = rb(m, a.wrapping_add(OB_SUBTYPE));
    let indice = u8::try_from((a.wrapping_sub(V_OBJSPACE) / OBJECT_SIZE) & 0x7F).unwrap_or(0);
    wb(
        m,
        a.wrapping_add(OB_SUBTYPE)
            .wrapping_add(u32::from(count))
            .wrapping_add(1),
        indice,
    );
    ww(m, a.wrapping_add(OB_ANGLE), 0x4080);
    ww(m, a.wrapping_add(0x3E), 0xFE00);
    if subtipo & 0x10 != 0 {
        wl(m, a.wrapping_add(OB_MAP), 0x8828);
        ww(m, a.wrapping_add(OB_GFX), 0x43AA);
        wb(m, a.wrapping_add(OB_FRAME), 1);
        wb(m, a.wrapping_add(OB_PRIORITY), 2);
        wb(m, a.wrapping_add(OB_COLTYPE), 0x81);
    }
    if zona == 5 {
        oscilar(m, a);
        exibir_e_remover(m, a);
    } else {
        detectar(m, a);
        oscilar(m, a);
        exibir_e_remover(m, a);
    }
    Ok(())
}

fn detectar(m: &mut Memoria, a: u32) {
    let altura = u16::from(rb(m, a.wrapping_add(OB_HEIGHT)));
    detectar_altura(m, a, altura);
}

pub(super) fn detectar_altura(m: &mut Memoria, a: u32, altura: u16) {
    if i16::from_be_bytes(rw(m, V_PLAYER.wrapping_add(OB_VELY)).to_be_bytes()) < 0 {
        return;
    }
    let largura = u16::from(rb(m, a.wrapping_add(OB_ACTWID)));
    let pos = rw(m, V_PLAYER.wrapping_add(OB_X))
        .wrapping_sub(rw(m, a.wrapping_add(OB_X)))
        .wrapping_add(largura);
    if i16::from_be_bytes(pos.to_be_bytes()) < 0 || pos >= largura.wrapping_mul(2) {
        return;
    }
    let py = rw(m, V_PLAYER.wrapping_add(OB_Y));
    let ph = i16::from(i8::from_be_bytes([rb(m, V_PLAYER.wrapping_add(OB_HEIGHT))]));
    let distancia = rw(m, a.wrapping_add(OB_Y))
        .wrapping_sub(altura)
        .wrapping_sub(
            py.wrapping_add(u16::from_be_bytes(ph.to_be_bytes()))
                .wrapping_add(4),
        );
    if distancia != 0 && distancia < 0xFFF0 {
        return;
    }
    if rb(m, F_PLAYERCTRL) & 0x80 != 0 || rb(m, V_PLAYER.wrapping_add(OB_ROUTINE)) >= 6 {
        return;
    }
    ww(
        m,
        V_PLAYER.wrapping_add(OB_Y),
        py.wrapping_add(distancia).wrapping_add(3),
    );
    wb(m, a.wrapping_add(OB_ROUTINE), 4);
    assentar(m, a);
}

/// `Plat_NoCheck`: associa Sonic à plataforma após a colisão já ter sido verificada.
pub(super) fn assentar(m: &mut Memoria, a: u32) {
    let status = rb(m, V_PLAYER.wrapping_add(OB_STATUS));
    if status & STATUS_ON_OBJECT != 0 {
        let indice = u32::from(rb(m, V_PLAYER.wrapping_add(SON_STANDONOBJECT)));
        let anterior = V_OBJSPACE.wrapping_add(indice.wrapping_mul(OBJECT_SIZE));
        let st = rb(m, anterior.wrapping_add(OB_STATUS)) & !STATUS_ON_OBJECT;
        wb(m, anterior.wrapping_add(OB_STATUS), st);
        wb(m, anterior.wrapping_add(OB_ROUTINE_2), 0);
        if rb(m, anterior.wrapping_add(OB_ROUTINE)) == 4 {
            wb(m, anterior.wrapping_add(OB_ROUTINE), 2);
        }
    }
    let indice = u8::try_from((a.wrapping_sub(V_OBJSPACE) / OBJECT_SIZE) & 0x7F).unwrap_or(0);
    wb(m, V_PLAYER.wrapping_add(SON_STANDONOBJECT), indice);
    wb(m, V_PLAYER.wrapping_add(OB_ANGLE), 0);
    ww(m, V_PLAYER.wrapping_add(OB_VELY), 0);
    let vx = rw(m, V_PLAYER.wrapping_add(OB_VELX));
    ww(m, V_PLAYER.wrapping_add(OB_INERTIA), vx);
    if status & STATUS_IN_AIR != 0 {
        sonic_reset_on_floor(m, V_PLAYER);
    }
    let ps = rb(m, V_PLAYER.wrapping_add(OB_STATUS)) | STATUS_ON_OBJECT;
    let os = rb(m, a.wrapping_add(OB_STATUS)) | STATUS_ON_OBJECT;
    wb(m, V_PLAYER.wrapping_add(OB_STATUS), ps);
    wb(m, a.wrapping_add(OB_STATUS), os);
}

pub(super) fn sair(m: &mut Memoria, a: u32) {
    let largura = u16::from(rb(m, a.wrapping_add(OB_ACTWID)));
    let ps = rb(m, V_PLAYER.wrapping_add(OB_STATUS));
    let pos = rw(m, V_PLAYER.wrapping_add(OB_X))
        .wrapping_sub(rw(m, a.wrapping_add(OB_X)))
        .wrapping_add(largura);
    if ps & STATUS_IN_AIR != 0
        || i16::from_be_bytes(pos.to_be_bytes()) < 0
        || pos >= largura.wrapping_mul(2)
    {
        wb(m, V_PLAYER.wrapping_add(OB_STATUS), ps & !STATUS_ON_OBJECT);
        wb(m, a.wrapping_add(OB_ROUTINE), 2);
        let os = rb(m, a.wrapping_add(OB_STATUS)) & !STATUS_ON_OBJECT;
        wb(m, a.wrapping_add(OB_STATUS), os);
    }
}

fn oscilar(m: &mut Memoria, a: u32) {
    let mut angulo = rb(m, OSCILACAO);
    if rb(m, a.wrapping_add(OB_STATUS)) & 1 != 0 {
        angulo = 0x80u8.wrapping_sub(angulo);
    }
    let (seno, cosseno) = calc_sine(m, angulo);
    let y = rw(m, a.wrapping_add(ORIG_Y));
    let x = rw(m, a.wrapping_add(ORIG_X));
    let count = rb(m, a.wrapping_add(OB_SUBTYPE));
    for i in 0..=u32::from(count) {
        let idx = rb(
            m,
            a.wrapping_add(OB_SUBTYPE).wrapping_add(1).wrapping_add(i),
        );
        let b = V_OBJSPACE.wrapping_add(u32::from(idx).wrapping_mul(OBJECT_SIZE));
        let raio = i32::from(rb(m, b.wrapping_add(RAIO)));
        let dy = raio.wrapping_mul(i32::from(seno)) >> 8;
        let dx = raio.wrapping_mul(i32::from(cosseno)) >> 8;
        let dy = u16::try_from(u32::from_be_bytes(dy.to_be_bytes()) & 0xFFFF).unwrap_or(0);
        let dx = u16::try_from(u32::from_be_bytes(dx.to_be_bytes()) & 0xFFFF).unwrap_or(0);
        ww(m, b.wrapping_add(OB_Y), y.wrapping_add(dy));
        ww(m, b.wrapping_add(OB_X), x.wrapping_add(dx));
    }
}

fn mover_sonic(m: &mut Memoria, a: u32, anterior: u16) {
    let altura = u16::from(rb(m, a.wrapping_add(OB_HEIGHT))).wrapping_add(1);
    mover_sonic_altura(m, a, anterior, altura);
}

pub(super) fn mover_sonic_altura(m: &mut Memoria, a: u32, anterior: u16, altura: u16) {
    if rb(m, F_PLAYERCTRL) & 0x80 != 0
        || rb(m, V_PLAYER.wrapping_add(OB_ROUTINE)) >= 6
        || rw(m, V_DEBUGUSE) != 0
    {
        return;
    }
    let y = rw(m, a.wrapping_add(OB_Y))
        .wrapping_sub(altura)
        .wrapping_sub(u16::from(rb(m, V_PLAYER.wrapping_add(OB_HEIGHT))));
    let x = rw(m, V_PLAYER.wrapping_add(OB_X))
        .wrapping_add(rw(m, a.wrapping_add(OB_X)).wrapping_sub(anterior));
    ww(m, V_PLAYER.wrapping_add(OB_Y), y);
    ww(m, V_PLAYER.wrapping_add(OB_X), x);
}

fn exibir_e_remover(m: &mut Memoria, a: u32) {
    display_sprite(m, a);
    let origem = rw(m, a.wrapping_add(ORIG_X));
    if !fora_da_janela(m, origem) {
        return;
    }
    let count = rb(m, a.wrapping_add(OB_SUBTYPE));
    for i in 0..=u32::from(count) {
        let indice = rb(
            m,
            a.wrapping_add(OB_SUBTYPE).wrapping_add(1).wrapping_add(i),
        );
        let b = V_OBJSPACE.wrapping_add(u32::from(indice).wrapping_mul(OBJECT_SIZE));
        delete_object(m, b);
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn cria_corrente_e_atualiza_todas_as_posicoes() {
        let mut rom = vec![0; 0x10000];
        let sine = usize::try_from(SINE_DATA).unwrap();
        rom[sine..sine.wrapping_add(2)].copy_from_slice(&256i16.to_be_bytes());
        let mut m = Memoria::nova(rom);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x15);
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 2);
        ww(&mut m, a.wrapping_add(OB_X), 0x100);
        ww(&mut m, a.wrapping_add(OB_Y), 0x100);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_SUBTYPE)), 3);
        assert_eq!(rb(&mut m, a.wrapping_add(RAIO)), 40);
        assert_eq!(rw(&mut m, a.wrapping_add(OB_Y)), 0x128);
        let b = a.wrapping_add(OBJECT_SIZE);
        assert_eq!(rb(&mut m, b.wrapping_add(OB_ROUTINE)), 10);
        assert_eq!(rw(&mut m, b.wrapping_add(OB_Y)), 0x120);
        let ancora = b.wrapping_add(OBJECT_SIZE.wrapping_mul(2));
        assert_eq!(rb(&mut m, ancora.wrapping_add(OB_FRAME)), 2);
    }
}
