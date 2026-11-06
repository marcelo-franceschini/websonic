//! Hashes usados no trace. SHA-1 serve para identificar e comparar, nao para seguranca.

pub use sonic1_rom::{em_hex, sha1};

/// Hash de uma lista de pedacos, como se fossem um bloco so.
pub fn sha1_de_partes(partes: &[&[u8]]) -> [u8; 20] {
    let total: usize = partes.iter().map(|p| p.len()).sum();
    let mut junto = Vec::with_capacity(total);
    for parte in partes {
        junto.extend_from_slice(parte);
    }
    sha1(&junto)
}
