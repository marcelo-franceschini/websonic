//! Tratador de VBlank (rótulos `VBlank`, `VBlank_Lag`, `VBlank_Sega`, `VBlank_SegaPCM`,
//! `VBlank_PaletteFade`, `VBlank_StandardTransfers`) e leitura dos controles (`ReadJoypads`).
//!
//! Só as rotinas do índice de VBlank usadas no trecho já implementado. As outras devolvem
//! [`ErroAbertura::NaoImplementado`].

use super::boot::esperar_z80;
use super::ram::*;
use super::{ErroAbertura, rb, rl, rw, wb, wl, ww};
use crate::memoria::{Memoria, Tamanho};
use crate::som::driver::Driver;

/// Índice de `v_vblank_routine` (já mascarado com `$3E`).
pub const ROTINA_SEGA: u8 = 0x02;
pub const ROTINA_PALETTE_FADE: u8 = 0x12;
/// Índice de `VBlank_Title` (quadro da tela do título).
pub const ROTINA_TITULO: u8 = 0x04;
/// Índice de `VBlank_TitleCards` (cartões de título da fase; também `VBlank_Ending`).
pub const ROTINA_CARTOES: u8 = 0x0C;
/// Índice de `VBlank_Levels` (a fase e a demo).
pub const ROTINA_NIVEL: u8 = 0x08;
pub const ROTINA_SEGA_PCM: u8 = 0x14;

/// Macros `writeCRAM` e `writeVRAM`: DMA do 68000 para a VDP. `cram` escolhe o tipo de destino.
fn dma_para_vdp(m: &mut Memoria, fonte: u32, bytes: u32, destino: u32, cram: bool) {
    let n = bytes >> 1;
    wl(
        m,
        VDP_CONTROLE,
        0x9400_9300 | ((n & 0xFF00) << 8) | (n & 0xFF),
    );
    let f = fonte >> 1;
    wl(
        m,
        VDP_CONTROLE,
        0x9600_9500 | ((f & 0xFF00) << 8) | (f & 0xFF),
    );
    ww(
        m,
        VDP_CONTROLE,
        0x9700 | u16::try_from((f & 0xFF_0000) >> 16).unwrap_or(0) & 0x7F,
    );
    let base: u16 = if cram { 0xC000 } else { 0x4000 };
    ww(
        m,
        VDP_CONTROLE,
        base.wrapping_add(u16::try_from(destino & 0x3FFF).unwrap_or(0)),
    );
    let alto = 0x80u16.wrapping_add(u16::try_from((destino & 0xC000) >> 14).unwrap_or(0));
    ww(m, V_VDP_BUFFER2, alto);
    let v = rw(m, V_VDP_BUFFER2);
    ww(m, VDP_CONTROLE, v);
}

/// `ReadJoypads`: lê os dois controles e atualiza os bytes de pressionado e de apertado agora.
pub fn read_joypads(m: &mut Memoria) {
    let mut a0 = V_JPADHOLD1;
    for porta in [0xA1_0003u32, 0xA1_0005] {
        wb(m, porta, 0);
        let mut d0 = rb(m, porta).wrapping_shl(2) & 0xC0;
        wb(m, porta, 0x40);
        let d1 = rb(m, porta) & 0x3F;
        d0 = !(d0 | d1);
        let anterior = rb(m, a0);
        let mudou = anterior ^ d0;
        wb(m, a0, d0);
        wb(m, a0.wrapping_add(1), mudou & d0);
        a0 = a0.wrapping_add(2);
    }
}

/// `VBlank_StandardTransfers`: pára o Z80, lê os controles e transfere paleta, sprites e
/// rolagem horizontal para a VDP.
pub fn standard_transfers(m: &mut Memoria) -> Result<(), ErroAbertura> {
    ww(m, Z80_BUSREQ, 0x100);
    esperar_z80(m)?;
    read_joypads(m);
    if rb(m, F_WTR_STATE) != 0 {
        dma_para_vdp(m, V_PALETTE_WATER, 0x80, 0, true);
    } else {
        dma_para_vdp(m, V_PALETTE, 0x80, 0, true);
    }
    let sprites = V_SPRITETABLEBUFFER_END.wrapping_sub(V_SPRITETABLEBUFFER);
    dma_para_vdp(m, V_SPRITETABLEBUFFER, sprites, 0xF800, false);
    dma_para_vdp(m, V_HSCROLLTABLEBUFFER, 0x380, 0xFC00, false);
    ww(m, Z80_BUSREQ, 0);
    Ok(())
}

/// `movem.l` de `longas` longas de `origem` para `destino` (as cópias `_dup` do VBlank).
fn copiar_longas(m: &mut Memoria, origem: u32, destino: u32, longas: u32) {
    for k in 0..longas {
        let v = rl(m, origem.wrapping_add(k.wrapping_mul(4)));
        wl(m, destino.wrapping_add(k.wrapping_mul(4)), v);
    }
}

/// `VBlank_TitleCards`: como as transferências padrão, mas com a linha do HBlank, a rolagem antes
/// dos sprites e a arte do Sonic quando ela mudou. Depois copia as posições da tela e as flags de
/// rolagem para os `_dup`, desenha as faixas novas, anima os tiles, atualiza o HUD e anda a fila
/// de arte.
fn vblank_cartoes(m: &mut Memoria, nivel: bool) -> Result<(), ErroAbertura> {
    ww(m, Z80_BUSREQ, 0x100);
    esperar_z80(m)?;
    read_joypads(m);
    if rb(m, F_WTR_STATE) != 0 {
        dma_para_vdp(m, V_PALETTE_WATER, 0x80, 0, true);
    } else {
        dma_para_vdp(m, V_PALETTE, 0x80, 0, true);
    }
    let h = rw(m, V_HBLANK_HREG);
    ww(m, VDP_CONTROLE, h);
    dma_para_vdp(m, V_HSCROLLTABLEBUFFER, 0x380, 0xFC00, false);
    let sprites = V_SPRITETABLEBUFFER_END.wrapping_sub(V_SPRITETABLEBUFFER);
    dma_para_vdp(m, V_SPRITETABLEBUFFER, sprites, 0xF800, false);
    if rb(m, F_SONFRAMECHG) != 0 {
        let arte = V_SGFX_BUFFER_END.wrapping_sub(V_SGFX_BUFFER);
        let destino = u32::from(ARTTILE_SONIC).wrapping_mul(32);
        dma_para_vdp(m, V_SGFX_BUFFER, arte, destino, false);
        wb(m, F_SONFRAMECHG, 0);
    }
    ww(m, Z80_BUSREQ, 0);
    // `movem.l` de `v_screenposx` até `v_bg3screenposy` (oito longas) e das quatro flags.
    copiar_longas(m, V_SCREENPOSX, V_SCREENPOSX_DUP, 8);
    copiar_longas(m, V_FG_SCROLL_FLAGS, V_FG_SCROLL_FLAGS_DUP, 2);
    // `ReadJoypads` deixa `a0` em `v_jpadhold1 + 4`; nada depois dele o muda (D7.26).
    super::desenho::load_tiles_as_you_move(m, V_JPADHOLD1.wrapping_add(4))?;
    super::animacao::animate_level_gfx(m)?;
    super::hud::hud_update(m)?;
    if nivel {
        // `VBlank_Levels` anda só 3 tiles da fila e conta o temporizador genérico.
        super::plc::process_plc_3(m);
        temporizador_generico(m);
    } else {
        super::plc::process_plc_9(m);
    }
    Ok(())
}

/// Fim de `VBlank_Sega`, `VBlank_SegaPCM` e `VBlank_Title`: decrementa o temporizador genérico.
fn temporizador_generico(m: &mut Memoria) {
    let t = rw(m, V_GENERICTIMER);
    if t != 0 {
        ww(m, V_GENERICTIMER, t.wrapping_sub(1));
    }
}

/// `VBlank`: o tratador inteiro, da entrada até o `rte`. Roda o driver de som no fim
/// (`VBlank_Music`) e conta o VBlank.
pub fn tratar(m: &mut Memoria) -> Result<(), ErroAbertura> {
    let rotina = rb(m, V_VBLANK_ROUTINE);
    if rotina != 0 {
        let _ = rw(m, VDP_CONTROLE);
        wl(m, VDP_CONTROLE, 0x4000_0010);
        let y = rl(m, V_SCRPOSY_VDP);
        wl(m, VDP_DADOS, y);
        wb(m, V_VBLANK_ROUTINE, 0);
        ww(m, F_HBLANK_PAL, 1);
        match rotina & 0x3E {
            ROTINA_SEGA => {
                standard_transfers(m)?;
                temporizador_generico(m);
            }
            ROTINA_SEGA_PCM => temporizador_generico(m),
            ROTINA_TITULO => {
                standard_transfers(m)?;
                // `ReadJoypads` deixa `a0` em `v_jpadhold1 + 4`, e as macros de DMA só usam `a5`:
                // é esse o `a0` que `DrawBlock` lê num chunk zero (D7.26).
                super::desenho::load_tiles_as_you_move_bg_only(m, V_JPADHOLD1.wrapping_add(4))?;
                super::plc::process_plc_9(m);
                temporizador_generico(m);
            }
            ROTINA_CARTOES => vblank_cartoes(m, false)?,
            ROTINA_NIVEL => vblank_cartoes(m, true)?,
            ROTINA_PALETTE_FADE => {
                standard_transfers(m)?;
                let h = rw(m, V_HBLANK_HREG);
                ww(m, VDP_CONTROLE, h);
                // `ProcessPLC_9Tiles`: até 9 tiles da fila de arte, se houver arte em curso.
                super::plc::process_plc_9(m);
            }
            _ => return Err(ErroAbertura::NaoImplementado("rotina de VBlank")),
        }
    }
    // `VBlank_Lag` fora de fase: só o driver de som.
    Driver::novo(m).update_music();
    let c = rl(m, V_VBLANK_COUNT).wrapping_add(1);
    wl(m, V_VBLANK_COUNT, c);
    let _ = Tamanho::Byte;
    Ok(())
}
