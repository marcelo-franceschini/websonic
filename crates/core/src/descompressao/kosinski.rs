//! Kosinski. Reproduz a rotina `KosDec` do disassembly.
//!
//! Formato (descrição pública, ver `docs/conhecimento/descompressao.md`):
//! - Campo de descrição de 16 bits, little-endian, consumido do bit menos significativo.
//! - A próxima palavra de descrição é lida assim que o 16º bit da atual é usado, antes dos
//!   bytes de dados do código em curso.
//! - Bit 1: copia um byte literal.
//! - Bits 0,1: cópia "separada". Dois bytes (baixo, alto): distância de 13 bits e contagem nos
//!   3 bits baixos do alto. Contagem 0 no alto: um terceiro byte dá a contagem. 0 termina o
//!   fluxo e 1 não produz saída.
//! - Bits 0,0 e mais dois bits: cópia "em linha". Contagem 2 a 5 (primeiro bit é o mais
//!   significativo) e um byte de distância, de 1 a 256.

use super::{Bytes, Erro, Resultado, empurrar};
use crate::decisao;

/// Palavra de descrição com recarga antecipada.
struct Descricao {
    palavra: u16,
    restantes: u8,
}

impl Descricao {
    fn nova(bytes: &mut Bytes<'_>) -> Result<Self, Erro> {
        let mut d = Self {
            palavra: 0,
            restantes: 0,
        };
        d.recarregar(bytes)?;
        Ok(d)
    }

    fn recarregar(&mut self, bytes: &mut Bytes<'_>) -> Result<(), Erro> {
        let baixo = bytes.byte()?;
        let alto = bytes.byte()?;
        self.palavra = u16::from_le_bytes([baixo, alto]);
        self.restantes = 16;
        Ok(())
    }

    fn bit(&mut self, bytes: &mut Bytes<'_>) -> Result<bool, Erro> {
        if self.restantes == 0 {
            return Err(Erro::EntradaTruncada);
        }
        let bit = self.palavra & 1 == 1;
        self.palavra = self.palavra.wrapping_shr(1);
        self.restantes = self.restantes.checked_sub(1).ok_or(Erro::EntradaTruncada)?;
        if self.restantes == 0 {
            // Recarga antecipada. Se a entrada acabou aqui, o erro só aparece se outro bit
            // for pedido: um fluxo válido termina antes.
            let _ = self.recarregar(bytes);
        }
        Ok(bit)
    }
}

/// Copia `contagem` bytes da saída, `distancia` bytes para trás.
fn copiar(
    saida: &mut Vec<u8>,
    distancia: usize,
    contagem: usize,
    limite: usize,
) -> Result<(), Erro> {
    for _ in 0..contagem {
        let origem = saida
            .len()
            .checked_sub(distancia)
            .ok_or(Erro::FormatoInvalido("distancia maior que a saida"))?;
        let byte = *saida
            .get(origem)
            .ok_or(Erro::FormatoInvalido("distancia maior que a saida"))?;
        empurrar(saida, byte, limite)?;
    }
    Ok(())
}

/// Descomprime um fluxo Kosinski.
pub fn descomprimir(entrada: &[u8], limite_saida: usize) -> Result<Resultado, Erro> {
    let mut bytes = Bytes::novo(entrada);
    let mut descricao = Descricao::nova(&mut bytes)?;
    let mut saida = Vec::new();
    loop {
        if decisao!("KosDec:literal", descricao.bit(&mut bytes)?) {
            let literal = bytes.byte()?;
            empurrar(&mut saida, literal, limite_saida)?;
            continue;
        }
        if decisao!("KosDec:copia_separada", descricao.bit(&mut bytes)?) {
            let baixo = bytes.byte()?;
            let alto = bytes.byte()?;
            let campo = u32::from(alto & 0xF8).wrapping_shl(5) | u32::from(baixo);
            let distancia = usize::try_from(0x2000u32.wrapping_sub(campo))
                .map_err(|_| Erro::FormatoInvalido("distancia"))?;
            let contagem = if decisao!("KosDec:contagem_no_alto", alto & 7 != 0) {
                usize::from(alto & 7)
                    .checked_add(2)
                    .ok_or(Erro::FormatoInvalido("contagem"))?
            } else {
                let n = bytes.byte()?;
                if decisao!("KosDec:fim", n == 0) {
                    break;
                }
                if decisao!("KosDec:pulo", n == 1) {
                    continue;
                }
                usize::from(n)
                    .checked_add(1)
                    .ok_or(Erro::FormatoInvalido("contagem"))?
            };
            copiar(&mut saida, distancia, contagem, limite_saida)?;
        } else {
            let alto = u8::from(descricao.bit(&mut bytes)?);
            let baixo = u8::from(descricao.bit(&mut bytes)?);
            let contagem = usize::from(alto.wrapping_shl(1) | baixo)
                .checked_add(2)
                .ok_or(Erro::FormatoInvalido("contagem"))?;
            let deslocamento = bytes.byte()?;
            let distancia = 0x100usize.saturating_sub(usize::from(deslocamento));
            copiar(&mut saida, distancia, contagem, limite_saida)?;
        }
    }
    Ok(Resultado {
        saida,
        consumidos: bytes.posicao(),
    })
}
