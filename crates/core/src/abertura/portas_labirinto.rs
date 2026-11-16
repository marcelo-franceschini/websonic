//! Portas de Labyrinth do objeto 0x56 (`FloatingBlock`, REV01).
use super::super::ram::*;
use super::super::{ErroAbertura, rb, rw, wb, wl, ww};
use super::{delete_object, display_sprite, fora_da_janela, solid_object};
use crate::memoria::Memoria;

const ORIG_Y: u32 = 0x30;
const ORIG_X: u32 = 0x34;
const MOVENDO: u32 = 0x38;
const DISTANCIA: u32 = 0x3a;
const CHAVE: u32 = 0x3c;
const F_SWITCH: u32 = 0xFF_F7E0;
const F_WTUNNELDISALLOW: u32 = 0xFF_F7C9;

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => iniciar(m, a)?,
        2 => {}
        _ => {
            return Err(ErroAbertura::NaoImplementado(
                "rotina da porta de Labyrinth",
            ));
        }
    }
    let tipo = rb(m, a.wrapping_add(OB_SUBTYPE)) & 15;
    match tipo {
        5 | 6 | 12 | 13 => mover(m, a, tipo),
        _ => return Err(ErroAbertura::NaoImplementado("tipo da porta de Labyrinth")),
    }
    if rb(m, a.wrapping_add(OB_RENDER)) & 0x80 != 0 {
        let largura = u16::from(rb(m, a.wrapping_add(OB_ACTWID))).wrapping_add(11);
        let altura = u16::from(rb(m, a.wrapping_add(OB_HEIGHT)));
        solid_object(m, a, largura, altura, altura.wrapping_add(1))?;
    }
    let origem = rw(m, a.wrapping_add(ORIG_X));
    if fora_da_janela(m, origem) {
        delete_object(m, a);
    } else {
        display_sprite(m, a);
    }
    Ok(())
}

fn iniciar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let subtipo = rb(m, a.wrapping_add(OB_SUBTYPE));
    let forma = (subtipo >> 4) & 7;
    if rb(m, V_ZONE) != ID_LZ || subtipo & 0x80 == 0 || (forma != 6 && forma != 7) {
        return Err(ErroAbertura::NaoImplementado(
            "variante nao Labyrinth do bloco 56",
        ));
    }
    wb(m, a.wrapping_add(OB_ROUTINE), 2);
    wl(m, a.wrapping_add(OB_MAP), 0x10dd4);
    ww(m, a.wrapping_add(OB_GFX), 0x43c4);
    wb(m, a.wrapping_add(OB_RENDER), 4);
    wb(m, a.wrapping_add(OB_PRIORITY), 3);
    let (largura, altura) = if forma == 6 { (8, 32) } else { (64, 16) };
    wb(m, a.wrapping_add(OB_ACTWID), largura);
    wb(m, a.wrapping_add(OB_HEIGHT), altura);
    wb(m, a.wrapping_add(OB_FRAME), forma);
    let x = rw(m, a.wrapping_add(OB_X));
    let y = rw(m, a.wrapping_add(OB_Y));
    ww(m, a.wrapping_add(ORIG_X), x);
    ww(m, a.wrapping_add(ORIG_Y), y);
    wb(m, a.wrapping_add(CHAVE), subtipo & 15);
    wb(
        m,
        a.wrapping_add(OB_SUBTYPE),
        if forma == 6 { 5 } else { 12 },
    );
    ww(
        m,
        a.wrapping_add(DISTANCIA),
        if forma == 6 { 64 } else { 128 },
    );
    let numero = rb(m, a.wrapping_add(OB_RESPAWNNO));
    if numero != 0 {
        let endereco = V_OBJSTATE.wrapping_add(2).wrapping_add(u32::from(numero));
        let valor = rb(m, endereco) & 0x7f;
        wb(m, endereco, valor);
        if valor & 1 != 0 {
            wb(
                m,
                a.wrapping_add(OB_SUBTYPE),
                if forma == 6 { 6 } else { 13 },
            );
            ww(m, a.wrapping_add(DISTANCIA), 0);
        }
    }
    Ok(())
}

fn mover(m: &mut Memoria, a: u32, tipo: u8) {
    let abrindo = tipo == 5 || tipo == 12;
    let horizontal = tipo == 12 || tipo == 13;
    let chave = rb(m, a.wrapping_add(CHAVE));
    let acionada = rb(m, F_SWITCH.wrapping_add(u32::from(chave)));
    let mut movendo = rb(m, a.wrapping_add(MOVENDO)) != 0;
    if !movendo {
        if tipo == 5 && rw(m, V_ZONE_ACT) == 0x0100 && chave == 3 {
            let sonic_x = rw(m, V_PLAYER.wrapping_add(OB_X));
            let x = rw(m, a.wrapping_add(OB_X));
            wb(m, F_WTUNNELDISALLOW, u8::from(sonic_x < x));
        }
        if (abrindo && acionada & 1 != 0) || (!abrindo && acionada & 0x80 != 0) {
            if tipo == 5 && rw(m, V_ZONE_ACT) == 0x0100 && chave == 3 {
                wb(m, F_WTUNNELDISALLOW, 0);
            }
            movendo = true;
            wb(m, a.wrapping_add(MOVENDO), 1);
        }
    }
    let mut distancia = rw(m, a.wrapping_add(DISTANCIA));
    if movendo {
        let limite = if horizontal {
            128
        } else {
            u16::from(rb(m, a.wrapping_add(OB_HEIGHT))).wrapping_mul(2)
        };
        if (abrindo && distancia == 0) || (!abrindo && distancia == limite) {
            wb(
                m,
                a.wrapping_add(OB_SUBTYPE),
                if abrindo {
                    tipo.wrapping_add(1)
                } else {
                    tipo.wrapping_sub(1)
                },
            );
            wb(m, a.wrapping_add(MOVENDO), 0);
            let n = rb(m, a.wrapping_add(OB_RESPAWNNO));
            if n != 0 {
                let endereco = V_OBJSTATE.wrapping_add(2).wrapping_add(u32::from(n));
                let estado = rb(m, endereco);
                wb(m, endereco, if abrindo { estado | 1 } else { estado & !1 });
            }
        } else {
            distancia = if abrindo {
                distancia.wrapping_sub(2)
            } else {
                distancia.wrapping_add(2)
            };
            ww(m, a.wrapping_add(DISTANCIA), distancia);
        }
    }
    let invertida = rb(m, a.wrapping_add(OB_STATUS)) & 1 != 0;
    let desloc = if invertida {
        distancia.wrapping_neg()
    } else {
        distancia
    };
    if horizontal {
        let desloc = if invertida {
            desloc.wrapping_add(128)
        } else {
            desloc
        };
        let origem = rw(m, a.wrapping_add(ORIG_X));
        ww(m, a.wrapping_add(OB_X), origem.wrapping_add(desloc));
    } else {
        let origem = rw(m, a.wrapping_add(ORIG_Y));
        ww(m, a.wrapping_add(OB_Y), origem.wrapping_add(desloc));
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    fn porta(subtipo: u8) -> Memoria {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        wb(&mut m, V_ZONE, ID_LZ);
        wb(&mut m, V_LVLOBJSPACE.wrapping_add(OB_ID), 0x56);
        wb(&mut m, V_LVLOBJSPACE.wrapping_add(OB_SUBTYPE), subtipo);
        ww(&mut m, V_LVLOBJSPACE.wrapping_add(OB_X), 0x200);
        ww(&mut m, V_LVLOBJSPACE.wrapping_add(OB_Y), 0x300);
        m
    }
    #[test]
    fn porta_vertical_abre_por_chave_e_guarda_estado() {
        let mut m = porta(0xe0);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_RESPAWNNO), 2);
        executar(&mut m, a).unwrap();
        assert_eq!(rw(&mut m, a.wrapping_add(OB_Y)), 0x340);
        wb(&mut m, F_SWITCH, 1);
        for _ in 0..33 {
            executar(&mut m, a).unwrap();
        }
        assert_eq!(rw(&mut m, a.wrapping_add(OB_Y)), 0x300);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_SUBTYPE)), 6);
        assert_eq!(rb(&mut m, V_OBJSTATE.wrapping_add(4)) & 1, 1);
    }
    #[test]
    fn porta_horizontal_aberta_e_restaurada_do_respawn() {
        let mut m = porta(0xf1);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_RESPAWNNO), 2);
        wb(&mut m, V_OBJSTATE.wrapping_add(4), 0x81);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_SUBTYPE)), 13);
        assert_eq!(rw(&mut m, a.wrapping_add(OB_X)), 0x200);
        assert_eq!(rb(&mut m, V_OBJSTATE.wrapping_add(4)), 1);
    }
    #[test]
    fn terceira_porta_controla_tunel_de_vento() {
        let mut m = porta(0xe3);
        let a = V_LVLOBJSPACE;
        ww(&mut m, V_PLAYER.wrapping_add(OB_X), 0x100);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, F_WTUNNELDISALLOW), 1);
        wb(&mut m, F_SWITCH.wrapping_add(3), 1);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, F_WTUNNELDISALLOW), 0);
    }
}
