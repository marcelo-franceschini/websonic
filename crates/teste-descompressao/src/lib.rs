//! Compressores de teste dos três formatos. Servem só para gerar entradas válidas para os
//! descompressores do núcleo. Não usam nenhum conteúdo da ROM. Não entram no jogo.

pub mod enigma;
pub mod kosinski;
pub mod nemesis;

/// Gerador pseudoaleatório determinístico (xorshift64), igual em nativo e em WebAssembly.
pub struct Gerador(u64);

impl Gerador {
    pub fn novo(semente: u64) -> Self {
        Self(semente | 1)
    }

    pub fn proximo(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// Valor em `0..limite` (limite maior que zero).
    pub fn ate(&mut self, limite: u64) -> u64 {
        self.proximo() % limite
    }

    pub fn bytes(&mut self, n: usize) -> Vec<u8> {
        (0..n).map(|_| self.proximo().to_le_bytes()[0]).collect()
    }

    /// Bytes com repetições e corridas, para exercitar as cópias.
    pub fn bytes_repetitivos(&mut self, n: usize) -> Vec<u8> {
        let mut v: Vec<u8> = Vec::new();
        while v.len() < n {
            match self.ate(4) {
                0 => v.push(self.proximo().to_le_bytes()[0]),
                1 => {
                    let b = self.proximo().to_le_bytes()[0];
                    let k = self.ate(20) as usize + 1;
                    v.extend(std::iter::repeat_n(b, k));
                }
                2 if !v.is_empty() => {
                    let dist = self.ate(v.len() as u64) as usize + 1;
                    let k = self.ate(40) as usize + 1;
                    for _ in 0..k {
                        let b = v[v.len() - dist];
                        v.push(b);
                    }
                }
                _ => v.push(0),
            }
        }
        v.truncate(n);
        v
    }
}
