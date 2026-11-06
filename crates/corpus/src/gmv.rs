//! Conversor de filme GMV (Gens Movie) para mascaras do projeto.
//!
//! Fonte do formato: pagina GMV do TASVideos (https://tasvideos.org/EmulatorResources/Gens/GMV),
//! lida em 06/11/2026. Cabecalho de 64 bytes ("Gens Movie TEST" na assinatura), depois 3 bytes por
//! quadro: controle 1, controle 2, byte extra. Bits ativos em zero:
//! 01 Cima, 02 Baixo, 04 Esquerda, 08 Direita, 10 A, 20 B, 40 C, 80 Start.
//! A documentacao nao separa o layout de 3 e 6 botoes; so o controle 1 de 3 botoes e aceito.

use crate::{Erro, MAX_QUADROS};

const CABECALHO: usize = 64;
const ASSINATURA: &[u8] = b"Gens Movie TEST";

/// Converte um GMV. Recusa assinatura errada, controle 1 que nao seja de 3 botoes e filme de 50 Hz.
/// Retorna as mascaras do projeto (ver `lib.rs`).
pub fn converter(dados: &[u8]) -> Result<Vec<u8>, Erro> {
    if dados.len() < CABECALHO || !dados.starts_with(ASSINATURA) {
        return Err(Erro::Formato("assinatura GMV"));
    }
    if dados[0x14] != b'3' {
        return Err(Erro::Formato("controle 1 nao e de 3 botoes"));
    }
    if dados[0x16] & 0x80 != 0 {
        return Err(Erro::Formato("filme de 50 Hz (PAL fora do escopo)"));
    }
    let corpo = &dados[CABECALHO..];
    if !corpo.len().is_multiple_of(3) {
        return Err(Erro::Formato("tamanho do corpo nao e multiplo de 3"));
    }
    let n = corpo.len() / 3;
    if n > MAX_QUADROS {
        return Err(Erro::Grande);
    }
    Ok(corpo
        .as_chunks::<3>()
        .0
        .iter()
        .map(|q| mascara(q[0]))
        .collect())
}

/// Troca de ativo-em-zero para ativo-em-um e reordena A, B, C para o mapa do projeto.
fn mascara(gmv: u8) -> u8 {
    let p = !gmv;
    let mut m = p & 0x0f; // direcoes: mesmos bits
    if p & 0x10 != 0 {
        m |= 0x40; // A
    }
    if p & 0x20 != 0 {
        m |= 0x10; // B
    }
    if p & 0x40 != 0 {
        m |= 0x20; // C
    }
    if p & 0x80 != 0 {
        m |= 0x80; // Start
    }
    m
}

#[cfg(test)]
mod testes {
    use super::*;

    /// Filme inventado por nos (nao e de nenhum jogo).
    fn filme(quadros: &[[u8; 3]], p1: u8, flags: u8) -> Vec<u8> {
        let mut v = vec![0u8; CABECALHO];
        v[..ASSINATURA.len()].copy_from_slice(ASSINATURA);
        v[0x0f] = b'A';
        v[0x14] = p1;
        v[0x15] = b'3';
        v[0x16] = flags;
        for q in quadros {
            v.extend_from_slice(q);
        }
        v
    }

    #[test]
    fn converte_botoes() {
        // sem botao = ff; so Direita = f7; A = ef; B = df; C = bf; Start = 7f.
        let f = filme(
            &[
                [0xff; 3],
                [0xf7, 0xff, 0xff],
                [0xef, 0xff, 0xff],
                [0xdf, 0xff, 0xff],
                [0xbf, 0xff, 0xff],
                [0x7f, 0xff, 0xff],
            ],
            b'3',
            0,
        );
        assert_eq!(converter(&f), Ok(vec![0x00, 0x08, 0x40, 0x10, 0x20, 0x80]));
    }

    #[test]
    fn recusa_malformados() {
        assert!(converter(b"curto").is_err());
        assert!(converter(&filme(&[], b'6', 0)).is_err());
        assert!(converter(&filme(&[], b'3', 0x80)).is_err());
        let mut f = filme(&[[0xff; 3]], b'3', 0);
        f.push(0);
        assert!(converter(&f).is_err());
        let mut f = filme(&[], b'3', 0);
        f[0] = b'X';
        assert!(converter(&f).is_err());
    }

    #[test]
    fn vazio_e_valido() {
        assert_eq!(converter(&filme(&[], b'3', 0)), Ok(vec![]));
    }
}
