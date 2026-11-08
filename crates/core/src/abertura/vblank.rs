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

/// `VBlank_SegaPCM` (e o fim de `VBlank_Sega`): decrementa o temporizador genérico.
fn segapcm(m: &mut Memoria) {
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
                segapcm(m);
            }
            ROTINA_SEGA_PCM => segapcm(m),
            ROTINA_PALETTE_FADE => {
                standard_transfers(m)?;
                let h = rw(m, V_HBLANK_HREG);
                ww(m, VDP_CONTROLE, h);
                // `ProcessPLC_9Tiles` sem fila de arte não faz nada visível (só lê a RAM).
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
