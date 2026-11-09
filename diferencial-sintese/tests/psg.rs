//! PSG do jogo contra o Nuked-PSG, com escritas sinteticas (sem ROM) em instantes variados.
use sonic1_audio::psg::Psg;
use sonic1_diferencial_sintese::RefPsg;

/// Gerador pseudoaleatorio fixo (xorshift), para o teste ser deterministico.
struct Rng(u64);
impl Rng {
    fn prox(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

/// Roda `ciclos` ciclos nos dois chips com as escritas dadas (ciclo, byte) e compara em cada amostra.
fn compara(escritas: &[(u64, u8)], ciclos: u64) -> Result<usize, String> {
    let mut vistos = std::collections::BTreeSet::new();
    let mut a = Psg::novo();
    let mut r = RefPsg::novo();
    let mut i = 0;
    for c in 0..ciclos {
        while i < escritas.len() && escritas[i].0 == c {
            a.escreve(escritas[i].1);
            r.escreve(escritas[i].1);
            i += 1;
        }
        a.ciclo();
        r.ciclo();
        if c % 16 == 15 {
            let (x, y) = (a.saida_empacotada(), r.le());
            vistos.insert(y);
            if x != y {
                return Err(format!("amostra {} (ciclo {c}): nucleo {x:04X} referencia {y:04X}", c / 16));
            }
        }
    }
    Ok(vistos.len())
}

#[test]
fn silencio_depois_do_reset() {
    assert_eq!(compara(&[], 16 * 200).unwrap(), 1);
}

#[test]
fn tom_simples() {
    // Canal 0: periodo 0x020, volume 0.
    assert!(compara(&[(10, 0x80), (20, 0x02), (30, 0x90)], 16 * 5000).unwrap() >= 2);
}

#[test]
fn ruido_e_tons() {
    let e = [
        (5, 0x80), (6, 0x05), (7, 0x90),
        (8, 0xA0), (9, 0x03), (10, 0xB0),
        (11, 0xC0), (12, 0x01), (13, 0xD0),
        (14, 0xE7), (15, 0xF0),
    ];
    assert!(compara(&e, 16 * 20000).unwrap() > 4);
}

#[test]
fn escritas_aleatorias() {
    for semente in 1..=40u64 {
        let mut g = Rng(semente.wrapping_mul(0x9E3779B97F4A7C15) | 1);
        let ciclos = 16 * 6000;
        let mut e: Vec<(u64, u8)> = (0..300)
            .map(|_| (g.prox() % ciclos, g.prox() as u8))
            .collect();
        e.sort_by_key(|x| x.0);
        // So uma escrita por ciclo (o barramento nao aceita duas).
        e.dedup_by_key(|x| x.0);
        compara(&e, ciclos).unwrap_or_else(|m| panic!("semente {semente}: {m}"));
    }
}
