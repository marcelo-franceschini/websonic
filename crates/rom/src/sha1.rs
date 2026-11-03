//! SHA-1 em Rust puro, sem dependências.
//!
//! O SHA-1 aqui só serve para IDENTIFICAR a ROM (o hash de referência é um SHA-1).
//! Não é usado para segurança.

/// Calcula o SHA-1 de `dados`.
pub fn sha1(dados: &[u8]) -> [u8; 20] {
    let mut h: [u32; 5] = [
        0x6745_2301,
        0xEFCD_AB89,
        0x98BA_DCFE,
        0x1032_5476,
        0xC3D2_E1F0,
    ];

    // Blocos completos de 64 bytes.
    let (blocos, resto) = dados.as_chunks::<64>();
    for bloco in blocos {
        processar(&mut h, bloco);
    }

    // Último bloco (ou dois): resto, byte 0x80, zeros e o tamanho em bits (64 bits, big endian).
    let mut final_ = [0u8; 128];
    final_[..resto.len()].copy_from_slice(resto);
    final_[resto.len()] = 0x80;
    let total = if resto.len() < 56 { 64 } else { 128 };
    let bits = (dados.len() as u64).wrapping_mul(8);
    final_[total - 8..total].copy_from_slice(&bits.to_be_bytes());
    for bloco in final_[..total].as_chunks::<64>().0 {
        processar(&mut h, bloco);
    }

    let mut saida = [0u8; 20];
    for (i, palavra) in h.iter().enumerate() {
        saida[i * 4..i * 4 + 4].copy_from_slice(&palavra.to_be_bytes());
    }
    saida
}

/// Processa um bloco de 64 bytes.
fn processar(h: &mut [u32; 5], bloco: &[u8; 64]) {
    let mut w = [0u32; 80];
    for (i, pedaco) in bloco.as_chunks::<4>().0.iter().enumerate() {
        w[i] = u32::from_be_bytes(*pedaco);
    }
    for i in 16..80 {
        w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
    }

    let [mut a, mut b, mut c, mut d, mut e] = *h;
    for (i, palavra) in w.iter().enumerate() {
        let (f, k) = match i {
            0..=19 => ((b & c) | (!b & d), 0x5A82_7999),
            20..=39 => (b ^ c ^ d, 0x6ED9_EBA1),
            40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1B_BCDC),
            _ => (b ^ c ^ d, 0xCA62_C1D6),
        };
        let temp = a
            .rotate_left(5)
            .wrapping_add(f)
            .wrapping_add(e)
            .wrapping_add(k)
            .wrapping_add(*palavra);
        e = d;
        d = c;
        c = b.rotate_left(30);
        b = a;
        a = temp;
    }

    h[0] = h[0].wrapping_add(a);
    h[1] = h[1].wrapping_add(b);
    h[2] = h[2].wrapping_add(c);
    h[3] = h[3].wrapping_add(d);
    h[4] = h[4].wrapping_add(e);
}

/// Escreve o hash em hexadecimal minúsculo.
pub fn em_hex(hash: &[u8; 20]) -> String {
    hash.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod testes {
    use super::*;

    fn hex_de(dados: &[u8]) -> String {
        em_hex(&sha1(dados))
    }

    // Vetores do FIPS 180 e de implementações de referência.
    #[test]
    fn vazio() {
        assert_eq!(hex_de(b""), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    }

    #[test]
    fn abc() {
        assert_eq!(hex_de(b"abc"), "a9993e364706816aba3e25717850c26c9cd0d89d");
    }

    #[test]
    fn frase_conhecida() {
        assert_eq!(
            hex_de(b"The quick brown fox jumps over the lazy dog"),
            "2fd4e1c67a2d28fced849ee1bb76e7391b93eb12"
        );
    }

    #[test]
    fn um_milhao_de_a() {
        let dados = vec![b'a'; 1_000_000];
        assert_eq!(hex_de(&dados), "34aa973cd4c4daa4f61eeb2bdbad27316534016f");
    }

    // Fronteiras do preenchimento: 55 cabe em um bloco, 56 exige dois, 64 fecha um bloco.
    // Valores conferidos com `sha1sum`.
    #[test]
    fn fronteiras_do_preenchimento() {
        for (tamanho, esperado) in FRONTEIRAS {
            let dados = vec![b'a'; *tamanho];
            assert_eq!(hex_de(&dados), *esperado, "tamanho {tamanho}");
        }
    }

    const FRONTEIRAS: &[(usize, &str)] = &[
        (55, "c1c8bbdc22796e28c0e15163d20899b65621d65a"),
        (56, "c2db330f6083854c99d4b5bfb6e8f29f201be699"),
        (63, "03f09f5b158a7a8cdad920bddc29b81c18a551f5"),
        (64, "0098ba824b5c16427bd7a1122a5a442a25ec644d"),
        (65, "11655326c708d70319be2610e8a57d9a5b959d3b"),
        (119, "ee971065aaa017e0632a8ca6c77bb3bf8b1dfc56"),
        (120, "f34c1488385346a55709ba056ddd08280dd4c6d6"),
        (128, "ad5b3fdbcb526778c2839d2f151ea753995e26a0"),
    ];
}
