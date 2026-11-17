//! Caterkiller da Marble e Scrap Brain (objeto 78).
use super::boot::rom_b;
use super::fisica::{obj_floor_dist, object_fall};
use super::objetos::{delete_object, display_sprite, fora_da_janela};
use super::ram::*;
use super::{ErroAbertura, rb, rl, rw, wb, wl, ww};
use crate::memoria::Memoria;

const TEMPO: u32 = 0x2A;
const MODO: u32 = 0x2B;
const PISO: u32 = 0x2C;
const PAI: u32 = 0x3C;

pub fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => iniciar(m, a),
        2 => cabeca(m, a),
        4 | 6 | 8 => corpo(m, a),
        10 => delete_object(m, a),
        12 => fragmento(m, a),
        _ => return Err(ErroAbertura::NaoImplementado("rotina do Caterkiller")),
    }
    Ok(())
}

fn livre(m: &mut Memoria, a: u32) -> Option<u32> {
    let mut b = a.wrapping_add(OBJECT_SIZE);
    while b < V_OBJSPACE_END {
        if rb(m, b.wrapping_add(OB_ID)) == 0 {
            return Some(b);
        }
        b = b.wrapping_add(OBJECT_SIZE);
    }
    None
}

fn iniciar(m: &mut Memoria, a: u32) {
    wb(m, a.wrapping_add(OB_HEIGHT), 7);
    wb(m, a.wrapping_add(OB_WIDTH), 8);
    object_fall(m, a);
    let distancia = obj_floor_dist(m, a);
    if i16::from_be_bytes(distancia.to_be_bytes()) >= 0 {
        return;
    }
    let y = rw(m, a.wrapping_add(OB_Y)).wrapping_add(distancia);
    ww(m, a.wrapping_add(OB_Y), y);
    ww(m, a.wrapping_add(OB_VELY), 0);
    wb(m, a.wrapping_add(OB_ROUTINE), 2);
    wl(m, a.wrapping_add(OB_MAP), 0x1751A);
    let gfx = if rb(m, V_ZONE) == ID_SBZ {
        0x22B0
    } else {
        0x24FF
    };
    ww(m, a.wrapping_add(OB_GFX), gfx);
    let status = (rb(m, a.wrapping_add(OB_RENDER)) & 3) | 4;
    wb(m, a.wrapping_add(OB_RENDER), status);
    wb(m, a.wrapping_add(OB_STATUS), status);
    wb(m, a.wrapping_add(OB_PRIORITY), 4);
    wb(m, a.wrapping_add(OB_ACTWID), 8);
    wb(m, a.wrapping_add(OB_COLTYPE), 0x0B);
    let passo = if status & 1 == 0 {
        12u16
    } else {
        0u16.wrapping_sub(12)
    };
    let mut x = rw(m, a.wrapping_add(OB_X));
    let mut anterior = a;
    for i in 0..3u8 {
        let Some(b) = livre(m, anterior) else {
            despawn(m, a);
            return;
        };
        wb(m, b.wrapping_add(OB_ID), 0x78);
        wb(
            m,
            b.wrapping_add(OB_ROUTINE),
            4u8.wrapping_add(i.wrapping_mul(2)),
        );
        wl(m, b.wrapping_add(OB_MAP), 0x1751A);
        ww(m, b.wrapping_add(OB_GFX), gfx);
        wb(m, b.wrapping_add(OB_PRIORITY), 5);
        wb(m, b.wrapping_add(OB_ACTWID), 8);
        wb(m, b.wrapping_add(OB_COLTYPE), 0xCB);
        x = x.wrapping_add(passo);
        ww(m, b.wrapping_add(OB_X), x);
        ww(m, b.wrapping_add(OB_Y), y);
        wb(m, b.wrapping_add(OB_STATUS), status);
        wb(m, b.wrapping_add(OB_RENDER), status);
        wb(m, b.wrapping_add(OB_FRAME), 8);
        wl(m, b.wrapping_add(PAI), anterior);
        wb(m, b.wrapping_add(PAI), 4u8.wrapping_add(i.wrapping_mul(4)));
        anterior = b;
    }
    wb(m, a.wrapping_add(TEMPO), 7);
    wb(m, a.wrapping_add(PAI), 0);
    cabeca(m, a);
}

fn despawn(m: &mut Memoria, a: u32) {
    let n = rb(m, a.wrapping_add(OB_RESPAWNNO));
    if n != 0 {
        let p = V_OBJSTATE.wrapping_add(2).wrapping_add(u32::from(n));
        let v = rb(m, p) & 0x7F;
        wb(m, p, v);
    }
    wb(m, a.wrapping_add(OB_ROUTINE), 10);
}

fn cabeca(m: &mut Memoria, a: u32) {
    if rb(m, a.wrapping_add(OB_STATUS)) & 0x80 != 0 {
        fragmentar(m, a);
        return;
    }
    let secundario = rb(m, a.wrapping_add(OB_ROUTINE_2));
    if secundario == 0 {
        let tempo = rb(m, a.wrapping_add(TEMPO)).wrapping_sub(1);
        wb(m, a.wrapping_add(TEMPO), tempo);
        if tempo & 0x80 == 0 {
            mostrar_cabeca(m, a);
            return;
        }
        wb(m, a.wrapping_add(OB_ROUTINE_2), 2);
        wb(m, a.wrapping_add(TEMPO), 16);
        let antigo = rb(m, a.wrapping_add(MODO));
        let modo = (antigo ^ 0x10) | 0x80;
        wb(m, a.wrapping_add(MODO), modo);
        if antigo & 0x10 != 0 {
            ww(m, a.wrapping_add(OB_VELX), 0xFF40);
            ww(m, a.wrapping_add(OB_INERTIA), 0x0040);
        } else {
            ww(m, a.wrapping_add(OB_VELX), 0);
            ww(m, a.wrapping_add(OB_INERTIA), 0xFFC0);
        }
    }
    let tempo = rb(m, a.wrapping_add(TEMPO)).wrapping_sub(1);
    wb(m, a.wrapping_add(TEMPO), tempo);
    if tempo & 0x80 != 0 {
        wb(m, a.wrapping_add(OB_ROUTINE_2), 0);
        wb(m, a.wrapping_add(TEMPO), 7);
        ww(m, a.wrapping_add(OB_VELX), 0);
        ww(m, a.wrapping_add(OB_INERTIA), 0);
    } else if rw(m, a.wrapping_add(OB_VELX)) != 0 {
        let mudou = mover_x(m, a);
        if mudou {
            let distancia = obj_floor_dist(m, a);
            let d = i16::from_be_bytes(distancia.to_be_bytes());
            if !(-8..12).contains(&d) {
                virar_cabeca(m, a);
            } else {
                let y = rw(m, a.wrapping_add(OB_Y)).wrapping_add(distancia);
                ww(m, a.wrapping_add(OB_Y), y);
                registrar_piso(m, a, distancia.to_be_bytes()[1]);
            }
        }
    }
    mostrar_cabeca(m, a);
}

fn mostrar_cabeca(m: &mut Memoria, a: u32) {
    let modo = rb(m, a.wrapping_add(MODO));
    if modo & 0x80 != 0 {
        let angulo = rb(m, a.wrapping_add(OB_ANGLE));
        wb(m, a.wrapping_add(OB_ANGLE), angulo.wrapping_add(4));
        let frame = rom_b(m, 0x1749Au32.wrapping_add(u32::from(angulo & 0x7F)));
        if frame & 0x80 != 0 {
            wb(m, a.wrapping_add(MODO), modo & 0x7F);
        } else {
            wb(m, a.wrapping_add(OB_FRAME), frame.wrapping_add(modo & 0x10));
        }
    }
    let x = rw(m, a.wrapping_add(OB_X));
    if fora_da_janela(m, x) {
        despawn(m, a);
    } else {
        display_sprite(m, a);
    }
}

fn mover_x(m: &mut Memoria, a: u32) -> bool {
    let antes = rl(m, a.wrapping_add(OB_X));
    let mut vel = rw(m, a.wrapping_add(OB_VELX));
    if rb(m, a.wrapping_add(OB_STATUS)) & 1 != 0 {
        vel = vel.wrapping_neg();
    }
    let sinal = i32::from(i16::from_be_bytes(vel.to_be_bytes())).wrapping_shl(8);
    let depois = antes.wrapping_add(u32::from_be_bytes(sinal.to_be_bytes()));
    wl(m, a.wrapping_add(OB_X), depois);
    antes >> 16 != depois >> 16
}
fn registrar_piso(m: &mut Memoria, a: u32, d: u8) {
    let p = rb(m, a.wrapping_add(PAI));
    wb(m, a.wrapping_add(PISO).wrapping_add(u32::from(p)), d);
    wb(m, a.wrapping_add(PAI), p.wrapping_add(1) & 15);
}
fn virar_cabeca(m: &mut Memoria, a: u32) {
    let p = rb(m, a.wrapping_add(PAI));
    wb(m, a.wrapping_add(PISO).wrapping_add(u32::from(p)), 0x80);
    let sub = rw(m, a.wrapping_add(OB_X).wrapping_add(2)).wrapping_neg();
    ww(m, a.wrapping_add(OB_X).wrapping_add(2), sub);
    let mut p_prox = p;
    let status = rb(m, a.wrapping_add(OB_STATUS));
    if sub != 0 && (status & 1 != 0) {
        let x = rw(m, a.wrapping_add(OB_X)).wrapping_sub(1);
        ww(m, a.wrapping_add(OB_X), x);
        p_prox = p_prox.wrapping_add(1);
        wb(
            m,
            a.wrapping_add(PISO).wrapping_add(u32::from(p_prox & 15)),
            0,
        );
    }
    let novo_status = status ^ 1;
    wb(m, a.wrapping_add(OB_STATUS), novo_status);
    wb(m, a.wrapping_add(OB_RENDER), novo_status);
    wb(m, a.wrapping_add(PAI), p_prox.wrapping_add(1) & 15);
}

fn virar_corpo(m: &mut Memoria, a: u32, vel_x: u16) {
    let p = rb(m, a.wrapping_add(PAI));
    wb(m, a.wrapping_add(PISO).wrapping_add(u32::from(p)), 0x80);
    let sub = rw(m, a.wrapping_add(OB_X).wrapping_add(2)).wrapping_neg();
    ww(m, a.wrapping_add(OB_X).wrapping_add(2), sub);
    let mut p_prox = p;
    let status = rb(m, a.wrapping_add(OB_STATUS));
    if sub != 0 && (status & 1 != 0) && vel_x == 0xFF40 {
        let x = rw(m, a.wrapping_add(OB_X)).wrapping_sub(1);
        ww(m, a.wrapping_add(OB_X), x);
        p_prox = p_prox.wrapping_add(1);
        wb(
            m,
            a.wrapping_add(PISO).wrapping_add(u32::from(p_prox & 15)),
            0,
        );
    }
    let novo_status = status ^ 1;
    wb(m, a.wrapping_add(OB_STATUS), novo_status);
    wb(m, a.wrapping_add(OB_RENDER), novo_status);
    wb(m, a.wrapping_add(PAI), p_prox.wrapping_add(1) & 15);
}

fn corpo(m: &mut Memoria, a: u32) {
    let pai = rl(m, a.wrapping_add(PAI)) & 0x00FF_FFFF;
    if rb(m, a.wrapping_add(OB_STATUS)) & 0x80 != 0 || rb(m, pai.wrapping_add(OB_ROUTINE)) == 12 {
        let status = rb(m, pai.wrapping_add(OB_STATUS)) | 0x80;
        wb(m, pai.wrapping_add(OB_STATUS), status);
        fragmentar(m, a);
        return;
    }
    let modo = rb(m, pai.wrapping_add(MODO));
    wb(m, a.wrapping_add(MODO), modo);
    if rb(m, a.wrapping_add(OB_ROUTINE)) == 6 && modo & 0x80 != 0 {
        let angulo = rb(m, a.wrapping_add(OB_ANGLE));
        let indice = u32::from(angulo & 0x7F);
        let incremento = if rom_b(m, 0x1749Eu32.wrapping_add(indice)) & 0x80 != 0 {
            8
        } else {
            4
        };
        wb(m, a.wrapping_add(OB_ANGLE), angulo.wrapping_add(incremento));
        wb(
            m,
            a.wrapping_add(OB_FRAME),
            rom_b(m, 0x1749Au32.wrapping_add(indice)).wrapping_add(8),
        );
    }
    let rotina = rb(m, pai.wrapping_add(OB_ROUTINE_2));
    wb(m, a.wrapping_add(OB_ROUTINE_2), rotina);
    if rotina != 0 {
        let inercia = rw(m, pai.wrapping_add(OB_INERTIA));
        let velocidade = rw(m, pai.wrapping_add(OB_VELX)).wrapping_add(inercia);
        ww(m, a.wrapping_add(OB_INERTIA), inercia);
        ww(m, a.wrapping_add(OB_VELX), velocidade);
        if mover_x(m, a) {
            let pos = rb(m, a.wrapping_add(PAI));
            let piso = rb(m, pai.wrapping_add(PISO).wrapping_add(u32::from(pos)));
            if piso == 0x80 {
                virar_corpo(m, a, velocidade);
            } else {
                let delta = u16::from_be_bytes(i16::from(i8::from_be_bytes([piso])).to_be_bytes());
                let y = rw(m, a.wrapping_add(OB_Y)).wrapping_add(delta);
                ww(m, a.wrapping_add(OB_Y), y);
                registrar_piso(m, a, piso);
            }
        }
    }
    if rb(m, pai.wrapping_add(OB_ID)) == ID_EXPLOSION_ITEM
        || rb(m, pai.wrapping_add(OB_ROUTINE)) == 10
    {
        wb(m, a.wrapping_add(OB_ROUTINE), 10);
    }
    display_sprite(m, a);
}

fn fragmentar(m: &mut Memoria, a: u32) {
    let rotina = rb(m, a.wrapping_add(OB_ROUTINE));
    let velocidade = match rotina {
        2 => 0xFE00u16,
        4 => 0xFE80,
        6 => 0x0180,
        _ => 0x0200,
    };
    let velocidade = if rb(m, a.wrapping_add(OB_STATUS)) & 1 != 0 {
        velocidade.wrapping_neg()
    } else {
        velocidade
    };
    ww(m, a.wrapping_add(OB_VELX), velocidade);
    ww(m, a.wrapping_add(OB_VELY), 0xFC00);
    wb(m, a.wrapping_add(OB_ROUTINE), 12);
    let frame = rb(m, a.wrapping_add(OB_FRAME)) & 0xF8;
    wb(m, a.wrapping_add(OB_FRAME), frame);
    fragmento(m, a);
}
fn fragmento(m: &mut Memoria, a: u32) {
    object_fall(m, a);
    if i16::from_be_bytes(rw(m, a.wrapping_add(OB_VELY)).to_be_bytes()) >= 0 {
        let d = obj_floor_dist(m, a);
        if i16::from_be_bytes(d.to_be_bytes()) < 0 {
            let y = rw(m, a.wrapping_add(OB_Y)).wrapping_add(d);
            ww(m, a.wrapping_add(OB_Y), y);
            ww(m, a.wrapping_add(OB_VELY), 0xFC00);
        }
    }
    if rb(m, a.wrapping_add(OB_RENDER)) & 0x80 == 0 {
        despawn(m, a);
    } else {
        display_sprite(m, a);
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn segmentos_seguem_estado_e_morte_do_pai() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let pai = V_LVLOBJSPACE;
        let filho = pai.wrapping_add(OBJECT_SIZE);
        wb(&mut m, pai.wrapping_add(OB_ID), 0x78);
        wb(&mut m, pai.wrapping_add(OB_ROUTINE), 2);
        wb(&mut m, filho.wrapping_add(OB_ID), 0x78);
        wb(&mut m, filho.wrapping_add(OB_ROUTINE), 4);
        wl(&mut m, filho.wrapping_add(PAI), pai);
        wb(&mut m, filho.wrapping_add(PAI), 4);
        wb(&mut m, pai.wrapping_add(OB_ROUTINE), 10);
        executar(&mut m, filho).unwrap();
        assert_eq!(rb(&mut m, filho.wrapping_add(OB_ROUTINE)), 10);
        executar(&mut m, filho).unwrap();
        assert_eq!(rb(&mut m, filho.wrapping_add(OB_ID)), 0);
    }
}
