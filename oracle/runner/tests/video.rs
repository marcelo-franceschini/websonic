//! Testes do video do oraculo (P6 e P7), so com ROM sintetica.

use sha1_ext::*;
use websonic_oracle::rom_sintetica::{PALAVRAS_DMA_VIDEO, programa_video};
use websonic_oracle::{Entradas, Oraculo};

mod sha1_ext {
    pub use sonic1_fidelidade::hash::sha1;
}

/// (numero do quadro, largura, palavras de DMA) por quadro.
type PorQuadro = Vec<(u32, u32, Vec<u16>)>;

fn rodar(quadros: u32) -> (PorQuadro, Vec<u16>) {
    let mut o = Oraculo::novo(programa_video(), Entradas::default()).unwrap();
    let mut dma = Vec::new();
    let mut ultimo = Vec::new();
    for _ in 0..quadros {
        let q = o.executar_quadro().unwrap();
        assert!(o.imagem().erro.is_none());
        dma.push((q.numero, o.imagem().largura, o.dma_do_quadro().to_vec()));
        ultimo = o.imagem().pontos.clone();
    }
    (dma, ultimo)
}

#[test]
fn imagem_so_com_o_fundo_azul_tem_o_hash_calculado_a_mao() {
    let (info, pontos) = rodar(4);
    assert_eq!(info[3].1, 320);
    assert_eq!(pontos.len(), 320 * 224);
    // Azul maximo: BBB=7, GGG=0, RRR=0, ou seja, 7 << 6 = 448 = $01C0.
    assert!(pontos.iter().all(|p| *p == 448));
    // Modelo independente: 71680 pontos de 2 bytes $C0 $01.
    let esperado: Vec<u8> = (0..320 * 224).flat_map(|_| [0xC0u8, 0x01]).collect();
    let bytes: Vec<u8> = pontos.iter().flat_map(|p| p.to_le_bytes()).collect();
    assert_eq!(sha1(&bytes), sha1(&esperado));
}

#[test]
fn dma_entrega_as_palavras_lidas_na_ordem() {
    let (info, _) = rodar(4);
    let todas: Vec<u16> = info.iter().flat_map(|(_, _, p)| p.clone()).collect();
    assert_eq!(todas, PALAVRAS_DMA_VIDEO);
}

#[test]
fn duas_execucoes_dao_o_mesmo_video() {
    assert_eq!(rodar(4), rodar(4));
}
