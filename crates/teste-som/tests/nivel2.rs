//! Experimento do nível 2 (7.8): modelo de custo dos laços com ROM sintética.

use sonic1_core::nivel2::{laco_da_limpeza, laco_da_soma, laco_da_voz};

fn rom_com(a: usize, bytes: &[u8]) -> Vec<u8> {
    let mut r = vec![0u8; 0x1000];
    r[a..a + bytes.len()].copy_from_slice(bytes);
    r
}

/// Modelo independente: simula o laço duplo contando ciclos volta a volta.
fn simula_voz(o: u64, i: u64, nops: u64) -> u64 {
    let mut c = 8; // move.w #o,d1
    let mut d1 = o;
    loop {
        c += 8; // move.w #i,d0
        let mut d0 = i;
        loop {
            c += 4 * nops;
            if d0 == 0 {
                c += 14;
                break;
            }
            c += 10;
            d0 -= 1;
        }
        if d1 == 0 {
            c += 14;
            break;
        }
        c += 10;
        d1 -= 1;
    }
    c
}

#[test]
fn laco_da_voz_confere_com_a_simulacao_volta_a_volta() {
    for (o, i, nops) in [(0u16, 0u16, 1u64), (2, 5, 1), (3, 9, 2), (1, 300, 1)] {
        let mut code = vec![
            0x32,
            0x3C,
            (o >> 8) as u8,
            o as u8,
            0x30,
            0x3C,
            (i >> 8) as u8,
            i as u8,
        ];
        for _ in 0..nops {
            code.extend([0x4E, 0x71]);
        }
        let inicio_interno = 0x108i64;
        let primeiro_nop = 0x10Ci64;
        let dbf0 = primeiro_nop + 2 * i64::try_from(nops).unwrap();
        let d0 = i16::try_from(primeiro_nop - (dbf0 + 2)).unwrap();
        code.extend([0x51, 0xC8, (d0 >> 8) as u8, d0 as u8]);
        let dbf1 = dbf0 + 4;
        let d1 = i16::try_from(inicio_interno - (dbf1 + 2)).unwrap();
        code.extend([0x51, 0xC9, (d1 >> 8) as u8, d1 as u8]);
        let pos_dbf1 = u32::try_from(dbf1).unwrap();
        code.extend([0x58, 0x4F]);
        let rom = rom_com(0x104, &code);
        let p = laco_da_voz(&rom, 0x104).unwrap();
        assert_eq!(
            p.ciclos,
            simula_voz(u64::from(o), u64::from(i), nops),
            "{o} {i} {nops}"
        );
        assert_eq!(p.fim, pos_dbf1 + 4);
    }
}

#[test]
fn forma_desconhecida_nao_e_prevista() {
    assert_eq!(laco_da_voz(&rom_com(0x104, &[0x4E, 0x75]), 0x104), None);
    assert_eq!(laco_da_soma(&rom_com(0x104, &[0, 0]), 0x104), None);
    assert_eq!(laco_da_limpeza(&rom_com(0x104, &[0xFF, 0xFF]), 0x104), None);
}

#[test]
fn nunca_entra_em_panico_com_bytes_quaisquer() {
    let mut x = 12345u32;
    for _ in 0..200 {
        let mut rom = vec![0u8; 0x200];
        for b in rom.iter_mut() {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            *b = x.to_le_bytes()[0];
        }
        for a in [0u32, 4, 0x1F0, 0x1FF, 0xFFFF_FFFF] {
            let _ = laco_da_voz(&rom, a);
            let _ = laco_da_soma(&rom, a);
            let _ = laco_da_limpeza(&rom, a);
        }
    }
}

#[test]
fn laco_da_voz_com_moveq_custa_quatro_ciclos_a_menos_por_constante() {
    // moveq #2,d1 ; moveq #-1,d0 (contagem 65535) ; nop ; dbf d0 ; dbf d1 ; addq.w #4,sp
    let code: Vec<u8> = vec![
        0x72, 0x02, 0x70, 0xFF, 0x4E, 0x71, 0x51, 0xC8, 0xFF, 0xFC, 0x51, 0xC9, 0xFF, 0xF6, 0x58,
        0x4F,
    ];
    let rom = rom_com(0x104, &code);
    let p = laco_da_voz(&rom, 0x104).unwrap();
    // Passadas externas: 3. Constantes: 1 + 3 moveq de 4 ciclos em vez de move.w de 8.
    let esperado = simula_voz(2, 65535, 1) - 4 * (1 + 3);
    assert_eq!(p.ciclos, esperado);
}

#[test]
fn preambulo_da_voz_move_imediato_para_endereco_longo_tem_oito_bytes() {
    // move.b #$88,(abs).l ; move.w #0,(abs).l ; moveq #0,d1 ; moveq #0,d0 ; dbf d0 ; dbf d1 ; addq
    let code: Vec<u8> = vec![
        0x13, 0xFC, 0x00, 0x88, 0x00, 0xA0, 0x1F, 0xFF, // 8 bytes, 20 ciclos
        0x33, 0xFC, 0x00, 0x00, 0x00, 0xA1, 0x11, 0x00, // 8 bytes, 20 ciclos
        0x72, 0x00, 0x70, 0x00, 0x4E, 0x71, 0x51, 0xC8, 0xFF, 0xFC, 0x51, 0xC9, 0xFF, 0xF6, 0x58,
        0x4F,
    ];
    let rom = rom_com(0x104, &code);
    let p = laco_da_voz(&rom, 0x104).unwrap();
    assert_eq!(p.fim, 0x104 + 30);
    assert_eq!(p.ciclos, 40 + simula_voz(0, 0, 1) - 4 * 2);
}
