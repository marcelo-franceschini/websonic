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
    let r = nemesis::descomprimir(fluxo_da_rom(mem, origem)?, LIMITE_SAIDA)
        .map_err(ErroRotina::Descompressao)?;
    Ok(gravar(mem, destino, &r.saida))
}
