//! Desenho do fundo da GHZ (`_inc/Level Drawing (REV01).asm`): `DrawChunks` na parte C do título
//! e `LoadTilesAsYouMove_BGOnly` no VBlank do título, com `DrawBG_Top`, `DrawBG_Bottom`, as
//! faixas `DrawBlocks_*`, `GetBlockData`, `DrawBlock` e `Calc_VRAM_Pos`. Cada função cita o rótulo
//! que reproduz. O layout e os chunks vêm da RAM (carregados na parte B).
//!
//! O registrador `a0` não é estado do núcleo. Ele só aparece porque `GetBlockData` não o atualiza
//! num chunk zero, e `DrawBlock` lê as flags de `(a0)` mesmo assim. Quem chama diz o valor que o
//! original tem nele (ver `ENDERECO_A0_INICIAL` e o VBlank do título), e cada faixa devolve o
//! `a0` que sobrou para a seguinte.

use super::ram::*;
use super::{ErroAbertura, rb, rl, rw, wb, wl};
use crate::memoria::Memoria;

/// `d7` de `DrawBlock`: uma linha de plano são $80 bytes, somados na palavra alta do comando.
const AVANCO_DE_LINHA: u32 = 0x80_0000;

/// `a0` quando `DrawChunks` começa. O `PaletteFadeOut` que vem antes escurece a paleta de água
/// (`FadeOut_ToBlack`). Cada chamada de `FadeOut_DecColor` avança `a0` em exatamente dois bytes,
/// e são 64 cores a partir de `v_palette_water`. Ao fim, `a0` vale `v_palette_water + $80`, e
/// `ClearScreen` (que usa `a1`, `a5` e `d0` a `d1`) não o altera.
const ENDERECO_A0_INICIAL: u32 = V_PALETTE_WATER + 0x80;

/// Coordenadas da tela em pixels, como o `moveq #-16` e o `move.w #224` do original as põem.
const ACIMA: u16 = 0xFFF0;
const ABAIXO: u16 = 224;
const A_ESQUERDA: u16 = 0xFFF0;
const A_DIREITA: u16 = 320;
const MEIO_DA_TELA: u16 = 224 / 2;

/// `DrawBlocks_LR`: a largura da tela mais duas colunas (22 blocos).
const BLOCOS_NA_LARGURA: u16 = (320 + 16 + 16) / 16;
/// `DrawBlocks_TB`: a altura da tela mais duas linhas (16 blocos).
const BLOCOS_NA_ALTURA: u16 = (224 + 16 + 16) / 16;
/// Largura do plano em blocos (`(512/16)-1` no `dbf`).
const BLOCOS_NO_PLANO: u16 = 512 / 16;

/// Como a faixa acha o bloco: `GetBlockData` soma o X da tela; `GetBlockData_2` não.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SomaX {
    Sim,
    Nao,
}

/// `Calc_VRAM_Pos`: soma o X da tela (`a3`) e segue como `Calc_VRAM_Pos_2`.
fn calc_vram_pos(m: &mut Memoria, y: u16, x: u16, a3: u32) -> u32 {
    let x = x.wrapping_add(rw(m, a3));
    calc_vram_pos_2(m, y, x, a3)
}

/// `Calc_VRAM_Pos_2`: comando da VRAM para a posição do plano. Só o Y da tela (`4(a3)`) é
/// somado; o plano tem $200 de largura e $100 de altura, em blocos de 16 pixels.
fn calc_vram_pos_2(m: &mut Memoria, y: u16, x: u16, a3: u32) -> u32 {
    let y = y.wrapping_add(rw(m, a3.wrapping_add(4))) & 0xF0;
    let x = x & 0x1F0;
    let posicao = y.wrapping_shl(4).wrapping_add(x >> 2);
    0x0003_0000 | u32::from(posicao)
}

/// `GetBlockData`: soma o X da tela e segue como `GetBlockData_2`.
fn get_block_data(m: &mut Memoria, a0_antigo: u32, y: u16, x: u16, a3: u32, a4: u32) -> (u32, u32) {
    let x = x.wrapping_add(rw(m, a3));
    get_block_data_2(m, a0_antigo, y, x, a3, a4)
}

/// `GetBlockData_2`: o bloco em `(x, y)` do plano, somando só o Y da tela. Devolve `(a0, a1)`: o
/// endereço da palavra do bloco no chunk, e o endereço dos oito bytes do bloco em `v_16x16`.
/// Chunk zero não tem bloco: devolve `a0` como estava e `a1` no início de `v_16x16` (o
/// `DrawBlock` desenha assim mesmo).
fn get_block_data_2(
    m: &mut Memoria,
    a0_antigo: u32,
    y: u16,
    x: u16,
    a3: u32,
    a4: u32,
) -> (u32, u32) {
    let y = y.wrapping_add(rw(m, a3.wrapping_add(4)));
    // Linha do layout: `(y >> 1) & $380`. Coluna: `(x >> 3) >> 5`, com `& $7F`.
    let linha = (y >> 1) & 0x380;
    let x_bloco = x >> 3;
    let indice = ((x_bloco >> 5) & 0x7F).wrapping_add(linha);
    let chunk = rb(m, a4.wrapping_add(u32::from(indice)));
    if chunk == 0 {
        return (a0_antigo, V_16X16);
    }
    // `subq.b #1` e `ror.w #7`: cada chunk ocupa $200 bytes a partir de `v_256x256`.
    let base_chunk = u16::from(chunk.wrapping_sub(1) & 0x7F).wrapping_mul(0x200);
    let dy = y.wrapping_mul(2) & 0x1E0;
    let dx = x_bloco & 0x1E;
    let posicao = V_256X256.wrapping_add(u32::from(base_chunk.wrapping_add(dy).wrapping_add(dx)));
    let id = rw(m, posicao) & 0x3FF;
    (posicao, V_16X16.wrapping_add(u32::from(id).wrapping_mul(8)))
}

/// `DrawBlock`: escreve as duas linhas de tiles de um bloco na VRAM, com as flips que a palavra
/// do bloco pede (`a0` aponta para essa palavra, `a1` para os oito bytes do bloco).
fn draw_block(m: &mut Memoria, comando: u32, d2: u16, a0: u32, a1: u32) {
    // `or.w d2,d0` e `swap d0`: a palavra baixa do comando vai para a alta.
    let baixa = u16::try_from(comando & 0xFFFF).unwrap_or(0) | d2;
    let cmd = (comando & 0xFFFF_0000 | u32::from(baixa)).rotate_left(16);
    // Flip vertical e horizontal estão nos bits 4 e 3 do byte alto da palavra do bloco.
    let flags = rb(m, a0);
    let flip_y = flags & 0x10 != 0;
    let flip_x = flags & 0x08 != 0;
    let topo = rl(m, a1);
    let base = rl(m, a1.wrapping_add(4));
    // Cada par é (linha escrita em `cmd`, linha escrita em `cmd + AVANCO_DE_LINHA`).
    // Com flip vertical, a linha de baixo sai primeiro, em cima, e as linhas não são trocadas.
    let (primeira, segunda) = match (flip_y, flip_x) {
        (false, false) => (topo, base),
        (false, true) => (espelhar(topo), espelhar(base)),
        (true, false) => (base ^ 0x1000_1000, topo ^ 0x1000_1000),
        (true, true) => (espelhar_virado(base), espelhar_virado(topo)),
    };
    wl(m, VDP_CONTROLE, cmd);
    wl(m, VDP_DADOS, primeira);
    wl(m, VDP_CONTROLE, cmd.wrapping_add(AVANCO_DE_LINHA));
    wl(m, VDP_DADOS, segunda);
}

/// Espelhamento horizontal de uma linha de dois tiles: troca os tiles e inverte o bit de flip.
fn espelhar(v: u32) -> u32 {
    (v ^ 0x0800_0800).rotate_left(16)
}

/// Espelhamento e flip vertical juntos: como [`espelhar`], com os dois bits de flip.
fn espelhar_virado(v: u32) -> u32 {
    (v ^ 0x1800_1800).rotate_left(16)
}

/// O que as faixas recebem do chamador: a posição da tela (`a3`), o layout (`a4`) e a base do
/// plano (`d2`).
#[derive(Clone, Copy)]
struct Plano {
    a3: u32,
    a4: u32,
    d2: u16,
}

/// `DrawBlocks_LR_2` e `DrawBlocks_LR_3`: uma faixa horizontal de `blocos` blocos na linha `y`, a
/// partir de `x` e do comando `d0`. Devolve o `a0` que sobrou, para a próxima faixa.
fn draw_blocks_lr(
    m: &mut Memoria,
    mut a0: u32,
    comando: u32,
    (y, x): (u16, u16),
    blocos: u16,
    soma_x: SomaX,
    p: Plano,
) -> u32 {
    let mut d1 = comando;
    for k in 0..blocos {
        let x = x.wrapping_add(k.wrapping_mul(16));
        let (a0_novo, a1) = match soma_x {
            SomaX::Sim => get_block_data(m, a0, y, x, p.a3, p.a4),
            SomaX::Nao => get_block_data_2(m, a0, y, x, p.a3, p.a4),
        };
        a0 = a0_novo;
        draw_block(m, d1, p.d2, a0, a1);
        // `addq.b #4,d1` e `andi.b #$7F,d1`: só o byte baixo do endereço anda, com a máscara.
        let baixo = (d1 & 0xFF).wrapping_add(4) & 0x7F;
        d1 = (d1 & 0xFFFF_FF00) | baixo;
    }
    a0
}

/// `DrawBlocks_TB_2` (e `DrawBlocks_TB`): uma coluna de `blocos` blocos na coluna `x`, a partir
/// de `y` e do comando `d0`. Devolve o `a0` que sobrou.
fn draw_blocks_tb(
    m: &mut Memoria,
    mut a0: u32,
    comando: u32,
    (y, x): (u16, u16),
    blocos: u16,
    p: Plano,
) -> u32 {
    let mut d1 = comando;
    for k in 0..blocos {
        let y = y.wrapping_add(k.wrapping_mul(16));
        let (a0_novo, a1) = get_block_data(m, a0, y, x, p.a3, p.a4);
        a0 = a0_novo;
        draw_block(m, d1, p.d2, a0, a1);
        // `addi.w #$100,d1` e `andi.w #$FFF,d1`: a palavra baixa desce duas linhas do plano.
        let baixa = (d1 & 0xFFFF).wrapping_add(0x100) & 0xFFF;
        d1 = (d1 & 0xFFFF_0000) | baixa;
    }
    a0
}

/// `bclr #bit,(a)`: limpa o bit do byte e diz se ele estava ligado.
fn bclr(m: &mut Memoria, a: u32, bit: u8) -> bool {
    let mascara = 1u8.wrapping_shl(u32::from(bit));
    let v = rb(m, a);
    wb(m, a, v & !mascara);
    v & mascara != 0
}

/// `DrawBG_Top`: a parte de cima do fundo, pelas flags do byte `a2` (bits 0 a 3: faixa de cima,
/// de baixo, da esquerda e da direita; bits 4 e 5: a linha inteira do plano em cima e embaixo).
fn draw_bg_top(m: &mut Memoria, mut a0: u32, a2: u32, p: Plano) -> u32 {
    if rb(m, a2) == 0 {
        return a0;
    }
    if bclr(m, a2, 0) {
        let cmd = calc_vram_pos(m, ACIMA, A_ESQUERDA, p.a3);
        let pos = (ACIMA, A_ESQUERDA);
        a0 = draw_blocks_lr(m, a0, cmd, pos, BLOCOS_NA_LARGURA, SomaX::Sim, p);
    }
    if bclr(m, a2, 1) {
        let cmd = calc_vram_pos(m, ABAIXO, A_ESQUERDA, p.a3);
        let pos = (ABAIXO, A_ESQUERDA);
        a0 = draw_blocks_lr(m, a0, cmd, pos, BLOCOS_NA_LARGURA, SomaX::Sim, p);
    }
    if bclr(m, a2, 2) {
        let cmd = calc_vram_pos(m, ACIMA, A_ESQUERDA, p.a3);
        a0 = draw_blocks_tb(m, a0, cmd, (ACIMA, A_ESQUERDA), BLOCOS_NA_ALTURA, p);
    }
    if bclr(m, a2, 3) {
        let cmd = calc_vram_pos(m, ACIMA, A_DIREITA, p.a3);
        a0 = draw_blocks_tb(m, a0, cmd, (ACIMA, A_DIREITA), BLOCOS_NA_ALTURA, p);
    }
    if bclr(m, a2, 4) {
        let cmd = calc_vram_pos_2(m, ACIMA, 0, p.a3);
        a0 = draw_blocks_lr(m, a0, cmd, (ACIMA, 0), BLOCOS_NO_PLANO, SomaX::Nao, p);
    }
    if bclr(m, a2, 5) {
        let cmd = calc_vram_pos_2(m, ABAIXO, 0, p.a3);
        a0 = draw_blocks_lr(m, a0, cmd, (ABAIXO, 0), BLOCOS_NO_PLANO, SomaX::Nao, p);
    }
    a0
}

/// `DrawBG_Bottom` (REV01): a parte de baixo do fundo. O bit 0 do byte `a2` desenha três blocos
/// em coluna à esquerda da tela, a partir do meio da altura; o bit 1, à direita. A SBZ tem um
/// desenho próprio, que ainda não existe aqui.
fn draw_bg_bottom(m: &mut Memoria, mut a0: u32, a2: u32, p: Plano) -> Result<u32, ErroAbertura> {
    if rb(m, a2) == 0 {
        return Ok(a0);
    }
    if rb(m, V_ZONE) == ID_SBZ {
        return Err(ErroAbertura::NaoImplementado("fundo da SBZ"));
    }
    if bclr(m, a2, 0) {
        let cmd = calc_vram_pos(m, MEIO_DA_TELA, A_ESQUERDA, p.a3);
        a0 = draw_blocks_tb(m, a0, cmd, (MEIO_DA_TELA, A_ESQUERDA), 3, p);
    }
    if bclr(m, a2, 1) {
        let cmd = calc_vram_pos(m, MEIO_DA_TELA, A_DIREITA, p.a3);
        a0 = draw_blocks_tb(m, a0, cmd, (MEIO_DA_TELA, A_DIREITA), 3, p);
    }
    Ok(a0)
}

/// `LoadTilesAsYouMove_BGOnly`: desenha as faixas novas do fundo que a rolagem do título pediu.
/// `a0` é o valor que o registrador tem na entrada (o chamador diz qual).
pub fn load_tiles_as_you_move_bg_only(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let fundo = Plano {
        a3: V_BGSCREENPOSX,
        a4: V_LVLLAYOUT_BG,
        d2: 0x6000,
    };
    let a0 = draw_bg_top(m, a0, V_BG1_SCROLL_FLAGS, fundo);
    let fundo_2 = Plano {
        a3: V_BG2SCREENPOSX,
        ..fundo
    };
    draw_bg_bottom(m, a0, V_BG2_SCROLL_FLAGS, fundo_2)?;
    Ok(())
}

/// `DrawChunks` (REV01): o plano inteiro de fundo, 16 faixas de 32 blocos, de $-10 a $F0 na tela.
/// `a3` é a posição do fundo (`v_bgscreenposx`), `a4` o layout de fundo e `d2` a base do plano
/// (`$6000` para o fundo do título).
pub fn draw_chunks(m: &mut Memoria, a3: u32, a4: u32, d2: u16) {
    let p = Plano { a3, a4, d2 };
    let mut d4: u16 = ACIMA;
    let mut a0 = ENDERECO_A0_INICIAL;
    for _ in 0..BLOCOS_NA_ALTURA {
        let comando = calc_vram_pos(m, d4, 0, a3);
        a0 = draw_blocks_lr(m, a0, comando, (d4, 0), BLOCOS_NO_PLANO, SomaX::Sim, p);
        d4 = d4.wrapping_add(16);
    }
}
