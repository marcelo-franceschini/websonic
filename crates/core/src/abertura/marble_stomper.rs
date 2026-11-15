//! Blocos suspensos em correntes de Marble Zone (objeto 31).
use super::boot::{rom_b, rom_w};
use super::objetos::{delete_object, display_sprite, fora_da_janela, solid_object};
use super::ram::*;
use super::{ErroAbertura, rb, rl, rw, wb, wl, ww};
use crate::memoria::Memoria;

const ORIG_Y: u32 = 0x30;
const ATUAL: u32 = 0x32;
const LIMITE: u32 = 0x34;
const SUBINDO: u32 = 0x36;
const ESPERA: u32 = 0x38;
const BOTAO: u32 = 0x3A;
const PAI: u32 = 0x3C;
const Y_GLOBAL: u32 = 0xFF_F7A4;
const SWITCH: u32 = 0xFF_F7E0;

pub fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => iniciar(m, a)?,
        2 => bloco(m, a)?,
        4 => filho(m, a, false),
        6 => desenhar(m, a),
        8 => filho(m, a, true),
        _ => {
            return Err(ErroAbertura::NaoImplementado(
                "rotina de bloco com corrente",
            ));
        }
    }
    Ok(())
}

fn proximo_livre(m: &mut Memoria, a: u32) -> Option<u32> {
    let mut b = a.wrapping_add(OBJECT_SIZE);
    while b < V_OBJSPACE_END {
        if rb(m, b.wrapping_add(OB_ID)) == 0 {
            return Some(b);
        }
        b = b.wrapping_add(OBJECT_SIZE);
    }
    None
}

fn iniciar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let mut subtipo = rb(m, a.wrapping_add(OB_SUBTYPE));
    if subtipo & 0x80 != 0 {
        let tabela = 0xBD1Eu32.wrapping_add(u32::from(subtipo & 0x7F).wrapping_mul(2));
        wb(m, a.wrapping_add(BOTAO), rom_b(m, tabela));
        subtipo = rom_b(m, tabela.wrapping_add(1));
        wb(m, a.wrapping_add(OB_SUBTYPE), subtipo);
    }
    let comprimento = rom_w(
        m,
        0xBD2Eu32.wrapping_add(u32::from(subtipo & 15).wrapping_mul(2)),
    );
    if subtipo & 15 == 0 {
        ww(m, a.wrapping_add(ATUAL), comprimento);
    }
    let x = rw(m, a.wrapping_add(OB_X));
    let y = rw(m, a.wrapping_add(OB_Y));
    let mut b = a;
    for i in 0..4u32 {
        let anterior = b;
        if i != 0 {
            let Some(livre) = proximo_livre(m, b) else {
                break;
            };
            b = livre;
        }
        let t = 0xBD22u32.wrapping_add(i.wrapping_mul(3));
        let rotina = rom_b(m, t);
        let desloc = u16::from_be_bytes(
            i16::from(i8::from_be_bytes([rom_b(m, t.wrapping_add(1))])).to_be_bytes(),
        );
        let frame = rom_b(m, t.wrapping_add(2));
        if frame == 1 && subtipo & 0xF0 == 0x20 {
            b = anterior;
            continue;
        }
        wb(m, b.wrapping_add(OB_ROUTINE), rotina);
        wb(m, b.wrapping_add(OB_ID), 0x31);
        ww(m, b.wrapping_add(OB_X), x);
        let y_parte = y.wrapping_add(desloc);
        ww(m, b.wrapping_add(OB_Y), y_parte);
        wl(m, b.wrapping_add(OB_MAP), 0xC1D8);
        ww(m, b.wrapping_add(OB_GFX), 0x300);
        wb(m, b.wrapping_add(OB_RENDER), 4);
        ww(m, b.wrapping_add(ORIG_Y), y_parte);
        wb(m, b.wrapping_add(OB_SUBTYPE), subtipo);
        wb(
            m,
            b.wrapping_add(OB_ACTWID),
            if frame == 1 { 56 } else { 16 },
        );
        ww(m, b.wrapping_add(LIMITE), comprimento);
        wb(
            m,
            b.wrapping_add(OB_PRIORITY),
            if frame == 2 { 3 } else { 4 },
        );
        wb(m, b.wrapping_add(OB_FRAME), frame);
        if frame == 1 {
            wb(m, b.wrapping_add(OB_COLTYPE), 0x90);
        }
        wl(m, b.wrapping_add(PAI), a);
    }
    let tabela = 0xBE1Cu32.wrapping_add(u32::from((subtipo >> 3) & 14));
    wb(m, a.wrapping_add(OB_ACTWID), rom_b(m, tabela));
    wb(
        m,
        a.wrapping_add(OB_FRAME),
        rom_b(m, tabela.wrapping_add(1)),
    );
    bloco(m, a)
}

fn bloco(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let tipo = rb(m, a.wrapping_add(OB_SUBTYPE)) & 15;
    if tipo == 0 {
        let numero = rb(m, a.wrapping_add(BOTAO));
        let ligado = rb(m, SWITCH.wrapping_add(u32::from(numero))) != 0;
        if ligado {
            let atual = rw(m, a.wrapping_add(ATUAL));
            let bloqueado = rw(m, Y_GLOBAL) & 0x8000 != 0 && atual >> 8 == 0x10;
            if atual != 0 && !bloqueado {
                subir(m, a, false);
            }
        } else {
            cair(m, a, false);
        }
    } else if tipo == 3 || tipo == 5 {
        let d = rw(m, V_PLAYER.wrapping_add(OB_X)).wrapping_sub(rw(m, a.wrapping_add(OB_X)));
        if d.min(d.wrapping_neg()) < 144 {
            let subtipo = rb(m, a.wrapping_add(OB_SUBTYPE)).wrapping_add(1);
            wb(m, a.wrapping_add(OB_SUBTYPE), subtipo);
        }
    } else if rw(m, a.wrapping_add(SUBINDO)) != 0 {
        let atraso = rw(m, a.wrapping_add(ESPERA));
        if atraso != 0 {
            ww(m, a.wrapping_add(ESPERA), atraso.wrapping_sub(1));
        } else {
            subir(m, a, true);
        }
    } else {
        cair(m, a, true);
    }
    let atual = u16::from(rb(m, a.wrapping_add(ATUAL)));
    let y = rw(m, a.wrapping_add(ORIG_Y)).wrapping_add(atual);
    ww(m, a.wrapping_add(OB_Y), y);
    ww(m, Y_GLOBAL, y);
    let largura = u16::from(rb(m, a.wrapping_add(OB_ACTWID))).wrapping_add(11);
    solid_object(m, a, largura, 12, 13)?;
    if rb(m, a.wrapping_add(OB_STATUS)) & 8 != 0 && atual < 16 {
        super::fisica::kill_sonic(m);
    }
    desenhar(m, a);
    Ok(())
}

fn subir(m: &mut Memoria, a: u32, auto: bool) {
    let atual = rw(m, a.wrapping_add(ATUAL));
    if rb(m, V_VBLANK_BYTE) & 15 == 0 && rb(m, a.wrapping_add(OB_RENDER)) & 0x80 != 0 {
        wb(m, V_SOUNDQUEUE1, 0xC7);
    }
    if atual < 0x80 {
        ww(m, a.wrapping_add(ATUAL), 0);
        ww(m, a.wrapping_add(OB_VELY), 0);
        if auto {
            ww(m, a.wrapping_add(SUBINDO), 0);
        }
    } else {
        ww(m, a.wrapping_add(ATUAL), atual.wrapping_sub(0x80));
    }
}
fn cair(m: &mut Memoria, a: u32, auto: bool) {
    let atual = rw(m, a.wrapping_add(ATUAL));
    let limite = rw(m, a.wrapping_add(LIMITE));
    if atual == limite {
        return;
    }
    let velocidade = rw(m, a.wrapping_add(OB_VELY));
    ww(m, a.wrapping_add(OB_VELY), velocidade.wrapping_add(0x70));
    let proximo = atual.wrapping_add(velocidade);
    if proximo >= limite {
        ww(m, a.wrapping_add(ATUAL), limite);
        ww(m, a.wrapping_add(OB_VELY), 0);
        if auto {
            ww(m, a.wrapping_add(SUBINDO), 1);
            ww(m, a.wrapping_add(ESPERA), 60);
        }
        if rb(m, a.wrapping_add(OB_RENDER)) & 0x80 != 0 {
            wb(m, V_SOUNDQUEUE1, 0xBD);
        }
    } else {
        ww(m, a.wrapping_add(ATUAL), proximo);
    }
}
fn filho(m: &mut Memoria, a: u32, corrente: bool) {
    let pai = rl(m, a.wrapping_add(PAI));
    let distancia = rb(m, pai.wrapping_add(ATUAL));
    if corrente {
        wb(m, a.wrapping_add(OB_HEIGHT), 128);
        let render = rb(m, a.wrapping_add(OB_RENDER)) | 0x10;
        wb(m, a.wrapping_add(OB_RENDER), render);
        wb(
            m,
            a.wrapping_add(OB_FRAME),
            (distancia >> 5).wrapping_add(3),
        );
    }
    let y = rw(m, a.wrapping_add(ORIG_Y)).wrapping_add(u16::from(distancia));
    ww(m, a.wrapping_add(OB_Y), y);
    desenhar(m, a);
}
fn desenhar(m: &mut Memoria, a: u32) {
    display_sprite(m, a);
    let x = rw(m, a.wrapping_add(OB_X));
    if fora_da_janela(m, x) {
        delete_object(m, a);
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn monta_pecas_e_inicia_movimento_automatico() {
        let mut rom = vec![0; 0x10000];
        rom[0xBD22..0xBD2E].copy_from_slice(&[2, 0, 0, 4, 28, 1, 8, 0xCC, 3, 6, 0xF0, 2]);
        rom[0xBD30..0xBD32].copy_from_slice(&0xA000u16.to_be_bytes());
        rom[0xBE1C..0xBE1E].copy_from_slice(&[56, 0]);
        let mut m = Memoria::nova(rom);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x31);
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 1);
        ww(&mut m, a.wrapping_add(OB_X), 0x100);
        ww(&mut m, a.wrapping_add(OB_Y), 0x200);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(
            rb(&mut m, a.wrapping_add(OBJECT_SIZE).wrapping_add(OB_COLTYPE)),
            0x90
        );
        assert_eq!(rw(&mut m, a.wrapping_add(LIMITE)), 0xA000);
        assert_eq!(rw(&mut m, a.wrapping_add(OB_VELY)), 0x70);
        executar(&mut m, a).unwrap();
        assert_eq!(rw(&mut m, a.wrapping_add(ATUAL)), 0x70);
    }
}
