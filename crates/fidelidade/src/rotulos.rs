//! Mapa de rotulos: nomes, tipos e valores. So metadado, sem dado da ROM.
//!
//! Formato de `refs/rotulos.txt`, uma entrada por linha, campos separados por TAB:
//!
//! ```text
//! nome<TAB>tipo<TAB>valor_em_hexadecimal
//! ```
//!
//! `tipo` e `rotina`, `variavel` ou `constante`. Linhas vazias e linhas iniciadas por `#` sao
//! ignoradas. Qualquer outra linha fora do padrao e erro: o formato rejeita o que nao e metadado.

use std::collections::BTreeMap;
use std::fmt;

/// Tipo de rotulo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TipoRotulo {
    Rotina,
    Variavel,
    Constante,
}

impl TipoRotulo {
    fn de_texto(texto: &str) -> Option<Self> {
        match texto {
            "rotina" => Some(Self::Rotina),
            "variavel" => Some(Self::Variavel),
            "constante" => Some(Self::Constante),
            _ => None,
        }
    }

    pub fn nome(self) -> &'static str {
        match self {
            Self::Rotina => "rotina",
            Self::Variavel => "variavel",
            Self::Constante => "constante",
        }
    }
}

/// Erro de formato, com o numero da linha (a partir de 1).
#[derive(Debug, PartialEq, Eq)]
pub struct ErroRotulo {
    pub linha: usize,
    pub motivo: &'static str,
}

impl fmt::Display for ErroRotulo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "linha {}: {}", self.linha, self.motivo)
    }
}

impl std::error::Error for ErroRotulo {}

/// Rotulos ordenados por nome (ordem deterministica).
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Rotulos {
    mapa: BTreeMap<String, (TipoRotulo, u32)>,
}

fn nome_valido(nome: &str) -> bool {
    !nome.is_empty()
        && nome.len() <= 128
        && nome
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '@'))
}

impl Rotulos {
    /// Le o texto inteiro. Para na primeira linha invalida.
    pub fn analisar(texto: &str) -> Result<Self, ErroRotulo> {
        let mut mapa = BTreeMap::new();
        for (i, linha) in texto.lines().enumerate() {
            let numero = i + 1;
            if linha.is_empty() || linha.starts_with('#') {
                continue;
            }
            let campos: Vec<&str> = linha.split('\t').collect();
            let [nome, tipo, valor] = campos.as_slice() else {
                return Err(ErroRotulo {
                    linha: numero,
                    motivo: "esperados 3 campos separados por TAB",
                });
            };
            if !nome_valido(nome) {
                return Err(ErroRotulo {
                    linha: numero,
                    motivo: "nome invalido",
                });
            }
            let Some(tipo) = TipoRotulo::de_texto(tipo) else {
                return Err(ErroRotulo {
                    linha: numero,
                    motivo: "tipo desconhecido",
                });
            };
            if valor.is_empty()
                || valor.len() > 8
                || !valor
                    .chars()
                    .all(|c| c.is_ascii_digit() || matches!(c, 'a'..='f'))
            {
                return Err(ErroRotulo {
                    linha: numero,
                    motivo: "valor deve ser hexadecimal minusculo de 1 a 8 digitos",
                });
            }
            let Ok(valor) = u32::from_str_radix(valor, 16) else {
                return Err(ErroRotulo {
                    linha: numero,
                    motivo: "valor invalido",
                });
            };
            if mapa.insert((*nome).to_owned(), (tipo, valor)).is_some() {
                return Err(ErroRotulo {
                    linha: numero,
                    motivo: "nome repetido",
                });
            }
        }
        Ok(Self { mapa })
    }

    /// Valor de um rotulo pelo nome.
    pub fn resolver(&self, nome: &str) -> Option<u32> {
        self.mapa.get(nome).map(|(_, valor)| *valor)
    }

    /// Tipo de um rotulo pelo nome.
    pub fn tipo(&self, nome: &str) -> Option<TipoRotulo> {
        self.mapa.get(nome).map(|(tipo, _)| *tipo)
    }

    pub fn len(&self) -> usize {
        self.mapa.len()
    }

    pub fn is_empty(&self) -> bool {
        self.mapa.is_empty()
    }

    /// Linha canonica de uma entrada, para quem gera o arquivo.
    pub fn formatar(nome: &str, tipo: TipoRotulo, valor: u32) -> String {
        format!("{nome}\t{}\t{valor:x}", tipo.nome())
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn le_entradas_e_resolve() {
        let r =
            Rotulos::analisar("# comentario\n\nmeu_rotulo\trotina\t200\nv_var\tvariavel\tffff00\n")
                .unwrap();
        assert_eq!(r.len(), 2);
        assert_eq!(r.resolver("meu_rotulo"), Some(0x200));
        assert_eq!(r.tipo("v_var"), Some(TipoRotulo::Variavel));
        assert_eq!(r.resolver("nao_existe"), None);
    }

    #[test]
    fn rejeita_o_que_nao_e_metadado() {
        // Linha de listagem com bytes, espacos em vez de TAB, valor com prefixo, nome com espaco.
        for ruim in [
            "000200 4E71 4E71 nop",
            "a rotina 200",
            "x\trotina\t$200",
            "x\trotina\t0x200",
            "x y\trotina\t200",
            "x\trotina\t200\textra",
            "x\toutro\t200",
            "x\trotina\t123456789",
            "x\trotina\tABC",
            "x\trotina\t",
        ] {
            assert!(
                Rotulos::analisar(ruim).is_err(),
                "deveria rejeitar: {ruim:?}"
            );
        }
    }

    #[test]
    fn rejeita_nome_repetido_e_informa_a_linha() {
        let e = Rotulos::analisar("a\trotina\t1\nb\trotina\t2\na\trotina\t3\n").unwrap_err();
        assert_eq!(
            e,
            ErroRotulo {
                linha: 3,
                motivo: "nome repetido"
            }
        );
    }

    #[test]
    fn formatar_volta_no_analisar() {
        let linha = Rotulos::formatar("VBlank", TipoRotulo::Rotina, 0x1a2b);
        let r = Rotulos::analisar(&linha).unwrap();
        assert_eq!(r.resolver("VBlank"), Some(0x1a2b));
    }
}
