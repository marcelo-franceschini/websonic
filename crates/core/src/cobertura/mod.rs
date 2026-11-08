//! Registro de decisões do núcleo, para cobertura (5.6) e bugs injetados (5.7).
//!
//! Toda decisão da lógica do jogo que espelha um desvio do original passa por [`decidir`]
//! (macro [`decisao!`]). O `id` é um nome estável, no formato `Rotina:decisao`. Infraestrutura
//! (laço de quadro, pontos de checagem) NÃO usa isto: não tem par no original.
//!
//! Sem as features `cobertura` e `mutacao`, [`decidir`] devolve a condição e some na compilação.
//! Com `cobertura`, conta quantas vezes cada decisão foi verdadeira e falsa e pode guardar a
//! sequência de resultados. Com `mutacao`, a decisão cujo `id` é o alvo devolve o contrário.

/// Passa a condição pelo registro de decisões. Uso: `if decisao!("KosDec:literal", bit) { ... }`.
#[macro_export]
macro_rules! decisao {
    ($id:expr, $cond:expr) => {
        $crate::cobertura::decidir($id, $cond)
    };
}

#[cfg(not(feature = "cobertura"))]
#[inline(always)]
pub const fn decidir(_id: &'static str, condicao: bool) -> bool {
    condicao
}

#[cfg(feature = "cobertura")]
pub use ativo::*;

#[cfg(feature = "cobertura")]
mod ativo {
    use std::cell::{Cell, RefCell};
    use std::collections::BTreeMap;

    thread_local! {
        static CONTAGENS: RefCell<BTreeMap<&'static str, (u64, u64)>> = const { RefCell::new(BTreeMap::new()) };
        static LOG: RefCell<Vec<(&'static str, bool)>> = const { RefCell::new(Vec::new()) };
        static LOGAR: Cell<bool> = const { Cell::new(false) };
        static ALVO: RefCell<Option<String>> = const { RefCell::new(None) };
        static ALVO_INICIADO: Cell<bool> = const { Cell::new(false) };
    }

    /// Registra a decisão e devolve o resultado (invertido se for o mutante, com `mutacao`).
    pub fn decidir(id: &'static str, condicao: bool) -> bool {
        let resultado = invertida(id, condicao);
        CONTAGENS.with(|c| {
            let mut c = c.borrow_mut();
            let e = c.entry(id).or_insert((0, 0));
            if resultado {
                e.0 = e.0.saturating_add(1);
            } else {
                e.1 = e.1.saturating_add(1);
            }
        });
        if LOGAR.with(Cell::get) {
            LOG.with(|l| l.borrow_mut().push((id, resultado)));
        }
        resultado
    }

    #[cfg(feature = "mutacao")]
    fn invertida(id: &'static str, condicao: bool) -> bool {
        // O mutante pode vir do ambiente (`WEBSONIC_MUTANTE=Rotina:decisao`), lido uma vez por thread.
        if !ALVO_INICIADO.with(Cell::get) {
            ALVO_INICIADO.with(|i| i.set(true));
            if let Ok(v) = std::env::var("WEBSONIC_MUTANTE") {
                ALVO.with(|a| *a.borrow_mut() = Some(v));
            }
        }
        let e_alvo = ALVO.with(|a| a.borrow().as_deref() == Some(id));
        if e_alvo { !condicao } else { condicao }
    }

    #[cfg(not(feature = "mutacao"))]
    fn invertida(_id: &'static str, condicao: bool) -> bool {
        condicao
    }

    /// Escolhe o mutante: a decisão com este `id` passa a devolver o contrário (precisa da
    /// feature `mutacao`; sem ela, a chamada não tem efeito).
    pub fn definir_mutante(alvo: Option<&str>) {
        ALVO_INICIADO.with(|i| i.set(true));
        ALVO.with(|a| *a.borrow_mut() = alvo.map(str::to_owned));
    }

    /// Liga ou desliga o registro da sequência de resultados.
    pub fn registrar_sequencia(ligado: bool) {
        LOGAR.with(|l| l.set(ligado));
    }

    /// Zera contagens e sequência.
    pub fn zerar() {
        CONTAGENS.with(|c| c.borrow_mut().clear());
        LOG.with(|l| l.borrow_mut().clear());
    }

    /// Por decisão: (vezes verdadeira, vezes falsa).
    pub fn contagens() -> BTreeMap<&'static str, (u64, u64)> {
        CONTAGENS.with(|c| c.borrow().clone())
    }

    /// Sequência de resultados na ordem em que as decisões foram tomadas.
    pub fn sequencia() -> Vec<(&'static str, bool)> {
        LOG.with(|l| l.borrow().clone())
    }
}

#[cfg(all(test, feature = "mutacao"))]
mod testes {
    use super::*;

    #[test]
    fn mutante_inverte_so_a_decisao_alvo_e_conta() {
        zerar();
        definir_mutante(None);
        assert!(decidir("A:x", true));
        assert!(!decidir("B:y", false));
        definir_mutante(Some("A:x"));
        assert!(!decidir("A:x", true));
        assert!(!decidir("B:y", false));
        let c = contagens();
        // A contagem registra o resultado depois da inversao.
        assert_eq!(c["A:x"], (1, 1));
        assert_eq!(c["B:y"], (0, 2));
        definir_mutante(None);
    }

    #[test]
    fn sequencia_guarda_a_ordem() {
        zerar();
        registrar_sequencia(true);
        decidir("A:x", true);
        decidir("B:y", false);
        decidir("A:x", false);
        assert_eq!(
            sequencia(),
            vec![("A:x", true), ("B:y", false), ("A:x", false)]
        );
        registrar_sequencia(false);
    }
}
