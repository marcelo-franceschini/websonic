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
const STATUS_XFLIP: u8 = 1;
const STATUS_IN_AIR: u8 = 2;
const STATUS_ROLLING: u8 = 4;
const STATUS_ON_OBJECT: u8 = 8;
const STATUS_PUSHING: u8 = 0x20;
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
fn calc_sine(m: &Memoria, angulo: u8) -> (i16, i16) {
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

/// `ObjFloorDist`: a distância do pé do objeto ao piso (`d1`).
fn obj_floor_dist(m: &mut Memoria, a0: u32) -> u16 {
    let d3 = rw(m, a0.wrapping_add(OB_X));
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
    find_floor(m, b, d2, d3)
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

/// `Sonic_AnglePos`: gruda o Sonic no piso. Só o modo de piso existe aqui.
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
    if d0 & 0xC0 != 0 {
        return Err(ErroAbertura::NaoImplementado("Sonic na parede ou no teto"));
    }
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
    if distancia <= 0xE {
        ww(m, a0.wrapping_add(OB_Y), y.wrapping_add(d1));
        return Ok(());
    }
    if rb(m, a0.wrapping_add(SON_STICKTOCONVEX)) != 0 {
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

/// `Sonic_Jump`: sem botão de pulo não faz nada.
fn sonic_jump(m: &mut Memoria) -> Result<(), ErroAbertura> {
    if rb(m, V_JPADPRESS2) & 0x70 != 0 {
        return Err(ErroAbertura::NaoImplementado("pulo do Sonic"));
    }
    Ok(())
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

/// `Sonic_Move`: sem botões, só a rolagem do olhar e a animação parada.
fn sonic_move(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let d5 = rw(m, V_SONSPEEDACC);
    if rb(m, F_SLIDEMODE) != 0 {
        return sonic_angle_speed(m, a0);
    }
    if rw(m, a0.wrapping_add(SON_LOCKTIME)) != 0 {
        return sonic_reset_scr(m, a0, d5);
    }
    if rb(m, V_JPADHOLD2) & 0x0C != 0 {
        return Err(ErroAbertura::NaoImplementado("Sonic andando"));
    }
    let angulo = rb(m, a0.wrapping_add(OB_ANGLE));
    if angulo.wrapping_add(0x20) & 0xC0 != 0 || rw(m, a0.wrapping_add(OB_INERTIA)) != 0 {
        return sonic_reset_scr(m, a0, d5);
    }
    let status = rb(m, a0.wrapping_add(OB_STATUS)) & !STATUS_PUSHING;
    wb(m, a0.wrapping_add(OB_STATUS), status);
    wb(m, a0.wrapping_add(OB_ANIM), ANIM_WAIT);
    if status & STATUS_ON_OBJECT != 0 {
        return Err(ErroAbertura::NaoImplementado(
            "Sonic parado em cima de objeto",
        ));
    }
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
    // `Sonic_LookUp` e `Sonic_Duck`: com os botões zerados nenhum dos dois age.
    if rb(m, V_JPADHOLD2) & 0x03 != 0 {
        return Err(ErroAbertura::NaoImplementado(
            "Sonic olhando para cima ou agachando",
        ));
    }
    sonic_reset_scr(m, a0, d5)
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
    // `Sonic_WallSpeedAdjust`: no chão plano e parado não há o que ajustar.
    if angulo.wrapping_add(0x40) & 0x80 != 0 || inercia == 0 {
        return Ok(());
    }
    Err(ErroAbertura::NaoImplementado("Sonic contra a parede"))
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
fn speed_to_pos(m: &mut Memoria, a0: u32) {
    let x = rl(m, a0.wrapping_add(OB_X));
    let y = rl(m, a0.wrapping_add(OB_Y));
    let vx = ext_l(rw(m, a0.wrapping_add(OB_VELX))).wrapping_shl(8);
    let vy = ext_l(rw(m, a0.wrapping_add(OB_VELY))).wrapping_shl(8);
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
        _ => Err(ErroAbertura::NaoImplementado("rotina do Sonic")),
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
                sonic_jump(m)?;
                sonic_slope_resist_walk(m, a0);
                sonic_move(m, a0)?;
                sonic_roll(m, a0)?;
                sonic_level_bound(m, a0)?;
                speed_to_pos(m, a0);
                sonic_angle_pos(m, a0)?;
                sonic_slope_repel(m, a0);
            }
            _ => return Err(ErroAbertura::NaoImplementado("Sonic no ar ou rolando")),
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

/// `Sonic_Display`: mostra o sprite. Piscar, invencibilidade e tênis exigem o tempo de cada um.
fn sonic_display(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    if rw(m, a0.wrapping_add(SON_FLASHTIME)) != 0 {
        return Err(ErroAbertura::NaoImplementado("Sonic piscando"));
    }
    display_sprite(m, a0);
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

/// `ReactToItem`: procura objetos que acabaram de aparecer na tela e têm colisão. No primeiro
/// quadro nenhum objeto foi desenhado ainda; se algum já tiver sido, o núcleo para.
fn react_to_item(m: &mut Memoria) -> Result<(), ErroAbertura> {
    let mut a1 = V_LVLOBJSPACE;
    while a1 < V_LVLOBJEND {
        if rb(m, a1.wrapping_add(OB_RENDER)) & 0x80 != 0 && rb(m, a1.wrapping_add(OB_COLTYPE)) != 0
        {
            return Err(ErroAbertura::NaoImplementado("colisão do Sonic com objeto"));
        }
        a1 = a1.wrapping_add(OBJECT_SIZE);
    }
    Ok(())
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
fn sonic_animate(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
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
    Err(ErroAbertura::NaoImplementado(
        "animação de andar, correr ou rolar",
    ))
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
fn sonic_load_gfx(m: &mut Memoria, a0: u32) {
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
