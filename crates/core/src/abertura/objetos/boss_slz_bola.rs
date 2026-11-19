//! Bola explosiva do chefe da Star Light, objeto 0x7B (REV01).
use super::super::boot::rom_w;
use super::super::ram::*;
use super::super::{ErroAbertura, fisica, rb, rl, rw, wb, wl, ww};
use super::{delete_object, display_sprite, find_free_obj, fora_da_janela};
use crate::memoria::Memoria;

const BASE_X: u32 = 0x30;
const BASE_Y: u32 = 0x34;
const LADO: u32 = 0x3A;
const GANGORRA: u32 = 0x3C;
const OFFSET_Y: u32 = 0x197C8;

fn altura(m: &mut Memoria, a: u32, gangorra: u32) -> u16 {
    let mut quadro = u32::from(rb(m, gangorra.wrapping_add(OB_FRAME)));
    if rw(m, a.wrapping_add(OB_X)) < rw(m, a.wrapping_add(BASE_X)) {
        quadro = quadro.wrapping_add(2);
    }
    let desloc = rom_w(m, OFFSET_Y.wrapping_add(quadro.wrapping_mul(2)));
    rw(m, a.wrapping_add(BASE_Y)).wrapping_add(desloc)
}

fn piscar(m: &mut Memoria, a: u32) {
    let tempo = rw(m, a.wrapping_add(OB_SUBTYPE));
    if tempo == 120 {
        wb(m, a.wrapping_add(OB_DELAY_ANI), 5);
    }
    if tempo == 60 {
        wb(m, a.wrapping_add(OB_DELAY_ANI), 2);
    }
    let t = rb(m, a.wrapping_add(OB_TIME_FRAME)).wrapping_sub(1);
    wb(m, a.wrapping_add(OB_TIME_FRAME), t);
    if i8::from_be_bytes([t]) <= 0 {
        let frame = rb(m, a.wrapping_add(OB_FRAME));
        wb(m, a.wrapping_add(OB_FRAME), frame ^ 1);
        let atraso = rb(m, a.wrapping_add(OB_DELAY_ANI));
        wb(m, a.wrapping_add(OB_TIME_FRAME), atraso);
    }
}

fn lancar_sonic(m: &mut Memoria, a: u32, gangorra: u32, lado: u8) {
    wb(m, gangorra.wrapping_add(LADO), lado);
    wb(m, a.wrapping_add(LADO), lado);
    let frame = rb(m, gangorra.wrapping_add(OB_FRAME));
    let status = rb(m, gangorra.wrapping_add(OB_STATUS));
    if lado != frame && status & 8 != 0 {
        wb(m, gangorra.wrapping_add(OB_STATUS), status & !8);
        wb(m, gangorra.wrapping_add(OB_ROUTINE_2), 0);
        wb(m, gangorra.wrapping_add(OB_ROUTINE), 2);
        let mut vy = 0u16.wrapping_sub(rw(m, a.wrapping_add(OB_VELY)));
        if frame == 1 {
            vy = u16::from_be_bytes((i16::from_be_bytes(vy.to_be_bytes()) >> 1).to_be_bytes());
        }
        ww(m, V_PLAYER.wrapping_add(OB_VELY), vy);
        let s = rb(m, V_PLAYER.wrapping_add(OB_STATUS));
        wb(m, V_PLAYER.wrapping_add(OB_STATUS), (s | 2) & !8);
        wb(m, V_PLAYER.wrapping_add(0x3C), 0);
        if s & STATUS_ROLLING == 0 {
            let s = rb(m, V_PLAYER.wrapping_add(OB_STATUS));
            wb(m, V_PLAYER.wrapping_add(OB_STATUS), s | STATUS_ROLLING);
            wb(m, V_PLAYER.wrapping_add(OB_HEIGHT), 14);
            wb(m, V_PLAYER.wrapping_add(OB_WIDTH), 7);
            wb(m, V_PLAYER.wrapping_add(OB_ANIM), ANIM_ROLL);
            let y = rw(m, V_PLAYER.wrapping_add(OB_Y));
            ww(m, V_PLAYER.wrapping_add(OB_Y), y.wrapping_add(5));
            if rw(m, V_PLAYER.wrapping_add(OB_INERTIA)) == 0 {
                ww(m, V_PLAYER.wrapping_add(OB_INERTIA), 0x200);
            }
        }
        wb(m, V_PLAYER.wrapping_add(OB_ROUTINE), 2);
        wb(m, V_SOUNDQUEUE1, 0xCC);
    }
    ww(m, a.wrapping_add(OB_VELX), 0);
    ww(m, a.wrapping_add(OB_VELY), 0);
    let rotina = rb(m, a.wrapping_add(OB_ROUTINE)).wrapping_add(2);
    wb(m, a.wrapping_add(OB_ROUTINE), rotina);
    piscar(m, a);
}

fn queda(m: &mut Memoria, a: u32) {
    fisica::object_fall(m, a);
    let gangorra = rl(m, a.wrapping_add(GANGORRA));
    let limite = altura(m, a, gangorra);
    if i16::from_be_bytes(limite.to_be_bytes())
        <= i16::from_be_bytes(rw(m, a.wrapping_add(OB_Y)).to_be_bytes())
    {
        let lado = if rb(m, a.wrapping_add(OB_STATUS)) & 1 != 0 {
            0
        } else {
            2
        };
        ww(m, a.wrapping_add(OB_SUBTYPE), 240);
        wb(m, a.wrapping_add(OB_DELAY_ANI), 10);
        wb(m, a.wrapping_add(OB_TIME_FRAME), 10);
        lancar_sonic(m, a, gangorra, lado);
    }
}

fn contato_chefe(m: &mut Memoria, a: u32) {
    let mut boss = V_OBJSPACE.wrapping_add(OBJECT_SIZE);
    for _ in 0..63 {
        if rb(m, boss.wrapping_add(OB_ID)) == 0x7A {
            let bx = rw(m, boss.wrapping_add(OB_X));
            let by = rw(m, boss.wrapping_add(OB_Y));
            let x = rw(m, a.wrapping_add(OB_X));
            let y = rw(m, a.wrapping_add(OB_Y));
            if x.wrapping_add(8) >= bx.wrapping_sub(24)
                && bx.wrapping_add(24) >= x.wrapping_sub(8)
                && y.wrapping_add(8) >= by.wrapping_sub(24)
                && by.wrapping_add(24) >= y.wrapping_sub(8)
            {
                let rotina = rb(m, a.wrapping_add(OB_ROUTINE)).wrapping_add(2);
                wb(m, a.wrapping_add(OB_ROUTINE), rotina);
                ww(m, a.wrapping_add(OB_SUBTYPE), 0);
                wb(m, boss.wrapping_add(OB_COLTYPE), 0);
                let hits = rb(m, boss.wrapping_add(0x21)).wrapping_sub(1);
                wb(m, boss.wrapping_add(0x21), hits);
                if hits == 0 {
                    let status = rb(m, boss.wrapping_add(OB_STATUS));
                    wb(m, boss.wrapping_add(OB_STATUS), status | 0x80);
                    ww(m, a.wrapping_add(OB_VELX), 0);
                    ww(m, a.wrapping_add(OB_VELY), 0);
                }
            }
            break;
        }
        boss = boss.wrapping_add(OBJECT_SIZE);
    }
}

fn movimento_lancado(m: &mut Memoria, a: u32) {
    contato_chefe(m, a);
    let gangorra = rl(m, a.wrapping_add(GANGORRA));
    let vy = rw(m, a.wrapping_add(OB_VELY));
    fisica::object_fall(m, a);
    if vy & 0x8000 != 0 {
        let limite = rw(m, a.wrapping_add(BASE_Y)).wrapping_sub(47);
        if i16::from_be_bytes(limite.to_be_bytes())
            <= i16::from_be_bytes(rw(m, a.wrapping_add(OB_Y)).to_be_bytes())
        {
            fisica::object_fall(m, a);
        }
        piscar(m, a);
    } else if i16::from_be_bytes(altura(m, a, gangorra).to_be_bytes())
        <= i16::from_be_bytes(rw(m, a.wrapping_add(OB_Y)).to_be_bytes())
    {
        let lado = if rw(m, a.wrapping_add(OB_VELX)) & 0x8000 != 0 {
            2
        } else {
            0
        };
        ww(m, a.wrapping_add(OB_SUBTYPE), 0);
        lancar_sonic(m, a, gangorra, lado);
    } else {
        piscar(m, a);
    }
}

fn quicar(m: &mut Memoria, a: u32) {
    let gangorra = rl(m, a.wrapping_add(GANGORRA));
    let lado = rb(m, a.wrapping_add(LADO));
    let lado_atual = rb(m, gangorra.wrapping_add(LADO));
    if lado != lado_atual {
        let diferenca = lado.abs_diff(lado_atual);
        let (vy, mut vx) = if diferenca == 1 {
            (0xF7E8, 0xFEECu16)
        } else if i16::from_be_bytes(rw(m, gangorra.wrapping_add(0x38)).to_be_bytes()) >= 0x9C0 {
            (0xF5E0, 0xFF80u16)
        } else {
            (0xF6A0, 0xFF0Cu16)
        };
        if rw(m, a.wrapping_add(OB_X)) < rw(m, a.wrapping_add(BASE_X)) {
            vx = 0u16.wrapping_sub(vx);
        }
        ww(m, a.wrapping_add(OB_VELX), vx);
        ww(m, a.wrapping_add(OB_VELY), vy);
        wb(m, a.wrapping_add(OB_FRAME), 1);
        ww(m, a.wrapping_add(OB_SUBTYPE), 32);
        wb(m, a.wrapping_add(OB_ROUTINE), 6);
        movimento_lancado(m, a);
    } else {
        let y = altura(m, a, gangorra);
        let base = rw(m, a.wrapping_add(BASE_X));
        let x = if rw(m, a.wrapping_add(OB_X)) < base {
            base.wrapping_sub(40)
        } else {
            base.wrapping_add(40)
        };
        ww(m, a.wrapping_add(OB_X), x);
        ww(m, a.wrapping_add(OB_Y), y);
        ww(m, a.wrapping_add(OB_X).wrapping_add(2), 0);
        ww(m, a.wrapping_add(OB_Y).wrapping_add(2), 0);
        let t = rw(m, a.wrapping_add(OB_SUBTYPE)).wrapping_sub(1);
        ww(m, a.wrapping_add(OB_SUBTYPE), t);
        if t == 0 {
            ww(m, a.wrapping_add(OB_SUBTYPE), 32);
            wb(m, a.wrapping_add(OB_ROUTINE), 8);
        } else {
            piscar(m, a);
        }
    }
}

fn explodir(m: &mut Memoria, a: u32) {
    wb(m, a.wrapping_add(OB_ID), 0x3F);
    wb(m, a.wrapping_add(OB_ROUTINE), 0);
    if rw(m, a.wrapping_add(OB_SUBTYPE)) != 32 {
        return;
    }
    let y = rw(m, a.wrapping_add(BASE_Y));
    ww(m, a.wrapping_add(OB_Y), y);
    let mut indice = 0usize;
    let velocidades = [
        (0xFF00, 0xFCC0),
        (0xFF60, 0xFDC0),
        (0x0100, 0xFCC0),
        (0x00A0, 0xFDC0),
    ];
    for _ in 0..4 {
        let Some(f) = find_free_obj(m) else {
            continue;
        };
        wb(m, f.wrapping_add(OB_ID), 0x7B);
        wb(m, f.wrapping_add(OB_ROUTINE), 10);
        wl(m, f.wrapping_add(OB_MAP), 0x1988E);
        wb(m, f.wrapping_add(OB_PRIORITY), 3);
        ww(m, f.wrapping_add(OB_GFX), 0x518);
        let x = rw(m, a.wrapping_add(OB_X));
        ww(m, f.wrapping_add(OB_X), x);
        ww(m, f.wrapping_add(OB_Y), y);
        let (vx, vy) = velocidades[indice];
        indice = indice.wrapping_add(1);
        ww(m, f.wrapping_add(OB_VELX), vx);
        ww(m, f.wrapping_add(OB_VELY), vy);
        wb(m, f.wrapping_add(OB_COLTYPE), 0x98);
        wb(m, f.wrapping_add(OB_RENDER), 0x84);
        wb(m, f.wrapping_add(OB_ACTWID), 12);
    }
}

/// Executa a bola, seus choques e seus fragmentos.
pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    if rb(m, a.wrapping_add(OB_ROUTINE)) == 0 {
        wl(m, a.wrapping_add(OB_MAP), 0x120BA);
        ww(m, a.wrapping_add(OB_GFX), 0x518);
        wb(m, a.wrapping_add(OB_FRAME), 1);
        let render = rb(m, a.wrapping_add(OB_RENDER));
        wb(m, a.wrapping_add(OB_RENDER), render | 4);
        wb(m, a.wrapping_add(OB_PRIORITY), 4);
        wb(m, a.wrapping_add(OB_COLTYPE), 0x8B);
        wb(m, a.wrapping_add(OB_ACTWID), 12);
        let gangorra = rl(m, a.wrapping_add(GANGORRA));
        let x = rw(m, gangorra.wrapping_add(OB_X));
        let y = rw(m, gangorra.wrapping_add(OB_Y));
        ww(m, a.wrapping_add(BASE_X), x);
        ww(m, a.wrapping_add(BASE_Y), y);
        let status = rb(m, a.wrapping_add(OB_STATUS));
        if i16::from_be_bytes(rw(m, a.wrapping_add(OB_X)).to_be_bytes())
            > i16::from_be_bytes(x.to_be_bytes())
        {
            wb(m, a.wrapping_add(OB_STATUS), status | 1);
        } else {
            wb(m, a.wrapping_add(OB_STATUS), status & !1);
            wb(m, a.wrapping_add(LADO), 2);
        }
        wb(m, a.wrapping_add(OB_ROUTINE), 2);
    }
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        2 => queda(m, a),
        4 => quicar(m, a),
        6 => movimento_lancado(m, a),
        8 => explodir(m, a),
        10 => {
            fisica::speed_to_pos(m, a);
            let x = rw(m, a.wrapping_add(OB_X));
            let y = rw(m, a.wrapping_add(OB_Y));
            ww(m, a.wrapping_add(BASE_X), x);
            ww(m, a.wrapping_add(BASE_Y), y);
            let vy = rw(m, a.wrapping_add(OB_VELY)).wrapping_add(0x18);
            ww(m, a.wrapping_add(OB_VELY), vy);
            let quadro = (rw(m, V_VBLANK_BYTE.wrapping_sub(1)) & 4) >> 2;
            wb(
                m,
                a.wrapping_add(OB_FRAME),
                u8::try_from(quadro).unwrap_or(0),
            );
            if rb(m, a.wrapping_add(OB_RENDER)) & 0x80 == 0 {
                delete_object(m, a);
            }
        }
        _ => return Err(ErroAbertura::NaoImplementado("rotina da bola do chefe SLZ")),
    }
    let base = rw(m, a.wrapping_add(BASE_X));
    if fora_da_janela(m, base) {
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
    fn bola_aterrissa_e_inicia_contagem() {
        let mut rom = vec![0; 0x20000];
        rom[0x197C8..0x197CA].copy_from_slice(&(-8i16).to_be_bytes());
        let mut m = Memoria::nova(rom);
        let gangorra = V_LVLOBJSPACE;
        let bola = gangorra.wrapping_add(OBJECT_SIZE);
        wb(&mut m, gangorra.wrapping_add(OB_ID), 0x5E);
        ww(&mut m, gangorra.wrapping_add(OB_X), 0x200);
        ww(&mut m, gangorra.wrapping_add(OB_Y), 0x200);
        ww(&mut m, bola.wrapping_add(OB_X), 0x228);
        ww(&mut m, bola.wrapping_add(OB_Y), 0x200);
        wl(&mut m, bola.wrapping_add(GANGORRA), gangorra);
        executar(&mut m, bola).unwrap();
        assert_eq!(rb(&mut m, bola.wrapping_add(OB_ROUTINE)), 4);
        assert_eq!(rw(&mut m, bola.wrapping_add(OB_SUBTYPE)), 240);
        assert_eq!(rb(&mut m, bola.wrapping_add(OB_COLTYPE)), 0x8B);
    }
    #[test]
    fn choque_da_bola_causa_dano_e_dispara_derrota() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let boss = V_LVLOBJSPACE;
        let bola = boss.wrapping_add(OBJECT_SIZE);
        wb(&mut m, boss.wrapping_add(OB_ID), 0x7A);
        ww(&mut m, boss.wrapping_add(OB_X), 0x200);
        ww(&mut m, boss.wrapping_add(OB_Y), 0x200);
        wb(&mut m, boss.wrapping_add(0x21), 1);
        ww(&mut m, bola.wrapping_add(OB_X), 0x200);
        ww(&mut m, bola.wrapping_add(OB_Y), 0x200);
        wb(&mut m, bola.wrapping_add(OB_ROUTINE), 6);
        contato_chefe(&mut m, bola);
        assert_eq!(rb(&mut m, bola.wrapping_add(OB_ROUTINE)), 8);
        assert_eq!(rb(&mut m, boss.wrapping_add(0x21)), 0);
        assert_eq!(rb(&mut m, boss.wrapping_add(OB_STATUS)) & 0x80, 0x80);
    }
    #[test]
    fn explosao_cria_quatro_fragmentos() {
        let mut m = Memoria::nova(vec![]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x7B);
        ww(&mut m, a.wrapping_add(OB_SUBTYPE), 32);
        explodir(&mut m, a);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ID)), 0x3F);
        for i in 1..=4u32 {
            let f = a.wrapping_add(i.wrapping_mul(OBJECT_SIZE));
            assert_eq!(rb(&mut m, f.wrapping_add(OB_ID)), 0x7B);
            assert_eq!(rb(&mut m, f.wrapping_add(OB_ROUTINE)), 10);
        }
    }
}
