//! Blocos flutuantes de SYZ/SLZ e portas de LZ, objeto 0x56 (`FloatingBlock`, REV01).
use super::super::ram::*;
use super::super::{ErroAbertura, rb, rw, wb, wl, ww};
use super::{delete_object, display_sprite, fora_da_janela, solid_object};
use crate::memoria::Memoria;

const MAP_FBLOCK: u32 = 0x10DD4;
const ORIG_Y: u32 = 0x30;
const ORIG_X: u32 = 0x34;
const MOVENDO: u32 = 0x38;
const DISTANCIA: u32 = 0x3A;
const CHAVE: u32 = 0x3C;
const F_SWITCH: u32 = 0xFF_F7E0;
const F_WTUNNELDISALLOW: u32 = 0xFF_F7C9;

const FBLOCK_VAR: [(u8, u8); 8] = [
    (16, 16), // 0: 32x32 (SYZ 1x1 block)
    (32, 32), // 1: 64x64 (SYZ 2x2 square)
    (16, 32), // 2: 32x64 (SYZ 1x2 door)
    (32, 26), // 3: 64x52 (SYZ special block moving right in SYZ3)
    (16, 39), // 4: 32x78 (unused)
    (16, 16), // 5: 32x32 (SLZ rotating stairway block)
    (8, 32),  // 6: 16x64 (LZ small vertical door)
    (64, 16), // 7: 128x32 (LZ large sideways 4x1 block)
];

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            if iniciar(m, a)? {
                return Ok(());
            }
        }
        2 => {}
        _ => return Err(ErroAbertura::NaoImplementado("rotina do bloco flutuante 56")),
    }

    let anterior_x = rw(m, a.wrapping_add(OB_X));
    let tipo = rb(m, a.wrapping_add(OB_SUBTYPE)) & 0x0F;
    match tipo {
        0 => {}
        1 => {
            let osc = u16::from(rb(m, V_OSCILLATE.wrapping_add(0x0A)));
            let mut d0 = osc;
            let d1 = 0x40u16;
            if rb(m, a.wrapping_add(OB_STATUS)) & 1 != 0 {
                d0 = d1.wrapping_sub(d0);
            }
            let ox = rw(m, a.wrapping_add(ORIG_X));
            ww(m, a.wrapping_add(OB_X), ox.wrapping_sub(d0));
        }
        2 => {
            let osc = u16::from(rb(m, V_OSCILLATE.wrapping_add(0x1E)));
            let mut d0 = osc;
            let d1 = 0x80u16;
            if rb(m, a.wrapping_add(OB_STATUS)) & 1 != 0 {
                d0 = d1.wrapping_sub(d0);
            }
            let ox = rw(m, a.wrapping_add(ORIG_X));
            ww(m, a.wrapping_add(OB_X), ox.wrapping_sub(d0));
        }
        3 => {
            let osc = u16::from(rb(m, V_OSCILLATE.wrapping_add(0x0A)));
            let mut d0 = osc;
            let d1 = 0x40u16;
            if rb(m, a.wrapping_add(OB_STATUS)) & 1 != 0 {
                d0 = d1.wrapping_sub(d0);
            }
            let oy = rw(m, a.wrapping_add(ORIG_Y));
            ww(m, a.wrapping_add(OB_Y), oy.wrapping_sub(d0));
        }
        4 => {
            let osc = u16::from(rb(m, V_OSCILLATE.wrapping_add(0x1E)));
            let mut d0 = osc;
            let d1 = 0x80u16;
            if rb(m, a.wrapping_add(OB_STATUS)) & 1 != 0 {
                d0 = d1.wrapping_sub(d0);
            }
            let oy = rw(m, a.wrapping_add(ORIG_Y));
            ww(m, a.wrapping_add(OB_Y), oy.wrapping_sub(d0));
        }
        5 => mover_lz_porta_pequena_abrir(m, a),
        6 => mover_lz_porta_pequena_fechar(m, a),
        7 => mover_syz3_horizontal(m, a),
        8..=11 => mover_slz_stair(m, a, tipo),
        12 => mover_lz_porta_grande_abrir(m, a),
        13 => mover_lz_porta_grande_fechar(m, a),
        _ => return Err(ErroAbertura::NaoImplementado("tipo do bloco flutuante 56")),
    }

    if rb(m, a.wrapping_add(OB_RENDER)) & 0x80 != 0 {
        let largura = u16::from(rb(m, a.wrapping_add(OB_ACTWID))).wrapping_add(11);
        let altura = u16::from(rb(m, a.wrapping_add(OB_HEIGHT)));
        solid_object(m, a, largura, altura, altura.wrapping_add(1))?;
        if rb(m, a.wrapping_add(OB_STATUS)) & 8 != 0 {
            let dx = rw(m, a.wrapping_add(OB_X)).wrapping_sub(anterior_x);
            if dx != 0 {
                let sx = rw(m, V_PLAYER.wrapping_add(OB_X)).wrapping_add(dx);
                ww(m, V_PLAYER.wrapping_add(OB_X), sx);
            }
        }
    }

    let orig_x = rw(m, a.wrapping_add(ORIG_X));
    if fora_da_janela(m, orig_x) {
        if rb(m, a.wrapping_add(OB_SUBTYPE)) == 0x37 && rb(m, a.wrapping_add(MOVENDO)) != 0 {
            display_sprite(m, a);
        } else {
            delete_object(m, a);
        }
    } else {
        display_sprite(m, a);
    }
    Ok(())
}

fn iniciar(m: &mut Memoria, a: u32) -> Result<bool, ErroAbertura> {
    wb(m, a.wrapping_add(OB_ROUTINE), 2);
    wl(m, a.wrapping_add(OB_MAP), MAP_FBLOCK);
    let gfx = if rb(m, V_ZONE) == ID_LZ {
        0x43C4
    } else {
        0x4000
    };
    ww(m, a.wrapping_add(OB_GFX), gfx);
    wb(m, a.wrapping_add(OB_RENDER), 4);
    wb(m, a.wrapping_add(OB_PRIORITY), 3);

    let subtype = rb(m, a.wrapping_add(OB_SUBTYPE));
    let forma = ((subtype >> 3) & 0x0E) >> 1;
    let (largura, altura) = FBLOCK_VAR[forma as usize];
    wb(m, a.wrapping_add(OB_ACTWID), largura);
    wb(m, a.wrapping_add(OB_HEIGHT), altura);
    wb(m, a.wrapping_add(OB_FRAME), forma);

    let x = rw(m, a.wrapping_add(OB_X));
    let y = rw(m, a.wrapping_add(OB_Y));
    ww(m, a.wrapping_add(ORIG_X), x);
    ww(m, a.wrapping_add(ORIG_Y), y);
    ww(m, a.wrapping_add(DISTANCIA), u16::from(altura).wrapping_mul(2));

    // Hardcoded special treatment for horizontally moving block in SYZ3 (subtype 0x37, REV01)
    if subtype == 0x37 {
        if x == 0x1BB8 {
            if rb(m, F_OBJ56) != 0 {
                delete_object(m, a);
                return Ok(true);
            }
        } else {
            wb(m, a.wrapping_add(OB_SUBTYPE), 0);
            if rb(m, F_OBJ56) == 0 {
                delete_object(m, a);
                return Ok(true);
            }
        }
    }

    if rb(m, V_ZONE) != ID_LZ {
        let sub_low = subtype & 0x0F;
        if sub_low >= 8 {
            let osc_rate_offset = 0x2A + 2 + u32::from(sub_low.wrapping_sub(8)).wrapping_mul(4);
            let rate = rw(m, V_OSCILLATE.wrapping_add(osc_rate_offset)) as i16;
            if rate < 0 {
                let st = rb(m, a.wrapping_add(OB_STATUS)) ^ 1;
                wb(m, a.wrapping_add(OB_STATUS), st);
            }
        }
    }

    if subtype & 0x80 != 0 {
        wb(m, a.wrapping_add(CHAVE), subtype & 0x0F);
        if forma == 7 {
            wb(m, a.wrapping_add(OB_SUBTYPE), 12);
            ww(m, a.wrapping_add(DISTANCIA), 128);
        } else {
            wb(m, a.wrapping_add(OB_SUBTYPE), 5);
        }
        let respawn = rb(m, a.wrapping_add(OB_RESPAWNNO));
        if respawn != 0 {
            let addr = V_OBJSTATE.wrapping_add(2).wrapping_add(u32::from(respawn));
            let val = rb(m, addr) & 0x7F;
            wb(m, addr, val);
            if val & 1 != 0 {
                let cur_sub = rb(m, a.wrapping_add(OB_SUBTYPE));
                wb(m, a.wrapping_add(OB_SUBTYPE), cur_sub.wrapping_add(1));
                ww(m, a.wrapping_add(DISTANCIA), 0);
            }
        }
    }
    Ok(false)
}

fn mover_lz_porta_pequena_abrir(m: &mut Memoria, a: u32) {
    let mut movendo = rb(m, a.wrapping_add(MOVENDO)) != 0;
    let chave = rb(m, a.wrapping_add(CHAVE));
    if !movendo {
        if rw(m, V_ZONE_ACT) == 0x0100 && chave == 3 {
            let sonic_x = rw(m, V_PLAYER.wrapping_add(OB_X));
            let x = rw(m, a.wrapping_add(OB_X));
            wb(m, F_WTUNNELDISALLOW, u8::from(sonic_x < x));
        }
        let acionada = rb(m, F_SWITCH.wrapping_add(u32::from(chave)));
        if acionada & 1 != 0 {
            if rw(m, V_ZONE_ACT) == 0x0100 && chave == 3 {
                wb(m, F_WTUNNELDISALLOW, 0);
            }
            movendo = true;
            wb(m, a.wrapping_add(MOVENDO), 1);
        }
    }
    let mut dist = rw(m, a.wrapping_add(DISTANCIA));
    if movendo {
        if dist == 0 {
            wb(m, a.wrapping_add(OB_SUBTYPE), 6);
            wb(m, a.wrapping_add(MOVENDO), 0);
            let respawn = rb(m, a.wrapping_add(OB_RESPAWNNO));
            if respawn != 0 {
                let addr = V_OBJSTATE.wrapping_add(2).wrapping_add(u32::from(respawn));
                let val = rb(m, addr) | 1;
                wb(m, addr, val);
            }
        } else {
            dist = dist.wrapping_sub(2);
            ww(m, a.wrapping_add(DISTANCIA), dist);
        }
    }
    let mut d0 = dist;
    if rb(m, a.wrapping_add(OB_STATUS)) & 1 != 0 {
        d0 = d0.wrapping_neg();
    }
    let oy = rw(m, a.wrapping_add(ORIG_Y));
    ww(m, a.wrapping_add(OB_Y), oy.wrapping_add(d0));
}

fn mover_lz_porta_pequena_fechar(m: &mut Memoria, a: u32) {
    let mut movendo = rb(m, a.wrapping_add(MOVENDO)) != 0;
    let chave = rb(m, a.wrapping_add(CHAVE));
    if !movendo {
        let acionada = rb(m, F_SWITCH.wrapping_add(u32::from(chave)));
        if (acionada as i8) < 0 {
            movendo = true;
            wb(m, a.wrapping_add(MOVENDO), 1);
        }
    }
    let max_dist = u16::from(rb(m, a.wrapping_add(OB_HEIGHT))).wrapping_mul(2);
    let mut dist = rw(m, a.wrapping_add(DISTANCIA));
    if movendo {
        if dist >= max_dist {
            wb(m, a.wrapping_add(OB_SUBTYPE), 5);
            wb(m, a.wrapping_add(MOVENDO), 0);
            let respawn = rb(m, a.wrapping_add(OB_RESPAWNNO));
            if respawn != 0 {
                let addr = V_OBJSTATE.wrapping_add(2).wrapping_add(u32::from(respawn));
                let val = rb(m, addr) & !1;
                wb(m, addr, val);
            }
        } else {
            dist = dist.wrapping_add(2);
            ww(m, a.wrapping_add(DISTANCIA), dist);
        }
    }
    let mut d0 = dist;
    if rb(m, a.wrapping_add(OB_STATUS)) & 1 != 0 {
        d0 = d0.wrapping_neg();
    }
    let oy = rw(m, a.wrapping_add(ORIG_Y));
    ww(m, a.wrapping_add(OB_Y), oy.wrapping_add(d0));
}

fn mover_lz_porta_grande_abrir(m: &mut Memoria, a: u32) {
    let mut movendo = rb(m, a.wrapping_add(MOVENDO)) != 0;
    let chave = rb(m, a.wrapping_add(CHAVE));
    if !movendo {
        let acionada = rb(m, F_SWITCH.wrapping_add(u32::from(chave)));
        if acionada & 1 != 0 {
            movendo = true;
            wb(m, a.wrapping_add(MOVENDO), 1);
        }
    }
    let mut dist = rw(m, a.wrapping_add(DISTANCIA));
    if movendo {
        if dist == 0 {
            wb(m, a.wrapping_add(OB_SUBTYPE), 13);
            wb(m, a.wrapping_add(MOVENDO), 0);
            let respawn = rb(m, a.wrapping_add(OB_RESPAWNNO));
            if respawn != 0 {
                let addr = V_OBJSTATE.wrapping_add(2).wrapping_add(u32::from(respawn));
                let val = rb(m, addr) | 1;
                wb(m, addr, val);
            }
        } else {
            dist = dist.wrapping_sub(2);
            ww(m, a.wrapping_add(DISTANCIA), dist);
        }
    }
    let mut d0 = dist;
    if rb(m, a.wrapping_add(OB_STATUS)) & 1 != 0 {
        d0 = d0.wrapping_neg().wrapping_add(128);
    }
    let ox = rw(m, a.wrapping_add(ORIG_X));
    ww(m, a.wrapping_add(OB_X), ox.wrapping_add(d0));
}

fn mover_lz_porta_grande_fechar(m: &mut Memoria, a: u32) {
    let mut movendo = rb(m, a.wrapping_add(MOVENDO)) != 0;
    let chave = rb(m, a.wrapping_add(CHAVE));
    if !movendo {
        let acionada = rb(m, F_SWITCH.wrapping_add(u32::from(chave)));
        if (acionada as i8) < 0 {
            movendo = true;
            wb(m, a.wrapping_add(MOVENDO), 1);
        }
    }
    let mut dist = rw(m, a.wrapping_add(DISTANCIA));
    if movendo {
        if dist >= 128 {
            wb(m, a.wrapping_add(OB_SUBTYPE), 12);
            wb(m, a.wrapping_add(MOVENDO), 0);
            let respawn = rb(m, a.wrapping_add(OB_RESPAWNNO));
            if respawn != 0 {
                let addr = V_OBJSTATE.wrapping_add(2).wrapping_add(u32::from(respawn));
                let val = rb(m, addr) & !1;
                wb(m, addr, val);
            }
        } else {
            dist = dist.wrapping_add(2);
            ww(m, a.wrapping_add(DISTANCIA), dist);
        }
    }
    let mut d0 = dist;
    if rb(m, a.wrapping_add(OB_STATUS)) & 1 != 0 {
        d0 = d0.wrapping_neg().wrapping_add(128);
    }
    let ox = rw(m, a.wrapping_add(ORIG_X));
    ww(m, a.wrapping_add(OB_X), ox.wrapping_add(d0));
}

fn mover_syz3_horizontal(m: &mut Memoria, a: u32) {
    let mut movendo = rb(m, a.wrapping_add(MOVENDO)) != 0;
    if !movendo {
        if rb(m, F_SWITCH.wrapping_add(0x0F)) != 0 {
            movendo = true;
            wb(m, a.wrapping_add(MOVENDO), 1);
            ww(m, a.wrapping_add(DISTANCIA), 0);
        }
    }
    if movendo {
        let x = rw(m, a.wrapping_add(OB_X)).wrapping_add(1);
        ww(m, a.wrapping_add(OB_X), x);
        ww(m, a.wrapping_add(ORIG_X), x);
        let dist = rw(m, a.wrapping_add(DISTANCIA)).wrapping_add(1);
        ww(m, a.wrapping_add(DISTANCIA), dist);
        if dist >= 0x380 {
            wb(m, F_OBJ56, 1);
            wb(m, a.wrapping_add(MOVENDO), 0);
            wb(m, a.wrapping_add(OB_SUBTYPE), 0);
        }
    }
}

fn mover_slz_stair(m: &mut Memoria, a: u32, tipo: u8) {
    let (d1, osc_off, rate_off, div2) = match tipo {
        8 => (0x10u16, 0x2A, 0x2C, true),
        9 => (0x30u16, 0x2E, 0x30, false),
        10 => (0x50u16, 0x32, 0x34, false),
        11 => (0x70u16, 0x36, 0x38, false),
        _ => return,
    };
    let mut d0 = u16::from(rb(m, V_OSCILLATE.wrapping_add(osc_off)));
    if div2 {
        d0 >>= 1;
    }
    let d3 = rw(m, V_OSCILLATE.wrapping_add(rate_off)) as i16;
    if d3 == 0 {
        let st = rb(m, a.wrapping_add(OB_STATUS)).wrapping_add(1) & 3;
        wb(m, a.wrapping_add(OB_STATUS), st);
    }
    let d2 = rb(m, a.wrapping_add(OB_STATUS)) & 3;
    let ox = rw(m, a.wrapping_add(ORIG_X));
    let oy = rw(m, a.wrapping_add(ORIG_Y));
    match d2 {
        0 => {
            let x = ox.wrapping_add(d0.wrapping_sub(d1));
            let y = oy.wrapping_sub(d1);
            ww(m, a.wrapping_add(OB_X), x);
            ww(m, a.wrapping_add(OB_Y), y);
        }
        1 => {
            let d1_minus_1 = d1.wrapping_sub(1);
            let y = oy.wrapping_sub(d0.wrapping_sub(d1_minus_1));
            let x = ox.wrapping_add(d1);
            ww(m, a.wrapping_add(OB_X), x);
            ww(m, a.wrapping_add(OB_Y), y);
        }
        2 => {
            let d1_minus_1 = d1.wrapping_sub(1);
            let x = ox.wrapping_sub(d0.wrapping_sub(d1_minus_1));
            let y = oy.wrapping_add(d1);
            ww(m, a.wrapping_add(OB_X), x);
            ww(m, a.wrapping_add(OB_Y), y);
        }
        3 => {
            let y = oy.wrapping_add(d0.wrapping_sub(d1));
            let x = ox.wrapping_sub(d1);
            ww(m, a.wrapping_add(OB_X), x);
            ww(m, a.wrapping_add(OB_Y), y);
        }
        _ => {}
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
