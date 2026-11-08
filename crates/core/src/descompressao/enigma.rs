//! Enigma. Reproduz a rotina `EniDec` do disassembly.
//!
//! Formato (descrição pública, ver `docs/conhecimento/descompressao.md`):
//! - Cabeçalho de 6 bytes: bits por valor em linha, máscara de 5 bits de flags (prioridade,
//!   paleta, espelhamento), palavra de valor incremental e palavra de valor comum (big-endian).
//! - Depois, um fluxo de bits (do mais significativo ao menos). Prefixos:
//!   `00` + 4 bits de contagem: valor incremental repetido, subindo 1 a cada palavra.
//!   `01` + 4 bits: valor comum repetido.
//!   `1` + modo de 2 bits + contagem de 4 bits: modos 0, 1 e 2 repetem um valor em linha
//!   (igual, subindo 1, descendo 1) e o modo 3 traz uma lista de valores em linha. Modo 3 com
//!   contagem 15 termina o fluxo.
//! - Um valor em linha são os bits de flags presentes na máscara (do mais significativo ao
//!   menos) e depois os bits baixos. A saída soma o tile inicial recebido a cada palavra.
//! - A saída são palavras de 16 bits big-endian.

use super::{Bits, Bytes, Erro, Resultado, empurrar};
use crate::decisao;

/// Descomprime um fluxo Enigma. `tile_inicial` é somado a cada palavra (registrador de
/// entrada do original).
pub fn descomprimir(
    entrada: &[u8],
    tile_inicial: u16,
    limite_saida: usize,
) -> Result<Resultado, Erro> {
    let mut cabecalho = Bytes::novo(entrada);
    let bits_por_valor = cabecalho.byte()?;
    let mascara = cabecalho.byte()?;
    let valor_incremental = cabecalho.palavra_be()?;
    let valor_comum = cabecalho.palavra_be()?;
    if bits_por_valor > 16 {
        return Err(Erro::FormatoInvalido("bits por valor maior que 16"));
    }
    if mascara & 0xE0 != 0 {
        return Err(Erro::FormatoInvalido("mascara de flags maior que 5 bits"));
    }
    let mut incremental = valor_incremental.wrapping_add(tile_inicial);
    let comum = valor_comum.wrapping_add(tile_inicial);

    let mut bits = Bits::novo(cabecalho);
    let mut saida = Vec::new();

    // Valor em linha: flags presentes e bits baixos, mais o tile inicial.
    let valor_em_linha = |bits: &mut Bits<'_>| -> Result<u16, Erro> {
        let mut valor: u16 = 0;
        for k in (0..5u32).rev() {
            if mascara.wrapping_shr(k) & 1 == 1 && bits.bit()? == 1 {
                valor |= 1u16.wrapping_shl(k.wrapping_add(11));
            }
        }
        valor |= bits.bits(bits_por_valor)?;
        Ok(valor.wrapping_add(tile_inicial))
    };
    let empurrar_palavra = |saida: &mut Vec<u8>, palavra: u16| -> Result<(), Erro> {
        for byte in palavra.to_be_bytes() {
            empurrar(saida, byte, limite_saida)?;
        }
        Ok(())
    };

    loop {
        if decisao!("EniDec:copia_em_linha", bits.bit()? == 1) {
            let modo = bits.bits(2)?;
            let contagem = bits.bits(4)?;
            if decisao!("EniDec:modo_lista", modo == 3) {
                if decisao!("EniDec:fim", contagem == 0xF) {
                    break;
                }
                for _ in 0..=contagem {
                    let palavra = valor_em_linha(&mut bits)?;
                    empurrar_palavra(&mut saida, palavra)?;
                }
            } else {
                let mut palavra = valor_em_linha(&mut bits)?;
                for _ in 0..=contagem {
                    empurrar_palavra(&mut saida, palavra)?;
                    palavra = match modo {
                        1 => palavra.wrapping_add(1),
                        2 => palavra.wrapping_sub(1),
                        _ => palavra,
                    };
                }
            }
        } else if decisao!("EniDec:incremental", bits.bit()? == 0) {
            let contagem = bits.bits(4)?;
            for _ in 0..=contagem {
                empurrar_palavra(&mut saida, incremental)?;
                incremental = incremental.wrapping_add(1);
            }
        } else {
            let contagem = bits.bits(4)?;
            for _ in 0..=contagem {
                empurrar_palavra(&mut saida, comum)?;
            }
        }
    }
    Ok(Resultado {
        saida,
        consumidos: bits.posicao(),
    })
}
