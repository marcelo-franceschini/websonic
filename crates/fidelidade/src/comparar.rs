//! Comparacao de traces. Para na primeira divergencia, no quadro e na categoria.

use crate::formato::{Categoria, Erro, Leitor, Quadro};
use std::io::Read;

/// Onde a divergencia aconteceu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Onde {
    /// Os cabecalhos diferem, entao os traces nao sao comparaveis.
    Cabecalho,
    /// Um trace acabou antes do outro.
    Tamanho { esquerda: bool },
    /// Numero do quadro diferente.
    NumeroDoQuadro,
    /// Relogio mestre no fim do quadro diferente.
    RelogioFinal,
    /// Hash de uma categoria diferente. Se for `Eventos`, `evento` aponta o primeiro evento que difere.
    Categoria {
        categoria: Categoria,
        evento: Option<usize>,
    },
}

/// Primeira divergencia achada.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Divergencia {
    /// Numero do quadro (0 para o cabecalho).
    pub quadro: u32,
    pub onde: Onde,
}

/// Compara dois quadros. Devolve a primeira divergencia, na ordem das categorias.
pub fn comparar_quadros(a: &Quadro, b: &Quadro) -> Option<Divergencia> {
    let aqui = |onde| {
        Some(Divergencia {
            quadro: a.numero,
            onde,
        })
    };
    if a.numero != b.numero {
        return aqui(Onde::NumeroDoQuadro);
    }
    if a.mclk_fim != b.mclk_fim {
        return aqui(Onde::RelogioFinal);
    }
    for categoria in Categoria::TODAS {
        let i = categoria as usize;
        // O hash de eventos e refeito dos eventos, para nao confiar no arquivo.
        let (ha, hb) = if categoria == Categoria::Eventos {
            (
                Quadro::hash_dos_eventos(&a.eventos),
                Quadro::hash_dos_eventos(&b.eventos),
            )
        } else {
            (a.hashes[i], b.hashes[i])
        };
        if ha != hb {
            let evento = (categoria == Categoria::Eventos).then(|| {
                a.eventos
                    .iter()
                    .zip(&b.eventos)
                    .position(|(x, y)| x != y)
                    .unwrap_or_else(|| a.eventos.len().min(b.eventos.len()))
            });
            return aqui(Onde::Categoria { categoria, evento });
        }
    }
    None
}

/// Compara dois traces em fluxo, quadro a quadro.
pub fn comparar_leitores<A: Read, B: Read>(
    esquerda: &mut Leitor<A>,
    direita: &mut Leitor<B>,
) -> Result<Option<Divergencia>, Erro> {
    if esquerda.cabecalho != direita.cabecalho {
        return Ok(Some(Divergencia {
            quadro: 0,
            onde: Onde::Cabecalho,
        }));
    }
    let mut proximo_numero = 0u32;
    loop {
        match (esquerda.proximo()?, direita.proximo()?) {
            (None, None) => return Ok(None),
            (Some(_), None) => {
                return Ok(Some(Divergencia {
                    quadro: proximo_numero,
                    onde: Onde::Tamanho { esquerda: false },
                }));
            }
            (None, Some(_)) => {
                return Ok(Some(Divergencia {
                    quadro: proximo_numero,
                    onde: Onde::Tamanho { esquerda: true },
                }));
            }
            (Some(a), Some(b)) => {
                if let Some(d) = comparar_quadros(&a, &b) {
                    return Ok(Some(d));
                }
                proximo_numero = a.numero.saturating_add(1);
            }
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::formato::testes_comuns::{quadro, serializar};

    fn leitor(quadros: &[Quadro]) -> Leitor<std::io::Cursor<Vec<u8>>> {
        let (buf, _) = serializar(quadros);
        Leitor::abrir(std::io::Cursor::new(buf)).unwrap()
    }

    #[test]
    fn iguais_nao_divergem() {
        let qs = vec![quadro(0, 3), quadro(1, 2)];
        assert_eq!(
            comparar_leitores(&mut leitor(&qs), &mut leitor(&qs)).unwrap(),
            None
        );
    }

    #[test]
    fn acusa_categoria_plantada() {
        for categoria in Categoria::TODAS
            .into_iter()
            .filter(|c| *c != Categoria::Eventos)
        {
            let base = vec![quadro(0, 2), quadro(1, 2), quadro(2, 2)];
            let mut outro = base.clone();
            outro[1].hashes[categoria as usize][3] ^= 0x40;
            let d = comparar_leitores(&mut leitor(&base), &mut leitor(&outro))
                .unwrap()
                .unwrap();
            assert_eq!(
                d,
                Divergencia {
                    quadro: 1,
                    onde: Onde::Categoria {
                        categoria,
                        evento: None
                    }
                }
            );
        }
    }

    #[test]
    fn acusa_evento_plantado() {
        let base = vec![quadro(0, 4), quadro(1, 4)];
        let mut outro = base.clone();
        outro[1].eventos[2].b ^= 1;
        let d = comparar_leitores(&mut leitor(&base), &mut leitor(&outro))
            .unwrap()
            .unwrap();
        assert_eq!(
            d,
            Divergencia {
                quadro: 1,
                onde: Onde::Categoria {
                    categoria: Categoria::Eventos,
                    evento: Some(2)
                }
            }
        );
    }

    #[test]
    fn acusa_relogio_e_tamanho() {
        let base = vec![quadro(0, 1), quadro(1, 1)];
        let mut outro = base.clone();
        outro[0].mclk_fim += 1;
        let d = comparar_leitores(&mut leitor(&base), &mut leitor(&outro))
            .unwrap()
            .unwrap();
        assert_eq!(d.onde, Onde::RelogioFinal);

        let curto = vec![quadro(0, 1)];
        let d = comparar_leitores(&mut leitor(&base), &mut leitor(&curto))
            .unwrap()
            .unwrap();
        assert_eq!(
            d,
            Divergencia {
                quadro: 1,
                onde: Onde::Tamanho { esquerda: false }
            }
        );
    }

    #[test]
    fn cabecalho_diferente() {
        let a = vec![quadro(0, 1)];
        let mut la = leitor(&a);
        let mut lb = leitor(&a);
        lb.cabecalho.modo = 1;
        let d = comparar_leitores(&mut la, &mut lb).unwrap().unwrap();
        assert_eq!(d.onde, Onde::Cabecalho);
    }
}
