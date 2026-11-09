//! Fila de arte da tela do título e da fase (sub-etapa 7.5, parte B): `NewPLC`, `ClearPLC`,
//! `RunPLC`, `ProcessPLC_9Tiles` e `ProcessPLC`. A descompressão Nemesis anda uma linha de 8
//! pixels por vez, como no original, e o estado entre os quadros fica nas variáveis `v_plc_*` da
//! RAM, que a comparação enxerga. Cada função cita o rótulo que reproduz (REV01).
//!
//! Limite: a parte alta de `d5` (a janela de bits) no `RunPLC` é zero aqui. No original é o que
//! sobrou no registrador. Se a comparação achar diferença em `v_plc_dataword`, é essa a causa.

use super::boot::{rom_b, rom_l, rom_w};
use super::ram::*;
use super::{ErroAbertura, rl, rw, wl, ww};
use crate::memoria::Memoria;

/// `v_ngfx_buffer`: tabela de códigos do Nemesis (`NemDec_BuildCodeTable`).
const V_NGFX_BUFFER: u32 = 0xFF_AA00;

/// Longas copiadas por `ProcessPLC_ShiftCue`: `(v_plc_buffer_only_end - v_plc_buffer -
/// plc_slot_size) / 4` (22). O `dbf` roda essa quantidade de vezes, como no original.
const LONGAS_DO_DESEMPILHAR: u32 = (V_PLC_BUFFER_ONLY_END - V_PLC_BUFFER - PLC_SLOT_SIZE) / 4;

/// `ClearPLC`: zera a fila inteira ($FFF680 a $FFF700).
pub fn clear_plc(m: &mut Memoria) {
    let mut a = V_PLC_BUFFER;
    while a < V_PLC_BUFFER_END {
        wl(m, a, 0);
        a = a.wrapping_add(4);
    }
}

/// `NewPLC`: esvazia a fila e copia a lista de PLC `id` (tabela `ArtLoadCues`) para ela.
pub fn new_plc(m: &mut Memoria, id: u16) {
    let deslocamento = rom_w(m, ART_LOAD_CUES.wrapping_add(u32::from(id).wrapping_mul(2)));
    let lista = ART_LOAD_CUES.wrapping_add(u32::from(deslocamento));
    clear_plc(m);
    let quantos = rom_w(m, lista);
    // `bmi`: contagem negativa quer dizer lista vazia. `dbf` faz (contagem + 1) entradas.
    if quantos & 0x8000 != 0 {
        return;
    }
    let mut origem = lista.wrapping_add(2);
    let mut destino = V_PLC_BUFFER;
    for _ in 0..=u32::from(quantos) {
        let arte = rom_l(m, origem);
        let vram = rom_w(m, origem.wrapping_add(4));
        wl(m, destino, arte);
        ww(m, destino.wrapping_add(4), vram);
        origem = origem.wrapping_add(PLC_SLOT_SIZE);
        destino = destino.wrapping_add(PLC_SLOT_SIZE);
    }
}

/// Coloca uma entrada na primeira posição livre da fila (uso de teste e de montagem manual).
pub fn enfileirar(m: &mut Memoria, arte: u32, vram: u16) {
    let mut a = V_PLC_BUFFER;
    while a < V_PLC_BUFFER_ONLY_END && rl(m, a) != 0 {
        a = a.wrapping_add(PLC_SLOT_SIZE);
    }
    wl(m, a, arte);
    ww(m, a.wrapping_add(4), vram);
}

/// `RunPLC`: se a fila tem entrada e nenhuma arte está em curso, começa a descompressão da
/// primeira entrada: monta a tabela de códigos e guarda o estado inicial do fluxo.
pub fn run_plc(m: &mut Memoria) -> Result<(), ErroAbertura> {
    if rl(m, V_PLC_BUFFER) == 0 || rw(m, V_PLC_PATTERNSLEFT) != 0 {
        return Ok(());
    }
    let mut a0 = rl(m, V_PLC_BUFFER);
    let mut rotina = NEM_PCD_WRITE_ROW_TO_VDP;
    let cabecalho = rom_w(m, a0);
    a0 = a0.wrapping_add(2);
    // Bit 15 liga o modo XOR, e a contagem de seções são os 15 bits baixos.
    if cabecalho & 0x8000 != 0 {
        rotina = NEM_PCD_WRITE_ROW_TO_VDP_XOR;
    }
    ww(m, V_PLC_PATTERNSLEFT, cabecalho & 0x7FFF);
    a0 = construir_tabela(m, a0)?;
    // Dois bytes da janela de bits, e o deslocamento começa em $10.
    let d5 = u16::from(rom_b(m, a0)).wrapping_shl(8) | u16::from(rom_b(m, a0.wrapping_add(1)));
    a0 = a0.wrapping_add(2);
    wl(m, V_PLC_BUFFER, a0);
    wl(m, V_PLC_PTRNEMCODE, rotina);
    wl(m, V_PLC_REPEATCOUNT, 0);
    wl(m, V_PLC_PALETTEINDEX, 0);
    wl(m, V_PLC_PREVIOUSROW, 0);
    wl(m, V_PLC_DATAWORD, u32::from(d5));
    wl(m, V_PLC_SHIFTVALUE, 0x10);
    Ok(())
}

/// `NemDec_BuildCodeTable` sobre o fluxo da ROM a partir de `a0`. Grava a tabela em
/// `v_ngfx_buffer` e devolve o endereço depois do `$FF` final.
fn construir_tabela(m: &mut Memoria, mut a0: u32) -> Result<u32, ErroAbertura> {
    let mut d0 = rom_b(m, a0);
    a0 = a0.wrapping_add(1);
    loop {
        // `NemBCT_ChkEnd`
        if d0 == 0xFF {
            return Ok(a0);
        }
        // `NemBCT_NewPALIndex`: o byte com o bit 7 dá o nibble de paleta.
        let mut d7 = u16::from(d0);
        loop {
            // Sem o `$FF` final, o laço leria zeros para sempre: fora da ROM é erro.
            if fora_da_rom(m, a0) {
                return Err(ErroAbertura::NaoImplementado(
                    "tabela de codigos sem fim dentro da ROM",
                ));
            }
            d0 = rom_b(m, a0);
            a0 = a0.wrapping_add(1);
            if d0 >= 0x80 {
                break;
            }
            let repeticao = u16::from(d0) & 0x70;
            d7 = (d7 & 0x0F) | repeticao;
            let tamanho = u16::from(d0 & 0x0F);
            d7 |= tamanho.wrapping_shl(8);
            let deslocamento_bits = 8u16.wrapping_sub(tamanho);
            if deslocamento_bits != 0 {
                // Código curto: ocupa todas as entradas que começam com ele.
                let codigo = rom_b(m, a0);
                a0 = a0.wrapping_add(1);
                let base = u16::from(codigo)
                    .wrapping_shl(u32::from(deslocamento_bits))
                    .wrapping_mul(2);
                let quantas = 1u16.wrapping_shl(u32::from(deslocamento_bits));
                for k in 0..quantas {
                    let posicao = base.wrapping_add(k.wrapping_mul(2));
                    ww(m, V_NGFX_BUFFER.wrapping_add(u32::from(posicao)), d7);
                }
            } else {
                let codigo = rom_b(m, a0);
                a0 = a0.wrapping_add(1);
                let posicao = u16::from(codigo).wrapping_mul(2);
                ww(m, V_NGFX_BUFFER.wrapping_add(u32::from(posicao)), d7);
            }
        }
    }
}

/// `ProcessPLC_9Tiles`: se há arte em curso, descomprime até 9 tiles neste VBlank.
pub fn process_plc_9(m: &mut Memoria) {
    if rw(m, V_PLC_PATTERNSLEFT) == 0 {
        return;
    }
    ww(m, V_PLC_FRAMEPATTERNSLEFT, 9);
    let destino = rw(m, V_PLC_BUFFER_DEST);
    ww(m, V_PLC_BUFFER_DEST, destino.wrapping_add(9 * 32));
    processar(m, u32::from(destino));
}

/// Estado do fluxo entre os quadros, lido das variáveis `v_plc_*`. Os registradores de dados
/// do original viram campos locais: `d0`, `d1` e `d6` são palavras, `d2` e `d5` são longas.
struct Estado {
    /// `v_plc_buffer`: próximo byte do fluxo na ROM.
    a0: u32,
    /// `v_plc_ptrnemcode`: rotina de escrita, que indica o modo XOR.
    a3: u32,
    /// `v_plc_repeatcount`: pixels que ainda faltam do código atual (contador do `dbf`).
    d0: u16,
    /// `v_plc_paletteindex`: nibble de paleta do código atual.
    d1: u16,
    /// `v_plc_previousrow`: linha anterior, acumulada no modo XOR.
    d2: u32,
    /// `v_plc_dataword`: janela de bits (a parte alta não é usada, ver o cabeçalho do módulo).
    d5: u32,
    /// `v_plc_shiftvalue`: posição do próximo bit na janela.
    d6: u16,
}

fn carregar(m: &Memoria) -> Estado {
    Estado {
        a0: rl_ro(m, V_PLC_BUFFER),
        a3: rl_ro(m, V_PLC_PTRNEMCODE),
        d0: u16::try_from(rl_ro(m, V_PLC_REPEATCOUNT) & 0xFFFF).unwrap_or(0),
        d1: u16::try_from(rl_ro(m, V_PLC_PALETTEINDEX) & 0xFFFF).unwrap_or(0),
        d2: rl_ro(m, V_PLC_PREVIOUSROW),
        d5: rl_ro(m, V_PLC_DATAWORD),
        d6: u16::try_from(rl_ro(m, V_PLC_SHIFTVALUE) & 0xFFFF).unwrap_or(0),
    }
}

fn salvar(m: &mut Memoria, e: &Estado) {
    wl(m, V_PLC_BUFFER, e.a0);
    wl(m, V_PLC_PTRNEMCODE, e.a3);
    wl(m, V_PLC_REPEATCOUNT, u32::from(e.d0));
    wl(m, V_PLC_PALETTEINDEX, u32::from(e.d1));
    wl(m, V_PLC_PREVIOUSROW, e.d2);
    wl(m, V_PLC_DATAWORD, e.d5);
    wl(m, V_PLC_SHIFTVALUE, u32::from(e.d6));
}

/// Leitura de uma longa da RAM sem alterar o estado da memória (o `ler` pede `&mut`).
fn rl_ro(m: &Memoria, a: u32) -> u32 {
    let i = usize::try_from(a.wrapping_sub(0xFF_0000)).unwrap_or(0);
    let r = m.ram();
    let mut b = [0u8; 4];
    for (k, d) in b.iter_mut().enumerate() {
        *d = r.get(i.wrapping_add(k)).copied().unwrap_or(0);
    }
    u32::from_be_bytes(b)
}

/// `ProcessPLC`: descomprime tiles a partir de `destino` (endereço da VRAM, em bytes).
fn processar(m: &mut Memoria, destino: u32) {
    // Comando de endereço da VRAM: a palavra alta leva o endereço, a baixa os bits altos dele.
    let comando = (((destino & 0x3FFF) | 0x4000) << 16) | (destino >> 14);
    wl(m, VDP_CONTROLE, comando);
    let mut e = carregar(m);
    let xor = e.a3 == NEM_PCD_WRITE_ROW_TO_VDP_XOR;
    loop {
        // Um tile: oito linhas. `NemPCD_NewRow` e as escritas de linha.
        for _ in 0..8 {
            let linha = decodificar_linha(m, &mut e);
            if xor {
                e.d2 ^= linha;
                wl(m, VDP_DADOS, e.d2);
            } else {
                wl(m, VDP_DADOS, linha);
            }
        }
        // `subq.w #1,(v_plc_patternsleft)`: acabou a arte, a fila anda e o estado não é salvo.
        let restam = rw(m, V_PLC_PATTERNSLEFT).wrapping_sub(1);
        ww(m, V_PLC_PATTERNSLEFT, restam);
        if restam == 0 {
            desempilhar(m);
            return;
        }
        // `subq.w #1,(v_plc_framepatternsleft)`: o quadro acabou quando o contador chega a zero.
        let do_quadro = rw(m, V_PLC_FRAMEPATTERNSLEFT).wrapping_sub(1);
        ww(m, V_PLC_FRAMEPATTERNSLEFT, do_quadro);
        if do_quadro == 0 {
            salvar(m, &e);
            return;
        }
    }
}

/// `ProcessPLC_ShiftCue`: tira a arte acabada da fila (desloca as entradas em 6 bytes). Como no
/// original (FixBugs = 0), o último espaço da fila não é limpo.
fn desempilhar(m: &mut Memoria) {
    let mut a = V_PLC_BUFFER;
    for _ in 0..LONGAS_DO_DESEMPILHAR {
        let v = rl(m, a.wrapping_add(PLC_SLOT_SIZE));
        wl(m, a, v);
        a = a.wrapping_add(4);
    }
}

/// `NemDec_ProcessCompressedData`: lê o próximo código do fluxo e deixa em `d0` a contagem
/// de pixels (menos um) e em `d1` o nibble de paleta. Códigos com prefixo `111111` são
/// inline: 3 bits de contagem e 4 de paleta.
fn decodificar_codigo(m: &mut Memoria, e: &mut Estado) {
    // `move.w d5,d1` e `lsr.w d7,d1`: a palavra baixa da janela, deslocada até o topo.
    let janela = e.d5 & 0xFFFF;
    let temp = janela
        .checked_shr(u32::from(e.d6.wrapping_sub(8)))
        .unwrap_or(0);
    if temp & 0xFF >= 0xFC {
        // `NemPCD_InlineData`: seis bits de prefixo, depois sete bits do código.
        e.d6 = e.d6.wrapping_sub(6);
        carregar_byte_se_preciso(m, e);
        e.d6 = e.d6.wrapping_sub(7);
        let inline = janela_baixa(e).checked_shr(u32::from(e.d6)).unwrap_or(0) & 0xFFFF;
        e.d1 = u16_de(inline) & 0x0F;
        e.d0 = u16_de(inline) & 0x70;
        carregar_byte_se_preciso(m, e);
        e.d0 >>= 4;
        return;
    }
    // Código da tabela: o índice são os 8 bits de `temp`, em palavras.
    let posicao = (temp & 0xFF).wrapping_mul(2);
    let endereco = V_NGFX_BUFFER.wrapping_add(posicao);
    let tamanho = u16::from(rb_ram(m, endereco));
    e.d6 = e.d6.wrapping_sub(tamanho);
    carregar_byte_se_preciso(m, e);
    let baixo = u16::from(rb_ram(m, endereco.wrapping_add(1)));
    e.d1 = baixo & 0x0F;
    e.d0 = (baixo & 0xF0) >> 4;
}

/// A palavra baixa da janela de bits, como `d5` vista pelas instruções de palavra.
fn janela_baixa(e: &Estado) -> u32 {
    e.d5 & 0xFFFF
}

/// Reinterpreta a palavra baixa de um valor de janela, sem mudar nenhum bit.
fn u16_de(v: u32) -> u16 {
    u16::try_from(v & 0xFFFF).unwrap_or(0)
}

/// `cmpi.w #9,d6` e `bhs`: se restam menos de 9 bits na janela, `d6` ganha 8, a janela anda um
/// byte (`asl.w #8,d5`) e o próximo byte do fluxo entra no baixo (`move.b (a0)+,d5`).
fn carregar_byte_se_preciso(m: &mut Memoria, e: &mut Estado) {
    if e.d6 >= 9 {
        return;
    }
    e.d6 = e.d6.wrapping_add(8);
    let alta = e.d5 & 0xFFFF_0000;
    let baixa = (e.d5 & 0xFFFF).wrapping_shl(8) & 0xFFFF;
    e.d5 = alta | baixa | u32::from(rom_b(m, e.a0));
    e.a0 = e.a0.wrapping_add(1);
}

/// Verdadeiro se `a` está fora do arquivo da ROM (a leitura daria zero, sem sentido).
fn fora_da_rom(m: &Memoria, a: u32) -> bool {
    usize::try_from(a).map_or(true, |i| i >= m.rom().len())
}

/// Byte da RAM (leitura sem efeito, para a tabela de códigos).
fn rb_ram(m: &Memoria, a: u32) -> u8 {
    let i = usize::try_from(a.wrapping_sub(0xFF_0000)).unwrap_or(0);
    m.ram()[i]
}

/// Uma linha de 8 pixels (`NemPCD_NewRow` até a escrita da linha). O contador `d0` faz o
/// papel do `dbf` de `NemPCD_WritePixel_Loop`: depois de cada pixel, se ele ainda não chegou
/// a -1, o pixel seguinte usa o mesmo nibble; se chegou, decodifica um código novo.
fn decodificar_linha(m: &mut Memoria, e: &mut Estado) -> u32 {
    let mut d4: u32 = 0;
    let mut d3: u16 = 8;
    let mut decodificar = false;
    loop {
        if !decodificar {
            // `dbf d0,NemPCD_WritePixel`
            e.d0 = e.d0.wrapping_sub(1);
            if e.d0 == 0xFFFF {
                decodificar = true;
            }
        }
        if decodificar {
            decodificar_codigo(m, e);
            decodificar = false;
        }
        // `NemPCD_WritePixel`: `lsl.l #4,d4` e `or.b d1,d4`.
        d4 = d4.wrapping_shl(4) | (u32::from(e.d1) & 0xFF);
        d3 = d3.wrapping_sub(1);
        if d3 == 0 {
            return d4;
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    /// A tabela da ROM de teste: um código de 4 bits (`0xA`) e o fim do fluxo.
    #[test]
    fn construir_tabela_aceita_o_mesmo_fluxo_do_descompressor() {
        let rom = [0x81, 0x04, 0x0A, 0xFF];
        let mut m = Memoria::nova(rom.to_vec());
        let fim = construir_tabela(&mut m, 0).unwrap();
        assert_eq!(fim, 4);
    }

    /// Um fluxo sem o `$FF` final não pode ficar num laço: tem de devolver erro ao sair da ROM.
    #[test]
    fn construir_tabela_recusa_fluxo_sem_fim() {
        let rom = [0x81, 0x04, 0x0A];
        let mut m = Memoria::nova(rom.to_vec());
        assert!(construir_tabela(&mut m, 0).is_err());
    }
}
