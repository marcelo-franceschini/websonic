//! Identificação e validação da ROM de referência (REV01).
//!
//! Este crate só compara identificadores (tamanho e SHA-1). Ele recebe os bytes que o
//! usuário escolheu e devolve um veredito. Não guarda nem envia a ROM.
#![forbid(unsafe_code)]

use std::borrow::Cow;

mod sha1;

pub use sha1::{em_hex, sha1};

/// Tamanho da ROM REV01, em bytes (512 KiB).
pub const TAMANHO_REV01: usize = 524_288;

/// SHA-1 da ROM REV01 (igual ao de `refs/rom-hashes.txt`).
pub const SHA1_REV01: &str = "69e102855d4389c3fd1a8f3dc7d193f8eee5fe5b";

/// Por que um arquivo foi recusado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Motivo {
    /// Nenhum byte.
    Vazia,
    /// Tamanho diferente do esperado.
    TamanhoDiferente { esperado: usize, obtido: usize },
    /// Tamanho certo, mas o SHA-1 não é o da REV01.
    HashDiferente,
}

impl Motivo {
    /// Texto curto para mostrar ao usuário.
    pub fn descricao(&self) -> String {
        match self {
            Motivo::Vazia => "o arquivo está vazio".to_string(),
            Motivo::TamanhoDiferente { esperado, obtido } => {
                format!("tamanho {obtido} bytes, a REV01 tem {esperado} bytes")
            }
            Motivo::HashDiferente => "o SHA-1 não é o da REV01".to_string(),
        }
    }
}

/// Resultado da checagem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Veredito {
    Rev01,
    Recusada(Motivo),
}

/// Hash, tamanho e veredito de um arquivo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identificacao {
    /// SHA-1 em hexadecimal minúsculo. Sempre calculado, mesmo se a ROM for recusada.
    pub sha1: String,
    pub tamanho: usize,
    pub veredito: Veredito,
}

/// Tamanho do cabeçalho de copiador Super Magic Drive (.smd), em bytes.
pub const CABECALHO_SMD: usize = 512;

/// Tamanho de cada bloco intercalado no formato Super Magic Drive (16 KiB).
const BLOCO_SMD: usize = 16_384;

/// Identifica cabeçalho Super Magic Drive (.smd) com blocos intercalados de 16 KiB.
pub fn eh_smd_intercalado(dados: &[u8]) -> bool {
    let Some(corpo_len) = dados.len().checked_sub(CABECALHO_SMD) else {
        return false;
    };
    corpo_len >= BLOCO_SMD
        && corpo_len.is_multiple_of(BLOCO_SMD)
        && dados[8] == 0xAA
        && dados[9] == 0xBB
}

/// Desintercala blocos de 16 KiB do formato SMD (8 KiB ímpares seguidos de 8 KiB pares).
pub fn desintercalar_smd(corpo: &[u8]) -> Vec<u8> {
    let metade = BLOCO_SMD / 2;
    let mut saida = vec![0u8; corpo.len()];
    for (bloco_in, bloco_out) in corpo
        .chunks_exact(BLOCO_SMD)
        .zip(saida.chunks_exact_mut(BLOCO_SMD))
    {
        let (impares, pares) = bloco_in.split_at(metade);
        for i in 0..metade {
            bloco_out[i * 2] = pares[i];
            bloco_out[i * 2 + 1] = impares[i];
        }
    }
    saida
}

/// Remove o cabeçalho SMD de 512 bytes e desintercala os blocos de 16 KiB quando necessário.
pub fn normalizar_rom(dados: &[u8], tamanho_esperado: usize) -> Cow<'_, [u8]> {
    if dados.len() == tamanho_esperado.saturating_add(CABECALHO_SMD) {
        let corpo = &dados[CABECALHO_SMD..];
        if eh_smd_intercalado(dados) {
            return Cow::Owned(desintercalar_smd(corpo));
        }
        return Cow::Borrowed(corpo);
    }
    Cow::Borrowed(dados)
}

/// Normaliza uma ROM REV01 (remove cabeçalho SMD e desintercala se aplicável).
pub fn normalizar_rev01(dados: &[u8]) -> Cow<'_, [u8]> {
    normalizar_rom(dados, TAMANHO_REV01)
}

/// Confere se `dados` é a ROM REV01.
pub fn identificar(dados: &[u8]) -> Identificacao {
    identificar_com(dados, TAMANHO_REV01, SHA1_REV01)
}

/// Igual a [`identificar`], com o tamanho e o hash esperados informados.
/// Existe para testar o caminho de sucesso com dados sintéticos, sem a ROM.
pub fn identificar_com(
    dados: &[u8],
    tamanho_esperado: usize,
    sha1_esperado: &str,
) -> Identificacao {
    let conteudo = normalizar_rom(dados, tamanho_esperado);
    let hash = em_hex(&sha1(&conteudo));
    let veredito = if conteudo.is_empty() {
        Veredito::Recusada(Motivo::Vazia)
    } else if conteudo.len() != tamanho_esperado {
        Veredito::Recusada(Motivo::TamanhoDiferente {
            esperado: tamanho_esperado,
            obtido: dados.len(),
        })
    } else if hash != sha1_esperado {
        Veredito::Recusada(Motivo::HashDiferente)
    } else {
        Veredito::Rev01
    };
    Identificacao {
        sha1: hash,
        tamanho: conteudo.len(),
        veredito,
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    /// O hash do código tem de ser o dos metadados do projeto.
    #[test]
    fn constante_confere_com_refs() {
        let refs = include_str!("../../../refs/rom-hashes.txt");
        assert!(
            refs.lines().any(|l| l == format!("sha1: {SHA1_REV01}")),
            "SHA1_REV01 diferente de refs/rom-hashes.txt"
        );
    }

    #[test]
    fn vazio_e_recusado() {
        let r = identificar(&[]);
        assert_eq!(r.veredito, Veredito::Recusada(Motivo::Vazia));
        assert_eq!(r.sha1, "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    }

    #[test]
    fn tamanho_errado_e_recusado_mas_o_hash_aparece() {
        let r = identificar(b"abc");
        assert_eq!(
            r.veredito,
            Veredito::Recusada(Motivo::TamanhoDiferente {
                esperado: TAMANHO_REV01,
                obtido: 3
            })
        );
        assert_eq!(r.sha1, "a9993e364706816aba3e25717850c26c9cd0d89d");
    }

    #[test]
    fn tamanho_certo_hash_errado_e_recusado() {
        let dados = vec![0u8; TAMANHO_REV01];
        let r = identificar(&dados);
        assert_eq!(r.veredito, Veredito::Recusada(Motivo::HashDiferente));
    }

    #[test]
    fn um_byte_a_mais_ou_a_menos_e_recusado() {
        assert!(matches!(
            identificar(&vec![0u8; TAMANHO_REV01 + 1]).veredito,
            Veredito::Recusada(Motivo::TamanhoDiferente { .. })
        ));
        assert!(matches!(
            identificar(&vec![0u8; TAMANHO_REV01 - 1]).veredito,
            Veredito::Recusada(Motivo::TamanhoDiferente { .. })
        ));
    }

    /// Caminho de sucesso com dados sintéticos: o hash esperado é o de "abc".
    #[test]
    fn aceita_quando_tamanho_e_hash_conferem() {
        let r = identificar_com(b"abc", 3, "a9993e364706816aba3e25717850c26c9cd0d89d");
        assert_eq!(r.veredito, Veredito::Rev01);
    }

    #[test]
    fn aceita_com_cabecalho_smd_de_512_bytes() {
        let mut dados = vec![0u8; 512];
        dados.extend_from_slice(b"abc");
        let r = identificar_com(&dados, 3, "a9993e364706816aba3e25717850c26c9cd0d89d");
        assert_eq!(r.veredito, Veredito::Rev01);
        assert_eq!(r.tamanho, 3);
    }

    #[test]
    fn aceita_smd_intercalado_em_blocos_de_16kib() {
        let linear: Vec<u8> = (0..BLOCO_SMD).map(|i| ((i * 7 + 3) & 0xFF) as u8).collect();
        let hash_linear = em_hex(&sha1(&linear));

        let mut smd = vec![0u8; CABECALHO_SMD + BLOCO_SMD];
        smd[0] = 1; // 1 bloco de 16 KiB
        smd[1] = 0x03;
        smd[8] = 0xAA;
        smd[9] = 0xBB;
        smd[10] = 0x06;
        let metade = BLOCO_SMD / 2;
        for i in 0..metade {
            smd[CABECALHO_SMD + i] = linear[i * 2 + 1];
            smd[CABECALHO_SMD + metade + i] = linear[i * 2];
        }

        let norm = normalizar_rom(&smd, BLOCO_SMD);
        assert_eq!(norm.as_ref(), linear.as_slice());

        let r = identificar_com(&smd, BLOCO_SMD, &hash_linear);
        assert_eq!(r.veredito, Veredito::Rev01);
        assert_eq!(r.tamanho, BLOCO_SMD);
        assert_eq!(r.sha1, hash_linear);
    }

    #[test]
    fn recusa_quando_o_hash_esperado_difere() {
        let r = identificar_com(b"abd", 3, "a9993e364706816aba3e25717850c26c9cd0d89d");
        assert_eq!(r.veredito, Veredito::Recusada(Motivo::HashDiferente));
    }

    #[test]
    fn descricao_dos_motivos() {
        assert_eq!(Motivo::Vazia.descricao(), "o arquivo está vazio");
        assert!(Motivo::HashDiferente.descricao().contains("SHA-1"));
        assert!(
            Motivo::TamanhoDiferente {
                esperado: 10,
                obtido: 3
            }
            .descricao()
            .contains("3 bytes")
        );
    }
}
