//! Máquina do Z80 do núcleo contra o Z80 do oráculo (sub-etapa 6.2), com ROM sintética.
//!
//! Os dois programas do Z80 foram montados à mão, a partir das tabelas de opcodes do manual do
//! Z80 (Zilog UM0080), com amostras e tabelas inventadas por nós. Nenhum byte vem da ROM do jogo.

use sonic1_core::memoria::Saida;
use sonic1_diferencial_som::{Op, Roteiro, rodar_nucleo, rodar_oraculo, rom_do_roteiro};

/// Programa A: código de partida (limpa a RAM com LDIR e fica num salto para si mesmo).
const PROGRAMA_A: [u8; 0x26] = [
    0xAF, 0x01, 0xD9, 0x1F, 0x11, 0x27, 0x00, 0x21, 0x26, 0x00, 0xF9, 0x77, 0xED, 0xB0, 0xDD, 0xE1,
    0xFD, 0xE1, 0xED, 0x47, 0xED, 0x4F, 0xD1, 0xE1, 0xF1, 0x08, 0xD9, 0xC1, 0xD1, 0xE1, 0xF1, 0xF9,
    0xF3, 0xED, 0x56, 0x36, 0xE9, 0xE9,
];

/// Programa B: driver de DAC. Tabela de decodificação e de amostras inventadas.
fn programa_b() -> Vec<(u16, u8)> {
    let mut b: Vec<u8> = vec![0; 0xFA];
    let mut p = |a: usize, bytes: &[u8]| b[a..a + bytes.len()].copy_from_slice(bytes);
    p(
        0x00,
        &[
            0xF3, 0xF3, 0xF3, 0x31, 0xFC, 0x1F, 0xDD, 0x21, 0x00, 0x40, 0xAF,
        ],
    );
    p(
        0x0B,
        &[
            0x32, 0xFD, 0x1F, 0x32, 0xFF, 0x1F, 0x3E, 0x00, 0x32, 0x00, 0x60,
        ],
    );
    p(
        0x16,
        &[
            0x06, 0x08, 0x3E, 0x00, 0x32, 0x00, 0x60, 0x0F, 0x10, 0xFA, 0x18, 0x10,
        ],
    );
    // Tabela de decodificação: 16 deltas inventados.
    p(
        0x22,
        &[
            0, 1, 2, 4, 8, 16, 32, 64, 128, 255, 254, 252, 248, 240, 224, 192,
        ],
    );
    p(
        0x32,
        &[
            0x21, 0xFF, 0x1F, 0x7E, 0xB7, 0xF2, 0x35, 0x00, 0xD6, 0x81, 0x77, 0xFE, 0x06,
        ],
    );
    p(
        0x3F,
        &[0x30, 0x78, 0x11, 0x00, 0x00, 0xFD, 0x21, 0xD6, 0x00],
    );
    p(
        0x48,
        &[
            0xCB, 0x27, 0xCB, 0x27, 0xCB, 0x27, 0x06, 0x00, 0x4F, 0xFD, 0x09,
        ],
    );
    p(
        0x53,
        &[
            0xFD, 0x5E, 0x00, 0xFD, 0x56, 0x01, 0xFD, 0x4E, 0x02, 0xFD, 0x46, 0x03, 0xD9,
        ],
    );
    p(
        0x60,
        &[
            0x16, 0x80, 0x21, 0xFD, 0x1F, 0x72, 0xDD, 0x36, 0x00, 0x2B, 0x1E, 0x2A,
        ],
    );
    p(
        0x6C,
        &[
            0xFD, 0x4E, 0x04, 0xDD, 0x72, 0x01, 0x36, 0x00, 0xD9, 0x26, 0x00,
        ],
    );
    p(
        0x77,
        &[
            0x1A, 0xE6, 0xF0, 0x0F, 0x0F, 0x0F, 0x0F, 0xC6, 0x22, 0x6F, 0x7E, 0xD9, 0x82, 0x57,
        ],
    );
    p(
        0x85,
        &[
            0x75, 0xDD, 0x73, 0x00, 0xDD, 0x72, 0x01, 0x74, 0x41, 0x10, 0xFE, 0xD9,
        ],
    );
    p(
        0x91,
        &[0x1A, 0xE6, 0x0F, 0xC6, 0x22, 0x6F, 0x7E, 0xD9, 0x82, 0x57],
    );
    p(
        0x9B,
        &[
            0x75, 0xDD, 0x73, 0x00, 0xDD, 0x72, 0x01, 0x74, 0x41, 0x10, 0xFE, 0xD9,
        ],
    );
    p(
        0xA7,
        &[
            0x3A, 0xFF, 0x1F, 0xCB, 0x7F, 0xC2, 0x32, 0x00, 0x13, 0x0B, 0x79, 0xB0,
        ],
    );
    p(0xB3, &[0xC2, 0x77, 0x00, 0xC3, 0x32, 0x00]);
    // Voz SEGA: DE = $8000 (início da janela), HL = 40 bytes, C = $2A, pitch 3.
    p(0xB9, &[0x11, 0x00, 0x80, 0x21, 0x28, 0x00, 0x0E, 0x2A]);
    p(
        0xC1,
        &[
            0x1A, 0xDD, 0x71, 0x00, 0xDD, 0x77, 0x01, 0x06, 0x03, 0x10, 0xFE,
        ],
    );
    p(
        0xCC,
        &[0x13, 0x2B, 0x7D, 0xB4, 0xC2, 0xC1, 0x00, 0xC3, 0x32, 0x00],
    );
    // Tabela de amostras: início, tamanho, pitch, enchimento (8 bytes cada).
    p(0xD6, &[0xEE, 0x00, 0x04, 0x00, 0x02, 0x00, 0x00, 0x00]);
    p(0xDE, &[0xF2, 0x00, 0x04, 0x00, 0x01, 0x00, 0x00, 0x00]);
    p(0xE6, &[0xF6, 0x00, 0x04, 0x00, 0x03, 0x00, 0x00, 0x00]);
    // Dados das três amostras.
    p(
        0xEE,
        &[
            0x12, 0x34, 0xAB, 0xCD, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22,
        ],
    );
    b.iter()
        .enumerate()
        .map(|(i, v)| (u16::try_from(i).unwrap(), *v))
        .collect()
}

/// Carrega um programa na RAM do Z80 e o inicia, como o código de partida do jogo faz.
fn carregar(programa: &[(u16, u8)], r: &mut Roteiro) {
    r.push(Op::Barramento(true));
    r.push(Op::SoltarReset(true));
    for (a, v) in programa {
        r.push(Op::Escrever(*a, *v));
    }
    r.push(Op::SoltarReset(false));
    r.push(Op::Barramento(false));
    r.push(Op::SoltarReset(true));
}

/// Pede o barramento, lê o status do DAC, escreve `cmd` em `zDAC_Sample` (se houver), devolve.
fn cutucar(r: &mut Roteiro, cmd: Option<u8>) {
    r.push(Op::Barramento(true));
    r.push(Op::Ler(0x1FFD));
    r.push(Op::Ler(0x1FFF));
    if let Some(c) = cmd {
        r.push(Op::Escrever(0x1FFF, c));
    }
    r.push(Op::Barramento(false));
}

fn comparar(roteiro: &Roteiro, quadros: u32) {
    let rom = rom_do_roteiro(roteiro);
    let o = rodar_oraculo(rom.clone(), quadros);
    let n = rodar_nucleo(rom, roteiro, &o.pontos);
    if !n.divergencias.is_empty() {
        for d in &n.divergencias {
            eprintln!("{d:?}");
        }
    }
    assert!(
        n.divergencias.is_empty(),
        "primeira divergencia de posicao: {:?} (de {} no total, {} pontos)",
        n.divergencias.first(),
        n.divergencias.len(),
        o.pontos.len()
    );
    assert_eq!(
        n.pontos_sobrando, 0,
        "pontos gravados que o nucleo nao consumiu"
    );
    assert_eq!(
        n.ym_z80.len(),
        o.ym_z80.len(),
        "numero de escritas do Z80 no YM"
    );
    assert_eq!(n.ym_z80, o.ym_z80, "escritas do Z80 no YM, na ordem");
    assert_eq!(n.ram_z80, o.ram_z80, "RAM do Z80");
    let _ = Saida::Psg(0);
}

#[test]
fn programa_a_limpa_a_ram_e_fica_no_salto_para_si_mesmo() {
    let a: Vec<(u16, u8)> = PROGRAMA_A
        .iter()
        .enumerate()
        .map(|(i, v)| (u16::try_from(i).unwrap(), *v))
        .collect();
    let mut r = Roteiro::new();
    carregar(&a, &mut r);
    // Amostra a cópia em vários instantes, do começo ao fim e depois dela.
    for volta in [1u16, 7, 40, 400, 3000, 9000, 9000, 9000, 9000] {
        r.push(Op::Esperar(volta));
        r.push(Op::Barramento(true));
        r.push(Op::Ler(0x0000));
        r.push(Op::Ler(0x0100));
        r.push(Op::Barramento(false));
    }
    r.push(Op::Barramento(true));
    r.push(Op::Ler(0x0000));
    comparar(&r, 20);
}

#[test]
fn programa_b_toca_amostras_de_dpcm_como_o_oraculo() {
    let mut r = Roteiro::new();
    carregar(&programa_b(), &mut r);
    // O driver inicia, espera, e recebe os três comandos de bateria em instantes variados.
    r.push(Op::Esperar(200));
    cutucar(&mut r, Some(0x81));
    for volta in [1u16, 5, 33, 90, 300] {
        r.push(Op::Esperar(volta));
        cutucar(&mut r, None);
    }
    r.push(Op::Esperar(400));
    cutucar(&mut r, Some(0x82));
    for volta in [2u16, 17, 120] {
        r.push(Op::Esperar(volta));
        cutucar(&mut r, None);
    }
    r.push(Op::Esperar(400));
    cutucar(&mut r, Some(0x83));
    r.push(Op::Esperar(600));
    cutucar(&mut r, None);
    r.push(Op::Barramento(true));
    r.push(Op::Ler(0x0000));
    comparar(&r, 20);
}

#[test]
fn programa_b_toca_a_voz_sega_pela_janela_de_rom() {
    let mut r = Roteiro::new();
    carregar(&programa_b(), &mut r);
    r.push(Op::Esperar(200));
    cutucar(&mut r, Some(0x88));
    for volta in [1u16, 9, 60, 500, 3000] {
        r.push(Op::Esperar(volta));
        cutucar(&mut r, None);
    }
    r.push(Op::Barramento(true));
    r.push(Op::Ler(0x0000));
    comparar(&r, 20);
}
