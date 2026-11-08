//! Memória do núcleo contra o oráculo (sub-etapa 5.1), com ROM sintética.

use sonic1_core::memoria::{Aviso, Memoria, PortaVdp, Saida, Tamanho};
use sonic1_diferencial::{Bancada, Regs, eventos_do_tipo, rom_com_trecho};
use sonic1_fidelidade::Tipo;

/// Libera o reset do Z80 e pede o barramento (como o jogo faz antes de usar a janela do Z80).
const PREFIXO_Z80: [u16; 8] = [
    0x33FC, 0x0100, 0x00A1, 0x1200, 0x33FC, 0x0100, 0x00A1, 0x1100,
];

fn opcode_leitura(t: Tamanho) -> u16 {
    // MOVE.s (A0),D0
    match t {
        Tamanho::Byte => 0x1010,
        Tamanho::Word => 0x3010,
        Tamanho::Long => 0x2010,
    }
}

fn opcode_escrita(t: Tamanho) -> u16 {
    // MOVE.s D1,(A0)
    match t {
        Tamanho::Byte => 0x1081,
        Tamanho::Word => 0x3081,
        Tamanho::Long => 0x2081,
    }
}

fn mascara(t: Tamanho) -> u32 {
    match t {
        Tamanho::Byte => 0xFF,
        Tamanho::Word => 0xFFFF,
        Tamanho::Long => 0xFFFF_FFFF,
    }
}

fn nucleo_com_prefixo(rom: &[u8], z80: bool) -> Memoria {
    let mut m = Memoria::nova(rom.to_vec());
    // No oraculo, o barramento aberto devolve a ultima palavra buscada: o RTS do trecho.
    m.definir_barramento_aberto(0x4E75);
    if z80 {
        m.escrever(0xA1_1200, Tamanho::Word, 0x0100).unwrap();
        m.escrever(0xA1_1100, Tamanho::Word, 0x0100).unwrap();
        m.tirar_saidas();
    }
    m
}

/// Endereços cujo valor lido NÃO depende do tempo nem do barramento aberto.
fn enderecos_de_leitura() -> Vec<u32> {
    let mut v = vec![
        0x00_0000, 0x00_0100, 0x00_0120, 0x01_FFFE, 0x02_0000, 0x03_FFFE, 0x1F_FFFE, 0x3F_FFFE,
        // Janela do Z80 (sem o YM).
        0xA0_0000, 0xA0_0010, 0xA0_1FFE, 0xA0_2010, 0xA0_6000, 0xA0_6100, 0xA0_8010,
        // E/S.
        0xA1_0000, 0xA1_0001, 0xA1_0002, 0xA1_0003, 0xA1_0005, 0xA1_0007, 0xA1_0009, 0xA1_000B,
        0xA1_000D, 0xA1_000F, 0xA1_0011, 0xA1_0013, 0xA1_0015, 0xA1_0017, 0xA1_001B, 0xA1_001E,
        // RAM e espelhos.
        0xE0_0000, 0xE0_FFFE, 0xF0_0010, 0xFF_0000, 0xFF_8000, 0xFF_FFFE,
    ];
    v.sort_unstable();
    v
}

#[test]
fn leituras_de_byte_palavra_e_longa_iguais_ao_oraculo() {
    let rom = rom_com_trecho(&[]);
    let mut divergencias = Vec::new();
    for z80 in [false, true] {
        for t in [Tamanho::Byte, Tamanho::Word, Tamanho::Long] {
            let mut trecho: Vec<u16> = if z80 { PREFIXO_Z80.to_vec() } else { vec![] };
            trecho.push(opcode_leitura(t));
            let mut banco = Bancada::nova(&trecho);
            for a in enderecos_de_leitura() {
                if t != Tamanho::Byte && a & 1 != 0 {
                    continue;
                }
                // Pokes: valores conhecidos na RAM e nos dois cantos do espelho.
                let pokes = [
                    (0xFF_0000, 0xDE),
                    (0xFF_0001, 0xAD),
                    (0xFF_FFFE, 0xBE),
                    (0xFF_FFFF, 0xEF),
                    (0xFF_8000, 0x12),
                ];
                let mut entrada = Regs::default();
                entrada.d[0] = 0xFFFF_FFFF;
                entrada.a[0] = a;
                let o = banco.rodar(&entrada, &pokes);
                let mut m = nucleo_com_prefixo(&rom, z80);
                for (e, v) in pokes {
                    m.escrever(e, Tamanho::Byte, u32::from(v)).unwrap();
                }
                let lido = m.ler(a, t).unwrap();
                let esperado = o.regs.d[0] & mascara(t);
                if lido != esperado {
                    divergencias.push(format!(
                        "z80={z80} {t:?} {a:06X}: oraculo {esperado:X} nucleo {lido:X}"
                    ));
                }
            }
        }
    }
    assert!(
        divergencias.is_empty(),
        "{} divergencias:\n{}",
        divergencias.len(),
        divergencias.join("\n")
    );
}

#[test]
fn escritas_e_leitura_de_volta_iguais_ao_oraculo() {
    let rom = rom_com_trecho(&[]);
    let enderecos = [
        0xFF_0010u32,
        0xE0_0010,
        0xA0_0010,
        0xA0_2012,
        0xA1_0003,
        0xA1_0005,
        0xA1_0009,
        0xA1_000B,
        0xA1_0015,
        0xA1_0007,
    ];
    let mut divergencias = Vec::new();
    for z80 in [false, true] {
        for t in [Tamanho::Byte, Tamanho::Word, Tamanho::Long] {
            let mut trecho: Vec<u16> = if z80 { PREFIXO_Z80.to_vec() } else { vec![] };
            trecho.push(opcode_escrita(t));
            trecho.push(opcode_leitura(t));
            let mut banco = Bancada::nova(&trecho);
            for a in enderecos {
                if t != Tamanho::Byte && a & 1 != 0 {
                    continue;
                }
                for v in [0x1234_5678u32, 0x0000_00C5, 0xFFFF_FF00] {
                    let mut entrada = Regs::default();
                    entrada.d[0] = 0;
                    entrada.d[1] = v;
                    entrada.a[0] = a;
                    let o = banco.rodar(&entrada, &[]);
                    let mut m = nucleo_com_prefixo(&rom, z80);
                    m.escrever(a, t, v & mascara(t)).unwrap();
                    let lido = m.ler(a, t).unwrap();
                    let esperado = o.regs.d[0] & mascara(t);
                    if lido != esperado {
                        divergencias.push(format!(
                            "z80={z80} {t:?} {a:06X}={v:X}: oraculo {esperado:X} nucleo {lido:X}"
                        ));
                    }
                }
            }
        }
    }
    assert!(
        divergencias.is_empty(),
        "{} divergencias:\n{}",
        divergencias.len(),
        divergencias.join("\n")
    );
}

/// Eventos de saída do oráculo no formato do núcleo.
fn saidas_do_oraculo(eventos: &[sonic1_fidelidade::Evento]) -> Vec<Saida> {
    eventos_do_tipo(
        eventos,
        &[Tipo::Vdp, Tipo::Psg, Tipo::Ym, Tipo::BarramentoZ80],
    )
    .iter()
    .map(|e| match e.tipo {
        Tipo::Vdp => Saida::Vdp {
            porta: if e.a == 0 {
                PortaVdp::Dados
            } else {
                PortaVdp::Controle
            },
            valor: u16::try_from(e.b).unwrap(),
            tamanho: if e.c == 2 {
                Tamanho::Word
            } else {
                Tamanho::Byte
            },
        },
        Tipo::Psg => Saida::Psg(u8::try_from(e.a).unwrap()),
        Tipo::Ym => Saida::Ym {
            porta: u8::try_from(e.a).unwrap(),
            valor: u8::try_from(e.b).unwrap(),
        },
        _ => {
            if e.a == 0 {
                Saida::BusReq(e.b != 0)
            } else {
                Saida::ResetZ80(e.b != 0)
            }
        }
    })
    .collect()
}

#[test]
fn escritas_de_hardware_geram_as_mesmas_saidas() {
    let rom = rom_com_trecho(&[]);
    // (endereco, tamanho, valor). O YM so e alcancavel com o barramento cedido.
    let casos: Vec<(u32, Tamanho, u32, bool)> = vec![
        (0xC0_0000, Tamanho::Word, 0x1234, false),
        (0xC0_0004, Tamanho::Word, 0x8F02, false),
        (0xC0_0004, Tamanho::Long, 0xC000_0000, false),
        (0xC0_0000, Tamanho::Byte, 0x7F, false),
        (0xC0_0005, Tamanho::Byte, 0x81, false),
        (0xC0_0011, Tamanho::Byte, 0x9F, false),
        (0xC0_0010, Tamanho::Word, 0x00BF, false),
        (0xA1_1100, Tamanho::Word, 0x0100, false),
        (0xA1_1100, Tamanho::Byte, 0x01, false),
        (0xA1_1200, Tamanho::Word, 0x0100, false),
        (0xA1_1200, Tamanho::Word, 0x0000, false),
        (0xA0_4000, Tamanho::Byte, 0x22, true),
        (0xA0_4001, Tamanho::Byte, 0x0F, true),
        (0xA0_4002, Tamanho::Byte, 0x2B, true),
        (0xA0_4003, Tamanho::Byte, 0x80, true),
        (0xA0_4002, Tamanho::Word, 0x2B00, true),
    ];
    let mut divergencias = Vec::new();
    for (a, t, v, z80) in casos {
        let mut trecho: Vec<u16> = if z80 { PREFIXO_Z80.to_vec() } else { vec![] };
        trecho.push(opcode_escrita(t));
        let mut banco = Bancada::nova(&trecho);
        let mut entrada = Regs::default();
        entrada.d[1] = v;
        entrada.a[0] = a;
        let o = banco.rodar(&entrada, &[]);
        let mut m = nucleo_com_prefixo(&rom, false);
        if z80 {
            m.escrever(0xA1_1200, Tamanho::Word, 0x0100).unwrap();
            m.escrever(0xA1_1100, Tamanho::Word, 0x0100).unwrap();
        }
        m.escrever(a, t, v & mascara(t)).unwrap();
        let do_oraculo = saidas_do_oraculo(&o.eventos);
        let do_nucleo = m.tirar_saidas();
        if do_oraculo != do_nucleo {
            divergencias.push(format!(
                "{a:06X} {t:?} {v:X}: oraculo {do_oraculo:?} nucleo {do_nucleo:?}"
            ));
        }
    }
    assert!(divergencias.is_empty(), "{}", divergencias.join("\n"));
}

#[test]
fn endereco_impar_em_palavra_e_excecao_no_oraculo_e_erro_no_nucleo() {
    // O oraculo toma a excecao de erro de endereco (vetor 3): o tratador e o laco parado em $300,
    // entao a rotina nao retorna. O nucleo devolve o erro.
    let mut m = Memoria::nova(vec![0; 16]);
    assert!(m.ler(0xFF_0001, Tamanho::Word).is_err());
    let mut banco = Bancada::nova(&[opcode_leitura(Tamanho::Word)]);
    let mut entrada = Regs::default();
    entrada.a[0] = 0xFF_0001;
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| banco.rodar(&entrada, &[])));
    assert!(
        r.is_err(),
        "o oraculo deveria nao retornar (excecao de endereco)"
    );
}

#[test]
fn acesso_sem_dispositivo_gera_aviso_e_nao_mexe_na_ram() {
    let mut m = Memoria::nova(vec![0; 16]);
    m.escrever(0x50_0000, Tamanho::Word, 0x1234).unwrap();
    m.ler(0x50_0000, Tamanho::Word).unwrap();
    assert_eq!(m.avisos().get(&Aviso::EscritaSemDispositivo), Some(&1));
    assert_eq!(m.avisos().get(&Aviso::LeituraSemDispositivo), Some(&1));
    assert!(m.ram().iter().all(|b| *b == 0));
}

#[test]
fn controle_de_3_botoes_igual_ao_oraculo_nas_256_combinacoes() {
    // move.b #$40,$A10009 ; move.b #$40,$A10003 ; move.b $A10003,d0 ;
    // move.b #0,$A10003 ; move.b $A10003,d1
    let trecho = [
        0x13FC, 0x0040, 0x00A1, 0x0009, 0x13FC, 0x0040, 0x00A1, 0x0003, 0x1039, 0x00A1, 0x0003,
        0x13FC, 0x0000, 0x00A1, 0x0003, 0x1239, 0x00A1, 0x0003,
    ];
    let rom = rom_com_trecho(&trecho);
    let mut banco = Bancada::com_rom(rom.clone());
    let mut divergencias = Vec::new();
    for mascara in 0..=255u8 {
        banco.oraculo().definir_botoes(mascara);
        let o = banco.rodar(&Regs::default(), &[]);
        let mut m = Memoria::nova(rom.clone());
        m.definir_botoes(mascara);
        m.escrever(0xA1_0009, Tamanho::Byte, 0x40).unwrap();
        m.escrever(0xA1_0003, Tamanho::Byte, 0x40).unwrap();
        let d0 = m.ler(0xA1_0003, Tamanho::Byte).unwrap();
        m.escrever(0xA1_0003, Tamanho::Byte, 0x00).unwrap();
        let d1 = m.ler(0xA1_0003, Tamanho::Byte).unwrap();
        if (d0, d1) != (o.regs.d[0] & 0xFF, o.regs.d[1] & 0xFF) {
            divergencias.push(format!(
                "mascara {mascara:02X}: oraculo {:02X} {:02X} nucleo {d0:02X} {d1:02X}",
                o.regs.d[0] & 0xFF,
                o.regs.d[1] & 0xFF
            ));
        }
    }
    assert!(divergencias.is_empty(), "{}", divergencias.join("\n"));
}
