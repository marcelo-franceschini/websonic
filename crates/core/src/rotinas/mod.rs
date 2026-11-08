//! Funções do núcleo que reproduzem rotinas do original, uma por rótulo.
//!
//! Cada função lê e escreve a [`Memoria`] como a rotina do original lê e escreve a memória, e
//! devolve só o que o chamador usa depois. Não há registradores como estado: o que o original
//! passa por registrador entra aqui como argumento, e o que ele devolve sai como valor de retorno.
//!
//! Piloto da Etapa 5: os três descompressores (já provados byte a byte na Etapa 3).

use crate::descompressao::{Erro, enigma, kosinski, nemesis};
use crate::memoria::{Memoria, Tamanho};

/// Teto da saída de qualquer descompressão do jogo (a RAM inteira).
/// Buffer de decodificação do Nemesis (`v_ngfx_buffer`).
const V_NGFX_BUFFER: u32 = 0xFF_AA00;
const LIMITE_SAIDA: usize = 0x1_0000;

/// Erros das rotinas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErroRotina {
    Descompressao(Erro),
    /// A origem está fora da ROM.
    OrigemForaDaRom,
}

fn fluxo_da_rom(mem: &Memoria, origem: u32) -> Result<&[u8], ErroRotina> {
    let i = usize::try_from(origem & 0x3F_FFFF).map_err(|_| ErroRotina::OrigemForaDaRom)?;
    mem.rom().get(i..).ok_or(ErroRotina::OrigemForaDaRom)
}

fn gravar(mem: &mut Memoria, destino: u32, bytes: &[u8]) -> u32 {
    let mut fim = destino;
    for b in bytes {
        // Escrita de byte em endereço qualquer nunca dá erro de alinhamento.
        let _ = mem.escrever(fim, Tamanho::Byte, u32::from(*b));
        fim = fim.wrapping_add(1);
    }
    fim
}

/// `KosDec`: descomprime Kosinski de `origem` (ROM) para `destino`. Devolve o endereço depois
/// do último byte escrito (o que o original deixa em A1).
pub fn kos_dec(mem: &mut Memoria, origem: u32, destino: u32) -> Result<u32, ErroRotina> {
    let r = kosinski::descomprimir(fluxo_da_rom(mem, origem)?, LIMITE_SAIDA)
        .map_err(ErroRotina::Descompressao)?;
    Ok(gravar(mem, destino, &r.saida))
}

/// `EniDec`: descomprime Enigma de `origem` (ROM) para `destino`, somando `tile_inicial` a cada
/// palavra. Devolve o endereço depois do último byte escrito.
pub fn eni_dec(
    mem: &mut Memoria,
    origem: u32,
    destino: u32,
    tile_inicial: u16,
) -> Result<u32, ErroRotina> {
    let r = enigma::descomprimir(fluxo_da_rom(mem, origem)?, tile_inicial, LIMITE_SAIDA)
        .map_err(ErroRotina::Descompressao)?;
    Ok(gravar(mem, destino, &r.saida))
}

/// `NemDecToRAM`: descomprime Nemesis de `origem` (ROM) para `destino`. Devolve o endereço depois
/// do último byte escrito.
pub fn nem_dec_to_ram(mem: &mut Memoria, origem: u32, destino: u32) -> Result<u32, ErroRotina> {
    let fluxo = fluxo_da_rom(mem, origem)?;
    let mut n = nemesis::Nemesis::novo(fluxo, LIMITE_SAIDA).map_err(ErroRotina::Descompressao)?;
    n.executar_ate(n.tiles_total())
        .map_err(ErroRotina::Descompressao)?;
    let tabela = n.palavras_da_tabela();
    let saida = n.saida().to_vec();
    // O original monta a tabela de códigos em `v_ngfx_buffer`, e essa RAM é comparada.
    for (desloc, palavra) in tabela {
        let _ = mem.escrever(
            V_NGFX_BUFFER.wrapping_add(desloc),
            Tamanho::Byte,
            u32::from(palavra >> 8),
        );
        let _ = mem.escrever(
            V_NGFX_BUFFER.wrapping_add(desloc).wrapping_add(1),
            Tamanho::Byte,
            u32::from(palavra & 0xFF),
        );
    }
    Ok(gravar(mem, destino, &saida))
}

#[cfg(test)]
mod testes {
    use super::*;

    /// A tabela de códigos aparece em `v_ngfx_buffer`, como na rotina original: um código de 4
    /// bits ocupa as 16 posições que começam com ele, e o resto do buffer fica intocado.
    #[test]
    fn nemesis_grava_a_tabela_de_codigos_na_ram() {
        // Cabeçalho de 1 tile, nibble 1, código 0xA de 4 bits, fim da tabela, 64 códigos.
        let mut rom = vec![0x00, 0x01, 0x81, 0x04, 0x0A, 0xFF];
        rom.extend([0xAA; 32]);
        let mut mem = Memoria::nova(rom);
        let fim = nem_dec_to_ram(&mut mem, 0, 0xFF_0000).unwrap();
        assert_eq!(fim, 0xFF_0020);
        let ram = mem.ram();
        let base = 0xAA00 + 0xA0 * 2;
        for k in 0..16 {
            assert_eq!(ram[base + 2 * k..base + 2 * k + 2], [0x04, 0x01]);
        }
        assert_eq!(ram[base - 1], 0);
        assert_eq!(ram[base + 32], 0);
    }
}
