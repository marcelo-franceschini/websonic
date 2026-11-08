//! Sub-etapas 7.1 e 7.2: boot e verificação da ROM, com ROM sintética escrita por nós.
//! Fontes do comportamento: manual do 68000 (Motorola) e mapa de memória do Mega Drive
//! (Genesis Programming, 68K Memory map). Nada aqui vem da ROM do jogo.

use sonic1_core::abertura::ram::*;
use sonic1_core::abertura::{Abertura, ErroAbertura, EventoQuadro, MARCAS, Marca, boot};
use sonic1_core::memoria::{Memoria, PortaVdp, Saida, Tamanho};

fn poe(rom: &mut [u8], a: u32, bytes: &[u8]) {
    let i = usize::try_from(a).unwrap();
    rom[i..i + bytes.len()].copy_from_slice(bytes);
}

/// ROM sintética com as tabelas nos endereços dos rótulos e a soma certa no cabeçalho.
fn rom_sintetica(semente: u32) -> Vec<u8> {
    let tam = 0x7_3000usize;
    let mut rom = vec![0u8; tam];
    // Bytes pseudoaleatórios (xorshift, semente fixa).
    let mut x = semente | 1;
    for b in rom.iter_mut() {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        *b = x.to_le_bytes()[0];
    }
    poe(
        &mut rom,
        SETUP_VALUES,
        &[0x80, 0x00, 0x3F, 0xFF, 0x01, 0x00],
    );
    for i in 0..24u8 {
        poe(&mut rom, SETUP_VALUES_VDP + u32::from(i), &[0x10 + i]);
    }
    poe(&mut rom, SETUP_VALUES_VDP_END, &[0x40, 0x00, 0x00, 0x80]);
    for i in 0..(SETUP_VALUES_Z80_END - SETUP_VALUES_Z80) {
        poe(
            &mut rom,
            SETUP_VALUES_Z80 + i,
            &[0xE0u8.wrapping_add(i as u8)],
        );
    }
    poe(
        &mut rom,
        SETUP_VALUES_Z80_END,
        &[0x81, 0x04, 0x8F, 0x02, 0xC0, 0, 0, 0, 0x40, 0, 0, 0x10],
    );
    poe(&mut rom, SETUP_VALUES_PSG, &[0x9F, 0xBF, 0xDF, 0xFF]);
    // Tabela da VDP do jogo: 19 palavras.
    for i in 0..19u32 {
        poe(
            &mut rom,
            VDP_SETUP_ARRAY + 2 * i,
            &[0x80 + (i as u8), 0x30 + i as u8],
        );
    }
    // Kosinski mínimo: três literais e o fim.
    poe(
        &mut rom,
        DAC_DRIVER,
        &[0x17, 0x00, 0xAA, 0xBB, 0xCC, 0x00, 0x00, 0x00],
    );
    // Fim da ROM e soma de verificação (modelo direto, independente do núcleo).
    let fim = u32::try_from(tam - 1).unwrap();
    poe(&mut rom, ROM_END_LOC, &fim.to_be_bytes());
    let soma = soma_modelo(&rom);
    poe(&mut rom, CHECKSUM, &soma.to_be_bytes());
    rom
}

/// Soma das palavras de $200 até a última palavra completa, em 16 bits.
fn soma_modelo(rom: &[u8]) -> u16 {
    rom[0x200..]
        .chunks(2)
        .map(|p| u16::from_be_bytes([p[0], p[1]]))
        .fold(0u16, |a, b| a.wrapping_add(b))
}

fn ctrl(m: &Memoria) -> Vec<(u16, Tamanho)> {
    m.saidas()
        .iter()
        .filter_map(|s| match s {
            Saida::Vdp {
                porta: PortaVdp::Controle,
                valor,
                tamanho,
            } => Some((*valor, *tamanho)),
            _ => None,
        })
        .collect()
}

#[test]
fn entry_point_escreve_os_registradores_na_ordem_e_o_programa_do_z80() {
    let rom = rom_sintetica(1);
    let mut m = Memoria::nova(rom.clone());
    boot::entry_point(&mut m).unwrap();
    let c = ctrl(&m);
    // 24 registradores: o valor da tabela no byte baixo e o número do registrador no alto.
    for (i, par) in c.iter().take(24).enumerate() {
        let esperado =
            0x8000 + 0x100 * u16::try_from(i).unwrap() + 0x10 + u16::try_from(i).unwrap();
        assert_eq!(par.0, esperado, "registrador {i}");
    }
    // O programa de partida do Z80 foi para a RAM dele, byte a byte.
    for i in 0..(SETUP_VALUES_Z80_END - SETUP_VALUES_Z80) {
        assert_eq!(
            m.ram_z80()[usize::try_from(i).unwrap()],
            0xE0u8.wrapping_add(i as u8)
        );
    }
    // 4 bytes de volume no PSG, na ordem.
    let psg: Vec<u8> = m
        .saidas()
        .iter()
        .filter_map(|s| {
            if let Saida::Psg(v) = s {
                Some(*v)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(psg, vec![0x9F, 0xBF, 0xDF, 0xFF]);
    // A RAM fica zerada.
    assert!(m.ram().iter().all(|b| *b == 0));
}

#[test]
fn soma_de_verificacao_confere_com_o_modelo_em_muitas_roms() {
    for semente in 1..=60u32 {
        let rom = rom_sintetica(semente.wrapping_mul(2_654_435_761));
        let mut m = Memoria::nova(rom);
        assert_eq!(boot::game_program(&mut m), Ok(()), "semente {semente}");
    }
}

#[test]
fn soma_errada_e_recusada() {
    let mut rom = rom_sintetica(7);
    rom[0x1000] ^= 1;
    let mut m = Memoria::nova(rom);
    assert_eq!(
        boot::game_program(&mut m),
        Err(ErroAbertura::ChecksumErrado)
    );
}

#[test]
fn quadros_com_marcas_levam_ao_modo_sega() {
    let rom = rom_sintetica(3);
    let mut a = Abertura::nova(rom);
    assert!(a.em_trecho_atomico());
    a.executar_quadro(&[EventoQuadro::Marca(Marca::GameProgram)]);
    assert!(a.em_trecho_atomico());
    assert_eq!(a.erro, None);
    for m in [
        Marca::ChecksumOk,
        Marca::VdpSetupGame,
        Marca::DacDriverLoad,
        Marca::JoypadInit,
        Marca::MainGameLoop,
    ] {
        assert!(a.em_trecho_atomico());
        a.executar_quadro(&[EventoQuadro::Marca(m)]);
        assert_eq!(a.erro, None, "{m:?}");
    }
    let ram = a.mem.ram();
    assert_eq!(&ram[0xFFFC..], &[0x69, 0x6E, 0x69, 0x74]);
    assert_eq!(ram[0xFFF8], 0);
    assert_eq!(ram[0xF600], 0);
    assert_eq!(&ram[0xF60C..0xF60E], &[0x80 + 1, 0x31]);
    assert_eq!(&ram[0xF624..0xF626], &[0x8A, 223]);
    // O programa do DAC foi descomprimido na RAM do Z80.
    assert_eq!(&a.mem.ram_z80()[..3], &[0xAA, 0xBB, 0xCC]);
}

#[test]
fn execucao_e_deterministica_e_nunca_entra_em_panico_com_rom_pequena() {
    let rodar = |rom: Vec<u8>| {
        let mut a = Abertura::nova(rom);
        a.executar_quadro(&[EventoQuadro::Marca(Marca::GameProgram)]);
        for m in [
            Marca::ChecksumOk,
            Marca::VdpSetupGame,
            Marca::DacDriverLoad,
            Marca::JoypadInit,
            Marca::MainGameLoop,
        ] {
            a.executar_quadro(&[EventoQuadro::Marca(m)]);
        }
        (a.mem.ram().to_vec(), a.mem.saidas().len(), a.erro)
    };
    assert_eq!(rodar(rom_sintetica(9)), rodar(rom_sintetica(9)));
    // ROM minúscula: erro, nunca pânico.
    let (_, _, erro) = rodar(vec![0; 16]);
    assert!(erro.is_some());
}

#[test]
fn marcas_tem_enderecos_distintos() {
    let mut e: Vec<u32> = MARCAS.iter().map(|m| m.2).collect();
    e.sort();
    e.dedup();
    assert_eq!(e.len(), MARCAS.len());
}
