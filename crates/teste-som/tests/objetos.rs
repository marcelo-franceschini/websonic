//! Sub-etapa 7.5: `DisplaySprite` e `BuildSprites` com mapeamento sintético (nada da ROM do jogo).
//! Fonte do formato da tabela de sprites da VDP: Sega Retro, "VDP sprite table" (8 bytes por
//! sprite: y, tamanho, ligação, tile, x).

use sonic1_core::abertura::objetos::{build_sprites, display_sprite};
use sonic1_core::abertura::ram::*;
use sonic1_core::memoria::Memoria;

const MAPA: usize = 0x1000;

fn rom() -> Vec<u8> {
    let mut r = vec![0u8; 0x2000];
    // Quadro 0 começa 2 bytes depois do início da tabela de deslocamentos.
    r[MAPA..MAPA + 2].copy_from_slice(&2u16.to_be_bytes());
    let pecas: [[u8; 5]; 2] = [
        [0xF8, 0x05, 0x00, 0x12, 0xF0],
        [0x10, 0x0A, 0x00, 0x34, 0x08],
    ];
    r[MAPA + 2] = 2;
    for (i, p) in pecas.iter().enumerate() {
        r[MAPA + 3 + 5 * i..MAPA + 8 + 5 * i].copy_from_slice(p);
    }
    r
}

fn objeto(m: &mut Memoria, a0: u32, render: u8) {
    let ram = m.ram_mut();
    let o = usize::try_from(a0 & 0xFFFF).unwrap();
    ram[o] = 1; // id
    ram[o + 1] = render;
    ram[o + 2..o + 4].copy_from_slice(&0x2000u16.to_be_bytes());
    ram[o + 4..o + 8].copy_from_slice(&u32::try_from(MAPA).unwrap().to_be_bytes());
    ram[o + 8..o + 10].copy_from_slice(&0x120u16.to_be_bytes()); // x
    ram[o + 0xA..o + 0xC].copy_from_slice(&0x100u16.to_be_bytes()); // y de tela
    ram[o + 0x18] = 0; // prioridade 0
    ram[o + 0x1A] = 0; // quadro 0
}

/// Modelo direto: as peças prontas para a VDP.
fn modelo(xflip: bool, yflip: bool) -> Vec<[u16; 4]> {
    let pecas: [(i16, u8, u16, i16); 2] = [(-8, 5, 0x12, -16), (0x10, 0x0A, 0x34, 8)];
    let (d2, d3) = (0x100i16, 0x120i16);
    pecas
        .iter()
        .enumerate()
        .map(|(i, &(y, tam, tile, x))| {
            let mut y2 = y;
            if yflip {
                y2 = -y - i16::from(((tam << 3) & 0x18) + 8);
            }
            let mut x2 = x;
            if xflip {
                x2 = -x - i16::from((tam.wrapping_mul(2) & 0x18) + 8);
            }
            let mut t = tile.wrapping_add(0x2000);
            if xflip {
                t ^= 0x0800;
            }
            if yflip {
                t ^= 0x1000;
            }
            let xv = ((x2 + d3) as u16) & 0x1FF;
            [
                (y2 + d2) as u16,
                (u16::from(tam) << 8) | u16::try_from(i + 1).unwrap(),
                t,
                if xv == 0 { 1 } else { xv },
            ]
        })
        .collect()
}

fn lido(m: &Memoria) -> Vec<[u16; 4]> {
    let b = &m.ram()[0xF800..];
    (0..2)
        .map(|i| {
            let e = &b[8 * i..8 * i + 8];
            [
                u16::from_be_bytes([e[0], e[1]]),
                u16::from_be_bytes([e[2], e[3]]),
                u16::from_be_bytes([e[4], e[5]]),
                u16::from_be_bytes([e[6], e[7]]),
            ]
        })
        .collect()
}

#[test]
fn as_quatro_orientacoes_conferem_com_o_modelo() {
    for (render, xf, yf) in [
        (0u8, false, false),
        (1, true, false),
        (2, false, true),
        (3, true, true),
    ] {
        let mut m = Memoria::nova(rom());
        let a0 = 0xFF_D080;
        objeto(&mut m, a0, render);
        display_sprite(&mut m, a0);
        build_sprites(&mut m).unwrap();
        assert_eq!(lido(&m), modelo(xf, yf), "orientação {render}");
        // Contador de sprites e fim da lista (ligação zero depois do último).
        assert_eq!(m.ram()[0xF62C], 2);
        assert_eq!(&m.ram()[0xF800 + 16..0xF800 + 20], &[0, 0, 0, 0]);
    }
}

#[test]
fn prioridade_menor_desenha_antes() {
    let mut m = Memoria::nova(rom());
    objeto(&mut m, 0xFF_D080, 0);
    objeto(&mut m, 0xFF_D0C0, 0);
    m.ram_mut()[0xD0C0 + 0x18 - 0xD000 + 0xD000 - 0xD000] = 0; // mantém a prioridade 0
    // O segundo objeto vai para a camada de prioridade $200 (índice 2).
    m.ram_mut()[0xD0C0 + 0x18] = 0x02;
    display_sprite(&mut m, 0xFF_D0C0);
    display_sprite(&mut m, 0xFF_D080);
    build_sprites(&mut m).unwrap();
    assert_eq!(m.ram()[0xF62C], 4);
    // O objeto de prioridade 0 (x $120) vem primeiro; o outro tem a ligação 3 e 4.
    let b = &m.ram()[0xF800..];
    assert_eq!(b[3], 1);
    assert_eq!(b[8 * 2 + 3], 3);
}

#[test]
fn nunca_entra_em_panico_com_fila_e_mapas_aleatorios() {
    let mut x = 99u32;
    for _ in 0..200 {
        let mut r = vec![0u8; 0x2000];
        for b in r.iter_mut() {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            *b = x.to_le_bytes()[0];
        }
        let mut m = Memoria::nova(r);
        for k in 0..6u32 {
            let a0 = 0xFF_D000 + 0x40 * k;
            objeto(&mut m, a0, (x >> 3) as u8);
            m.ram_mut()[usize::try_from(a0 & 0xFFFF).unwrap() + 0x1A] = (x >> 7) as u8;
            m.ram_mut()[usize::try_from(a0 & 0xFFFF).unwrap() + 0x18] = (x >> 11) as u8;
            display_sprite(&mut m, a0);
        }
        let _ = build_sprites(&mut m);
    }
}
