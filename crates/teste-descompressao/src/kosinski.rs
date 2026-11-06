//! Compressor Kosinski de teste (guloso). Reproduz o mesmo layout de bits que o descompressor
//! lê, inclusive a recarga antecipada da palavra de descrição.

struct Escritor {
    saida: Vec<u8>,
    /// Posição da palavra de descrição em preparo.
    slot: usize,
    palavra: u16,
    usados: u8,
}

impl Escritor {
    fn novo() -> Self {
        Self {
            saida: vec![0, 0],
            slot: 0,
            palavra: 0,
            usados: 0,
        }
    }

    fn bit(&mut self, b: bool) {
        if b {
            self.palavra |= 1 << self.usados;
        }
        self.usados += 1;
        if self.usados == 16 {
            let [lo, hi] = self.palavra.to_le_bytes();
            self.saida[self.slot] = lo;
            self.saida[self.slot + 1] = hi;
            self.slot = self.saida.len();
            self.saida.extend_from_slice(&[0, 0]);
            self.palavra = 0;
            self.usados = 0;
        }
    }

    fn fim(mut self) -> Vec<u8> {
        let [lo, hi] = self.palavra.to_le_bytes();
        self.saida[self.slot] = lo;
        self.saida[self.slot + 1] = hi;
        self.saida
    }
}

fn copia_separada(e: &mut Escritor, dist: usize, cont: usize) {
    // dist 1..=8192; cont 3..=9 (curta) ou 10..=256 (longa)
    let campo = 0x2000 - dist;
    let baixo = (campo & 0xFF) as u8;
    let alto_bits = ((campo >> 5) & 0xF8) as u8;
    e.bit(false);
    e.bit(true);
    e.saida.push(baixo);
    if (3..=9).contains(&cont) {
        e.saida.push(alto_bits | (cont - 2) as u8);
    } else {
        e.saida.push(alto_bits);
        e.saida.push((cont - 1) as u8);
    }
}

/// Comprime com busca gulosa. `usar_longas` liga as cópias longas.
pub fn comprimir(dados: &[u8]) -> Vec<u8> {
    let mut e = Escritor::novo();
    let mut i = 0;
    while i < dados.len() {
        // Melhor correspondência na janela de 8192 bytes.
        let (mut melhor_cont, mut melhor_dist) = (0usize, 0usize);
        let inicio = i.saturating_sub(8192);
        for j in inicio..i {
            let mut k = 0;
            while i + k < dados.len() && k < 256 && dados[j + k] == dados[i + k] {
                k += 1;
            }
            if k > melhor_cont {
                melhor_cont = k;
                melhor_dist = i - j;
            }
        }
        if (2..=5).contains(&melhor_cont) && melhor_dist <= 256 {
            let c = melhor_cont;
            e.bit(false);
            e.bit(false);
            e.bit((c - 2) & 2 != 0);
            e.bit((c - 2) & 1 != 0);
            e.saida.push((0x100 - melhor_dist) as u8);
            i += c;
        } else if melhor_cont >= 3 {
            copia_separada(&mut e, melhor_dist, melhor_cont);
            i += melhor_cont;
        } else {
            e.bit(true);
            e.saida.push(dados[i]);
            i += 1;
        }
    }
    // Terminador: bits 0,1 e bytes 00 F0 00.
    e.bit(false);
    e.bit(true);
    e.saida.extend_from_slice(&[0x00, 0xF0, 0x00]);
    e.fim()
}
