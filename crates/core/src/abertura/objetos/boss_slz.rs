//! Nave do chefe da Star Light Zone, objeto 0x7A (REV01).
use super::super::ram::*;
use super::super::{ErroAbertura, fisica, rb, rl, rw, wb, wl, ww};
use super::{animate_sprite, delete_object, display_sprite};
use crate::memoria::Memoria;

const BOSS_X: u32 = 0x30;
const BOSS_Y: u32 = 0x38;
const PAI: u32 = 0x34;
const TEMP: u32 = 0x3C;
const ANI: u32 = 0x18468;

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

fn inicializar(m: &mut Memoria, a: u32) {
    ww(m, a.wrapping_add(BOSS_X), 0x2188);
    ww(m, a.wrapping_add(BOSS_Y), 0x228);
    wb(m, a.wrapping_add(OB_COLTYPE), 0x0F);
    wb(m, a.wrapping_add(0x21), 8);
    let mut slot = a;
    for (rotina, anim, prioridade) in [(2, 0, 4), (4, 1, 4), (6, 7, 4), (8, 0, 3)] {
        wb(m, slot.wrapping_add(OB_ID), 0x7A);
        wb(m, slot.wrapping_add(OB_ROUTINE), rotina);
        wb(m, slot.wrapping_add(OB_ROUTINE_2), 0);
        wb(m, slot.wrapping_add(OB_ANIM), anim);
        wb(m, slot.wrapping_add(OB_PRIORITY), prioridade);
        wl(m, slot.wrapping_add(OB_MAP), 0x184B8);
        ww(m, slot.wrapping_add(OB_GFX), 0x400);
        wb(m, slot.wrapping_add(OB_RENDER), 4);
        wb(m, slot.wrapping_add(OB_ACTWID), 32);
        ww(m, slot.wrapping_add(OB_X), 0x2188);
        ww(m, slot.wrapping_add(OB_Y), 0x228);
        wl(m, slot.wrapping_add(PAI), a);
        if rotina == 8 {
            break;
        }
        let Some(next) = proximo_livre(m, slot) else {
            break;
        };
        slot = next;
    }
    let estado = rb(m, a.wrapping_add(OB_STATUS));
    wb(m, a.wrapping_add(OB_STATUS), estado & !1);
    let mut slot = V_OBJSPACE.wrapping_add(OBJECT_SIZE);
    let mut lista = a.wrapping_add(0x2A);
    for _ in 0..63 {
        if rb(m, slot.wrapping_add(OB_ID)) == 0x5E && rb(m, slot.wrapping_add(OB_SUBTYPE)) != 0 {
            ww(m, lista, u16::try_from(slot & 0xFFFF).unwrap_or(0));
            lista = lista.wrapping_add(2);
        }
        slot = slot.wrapping_add(OBJECT_SIZE);
    }
}

fn mover(m: &mut Memoria, a: u32) {
    for (pos, vel) in [(BOSS_X, OB_VELX), (BOSS_Y, OB_VELY)] {
        let velocidade = i16::from_be_bytes(rw(m, a.wrapping_add(vel)).to_be_bytes());
        let delta = i32::from(velocidade).wrapping_shl(8);
        let valor =
            rl(m, a.wrapping_add(pos)).wrapping_add(u32::from_be_bytes(delta.to_be_bytes()));
        wl(m, a.wrapping_add(pos), valor);
    }
    let x = rw(m, a.wrapping_add(BOSS_X));
    let y = rw(m, a.wrapping_add(BOSS_Y));
    ww(m, a.wrapping_add(OB_X), x);
    ww(m, a.wrapping_add(OB_Y), y);
}

fn balancar(m: &mut Memoria, a: u32) {
    mover(m, a);
    let fase = rb(m, a.wrapping_add(0x3F));
    wb(m, a.wrapping_add(0x3F), fase.wrapping_add(2));
    let (seno, _) = fisica::calc_sine(m, fase);
    let y =
        rw(m, a.wrapping_add(BOSS_Y)).wrapping_add(u16::from_be_bytes((seno >> 6).to_be_bytes()));
    ww(m, a.wrapping_add(OB_Y), y);
}

fn estado(m: &mut Memoria, a: u32) {
    if rb(m, a.wrapping_add(OB_ROUTINE_2)) >= 6 {
        return;
    }
    if rb(m, a.wrapping_add(OB_STATUS)) & 0x80 != 0 {
        fisica::add_points(m, 100);
        wb(m, a.wrapping_add(OB_ROUTINE_2), 6);
        wb(m, a.wrapping_add(TEMP), 120);
        ww(m, a.wrapping_add(OB_VELX), 0);
    } else if rb(m, a.wrapping_add(OB_COLTYPE)) == 0 {
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
    animate_sprite(m, a, ANI)?;
    let render = (rb(m, a.wrapping_add(OB_RENDER)) & !3) | (rb(m, a.wrapping_add(OB_STATUS)) & 3);
    wb(m, a.wrapping_add(OB_RENDER), render);
    display_sprite(m, a);
    Ok(())
}

fn fase_nave(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE_2)) {
        0 => {
            ww(m, a.wrapping_add(OB_VELX), 0xFF00);
            if rw(m, a.wrapping_add(BOSS_X)) < 0x2120 {
                wb(m, a.wrapping_add(OB_ROUTINE_2), 2);
            }
            balancar(m, a);
            estado(m, a);
        }
        2 => {
            let x = rw(m, a.wrapping_add(BOSS_X));
            let estado_atual = rb(m, a.wrapping_add(OB_STATUS));
            let direita = estado_atual & 1 != 0;
            ww(
                m,
                a.wrapping_add(OB_VELX),
                if direita { 0x200 } else { 0xFE00 },
            );
            if (!direita && i16::from_be_bytes(x.to_be_bytes()) <= 0x2008)
                || (direita && i16::from_be_bytes(x.to_be_bytes()) >= 0x2138)
            {
                wb(m, a.wrapping_add(OB_STATUS), estado_atual ^ 1);
            }
            let render_x = rw(m, a.wrapping_add(OB_X));
            let desloc = if direita {
                40u16
            } else {
                0u16.wrapping_sub(40)
            };
            let mut subtipo = 0xFF;
            for i in 0..3u32 {
                let ptr = rw(m, a.wrapping_add(0x2A).wrapping_add(i.wrapping_mul(2)));
                let serra = 0xFF_0000 | u32::from(ptr);
                if rb(m, serra.wrapping_add(OB_STATUS)) & 8 == 0
                    && rw(m, serra.wrapping_add(OB_X)).wrapping_add(desloc) == render_x
                {
                    subtipo = u8::try_from(2u32.wrapping_sub(i)).unwrap_or(0);
                    wb(m, a.wrapping_add(OB_ROUTINE_2), 4);
                    wb(m, a.wrapping_add(TEMP), 40);
                    break;
                }
            }
            wb(m, a.wrapping_add(OB_SUBTYPE), subtipo);
            balancar(m, a);
            estado(m, a);
        }
        4 => {
            if rb(m, a.wrapping_add(TEMP)) == 40 {
                let tipo = rb(m, a.wrapping_add(OB_SUBTYPE));
                if tipo > 2 {
                    wb(m, a.wrapping_add(OB_ROUTINE_2), 2);
                    balancar(m, a);
                    estado(m, a);
                    return Ok(());
                }
                let pos = u32::from(2u8.wrapping_sub(tipo)).wrapping_mul(2);
                let ptr = rw(m, a.wrapping_add(0x2A).wrapping_add(pos));
                let serra = 0xFF_0000 | u32::from(ptr);
                let mut scan = V_OBJSPACE.wrapping_add(OBJECT_SIZE);
                let mut ocupada = false;
                for _ in 0..63 {
                    if rl(m, scan.wrapping_add(0x3C)) == serra {
                        ocupada = true;
                        break;
                    }
                    scan = scan.wrapping_add(OBJECT_SIZE);
                }
                if !ocupada {
                    if let Some(bola) = proximo_livre(m, serra) {
                        wb(m, bola.wrapping_add(OB_ID), 0x7B);
                        let x = rw(m, a.wrapping_add(OB_X));
                        let y = rw(m, a.wrapping_add(OB_Y)).wrapping_add(32);
                        ww(m, bola.wrapping_add(OB_X), x);
                        ww(m, bola.wrapping_add(OB_Y), y);
                        let estado_serra = rb(m, serra.wrapping_add(OB_STATUS));
                        wb(m, bola.wrapping_add(OB_STATUS), estado_serra);
                        wl(m, bola.wrapping_add(0x3C), serra);
                    } else {
                        ocupada = true;
                    }
                }
                if ocupada {
                    wb(m, a.wrapping_add(OB_ROUTINE_2), 2);
                    balancar(m, a);
                    estado(m, a);
                    return Ok(());
                }
            }
            let tempo = rb(m, a.wrapping_add(TEMP)).wrapping_sub(1);
            wb(m, a.wrapping_add(TEMP), tempo);
            if tempo == 0 {
                wb(m, a.wrapping_add(OB_ROUTINE_2), 2);
                balancar(m, a);
            }
            estado(m, a);
        }
        6 => {
            let t = rb(m, a.wrapping_add(TEMP)).wrapping_sub(1);
            wb(m, a.wrapping_add(TEMP), t);
            if t & 0x80 == 0 {
                super::super::boss_ghz::explosao(m, a);
            } else {
                wb(m, a.wrapping_add(OB_ROUTINE_2), 8);
                ww(m, a.wrapping_add(OB_VELX), 0);
                ww(m, a.wrapping_add(OB_VELY), 0);
                let status = rb(m, a.wrapping_add(OB_STATUS));
                wb(m, a.wrapping_add(OB_STATUS), (status | 1) & !0x80);
                wb(m, a.wrapping_add(TEMP), 0xE8);
                if rb(m, 0xFF_F7A7) == 0 {
                    wb(m, 0xFF_F7A7, 1);
                }
            }
        }
        8 => {
            let t = rb(m, a.wrapping_add(TEMP)).wrapping_add(1);
            wb(m, a.wrapping_add(TEMP), t);
            let vy = rw(m, a.wrapping_add(OB_VELY));
            if t == 0 {
                ww(m, a.wrapping_add(OB_VELY), 0);
            } else if t & 0x80 != 0 {
                ww(m, a.wrapping_add(OB_VELY), vy.wrapping_add(0x18));
            } else if t < 32 {
                ww(m, a.wrapping_add(OB_VELY), vy.wrapping_sub(8));
            } else if t == 32 {
                ww(m, a.wrapping_add(OB_VELY), 0);
                wb(m, V_SOUNDQUEUE0, 0x84);
            } else if t >= 42 {
                wb(m, a.wrapping_add(OB_ROUTINE_2), 10);
            }
            mover(m, a);
        }
        10 => {
            ww(m, a.wrapping_add(OB_VELX), 0x400);
            ww(m, a.wrapping_add(OB_VELY), 0xFFC0);
            let limite = rw(m, V_LIMITRIGHT2);
            if limite < 0x2160 {
                ww(m, V_LIMITRIGHT2, limite.wrapping_add(2));
            } else if rb(m, a.wrapping_add(OB_RENDER)) & 0x80 == 0 {
                delete_object(m, a);
                return renderizar(m, a);
            }
            mover(m, a);
            balancar(m, a);
        }
        _ => return Err(ErroAbertura::NaoImplementado("rotina da nave do chefe SLZ")),
    }
    renderizar(m, a)
}

/// Executa a nave e seus componentes do objeto 0x7A.
pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    if rb(m, a.wrapping_add(OB_ROUTINE)) == 0 {
        inicializar(m, a);
    }
    let rotina = rb(m, a.wrapping_add(OB_ROUTINE));
    if rotina == 2 {
        return fase_nave(m, a);
    }
    if !matches!(rotina, 4 | 6 | 8) {
        return Err(ErroAbertura::NaoImplementado("rotina do chefe SLZ"));
    }
    let pai = rl(m, a.wrapping_add(PAI));
    let fase = rb(m, pai.wrapping_add(OB_ROUTINE_2));
    if fase == 10 && rb(m, a.wrapping_add(OB_RENDER)) & 0x80 == 0 {
        delete_object(m, a);
        return Ok(());
    }
    if rotina == 4 {
        let anim = if fase == 10 {
            6
        } else if fase >= 6 {
            10
        } else if rb(m, pai.wrapping_add(OB_COLTYPE)) == 0 {
            5
        } else if rb(m, V_PLAYER.wrapping_add(OB_ROUTINE)) >= 4 {
            4
        } else {
            1
        };
        wb(m, a.wrapping_add(OB_ANIM), anim);
    } else if rotina == 6 {
        let anim = if fase == 10 {
            11
        } else if (4..=8).contains(&fase) {
            7
        } else {
            8
        };
        wb(m, a.wrapping_add(OB_ANIM), anim);
    } else {
        wl(m, a.wrapping_add(OB_MAP), 0x18580);
        ww(m, a.wrapping_add(OB_GFX), 0x246C);
        wb(m, a.wrapping_add(OB_FRAME), 3);
    }
    let x = rw(m, pai.wrapping_add(OB_X));
    let y = rw(m, pai.wrapping_add(OB_Y));
    let status = rb(m, pai.wrapping_add(OB_STATUS));
    ww(m, a.wrapping_add(OB_X), x);
    ww(m, a.wrapping_add(OB_Y), y);
    wb(m, a.wrapping_add(OB_STATUS), status);
    if rotina != 8 {
        return renderizar(m, a);
    }
    let render = (rb(m, a.wrapping_add(OB_RENDER)) & !3) | (status & 3);
    wb(m, a.wrapping_add(OB_RENDER), render);
    display_sprite(m, a);
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn inicializa_componentes_e_registra_balancas_livres() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let serra = V_LVLOBJSPACE;
        let boss = serra.wrapping_add(OBJECT_SIZE);
        wb(&mut m, serra.wrapping_add(OB_ID), 0x5E);
        wb(&mut m, serra.wrapping_add(OB_SUBTYPE), 1);
        executar(&mut m, boss).unwrap();
        assert_eq!(rw(&mut m, boss.wrapping_add(OB_X)), 0x2187);
        assert_eq!(
            rw(&mut m, boss.wrapping_add(0x2A)),
            u16::try_from(serra & 0xFFFF).unwrap()
        );
        assert_eq!(rb(&mut m, boss.wrapping_add(0x21)), 8);
        assert_eq!(
            rb(
                &mut m,
                boss.wrapping_add(OBJECT_SIZE).wrapping_add(OB_ROUTINE)
            ),
            4
        );
    }
    #[test]
    fn derrota_inicia_temporizador_e_recuperacao() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let boss = V_LVLOBJSPACE;
        wb(&mut m, boss.wrapping_add(OB_ID), 0x7A);
        wb(&mut m, boss.wrapping_add(OB_ROUTINE), 2);
        wb(&mut m, boss.wrapping_add(OB_STATUS), 0x80);
        executar(&mut m, boss).unwrap();
        assert_eq!(rb(&mut m, boss.wrapping_add(OB_ROUTINE_2)), 6);
        assert_eq!(rb(&mut m, boss.wrapping_add(TEMP)), 120);
        wb(&mut m, boss.wrapping_add(TEMP), 0);
        executar(&mut m, boss).unwrap();
        assert_eq!(rb(&mut m, boss.wrapping_add(OB_ROUTINE_2)), 8);
        assert_eq!(rb(&mut m, boss.wrapping_add(TEMP)), 0xE8);
        assert_eq!(rb(&mut m, 0xFF_F7A7), 1);
    }

    #[test]
    fn bola_e_criada_na_balanca_escolhida() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let serra = V_LVLOBJSPACE;
        let boss = serra.wrapping_add(OBJECT_SIZE);
        wb(&mut m, serra.wrapping_add(OB_ID), 0x5E);
        wb(&mut m, boss.wrapping_add(OB_ID), 0x7A);
        wb(&mut m, boss.wrapping_add(OB_ROUTINE), 2);
        wb(&mut m, boss.wrapping_add(OB_ROUTINE_2), 4);
        wb(&mut m, boss.wrapping_add(OB_SUBTYPE), 2);
        wb(&mut m, boss.wrapping_add(TEMP), 40);
        wb(&mut m, boss.wrapping_add(OB_COLTYPE), 15);
        ww(
            &mut m,
            boss.wrapping_add(0x2A),
            u16::try_from(serra & 0xFFFF).unwrap(),
        );
        executar(&mut m, boss).unwrap();
        let bola = boss.wrapping_add(OBJECT_SIZE);
        assert_eq!(rb(&mut m, bola.wrapping_add(OB_ID)), 0x7B);
        assert_eq!(rl(&mut m, bola.wrapping_add(0x3C)), serra);
        assert_eq!(rb(&mut m, boss.wrapping_add(TEMP)), 39);
    }
}
