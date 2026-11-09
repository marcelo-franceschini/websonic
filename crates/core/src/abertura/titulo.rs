//! Tela do título (sub-etapa 7.5). Cada função cita o rótulo que reproduz.

use super::boot::rom_w;
use super::cenario;
use super::objetos::{build_sprites, execute_objects};
use super::ram::*;
use super::sega::{clear_screen, copy_tilemap, pal_load_para};
use super::{ErroAbertura, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;
use crate::rotinas;

fn rot(e: rotinas::ErroRotina) -> ErroAbertura {
    ErroAbertura::Rotina(e)
}

/// `locVRAM` para o endereço de uma tile (`tile` vezes o tamanho de 32 bytes).
fn loc_vram(tile: u32) -> u32 {
    let a = tile.wrapping_mul(32);
    0x4000_0000u32
        .wrapping_add((a & 0x3FFF) << 16)
        .wrapping_add((a & 0xC000) >> 14)
}

fn zerar(m: &mut Memoria, ini: u32, fim: u32) {
    let mut a = ini;
    while a < fim {
        wl(m, a, 0);
        a = a.wrapping_add(4);
    }
}

/// `GM_Title`, da saída do primeiro desvanecimento até a chamada de `PaletteFadeIn`:
/// carrega o programa do Z80, a arte e os mapas do "SONIC TEAM PRESENTS" e cria o objeto.
pub fn gm_title_preparo_a(m: &mut Memoria) -> Result<(), ErroAbertura> {
    super::boot::dac_driver_load(m)?;
    for w in [0x8004u16, 0x8230, 0x8407, 0x9001, 0x9200, 0x8B03, 0x8720] {
        ww(m, VDP_CONTROLE, w);
    }
    wb(m, F_WTR_STATE, 0);
    clear_screen(m)?;
    zerar(m, V_OBJSPACE, V_OBJSPACE_END);
    wl(m, VDP_CONTROLE, loc_vram(0));
    rotinas::nem_dec(m, NEM_JAP_NAMES).map_err(rot)?;
    wl(
        m,
        VDP_CONTROLE,
        loc_vram(u32::from(ARTTILE_SONIC_TEAM_FONT)),
    );
    rotinas::nem_dec(m, NEM_CREDIT_TEXT).map_err(rot)?;
    rotinas::eni_dec(m, ENI_JAP_NAMES, V_RAM_START, 0).map_err(rot)?;
    copy_tilemap(m, V_RAM_START, 0xC000, 40, 28);
    zerar(m, V_PALETTE_FADING, V_PALETTE_FADING_END);
    pal_load_para(m, 3, 0x80);
    wb(m, V_SONICTEAM.wrapping_add(OB_ID), ID_CREDITS_TEXT);
    execute_objects(m)?;
    build_sprites(m)
}

/// `PaletteFadeIn` (começo): pinta de preto a paleta ativa e pede o primeiro VBlank do fade.
pub fn palette_fade_in_inicio(m: &mut Memoria) {
    ww(m, V_PFADE_START, 0x003F);
    let inicio = u32::from(rb(m, V_PFADE_START));
    let n = u32::from(rb(m, V_PFADE_START.wrapping_add(1)));
    let mut a0 = V_PALETTE.wrapping_add(inicio);
    for _ in 0..=n {
        ww(m, a0, 0);
        a0 = a0.wrapping_add(2);
    }
    wb(m, V_VBLANK_ROUTINE, super::vblank::ROTINA_PALETTE_FADE);
}

/// Parte B do título, fatia 1: do fim do aparecimento do "SONIC TEAM PRESENTS" até a
/// `PaletteFadeOut` seguinte (linhas 1952 a 1995 de `GM_Title`). Carrega as artes do título na
/// VRAM, a fonte do menu, as variáveis de sessão e o cenário da GHZ (tamanhos, deformação, blocos
/// e layout). O `RunPLC` de dentro do desvanecimento não faz nada: a fila de arte está vazia.
pub fn parte_b1(m: &mut Memoria) -> Result<(), ErroAbertura> {
    for (tile, origem) in [
        (ARTTILE_TITLE_FOREGROUND, NEM_TITLE_FG),
        (ARTTILE_TITLE_SONIC, NEM_TITLE_SONIC),
        (ARTTILE_TITLE_TRADEMARK, NEM_TITLE_TM),
    ] {
        wl(m, VDP_CONTROLE, loc_vram(u32::from(tile)));
        rotinas::nem_dec(m, origem).map_err(rot)?;
    }
    // `Art_Text`: fonte do menu de fases, sem compressão, uma palavra por vez na porta de dados.
    wl(
        m,
        VDP_CONTROLE,
        loc_vram(u32::from(ARTTILE_LEVEL_SELECT_FONT)),
    );
    let mut a = ART_TEXT;
    while a < ART_TEXT_END {
        let w = rom_w(m, a);
        ww(m, VDP_DADOS, w);
        a = a.wrapping_add(2);
    }
    // Variáveis de sessão. O `UnusedOptimization` da referência é 0, então `v_unused2` também.
    wb(m, V_LASTLAMP, 0);
    ww(m, V_DEBUGUSE, 0);
    ww(m, F_DEMO, 0);
    ww(m, V_UNUSED2, 0);
    // `id_GHZ_act1` é zero: a fase é a GHZ1.
    ww(m, V_ZONE_ACT, 0);
    ww(m, V_PCYC_TIME, 0);
    cenario::level_size_load(m)?;
    cenario::deform_layers(m)?;
    // Blocos 16x16 (EniDec, tile inicial 0) e blocos 256x256 (KosDec) da GHZ.
    rotinas::eni_dec(m, BLK16_GHZ, V_16X16, 0).map_err(rot)?;
    rotinas::kos_dec(m, BLK256_GHZ, V_256X256).map_err(rot)?;
    cenario::level_layout_load(m)
}

/// Começo de `PaletteFadeOut`: a paleta inteira (a partir de 0) e o pedido do primeiro VBlank do
/// desvanecimento. Os passos de cada VBlank ficam em [`super::Abertura`].
pub fn palette_fade_out_inicio(m: &mut Memoria) {
    ww(m, V_PFADE_START, 0x003F);
    wb(m, V_VBLANK_ROUTINE, super::vblank::ROTINA_PALETTE_FADE);
}

/// `FadeIn_AddColor`: sobe um passo de um canal da cor em `a0` rumo à cor-alvo em `a1`.
fn fade_in_add_color(m: &mut Memoria, a0: u32, a1: u32) -> (u32, u32) {
    let alvo = rw(m, a1);
    let atual = rw(m, a0);
    let a1 = a1.wrapping_add(2);
    if alvo == atual {
        return (a0.wrapping_add(2), a1);
    }
    let azul = atual.wrapping_add(0x200);
    if azul <= alvo {
        ww(m, a0, azul);
        return (a0.wrapping_add(2), a1);
    }
    let verde = atual.wrapping_add(0x020);
    if verde <= alvo {
        ww(m, a0, verde);
        return (a0.wrapping_add(2), a1);
    }
    ww(m, a0, atual.wrapping_add(2));
    (a0.wrapping_add(2), a1)
}

/// `FadeIn_FromBlack`: um passo do aparecimento da paleta (e da de água na Labyrinth).
pub fn fade_in_from_black(m: &mut Memoria) {
    let inicio = u32::from(rb(m, V_PFADE_START));
    let n = u32::from(rb(m, V_PFADE_START.wrapping_add(1)));
    let mut a0 = V_PALETTE.wrapping_add(inicio);
    let mut a1 = V_PALETTE_FADING.wrapping_add(inicio);
    for _ in 0..=n {
        (a0, a1) = fade_in_add_color(m, a0, a1);
    }
    if rb(m, V_ZONE) == 1 {
        let mut a0 = V_PALETTE_WATER.wrapping_add(inicio);
        let mut a1 = V_PALETTE_WATER_FADING.wrapping_add(inicio);
        for _ in 0..=n {
            (a0, a1) = fade_in_add_color(m, a0, a1);
        }
    }
}
