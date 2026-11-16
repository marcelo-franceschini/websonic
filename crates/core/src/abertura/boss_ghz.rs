//! Nave e bola do chefe da Green Hill Zone (REV01).
use super::boot::rom_b;
use super::ram::*;
use super::{ErroAbertura, fisica, objetos, rb, rl, rw, wb, wl, ww};
use crate::memoria::Memoria;

const MAP_EGGMAN: u32 = 0x1_84B8;
const ANI_EGGMAN: u32 = 0x1_8468;
const MAP_BOSS_ITEMS: u32 = 0x1_8580;
const MAP_SWING: u32 = 0x82C6;
const MAP_BALL: u32 = 0x8828;
const BOSSSTATUS: u32 = 0xFF_F7A7;
const PARENT: u32 = 0x34;
const BOSS_X: u32 = 0x30;
const BOSS_Y: u32 = 0x38;
const TIMER: u32 = 0x3C;
const FLASH: u32 = 0x3E;

fn next_free(m: &mut Memoria, a: u32) -> Option<u32> {
    let mut slot = a.wrapping_add(OBJECT_SIZE);
    while slot < V_LVLOBJEND {
        if rb(m, slot.wrapping_add(OB_ID)) == 0 {
            return Some(slot);
        }
        slot = slot.wrapping_add(OBJECT_SIZE);
    }
    None
}
fn word_signed(v: u16) -> i16 {
    i16::from_be_bytes(v.to_be_bytes())
}
fn as_word(v: i16) -> u16 {
    u16::from_be_bytes(v.to_be_bytes())
}
fn signed_long(v: i32) -> u32 {
    u32::from_be_bytes(v.to_be_bytes())
}
fn boss_move(m: &mut Memoria, a: u32) {
    for (pos, vel) in [(BOSS_X, OB_VELX), (BOSS_Y, OB_VELY)] {
        let passo = i32::from(word_signed(rw(m, a.wrapping_add(vel)))).wrapping_shl(8);
        let valor = rl(m, a.wrapping_add(pos)).wrapping_add(signed_long(passo));
        wl(m, a.wrapping_add(pos), valor);
    }
}
fn preparar(m: &mut Memoria, a: u32) {
    let x = rw(m, a.wrapping_add(OB_X));
    let y = rw(m, a.wrapping_add(OB_Y));
    let mut slot = a;
    for (rotina, anim) in [(2, 0), (4, 1), (6, 7)] {
        wb(m, slot.wrapping_add(OB_ROUTINE), rotina);
        wb(m, slot.wrapping_add(OB_ID), 0x3D);
        ww(m, slot.wrapping_add(OB_X), x);
        ww(m, slot.wrapping_add(OB_Y), y);
        wl(m, slot.wrapping_add(OB_MAP), MAP_EGGMAN);
        ww(m, slot.wrapping_add(OB_GFX), 0x400);
        wb(m, slot.wrapping_add(OB_RENDER), 4);
        wb(m, slot.wrapping_add(OB_ACTWID), 32);
        wb(m, slot.wrapping_add(OB_PRIORITY), 3);
        wb(m, slot.wrapping_add(OB_ANIM), anim);
        wl(m, slot.wrapping_add(PARENT), a);
        match next_free(m, slot) {
            Some(proximo) => slot = proximo,
            None => break,
        }
    }
    ww(m, a.wrapping_add(BOSS_X), x);
    ww(m, a.wrapping_add(BOSS_Y), y);
    wb(m, a.wrapping_add(OB_COLTYPE), 0x0F);
    wb(m, a.wrapping_add(0x21), 8);
}
pub(super) fn explosao(m: &mut Memoria, a: u32) {
    if rb(m, V_VBLANK_BYTE) & 7 != 0 {
        return;
    }
    if let Some(livre) = objetos::find_free_obj(m) {
        let x = rw(m, a.wrapping_add(OB_X));
        let y = rw(m, a.wrapping_add(OB_Y));
        let rnd = fisica::random_number(m);
        wb(m, livre.wrapping_add(OB_ID), 0x3F);
        ww(
            m,
            livre.wrapping_add(OB_X),
            x.wrapping_add((rnd & 0xFF).wrapping_shr(2).wrapping_sub(32)),
        );
        ww(
            m,
            livre.wrapping_add(OB_Y),
            y.wrapping_add(rnd.wrapping_shr(11)),
        );
    }
}
fn atualizar_nave(m: &mut Memoria, a: u32) {
    let angulo = rb(m, a.wrapping_add(0x3F));
    let (seno, _) = fisica::calc_sine(m, angulo);
    let y = rw(m, a.wrapping_add(BOSS_Y)).wrapping_add(as_word(seno >> 6));
    let x = rw(m, a.wrapping_add(BOSS_X));
    ww(m, a.wrapping_add(OB_Y), y);
    ww(m, a.wrapping_add(OB_X), x);
    wb(m, a.wrapping_add(0x3F), angulo.wrapping_add(2));
    if rb(m, a.wrapping_add(OB_ROUTINE_2)) >= 8 {
        return;
    }
    if rb(m, a.wrapping_add(OB_STATUS)) & 0x80 != 0 {
        fisica::add_points(m, 100);
        wb(m, a.wrapping_add(OB_ROUTINE_2), 8);
        ww(m, a.wrapping_add(TIMER), 0xB3);
        return;
    }
    if rb(m, a.wrapping_add(OB_COLTYPE)) != 0 {
        return;
    }
    let mut flash = rb(m, a.wrapping_add(FLASH));
    if flash == 0 {
        flash = 0x20;
        wb(m, V_SOUNDQUEUE1, 0xAC);
    }
    let cor = if rw(m, V_PALETTE.wrapping_add(0x22)) == 0 {
        0xEEE
    } else {
        0
    };
    ww(m, V_PALETTE.wrapping_add(0x22), cor);
    flash = flash.wrapping_sub(1);
    wb(m, a.wrapping_add(FLASH), flash);
    if flash == 0 {
        wb(m, a.wrapping_add(OB_COLTYPE), 0x0F);
    }
}
fn renderizar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    objetos::animate_sprite(m, a, ANI_EGGMAN)?;
    let render = (rb(m, a.wrapping_add(OB_RENDER)) & !3) | (rb(m, a.wrapping_add(OB_STATUS)) & 3);
    wb(m, a.wrapping_add(OB_RENDER), render);
    objetos::display_sprite(m, a);
    Ok(())
}
fn nave(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a.wrapping_add(OB_ROUTINE_2));
    match rotina {
        0 => {
            ww(m, a.wrapping_add(OB_VELY), 0x100);
            boss_move(m, a);
            if rw(m, a.wrapping_add(BOSS_Y)) == 0x338 {
                ww(m, a.wrapping_add(OB_VELY), 0);
                wb(m, a.wrapping_add(OB_ROUTINE_2), 2);
            }
        }
        2 => {
            ww(m, a.wrapping_add(OB_VELX), 0xFF00);
            ww(m, a.wrapping_add(OB_VELY), 0xFFC0);
            boss_move(m, a);
            if rw(m, a.wrapping_add(BOSS_X)) == 0x2A00 {
                ww(m, a.wrapping_add(OB_VELX), 0);
                ww(m, a.wrapping_add(OB_VELY), 0);
                wb(m, a.wrapping_add(OB_ROUTINE_2), 4);
                if let Some(livre) = next_free(m, a) {
                    let x = rw(m, a.wrapping_add(BOSS_X));
                    let y = rw(m, a.wrapping_add(BOSS_Y));
                    wb(m, livre.wrapping_add(OB_ID), 0x48);
                    ww(m, livre.wrapping_add(OB_X), x);
                    ww(m, livre.wrapping_add(OB_Y), y);
                    wl(m, livre.wrapping_add(PARENT), a);
                }
                ww(m, a.wrapping_add(TIMER), 119);
            }
        }
        4 => {
            let t = rw(m, a.wrapping_add(TIMER)).wrapping_sub(1);
            ww(m, a.wrapping_add(TIMER), t);
            if word_signed(t) < 0 {
                wb(m, a.wrapping_add(OB_ROUTINE_2), 6);
                let origem = rw(m, a.wrapping_add(BOSS_X)) == 0x2A00;
                ww(m, a.wrapping_add(TIMER), if origem { 127 } else { 63 });
                ww(
                    m,
                    a.wrapping_add(OB_VELX),
                    if origem { 0x40 } else { 0x100 },
                );
            }
            if rb(m, a.wrapping_add(OB_STATUS)) & 1 == 0 {
                let v = rw(m, a.wrapping_add(OB_VELX));
                ww(m, a.wrapping_add(OB_VELX), 0u16.wrapping_sub(v));
            }
        }
        6 => {
            let t = rw(m, a.wrapping_add(TIMER)).wrapping_sub(1);
            ww(m, a.wrapping_add(TIMER), t);
            if word_signed(t) < 0 {
                let status = rb(m, a.wrapping_add(OB_STATUS));
                wb(m, a.wrapping_add(OB_STATUS), status ^ 1);
                ww(m, a.wrapping_add(TIMER), 63);
                wb(m, a.wrapping_add(OB_ROUTINE_2), 4);
                ww(m, a.wrapping_add(OB_VELX), 0);
            } else {
                boss_move(m, a);
            }
        }
        8 => {
            let t = rw(m, a.wrapping_add(TIMER)).wrapping_sub(1);
            ww(m, a.wrapping_add(TIMER), t);
            if word_signed(t) >= 0 {
                explosao(m, a);
            } else {
                let status = rb(m, a.wrapping_add(OB_STATUS));
                wb(m, a.wrapping_add(OB_STATUS), (status | 1) & !0x80);
                ww(m, a.wrapping_add(OB_VELX), 0);
                wb(m, a.wrapping_add(OB_ROUTINE_2), 10);
                ww(m, a.wrapping_add(TIMER), 0u16.wrapping_sub(38));
                if rb(m, BOSSSTATUS) == 0 {
                    wb(m, BOSSSTATUS, 1);
                }
            }
            return renderizar(m, a);
        }
        10 => {
            let t = rw(m, a.wrapping_add(TIMER)).wrapping_add(1);
            ww(m, a.wrapping_add(TIMER), t);
            let vy = rw(m, a.wrapping_add(OB_VELY));
            if t == 0 {
                ww(m, a.wrapping_add(OB_VELY), 0);
            } else if word_signed(t) < 0 {
                ww(m, a.wrapping_add(OB_VELY), vy.wrapping_add(0x18));
            } else if t < 0x30 {
                ww(m, a.wrapping_add(OB_VELY), vy.wrapping_sub(8));
            } else if t == 0x30 {
                ww(m, a.wrapping_add(OB_VELY), 0);
                wb(m, V_SOUNDQUEUE0, 0x81);
            } else if t >= 0x38 {
                wb(m, a.wrapping_add(OB_ROUTINE_2), 12);
            }
            boss_move(m, a);
        }
        12 => {
            ww(m, a.wrapping_add(OB_VELX), 0x400);
            ww(m, a.wrapping_add(OB_VELY), 0xFFC0);
            let limite = rw(m, V_LIMITRIGHT2);
            if limite != 0x2AC0 {
                ww(m, V_LIMITRIGHT2, limite.wrapping_add(2));
            } else if rb(m, a.wrapping_add(OB_RENDER)) & 0x80 == 0 {
                objetos::delete_object(m, a);
                return renderizar(m, a);
            }
            boss_move(m, a);
        }
        _ => return Err(ErroAbertura::NaoImplementado("rotina da nave GHZ")),
    }
    atualizar_nave(m, a);
    renderizar(m, a)
}
fn parte(m: &mut Memoria, a: u32, chama: bool) -> Result<(), ErroAbertura> {
    let pai = rl(m, a.wrapping_add(PARENT));
    let fase = rb(m, pai.wrapping_add(OB_ROUTINE_2));
    let anim = if chama {
        if fase == 12 {
            11
        } else if rw(m, pai.wrapping_add(OB_VELX)) != 0 {
            8
        } else {
            7
        }
    } else if fase == 12 {
        6
    } else if fase >= 10 {
        10
    } else if rb(m, pai.wrapping_add(OB_COLTYPE)) == 0 {
        5
    } else if (fase == 4 && rw(m, pai.wrapping_add(BOSS_X)) == 0x2A00)
        || rb(m, V_PLAYER.wrapping_add(OB_ROUTINE)) >= 4
    {
        4
    } else {
        1
    };
    wb(m, a.wrapping_add(OB_ANIM), anim);
    if fase == 12 && rb(m, a.wrapping_add(OB_RENDER)) & 0x80 == 0 {
        objetos::delete_object(m, a);
        return Ok(());
    }
    let x = rw(m, pai.wrapping_add(OB_X));
    let y = rw(m, pai.wrapping_add(OB_Y));
    let status = rb(m, pai.wrapping_add(OB_STATUS));
    ww(m, a.wrapping_add(OB_X), x);
    ww(m, a.wrapping_add(OB_Y), y);
    wb(m, a.wrapping_add(OB_STATUS), status);
    renderizar(m, a)
}
pub fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            preparar(m, a);
            nave(m, a)
        }
        2 => nave(m, a),
        4 => parte(m, a, false),
        6 => parte(m, a, true),
        _ => Err(ErroAbertura::NaoImplementado("objeto da nave GHZ")),
    }
}

fn indice_slot(a: u32) -> u8 {
    u8::try_from(a.wrapping_sub(V_OBJSPACE).wrapping_shr(6) & 0x7F).unwrap_or(0)
}
fn iniciar_bola(m: &mut Memoria, a: u32) {
    wb(m, a.wrapping_add(OB_ROUTINE), 2);
    ww(m, a.wrapping_add(OB_ANGLE), 0x4080);
    ww(m, a.wrapping_add(FLASH), 0xFE00);
    wl(m, a.wrapping_add(OB_MAP), MAP_BOSS_ITEMS);
    ww(m, a.wrapping_add(OB_GFX), 0x46C);
    wb(m, a.wrapping_add(OB_SUBTYPE), 0);
    let pai = rl(m, a.wrapping_add(PARENT));
    let x = rw(m, a.wrapping_add(OB_X));
    let y = rw(m, a.wrapping_add(OB_Y));
    let mut slot = a;
    for i in 0..6u32 {
        if i != 0 {
            let Some(livre) = next_free(m, a) else {
                break;
            };
            slot = livre;
            ww(m, slot.wrapping_add(OB_X), x);
            ww(m, slot.wrapping_add(OB_Y), y);
            wb(m, slot.wrapping_add(OB_ID), 0x48);
            wb(m, slot.wrapping_add(OB_ROUTINE), 6);
            wl(m, slot.wrapping_add(OB_MAP), MAP_SWING);
            ww(m, slot.wrapping_add(OB_GFX), 0x380);
            wb(m, slot.wrapping_add(OB_FRAME), 1);
            wb(m, a.wrapping_add(OB_SUBTYPE), u8::try_from(i).unwrap_or(0));
        }
        wb(
            m,
            a.wrapping_add(OB_SUBTYPE).wrapping_add(1).wrapping_add(i),
            indice_slot(slot),
        );
        wb(m, slot.wrapping_add(OB_RENDER), 4);
        wb(m, slot.wrapping_add(OB_ACTWID), 8);
        wb(m, slot.wrapping_add(OB_PRIORITY), 6);
        wl(m, slot.wrapping_add(PARENT), pai);
    }
    wb(m, slot.wrapping_add(OB_ROUTINE), 8);
    wl(m, slot.wrapping_add(OB_MAP), MAP_BALL);
    ww(m, slot.wrapping_add(OB_GFX), 0x43AA);
    wb(m, slot.wrapping_add(OB_FRAME), 1);
    wb(m, slot.wrapping_add(OB_PRIORITY), 5);
    wb(m, slot.wrapping_add(OB_COLTYPE), 0x81);
}
fn mover_corrente(m: &mut Memoria, a: u32) {
    let angulo = rb(m, a.wrapping_add(OB_ANGLE));
    let (seno, cosseno) = fisica::calc_sine(m, angulo);
    let y = rw(m, a.wrapping_add(0x38));
    let x = rw(m, a.wrapping_add(0x3A));
    let n = u32::from(rb(m, a.wrapping_add(OB_SUBTYPE)));
    for i in 0..=n {
        let idx = rb(
            m,
            a.wrapping_add(OB_SUBTYPE).wrapping_add(1).wrapping_add(i),
        );
        let slot = V_OBJSPACE.wrapping_add(u32::from(idx).wrapping_mul(OBJECT_SIZE));
        let raio = i32::from(rb(m, slot.wrapping_add(0x3C)));
        let dy = raio.wrapping_mul(i32::from(seno)) >> 8;
        let dx = raio.wrapping_mul(i32::from(cosseno)) >> 8;
        let dy = u16::try_from(signed_long(dy) & 0xFFFF).unwrap_or(0);
        let dx = u16::try_from(signed_long(dx) & 0xFFFF).unwrap_or(0);
        ww(m, slot.wrapping_add(OB_Y), y.wrapping_add(dy));
        ww(m, slot.wrapping_add(OB_X), x.wrapping_add(dx));
    }
}
fn base_bola(m: &mut Memoria, a: u32, balancar: bool) {
    if !balancar {
        let n = u32::from(rb(m, a.wrapping_add(OB_SUBTYPE)));
        let mut completo = false;
        for i in 0..=n {
            let idx = rb(
                m,
                a.wrapping_add(OB_SUBTYPE).wrapping_add(1).wrapping_add(i),
            );
            let slot = V_OBJSPACE.wrapping_add(u32::from(idx).wrapping_mul(OBJECT_SIZE));
            let alvo = rom_b(m, 0x1_835E_u32.wrapping_add(i));
            let atual = rb(m, slot.wrapping_add(0x3C));
            if atual != alvo {
                wb(m, slot.wrapping_add(0x3C), atual.wrapping_add(1));
            }
            completo = atual == alvo;
        }
        let pai = rl(m, a.wrapping_add(PARENT));
        if completo && rb(m, pai.wrapping_add(OB_ROUTINE_2)) == 6 {
            wb(m, a.wrapping_add(OB_ROUTINE), 4);
        }
        let ancora = rw(m, a.wrapping_add(0x32));
        if ancora != 32 {
            ww(m, a.wrapping_add(0x32), ancora.wrapping_add(1));
        }
    }
    let pai = rl(m, a.wrapping_add(PARENT));
    let quadro = rb(m, a.wrapping_add(OB_ANI_FRAME));
    let novo = quadro.wrapping_add(32);
    wb(m, a.wrapping_add(OB_ANI_FRAME), novo);
    if novo < quadro {
        let frame = rb(m, a.wrapping_add(OB_FRAME));
        wb(m, a.wrapping_add(OB_FRAME), frame ^ 1);
    }
    let x = rw(m, pai.wrapping_add(OB_X));
    let y = rw(m, pai.wrapping_add(OB_Y)).wrapping_add(rw(m, a.wrapping_add(0x32)));
    let status = rb(m, pai.wrapping_add(OB_STATUS));
    ww(m, a.wrapping_add(0x3A), x);
    ww(m, a.wrapping_add(0x38), y);
    wb(m, a.wrapping_add(OB_STATUS), status);
    if status & 0x80 != 0 {
        wb(m, a.wrapping_add(OB_ID), 0x3F);
        wb(m, a.wrapping_add(OB_ROUTINE), 0);
    }
    if balancar {
        let direcao = rb(m, a.wrapping_add(0x3D));
        let velocidade = rw(m, a.wrapping_add(0x3E));
        let velocidade = if direcao == 0 {
            velocidade.wrapping_add(8)
        } else {
            velocidade.wrapping_sub(8)
        };
        ww(m, a.wrapping_add(0x3E), velocidade);
        let angulo = rw(m, a.wrapping_add(OB_ANGLE)).wrapping_add(velocidade);
        ww(m, a.wrapping_add(OB_ANGLE), angulo);
        if velocidade == 0x200 {
            wb(m, a.wrapping_add(0x3D), 1);
        } else if velocidade == 0xFE00 {
            wb(m, a.wrapping_add(0x3D), 0);
        }
    }
    mover_corrente(m, a);
    objetos::display_sprite(m, a);
}
pub fn bola(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => iniciar_bola(m, a),
        2 => base_bola(m, a, false),
        4 => base_bola(m, a, true),
        6 | 8 => {
            let bola = rb(m, a.wrapping_add(OB_ROUTINE)) == 8;
            if bola {
                let frame = rb(m, a.wrapping_add(OB_FRAME));
                wb(m, a.wrapping_add(OB_FRAME), if frame == 0 { 1 } else { 0 });
            }
            let pai = rl(m, a.wrapping_add(PARENT));
            if rb(m, pai.wrapping_add(OB_STATUS)) & 0x80 != 0 {
                if bola {
                    wb(m, a.wrapping_add(OB_COLTYPE), 0);
                    explosao(m, a);
                    let tempo = rb(m, a.wrapping_add(TIMER)).wrapping_sub(1);
                    wb(m, a.wrapping_add(TIMER), tempo);
                    if tempo & 0x80 != 0 {
                        wb(m, a.wrapping_add(OB_ID), 0x3F);
                        wb(m, a.wrapping_add(OB_ROUTINE), 0);
                    }
                } else {
                    wb(m, a.wrapping_add(OB_ID), 0x3F);
                    wb(m, a.wrapping_add(OB_ROUTINE), 0);
                }
            }
            objetos::display_sprite(m, a);
        }
        _ => return Err(ErroAbertura::NaoImplementado("rotina da bola GHZ")),
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    #[ignore = "requer SONIC1_ROM com a ROM REV01"]
    fn ciclo_da_luta_com_rom_real() {
        let caminho = std::env::var_os("SONIC1_ROM").expect("SONIC1_ROM");
        let rom = std::fs::read(caminho).expect("leitura da ROM");
        let mut m = Memoria::nova(rom);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x3D);
        ww(&mut m, a.wrapping_add(OB_X), 0x2A60);
        ww(&mut m, a.wrapping_add(OB_Y), 0x280);
        ww(&mut m, V_LIMITRIGHT2, 0x2960);
        for quadro in 0..420u16 {
            wb(&mut m, V_VBLANK_BYTE, quadro.to_be_bytes()[1]);
            objetos::execute_objects(&mut m).unwrap();
            objetos::build_sprites(&mut m).unwrap();
        }
        assert!(rb(&mut m, a.wrapping_add(OB_ROUTINE_2)) >= 4);
        let estado = rb(&mut m, a.wrapping_add(OB_STATUS));
        wb(&mut m, a.wrapping_add(OB_STATUS), estado | 0x80);
        for quadro in 0..300u16 {
            wb(&mut m, V_VBLANK_BYTE, quadro.to_be_bytes()[1]);
            objetos::execute_objects(&mut m).unwrap();
            objetos::build_sprites(&mut m).unwrap();
        }
        assert_eq!(rb(&mut m, BOSSSTATUS), 1);
        assert!(rw(&mut m, V_LIMITRIGHT2) > 0x2960);
    }

    #[test]
    fn inicializa_tres_partes_e_desce() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x3D);
        ww(&mut m, a.wrapping_add(OB_X), 0x2A60);
        ww(&mut m, a.wrapping_add(OB_Y), 0x280);
        executar(&mut m, a).unwrap();
        assert_eq!(rw(&mut m, a.wrapping_add(BOSS_Y)), 0x281);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_COLTYPE)), 0x0F);
        for (indice, rotina) in [(1u32, 4), (2, 6)] {
            let parte = a.wrapping_add(OBJECT_SIZE.wrapping_mul(indice));
            assert_eq!(rb(&mut m, parte.wrapping_add(OB_ID)), 0x3D);
            assert_eq!(rb(&mut m, parte.wrapping_add(OB_ROUTINE)), rotina);
            assert_eq!(rl(&mut m, parte.wrapping_add(PARENT)), a);
        }
    }

    #[test]
    fn cria_bola_ao_chegar_a_posicao_de_luta() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x3D);
        wb(&mut m, a.wrapping_add(OB_ROUTINE), 2);
        wb(&mut m, a.wrapping_add(OB_ROUTINE_2), 2);
        ww(&mut m, a.wrapping_add(BOSS_X), 0x2A01);
        ww(&mut m, a.wrapping_add(BOSS_Y), 0x338);
        ww(&mut m, a.wrapping_add(OB_COLTYPE), 0x0F00);
        nave(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE_2)), 4);
        assert_eq!(rw(&mut m, a.wrapping_add(TIMER)), 119);
        let bola = a.wrapping_add(OBJECT_SIZE);
        assert_eq!(rb(&mut m, bola.wrapping_add(OB_ID)), 0x48);
        assert_eq!(rl(&mut m, bola.wrapping_add(PARENT)), a);
        iniciar_bola(&mut m, bola);
        assert_eq!(rb(&mut m, bola.wrapping_add(OB_SUBTYPE)), 5);
        let ponta = bola.wrapping_add(OBJECT_SIZE.wrapping_mul(5));
        assert_eq!(rb(&mut m, ponta.wrapping_add(OB_ROUTINE)), 8);
        assert_eq!(rb(&mut m, ponta.wrapping_add(OB_COLTYPE)), 0x81);
    }

    #[test]
    fn derrota_inicia_explosoes_e_escape_abre_a_camera() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_STATUS), 0x80);
        atualizar_nave(&mut m, a);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ROUTINE_2)), 8);
        assert_eq!(rw(&mut m, a.wrapping_add(TIMER)), 0xB3);
        wb(&mut m, a.wrapping_add(OB_ROUTINE_2), 12);
        ww(&mut m, V_LIMITRIGHT2, 0x2960);
        nave(&mut m, a).unwrap();
        assert_eq!(rw(&mut m, V_LIMITRIGHT2), 0x2962);
    }
}
