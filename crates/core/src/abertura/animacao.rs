//! Tiles animados da fase (`_inc/AnimateLevelGfx.asm`): o anel gigante e a GHZ (cachoeira, flor
//! grande e flores pequenas). As outras zonas devolvem `NaoImplementado`.

use super::boot::rom_l;
use super::ram::*;
use super::{ErroAbertura, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;

/// Bytes de um tile (`tile_size`).
const TAMANHO_DO_TILE: u32 = 32;
/// Tiles que o anel gigante copia por quadro.
const TILES_DO_ANEL_POR_QUADRO: u16 = 14;
/// Sequência das flores pequenas (`.flowerSeq`): o quadro de arte de cada passo.
const SEQUENCIA_DAS_FLORES: [u8; 4] = [0, 1, 2, 1];

/// `locVRAM`: comando de escrita na VRAM para o endereço `a`.
fn comando_vram(a: u32) -> u32 {
    0x4000_0000u32
        .wrapping_add((a & 0x3FFF) << 16)
        .wrapping_add((a & 0xC000) >> 14)
}

/// `LoadTiles`: `tiles` tiles da ROM em `origem` para a porta de dados da VDP, oito longas cada.
fn load_tiles(m: &mut Memoria, origem: u32, tiles: u16) {
    let mut a1 = origem;
    for _ in 0..u32::from(tiles).wrapping_mul(8) {
        let v = rom_l(m, a1);
        wl(m, VDP_DADOS, v);
        a1 = a1.wrapping_add(4);
    }
}

/// `AnimateLevelGfx`: nada com o jogo pausado; senão o anel gigante e os tiles da zona.
pub fn animate_level_gfx(m: &mut Memoria) -> Result<(), ErroAbertura> {
    if rw(m, F_PAUSE) != 0 {
        return Ok(());
    }
    ani_art_giant_ring(m);
    match rb(m, V_ZONE) {
        0 => {
            ani_art_ghz(m);
            Ok(())
        }
        // LZ, SLZ e SYZ não têm tiles animados (`AniArt_none`).
        1 | 3 | 4 => Ok(()),
        _ => Err(ErroAbertura::NaoImplementado(
            "tiles animados de outra zona",
        )),
    }
}

/// `AniArt_GiantRing`: enquanto `v_gfxbigring` não é zero, copia 14 tiles do anel gigante por
/// quadro, do fim para o começo.
fn ani_art_giant_ring(m: &mut Memoria) {
    let restante = rw(m, V_GFXBIGRING);
    if restante == 0 {
        return;
    }
    let restante = restante.wrapping_sub(TILES_DO_ANEL_POR_QUADRO.wrapping_mul(32));
    ww(m, V_GFXBIGRING, restante);
    let origem = ART_BIGRING.wrapping_add(u32::from(restante));
    // `addi.w #ArtTile_Giant_Ring*tile_size,d0`, depois `lsl.l #2`, `lsr.w #2`, `ori.w #$4000` e
    // `swap`: o mesmo comando do `locVRAM` para a palavra de endereço.
    let endereco = restante.wrapping_add(ARTTILE_GIANT_RING.wrapping_mul(32));
    wl(m, VDP_CONTROLE, comando_vram(u32::from(endereco)));
    load_tiles(m, origem, TILES_DO_ANEL_POR_QUADRO);
}

/// Um temporizador de animação: decrementa; se ficou negativo, recarrega com `duracao - 1`,
/// avança o quadro e devolve o quadro que estava antes. Se ainda há tempo, devolve `None`.
fn passo(m: &mut Memoria, tempo: u32, quadro: u32, duracao: u8) -> Option<u8> {
    let t = rb(m, tempo).wrapping_sub(1);
    wb(m, tempo, t);
    // `bpl`: ainda há tempo.
    if t & 0x80 == 0 {
        return None;
    }
    wb(m, tempo, duracao.wrapping_sub(1));
    let atual = rb(m, quadro);
    wb(m, quadro, atual.wrapping_add(1));
    Some(atual)
}

/// `AniArt_GHZ`: cachoeira (2 quadros de 8 tiles, a cada 6 quadros), flor grande (2 quadros de
/// 16 tiles, a cada 16) e flores pequenas (12 tiles, sequência 0 1 2 1). Quando uma das
/// animações copia tiles, as seguintes não rodam neste quadro (`bra.w LoadTiles`).
fn ani_art_ghz(m: &mut Memoria) {
    if let Some(q) = passo(m, V_LANI0_TIME, V_LANI0_FRAME, 6) {
        let origem = ART_GHZWATER.wrapping_add(u32::from(q & 1).wrapping_mul(8 * TAMANHO_DO_TILE));
        let destino = u32::from(ARTTILE_GHZ_WATERFALL).wrapping_mul(TAMANHO_DO_TILE);
        wl(m, VDP_CONTROLE, comando_vram(destino));
        load_tiles(m, origem, 8);
        return;
    }
    if let Some(q) = passo(m, V_LANI1_TIME, V_LANI1_FRAME, 16) {
        let origem =
            ART_GHZFLOWER1.wrapping_add(u32::from(q & 1).wrapping_mul(16 * TAMANHO_DO_TILE));
        let destino = u32::from(ARTTILE_GHZ_BIG_FLOWER_1).wrapping_mul(TAMANHO_DO_TILE);
        wl(m, VDP_CONTROLE, comando_vram(destino));
        load_tiles(m, origem, 16);
        return;
    }
    if let Some(q) = passo(m, V_LANI2_TIME, V_LANI2_FRAME, 8) {
        let arte = SEQUENCIA_DAS_FLORES
            .get(usize::from(q & 3))
            .copied()
            .unwrap_or(0);
        // Os quadros 0 e 2 (a flor aberta e fechada) duram mais.
        if arte & 1 == 0 {
            wb(m, V_LANI2_TIME, 128 - 1);
        }
        let destino = u32::from(ARTTILE_GHZ_SMALL_FLOWER).wrapping_mul(TAMANHO_DO_TILE);
        wl(m, VDP_CONTROLE, comando_vram(destino));
        // `lsl.w #7` e vezes 3: cada quadro tem 12 tiles ($180 bytes).
        let origem = ART_GHZFLOWER2.wrapping_add(u32::from(arte).wrapping_mul(0x180));
        load_tiles(m, origem, 12);
    }
}
