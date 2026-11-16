//! Chefe da Spring Yard e blocos da arena (REV01).
use super::super::ram::*;
use super::super::{ErroAbertura, fisica, objetos, rb, rl, rw, wb, wl, ww};
use crate::memoria::Memoria;

const MAP_BLOCK: u32 = 0x19FAE;
const COMANDO: u32 = 0x29;
const PAI: u32 = 0x34;

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

const MAP_EGGMAN: u32 = 0x184B8;
const ANI_EGGMAN: u32 = 0x18468;
const MAP_BOSS_ITEMS: u32 = 0x18580;
const BOSS_X: u32 = 0x30;
const BOSS_Y: u32 = 0x38;
const TIMER: u32 = 0x3C;

fn criar_chefe(m: &mut Memoria, a: u32) {
    ww(m, a.wrapping_add(OB_X), 0x2DB0);
    ww(m, a.wrapping_add(OB_Y), 0x4DA);
    ww(m, a.wrapping_add(BOSS_X), 0x2DB0);
    ww(m, a.wrapping_add(BOSS_Y), 0x4DA);
    wb(m, a.wrapping_add(OB_COLTYPE), 0x0F);
    wb(m, a.wrapping_add(0x21), 8);
    let mut slot = a;
    for (rotina, anim) in [(2, 0), (4, 1), (6, 7), (8, 0)] {
        wb(m, slot.wrapping_add(OB_ID), 0x75);
        wb(m, slot.wrapping_add(OB_ROUTINE), rotina);
        wb(m, slot.wrapping_add(OB_ROUTINE_2), 0);
        wb(m, slot.wrapping_add(OB_ANIM), anim);
        wb(m, slot.wrapping_add(OB_PRIORITY), 5);
        wl(m, slot.wrapping_add(OB_MAP), MAP_EGGMAN);
        ww(m, slot.wrapping_add(OB_GFX), 0x400);
        wb(m, slot.wrapping_add(OB_RENDER), 4);
        wb(m, slot.wrapping_add(OB_ACTWID), 32);
        ww(m, slot.wrapping_add(OB_X), 0x2DB0);
        ww(m, slot.wrapping_add(OB_Y), 0x4DA);
        wl(m, slot.wrapping_add(PAI), a);
        if rotina == 8 {
            break;
        }
        let Some(next) = proximo_livre(m, slot) else {
            break;
        };
        slot = next;
    }
    let status = rb(m, a.wrapping_add(OB_STATUS));
    wb(m, a.wrapping_add(OB_STATUS), status & !1);
}

fn mover_chefe(m: &mut Memoria, a: u32) {
    for (pos, vel) in [(BOSS_X, OB_VELX), (BOSS_Y, OB_VELY)] {
        let velocidade = i16::from_be_bytes(rw(m, a.wrapping_add(vel)).to_be_bytes());
        let deslocamento = i32::from(velocidade).wrapping_shl(8);
        let novo =
            rl(m, a.wrapping_add(pos)).wrapping_add(u32::from_be_bytes(deslocamento.to_be_bytes()));
        wl(m, a.wrapping_add(pos), novo);
    }
    let x = rw(m, a.wrapping_add(BOSS_X));
    let y = rw(m, a.wrapping_add(BOSS_Y));
    ww(m, a.wrapping_add(OB_X), x);
    ww(m, a.wrapping_add(OB_Y), y);
}

fn atualizar_estado(m: &mut Memoria, a: u32) {
    let x = rw(m, a.wrapping_add(OB_X));
    wb(
        m,
        a.wrapping_add(PAI),
        u8::try_from(x.wrapping_sub(0x2C00) >> 5).unwrap_or(0),
    );
    if rb(m, a.wrapping_add(OB_STATUS)) & 0x80 != 0 {
        fisica::add_points(m, 100);
        wb(m, a.wrapping_add(OB_ROUTINE_2), 6);
        ww(m, a.wrapping_add(TIMER), 180);
        ww(m, a.wrapping_add(OB_VELX), 0);
        return;
    }
    if rb(m, a.wrapping_add(OB_COLTYPE)) == 0 {
        let mut flash = rb(m, a.wrapping_add(0x3E));
        if flash == 0 {
            flash = 32;
            wb(m, V_SOUNDQUEUE1, 0xAC);
        }
        let cor = if rw(m, V_PALETTE.wrapping_add(0x22)) == 0 {
            0xEEE
        } else {
            0
        };
        ww(m, V_PALETTE.wrapping_add(0x22), cor);
        flash = flash.wrapping_sub(1);
        wb(m, a.wrapping_add(0x3E), flash);
        if flash == 0 {
            wb(m, a.wrapping_add(OB_COLTYPE), 0x0F);
        }
    }
}

fn renderizar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    objetos::animate_sprite(m, a, ANI_EGGMAN)?;
    let render = (rb(m, a.wrapping_add(OB_RENDER)) & !3) | (rb(m, a.wrapping_add(OB_STATUS)) & 3);
    wb(m, a.wrapping_add(OB_RENDER), render);
    objetos::display_sprite(m, a);
    Ok(())
}

fn bloco_alvo(m: &mut Memoria, a: u32) -> Option<u32> {
    let endereco = rw(m, a.wrapping_add(0x36));
    (endereco != 0).then_some(0xFF_0000u32 | u32::from(endereco))
}

fn atacar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let alvo = bloco_alvo(m, a);
    match rb(m, a.wrapping_add(OB_SUBTYPE)) {
        0 => {
            ww(m, a.wrapping_add(OB_VELY), 0x180);
            if rw(m, a.wrapping_add(BOSS_Y)) >= 0x556 {
                ww(m, a.wrapping_add(BOSS_Y), 0x556);
                ww(m, a.wrapping_add(TIMER), 0);
                if let Some(bloco) = alvo {
                    wb(m, bloco.wrapping_add(COMANDO), 0xFF);
                    wb(m, a.wrapping_add(COMANDO), 0xFF);
                    wl(m, bloco.wrapping_add(PAI), a);
                    ww(m, a.wrapping_add(TIMER), 50);
                }
                ww(m, a.wrapping_add(OB_VELY), 0);
                wb(m, a.wrapping_add(OB_SUBTYPE), 2);
            }
            mover_chefe(m, a);
        }
        2 => {
            let tempo = rw(m, a.wrapping_add(TIMER)).wrapping_sub(1);
            ww(m, a.wrapping_add(TIMER), tempo);
            if tempo & 0x8000 != 0 {
                wb(m, a.wrapping_add(OB_SUBTYPE), 4);
                ww(
                    m,
                    a.wrapping_add(OB_VELY),
                    if alvo.is_some() { 0xF800 } else { 0xFC00 },
                );
            }
            let oscilacao = if tempo & 0x8000 == 0 && tempo <= 30 {
                if tempo & 2 != 0 { 0xFFFE } else { 2 }
            } else {
                0
            };
            let x = rw(m, a.wrapping_add(BOSS_X));
            let y = rw(m, a.wrapping_add(BOSS_Y)).wrapping_add(oscilacao);
            ww(m, a.wrapping_add(OB_X), x);
            ww(m, a.wrapping_add(OB_Y), y);
        }
        4 => {
            let limite = if alvo.is_some() { 0x4C2 } else { 0x4DA };
            let y = rw(m, a.wrapping_add(BOSS_Y));
            if i16::from_be_bytes(y.to_be_bytes()) <= limite {
                ww(
                    m,
                    a.wrapping_add(TIMER),
                    if alvo.is_some() { 45 } else { 8 },
                );
                wb(m, a.wrapping_add(OB_SUBTYPE), 6);
                ww(m, a.wrapping_add(OB_VELY), 0);
            } else {
                let vel = rw(m, a.wrapping_add(OB_VELY));
                if i16::from_be_bytes(vel.to_be_bytes()) < -64 {
                    ww(m, a.wrapping_add(OB_VELY), vel.wrapping_add(12));
                }
            }
            mover_chefe(m, a);
        }
        6 => {
            let tempo = rw(m, a.wrapping_add(TIMER)).wrapping_sub(1);
            ww(m, a.wrapping_add(TIMER), tempo);
            if tempo == 0 {
                if let Some(bloco) = alvo {
                    wb(m, bloco.wrapping_add(COMANDO), 10);
                }
                ww(m, a.wrapping_add(0x36), 0);
            }
            if tempo == u16::from_be_bytes((-30i16).to_be_bytes()) {
                wb(m, a.wrapping_add(COMANDO), 0);
                wb(m, a.wrapping_add(OB_ROUTINE_2), 2);
                wb(m, a.wrapping_add(0x3D), 0xFF);
            } else {
                let alvo_atual = bloco_alvo(m, a);
                let y = rw(m, a.wrapping_add(BOSS_Y));
                let passo = if alvo_atual.is_some() { 2 } else { 1 };
                let y = if y == 0x4DA {
                    y
                } else if y < 0x4DA {
                    y.wrapping_add(passo)
                } else {
                    y.wrapping_sub(passo)
                };
                ww(m, a.wrapping_add(BOSS_Y), y);
                let x = rw(m, a.wrapping_add(BOSS_X));
                let oscilacao = if alvo_atual.is_some() {
                    if tempo & 1 != 0 { 0xFFFE } else { 2 }
                } else {
                    0
                };
                ww(m, a.wrapping_add(OB_X), x);
                ww(m, a.wrapping_add(OB_Y), y.wrapping_add(oscilacao));
            }
        }
        _ => return Err(ErroAbertura::NaoImplementado("fase de ataque do chefe SYZ")),
    }
    atualizar_estado(m, a);
    Ok(())
}

/// Cria a nave e seus componentes e executa a aproximação inicial.
pub fn chefe(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    if rb(m, a.wrapping_add(OB_ROUTINE)) == 0 {
        criar_chefe(m, a);
    }
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        2 => {
            let rotina = rb(m, a.wrapping_add(OB_ROUTINE_2));
            if rotina >= 6 {
                return derrota(m, a, rotina);
            }
            if rb(m, a.wrapping_add(OB_ROUTINE_2)) == 4 {
                atacar(m, a)?;
                return renderizar(m, a);
            }
            match rb(m, a.wrapping_add(OB_ROUTINE_2)) {
                0 => {
                    ww(m, a.wrapping_add(OB_VELX), 0xFF00);
                    if rw(m, a.wrapping_add(BOSS_X)) < 0x2D38 {
                        wb(m, a.wrapping_add(OB_ROUTINE_2), 2);
                    }
                }
                2 => {
                    let x = rw(m, a.wrapping_add(BOSS_X));
                    let status = rb(m, a.wrapping_add(OB_STATUS));
                    let direita = status & 1 != 0;
                    ww(
                        m,
                        a.wrapping_add(OB_VELX),
                        if direita { 0x140 } else { 0xFEC0 },
                    );
                    if (!direita && i16::from_be_bytes(x.to_be_bytes()) <= 0x2C08)
                        || (direita && i16::from_be_bytes(x.to_be_bytes()) >= 0x2D38)
                    {
                        wb(m, a.wrapping_add(OB_STATUS), status ^ 1);
                        wb(m, a.wrapping_add(0x3D), 0);
                    }
                    let alinhamento = x.wrapping_sub(0x2C10) & 0x1F;
                    let sonic_x = rw(m, V_PLAYER.wrapping_add(OB_X));
                    let indice_sonic =
                        i16::from_be_bytes(sonic_x.wrapping_sub(0x2C00).to_be_bytes()) >> 5;
                    let indice = rb(m, a.wrapping_add(PAI));
                    if alinhamento >= 30
                        && rb(m, a.wrapping_add(0x3D)) == 0
                        && indice_sonic.to_be_bytes()[1] == indice
                    {
                        let centro = 0x2C10u16.wrapping_add(u16::from(indice).wrapping_mul(32));
                        ww(m, a.wrapping_add(BOSS_X), centro);
                        ww(m, a.wrapping_add(0x36), 0);
                        let mut slot = V_OBJSPACE.wrapping_add(OBJECT_SIZE);
                        for _ in 0..63 {
                            if rb(m, slot.wrapping_add(OB_ID)) == 0x76
                                && rb(m, slot.wrapping_add(OB_SUBTYPE)) == indice
                            {
                                ww(
                                    m,
                                    a.wrapping_add(0x36),
                                    u16::try_from(slot & 0xFFFF).unwrap_or(0),
                                );
                                break;
                            }
                            slot = slot.wrapping_add(OBJECT_SIZE);
                        }
                        wb(m, a.wrapping_add(OB_ROUTINE_2), 4);
                        ww(m, a.wrapping_add(OB_SUBTYPE), 0);
                        ww(m, a.wrapping_add(OB_VELX), 0);
                    }
                }
                _ => {
                    return Err(ErroAbertura::NaoImplementado(
                        "ataque e derrota do chefe SYZ",
                    ));
                }
            }
            let fase = rb(m, a.wrapping_add(0x3F));
            wb(m, a.wrapping_add(0x3F), fase.wrapping_add(2));
            let (seno, _) = fisica::calc_sine(m, fase);
            ww(
                m,
                a.wrapping_add(OB_VELY),
                u16::from_be_bytes((seno >> 2).to_be_bytes()),
            );
            mover_chefe(m, a);
            atualizar_estado(m, a);
            renderizar(m, a)?;
        }
        4 | 6 | 8 => {
            let pai = rl(m, a.wrapping_add(PAI));
            if rb(m, pai.wrapping_add(OB_ID)) != 0x75 {
                objetos::delete_object(m, a);
                return Ok(());
            }
            let x = rw(m, pai.wrapping_add(OB_X));
            let y = rw(m, pai.wrapping_add(OB_Y));
            ww(m, a.wrapping_add(OB_X), x);
            ww(m, a.wrapping_add(OB_Y), y);
            let status = rb(m, pai.wrapping_add(OB_STATUS));
            wb(m, a.wrapping_add(OB_STATUS), status);
            let fase = rb(m, pai.wrapping_add(OB_ROUTINE_2));
            if rb(m, a.wrapping_add(OB_ROUTINE)) == 8 {
                wl(m, a.wrapping_add(OB_MAP), MAP_BOSS_ITEMS);
                ww(m, a.wrapping_add(OB_GFX), 0x246C);
                wb(m, a.wrapping_add(OB_FRAME), 5);
                wb(m, a.wrapping_add(OB_ACTWID), 8);
                wb(m, a.wrapping_add(OB_HEIGHT), 12);
                let mut desloc = rw(m, a.wrapping_add(TIMER));
                let subfase = rb(m, pai.wrapping_add(OB_SUBTYPE));
                if fase == 4 && subfase == 0 {
                    if i16::from_be_bytes(desloc.to_be_bytes()) < 0x94 {
                        desloc = desloc.wrapping_add(7);
                    }
                } else if (fase != 4
                    || (subfase == 6 && rw(m, pai.wrapping_add(TIMER)) & 0x8000 != 0))
                    && i16::from_be_bytes(desloc.to_be_bytes()) > 0
                {
                    desloc = desloc.wrapping_sub(5);
                }
                ww(m, a.wrapping_add(TIMER), desloc);
                let vertical = i16::from_be_bytes(desloc.to_be_bytes()) >> 2;
                ww(
                    m,
                    a.wrapping_add(OB_Y),
                    y.wrapping_add(u16::from_be_bytes(vertical.to_be_bytes())),
                );
                let perigoso = rb(m, pai.wrapping_add(OB_COLTYPE)) != 0
                    && rb(m, pai.wrapping_add(COMANDO)) == 0;
                wb(
                    m,
                    a.wrapping_add(OB_COLTYPE),
                    if perigoso { 0x84 } else { 0 },
                );
                objetos::display_sprite(m, a);
            } else {
                if rb(m, a.wrapping_add(OB_ROUTINE)) == 6 {
                    let anim = if fase == 10 {
                        11
                    } else if rw(m, pai.wrapping_add(OB_VELX)) != 0 {
                        8
                    } else {
                        7
                    };
                    wb(m, a.wrapping_add(OB_ANIM), anim);
                } else {
                    let anim = if fase == 6 || fase == 8 {
                        10
                    } else if fase == 10 {
                        6
                    } else if rb(m, pai.wrapping_add(OB_COLTYPE)) == 0 {
                        5
                    } else if rb(m, V_PLAYER.wrapping_add(OB_ROUTINE)) >= 4 {
                        4
                    } else if fase == 4 && rb(m, pai.wrapping_add(OB_SUBTYPE)) == 2 {
                        6
                    } else {
                        1
                    };
                    wb(m, a.wrapping_add(OB_ANIM), anim);
                }
                renderizar(m, a)?;
            }
        }
        _ => return Err(ErroAbertura::NaoImplementado("rotina do chefe SYZ")),
    }
    Ok(())
}

/// `BSYZ_Explode`, `BSYZ_Recover` e `BSYZ_Escape`.
fn derrota(m: &mut Memoria, a: u32, rotina: u8) -> Result<(), ErroAbertura> {
    match rotina {
        6 => {
            let tempo = rw(m, a.wrapping_add(TIMER)).wrapping_sub(1);
            ww(m, a.wrapping_add(TIMER), tempo);
            if tempo & 0x8000 == 0 {
                super::super::boss_ghz::explosao(m, a);
            } else {
                wb(m, a.wrapping_add(OB_ROUTINE_2), 8);
                ww(m, a.wrapping_add(OB_VELY), 0);
                ww(m, a.wrapping_add(OB_VELX), 0);
                let estado = rb(m, a.wrapping_add(OB_STATUS));
                wb(m, a.wrapping_add(OB_STATUS), (estado | 1) & !0x80);
                ww(m, a.wrapping_add(TIMER), 0xFFFF);
                if rb(m, 0xFF_F7A7) == 0 {
                    wb(m, 0xFF_F7A7, 1);
                }
            }
        }
        8 => {
            let tempo = rw(m, a.wrapping_add(TIMER)).wrapping_add(1);
            ww(m, a.wrapping_add(TIMER), tempo);
            let vy = rw(m, a.wrapping_add(OB_VELY));
            if tempo == 0 {
                ww(m, a.wrapping_add(OB_VELY), 0);
            } else if tempo & 0x8000 != 0 {
                ww(m, a.wrapping_add(OB_VELY), vy.wrapping_add(0x18));
            } else if tempo < 32 {
                ww(m, a.wrapping_add(OB_VELY), vy.wrapping_sub(8));
            } else if tempo == 32 {
                ww(m, a.wrapping_add(OB_VELY), 0);
                wb(m, V_SOUNDQUEUE0, 0x85);
            } else if tempo >= 42 {
                wb(m, a.wrapping_add(OB_ROUTINE_2), 10);
            }
            mover_chefe(m, a);
        }
        10 => {
            ww(m, a.wrapping_add(OB_VELX), 0x400);
            ww(m, a.wrapping_add(OB_VELY), 0xFFC0);
            let limite = rw(m, V_LIMITRIGHT2);
            if limite < 0x2D40 {
                ww(m, V_LIMITRIGHT2, limite.wrapping_add(2));
            } else if rb(m, a.wrapping_add(OB_RENDER)) & 0x80 == 0 {
                objetos::delete_object(m, a);
                return renderizar(m, a);
            }
            mover_chefe(m, a);
            let fase = rb(m, a.wrapping_add(0x3F));
            wb(m, a.wrapping_add(0x3F), fase.wrapping_add(2));
            let (seno, _) = fisica::calc_sine(m, fase);
            ww(
                m,
                a.wrapping_add(OB_VELY),
                u16::from_be_bytes((seno >> 2).to_be_bytes()),
            );
            mover_chefe(m, a);
        }
        _ => {
            return Err(ErroAbertura::NaoImplementado(
                "rotina de derrota do chefe SYZ",
            ));
        }
    }
    renderizar(m, a)
}

fn criar_blocos(m: &mut Memoria, a: u32) {
    let mut slot = a;
    for i in 0..10u16 {
        wb(m, slot.wrapping_add(OB_ID), 0x76);
        wl(m, slot.wrapping_add(OB_MAP), MAP_BLOCK);
        ww(m, slot.wrapping_add(OB_GFX), 0x4000);
        wb(m, slot.wrapping_add(OB_RENDER), 4);
        wb(m, slot.wrapping_add(OB_ACTWID), 16);
        wb(m, slot.wrapping_add(OB_HEIGHT), 16);
        wb(m, slot.wrapping_add(OB_PRIORITY), 3);
        ww(
            m,
            slot.wrapping_add(OB_X),
            0x2C10u16.wrapping_add(i.wrapping_mul(32)),
        );
        ww(m, slot.wrapping_add(OB_Y), 0x582);
        ww(m, slot.wrapping_add(OB_SUBTYPE), i.wrapping_mul(0x101));
        wb(m, slot.wrapping_add(OB_ROUTINE), 2);
        if i == 9 {
            break;
        }
        let Some(p) = objetos::find_free_obj(m) else {
            break;
        };
        slot = p;
    }
}

fn quebrar(m: &mut Memoria, a: u32) {
    wb(m, a.wrapping_add(OB_ROUTINE), 4);
    wb(m, a.wrapping_add(OB_ACTWID), 8);
    wb(m, a.wrapping_add(OB_HEIGHT), 8);
    let mut slot = a;
    for (i, (vx, vy, dx, dy)) in [
        (0xFE80, 0xFE00, 0xFFF8, 0xFFF8),
        (0x0180, 0xFE00, 0x0010, 0),
        (0xFF00, 0xFF00, 0, 0x0010),
        (0x0100, 0xFF00, 0x0010, 0x0010),
    ]
    .into_iter()
    .enumerate()
    {
        if slot != a {
            for offset in (0..OBJECT_SIZE).step_by(4) {
                let v = rl(m, a.wrapping_add(offset));
                wl(m, slot.wrapping_add(offset), v);
            }
        }
        ww(m, slot.wrapping_add(OB_VELX), vx);
        ww(m, slot.wrapping_add(OB_VELY), vy);
        let x = rw(m, slot.wrapping_add(OB_X)).wrapping_add(dx);
        let y = rw(m, slot.wrapping_add(OB_Y)).wrapping_add(dy);
        ww(m, slot.wrapping_add(OB_X), x);
        ww(m, slot.wrapping_add(OB_Y), y);
        wb(
            m,
            slot.wrapping_add(OB_FRAME),
            u8::try_from(i.wrapping_add(1)).unwrap_or(0),
        );
        if i == 3 {
            break;
        }
        let Some(p) = proximo_livre(m, slot) else {
            break;
        };
        slot = p;
    }
    wb(m, V_SOUNDQUEUE1, 0xCB);
}

pub fn bloco(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => criar_blocos(m, a),
        2 => {
            let cmd = rb(m, a.wrapping_add(COMANDO));
            if cmd == rb(m, a.wrapping_add(OB_SUBTYPE)) {
                objetos::solid_object(m, a, 27, 16, 17)?;
            } else if cmd & 0x80 != 0 {
                let pai = rl(m, a.wrapping_add(PAI));
                if rb(m, pai.wrapping_add(0x21)) == 0 {
                    quebrar(m, a);
                } else {
                    let x = rw(m, pai.wrapping_add(OB_X));
                    let mut y = rw(m, pai.wrapping_add(OB_Y)).wrapping_add(44);
                    if pai >= a {
                        let vel =
                            i16::from_be_bytes(rw(m, pai.wrapping_add(OB_VELY)).to_be_bytes()) >> 8;
                        y = y.wrapping_add(u16::from_be_bytes(vel.to_be_bytes()));
                    }
                    ww(m, a.wrapping_add(OB_X), x);
                    ww(m, a.wrapping_add(OB_Y), y);
                }
            } else {
                quebrar(m, a);
            }
            objetos::display_sprite(m, a);
        }
        4 => {
            if rb(m, a.wrapping_add(OB_RENDER)) & 0x80 == 0 {
                objetos::delete_object(m, a);
            } else {
                fisica::object_fall(m, a);
                objetos::display_sprite(m, a);
            }
        }
        _ => {
            return Err(ErroAbertura::NaoImplementado(
                "rotina dos blocos do chefe SYZ",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn derrota_avanca_recuperacao_e_libera_limite() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x75);
        wb(&mut m, a.wrapping_add(OB_ROUTINE), 2);
        wb(&mut m, a.wrapping_add(OB_ROUTINE_2), 6);
        chefe(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE_2)), 8);
        assert_eq!(rb(&mut m, 0xFF_F7A7), 1);
        for _ in 0..43 {
            chefe(&mut m, a).unwrap();
        }
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE_2)), 10);
        ww(&mut m, V_LIMITRIGHT2, 0x2D00);
        chefe(&mut m, a).unwrap();
        assert_eq!(rw(&mut m, V_LIMITRIGHT2), 0x2D02);
    }
    #[test]
    fn chefe_cria_componentes_e_inicia_aproximacao() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        chefe(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(rw(&mut m, a.wrapping_add(OB_X)), 0x2DAF);
        assert_eq!(rb(&mut m, a.wrapping_add(0x21)), 8);
        for i in 1..4u32 {
            let slot = a.wrapping_add(i.wrapping_mul(OBJECT_SIZE));
            assert_eq!(rb(&mut m, slot.wrapping_add(OB_ID)), 0x75);
            assert_eq!(rl(&mut m, slot.wrapping_add(PAI)), a);
            assert_eq!(
                rb(&mut m, slot.wrapping_add(OB_ROUTINE)),
                u8::try_from(i.wrapping_mul(2).wrapping_add(2)).unwrap()
            );
        }
    }

    #[test]
    fn ataque_captura_e_comanda_destruicao_do_bloco() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        let bloco = a.wrapping_add(OBJECT_SIZE);
        wb(&mut m, a.wrapping_add(OB_ID), 0x75);
        wb(&mut m, a.wrapping_add(OB_ROUTINE), 2);
        wb(&mut m, a.wrapping_add(OB_ROUTINE_2), 4);
        wb(&mut m, a.wrapping_add(OB_COLTYPE), 15);
        ww(&mut m, a.wrapping_add(BOSS_Y), 0x556);
        ww(
            &mut m,
            a.wrapping_add(0x36),
            u16::try_from(bloco & 0xFFFF).unwrap(),
        );
        wb(&mut m, bloco.wrapping_add(OB_ID), 0x76);
        chefe(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, bloco.wrapping_add(COMANDO)), 0xFF);
        assert_eq!(rw(&mut m, a.wrapping_add(TIMER)), 50);
        assert_eq!(rl(&mut m, bloco.wrapping_add(PAI)), a);
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 6);
        ww(&mut m, a.wrapping_add(TIMER), 1);
        chefe(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, bloco.wrapping_add(COMANDO)), 10);
        assert_eq!(rw(&mut m, a.wrapping_add(0x36)), 0);
    }

    #[test]
    fn patrulha_escolhe_bloco_na_posicao_de_sonic() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let boss = V_LVLOBJSPACE;
        let bloco = boss.wrapping_add(OBJECT_SIZE);
        wb(&mut m, boss.wrapping_add(OB_ID), 0x75);
        wb(&mut m, boss.wrapping_add(OB_ROUTINE), 2);
        wb(&mut m, boss.wrapping_add(OB_ROUTINE_2), 2);
        wb(&mut m, boss.wrapping_add(OB_COLTYPE), 15);
        wb(&mut m, boss.wrapping_add(PAI), 1);
        ww(&mut m, boss.wrapping_add(BOSS_X), 0x2C4F);
        ww(&mut m, boss.wrapping_add(BOSS_Y), 0x4DA);
        ww(&mut m, V_PLAYER.wrapping_add(OB_X), 0x2C20);
        wb(&mut m, bloco.wrapping_add(OB_ID), 0x76);
        wb(&mut m, bloco.wrapping_add(OB_SUBTYPE), 1);
        chefe(&mut m, boss).unwrap();
        assert_eq!(rb(&mut m, boss.wrapping_add(OB_ROUTINE_2)), 4);
        assert_eq!(
            rw(&mut m, boss.wrapping_add(0x36)),
            u16::try_from(bloco & 0xFFFF).unwrap()
        );
        assert_eq!(rw(&mut m, boss.wrapping_add(BOSS_X)), 0x2C30);
    }

    #[test]
    fn cria_dez_blocos_nas_posicoes_da_arena() {
        let mut m = Memoria::nova(vec![]);
        wb(&mut m, V_LVLOBJSPACE, 0x76);
        bloco(&mut m, V_LVLOBJSPACE).unwrap();
        for i in 0..10u32 {
            let a = V_LVLOBJSPACE.wrapping_add(i.wrapping_mul(OBJECT_SIZE));
            assert_eq!(rb(&mut m, a.wrapping_add(OB_ID)), 0x76);
            assert_eq!(
                rw(&mut m, a.wrapping_add(OB_X)),
                0x2C10u16.wrapping_add(u16::try_from(i).unwrap().wrapping_mul(32))
            );
            assert_eq!(
                rw(&mut m, a.wrapping_add(OB_SUBTYPE)),
                u16::try_from(i).unwrap().wrapping_mul(0x101)
            );
        }
    }
    #[test]
    fn bloco_quebrado_cria_quatro_fragmentos() {
        let mut m = Memoria::nova(vec![]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x76);
        wb(&mut m, a.wrapping_add(OB_ROUTINE), 2);
        wb(&mut m, a.wrapping_add(COMANDO), 10);
        bloco(&mut m, a).unwrap();
        for i in 0..4u32 {
            let slot = a.wrapping_add(i.wrapping_mul(OBJECT_SIZE));
            assert_eq!(rb(&mut m, slot.wrapping_add(OB_ROUTINE)), 4);
            assert_eq!(
                rb(&mut m, slot.wrapping_add(OB_FRAME)),
                u8::try_from(i.wrapping_add(1)).unwrap()
            );
        }
    }
}
