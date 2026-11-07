//! Mapa de acessos a RAM por contexto (pedido P2). ROM sintetica escrita por nos.

use websonic_oracle::rom_sintetica::programa_vblank;
use websonic_oracle::{Entradas, Oraculo};

#[test]
fn contador_do_vblank_fica_so_no_contexto_vblank() {
    oracle_trace::acessos_ligar();
    let mut oraculo = Oraculo::novo(programa_vblank(), Entradas::default()).unwrap();
    for _ in 0..4 {
        oraculo.executar_quadro().unwrap();
    }
    let mapa = oracle_trace::mapa_acessos();
    // O contador $FF0100..$FF0103 e lido e escrito so dentro da interrupcao de VBlank.
    // Bits: contexto 1 (VBlank) = leitura 3, escrita 4. O laco principal (bits 0 a 2) nao toca.
    for (i, bits) in mapa[0x100..0x104].iter().enumerate() {
        assert_eq!(bits & 0b111, 0, "o laco principal nao toca em {i:#x}");
        assert_eq!(bits & 0b11_1000, 0b01_1000, "VBlank le e escreve {i:#x}");
    }
    // O quadro de excecao ($FF7FFA a $FF7FFF) e escrito no laco principal (antes de entrar na
    // interrupcao) e lido no VBlank (pelo RTE).
    for (i, bits) in mapa[0x7FFA..0x8000].iter().enumerate() {
        assert_ne!(bits & 0b10, 0, "escrita no principal em {i:#x}");
        assert_ne!(bits & 0b1000, 0, "leitura no VBlank em {i:#x}");
    }
    // Nada mais no mapa fora do contador e da pilha.
    let tocados = mapa.iter().filter(|b| **b != 0).count();
    assert_eq!(tocados, 4 + 6);
}
