//! Desenho do fundo da GHZ na tela do título (sub-etapa 7.5, parte C): `DrawChunks`,
//! `DrawBlocks_LR_2`, `GetBlockData`, `DrawBlock` e `Calc_VRAM_Pos`, da referência REV01. Cada
//! função cita o rótulo que reproduz. O layout e os chunks vêm da RAM (carregados na parte B).
//!
//! Limite: em `GetBlockData`, um chunk zero não atualiza `a0`. O original usa o que sobrou nesse
//! registrador para ler as flags do bloco. Aqui `a0` é uma variável local de `draw_chunks`, que
//! começa em zero. Se a comparação acusar diferença nas linhas com chunk zero, é esta a causa.

use super::ram::*;
use super::wl;
use crate::memoria::Memoria;

/// `d7` de `DrawBlock`: uma linha de plano são $80 bytes, somados na palavra alta do comando.
const AVANCO_DE_LINHA: u32 = 0x80_0000;

/// Leituras da RAM sem mudar o estado (a memória pede `&mut` só para contar acessos).
fn ler_b(m: &Memoria, a: u32) -> u8 {
    m.ram()[usize::try_from(a.wrapping_sub(0xFF_0000)).unwrap_or(0)]
}

fn ler_w(m: &Memoria, a: u32) -> u16 {
    u16::from_be_bytes([ler_b(m, a), ler_b(m, a.wrapping_add(1))])
}

fn ler_l(m: &Memoria, a: u32) -> u32 {
    u32::from_be_bytes([
        ler_b(m, a),
        ler_b(m, a.wrapping_add(1)),
        ler_b(m, a.wrapping_add(2)),
        ler_b(m, a.wrapping_add(3)),
    ])
}

/// `Calc_VRAM_Pos`: comando da VRAM para a posição do plano. `y` e `x` são as coordenadas da
/// tela, somadas à posição do fundo (`a3`); o plano tem $100 de largura e $200 de altura.
fn calc_vram_pos(m: &Memoria, y: u16, x: u16, a3: u32) -> u32 {
    let y = y.wrapping_add(ler_w(m, a3.wrapping_add(4))) & 0xF0;
    let x = x.wrapping_add(ler_w(m, a3)) & 0x1F0;
    let posicao = y.wrapping_shl(4).wrapping_add(x >> 2);
    0x0003_0000 | u32::from(posicao)
}

/// `GetBlockData`: o bloco em `(x, y)` do plano. Devolve `(a0, a1)`: o endereço da palavra do
/// bloco no chunk, e o endereço dos oito bytes do bloco em `v_16x16`. Chunk zero não tem bloco:
/// devolve `a0` como estava e `a1` no início de `v_16x16` (o `DrawBlock` desenha assim mesmo).
fn get_block_data(m: &Memoria, a0_antigo: u32, y: u16, x: u16, a3: u32, a4: u32) -> (u32, u32) {
    let x = x.wrapping_add(ler_w(m, a3));
    let y = y.wrapping_add(ler_w(m, a3.wrapping_add(4)));
    // Linha do layout: `(y >> 1) & $380`. Coluna: `(x >> 3) >> 5`, com `& $7F`.
    let linha = (y >> 1) & 0x380;
    let x_bloco = x >> 3;
    let indice = ((x_bloco >> 5) & 0x7F).wrapping_add(linha);
    let chunk = ler_b(m, a4.wrapping_add(u32::from(indice)));
    if chunk == 0 {
        return (a0_antigo, V_16X16);
    }
    // `subq.b #1` e `ror.w #7`: cada chunk ocupa $200 bytes a partir de `v_256x256`.
    let base_chunk = u16::from(chunk.wrapping_sub(1) & 0x7F).wrapping_mul(0x200);
    let dy = y.wrapping_mul(2) & 0x1E0;
    let dx = x_bloco & 0x1E;
    let posicao = V_256X256.wrapping_add(u32::from(base_chunk.wrapping_add(dy).wrapping_add(dx)));
    let id = ler_w(m, posicao) & 0x3FF;
    (posicao, V_16X16.wrapping_add(u32::from(id).wrapping_mul(8)))
}

/// `DrawBlock`: escreve as duas linhas de tiles de um bloco na VRAM, com as flips que a palavra
/// do bloco pede (`a0` aponta para essa palavra, `a1` para os oito bytes do bloco).
fn draw_block(m: &mut Memoria, comando: u32, d2: u16, a0: u32, a1: u32) {
    // `or.w d2,d0` e `swap d0`: a palavra baixa do comando vai para a alta.
    let baixa = u16::try_from(comando & 0xFFFF).unwrap_or(0) | d2;
    let cmd = (comando & 0xFFFF_0000 | u32::from(baixa)).rotate_left(16);
    // Flip vertical e horizontal estão nos bits 4 e 3 do byte alto da palavra do bloco.
    let flags = ler_b(m, a0);
    let flip_y = flags & 0x10 != 0;
    let flip_x = flags & 0x08 != 0;
    let topo = ler_l(m, a1);
    let base = ler_l(m, a1.wrapping_add(4));
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

/// `DrawBlocks_LR_2`: uma faixa horizontal de 32 blocos na linha `y`, a partir do comando `d0`.
fn draw_blocks_lr_2(
    m: &mut Memoria,
    mut a0: u32,
    comando: u32,
    y: u16,
    d2: u16,
    a3: u32,
    a4: u32,
) -> u32 {
    let mut d1 = comando;
    for k in 0..32u16 {
        let x = k.wrapping_mul(16);
        let (a0_novo, a1) = get_block_data(m, a0, y, x, a3, a4);
        a0 = a0_novo;
        draw_block(m, d1, d2, a0, a1);
        // `addq.b #4,d1` e `andi.b #$7F,d1`: só o byte baixo do endereço anda, com a máscara.
        let baixo = (d1 & 0xFF).wrapping_add(4) & 0x7F;
        d1 = (d1 & 0xFFFF_FF00) | baixo;
    }
    a0
}

/// `DrawChunks` (REV01): o plano inteiro de fundo, 16 faixas de 32 blocos, de $-10 a $F0 na tela.
/// `a3` é a posição do fundo (`v_bgscreenposx`), `a4` o layout de fundo e `d2` a base do plano
/// (`$6000` para o fundo do título). Devolve o `a0` final, que o chamador não usa.
pub fn draw_chunks(m: &mut Memoria, a3: u32, a4: u32, d2: u16) {
    let mut d4: u16 = 0xFFF0;
    let mut a0: u32 = 0;
    for _ in 0..16 {
        let comando = calc_vram_pos(m, d4, 0, a3);
        a0 = draw_blocks_lr_2(m, a0, comando, d4, d2, a3, a4);
        d4 = d4.wrapping_add(16);
    }
}
