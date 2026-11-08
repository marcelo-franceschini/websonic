//! `cargo fidelidade bugs-conhecidos`: relatório da lista de verificação dos trechos `FixBugs`
//! (sub-etapa 5.8).
//!
//! A lista mora em `docs/bugs-conhecidos.md` (gerada por `cargo xtask bugs-conhecidos`, só com
//! arquivo, linha e rótulo). Cada linha da tabela é um item `BK-nnnn` com etapa-alvo. Um item
//! está coberto quando algum teste da execução tem `BK_nnnn` no nome. Este relatório lê os nomes
//! dos testes (`cargo test --workspace -- --list`), não decide nada pelo jogo.
//!
//! Etapa 5: o relatório existe e roda; a cobertura é de 0 de N porque os trechos moram em rotinas
//! das Etapas 6 a 12. A Etapa 14 usa `--exigir-todos`.
//!
//! Uso: `cargo fidelidade bugs-conhecidos [--exigir-todos] [--saida ARQ]`
use crate::manifesto::ler_testes;
use crate::{opcao, raiz};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::process::Command;

/// Um item da lista.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub id: String,
    pub etapa: String,
    pub estado: String,
}

/// Lê as linhas `| BK-0001 | arquivo | linha | rotulo | etapa | estado |` do documento.
pub fn ler_lista(md: &str) -> Vec<Item> {
    md.lines()
        .filter(|l| l.starts_with("| BK-"))
        .filter_map(|l| {
            let c: Vec<&str> = l.trim_matches('|').split('|').map(str::trim).collect();
            let [id, _arq, _linha, _rotulo, etapa, estado] = c[..] else {
                return None;
            };
            Some(Item {
                id: id.to_string(),
                etapa: etapa.to_string(),
                estado: estado.to_string(),
            })
        })
        .collect()
}

/// Ids cobertos: itens cujo `BK_nnnn` aparece no nome de algum teste.
pub fn cobertos(itens: &[Item], testes: &[String]) -> BTreeSet<String> {
    itens
        .iter()
        .filter(|i| {
            let chave = i.id.replace('-', "_");
            testes.iter().any(|t| t.contains(&chave))
        })
        .map(|i| i.id.clone())
        .collect()
}

pub fn texto(itens: &[Item], cob: &BTreeSet<String>) -> String {
    let mut por_etapa: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for i in itens {
        let e = por_etapa.entry(i.etapa.as_str()).or_insert((0, 0));
        e.0 += 1;
        if cob.contains(&i.id) {
            e.1 += 1;
        }
    }
    let mut t = format!(
        "bugs conhecidos: {} cobertos de {}\n",
        cob.len(),
        itens.len()
    );
    for (etapa, (n, c)) in por_etapa {
        t.push_str(&format!("etapa {etapa}: {c} de {n}\n"));
    }
    t
}

pub fn executar(args: &[String]) -> Result<(), String> {
    let md =
        fs::read_to_string(raiz().join("docs/bugs-conhecidos.md")).map_err(|e| e.to_string())?;
    let itens = ler_lista(&md);
    if itens.is_empty() {
        return Err("docs/bugs-conhecidos.md sem itens (rode cargo xtask bugs-conhecidos)".into());
    }
    let saida = Command::new("sh")
        .args(["-c", "cargo test --workspace -- --list 2>&1"])
        .current_dir(raiz())
        .output()
        .map_err(|e| e.to_string())?;
    let nomes: Vec<String> = ler_testes(&String::from_utf8_lossy(&saida.stdout))
        .into_iter()
        .map(|t| t.nome)
        .collect();
    let cob = cobertos(&itens, &nomes);
    let t = texto(&itens, &cob);
    print!("{t}");
    if let Some(s) = opcao(args, "--saida") {
        fs::write(s, &t).map_err(|e| e.to_string())?;
    }
    if args.iter().any(|a| a == "--exigir-todos") && cob.len() != itens.len() {
        return Err(format!(
            "faltam casos de teste: {} de {} cobertos",
            cob.len(),
            itens.len()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    const MD: &str = "| id | arquivo | linha | rotulo | etapa | estado |\n| --- | --- | --- | --- | --- | --- |\n| BK-0001 | a.asm | 10 | Rot1 | 9 | pendente |\n| BK-0002 | b.asm | 20 | Rot2 | 9 | pendente |\n| BK-0003 | c.asm | 30 | Rot3 | 11 | pendente |\n";

    #[test]
    fn le_a_lista_e_conta_os_cobertos() {
        let itens = ler_lista(MD);
        assert_eq!(itens.len(), 3);
        let testes = vec![
            "sonic1_core::fisica::bk_0002_reproduz_o_bug".to_string(),
            "x::outro".to_string(),
        ];
        // O nome do teste usa BK_0002 (maiusculo ou minusculo conta? so a forma exata).
        assert!(cobertos(&itens, &testes).is_empty());
        let testes = vec!["sonic1_core::fisica::teste_BK_0002_reproduz".to_string()];
        let c = cobertos(&itens, &testes);
        assert_eq!(c, ["BK-0002".to_string()].into());
        let t = texto(&itens, &c);
        assert!(t.contains("1 cobertos de 3"));
        assert!(t.contains("etapa 9: 1 de 2"));
        assert!(t.contains("etapa 11: 0 de 1"));
    }
}
