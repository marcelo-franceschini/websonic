//! Fumaça da comparação com ROM sintética escrita por nós (nada da ROM do jogo).
use sonic1_diferencial_abertura::{Nomes, comparar};
use websonic_oracle::rom_sintetica::RomSintetica;

fn rom(programa: &[u16]) -> Vec<u8> {
    let mut r = RomSintetica::nova(0x00FF_FE00, 0x200);
    r.palavras(0x200, programa);
    r.finalizar()
}

fn em_thread<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(64 << 20)
        .spawn(f)
        .unwrap()
        .join()
        .unwrap()
}

#[test]
fn programa_parado_sem_marcas_nao_diverge() {
    // bra.s *
    let rel = em_thread(|| comparar(rom(&[0x60FE]), 5, &Nomes::carregar("")).unwrap());
    assert_eq!(rel.quadros, 5);
    assert!(rel.zero(), "{:?}", rel.erro_nucleo);
    assert!(rel.marcas_vistas.is_empty());
}

#[test]
fn marca_no_oraculo_sem_o_mesmo_efeito_no_jogo_e_acusada() {
    // jmp $300 ; em $300: bra.s *. O núcleo executa o boot (que escreve na VDP) e o oráculo não.
    let mut p = vec![0x4EF9, 0x0000, 0x0300];
    p.resize(0x80, 0x4E71);
    p.push(0x60FE);
    let rel = em_thread(move || comparar(rom(&p), 3, &Nomes::carregar("")).unwrap());
    assert_eq!(rel.marcas_vistas.get("GameProgram").map(Vec::len), Some(1));
    assert!(!rel.zero());
}
