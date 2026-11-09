//! Tela da Sega (sub-etapa 7.3): `GM_Sega`, `PalCycle_Sega`, `PaletteFadeOut`, `ClearScreen`,
//! `PalLoad`, `TilemapToVRAM`. Cada função cita o rótulo que reproduz. Tabelas e arte vêm da
//! ROM do jogador pelos endereços dos rótulos.

use super::boot::{fill_vram, rom_l, rom_w};
use super::ram::*;
use super::{ErroAbertura, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;
use crate::rotinas;

fn rotina(e: rotinas::ErroRotina) -> ErroAbertura {
    ErroAbertura::Rotina(e)
}

/// `QueueSound2` com o comando de parar a música; `ClearPLC`; começo de `PaletteFadeOut`.
pub fn gm_sega_inicio(m: &mut Memoria) {
    wb(m, V_SOUNDQUEUE1, BGM_STOP);
    super::plc::clear_plc(m);
    ww(m, V_PFADE_START, 0x003F);
    wb(m, V_VBLANK_ROUTINE, super::vblank::ROTINA_PALETTE_FADE);
}

/// `FadeOut_DecColor`: apaga um passo de um canal da cor em `a`. Devolve o endereço seguinte.
fn fade_out_dec_color(m: &mut Memoria, a: u32) -> u32 {
    let cor = rw(m, a);
    if cor == 0 {
        return a.wrapping_add(2);
    }
    let passo = if cor & 0x00E != 0 {
        0x002
    } else if cor & 0x0E0 != 0 {
        0x020
    } else if cor & 0xE00 != 0 {
        0x200
    } else {
        0
    };
    ww(m, a, cor.wrapping_sub(passo));
    a.wrapping_add(2)
}

/// `FadeOut_ToBlack`: um passo do desvanecimento da paleta normal e da de água.
pub fn fade_out_to_black(m: &mut Memoria) {
    let inicio = u32::from(rb(m, V_PFADE_START));
    let tamanho = u32::from(rb(m, V_PFADE_START.wrapping_add(1)));
    for base in [V_PALETTE, V_PALETTE_WATER] {
        let mut a = base.wrapping_add(inicio);
        for _ in 0..=tamanho {
            a = fade_out_dec_color(m, a);
        }
    }
}

/// `GM_Sega` depois do desvanecimento: registradores, limpeza, arte, mapas e paleta.
/// Termina com a exibição ligada, no rótulo `Sega_WaitPal`.
pub fn gm_sega_preparo(m: &mut Memoria) -> Result<(), ErroAbertura> {
    for w in [0x8004u16, 0x8230, 0x8407, 0x8700, 0x8B00] {
        ww(m, VDP_CONTROLE, w);
    }
    wb(m, F_WTR_STATE, 0);
    // disable_display: derruba o bit 6 do registrador 1 guardado.
    let b = rw(m, V_VDP_BUFFER1);
    ww(m, VDP_CONTROLE, (b & 0xFF00) | (b & 0xBF));
    clear_screen(m)?;
    wl(m, VDP_CONTROLE, 0x4000_0000);
    rotinas::nem_dec(m, NEM_SEGA_LOGO).map_err(rotina)?;
    rotinas::eni_dec(m, ENI_SEGA_LOGO, V_RAM_START, 0).map_err(rotina)?;
    copy_tilemap(m, V_RAM_START, 0xE000 + 0x510, 24, 8);
    copy_tilemap(m, V_RAM_START + 24 * 8 * 2, 0xC000, 40, 28);
    // REV01: console japonês esconde o "TM" com um retângulo branco.
    if rb(m, V_MEGADRIVE) & 0x80 == 0 {
        copy_tilemap(m, V_RAM_START + 0xA40, 0xC000 + 0x53A, 3, 2);
    }
    pal_load(m, 0);
    ww(m, V_PCYC_NUM, 0xFFF6);
    ww(m, V_PCYC_TIME, 0);
    // enable_display
    let b = rw(m, V_VDP_BUFFER1);
    ww(m, VDP_CONTROLE, (b & 0xFF00) | (b & 0xFF) | 0x40);
    Ok(())
}

/// `ClearScreen`: planos A e B, rolagem e os buffers de sprites e de rolagem.
pub fn clear_screen(m: &mut Memoria) -> Result<(), ErroAbertura> {
    fill_vram(m, 0, 0xC000, 0xC000 + 0x1000)?;
    fill_vram(m, 0, 0xE000, 0xE000 + 0x1000)?;
    wl(m, V_SCRPOSY_VDP, 0);
    wl(m, V_SCRPOSX_VDP, 0);
    // REV01 sem FixBugs: cada limpeza passa uma longa do fim do buffer.
    for (ini, fim) in [
        (V_SPRITETABLEBUFFER, V_SPRITETABLEBUFFER_END.wrapping_add(4)),
        (
            V_HSCROLLTABLEBUFFER,
            V_HSCROLLTABLEBUFFER_END_PADDED.wrapping_add(4),
        ),
    ] {
        let mut a = ini;
        while a < fim {
            wl(m, a, 0);
            a = a.wrapping_add(4);
        }
    }
    Ok(())
}

/// `TilemapToVRAM` com a macro `copyTilemap`: copia `largura` por `altura` palavras de `fonte`.
pub fn copy_tilemap(m: &mut Memoria, fonte: u32, destino: u32, largura: u32, altura: u32) {
    let mut d0 = 0x4000_0000u32
        .wrapping_add((destino & 0x3FFF) << 16)
        .wrapping_add((destino & 0xC000) >> 14);
    let mut a1 = fonte;
    for _ in 0..altura {
        wl(m, VDP_CONTROLE, d0);
        for _ in 0..largura {
            let w = rw(m, a1);
            ww(m, VDP_DADOS, w);
            a1 = a1.wrapping_add(2);
        }
        d0 = d0.wrapping_add(0x80_0000);
    }
}

/// `PalLoad`: copia a paleta `id` da tabela `Pal_Index` direto para a paleta ativa.
pub fn pal_load(m: &mut Memoria, id: u32) {
    pal_load_para(m, id, 0);
}

/// `PalLoad` e `PalLoad_Fade`: `soma` desloca o destino (`$80` carrega no buffer do fade-in).
pub fn pal_load_para(m: &mut Memoria, id: u32, soma: u32) {
    let e = PAL_INDEX.wrapping_add(id.wrapping_mul(8));
    let mut a2 = rom_l(m, e);
    let alvo = rom_w(m, e.wrapping_add(4));
    // `movea.w` estende o sinal: o endereço fica na RAM alta.
    let mut a3 = (u32::from(alvo) | 0xFFFF_0000).wrapping_add(soma);
    let n = rom_w(m, e.wrapping_add(6));
    for _ in 0..=u32::from(n) {
        let l = rom_l(m, a2);
        wl(m, a3, l);
        a2 = a2.wrapping_add(4);
        a3 = a3.wrapping_add(4);
    }
}

fn i16_de(w: u16) -> i32 {
    i32::from(i16::from_be_bytes(w.to_be_bytes()))
}

/// `PalCycle_Sega`. Devolve `true` enquanto o efeito continua (o original devolve Z limpo).
pub fn pal_cycle_sega(m: &mut Memoria) -> bool {
    if rb(m, V_PCYC_TIME.wrapping_add(1)) != 0 {
        return pcyc_sega_fade_in(m);
    }
    let a1 = V_PALETTE_LINE_2;
    let mut a0 = PAL_SEGA1;
    let n = PAL_SEGA1_END.wrapping_sub(PAL_SEGA1) / 2;
    let mut d1 = i32::try_from(n).unwrap_or(0).wrapping_sub(1);
    let mut d0 = i16_de(rw(m, V_PCYC_NUM));
    // `.findScanStart`: pula as cores que ainda não entram (posição negativa).
    while d0 < 0 {
        a0 = a0.wrapping_add(2);
        d1 = d1.wrapping_sub(1);
        d0 = d0.wrapping_add(2);
    }
    // `.doLightScan`
    loop {
        if d0 & 0x1E == 0 {
            d0 = d0.wrapping_add(2);
        }
        if d0 < 0x60 {
            let w = rom_w(m, a0);
            ww(m, a1.wrapping_add(u32::try_from(d0).unwrap_or(0)), w);
            a0 = a0.wrapping_add(2);
        }
        d0 = d0.wrapping_add(2);
        d1 = d1.wrapping_sub(1);
        if d1 == -1 {
            break;
        }
    }
    let mut d0 = i16_de(rw(m, V_PCYC_NUM)).wrapping_add(2);
    if d0 & 0x1E == 0 {
        d0 = d0.wrapping_add(2);
    }
    if d0 >= 0x64 {
        ww(m, V_PCYC_TIME, 0x0401);
        d0 = -12;
    }
    ww(m, V_PCYC_NUM, u16::try_from(d0 & 0xFFFF).unwrap_or(0));
    true
}

/// `PCycSega_FadeIn`: a segunda parte do ciclo, o aparecimento.
fn pcyc_sega_fade_in(m: &mut Memoria) -> bool {
    let t = rb(m, V_PCYC_TIME).wrapping_sub(1);
    wb(m, V_PCYC_TIME, t);
    if t & 0x80 == 0 {
        return true;
    }
    wb(m, V_PCYC_TIME, 4);
    let d0 = rw(m, V_PCYC_NUM).wrapping_add(12);
    if d0 >= 48 {
        return false;
    }
    ww(m, V_PCYC_NUM, d0);
    let mut a0 = PAL_SEGA2.wrapping_add(u32::from(d0));
    let mut a1 = V_PALETTE_LINE_1.wrapping_add(4);
    for _ in 0..2 {
        let l = rom_l(m, a0);
        wl(m, a1, l);
        a0 = a0.wrapping_add(4);
        a1 = a1.wrapping_add(4);
    }
    let w = rom_w(m, a0);
    ww(m, a1, w);
    a0 = a0.wrapping_add(2);
    // `a0` fica na sexta cor: ela preenche o resto das linhas 2 a 4, sem a cor transparente.
    let mut d0: u32 = 0;
    for _ in 0..=0x2C {
        if d0 & 0x1E == 0 {
            d0 = d0.wrapping_add(2);
        }
        let c = rom_w(m, a0);
        ww(m, V_PALETTE_LINE_2.wrapping_add(d0), c);
        d0 = d0.wrapping_add(2);
    }
    true
}
