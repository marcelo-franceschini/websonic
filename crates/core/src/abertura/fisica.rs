//! O primeiro quadro do Sonic na demo (`_incObj/01 Sonic.asm`, `Sonic AnglePos.asm`,
//! `sub FindNearestTile & FindFloor & FindWall.asm`). O `ExecuteObjects` de `Level_LoadObj` já roda
//! o objeto do Sonic uma vez, com os botões zerados, antes de `Level_StartGame`. Aqui existe só o
//! que esse quadro usa: o modo no chão (`Sonic_MdNormal`) com o ângulo de piso, a animação, os
//! gráficos e as checagens que dependem de objetos ou de botões devolvem `NaoImplementado`. A
//! física completa é da Etapa 9.

use super::boot::{rom_b, rom_l, rom_w};
use super::objetos::display_sprite;
use super::ram::*;
use super::{ErroAbertura, rb, rl, rw, wb, wl, ww};
use crate::memoria::Memoria;

/// Gravidade e velocidades do Sonic (`son_*`).
const SON_MAXSPEED: u16 = 0x600;
const SON_ACCELERATION: u16 = 0xC;
const SON_DECELERATION: u16 = 0x80;
const SONIC_HEIGHT: u8 = 19;
const SONIC_WIDTH: u8 = 9;
const SONIC_ROLL_HEIGHT: u8 = 14;
const SONIC_ROLL_WIDTH: u8 = 7;
const SFX_ROLL: u8 = 0xBE;

/// Bits de `obStatus` e de `obRender`.
const RENDER_FLIPS: u8 = 3;
const RENDER_LOOPING: u8 = 0x40;
/// `sprite_cam_field`: posição pelo campo de jogo.
const RENDER_CAM_FIELD: u8 = 4;

fn s16(w: u16) -> i16 {
    i16::from_be_bytes(w.to_be_bytes())
}

fn u16_de(i: i16) -> u16 {
    u16::from_be_bytes(i.to_be_bytes())
}

/// `ext.w`: o byte com sinal vira palavra.
fn ext_w(b: u8) -> u16 {
    u16_de(i16::from(i8::from_be_bytes([b])))
}

fn bit(v: u16, n: u32) -> bool {
    v.checked_shr(n).unwrap_or(0) & 1 != 0
}

/// `CalcSine`: devolve `(seno, cosseno)` do ângulo (`d0`, `d1`), lidos da tabela da ROM.
pub(crate) fn calc_sine(m: &Memoria, angulo: u8) -> (i16, i16) {
    let d0 = u32::from(angulo).wrapping_mul(2);
    let seno = s16(rom_w(m, SINE_DATA.wrapping_add(d0)));
    let cosseno = s16(rom_w(m, SINE_DATA.wrapping_add(d0).wrapping_add(0x80)));
    (seno, cosseno)
}

/// `muls.w` e `asr.l #8`: o produto de duas palavras com sinal, deslocado, na palavra baixa.
fn muls_asr8(a: i16, b: i16) -> i16 {
    let p = i32::from(a).wrapping_mul(i32::from(b)) >> 8;
    s16(u16::try_from(p & 0xFFFF).unwrap_or(0))
}

// ---------------------------------------------------------------------------------------------
// Colisão de piso
// ---------------------------------------------------------------------------------------------

/// `FindNearestTile` (FixBugs = 0): o endereço da palavra do bloco em `(d3, d2)`. Um chunk vazio
/// devolve `$FFFFFF00`, e o original lê a RAM ali mesmo.
fn find_nearest_tile(m: &mut Memoria, a0: u32, d2: u16, d3: u16) -> u32 {
    let linha = (d2 >> 1) & 0x380;
    let coluna = (d3 >> 8) & 0x7F;
    let indice = u32::from(linha.wrapping_add(coluna));
    let chunk = rb(m, V_LVLLAYOUT_FG.wrapping_add(indice));
    if chunk == 0 {
        return 0xFF_FF00;
    }
    let mut d1 = chunk;
    if chunk & 0x80 != 0 {
        // Chunk especial (laços): o 29 vira o 51 quando o sprite está em laço.
        let mut v = chunk & 0x7F;
        if rb(m, a0.wrapping_add(OB_RENDER)) & RENDER_LOOPING != 0 {
            v = v.wrapping_add(1);
            if v == 0x29 {
                v = 0x51;
            }
        }
        d1 = v;
    }
    let base = u16::from(d1.wrapping_sub(1)).rotate_right(7);
    let dy = d2.wrapping_mul(2) & 0x1E0;
    let dx = (d3 >> 3) & 0x1E;
    let posicao = base.wrapping_add(dy).wrapping_add(dx);
    0xFF_0000 | u32::from(posicao)
}

/// Parâmetros comuns de `FindFloor`: `d5` é o bit de solidez, `d6` o espelhamento e `a3` o passo.
#[derive(Clone, Copy)]
struct Busca {
    a0: u32,
    d5: u32,
    d6: u16,
    a3: u16,
    a4: u32,
}

/// O que `FindFloor` e `FindFloor2` calculam depois de achar um bloco sólido.
enum Piso {
    /// O bloco está vazio ou sem colisão.
    Vazio,
    /// Distância dentro do bloco (`d1`).
    Dist(u16),
    /// `.negfloor` em `FindFloor2` com a soma ainda negativa.
    Negativo(u16),
    /// `.negfloor` com a soma não negativa.
    NegativoVazio,
    /// Altura máxima no bloco ($10): olha o bloco vizinho.
    Maximo,
}

/// A parte comum de `FindFloor` e `FindFloor2` depois de achar um bloco com colisão: a altura
/// da coluna em `CollArray1`.
fn altura_no_bloco(m: &mut Memoria, b: Busca, d2: u16, d3: u16, palavra: u16) -> Piso {
    let id = palavra & 0x7FF;
    if id == 0 || !bit(palavra, b.d5) {
        return Piso::Vazio;
    }
    let indice_col = rl(m, V_COLLINDEX);
    let d0 = u16::from(rb(m, indice_col.wrapping_add(u32::from(id))));
    if d0 == 0 {
        return Piso::Vazio;
    }
    let mut angulo = rom_b(m, ANGLEMAP.wrapping_add(u32::from(d0)));
    let linha = d0.wrapping_shl(4);
    let mut d1 = d3;
    if bit(palavra, 0xB) {
        d1 = !d1;
        angulo = angulo.wrapping_neg();
    }
    if bit(palavra, 0xC) {
        angulo = angulo.wrapping_add(0x40).wrapping_neg().wrapping_sub(0x40);
    }
    wb(m, b.a4, angulo);
    let coluna = (d1 & 0xF).wrapping_add(linha);
    let mut d0 = ext_w(rom_b(m, COLLARRAY1.wrapping_add(u32::from(coluna))));
    if bit(palavra ^ b.d6, 0xC) {
        d0 = d0.wrapping_neg();
    }
    if d0 == 0 {
        return Piso::Vazio;
    }
    if s16(d0) < 0 {
        let soma = d0.wrapping_add(d2 & 0xF);
        return if s16(soma) >= 0 {
            Piso::NegativoVazio
        } else {
            Piso::Negativo(d2 & 0xF)
        };
    }
    if d0 & 0xFF == 0x10 {
        return Piso::Maximo;
    }
    let soma = d0.wrapping_add(d2 & 0xF);
    Piso::Dist(0xFu16.wrapping_sub(soma))
}

/// `FindFloor2`: o bloco vizinho, sem recursão.
fn find_floor2(m: &mut Memoria, b: Busca, d2: u16, d3: u16) -> u16 {
    let a1 = find_nearest_tile(m, b.a0, d2, d3);
    let palavra = rw(m, a1);
    match altura_no_bloco(m, b, d2, d3, palavra) {
        Piso::Dist(d1) => d1,
        Piso::Negativo(resto) => !resto,
        Piso::Vazio | Piso::NegativoVazio | Piso::Maximo => 0xFu16.wrapping_sub(d2 & 0xF),
    }
}

/// `FindFloor`: a distância de `(d3, d2)` até o piso, com o ângulo em `(a4)`.
fn find_floor(m: &mut Memoria, b: Busca, d2: u16, d3: u16) -> u16 {
    let a1 = find_nearest_tile(m, b.a0, d2, d3);
    let palavra = rw(m, a1);
    match altura_no_bloco(m, b, d2, d3, palavra) {
        Piso::Dist(d1) => d1,
        Piso::Maximo | Piso::Negativo(_) => {
            let d1 = find_floor2(m, b, d2.wrapping_sub(b.a3), d3);
            d1.wrapping_sub(0x10)
        }
        Piso::Vazio | Piso::NegativoVazio => {
            let d1 = find_floor2(m, b, d2.wrapping_add(b.a3), d3);
            d1.wrapping_add(0x10)
        }
    }
}

/// `FindWall` usa a tabela horizontal e avança na coluna, em vez da linha.
fn largura_no_bloco(m: &mut Memoria, b: Busca, y: u16, x: u16, palavra: u16) -> Piso {
    let id = palavra & 0x7ff;
    if id == 0 || !bit(palavra, b.d5) {
        return Piso::Vazio;
    }
    let indice = rl(m, V_COLLINDEX);
    let forma = u16::from(rb(m, indice.wrapping_add(u32::from(id))));
    if forma == 0 {
        return Piso::Vazio;
    }
    let mut angulo = rom_b(m, ANGLEMAP.wrapping_add(u32::from(forma)));
    let mut linha = y;
    if bit(palavra, 0xC) {
        linha = !linha;
        angulo = angulo.wrapping_add(0x40).wrapping_neg().wrapping_sub(0x40);
    }
    if bit(palavra, 0xB) {
        angulo = angulo.wrapping_neg();
    }
    wb(m, b.a4, angulo);
    let deslocamento = forma.wrapping_shl(4).wrapping_add(linha & 15);
    let mut largura = ext_w(rom_b(m, COLLARRAY2.wrapping_add(u32::from(deslocamento))));
    if bit(palavra ^ b.d6, 0xB) {
        largura = largura.wrapping_neg();
    }
    if largura == 0 {
        return Piso::Vazio;
    }
    if s16(largura) < 0 {
        return if s16(largura.wrapping_add(x & 15)) >= 0 {
            Piso::NegativoVazio
        } else {
            Piso::Negativo(x & 15)
        };
    }
    if largura == 16 {
        return Piso::Maximo;
    }
    Piso::Dist(15u16.wrapping_sub(largura.wrapping_add(x & 15)))
}

fn find_wall2(m: &mut Memoria, b: Busca, y: u16, x: u16) -> u16 {
    let a1 = find_nearest_tile(m, b.a0, y, x);
    let palavra = rw(m, a1);
    match largura_no_bloco(m, b, y, x, palavra) {
        Piso::Dist(d) => d,
        Piso::Negativo(resto) => !resto,
        Piso::Maximo => 15u16.wrapping_sub(16u16.wrapping_add(x & 15)),
        Piso::Vazio | Piso::NegativoVazio => 15u16.wrapping_sub(x & 15),
    }
}

fn find_wall(m: &mut Memoria, b: Busca, y: u16, x: u16) -> u16 {
    let a1 = find_nearest_tile(m, b.a0, y, x);
    let palavra = rw(m, a1);
    match largura_no_bloco(m, b, y, x, palavra) {
        Piso::Dist(d) => d,
        Piso::Maximo | Piso::Negativo(_) => {
            find_wall2(m, b, y, x.wrapping_sub(b.a3)).wrapping_sub(16)
        }
        Piso::Vazio | Piso::NegativoVazio => {
            find_wall2(m, b, y, x.wrapping_add(b.a3)).wrapping_add(16)
        }
    }
}

/// `ObjFloorDist_And_Angle`: a distância do pé do objeto ao piso (`d1`) e o ângulo (`d3`).
pub(super) fn obj_floor_dist_and_angle(m: &mut Memoria, a0: u32, d3: u16) -> (i16, u8) {
    let altura = ext_w(rb(m, a0.wrapping_add(OB_HEIGHT)));
    let d2 = rw(m, a0.wrapping_add(OB_Y)).wrapping_add(altura);
    wb(m, V_ANGLEBUFFER, 0);
    let b = Busca {
        a0,
        d5: 0xD,
        d6: 0,
        a3: 0x10,
        a4: V_ANGLEBUFFER,
    };
    let d1 = find_floor(m, b, d2, d3);
    let mut angulo = rb(m, V_ANGLEBUFFER);
    if angulo & 1 != 0 {
        angulo = 0;
    }
    (s16(d1), angulo)
}

/// `ObjFloorDist`: a distância do pé do objeto ao piso (`d1`).
pub(super) fn obj_floor_dist(m: &mut Memoria, a0: u32) -> u16 {
    let x = rw(m, a0.wrapping_add(OB_X));
    let (d1, _) = obj_floor_dist_and_angle(m, a0, x);
    u16_de(d1)
}

// ---------------------------------------------------------------------------------------------
// O modo no chão
// ---------------------------------------------------------------------------------------------

/// `Sonic_Angle`: escolhe o sensor mais próximo do piso e guarda o ângulo.
fn sonic_angle(m: &mut Memoria, a0: u32, d0: u16, d1: u16) -> u16 {
    let (angulo, d1) = if s16(d1) <= s16(d0) {
        (rb(m, V_ANGLEBUFFER2), d1)
    } else {
        (rb(m, V_ANGLEBUFFER), d0)
    };
    if angulo & 1 == 0 {
        wb(m, a0.wrapping_add(OB_ANGLE), angulo);
    } else {
        let atual = rb(m, a0.wrapping_add(OB_ANGLE));
        wb(
            m,
            a0.wrapping_add(OB_ANGLE),
            atual.wrapping_add(0x20) & 0xC0,
        );
    }
    d1
}

/// `Sonic_AnglePos`: gruda o Sonic no piso, paredes ou teto.
fn sonic_angle_pos(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let status = rb(m, a0.wrapping_add(OB_STATUS));
    if status & STATUS_ON_OBJECT != 0 {
        wb(m, V_ANGLEBUFFER, 0);
        wb(m, V_ANGLEBUFFER2, 0);
        return Ok(());
    }
    wb(m, V_ANGLEBUFFER, 3);
    wb(m, V_ANGLEBUFFER2, 3);
    let angulo = rb(m, a0.wrapping_add(OB_ANGLE));
    let mut d0 = angulo.wrapping_add(0x20);
    if d0 & 0x80 == 0 {
        // Piso ou lado esquerdo.
        let mut a = angulo;
        if a & 0x80 != 0 {
            a = a.wrapping_add(1);
        }
        d0 = a.wrapping_add(0x1F);
    } else {
        // Ângulo negativo: `subq.b #1`; positivo pula o `subq`.
        let a = if angulo & 0x80 != 0 {
            angulo.wrapping_sub(1)
        } else {
            angulo
        };
        d0 = a.wrapping_add(0x20);
    }
    let modo = d0 & 0xC0;
    match modo {
        0x00 => sonic_walk_floor(m, a0),
        0x40 => sonic_walk_vert_l(m, a0),
        0x80 => sonic_walk_ceiling(m, a0),
        0xC0 => sonic_walk_vert_r(m, a0),
        _ => unreachable!(),
    }
}

fn sonic_walk_floor(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let altura = ext_w(rb(m, a0.wrapping_add(OB_HEIGHT)));
    let largura = ext_w(rb(m, a0.wrapping_add(OB_WIDTH)));
    let y = rw(m, a0.wrapping_add(OB_Y));
    let x = rw(m, a0.wrapping_add(OB_X));
    let b1 = Busca {
        a0,
        d5: 0xD,
        d6: 0,
        a3: 0x10,
        a4: V_ANGLEBUFFER,
    };
    let primeiro = find_floor(m, b1, y.wrapping_add(altura), x.wrapping_add(largura));
    let b2 = Busca {
        a4: V_ANGLEBUFFER2,
        ..b1
    };
    let segundo = find_floor(
        m,
        b2,
        y.wrapping_add(altura),
        x.wrapping_add(largura.wrapping_neg()),
    );
    let d1 = sonic_angle(m, a0, primeiro, segundo);
    if d1 == 0 {
        return Ok(());
    }
    let distancia = s16(d1);
    if distancia < 0 {
        // Dentro do chão: até 14 pixels se corrigem; além disso o original não faz nada.
        if distancia >= -0xE {
            ww(m, a0.wrapping_add(OB_Y), y.wrapping_add(d1));
        }
        return Ok(());
    }
    if distancia <= 0xE || rb(m, a0.wrapping_add(SON_STICKTOCONVEX)) != 0 {
        ww(m, a0.wrapping_add(OB_Y), y.wrapping_add(d1));
        return Ok(());
    }
    let s = rb(m, a0.wrapping_add(OB_STATUS));
    wb(
        m,
        a0.wrapping_add(OB_STATUS),
        (s | STATUS_IN_AIR) & !STATUS_PUSHING,
    );
    wb(m, a0.wrapping_add(OB_PREV_ANI), ANIM_RUN);
    Ok(())
}

fn sonic_walk_vert_r(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let altura = ext_w(rb(m, a0.wrapping_add(OB_HEIGHT)));
    let largura = ext_w(rb(m, a0.wrapping_add(OB_WIDTH)));
    let y = rw(m, a0.wrapping_add(OB_Y));
    let x = rw(m, a0.wrapping_add(OB_X));
    let b1 = Busca {
        a0,
        d5: 0xD,
        d6: 0,
        a3: 0x10,
        a4: V_ANGLEBUFFER,
    };
    let primeiro = find_wall(m, b1, y.wrapping_sub(largura), x.wrapping_add(altura));
    let b2 = Busca {
        a4: V_ANGLEBUFFER2,
        ..b1
    };
    let segundo = find_wall(m, b2, y.wrapping_add(largura), x.wrapping_add(altura));
    let d1 = sonic_angle(m, a0, primeiro, segundo);
    if d1 == 0 {
        return Ok(());
    }
    let distancia = s16(d1);
    if distancia < 0 {
        if distancia >= -0xE {
            ww(m, a0.wrapping_add(OB_X), x.wrapping_add(d1));
        }
        return Ok(());
    }
    if distancia <= 0xE || rb(m, a0.wrapping_add(SON_STICKTOCONVEX)) != 0 {
        ww(m, a0.wrapping_add(OB_X), x.wrapping_add(d1));
        return Ok(());
    }
    let s = rb(m, a0.wrapping_add(OB_STATUS));
    wb(
        m,
        a0.wrapping_add(OB_STATUS),
        (s | STATUS_IN_AIR) & !STATUS_PUSHING,
    );
    wb(m, a0.wrapping_add(OB_PREV_ANI), ANIM_RUN);
    Ok(())
}

fn sonic_walk_ceiling(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let altura = ext_w(rb(m, a0.wrapping_add(OB_HEIGHT)));
    let largura = ext_w(rb(m, a0.wrapping_add(OB_WIDTH)));
    let y = rw(m, a0.wrapping_add(OB_Y));
    let x = rw(m, a0.wrapping_add(OB_X));
    let b1 = Busca {
        a0,
        d5: 0xD,
        d6: 0x1000,
        a3: 0xFFF0,
        a4: V_ANGLEBUFFER,
    };
    let d2 = y.wrapping_sub(altura) ^ 0xF;
    let primeiro = find_floor(m, b1, d2, x.wrapping_add(largura));
    let b2 = Busca {
        a4: V_ANGLEBUFFER2,
        ..b1
    };
    let segundo = find_floor(m, b2, d2, x.wrapping_sub(largura));
    let d1 = sonic_angle(m, a0, primeiro, segundo);
    if d1 == 0 {
        return Ok(());
    }
    let distancia = s16(d1);
    if distancia < 0 {
        if distancia >= -0xE {
            ww(m, a0.wrapping_add(OB_Y), y.wrapping_sub(d1));
        }
        return Ok(());
    }
    if distancia <= 0xE || rb(m, a0.wrapping_add(SON_STICKTOCONVEX)) != 0 {
        ww(m, a0.wrapping_add(OB_Y), y.wrapping_sub(d1));
        return Ok(());
    }
    let s = rb(m, a0.wrapping_add(OB_STATUS));
    wb(
        m,
        a0.wrapping_add(OB_STATUS),
        (s | STATUS_IN_AIR) & !STATUS_PUSHING,
    );
    wb(m, a0.wrapping_add(OB_PREV_ANI), ANIM_RUN);
    Ok(())
}

fn sonic_walk_vert_l(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let altura = ext_w(rb(m, a0.wrapping_add(OB_HEIGHT)));
    let largura = ext_w(rb(m, a0.wrapping_add(OB_WIDTH)));
    let y = rw(m, a0.wrapping_add(OB_Y));
    let x = rw(m, a0.wrapping_add(OB_X));
    let b1 = Busca {
        a0,
        d5: 0xD,
        d6: 0x800,
        a3: 0xFFF0,
        a4: V_ANGLEBUFFER,
    };
    let d3 = x.wrapping_sub(altura) ^ 0xF;
    let primeiro = find_wall(m, b1, y.wrapping_sub(largura), d3);
    let b2 = Busca {
        a4: V_ANGLEBUFFER2,
        ..b1
    };
    let segundo = find_wall(m, b2, y.wrapping_add(largura), d3);
    let d1 = sonic_angle(m, a0, primeiro, segundo);
    if d1 == 0 {
        return Ok(());
    }
    let distancia = s16(d1);
    if distancia < 0 {
        if distancia >= -0xE {
            ww(m, a0.wrapping_add(OB_X), x.wrapping_sub(d1));
        }
        return Ok(());
    }
    if distancia <= 0xE || rb(m, a0.wrapping_add(SON_STICKTOCONVEX)) != 0 {
        ww(m, a0.wrapping_add(OB_X), x.wrapping_sub(d1));
        return Ok(());
    }
    let s = rb(m, a0.wrapping_add(OB_STATUS));
    wb(
        m,
        a0.wrapping_add(OB_STATUS),
        (s | STATUS_IN_AIR) & !STATUS_PUSHING,
    );
    wb(m, a0.wrapping_add(OB_PREV_ANI), ANIM_RUN);
    Ok(())
}

/// `Sonic_Jump`: devolve verdadeiro se o pulo fez o laço do chão retornar mais cedo.
fn sonic_jump(m: &mut Memoria, a0: u32) -> Result<bool, ErroAbertura> {
    if rb(m, V_JPADPRESS2) & 0x70 == 0 {
        return Ok(false);
    }
    let angulo = rb(m, a0.wrapping_add(OB_ANGLE));
    let orientacao = angulo.wrapping_add(0xA0) & 0xC0;
    wb(m, V_ANGLEBUFFER, angulo.wrapping_add(0x80));
    wb(m, V_ANGLEBUFFER2, angulo.wrapping_add(0x80));
    let x = rw(m, a0.wrapping_add(OB_X));
    let y = rw(m, a0.wrapping_add(OB_Y));
    let altura = ext_w(rb(m, a0.wrapping_add(OB_HEIGHT)));
    let largura = ext_w(rb(m, a0.wrapping_add(OB_WIDTH)));
    let (direita, esquerda) = match orientacao {
        0x40 | 0xC0 => {
            let esquerda = orientacao == 0x40;
            let sensor_x = if esquerda {
                x.wrapping_sub(altura) ^ 15
            } else {
                x.wrapping_add(altura)
            };
            let mut b = Busca {
                a0,
                d5: 14,
                d6: if esquerda { 0x800 } else { 0 },
                a3: if esquerda { 0xFFF0 } else { 0x10 },
                a4: V_ANGLEBUFFER,
            };
            let primeiro = find_wall(m, b, y.wrapping_sub(largura), sensor_x);
            b.a4 = V_ANGLEBUFFER2;
            let segundo = find_wall(m, b, y.wrapping_add(largura), sensor_x);
            (primeiro, segundo)
        }
        _ => {
            let teto = orientacao == 0x80;
            let sensor_y = if teto {
                y.wrapping_sub(altura) ^ 15
            } else {
                y.wrapping_add(altura)
            };
            let mut b = Busca {
                a0,
                d5: if teto { 14 } else { 13 },
                d6: if teto { 0x1000 } else { 0 },
                a3: if teto { 0xFFF0 } else { 0x10 },
                a4: V_ANGLEBUFFER,
            };
            let primeiro = find_floor(m, b, sensor_y, x.wrapping_add(largura));
            b.a4 = V_ANGLEBUFFER2;
            let segundo = find_floor(m, b, sensor_y, x.wrapping_sub(largura));
            (primeiro, segundo)
        }
    };
    if s16(direita).min(s16(esquerda)) < 6 {
        return Ok(false);
    }
    let status = rb(m, a0.wrapping_add(OB_STATUS));
    let forca = if status & 0x40 != 0 { 0x380 } else { 0x680 };
    let (seno, cosseno) = calc_sine(m, angulo.wrapping_sub(0x40));
    let vx = rw(m, a0.wrapping_add(OB_VELX)).wrapping_add(u16_de(muls_asr8(cosseno, forca)));
    let vy = rw(m, a0.wrapping_add(OB_VELY)).wrapping_add(u16_de(muls_asr8(seno, forca)));
    ww(m, a0.wrapping_add(OB_VELX), vx);
    ww(m, a0.wrapping_add(OB_VELY), vy);
    wb(
        m,
        a0.wrapping_add(OB_STATUS),
        (status | STATUS_IN_AIR) & !STATUS_PUSHING,
    );
    wb(m, a0.wrapping_add(SON_JUMPING), 1);
    wb(m, a0.wrapping_add(SON_STICKTOCONVEX), 0);
    wb(m, V_SOUNDQUEUE1, 0xA0);
    // REV01 sem FixBugs recoloca o tamanho normal antes de testar o estado de rolagem.
    wb(m, a0.wrapping_add(OB_HEIGHT), SONIC_HEIGHT);
    wb(m, a0.wrapping_add(OB_WIDTH), SONIC_WIDTH);
    if status & STATUS_ROLLING == 0 {
        wb(m, a0.wrapping_add(OB_HEIGHT), SONIC_ROLL_HEIGHT);
        wb(m, a0.wrapping_add(OB_WIDTH), SONIC_ROLL_WIDTH);
        wb(m, a0.wrapping_add(OB_ANIM), ANIM_ROLL);
        wb(
            m,
            a0.wrapping_add(OB_STATUS),
            (status | STATUS_IN_AIR | STATUS_ROLLING) & !STATUS_PUSHING,
        );
        ww(m, a0.wrapping_add(OB_Y), y.wrapping_add(5));
    } else {
        let status = rb(m, a0.wrapping_add(OB_STATUS));
        wb(m, a0.wrapping_add(OB_STATUS), status | 0x10);
    }
    Ok(true)
}

/// `Sonic_SlopeResistWalk`: a rampa freia ou empurra a inércia.
fn sonic_slope_resist_walk(m: &mut Memoria, a0: u32) {
    let angulo = rb(m, a0.wrapping_add(OB_ANGLE));
    if angulo.wrapping_add(0x60) >= 0xC0 {
        return;
    }
    let (seno, _) = calc_sine(m, angulo);
    let d0 = muls_asr8(seno, 0x20);
    let inercia = rw(m, a0.wrapping_add(OB_INERTIA));
    if inercia == 0 {
        return;
    }
    if s16(inercia) < 0 || d0 != 0 {
        ww(
            m,
            a0.wrapping_add(OB_INERTIA),
            inercia.wrapping_add(u16_de(d0)),
        );
    }
}

/// `Sonic_Move`: entrada horizontal, animação parada e rolagem do olhar.
fn sonic_move(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let d5 = rw(m, V_SONSPEEDACC);
    if rb(m, F_SLIDEMODE) != 0 {
        return sonic_angle_speed(m, a0);
    }
    if rw(m, a0.wrapping_add(SON_LOCKTIME)) != 0 {
        return sonic_reset_scr(m, a0, d5);
    }
    let botoes = rb(m, V_JPADHOLD2);
    if botoes & 4 != 0 {
        sonic_move_horizontal(m, a0, false);
    }
    if botoes & 8 != 0 {
        sonic_move_horizontal(m, a0, true);
    }
    let angulo = rb(m, a0.wrapping_add(OB_ANGLE));
    if angulo.wrapping_add(0x20) & 0xC0 != 0 || rw(m, a0.wrapping_add(OB_INERTIA)) != 0 {
        return sonic_reset_scr(m, a0, d5);
    }
    let status = rb(m, a0.wrapping_add(OB_STATUS)) & !STATUS_PUSHING;
    wb(m, a0.wrapping_add(OB_STATUS), status);
    wb(m, a0.wrapping_add(OB_ANIM), ANIM_WAIT);
    if status & STATUS_ON_OBJECT != 0 {
        let indice = u32::from(rb(m, a0.wrapping_add(SON_STANDONOBJECT)));
        let objeto = V_OBJSPACE.wrapping_add(indice.wrapping_mul(OBJECT_SIZE));
        if rb(m, objeto.wrapping_add(OB_STATUS)) & 0x80 == 0 {
            let largura = u16::from(rb(m, objeto.wrapping_add(OB_ACTWID)));
            let posicao = largura
                .wrapping_add(rw(m, a0.wrapping_add(OB_X)))
                .wrapping_sub(rw(m, objeto.wrapping_add(OB_X)));
            let limite = largura.wrapping_mul(2).wrapping_sub(4);
            let lado = if s16(posicao) < 4 {
                Some(true)
            } else if s16(posicao) >= s16(limite) {
                Some(false)
            } else {
                None
            };
            if let Some(esquerda) = lado {
                let s = rb(m, a0.wrapping_add(OB_STATUS));
                wb(
                    m,
                    a0.wrapping_add(OB_STATUS),
                    if esquerda {
                        s | STATUS_XFLIP
                    } else {
                        s & !STATUS_XFLIP
                    },
                );
                wb(m, a0.wrapping_add(OB_ANIM), ANIM_BALANCE);
                return sonic_reset_scr(m, a0, d5);
            }
        }
    } else {
        let d1 = obj_floor_dist(m, a0);
        if s16(d1) >= 12 {
            let direita = rb(m, a0.wrapping_add(SON_ANGLERIGHT));
            let esquerda = rb(m, a0.wrapping_add(SON_ANGLELEFT));
            if direita == 3 {
                let s = rb(m, a0.wrapping_add(OB_STATUS)) & !STATUS_XFLIP;
                wb(m, a0.wrapping_add(OB_STATUS), s);
                wb(m, a0.wrapping_add(OB_ANIM), ANIM_BALANCE);
                return sonic_reset_scr(m, a0, d5);
            }
            if esquerda == 3 {
                let s = rb(m, a0.wrapping_add(OB_STATUS)) | STATUS_XFLIP;
                wb(m, a0.wrapping_add(OB_STATUS), s);
                wb(m, a0.wrapping_add(OB_ANIM), ANIM_BALANCE);
                return sonic_reset_scr(m, a0, d5);
            }
        }
    }
    let olhar = rw(m, V_LOOKSHIFT);
    if botoes & 1 != 0 {
        wb(m, a0.wrapping_add(OB_ANIM), ANIM_LOOKUP);
        if olhar != 0xC8 {
            ww(m, V_LOOKSHIFT, olhar.wrapping_add(2));
        }
        return sonic_angle_speed(m, a0);
    }
    if botoes & 2 != 0 {
        wb(m, a0.wrapping_add(OB_ANIM), ANIM_DUCK);
        if olhar != 8 {
            ww(m, V_LOOKSHIFT, olhar.wrapping_sub(2));
        }
        return sonic_angle_speed(m, a0);
    }
    sonic_reset_scr(m, a0, d5)
}

/// `Sonic_MoveLeft` e `Sonic_MoveRight`, inclusive o byte alterado no teste de derrapagem.
fn sonic_move_horizontal(m: &mut Memoria, a0: u32, direita: bool) {
    let inercia = rw(m, a0.wrapping_add(OB_INERTIA));
    let velocidade = s16(inercia);
    let aceleracao = rw(m, V_SONSPEEDACC);
    let maximo = rw(m, V_SONSPEEDMAX);
    let freio = rw(m, V_SONSPEEDDEC);
    let invertendo = if direita {
        velocidade < 0
    } else {
        velocidade > 0
    };
    if !invertendo {
        let status = rb(m, a0.wrapping_add(OB_STATUS));
        let estava_virado = status & STATUS_XFLIP != 0;
        let mut novo = if direita {
            status & !STATUS_XFLIP
        } else {
            status | STATUS_XFLIP
        };
        if estava_virado == direita {
            novo &= !STATUS_PUSHING;
            wb(m, a0.wrapping_add(OB_PREV_ANI), ANIM_RUN);
        }
        wb(m, a0.wrapping_add(OB_STATUS), novo);
        let valor = if direita {
            let v = inercia.wrapping_add(aceleracao);
            if s16(v) >= s16(maximo) { maximo } else { v }
        } else {
            let v = inercia.wrapping_sub(aceleracao);
            let limite = maximo.wrapping_neg();
            if s16(v) <= s16(limite) { limite } else { v }
        };
        ww(m, a0.wrapping_add(OB_INERTIA), valor);
        wb(m, a0.wrapping_add(OB_ANIM), ANIM_WALK);
        return;
    }
    let (valor, carry) = if direita {
        inercia.overflowing_add(freio)
    } else {
        inercia.overflowing_sub(freio)
    };
    let valor = if carry {
        if direita { 0x80 } else { 0xFF80 }
    } else {
        valor
    };
    ww(m, a0.wrapping_add(OB_INERTIA), valor);
    let angulo = rb(m, a0.wrapping_add(OB_ANGLE)).wrapping_add(0x20) & 0xC0;
    let teste = (valor & 0xFF00) | u16::from(angulo);
    if angulo == 0
        && if direita {
            s16(teste) <= -0x400
        } else {
            s16(teste) >= 0x400
        }
    {
        wb(m, a0.wrapping_add(OB_ANIM), 13);
        let status = rb(m, a0.wrapping_add(OB_STATUS));
        wb(
            m,
            a0.wrapping_add(OB_STATUS),
            if direita {
                status | STATUS_XFLIP
            } else {
                status & !STATUS_XFLIP
            },
        );
        wb(m, V_SOUNDQUEUE1, 0xA4);
    }
}

/// `CalcAngle`: quadrante e razão são inteiros; a aproximação está na ROM.
fn calc_angle(m: &Memoria, x: u16, y: u16) -> u8 {
    if x == 0 && y == 0 {
        return 0x40;
    }
    let ax = u32::from(s16(x).unsigned_abs());
    let ay = u32::from(s16(y).unsigned_abs());
    let mut angulo = if ax > ay {
        rom_b(
            m,
            ANGLE_DATA.wrapping_add(ay.wrapping_mul(256).checked_div(ax).unwrap_or(0)),
        )
    } else {
        0x40u8.wrapping_sub(rom_b(
            m,
            ANGLE_DATA.wrapping_add(ax.wrapping_mul(256).checked_div(ay).unwrap_or(0)),
        ))
    };
    if s16(x) < 0 {
        angulo = 0x80u8.wrapping_sub(angulo);
    }
    if s16(y) < 0 {
        angulo = angulo.wrapping_neg();
    }
    angulo
}

fn sonic_jump_direction(m: &mut Memoria, a0: u32) {
    let status = rb(m, a0.wrapping_add(OB_STATUS));
    if status & 0x10 == 0 {
        let mut vx = rw(m, a0.wrapping_add(OB_VELX));
        let aceleracao = rw(m, V_SONSPEEDACC).wrapping_mul(2);
        let maximo = rw(m, V_SONSPEEDMAX);
        let botoes = rb(m, V_JPADHOLD2);
        let mut novo_status = status;
        if botoes & 4 != 0 {
            novo_status |= STATUS_XFLIP;
            vx = vx.wrapping_sub(aceleracao);
            let limite = maximo.wrapping_neg();
            if s16(vx) <= s16(limite) {
                vx = limite;
            }
        }
        if botoes & 8 != 0 {
            novo_status &= !STATUS_XFLIP;
            vx = vx.wrapping_add(aceleracao);
            if s16(vx) >= s16(maximo) {
                vx = maximo;
            }
        }
        wb(m, a0.wrapping_add(OB_STATUS), novo_status);
        ww(m, a0.wrapping_add(OB_VELX), vx);
    }
    let olhar = rw(m, V_LOOKSHIFT);
    if olhar != 0x60 {
        ww(
            m,
            V_LOOKSHIFT,
            if olhar < 0x60 {
                olhar.wrapping_add(2)
            } else {
                olhar.wrapping_sub(2)
            },
        );
    }
    if rw(m, a0.wrapping_add(OB_VELY)) < 0xFC00 {
        return;
    }
    let vx = rw(m, a0.wrapping_add(OB_VELX));
    let atrito = u16_de(s16(vx) >> 5);
    if atrito == 0 {
        return;
    }
    let (novo, borrow) = vx.overflowing_sub(atrito);
    let valor = if (s16(atrito) > 0 && borrow) || (s16(atrito) < 0 && !borrow) {
        0
    } else {
        novo
    };
    ww(m, a0.wrapping_add(OB_VELX), valor);
}

/// Sensores paralelos de teto e piso; devolve a maior e a menor distância e o ângulo.
fn sensores_verticais(m: &mut Memoria, a0: u32, teto: bool) -> (u16, u16, u8) {
    let x = rw(m, a0.wrapping_add(OB_X));
    let y = rw(m, a0.wrapping_add(OB_Y));
    let altura = ext_w(rb(m, a0.wrapping_add(OB_HEIGHT)));
    let largura = ext_w(rb(m, a0.wrapping_add(OB_WIDTH)));
    let y = if teto {
        y.wrapping_sub(altura) ^ 15
    } else {
        y.wrapping_add(altura)
    };
    let mut b = Busca {
        a0,
        d5: if teto { 14 } else { 13 },
        d6: if teto { 0x1000 } else { 0 },
        a3: if teto { 0xFFF0 } else { 16 },
        a4: V_ANGLEBUFFER,
    };
    let d0 = find_floor(m, b, y, x.wrapping_add(largura));
    b.a4 = V_ANGLEBUFFER2;
    let d1 = find_floor(m, b, y, x.wrapping_sub(largura));
    let (maior, menor, angulo) = if s16(d1) <= s16(d0) {
        (d0, d1, rb(m, V_ANGLEBUFFER2))
    } else {
        (d1, d0, rb(m, V_ANGLEBUFFER))
    };
    (
        maior,
        menor,
        if angulo & 1 != 0 {
            if teto { 0x80 } else { 0 }
        } else {
            angulo
        },
    )
}

fn sensor_parede_rapido(m: &mut Memoria, a0: u32, esquerda: bool) -> u16 {
    let x = rw(m, a0.wrapping_add(OB_X));
    let y = rw(m, a0.wrapping_add(OB_Y));
    let x = if esquerda {
        x.wrapping_sub(10) ^ 15
    } else {
        x.wrapping_add(10)
    };
    let b = Busca {
        a0,
        d5: 14,
        d6: if esquerda { 0x800 } else { 0 },
        a3: if esquerda { 0xFFF0 } else { 16 },
        a4: V_ANGLEBUFFER,
    };
    find_wall(m, b, y, x)
}

/// `ObjHitWallLeft` e `ObjHitWallRight`, incluindo a assimetria da REV01 sem FixBugs.
pub(super) fn obj_hit_wall(m: &mut Memoria, a0: u32, distancia: u16, esquerda: bool) -> u16 {
    let x = rw(m, a0.wrapping_add(OB_X)).wrapping_add(distancia);
    let y = rw(m, a0.wrapping_add(OB_Y));
    wb(m, V_ANGLEBUFFER, 0);
    let b = Busca {
        a0,
        d5: 14,
        d6: if esquerda { 0x800 } else { 0 },
        a3: if esquerda { 0xFFF0 } else { 16 },
        a4: V_ANGLEBUFFER,
    };
    find_wall(m, b, y, x)
}

pub(super) fn sonic_reset_on_floor(m: &mut Memoria, a0: u32) {
    let status = rb(m, a0.wrapping_add(OB_STATUS));
    wb(
        m,
        a0.wrapping_add(OB_STATUS),
        status & !(STATUS_PUSHING | STATUS_IN_AIR | STATUS_ROLLING | 0x10),
    );
    if status & STATUS_ROLLING != 0 {
        wb(m, a0.wrapping_add(OB_HEIGHT), SONIC_HEIGHT);
        wb(m, a0.wrapping_add(OB_WIDTH), SONIC_WIDTH);
        wb(m, a0.wrapping_add(OB_ANIM), ANIM_WALK);
        let y = rw(m, a0.wrapping_add(OB_Y)).wrapping_sub(5);
        ww(m, a0.wrapping_add(OB_Y), y);
    }
    wb(m, a0.wrapping_add(SON_JUMPING), 0);
    ww(m, V_ITEMBONUS, 0);
}

/// `Sonic_Floor`: despacha os sensores conforme o vetor de velocidade.
fn sonic_floor(m: &mut Memoria, a0: u32) {
    let vx = rw(m, a0.wrapping_add(OB_VELX));
    let vy = rw(m, a0.wrapping_add(OB_VELY));
    let angulo = calc_angle(m, vx, vy);
    let rodado = angulo.wrapping_sub(0x20);
    let direcao = rodado & 0xC0;
    wb(m, V_UNUSED3, angulo);
    wb(m, V_UNUSED3.wrapping_add(1), rodado);
    wb(m, V_UNUSED3.wrapping_add(2), direcao);
    for esquerda in [true, false] {
        if (direcao == 0x40 && !esquerda) || (direcao == 0xC0 && esquerda) {
            continue;
        }
        let distancia = sensor_parede_rapido(m, a0, esquerda);
        if s16(distancia) < 0 {
            let x = rw(m, a0.wrapping_add(OB_X));
            ww(
                m,
                a0.wrapping_add(OB_X),
                if esquerda {
                    x.wrapping_sub(distancia)
                } else {
                    x.wrapping_add(distancia)
                },
            );
            ww(m, a0.wrapping_add(OB_VELX), 0);
            if direcao == 0x40 || direcao == 0xC0 {
                let vy = rw(m, a0.wrapping_add(OB_VELY));
                ww(m, a0.wrapping_add(OB_INERTIA), vy);
                return;
            }
        }
    }
    if direcao != 0 {
        let (_, distancia, angulo) = sensores_verticais(m, a0, true);
        if s16(distancia) < 0 {
            let y = rw(m, a0.wrapping_add(OB_Y)).wrapping_sub(distancia);
            ww(m, a0.wrapping_add(OB_Y), y);
            if direcao == 0x80 && angulo.wrapping_add(0x20) & 0x40 != 0 {
                wb(m, a0.wrapping_add(OB_ANGLE), angulo);
                sonic_reset_on_floor(m, a0);
                let vel = rw(m, a0.wrapping_add(OB_VELY));
                ww(
                    m,
                    a0.wrapping_add(OB_INERTIA),
                    if angulo & 0x80 != 0 {
                        vel.wrapping_neg()
                    } else {
                        vel
                    },
                );
            } else if direcao == 0x80 || s16(rw(m, a0.wrapping_add(OB_VELY))) < 0 {
                ww(m, a0.wrapping_add(OB_VELY), 0);
            }
            return;
        }
        if direcao == 0x80 || s16(rw(m, a0.wrapping_add(OB_VELY))) < 0 {
            return;
        }
    }
    let (maior, distancia, angulo) = sensores_verticais(m, a0, false);
    if direcao == 0 {
        wb(m, V_UNUSED3.wrapping_add(3), distancia.to_be_bytes()[1]);
    }
    if s16(distancia) >= 0 {
        return;
    }
    if direcao == 0 {
        let limite = vy.to_be_bytes()[0].wrapping_add(8).wrapping_neg();
        let menor_b = i8::from_be_bytes([distancia.to_be_bytes()[1]]);
        let maior_b = i8::from_be_bytes([maior.to_be_bytes()[1]]);
        let limite = i8::from_be_bytes([limite]);
        if menor_b < limite && maior_b < limite {
            return;
        }
    }
    let y = rw(m, a0.wrapping_add(OB_Y)).wrapping_add(distancia);
    ww(m, a0.wrapping_add(OB_Y), y);
    wb(m, a0.wrapping_add(OB_ANGLE), angulo);
    sonic_reset_on_floor(m, a0);
    wb(m, a0.wrapping_add(OB_ANIM), ANIM_WALK);
    if direcao != 0
        || angulo.wrapping_add(0x10) & 0x20 == 0 && angulo.wrapping_add(0x20) & 0x40 == 0
    {
        ww(m, a0.wrapping_add(OB_VELY), 0);
        let vx = rw(m, a0.wrapping_add(OB_VELX));
        ww(m, a0.wrapping_add(OB_INERTIA), vx);
    } else {
        let mut vel = rw(m, a0.wrapping_add(OB_VELY));
        if angulo.wrapping_add(0x20) & 0x40 == 0 {
            vel = u16_de(s16(vel) >> 1);
        } else {
            ww(m, a0.wrapping_add(OB_VELX), 0);
            if s16(vel) > 0xFC0 {
                vel = 0xFC0;
            }
        }
        ww(m, a0.wrapping_add(OB_VELY), vel);
        ww(
            m,
            a0.wrapping_add(OB_INERTIA),
            if angulo & 0x80 != 0 {
                vel.wrapping_neg()
            } else {
                vel
            },
        );
    }
}

fn sonic_no_ar(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let vy = rw(m, a0.wrapping_add(OB_VELY));
    if rb(m, a0.wrapping_add(SON_JUMPING)) != 0 {
        let limite = if rb(m, a0.wrapping_add(OB_STATUS)) & 0x40 != 0 {
            -0x200
        } else {
            -0x400
        };
        if s16(vy) < limite && rb(m, V_JPADHOLD2) & 0x70 == 0 {
            ww(m, a0.wrapping_add(OB_VELY), u16_de(limite));
        }
    } else if s16(vy) < -0xFC0 {
        ww(m, a0.wrapping_add(OB_VELY), u16_de(-0xFC0));
    }
    sonic_jump_direction(m, a0);
    sonic_level_bound(m, a0)?;
    speed_to_pos(m, a0);
    let gravidade = if rb(m, a0.wrapping_add(OB_STATUS)) & 0x40 != 0 {
        0x10
    } else {
        0x38
    };
    let vy = rw(m, a0.wrapping_add(OB_VELY)).wrapping_add(gravidade);
    ww(m, a0.wrapping_add(OB_VELY), vy);
    let angulo = rb(m, a0.wrapping_add(OB_ANGLE));
    if angulo != 0 {
        let novo = if angulo & 0x80 != 0 {
            angulo.checked_add(2).unwrap_or(0)
        } else {
            angulo.saturating_sub(2)
        };
        wb(m, a0.wrapping_add(OB_ANGLE), novo);
    }
    sonic_floor(m, a0);
    Ok(())
}

/// `Sonic_ResetScr` e `Sonic_CheckDpadLetGo`: o olhar volta ao centro e a inércia desacelera.
fn sonic_reset_scr(m: &mut Memoria, a0: u32, d5: u16) -> Result<(), ErroAbertura> {
    let olhar = rw(m, V_LOOKSHIFT);
    if olhar != 0x60 {
        let mut novo = olhar;
        if olhar < 0x60 {
            novo = novo.wrapping_add(4);
        }
        novo = novo.wrapping_sub(2);
        ww(m, V_LOOKSHIFT, novo);
    }
    if rb(m, V_JPADHOLD2) & 0x0C == 0 {
        let inercia = rw(m, a0.wrapping_add(OB_INERTIA));
        if inercia != 0 {
            let novo = if s16(inercia) > 0 {
                inercia.saturating_sub(d5)
            } else {
                inercia.checked_add(d5).unwrap_or(0)
            };
            ww(m, a0.wrapping_add(OB_INERTIA), novo);
        }
    }
    sonic_angle_speed(m, a0)
}

/// `Sonic_AngleSpeed`: a velocidade a partir da inércia e do ângulo.
fn sonic_angle_speed(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let angulo = rb(m, a0.wrapping_add(OB_ANGLE));
    let (seno, cosseno) = calc_sine(m, angulo);
    let inercia = s16(rw(m, a0.wrapping_add(OB_INERTIA)));
    ww(
        m,
        a0.wrapping_add(OB_VELX),
        u16_de(muls_asr8(cosseno, inercia)),
    );
    ww(
        m,
        a0.wrapping_add(OB_VELY),
        u16_de(muls_asr8(seno, inercia)),
    );
    sonic_wall_speed_adjust(m, a0, angulo, inercia)
}

/// `Sonic_WallSpeedAdjust`: usa a velocidade já calculada pelo modo de movimento.
fn sonic_wall_speed_adjust(
    m: &mut Memoria,
    a0: u32,
    angulo: u8,
    inercia: i16,
) -> Result<(), ErroAbertura> {
    if angulo.wrapping_add(0x40) & 0x80 != 0 || inercia == 0 {
        return Ok(());
    }
    let frente = if inercia < 0 {
        angulo.wrapping_add(0x40)
    } else {
        angulo.wrapping_sub(0x40)
    };
    let x = rl(m, a0.wrapping_add(OB_X))
        .wrapping_add(ext_l(rw(m, a0.wrapping_add(OB_VELX))).wrapping_shl(8));
    let y = rl(m, a0.wrapping_add(OB_Y))
        .wrapping_add(ext_l(rw(m, a0.wrapping_add(OB_VELY))).wrapping_shl(8));
    let mut x = u16::try_from(x >> 16).unwrap_or(0);
    let mut y = u16::try_from(y >> 16).unwrap_or(0);
    wb(m, V_ANGLEBUFFER, frente);
    wb(m, V_ANGLEBUFFER2, frente);
    let classe = if frente.wrapping_add(0x20) & 0x80 != 0 {
        (if frente & 0x80 != 0 {
            frente.wrapping_sub(1)
        } else {
            frente
        })
        .wrapping_add(0x20)
            & 0xC0
    } else {
        (if frente & 0x80 != 0 {
            frente.wrapping_add(1)
        } else {
            frente
        })
        .wrapping_add(0x1F)
            & 0xC0
    };
    let b = Busca {
        a0,
        d5: 14,
        d6: if classe == 0x40 {
            0x800
        } else if classe == 0x80 {
            0x1000
        } else {
            0
        },
        a3: if classe == 0x40 || classe == 0x80 {
            0xFFF0
        } else {
            16
        },
        a4: V_ANGLEBUFFER,
    };
    let distancia = match classe {
        0 => {
            y = y.wrapping_add(10);
            find_floor(m, b, y, x)
        }
        0x80 => {
            y = y.wrapping_sub(10) ^ 15;
            find_floor(m, b, y, x)
        }
        _ => {
            if frente & 0x38 == 0 {
                y = y.wrapping_add(8);
            }
            x = if classe == 0x40 {
                x.wrapping_sub(10) ^ 15
            } else {
                x.wrapping_add(10)
            };
            find_wall(m, b, y, x)
        }
    };
    if s16(distancia) >= 0 {
        return Ok(());
    }
    let ajuste = distancia.wrapping_shl(8);
    match frente.wrapping_add(0x20) & 0xC0 {
        0 => {
            let v = rw(m, a0.wrapping_add(OB_VELY)).wrapping_add(ajuste);
            ww(m, a0.wrapping_add(OB_VELY), v);
        }
        0x80 => {
            let v = rw(m, a0.wrapping_add(OB_VELY)).wrapping_sub(ajuste);
            ww(m, a0.wrapping_add(OB_VELY), v);
        }
        lado => {
            let v = rw(m, a0.wrapping_add(OB_VELX));
            let v = if lado == 0x40 {
                v.wrapping_sub(ajuste)
            } else {
                v.wrapping_add(ajuste)
            };
            ww(m, a0.wrapping_add(OB_VELX), v);
            let status = rb(m, a0.wrapping_add(OB_STATUS));
            wb(m, a0.wrapping_add(OB_STATUS), status | STATUS_PUSHING);
            ww(m, a0.wrapping_add(OB_INERTIA), 0);
        }
    }
    Ok(())
}

/// `Sonic_Roll`: só rola com velocidade e o botão de baixo; sem botão nada acontece.
fn sonic_roll(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    if rb(m, F_SLIDEMODE) != 0 {
        return Ok(());
    }
    let inercia = s16(rw(m, a0.wrapping_add(OB_INERTIA)));
    if inercia.unsigned_abs() < 0x80 {
        return Ok(());
    }
    if rb(m, V_JPADHOLD2) & 0x0C != 0 {
        return Ok(());
    }
    if rb(m, V_JPADHOLD2) & 0x02 != 0 {
        return sonic_chk_roll(m, a0);
    }
    Ok(())
}

/// `Sonic_SlopeResistRoll`: a força de subida é um quarto da de descida.
fn sonic_slope_resist_roll(m: &mut Memoria, a0: u32) {
    let angulo = rb(m, a0.wrapping_add(OB_ANGLE));
    if angulo.wrapping_add(0x60) >= 0xC0 {
        return;
    }
    let (seno, _) = calc_sine(m, angulo);
    let mut forca = muls_asr8(seno, 0x50);
    let inercia = rw(m, a0.wrapping_add(OB_INERTIA));
    if (s16(inercia) >= 0 && forca < 0) || (s16(inercia) < 0 && forca >= 0) {
        forca >>= 2;
    }
    ww(
        m,
        a0.wrapping_add(OB_INERTIA),
        inercia.wrapping_add(u16_de(forca)),
    );
}

/// `Sonic_RollSpeed`: aplica freio de direção, atrito e projeção de velocidade.
fn sonic_roll_speed(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    if rb(m, F_SLIDEMODE) == 0 {
        let freio = u16_de(s16(rw(m, V_SONSPEEDDEC)) >> 2);
        if rw(m, a0.wrapping_add(SON_LOCKTIME)) == 0 {
            let botoes = rb(m, V_JPADHOLD2);
            for direita in [false, true] {
                if botoes & if direita { 8 } else { 4 } == 0 {
                    continue;
                }
                let v = rw(m, a0.wrapping_add(OB_INERTIA));
                if (direita && s16(v) >= 0) || (!direita && s16(v) <= 0) {
                    let status = rb(m, a0.wrapping_add(OB_STATUS));
                    wb(
                        m,
                        a0.wrapping_add(OB_STATUS),
                        if direita {
                            status & !STATUS_XFLIP
                        } else {
                            status | STATUS_XFLIP
                        },
                    );
                    wb(m, a0.wrapping_add(OB_ANIM), ANIM_ROLL);
                } else {
                    let (novo, carry) = if direita {
                        v.overflowing_add(freio)
                    } else {
                        v.overflowing_sub(freio)
                    };
                    ww(
                        m,
                        a0.wrapping_add(OB_INERTIA),
                        if carry {
                            if direita { 0x80 } else { 0xFF80 }
                        } else {
                            novo
                        },
                    );
                }
            }
        }
        let atrito = u16_de(s16(rw(m, V_SONSPEEDACC)) >> 1);
        let inercia = rw(m, a0.wrapping_add(OB_INERTIA));
        if inercia != 0 {
            let v = if s16(inercia) > 0 {
                inercia.saturating_sub(atrito)
            } else {
                inercia.checked_add(atrito).unwrap_or(0)
            };
            ww(m, a0.wrapping_add(OB_INERTIA), v);
        }
        if rw(m, a0.wrapping_add(OB_INERTIA)) == 0 {
            let status = rb(m, a0.wrapping_add(OB_STATUS));
            wb(m, a0.wrapping_add(OB_STATUS), status & !STATUS_ROLLING);
            wb(m, a0.wrapping_add(OB_HEIGHT), SONIC_HEIGHT);
            wb(m, a0.wrapping_add(OB_WIDTH), SONIC_WIDTH);
            wb(m, a0.wrapping_add(OB_ANIM), ANIM_WAIT);
            let y = rw(m, a0.wrapping_add(OB_Y));
            ww(m, a0.wrapping_add(OB_Y), y.wrapping_sub(5));
        }
    }
    let angulo = rb(m, a0.wrapping_add(OB_ANGLE));
    let inercia = s16(rw(m, a0.wrapping_add(OB_INERTIA)));
    let (seno, cosseno) = calc_sine(m, angulo);
    let vx = muls_asr8(cosseno, inercia).clamp(-0x1000, 0x1000);
    ww(m, a0.wrapping_add(OB_VELX), u16_de(vx));
    ww(
        m,
        a0.wrapping_add(OB_VELY),
        u16_de(muls_asr8(seno, inercia)),
    );
    sonic_wall_speed_adjust(m, a0, angulo, inercia)
}

/// `Sonic_ChkRoll`: vira bola.
fn sonic_chk_roll(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let status = rb(m, a0.wrapping_add(OB_STATUS));
    if status & STATUS_ROLLING != 0 {
        return Ok(());
    }
    wb(m, a0.wrapping_add(OB_STATUS), status | STATUS_ROLLING);
    wb(m, a0.wrapping_add(OB_HEIGHT), SONIC_ROLL_HEIGHT);
    wb(m, a0.wrapping_add(OB_WIDTH), SONIC_ROLL_WIDTH);
    wb(m, a0.wrapping_add(OB_ANIM), ANIM_ROLL);
    let y = rw(m, a0.wrapping_add(OB_Y));
    let dif = u16::from(SONIC_HEIGHT.wrapping_sub(SONIC_ROLL_HEIGHT));
    ww(m, a0.wrapping_add(OB_Y), y.wrapping_add(dif));
    wb(m, V_SOUNDQUEUE1, SFX_ROLL);
    if rw(m, a0.wrapping_add(OB_INERTIA)) == 0 {
        ww(m, a0.wrapping_add(OB_INERTIA), 0x200);
    }
    Ok(())
}

/// `Sonic_LevelBound`: os limites da fase.
fn sonic_level_bound(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let x = rl(m, a0.wrapping_add(OB_X));
    let velx = ext_l(rw(m, a0.wrapping_add(OB_VELX)));
    let futuro = u16::try_from((x.wrapping_add(velx.wrapping_shl(8)) >> 16) & 0xFFFF).unwrap_or(0);
    let esquerda = rw(m, V_LIMITLEFT2).wrapping_add(16);
    let mut limite = esquerda;
    let mut lado = esquerda > futuro;
    if !lado {
        let mut direita = rw(m, V_LIMITRIGHT2).wrapping_add(320 - 24);
        if rb(m, F_LOCKSCREEN) == 0 {
            direita = direita.wrapping_add(64);
        }
        limite = direita;
        lado = direita <= futuro;
    }
    if lado {
        ww(m, a0.wrapping_add(OB_X), limite);
        ww(m, a0.wrapping_add(OB_X + 2), 0);
        ww(m, a0.wrapping_add(OB_VELX), 0);
        ww(m, a0.wrapping_add(OB_INERTIA), 0);
    }
    let fundo = rw(m, V_LIMITBTM2).wrapping_add(224);
    if s16(fundo) < s16(rw(m, a0.wrapping_add(OB_Y))) {
        return Err(ErroAbertura::NaoImplementado(
            "Sonic abaixo do limite da fase",
        ));
    }
    Ok(())
}

/// `ext.l`: a palavra com sinal vira longa.
fn ext_l(w: u16) -> u32 {
    u32::from_be_bytes(i32::from(s16(w)).to_be_bytes())
}

/// `SpeedToPos`: a posição (16.16) soma a velocidade (8.8, deslocada para 16.16).
pub(super) fn speed_to_pos(m: &mut Memoria, a0: u32) {
    let x = rl(m, a0.wrapping_add(OB_X));
    let y = rl(m, a0.wrapping_add(OB_Y));
    let vx = ext_l(rw(m, a0.wrapping_add(OB_VELX))).wrapping_shl(8);
    let vy = ext_l(rw(m, a0.wrapping_add(OB_VELY))).wrapping_shl(8);
    wl(m, a0.wrapping_add(OB_X), x.wrapping_add(vx));
    wl(m, a0.wrapping_add(OB_Y), y.wrapping_add(vy));
}

/// `ObjectFall`: aplica gravidade de 0x38 ao `obVelY` e atualiza a posição (16.16).
pub(super) fn object_fall(m: &mut Memoria, a0: u32) {
    let x = rl(m, a0.wrapping_add(OB_X));
    let y = rl(m, a0.wrapping_add(OB_Y));
    let vx = ext_l(rw(m, a0.wrapping_add(OB_VELX))).wrapping_shl(8);
    let vy = ext_l(rw(m, a0.wrapping_add(OB_VELY))).wrapping_shl(8);
    let nvy = rw(m, a0.wrapping_add(OB_VELY)).wrapping_add(0x38);
    ww(m, a0.wrapping_add(OB_VELY), nvy);
    wl(m, a0.wrapping_add(OB_X), x.wrapping_add(vx));
    wl(m, a0.wrapping_add(OB_Y), y.wrapping_add(vy));
}

/// `Sonic_SlopeRepel`: numa subida íngreme e devagar o Sonic cai da rampa.
fn sonic_slope_repel(m: &mut Memoria, a0: u32) {
    if rb(m, a0.wrapping_add(SON_STICKTOCONVEX)) != 0 {
        return;
    }
    let trava = rw(m, a0.wrapping_add(SON_LOCKTIME));
    if trava != 0 {
        ww(m, a0.wrapping_add(SON_LOCKTIME), trava.wrapping_sub(1));
        return;
    }
    let angulo = rb(m, a0.wrapping_add(OB_ANGLE));
    if angulo.wrapping_add(0x20) & 0xC0 == 0 {
        return;
    }
    let inercia = s16(rw(m, a0.wrapping_add(OB_INERTIA)));
    if inercia.unsigned_abs() >= 0x280 {
        return;
    }
    ww(m, a0.wrapping_add(OB_INERTIA), 0);
    let s = rb(m, a0.wrapping_add(OB_STATUS));
    wb(m, a0.wrapping_add(OB_STATUS), s | STATUS_IN_AIR);
    ww(m, a0.wrapping_add(SON_LOCKTIME), 30);
}

// ---------------------------------------------------------------------------------------------
// O objeto do Sonic
// ---------------------------------------------------------------------------------------------

/// `SonicPlayer` (objeto `01`): `Sonic_Main` e, no mesmo quadro, `Sonic_Control`.
pub fn sonic_player(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    if rw(m, V_DEBUGUSE) != 0 {
        return Err(ErroAbertura::NaoImplementado("modo de depuração"));
    }
    match rb(m, a0.wrapping_add(OB_ROUTINE)) {
        0 => {
            sonic_main(m, a0);
            sonic_control(m, a0)
        }
        2 => sonic_control(m, a0),
        4 => sonic_hurt(m, a0),
        6 => sonic_death(m, a0),
        8 => sonic_reset_level(m, a0),
        _ => Err(ErroAbertura::NaoImplementado(
            "rotina do Sonic desconhecida",
        )),
    }
}

/// `Sonic_Main`.
fn sonic_main(m: &mut Memoria, a0: u32) {
    wb(m, a0.wrapping_add(OB_ROUTINE), 2);
    wb(m, a0.wrapping_add(OB_HEIGHT), SONIC_HEIGHT);
    wb(m, a0.wrapping_add(OB_WIDTH), SONIC_WIDTH);
    wl(m, a0.wrapping_add(OB_MAP), MAP_SONIC);
    ww(m, a0.wrapping_add(OB_GFX), ARTTILE_SONIC);
    wb(m, a0.wrapping_add(OB_PRIORITY), 2);
    wb(m, a0.wrapping_add(OB_ACTWID), 48 / 2);
    wb(m, a0.wrapping_add(OB_RENDER), RENDER_CAM_FIELD);
    ww(m, V_SONSPEEDMAX, SON_MAXSPEED);
    ww(m, V_SONSPEEDACC, SON_ACCELERATION);
    ww(m, V_SONSPEEDDEC, SON_DECELERATION);
}

/// `Sonic_Control`.
fn sonic_control(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    if rw(m, F_DEBUGMODE) != 0 && rb(m, V_JPADPRESS1) & 0x10 != 0 {
        return Err(ErroAbertura::NaoImplementado("modo de depuração"));
    }
    if rb(m, F_LOCKCTRL) == 0 {
        let botoes = rw(m, V_JPADHOLD1);
        ww(m, V_JPADHOLD2, botoes);
    }
    if rb(m, F_PLAYERCTRL) & 1 == 0 {
        let status = rb(m, a0.wrapping_add(OB_STATUS)) & (STATUS_IN_AIR | STATUS_ROLLING);
        match status {
            0 => {
                if !sonic_jump(m, a0)? {
                    sonic_slope_resist_walk(m, a0);
                    sonic_move(m, a0)?;
                    sonic_roll(m, a0)?;
                    sonic_level_bound(m, a0)?;
                    speed_to_pos(m, a0);
                    sonic_angle_pos(m, a0)?;
                    sonic_slope_repel(m, a0);
                }
            }
            STATUS_IN_AIR | 6 => sonic_no_ar(m, a0)?,
            STATUS_ROLLING => {
                if !sonic_jump(m, a0)? {
                    sonic_slope_resist_roll(m, a0);
                    sonic_roll_speed(m, a0)?;
                    sonic_level_bound(m, a0)?;
                    speed_to_pos(m, a0);
                    sonic_angle_pos(m, a0)?;
                    sonic_slope_repel(m, a0);
                }
            }
            _ => {
                return Err(ErroAbertura::NaoImplementado(
                    "estado de movimento do Sonic",
                ));
            }
        }
    }
    sonic_display(m, a0)?;
    sonic_record_position(m, a0);
    // `Sonic_Water` só faz algo na LZ.
    if rb(m, V_ZONE) == ID_LZ {
        return Err(ErroAbertura::NaoImplementado("Sonic na água"));
    }
    let direita = rb(m, V_ANGLEBUFFER);
    let esquerda = rb(m, V_ANGLEBUFFER2);
    wb(m, a0.wrapping_add(SON_ANGLERIGHT), direita);
    wb(m, a0.wrapping_add(SON_ANGLELEFT), esquerda);
    if rb(m, F_WTUNNELMODE) != 0 {
        return Err(ErroAbertura::NaoImplementado("túnel de vento"));
    }
    sonic_animate(m, a0)?;
    if rb(m, F_PLAYERCTRL) & 0x80 == 0 {
        react_to_item(m)?;
    }
    sonic_loops(m, a0)?;
    sonic_load_gfx(m, a0);
    Ok(())
}

/// `Sonic_Hurt` (rotina 4): Sonic após sofrer dano, sendo lançado no ar.
fn sonic_hurt(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    speed_to_pos(m, a0);
    let gravidade = if rb(m, a0.wrapping_add(OB_STATUS)) & STATUS_UNDERWATER != 0 {
        0x10u16
    } else {
        0x38 - 8 // 0x30 = 48
    };
    let vy = rw(m, a0.wrapping_add(OB_VELY));
    ww(m, a0.wrapping_add(OB_VELY), vy.wrapping_add(gravidade));

    sonic_hurt_stop(m, a0)?;
    sonic_level_bound(m, a0)?;
    sonic_record_position(m, a0);
    sonic_animate(m, a0)?;
    sonic_load_gfx(m, a0);
    display_sprite(m, a0);
    Ok(())
}

/// `Sonic_HurtStop`: verifica colisão com chão após dano e restaura estado normal.
fn sonic_hurt_stop(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let limit_btm = rw(m, V_LIMITBTM2).wrapping_add(224);
    let y = rw(m, a0.wrapping_add(OB_Y));
    if y > limit_btm {
        kill_sonic(m);
        return Ok(());
    }
    sonic_floor(m, a0);
    if rb(m, a0.wrapping_add(OB_STATUS)) & STATUS_IN_AIR == 0 {
        ww(m, a0.wrapping_add(OB_VELY), 0);
        ww(m, a0.wrapping_add(OB_VELX), 0);
        ww(m, a0.wrapping_add(OB_INERTIA), 0);
        wb(m, a0.wrapping_add(OB_ANIM), ANIM_WALK);
        let rotina = rb(m, a0.wrapping_add(OB_ROUTINE)).wrapping_sub(2);
        wb(m, a0.wrapping_add(OB_ROUTINE), rotina);
        ww(m, a0.wrapping_add(SON_FLASHTIME), 120);
    }
    Ok(())
}

/// `Sonic_Death` (rotina 6): Sonic morrendo e caindo da tela.
fn sonic_death(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    sonic_handle_death(m, a0)?;
    object_fall(m, a0);
    sonic_record_position(m, a0);
    sonic_animate(m, a0)?;
    sonic_load_gfx(m, a0);
    display_sprite(m, a0);
    Ok(())
}

/// `Sonic_HandleDeath`: verifica se o Sonic caiu abaixo da tela e decrementa vidas.
fn sonic_handle_death(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let limit = rw(m, V_LIMITBTM2).wrapping_add(0x100);
    let y = rw(m, a0.wrapping_add(OB_Y));
    if y <= limit {
        return Ok(());
    }
    ww(m, a0.wrapping_add(OB_VELY), u16_de(-0x38i16));
    wb(m, a0.wrapping_add(OB_ROUTINE), 8);
    wb(m, F_TIMECOUNT, 0);
    let life_count = rb(m, F_LIFECOUNT).wrapping_add(1);
    wb(m, F_LIFECOUNT, life_count);
    let lives = rb(m, V_LIVES).wrapping_sub(1);
    wb(m, V_LIVES, lives);
    if lives != 0 && rb(m, F_TIMEOVER) == 0 {
        ww(m, a0.wrapping_add(SON_RESTARTIME), 60);
    } else {
        ww(m, a0.wrapping_add(SON_RESTARTIME), 0);
        wb(m, 0xFF_D080, 0x39);
        wb(m, 0xFF_D0C0, 0x39);
        if lives == 0 {
            wb(m, 0xFF_D0C0u32.wrapping_add(OB_FRAME), 1);
            wb(m, F_TIMEOVER, 0);
        } else {
            wb(m, 0xFF_D080u32.wrapping_add(OB_FRAME), 2);
            wb(m, 0xFF_D0C0u32.wrapping_add(OB_FRAME), 3);
        }
        wb(m, V_SOUNDQUEUE0, 0x8F);
        super::plc::add_plc(m, 3)?;
    }
    Ok(())
}

/// `Sonic_ResetLevel` (rotina 8): após morrer, espera 1 segundo para reiniciar a fase.
fn sonic_reset_level(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let t = rw(m, a0.wrapping_add(SON_RESTARTIME));
    if t != 0 {
        let nt = t.wrapping_sub(1);
        ww(m, a0.wrapping_add(SON_RESTARTIME), nt);
        if nt == 0 {
            ww(m, F_RESTART, 1);
        }
    }
    Ok(())
}

/// `Sonic_Display`: mostra o sprite, respeitando piscar de invulnerabilidade (`flashtime`).
fn sonic_display(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let flash = rw(m, a0.wrapping_add(SON_FLASHTIME));
    let mut deve_desenhar = true;
    if flash != 0 {
        ww(m, a0.wrapping_add(SON_FLASHTIME), flash.wrapping_sub(1));
        // lsr.w #3, d0 -> Carry é o bit 2 (máscara 4) do valor original
        if flash & 4 == 0 {
            deve_desenhar = false;
        }
    }
    if deve_desenhar {
        display_sprite(m, a0);
    }
    if rb(m, V_INVINC) != 0 || rb(m, V_SHOES) != 0 {
        return Err(ErroAbertura::NaoImplementado("invencibilidade e tênis"));
    }
    Ok(())
}

/// `Sonic_RecordPosition`: guarda a posição para a trilha da invencibilidade.
fn sonic_record_position(m: &mut Memoria, a0: u32) {
    let posicao = u32::from(rw(m, V_TRACKPOS));
    let destino = V_TRACKSONIC.wrapping_add(posicao);
    let x = rw(m, a0.wrapping_add(OB_X));
    let y = rw(m, a0.wrapping_add(OB_Y));
    ww(m, destino, x);
    ww(m, destino.wrapping_add(2), y);
    let byte = rb(m, V_TRACKBYTE).wrapping_add(4);
    wb(m, V_TRACKBYTE, byte);
}

/// `ReactToItem`: testa os retângulos dos objetos visíveis com tamanhos lidos da ROM.
fn react_to_item(m: &mut Memoria) -> Result<(), ErroAbertura> {
    let esquerda = rw(m, V_PLAYER.wrapping_add(OB_X)).wrapping_sub(8);
    let altura = rb(m, V_PLAYER.wrapping_add(OB_HEIGHT)).wrapping_sub(3);
    let mut topo = rw(m, V_PLAYER.wrapping_add(OB_Y)).wrapping_sub(u16::from(altura));
    let altura = if rb(m, V_PLAYER.wrapping_add(OB_FRAME)) == 0x39 {
        topo = topo.wrapping_add(12);
        10
    } else {
        u16::from(altura)
    };
    let mut a1 = V_LVLOBJSPACE;
    while a1 < V_LVLOBJEND {
        let tipo = rb(m, a1.wrapping_add(OB_COLTYPE));
        if rb(m, a1.wrapping_add(OB_RENDER)) & 0x80 != 0 && tipo != 0 {
            let tabela = REACT_SIZES
                .wrapping_add(u32::from(tipo & 0x3F).wrapping_mul(2))
                .wrapping_sub(2);
            let raio_x = u16::from(rom_b(m, tabela));
            let raio_y = u16::from(rom_b(m, tabela.wrapping_add(1)));
            let x = rw(m, a1.wrapping_add(OB_X)).wrapping_sub(raio_x);
            let y = rw(m, a1.wrapping_add(OB_Y)).wrapping_sub(raio_y);
            if retangulos_sobrepoem(x, esquerda, raio_x, 16)
                && retangulos_sobrepoem(y, topo, raio_y, altura.wrapping_mul(2))
            {
                if tipo & 0xC0 == 0x40 && tipo & 0x3F != 6 {
                    if rw(m, V_PLAYER.wrapping_add(SON_FLASHTIME)) < 90 {
                        let rotina = rb(m, a1.wrapping_add(OB_ROUTINE)).wrapping_add(2);
                        wb(m, a1.wrapping_add(OB_ROUTINE), rotina);
                    }
                    return Ok(());
                }
                if tipo == 0x46 {
                    let vy = s16(rw(m, V_PLAYER.wrapping_add(OB_VELY)));
                    if vy < 0 {
                        let sonic_y = rw(m, V_PLAYER.wrapping_add(OB_Y));
                        let obj_y = rw(m, a1.wrapping_add(OB_Y));
                        if sonic_y.wrapping_sub(16) >= obj_y {
                            ww(m, V_PLAYER.wrapping_add(OB_VELY), u16_de(vy.wrapping_neg()));
                            ww(m, a1.wrapping_add(OB_VELY), u16_de(-0x180i16));
                            let r2 = rb(m, a1.wrapping_add(OB_ROUTINE_2));
                            if r2 == 0 {
                                wb(m, a1.wrapping_add(OB_ROUTINE_2), 4);
                            }
                        }
                        return Ok(());
                    } else if rb(m, V_PLAYER.wrapping_add(OB_ANIM)) == ANIM_ROLL {
                        ww(m, V_PLAYER.wrapping_add(OB_VELY), u16_de(vy.wrapping_neg()));
                        let rotina = rb(m, a1.wrapping_add(OB_ROUTINE)).wrapping_add(2);
                        wb(m, a1.wrapping_add(OB_ROUTINE), rotina);
                        return Ok(());
                    } else {
                        return Ok(());
                    }
                }
                if tipo & 0xC0 == 0 {
                    let anim = rb(m, V_PLAYER.wrapping_add(OB_ANIM));
                    let invinc = rb(m, V_INVINC);
                    if invinc != 0 || anim == ANIM_ROLL {
                        let golpes = rb(m, a1.wrapping_add(0x21));
                        if golpes != 0 {
                            for campo in [OB_VELX, OB_VELY] {
                                let velocidade = s16(rw(m, V_PLAYER.wrapping_add(campo)));
                                ww(
                                    m,
                                    V_PLAYER.wrapping_add(campo),
                                    u16_de(velocidade.wrapping_neg() >> 1),
                                );
                            }
                            wb(m, a1.wrapping_add(OB_COLTYPE), 0);
                            let restantes = golpes.wrapping_sub(1);
                            wb(m, a1.wrapping_add(0x21), restantes);
                            if restantes == 0 {
                                let status = rb(m, a1.wrapping_add(OB_STATUS)) | 0x80;
                                wb(m, a1.wrapping_add(OB_STATUS), status);
                            }
                            return Ok(());
                        }
                        let status = rb(m, a1.wrapping_add(OB_STATUS)) | 0x80;
                        wb(m, a1.wrapping_add(OB_STATUS), status);

                        let combo = rw(m, V_ITEMBONUS);
                        let next_combo = combo.wrapping_add(2);
                        ww(m, V_ITEMBONUS, next_combo);
                        let (pts, pts_frame) = if next_combo >= 16 * 2 {
                            (1000, 10u16)
                        } else {
                            let d0 = if combo < 3 * 2 { combo } else { 3 * 2 };
                            let pts = match d0 {
                                0 => 10,
                                2 => 20,
                                4 => 50,
                                _ => 100,
                            };
                            (pts, d0)
                        };
                        ww(m, a1.wrapping_add(0x3E), pts_frame);
                        add_points(m, pts);

                        wb(m, a1.wrapping_add(OB_ID), ID_EXPLOSION_ITEM);
                        wb(m, a1.wrapping_add(OB_ROUTINE), 0);

                        let vy = s16(rw(m, V_PLAYER.wrapping_add(OB_VELY)));
                        if vy < 0 {
                            ww(
                                m,
                                V_PLAYER.wrapping_add(OB_VELY),
                                u16_de(vy.wrapping_add(0x100)),
                            );
                        } else {
                            let sonic_y = rw(m, V_PLAYER.wrapping_add(OB_Y));
                            let obj_y = rw(m, a1.wrapping_add(OB_Y));
                            if sonic_y >= obj_y {
                                ww(
                                    m,
                                    V_PLAYER.wrapping_add(OB_VELY),
                                    u16_de(vy.wrapping_sub(0x100)),
                                );
                            } else {
                                ww(m, V_PLAYER.wrapping_add(OB_VELY), u16_de(vy.wrapping_neg()));
                            }
                        }
                        return Ok(());
                    } else {
                        return hurt_sonic(m, a1);
                    }
                }
                if tipo & 0x80 != 0 {
                    return hurt_sonic(m, a1);
                }
                return Err(ErroAbertura::NaoImplementado("reação do Sonic ao objeto"));
            }
        }
        a1 = a1.wrapping_add(OBJECT_SIZE);
    }
    Ok(())
}

/// Subrotina `RandomNumber`: gerador de números pseudoaleatórios original do Sonic 1.
pub fn random_number(m: &mut Memoria) -> u16 {
    let mut d1 = rl(m, V_RANDOM);
    if d1 == 0 {
        d1 = 0x2A6D_365A;
    }
    let d0 = d1;
    d1 = d1.wrapping_shl(2).wrapping_add(d0);
    d1 = d1.wrapping_shl(3).wrapping_add(d0);
    let l = u16::try_from(d1 & 0xFFFF).unwrap_or(0);
    let h = u16::try_from((d1 >> 16) & 0xFFFF).unwrap_or(0);
    let sum = l.wrapping_add(h);
    let new_random = (u32::from(sum) << 16) | u32::from(l);
    wl(m, V_RANDOM, new_random);
    sum
}

/// O teste original usa carry das operações de 16 bits, inclusive nas bordas.
fn retangulos_sobrepoem(objeto: u16, sonic: u16, raio: u16, tamanho: u16) -> bool {
    let (separacao, emprestimo) = objeto.overflowing_sub(sonic);
    if emprestimo {
        separacao.overflowing_add(raio.wrapping_mul(2)).1
    } else {
        separacao <= tamanho
    }
}

pub fn add_points(m: &mut Memoria, d0: u32) {
    wb(m, F_SCORECOUNT, 1);
    let mut score = rl(m, V_SCORE).saturating_add(d0);
    if score > 999999 {
        score = 999999;
    }
    wl(m, V_SCORE, score);
    let scorelife = rl(m, V_SCORELIFE);
    if score >= scorelife {
        wl(m, V_SCORELIFE, scorelife.wrapping_add(5000));
        if i8::from_be_bytes([rb(m, V_MEGADRIVE)]) >= 0 {
            let vidas = rb(m, V_LIVES).wrapping_add(1);
            wb(m, V_LIVES, vidas);
            let vidas_f = rb(m, F_LIFECOUNT).wrapping_add(1);
            wb(m, F_LIFECOUNT, vidas_f);
            wb(m, V_SOUNDQUEUE0, 0x88);
        }
    }
}

pub(crate) fn hurt_sonic(m: &mut Memoria, a2: u32) -> Result<(), ErroAbertura> {
    if rw(m, V_PLAYER.wrapping_add(SON_FLASHTIME)) != 0 {
        return Ok(());
    }
    if rb(m, V_SHIELD) != 0 {
        wb(m, V_SHIELD, 0);
        wb(m, V_PLAYER.wrapping_add(OB_ROUTINE), 4);
        sonic_reset_on_floor(m, V_PLAYER);
        let status = rb(m, V_PLAYER.wrapping_add(OB_STATUS)) | STATUS_IN_AIR;
        wb(m, V_PLAYER.wrapping_add(OB_STATUS), status);
        ww(m, V_PLAYER.wrapping_add(OB_VELY), u16_de(-0x400i16));
        let sonic_x = rw(m, V_PLAYER.wrapping_add(OB_X));
        let obj_x = rw(m, a2.wrapping_add(OB_X));
        let vx = if sonic_x < obj_x { -0x200i16 } else { 0x200i16 };
        ww(m, V_PLAYER.wrapping_add(OB_VELX), u16_de(vx));
        ww(m, V_PLAYER.wrapping_add(OB_INERTIA), 0);
        wb(m, V_PLAYER.wrapping_add(OB_ANIM), ANIM_HURT);
        ww(m, V_PLAYER.wrapping_add(SON_FLASHTIME), 120);
        wb(m, V_SOUNDQUEUE1, 0xC6);
        return Ok(());
    }
    let rings = rw(m, V_RINGS);
    if rings > 0 {
        ww(m, V_RINGS, 0);
        wb(m, F_RINGCOUNT, 1);
        wb(m, V_PLAYER.wrapping_add(OB_ROUTINE), 4);
        sonic_reset_on_floor(m, V_PLAYER);
        let status = rb(m, V_PLAYER.wrapping_add(OB_STATUS)) | STATUS_IN_AIR;
        wb(m, V_PLAYER.wrapping_add(OB_STATUS), status);
        ww(m, V_PLAYER.wrapping_add(OB_VELY), u16_de(-0x400i16));
        let sonic_x = rw(m, V_PLAYER.wrapping_add(OB_X));
        let obj_x = rw(m, a2.wrapping_add(OB_X));
        let vx = if sonic_x < obj_x { -0x200i16 } else { 0x200i16 };
        ww(m, V_PLAYER.wrapping_add(OB_VELX), u16_de(vx));
        ww(m, V_PLAYER.wrapping_add(OB_INERTIA), 0);
        wb(m, V_PLAYER.wrapping_add(OB_ANIM), ANIM_HURT);
        ww(m, V_PLAYER.wrapping_add(SON_FLASHTIME), 120);
        wb(m, V_SOUNDQUEUE1, 0xC6);
        return Ok(());
    }
    kill_sonic(m);
    Ok(())
}

pub(crate) fn kill_sonic(m: &mut Memoria) {
    wb(m, V_PLAYER.wrapping_add(OB_ROUTINE), 6);
    sonic_reset_on_floor(m, V_PLAYER);
    let status = rb(m, V_PLAYER.wrapping_add(OB_STATUS)) | STATUS_IN_AIR;
    wb(m, V_PLAYER.wrapping_add(OB_STATUS), status);
    ww(m, V_PLAYER.wrapping_add(OB_VELY), u16_de(-0x700i16));
    ww(m, V_PLAYER.wrapping_add(OB_VELX), 0);
    ww(m, V_PLAYER.wrapping_add(OB_INERTIA), 0);
    let y = rw(m, V_PLAYER.wrapping_add(OB_Y));
    ww(m, V_PLAYER.wrapping_add(0x38), y);
    wb(m, V_PLAYER.wrapping_add(OB_ANIM), ANIM_DEATH);
    let gfx = rw(m, V_PLAYER.wrapping_add(OB_GFX));
    ww(m, V_PLAYER.wrapping_add(OB_GFX), gfx | 0x80);
    wb(m, V_INVINC, 0);
    wb(m, V_SOUNDQUEUE1, 0xC6);
}

/// Entradas isoladas para validar rotinas com estados preparados fora do jogo.
#[cfg(feature = "diagnostico")]
pub mod diagnostico {
    use super::*;

    pub fn mover_horizontal(m: &mut Memoria, a0: u32, direita: bool) {
        sonic_move_horizontal(m, a0, direita);
    }

    pub fn pular(m: &mut Memoria, a0: u32) -> Result<bool, ErroAbertura> {
        sonic_jump(m, a0)
    }

    pub fn parede(m: &mut Memoria, a0: u32, y: u16, x: u16, esquerda: bool) -> u16 {
        let b = Busca {
            a0,
            d5: 14,
            d6: if esquerda { 0x800 } else { 0 },
            a3: if esquerda { 0xFFF0 } else { 16 },
            a4: V_ANGLEBUFFER,
        };
        find_wall(m, b, y, x)
    }
}

/// `Sonic_Loops`: o sprite passa atrás ou na frente dos laços da GHZ.
fn sonic_loops(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let zona = rb(m, V_ZONE);
    if zona != 3 && zona != 0 {
        return Ok(());
    }
    let y = rw(m, a0.wrapping_add(OB_Y));
    let x = rb(m, a0.wrapping_add(OB_X));
    let indice = ((y >> 1) & 0x380).wrapping_add(u16::from(x) & 0x7F);
    let chunk = rb(m, V_LVLLAYOUT_FG.wrapping_add(u32::from(indice)));
    if chunk == rb(m, V_256ROLL1) || chunk == rb(m, V_256ROLL2) {
        return sonic_chk_roll(m, a0);
    }
    let render = rb(m, a0.wrapping_add(OB_RENDER));
    let esquerda = if chunk == rb(m, V_256LOOP1) {
        true
    } else if chunk == rb(m, V_256LOOP2) {
        if rb(m, a0.wrapping_add(OB_STATUS)) & STATUS_IN_AIR != 0 {
            wb(m, a0.wrapping_add(OB_RENDER), render & !RENDER_LOOPING);
            return Ok(());
        }
        true
    } else {
        wb(m, a0.wrapping_add(OB_RENDER), render & !RENDER_LOOPING);
        return Ok(());
    };
    if !esquerda {
        return Ok(());
    }
    let x = rw(m, a0.wrapping_add(OB_X)) & 0xFF;
    if x < 44 {
        wb(m, a0.wrapping_add(OB_RENDER), render & !RENDER_LOOPING);
        return Ok(());
    }
    if x >= 224 {
        wb(m, a0.wrapping_add(OB_RENDER), render | RENDER_LOOPING);
        return Ok(());
    }
    let angulo = rb(m, a0.wrapping_add(OB_ANGLE));
    if render & RENDER_LOOPING == 0 {
        if angulo != 0 && angulo <= 0x80 {
            wb(m, a0.wrapping_add(OB_RENDER), render | RENDER_LOOPING);
        }
    } else if angulo > 0x80 {
        wb(m, a0.wrapping_add(OB_RENDER), render & !RENDER_LOOPING);
    }
    Ok(())
}

/// `Sonic_Animate`: o roteiro da animação atual, com as animações de andar, correr, rolar e
/// empurrar, que têm tempo e quadro próprios.
pub(super) fn sonic_animate(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let anim = rb(m, a0.wrapping_add(OB_ANIM));
    if anim != rb(m, a0.wrapping_add(OB_PREV_ANI)) {
        wb(m, a0.wrapping_add(OB_PREV_ANI), anim);
        wb(m, a0.wrapping_add(OB_ANI_FRAME), 0);
        wb(m, a0.wrapping_add(OB_TIME_FRAME), 0);
    }
    let entrada = ANI_SONIC.wrapping_add(u32::from(anim).wrapping_mul(2));
    let roteiro = ANI_SONIC.wrapping_add(ext_l(rom_w(m, entrada)));
    let d0 = rom_b(m, roteiro);
    if d0 & 0x80 == 0 {
        let status = rb(m, a0.wrapping_add(OB_STATUS)) & STATUS_XFLIP;
        let render = rb(m, a0.wrapping_add(OB_RENDER)) & !RENDER_FLIPS;
        wb(m, a0.wrapping_add(OB_RENDER), render | status);
        let tempo = rb(m, a0.wrapping_add(OB_TIME_FRAME)).wrapping_sub(1);
        wb(m, a0.wrapping_add(OB_TIME_FRAME), tempo);
        if tempo & 0x80 == 0 {
            return Ok(());
        }
        wb(m, a0.wrapping_add(OB_TIME_FRAME), d0);
        return carregar_quadro(m, a0, roteiro);
    }
    // Andar, correr, rolar e empurrar.
    let tempo = rb(m, a0.wrapping_add(OB_TIME_FRAME)).wrapping_sub(1);
    wb(m, a0.wrapping_add(OB_TIME_FRAME), tempo);
    if tempo & 0x80 == 0 {
        return Ok(());
    }
    let status = rb(m, a0.wrapping_add(OB_STATUS));
    let inercia = rw(m, a0.wrapping_add(OB_INERTIA));
    let velocidade = if s16(inercia) < 0 {
        inercia.wrapping_neg()
    } else {
        inercia
    };
    let mut deslocamento = 0u8;
    let roteiro = if d0 == 0xFF && status & STATUS_PUSHING == 0 {
        let mut angulo = rb(m, a0.wrapping_add(OB_ANGLE));
        if status & STATUS_XFLIP == 0 {
            angulo = !angulo;
        }
        angulo = angulo.wrapping_add(0x10);
        let inversao = if angulo & 0x80 != 0 { RENDER_FLIPS } else { 0 };
        let render = rb(m, a0.wrapping_add(OB_RENDER)) & !RENDER_FLIPS;
        wb(
            m,
            a0.wrapping_add(OB_RENDER),
            render | ((status & STATUS_XFLIP) ^ inversao),
        );
        let mut octante = (angulo >> 4) & 6;
        let roteiro = if velocidade >= 0x600 {
            SONANI_RUN
        } else {
            octante = octante.wrapping_add(octante >> 1);
            SONANI_WALK
        };
        deslocamento = octante.wrapping_mul(2);
        let intervalo = 0x800u16.wrapping_sub(velocidade);
        wb(
            m,
            a0.wrapping_add(OB_TIME_FRAME),
            if s16(intervalo) < 0 {
                0
            } else {
                intervalo.to_be_bytes()[0]
            },
        );
        roteiro
    } else if d0 == 0xFE {
        let intervalo = 0x400u16.wrapping_sub(velocidade);
        wb(
            m,
            a0.wrapping_add(OB_TIME_FRAME),
            if s16(intervalo) < 0 {
                0
            } else {
                intervalo.to_be_bytes()[0]
            },
        );
        let render = rb(m, a0.wrapping_add(OB_RENDER)) & !RENDER_FLIPS;
        wb(
            m,
            a0.wrapping_add(OB_RENDER),
            render | (status & STATUS_XFLIP),
        );
        if velocidade >= 0x600 {
            SONANI_ROLL2
        } else {
            SONANI_ROLL
        }
    } else {
        let intervalo = 0x800u16.wrapping_sub(velocidade);
        wb(
            m,
            a0.wrapping_add(OB_TIME_FRAME),
            if s16(intervalo) < 0 {
                0
            } else {
                (intervalo >> 6).to_be_bytes()[1]
            },
        );
        let render = rb(m, a0.wrapping_add(OB_RENDER)) & !RENDER_FLIPS;
        wb(
            m,
            a0.wrapping_add(OB_RENDER),
            render | (status & STATUS_XFLIP),
        );
        SONANI_PUSH
    };
    carregar_quadro(m, a0, roteiro)?;
    let quadro = rb(m, a0.wrapping_add(OB_FRAME)).wrapping_add(deslocamento);
    wb(m, a0.wrapping_add(OB_FRAME), quadro);
    Ok(())
}

/// `.loadframe`: o próximo quadro do roteiro, com os códigos de fim `$FF`, `$FE` e `$FD`.
fn carregar_quadro(m: &mut Memoria, a0: u32, roteiro: u32) -> Result<(), ErroAbertura> {
    let d1 = u32::from(rb(m, a0.wrapping_add(OB_ANI_FRAME)));
    let mut d0 = rom_b(m, roteiro.wrapping_add(1).wrapping_add(d1));
    if d0 & 0x80 != 0 {
        d0 = d0.wrapping_add(1);
        if d0 == 0 {
            // `$FF`: recomeça.
            wb(m, a0.wrapping_add(OB_ANI_FRAME), 0);
            d0 = rom_b(m, roteiro.wrapping_add(1));
        } else {
            d0 = d0.wrapping_add(1);
            if d0 == 0 {
                // `$FE`: volta tantos quadros.
                let volta = rom_b(m, roteiro.wrapping_add(2).wrapping_add(d1));
                let atual = rb(m, a0.wrapping_add(OB_ANI_FRAME));
                wb(m, a0.wrapping_add(OB_ANI_FRAME), atual.wrapping_sub(volta));
                let d1 = d1.wrapping_sub(u32::from(volta));
                d0 = rom_b(m, roteiro.wrapping_add(1).wrapping_add(d1));
            } else {
                d0 = d0.wrapping_add(1);
                if d0 == 0 {
                    // `$FD`: troca de animação.
                    let nova = rom_b(m, roteiro.wrapping_add(2).wrapping_add(d1));
                    wb(m, a0.wrapping_add(OB_ANIM), nova);
                }
                return Ok(());
            }
        }
    }
    wb(m, a0.wrapping_add(OB_FRAME), d0);
    let q = rb(m, a0.wrapping_add(OB_ANI_FRAME)).wrapping_add(1);
    wb(m, a0.wrapping_add(OB_ANI_FRAME), q);
    Ok(())
}

/// `Sonic_LoadGfx`: quando o quadro mudou, monta no buffer a arte dos blocos que o `SonicDynPLC`
/// pede, e liga a flag para o VBlank transferir.
pub(super) fn sonic_load_gfx(m: &mut Memoria, a0: u32) {
    let quadro = rb(m, a0.wrapping_add(OB_FRAME));
    if quadro == rb(m, V_SONFRAMENUM) {
        return;
    }
    wb(m, V_SONFRAMENUM, quadro);
    let desloc = rom_w(
        m,
        SONIC_DYN_PLC.wrapping_add(u32::from(quadro).wrapping_mul(2)),
    );
    let mut a2 = SONIC_DYN_PLC.wrapping_add(ext_l(desloc));
    let entradas = rom_b(m, a2);
    a2 = a2.wrapping_add(1);
    if entradas == 0 {
        return;
    }
    let mut a3 = V_SGFX_BUFFER;
    wb(m, F_SONFRAMECHG, 1);
    for _ in 0..entradas {
        let b0 = rom_b(m, a2);
        let b1 = rom_b(m, a2.wrapping_add(1));
        a2 = a2.wrapping_add(2);
        let tiles = u32::from(b0 >> 4).wrapping_add(1);
        let palavra = u16::from_be_bytes([b0, b1]).wrapping_shl(5);
        let mut origem = ART_SONIC.wrapping_add(u32::from(palavra));
        for _ in 0..tiles {
            for k in 0..8u32 {
                let v = rom_l(m, origem.wrapping_add(k.wrapping_mul(4)));
                wl(m, a3.wrapping_add(k.wrapping_mul(4)), v);
            }
            origem = origem.wrapping_add(32);
            a3 = a3.wrapping_add(32);
        }
    }
}

#[cfg(test)]
mod testes_contato {
    use super::*;

    #[test]
    fn morte_respeita_limite_e_abre_game_over_ou_time_over() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        ww(&mut m, V_LIMITBTM2, 0x300);
        ww(&mut m, V_PLAYER + OB_Y, 0x400);
        wb(&mut m, V_LIVES, 1);
        sonic_handle_death(&mut m, V_PLAYER).unwrap();
        assert_eq!(rb(&mut m, V_LIVES), 1);
        ww(&mut m, V_PLAYER + OB_Y, 0x401);
        sonic_handle_death(&mut m, V_PLAYER).unwrap();
        assert_eq!(rb(&mut m, V_LIVES), 0);
        assert_eq!(rb(&mut m, 0xFF_D080), 0x39);
        assert_eq!(rb(&mut m, 0xFF_D0C0 + OB_FRAME), 1);
        assert_eq!(rb(&mut m, V_SOUNDQUEUE0), 0x8F);

        let mut m = Memoria::nova(vec![0; 0x20000]);
        ww(&mut m, V_PLAYER + OB_Y, 0x101);
        wb(&mut m, V_LIVES, 2);
        wb(&mut m, F_TIMEOVER, 1);
        sonic_handle_death(&mut m, V_PLAYER).unwrap();
        assert_eq!(rb(&mut m, V_LIVES), 1);
        assert_eq!(rb(&mut m, 0xFF_D080 + OB_FRAME), 2);
        assert_eq!(rb(&mut m, 0xFF_D0C0 + OB_FRAME), 3);
        assert_eq!(rw(&mut m, V_PLAYER + SON_RESTARTIME), 0);
    }

    #[test]
    fn ataque_ao_chefe_rebate_sonic_e_reduz_vida() {
        let mut rom = vec![0; 0x20000];
        let indice = usize::try_from(REACT_SIZES).unwrap().wrapping_add(28);
        rom[indice..indice + 2].copy_from_slice(&[24, 24]);
        let mut m = Memoria::nova(rom);
        ww(&mut m, V_PLAYER + OB_X, 100);
        ww(&mut m, V_PLAYER + OB_Y, 100);
        wb(&mut m, V_PLAYER + OB_HEIGHT, 19);
        wb(&mut m, V_PLAYER + OB_ANIM, ANIM_ROLL);
        ww(&mut m, V_PLAYER + OB_VELX, 0x200);
        ww(&mut m, V_PLAYER + OB_VELY, 0x400);
        wb(&mut m, V_LVLOBJSPACE + OB_RENDER, 0x80);
        wb(&mut m, V_LVLOBJSPACE + OB_COLTYPE, 0x0F);
        wb(&mut m, V_LVLOBJSPACE + 0x21, 1);
        ww(&mut m, V_LVLOBJSPACE + OB_X, 100);
        ww(&mut m, V_LVLOBJSPACE + OB_Y, 100);
        react_to_item(&mut m).unwrap();
        assert_eq!(rw(&mut m, V_PLAYER + OB_VELX), 0xFF00);
        assert_eq!(rw(&mut m, V_PLAYER + OB_VELY), 0xFE00);
        assert_eq!(rb(&mut m, V_LVLOBJSPACE + OB_COLTYPE), 0);
        assert_eq!(rb(&mut m, V_LVLOBJSPACE + 0x21), 0);
        assert_ne!(rb(&mut m, V_LVLOBJSPACE + OB_STATUS) & 0x80, 0);
    }

    #[test]
    fn contato_considera_largura_e_limites() {
        assert!(retangulos_sobrepoem(116, 100, 6, 16));
        assert!(!retangulos_sobrepoem(117, 100, 6, 16));
        assert!(retangulos_sobrepoem(90, 100, 6, 16));
        assert!(retangulos_sobrepoem(88, 100, 6, 16));
        assert!(!retangulos_sobrepoem(87, 100, 6, 16));
    }

    #[test]
    fn objeto_visivel_distante_nao_interrompe_e_anel_proximo_e_coletado() {
        let mut rom = vec![0; 0x20000];
        let indice = usize::try_from(REACT_SIZES).unwrap() + 12;
        rom[indice..indice + 2].copy_from_slice(&[6, 6]);
        let mut m = Memoria::nova(rom);
        ww(&mut m, V_PLAYER + OB_X, 100);
        ww(&mut m, V_PLAYER + OB_Y, 100);
        wb(&mut m, V_PLAYER + OB_HEIGHT, 19);
        wb(&mut m, V_LVLOBJSPACE + OB_RENDER, 0x80);
        wb(&mut m, V_LVLOBJSPACE + OB_COLTYPE, 0x47);
        wb(&mut m, V_LVLOBJSPACE + OB_ROUTINE, 2);
        ww(&mut m, V_LVLOBJSPACE + OB_X, 180);
        ww(&mut m, V_LVLOBJSPACE + OB_Y, 100);
        react_to_item(&mut m).unwrap();
        assert_eq!(rb(&mut m, V_LVLOBJSPACE + OB_ROUTINE), 2);
        ww(&mut m, V_LVLOBJSPACE + OB_X, 100);
        react_to_item(&mut m).unwrap();
        assert_eq!(rb(&mut m, V_LVLOBJSPACE + OB_ROUTINE), 4);
    }

    #[test]
    fn aceleracao_e_freio_preservam_sinal_e_limite() {
        let mut m = Memoria::nova(vec![]);
        ww(&mut m, V_SONSPEEDACC, 12);
        ww(&mut m, V_SONSPEEDDEC, 128);
        ww(&mut m, V_SONSPEEDMAX, 0x600);
        sonic_move_horizontal(&mut m, V_PLAYER, true);
        assert_eq!(rw(&mut m, V_PLAYER + OB_INERTIA), 12);
        sonic_move_horizontal(&mut m, V_PLAYER, false);
        assert_eq!(rw(&mut m, V_PLAYER + OB_INERTIA), 0xFF80);
        sonic_move_horizontal(&mut m, V_PLAYER, false);
        assert_eq!(rw(&mut m, V_PLAYER + OB_INERTIA), 0xFF74);
        ww(&mut m, V_PLAYER + OB_INERTIA, 0x05FC);
        sonic_move_horizontal(&mut m, V_PLAYER, true);
        assert_eq!(rw(&mut m, V_PLAYER + OB_INERTIA), 0x600);
    }

    #[test]
    fn pulo_em_piso_livre_aplica_impulso_e_troca_hitbox() {
        let mut rom = vec![0; 0x80000];
        let seno = usize::try_from(SINE_DATA + 0x180).unwrap();
        rom[seno..seno + 2].copy_from_slice(&(-256i16).to_be_bytes());
        let mut m = Memoria::nova(rom);
        ww(&mut m, V_PLAYER + OB_X, 100);
        ww(&mut m, V_PLAYER + OB_Y, 0x300);
        wb(&mut m, V_PLAYER + OB_HEIGHT, 19);
        wb(&mut m, V_PLAYER + OB_WIDTH, 9);
        wb(&mut m, V_JPADPRESS2, 0x10);
        assert!(sonic_jump(&mut m, V_PLAYER).unwrap());
        assert_eq!(rw(&mut m, V_PLAYER + OB_VELY), 0xF980);
        assert_eq!(rw(&mut m, V_PLAYER + OB_Y), 0x305);
        assert_eq!(rb(&mut m, V_PLAYER + OB_HEIGHT), 14);
        assert_eq!(rb(&mut m, V_PLAYER + OB_WIDTH), 7);
        assert_eq!(rb(&mut m, V_PLAYER + OB_STATUS) & 6, 6);
    }

    #[test]
    fn pulo_aceita_paredes_e_teto_quando_ha_espaco() {
        for angulo in [0x40, 0x80, 0xC0] {
            let mut m = Memoria::nova(vec![0; 0x80000]);
            ww(&mut m, V_PLAYER + OB_X, 0x300);
            ww(&mut m, V_PLAYER + OB_Y, 0x300);
            wb(&mut m, V_PLAYER + OB_HEIGHT, 19);
            wb(&mut m, V_PLAYER + OB_WIDTH, 9);
            wb(&mut m, V_PLAYER + OB_ANGLE, angulo);
            wb(&mut m, V_JPADPRESS2, 0x10);
            assert!(sonic_jump(&mut m, V_PLAYER).unwrap());
            assert_ne!(rb(&mut m, V_PLAYER + OB_STATUS) & STATUS_IN_AIR, 0);
        }
    }
}
