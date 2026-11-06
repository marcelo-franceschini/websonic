//! Corpus de entradas: so botoes por quadro (Etapa 2.3).
//!
//! Formato `.s1in`: texto, uma linha por quadro, mascara em 2 digitos hexadecimais, `#` comenta.
//! Bits: Cima 01, Baixo 02, Esquerda 04, Direita 08, B 10, C 20, A 40, Start 80.
//! O indice e o quadro do emulador. Num quadro de lag o jogo ignora a linha sozinho,
//! entao nenhuma conversao corrige lag.
//! Este crate nao contem dado da ROM.

pub mod gmv;
pub mod meta;

/// Maximo de quadros aceito (protecao contra arquivo gigante: 10 horas a 60 Hz).
pub const MAX_QUADROS: usize = 2_160_000;

/// Erros de leitura de entradas e metadados.
#[derive(Debug, PartialEq, Eq)]
pub enum Erro {
    /// Linha do `.s1in` que nao e uma mascara de 2 digitos hexadecimais.
    Linha(usize),
    /// Mais quadros que `MAX_QUADROS`.
    Grande,
    /// Cabecalho ou campo malformado (GMV ou `.meta`).
    Formato(&'static str),
}

/// Le o texto de um `.s1in`. Recusa linha malformada (exige exatamente 2 digitos).
pub fn ler_s1in(texto: &str) -> Result<Vec<u8>, Erro> {
    let mut quadros = Vec::new();
    for (i, linha) in texto.lines().enumerate() {
        let linha = linha.trim();
        if linha.is_empty() || linha.starts_with('#') {
            continue;
        }
        if linha.len() != 2 || !linha.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(Erro::Linha(i.saturating_add(1)));
        }
        let valor = u8::from_str_radix(linha, 16).map_err(|_| Erro::Linha(i.saturating_add(1)))?;
        if quadros.len() >= MAX_QUADROS {
            return Err(Erro::Grande);
        }
        quadros.push(valor);
    }
    Ok(quadros)
}

/// Escreve as mascaras em texto canonico (minusculas, uma por linha).
pub fn escrever_s1in(quadros: &[u8]) -> String {
    let mut saida = String::with_capacity(quadros.len().saturating_mul(3));
    for q in quadros {
        saida.push_str(&format!("{q:02x}\n"));
    }
    saida
}

/// SHA-1 das entradas (mesmo calculo do cabecalho do trace: sobre os bytes das mascaras).
pub fn sha1_das_entradas(quadros: &[u8]) -> [u8; 20] {
    sonic1_fidelidade::hash::sha1(quadros)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn ida_e_volta_identica() {
        let q: Vec<u8> = (0..=255u8).collect();
        let texto = escrever_s1in(&q);
        assert_eq!(ler_s1in(&texto), Ok(q.clone()));
        assert_eq!(
            sha1_das_entradas(&ler_s1in(&texto).unwrap()),
            sha1_das_entradas(&q)
        );
    }

    #[test]
    fn comentarios_e_vazias_sao_ignorados() {
        assert_eq!(ler_s1in("# a\n\n80\n  0c \n"), Ok(vec![0x80, 0x0c]));
    }

    #[test]
    fn malformada_e_recusada() {
        assert_eq!(ler_s1in("00\nzz\n"), Err(Erro::Linha(2)));
        assert_eq!(ler_s1in("0\n"), Err(Erro::Linha(1)));
        assert_eq!(ler_s1in("000\n"), Err(Erro::Linha(1)));
        assert_eq!(ler_s1in("-1\n"), Err(Erro::Linha(1)));
    }

    #[test]
    fn quadro_de_lag_nao_muda_o_arquivo() {
        // O arquivo guarda uma linha por quadro de video. O lag nao e tratado na conversao.
        let q = vec![0x00, 0x80, 0x80, 0x00];
        assert_eq!(ler_s1in(&escrever_s1in(&q)).unwrap(), q);
    }
}
