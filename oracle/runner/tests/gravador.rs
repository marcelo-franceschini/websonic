//! Gravador de entradas (2.2): `WEBSONIC_GRAVAR=BASE` grava `BASE.s1in` e `BASE.ram`.
//! Este arquivo tem um teste so, porque a variavel de ambiente vale para o processo todo.

use websonic_oracle::rom_sintetica::programa_entrada;
use websonic_oracle::{Entradas, Oraculo};

#[test]
fn grava_a_mascara_de_cada_quadro_e_a_ram_do_ultimo() {
    let base = std::env::temp_dir().join(format!("websonic-gravador-{}", std::process::id()));
    let base = base.to_str().unwrap().to_owned();
    // SAFETY: o teste e o unico codigo deste processo de teste; nenhuma outra thread le o ambiente.
    unsafe { std::env::set_var("WEBSONIC_GRAVAR", &base) };

    let entradas = Entradas::nova(vec![0x00, 0x01, 0x80, 0x48, 0x00]);
    let mut oraculo = Oraculo::novo(programa_entrada(), entradas).unwrap();
    for _ in 0..5 {
        oraculo.executar_quadro().unwrap();
    }
    // Sexto quadro: o botao muda no MEIO do quadro. A gravacao guarda o valor do inicio (00) e o
    // jogo ve o quadro inteiro com ele, entao a reproducao com a linha "00" gera a mesma RAM.
    oraculo.definir_botoes(0x00);
    for _ in 0..10 {
        assert!(oraculo.passo().unwrap().is_none());
    }
    oraculo.definir_botoes(0x01);
    while oraculo.passo().unwrap().is_none() {}
    // O jogo viu o quadro inteiro com o botao solto (bit 0 em 1), apesar da mudanca no meio.
    assert_eq!(oraculo.ler_byte_ram68k(0xFF_0000) & 0x01, 0x01);
    let texto = std::fs::read_to_string(format!("{base}.s1in")).unwrap();
    assert_eq!(texto, "00\n01\n80\n48\n00\n00\n");
    let ram = std::fs::read(format!("{base}.ram")).unwrap();
    assert_eq!(ram, oraculo.ram68k());
    let _ = std::fs::remove_file(format!("{base}.s1in"));
    let _ = std::fs::remove_file(format!("{base}.ram"));
}
