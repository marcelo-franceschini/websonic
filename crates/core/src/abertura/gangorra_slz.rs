//! Gangorras e bolas de espinhos da Star Light, objeto 0x5E (REV01).
use super::super::boot::{rom_b, rom_w};
use super::super::ram::*;
use super::super::{ErroAbertura, fisica, rb, rl, rw, wb, wl, ww};
use super::{delete_object, display_sprite, fora_da_janela};
use crate::memoria::Memoria;

const ORIG_X: u32 = 0x30;
const ORIG_Y: u32 = 0x34;
const LADO: u32 = 0x3A;
const PAI: u32 = 0x3C;

fn dados_inclinacao(m: &mut Memoria, a: u32) -> u32 {
    if rb(m, a.wrapping_add(OB_FRAME)) & 1 != 0 {
        0x12048
    } else {
        0x12018
    }
}

fn exit_platform(m: &mut Memoria, a0: u32, d1: u16) {
    let d2 = d1.wrapping_mul(2);
    let player_status = rb(m, V_PLAYER.wrapping_add(OB_STATUS));
    if player_status & STATUS_IN_AIR != 0 {
        let ps = player_status & !STATUS_ON_OBJECT;
        wb(m, V_PLAYER.wrapping_add(OB_STATUS), ps);
        wb(m, a0.wrapping_add(OB_ROUTINE), 2);
        let os = rb(m, a0.wrapping_add(OB_STATUS)) & !STATUS_ON_OBJECT;
        wb(m, a0.wrapping_add(OB_STATUS), os);
        return;
    }

    let px = rw(m, V_PLAYER.wrapping_add(OB_X));
    let ox = rw(m, a0.wrapping_add(OB_X));
    let d0 = px.wrapping_sub(ox).wrapping_add(d1);
    if i16::from_be_bytes(d0.to_be_bytes()) < 0 || d0 >= d2 {
        let ps = player_status & !STATUS_ON_OBJECT;
        wb(m, V_PLAYER.wrapping_add(OB_STATUS), ps);
        wb(m, a0.wrapping_add(OB_ROUTINE), 2);
        let os = rb(m, a0.wrapping_add(OB_STATUS)) & !STATUS_ON_OBJECT;
        wb(m, a0.wrapping_add(OB_STATUS), os);
    }
}

fn slope_object_assume_stood_on(m: &mut Memoria, a0: u32, d1: u16) {
    if rb(m, V_PLAYER.wrapping_add(OB_STATUS)) & STATUS_ON_OBJECT == 0 {
        return;
    }
    let px = rw(m, V_PLAYER.wrapping_add(OB_X));
    let ox = rw(m, a0.wrapping_add(OB_X));
    let mut d0 = px.wrapping_sub(ox).wrapping_add(d1);
    d0 >>= 1;
    let render = rb(m, a0.wrapping_add(OB_RENDER));
    if render & 1 != 0 {
        d0 = (!d0).wrapping_add(d1);
    }
    let idx = usize::from(d0 & 0x7FFF).min(47);
    let tabela = dados_inclinacao(m, a0);
    let slope_val = u16::from(rom_b(
        m,
        tabela.wrapping_add(u32::try_from(idx).unwrap_or(0)),
    ));
    let oy = rw(m, a0.wrapping_add(OB_Y));
    let ph = u16::from(rb(m, V_PLAYER.wrapping_add(OB_HEIGHT)));
    let new_py = oy.wrapping_sub(slope_val).wrapping_sub(ph);
    ww(m, V_PLAYER.wrapping_add(OB_Y), new_py);
}

fn slope_object(m: &mut Memoria, a0: u32, d1: u16) -> Result<(), ErroAbertura> {
    let p_vy = i16::from_be_bytes(rw(m, V_PLAYER.wrapping_add(OB_VELY)).to_be_bytes());
    if p_vy < 0 {
        return Ok(());
    }

    let px = rw(m, V_PLAYER.wrapping_add(OB_X));
    let ox = rw(m, a0.wrapping_add(OB_X));
    let mut d0 = px.wrapping_sub(ox).wrapping_add(d1);
    if i16::from_be_bytes(d0.to_be_bytes()) < 0 {
        return Ok(());
    }
    let d1_double = d1.wrapping_mul(2);
    if d0 >= d1_double {
        return Ok(());
    }

    let render = rb(m, a0.wrapping_add(OB_RENDER));
    if render & 1 != 0 {
        d0 = (!d0).wrapping_add(d1_double);
    }
    d0 >>= 1;
    let idx = usize::from(d0 & 0x7FFF).min(47);
    let tabela = dados_inclinacao(m, a0);
    let slope_val = rom_b(m, tabela.wrapping_add(u32::try_from(idx).unwrap_or(0)));

    let oy = rw(m, a0.wrapping_add(OB_Y));
    let surface_y = oy.wrapping_sub(u16::from(slope_val));

    let py = rw(m, V_PLAYER.wrapping_add(OB_Y));
    let ph = u16::from(rb(m, V_PLAYER.wrapping_add(OB_HEIGHT)));
    let p_bottom = py.wrapping_add(ph).wrapping_add(4);
    let dist = i16::from_be_bytes(surface_y.wrapping_sub(p_bottom).to_be_bytes());
    if !(-16..=-1).contains(&dist) {
        return Ok(());
    }

    if i8::from_be_bytes([rb(m, F_PLAYERCTRL)]) < 0 {
        return Ok(());
    }
    if rb(m, V_PLAYER.wrapping_add(OB_ROUTINE)) >= 6 {
        return Ok(());
    }

    let dist_u16 = u16::from_be_bytes(dist.to_be_bytes());
    let new_py = py.wrapping_add(dist_u16).wrapping_add(3);
    ww(m, V_PLAYER.wrapping_add(OB_Y), new_py);
    let rout = rb(m, a0.wrapping_add(OB_ROUTINE)).wrapping_add(2);
    wb(m, a0.wrapping_add(OB_ROUTINE), rout);

    let player_status = rb(m, V_PLAYER.wrapping_add(OB_STATUS));
    if player_status & STATUS_ON_OBJECT != 0 {
        let prev_idx = u32::from(rb(m, V_PLAYER.wrapping_add(SON_STANDONOBJECT)));
        let prev_obj = V_OBJSPACE.wrapping_add(prev_idx.wrapping_mul(OBJECT_SIZE));
        let ps = rb(m, prev_obj.wrapping_add(OB_STATUS)) & !STATUS_ON_OBJECT;
        wb(m, prev_obj.wrapping_add(OB_STATUS), ps);
        wb(m, prev_obj.wrapping_add(OB_ROUTINE_2), 0);
        if rb(m, prev_obj.wrapping_add(OB_ROUTINE)) == 4 {
            wb(m, prev_obj.wrapping_add(OB_ROUTINE), 2);
        }
    }

    let current_idx = u8::try_from((a0.wrapping_sub(V_OBJSPACE) / OBJECT_SIZE) & 0x7F).unwrap_or(0);
    wb(m, V_PLAYER.wrapping_add(SON_STANDONOBJECT), current_idx);
    wb(m, V_PLAYER.wrapping_add(OB_ANGLE), 0);
    ww(m, V_PLAYER.wrapping_add(OB_VELY), 0);
    let vx = rw(m, V_PLAYER.wrapping_add(OB_VELX));
    ww(m, V_PLAYER.wrapping_add(OB_INERTIA), vx);

    if player_status & STATUS_IN_AIR != 0 {
        super::super::fisica::sonic_reset_on_floor(m, V_PLAYER);
    }

    let ps = rb(m, V_PLAYER.wrapping_add(OB_STATUS)) | STATUS_ON_OBJECT;
    wb(m, V_PLAYER.wrapping_add(OB_STATUS), ps);
    let os = rb(m, a0.wrapping_add(OB_STATUS)) | STATUS_ON_OBJECT;
    wb(m, a0.wrapping_add(OB_STATUS), os);

    Ok(())
}

fn proximo_livre(m: &mut Memoria, a: u32) -> Option<u32> {
    let mut p = a.wrapping_add(OBJECT_SIZE);
    while p < V_LVLOBJEND {
        if rb(m, p.wrapping_add(OB_ID)) == 0 {
            return Some(p);
        }
        p = p.wrapping_add(OBJECT_SIZE);
    }
    None
}
fn mudar_quadro(m: &mut Memoria, a: u32, estado: u8) {
    let atual = rb(m, a.wrapping_add(OB_FRAME));
    if atual == estado {
        return;
    }
    let novo = if atual < estado {
        atual.wrapping_add(1)
    } else {
        atual.wrapping_sub(1)
    };
    wb(m, a.wrapping_add(OB_FRAME), novo);
    wb(m, a.wrapping_add(LADO), estado);
    let render = rb(m, a.wrapping_add(OB_RENDER));
    wb(
        m,
        a.wrapping_add(OB_RENDER),
        (render & !1) | u8::from(novo & 2 != 0),
    );
}
fn iniciar(m: &mut Memoria, a: u32) {
    wb(m, a.wrapping_add(OB_ROUTINE), 2);
    wl(m, a.wrapping_add(OB_MAP), 0x12078);
    ww(m, a.wrapping_add(OB_GFX), 0x374);
    let render = rb(m, a.wrapping_add(OB_RENDER));
    wb(m, a.wrapping_add(OB_RENDER), render | 4);
    wb(m, a.wrapping_add(OB_PRIORITY), 4);
    wb(m, a.wrapping_add(OB_ACTWID), 48);
    let x = rw(m, a.wrapping_add(OB_X));
    let y = rw(m, a.wrapping_add(OB_Y));
    ww(m, a.wrapping_add(ORIG_X), x);
    let status = rb(m, a.wrapping_add(OB_STATUS));
    if rb(m, a.wrapping_add(OB_SUBTYPE)) == 0
        && let Some(b) = proximo_livre(m, a)
    {
        wb(m, b.wrapping_add(OB_ID), 0x5E);
        wb(m, b.wrapping_add(OB_ROUTINE), 6);
        ww(m, b.wrapping_add(OB_X), x);
        ww(m, b.wrapping_add(OB_Y), y);
        wb(m, b.wrapping_add(OB_STATUS), status);
        wl(m, b.wrapping_add(PAI), a);
    }
    if status & 1 != 0 {
        wb(m, a.wrapping_add(OB_FRAME), 2);
    }
    let frame = rb(m, a.wrapping_add(OB_FRAME));
    wb(m, a.wrapping_add(LADO), frame);
}
fn alinhar_altura(m: &mut Memoria, a: u32, pai: u32) -> u16 {
    let mut frame = u32::from(rb(m, pai.wrapping_add(OB_FRAME)));
    if rw(m, a.wrapping_add(OB_X)) < rw(m, a.wrapping_add(ORIG_X)) {
        frame = frame.wrapping_add(2);
    }
    let deslocamento = rom_w(m, 0x1200Eu32.wrapping_add(frame.wrapping_mul(2)));
    rw(m, a.wrapping_add(ORIG_Y)).wrapping_add(deslocamento)
}
fn lancar_sonic(m: &mut Memoria, a: u32, pai: u32, lado: u8) {
    wb(m, pai.wrapping_add(LADO), lado);
    wb(m, a.wrapping_add(LADO), lado);
    let estado = rb(m, pai.wrapping_add(OB_STATUS));
    if rb(m, pai.wrapping_add(OB_FRAME)) != lado && estado & STATUS_ON_OBJECT != 0 {
        wb(m, pai.wrapping_add(OB_STATUS), estado & !STATUS_ON_OBJECT);
        wb(m, pai.wrapping_add(OB_ROUTINE_2), 0);
        wb(m, pai.wrapping_add(OB_ROUTINE), 2);
        let vy = 0u16.wrapping_sub(rw(m, a.wrapping_add(OB_VELY)));
        ww(m, V_PLAYER.wrapping_add(OB_VELY), vy);
        let estado = rb(m, V_PLAYER.wrapping_add(OB_STATUS));
        wb(
            m,
            V_PLAYER.wrapping_add(OB_STATUS),
            (estado | STATUS_IN_AIR) & !STATUS_ON_OBJECT,
        );
        wb(m, V_PLAYER.wrapping_add(0x3C), 0);
        wb(m, V_PLAYER.wrapping_add(OB_ANIM), 0x10);
        wb(m, V_PLAYER.wrapping_add(OB_ROUTINE), 2);
        wb(m, V_SOUNDQUEUE1, 0xCC);
    }
    ww(m, a.wrapping_add(OB_VELX), 0);
    ww(m, a.wrapping_add(OB_VELY), 0);
    wb(m, a.wrapping_add(OB_ROUTINE), 8);
}
fn bola(m: &mut Memoria, a: u32) {
    let rotina = rb(m, a.wrapping_add(OB_ROUTINE));
    if rotina == 6 {
        wb(m, a.wrapping_add(OB_ROUTINE), 8);
        wl(m, a.wrapping_add(OB_MAP), 0x120BA);
        ww(m, a.wrapping_add(OB_GFX), 0x4F0);
        let render = rb(m, a.wrapping_add(OB_RENDER));
        wb(m, a.wrapping_add(OB_RENDER), render | 4);
        wb(m, a.wrapping_add(OB_PRIORITY), 4);
        wb(m, a.wrapping_add(OB_COLTYPE), 0x8B);
        wb(m, a.wrapping_add(OB_ACTWID), 12);
        let x = rw(m, a.wrapping_add(OB_X));
        let y = rw(m, a.wrapping_add(OB_Y));
        ww(m, a.wrapping_add(ORIG_X), x);
        ww(m, a.wrapping_add(ORIG_Y), y);
        let invertida = rb(m, a.wrapping_add(OB_STATUS)) & 1 != 0;
        ww(
            m,
            a.wrapping_add(OB_X),
            if invertida {
                x.wrapping_sub(40)
            } else {
                x.wrapping_add(40)
            },
        );
        wb(m, a.wrapping_add(OB_FRAME), 1);
        if invertida {
            wb(m, a.wrapping_add(LADO), 2);
        }
    }
    let pai = rl(m, a.wrapping_add(PAI));
    if rb(m, a.wrapping_add(OB_ROUTINE)) == 8 {
        let lado = rb(m, a.wrapping_add(LADO));
        let estado = rb(m, pai.wrapping_add(LADO));
        if lado == estado {
            let y = alinhar_altura(m, a, pai);
            let base = rw(m, a.wrapping_add(ORIG_X));
            let x = if rw(m, a.wrapping_add(OB_X)) < base {
                base.wrapping_sub(40)
            } else {
                base.wrapping_add(40)
            };
            ww(m, a.wrapping_add(OB_X), x);
            ww(m, a.wrapping_add(OB_Y), y);
            ww(m, a.wrapping_add(OB_X).wrapping_add(2), 0);
            ww(m, a.wrapping_add(OB_Y).wrapping_add(2), 0);
            return;
        }
        let diferenca = lado.abs_diff(estado);
        let (vy, mut vx) = if diferenca == 1 {
            (0xF7E8, 0xFEECu16)
        } else if i16::from_be_bytes(rw(m, pai.wrapping_add(0x38)).to_be_bytes()) >= 0xA00 {
            (0xF200, 0xFF60u16)
        } else {
            (0xF510, 0xFF34u16)
        };
        if rw(m, a.wrapping_add(OB_X)) < rw(m, a.wrapping_add(ORIG_X)) {
            vx = 0u16.wrapping_sub(vx);
        }
        ww(m, a.wrapping_add(OB_VELX), vx);
        ww(m, a.wrapping_add(OB_VELY), vy);
        wb(m, a.wrapping_add(OB_ROUTINE), 10);
    }
    let vy = rw(m, a.wrapping_add(OB_VELY));
    fisica::object_fall(m, a);
    if vy & 0x8000 != 0 {
        let limite = rw(m, a.wrapping_add(ORIG_Y)).wrapping_sub(47);
        if i16::from_be_bytes(limite.to_be_bytes())
            <= i16::from_be_bytes(rw(m, a.wrapping_add(OB_Y)).to_be_bytes())
        {
            fisica::object_fall(m, a);
        }
    } else if i16::from_be_bytes(alinhar_altura(m, a, pai).to_be_bytes())
        <= i16::from_be_bytes(rw(m, a.wrapping_add(OB_Y)).to_be_bytes())
    {
        let lado = if rw(m, a.wrapping_add(OB_VELX)) & 0x8000 != 0 {
            2
        } else {
            0
        };
        lancar_sonic(m, a, pai, lado);
    }
}

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    if rb(m, a.wrapping_add(OB_ROUTINE)) == 0 {
        iniciar(m, a);
    }
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        2 => {
            let estado = rb(m, a.wrapping_add(LADO));
            mudar_quadro(m, a, estado);
            let vy = rw(m, V_PLAYER.wrapping_add(OB_VELY));
            ww(m, a.wrapping_add(0x38), vy);
            slope_object(m, a, 48)?;
        }
        4 => {
            let x = rw(m, a.wrapping_add(OB_X));
            let jogador = rw(m, V_PLAYER.wrapping_add(OB_X));
            let distancia = if x >= jogador {
                x.wrapping_sub(jogador)
            } else {
                jogador.wrapping_sub(x)
            };
            let estado = if distancia < 8 {
                1
            } else if x >= jogador {
                2
            } else {
                0
            };
            mudar_quadro(m, a, estado);
            exit_platform(m, a, 48);
            slope_object_assume_stood_on(m, a, 48);
        }
        6 | 8 | 10 => bola(m, a),
        _ => return Err(ErroAbertura::NaoImplementado("rotina da gangorra SLZ")),
    }
    let x = rw(m, a.wrapping_add(ORIG_X));
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
    fn gangorra_inicial_cria_bola_e_preserva_pai() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x5E);
        ww(&mut m, a.wrapping_add(OB_X), 0x200);
        ww(&mut m, a.wrapping_add(OB_Y), 0x180);
        executar(&mut m, a).unwrap();
        let b = a.wrapping_add(OBJECT_SIZE);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(rb(&mut m, b.wrapping_add(OB_ID)), 0x5E);
        assert_eq!(rb(&mut m, b.wrapping_add(OB_ROUTINE)), 6);
        assert_eq!(rl(&mut m, b.wrapping_add(PAI)), a);
        executar(&mut m, b).unwrap();
        assert_eq!(rb(&mut m, b.wrapping_add(OB_ROUTINE)), 8);
        assert_eq!(rw(&mut m, b.wrapping_add(OB_X)), 0x228);
    }
    #[test]
    fn mudanca_de_lado_lanca_bola_para_o_alto() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let pai = V_LVLOBJSPACE;
        let bola = pai.wrapping_add(OBJECT_SIZE);
        wb(&mut m, pai.wrapping_add(LADO), 2);
        wb(&mut m, bola.wrapping_add(OB_ROUTINE), 8);
        ww(&mut m, bola.wrapping_add(ORIG_X), 0x200);
        ww(&mut m, bola.wrapping_add(ORIG_Y), 0x180);
        ww(&mut m, bola.wrapping_add(OB_X), 0x228);
        wl(&mut m, bola.wrapping_add(PAI), pai);
        executar(&mut m, bola).unwrap();
        assert_eq!(rb(&mut m, bola.wrapping_add(OB_ROUTINE)), 10);
        assert_eq!(rw(&mut m, bola.wrapping_add(OB_VELX)), 0xFF34);
        assert!(rw(&mut m, bola.wrapping_add(OB_VELY)) & 0x8000 != 0);
    }
    #[test]
    fn estado_ajusta_quadro_gradualmente() {
        let mut m = Memoria::nova(vec![]);
        let a = V_LVLOBJSPACE;
        mudar_quadro(&mut m, a, 2);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_FRAME)), 1);
        mudar_quadro(&mut m, a, 2);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_FRAME)), 2);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_RENDER)) & 1, 1);
    }
}
