//! `Psg::amostra_estavel` é igual a 16 chamadas de `Psg::ciclo` (referência), nas 4 fases do
//! divisor, com escritas aleatórias entre as amostras.
use sonic1_audio::psg::Psg;

/// Gerador congruente simples (determinístico).
struct Rng(u64);
impl Rng {
    fn proximo(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }
}

#[test]
fn amostra_estavel_igual_a_16_ciclos() {
    let mut rng = Rng(99);
    let mut rapidas = 0u32;
    for fase in 0..4u32 {
        let (mut a, mut b) = (Psg::novo(), Psg::novo());
        for _ in 0..fase {
            a.ciclo();
            b.ciclo();
        }
        for _ in 0..20_000 {
            if rng.proximo().checked_rem(7) == Some(0) {
                let v = rng.proximo().to_le_bytes()[0];
                a.escreve(v);
                b.escreve(v);
            }
            for _ in 0..16 {
                a.ciclo();
            }
            if b.amostra_estavel() {
                rapidas = rapidas.saturating_add(1);
            } else {
                for _ in 0..16 {
                    b.ciclo();
                }
            }
            assert_eq!(a.saida_empacotada(), b.saida_empacotada());
            assert_eq!(format!("{a:?}"), format!("{b:?}"));
        }
    }
    // O caminho rápido precisa ser exercitado de fato (a maioria das amostras não tem escrita).
    assert!(rapidas > 40_000, "{rapidas} amostras pelo caminho rápido");
}
