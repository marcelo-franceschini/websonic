//! Fila de arte por quadros (parte C da Etapa 7): a descompressão Nemesis em passos de 9 tiles
//! tem de escrever na VDP exatamente o que o descompressor completo (`NemDec`) escreve, e deixar
//! a mesma tabela de códigos na RAM. ROM sintética, sem nada da ROM real.

use sonic1_core::abertura::plc;
use sonic1_core::abertura::ram::*;
use sonic1_core::memoria::{Memoria, PortaVdp, Saida, Tamanho};
use sonic1_core::rotinas;

/// Onde o fluxo Nemesis fica na ROM sintética.
const FLUXO: u32 = 0x4_0000;

/// Cabeçalho (número de tiles e bit XOR), tabela de dois códigos de 1 bit, e dados.
fn rom_com(cabecalho: [u8; 2]) -> Vec<u8> {
    let mut rom = vec![0u8; 0x8_0000];
    // Tabela: paleta 3; código `0` com 2 pixels; código `1` com 1 pixel.
    let tabela = [0x83, 0x11, 0x00, 0x01, 0x01, 0xFF];
    let dados = [0x55u8; 256];
    let fluxo: Vec<u8> = cabecalho
        .iter()
        .chain(tabela.iter())
        .chain(dados.iter())
        .copied()
        .collect();
    let inicio = usize::try_from(FLUXO).unwrap_or(0);
    rom[inicio..inicio.wrapping_add(fluxo.len())].copy_from_slice(&fluxo);
    rom
}

fn ram_w(m: &Memoria, a: u32) -> u16 {
    let i = usize::try_from(a.wrapping_sub(0xFF_0000)).unwrap_or(0);
    u16::from_be_bytes([m.ram()[i], m.ram()[i + 1]])
}

/// Escritas de dados da VDP, na ordem, como (valor, tamanho).
fn dados_da_vdp(saidas: Vec<Saida>) -> Vec<(u16, Tamanho)> {
    saidas
        .into_iter()
        .filter_map(|s| match s {
            Saida::Vdp {
                porta: PortaVdp::Dados,
                valor,
                tamanho,
            } => Some((valor, tamanho)),
            _ => None,
        })
        .collect()
}

/// Roda a arte pelo caminho do original (`NemDec`) e devolve as escritas e a tabela da RAM.
fn referencia(rom: Vec<u8>) -> (Vec<(u16, Tamanho)>, Vec<u8>) {
    let mut m = Memoria::nova(rom);
    rotinas::nem_dec(&mut m, FLUXO).unwrap();
    let tabela = m.ram()[0xAA00..0xAC00].to_vec();
    (dados_da_vdp(m.tirar_saidas()), tabela)
}

/// Roda a arte pelo caminho por quadros: `RunPLC` e `ProcessPLC_9Tiles` até acabar.
fn por_quadros(rom: Vec<u8>) -> (Vec<(u16, Tamanho)>, Vec<u8>, u32) {
    let mut m = Memoria::nova(rom);
    plc::enfileirar(&mut m, FLUXO, 0);
    plc::run_plc(&mut m);
    let mut quadros = 0u32;
    while ram_w(&m, V_PLC_PATTERNSLEFT) != 0 {
        plc::process_plc_9(&mut m);
        quadros += 1;
        assert!(quadros < 100, "a arte nunca termina");
    }
    let tabela = m.ram()[0xAA00..0xAC00].to_vec();
    (dados_da_vdp(m.tirar_saidas()), tabela, quadros)
}

#[test]
fn por_quadros_escreve_o_mesmo_que_o_descompressor() {
    // 12 tiles: um quadro leva 9, então são dois quadros (9 e 3).
    let rom = rom_com([0x00, 0x0C]);
    let (esperado, tabela_esperada) = referencia(rom.clone());
    let (obtido, tabela, quadros) = por_quadros(rom);
    assert!(!esperado.is_empty(), "a referencia tem de escrever algo");
    assert_eq!(quadros, 2);
    assert_eq!(obtido, esperado);
    assert_eq!(tabela, tabela_esperada);
}

#[test]
fn por_quadros_com_xor_escreve_o_mesmo_que_o_descompressor() {
    // Bit 15 do cabeçalho liga o XOR: cada linha sai XOR da anterior.
    let rom = rom_com([0x80, 0x0C]);
    let (esperado, _) = referencia(rom.clone());
    let (obtido, _, _) = por_quadros(rom);
    assert_eq!(obtido, esperado);
}

#[test]
fn fila_vazia_nao_mexe_na_ram_nem_na_vdp() {
    let mut m = Memoria::nova(rom_com([0x00, 0x01]));
    plc::run_plc(&mut m);
    plc::process_plc_9(&mut m);
    assert!(m.tirar_saidas().is_empty());
    assert_eq!(ram_w(&m, V_PLC_PATTERNSLEFT), 0);
}
