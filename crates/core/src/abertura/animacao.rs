//! Tiles animados da fase (`_inc/AnimateLevelGfx.asm`): o anel gigante e a GHZ (cachoeira, flor
//! grande e flores pequenas) e a MZ (lava, magma e tochas).

use super::boot::{rom_b, rom_l};
use super::ram::*;
use super::{ErroAbertura, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;

/// Bytes de um tile (`tile_size`).
const TAMANHO_DO_TILE: u32 = 32;
/// Tiles que o anel gigante copia por quadro.
const TILES_DO_ANEL_POR_QUADRO: u16 = 14;
/// Sequência das flores pequenas (`.flowerSeq`): o quadro de arte de cada passo.
const SEQUENCIA_DAS_FLORES: u32 = 0x1C8C4;

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
        ID_MZ => {
            ani_art_mz(m);
            Ok(())
        }
        // LZ, SLZ e SYZ não têm tiles animados (`AniArt_none`).
        1 | 3 | 4 => Ok(()),
        _ => Err(ErroAbertura::NaoImplementado(
            "tiles animados de outra zona",
        )),
    }
}

/// `AniArt_MZ`: o magma usa quatro faixas com deslocamento de bytes oscilante.
fn ani_art_mz(m: &mut Memoria) {
    const LAVA: u32 = 0x67516;
    const MAGMA: u32 = 0x67816;
    const TOCHA: u32 = 0x67E16;
    const LANI3_FRAME: u32 = 0xFF_F7B6;
    if let Some(quadro) = passo(m, V_LANI0_TIME, V_LANI0_FRAME, 20) {
        let novo = if quadro.wrapping_add(1) == 3 {
            0
        } else {
            quadro.wrapping_add(1)
        };
        wb(m, V_LANI0_FRAME, novo);
        wl(m, VDP_CONTROLE, comando_vram(0x2E2 * TAMANHO_DO_TILE));
        load_tiles(m, LAVA.wrapping_add(u32::from(novo).wrapping_mul(0x100)), 8);
    }
    if passo(m, V_LANI1_TIME, V_LANI1_FRAME, 2).is_some() {
        let quadro = rb(m, V_LANI0_FRAME);
        let origem = MAGMA.wrapping_add(u32::from(quadro).wrapping_mul(0x200));
        let oscilacao = rb(m, V_OSCILLATE.wrapping_add(0xA));
        wl(m, VDP_CONTROLE, comando_vram(0x2D2 * TAMANHO_DO_TILE));
        for coluna in 0..4u8 {
            let deslocamento = oscilacao.wrapping_add(coluna.wrapping_mul(4)) & 0x0F;
            for linha in 0..32u32 {
                let inicio = origem.wrapping_add(linha.wrapping_mul(16));
                let mut bytes = [0; 4];
                for (i, byte) in bytes.iter_mut().enumerate() {
                    let indice = deslocamento.wrapping_add(u8::try_from(i).unwrap_or(0)) & 0x0F;
                    *byte = rom_b(m, inicio.wrapping_add(u32::from(indice)));
                }
                if deslocamento == 14 {
                    ww(m, VDP_DADOS, u16::from_be_bytes([bytes[0], bytes[1]]));
                    ww(m, VDP_DADOS, u16::from_be_bytes([bytes[2], bytes[3]]));
                } else {
                    wl(m, VDP_DADOS, u32::from_be_bytes(bytes));
                }
            }
        }
        return;
    }
    if let Some(quadro) = passo(m, V_LANI2_TIME, LANI3_FRAME, 8) {
        wb(m, LANI3_FRAME, quadro.wrapping_add(1) & 3);
        wl(m, VDP_CONTROLE, comando_vram(0x2F2 * TAMANHO_DO_TILE));
        load_tiles(
            m,
            TOCHA.wrapping_add(u32::from(quadro).wrapping_mul(0xC0)),
            6,
        );
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
        let arte = rom_b(m, SEQUENCIA_DAS_FLORES.wrapping_add(u32::from(q & 3)));
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

#[cfg(test)]
mod testes {
    use super::*;
    use crate::memoria::{PortaVdp, Saida, Tamanho};

    #[test]
    fn mz_lava_e_magma_copiam_dados_da_rom_e_respeitam_os_tempos() {
        let mut rom = vec![0; 0x70000];
        for (i, byte) in rom[0x67516..0x68416].iter_mut().enumerate() {
            *byte = u8::try_from(i & 0xFF).unwrap();
        }
        let mut m = Memoria::nova(rom);
        wb(&mut m, V_ZONE, ID_MZ);
        wb(&mut m, V_OSCILLATE.wrapping_add(0xA), 15);
        animate_level_gfx(&mut m).unwrap();
        assert_eq!(rb(&mut m, V_LANI0_FRAME), 1);
        assert_eq!(rb(&mut m, V_LANI0_TIME), 19);
        assert_eq!(rb(&mut m, V_LANI1_FRAME), 1);
        assert_eq!(rb(&mut m, V_LANI2_TIME), 0);
        let saidas = m.saidas();
        // Duas palavras de comando, 128 palavras de lava e duas do comando do magma.
        assert_eq!(
            saidas[132],
            Saida::Vdp {
                porta: PortaVdp::Dados,
                valor: 0x0F00,
                tamanho: Tamanho::Word
            }
        );
        assert_eq!(
            saidas[133],
            Saida::Vdp {
                porta: PortaVdp::Dados,
                valor: 0x0102,
                tamanho: Tamanho::Word
            }
        );
        m.tirar_saidas();
        animate_level_gfx(&mut m).unwrap();
        assert_eq!(rb(&mut m, V_LANI2_TIME), 7);
        assert_eq!(rb(&mut m, 0xFF_F7B6), 1);
        assert_eq!(m.saidas().len(), 2 + 6 * 16);
    }

    #[test]
    fn pausa_nao_avanca_animacao() {
        let mut m = Memoria::nova(vec![0; 0x70000]);
        wb(&mut m, V_ZONE, ID_MZ);
        ww(&mut m, F_PAUSE, 1);
        animate_level_gfx(&mut m).unwrap();
        assert_eq!(rb(&mut m, V_LANI0_TIME), 0);
        assert!(m.saidas().is_empty());
    }
}
