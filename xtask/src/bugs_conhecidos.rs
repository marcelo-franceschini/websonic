//! `cargo xtask bugs-conhecidos`: lista de verificação dos trechos marcados com `FixBugs`
//! (sub-etapa 5.8).
//!
//! Lê os `.asm` de texto do disassembly (permitido pelo requisito; nunca os binários) e extrai,
//! por ocorrência de uma linha de condição que cita `FixBugs`, SÓ: arquivo, número da linha e o
//! rótulo global mais próximo acima. Nada de código, comentário ou tabela. As linhas de definição
//! (`FixBugs = 0`) e as mensagens de aviso não são itens.
//!
//! Uso: `cargo xtask bugs-conhecidos [--fonte DIR] [--saida docs/bugs-conhecidos.md]`
//! (padrão da fonte: `$SONIC1_EXT_DIR/disasm-texto`, ou `/mnt/disasm-texto` no contêiner).
use std::fs;
use std::path::{Path, PathBuf};

/// Uma ocorrência.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ocorrencia {
    pub arquivo: String,
    pub linha: usize,
    pub rotulo: String,
}

/// O texto de uma linha é uma condição que cita `FixBugs`?
pub fn e_condicao(linha: &str) -> bool {
    let t = linha.trim_start();
    let comeca_com_if = t.starts_with("if ") || t.starts_with("if\t") || t.starts_with("if(");
    comeca_com_if && t.contains("FixBugs") && !t.starts_with("warning")
}

/// Nome de rótulo global no começo da linha (coluna zero), sem os dois-pontos. Definições de
/// símbolo (`X = 1`, `X: equ ...`) e macros não são rótulos.
pub fn rotulo_global(linha: &str) -> Option<&str> {
    let primeiro = linha.chars().next()?;
    if !(primeiro.is_ascii_alphabetic() || primeiro == '_') {
        return None;
    }
    let fim = linha.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))?;
    let (nome, resto) = linha.split_at(fim);
    let resto_s = resto.trim_start_matches(':').trim_start();
    let e_definicao = resto_s.starts_with('=')
        || resto_s.starts_with("equ")
        || resto_s.starts_with("macro")
        || resto_s.starts_with("set ")
        || resto_s.starts_with("rsset");
    let e_diretiva = matches!(
        nome,
        "if" | "else" | "endif" | "include" | "even" | "cnop" | "org" | "rept" | "endm"
    );
    (!e_definicao && !e_diretiva).then_some(nome)
}

/// Ocorrências de um arquivo.
pub fn ocorrencias_do_texto(arquivo: &str, texto: &str) -> Vec<Ocorrencia> {
    let mut atual = String::new();
    let mut v = Vec::new();
    for (i, l) in texto.lines().enumerate() {
        if let Some(r) = rotulo_global(l) {
            atual = r.to_string();
        }
        if e_condicao(l) {
            v.push(Ocorrencia {
                arquivo: arquivo.to_string(),
                linha: i.saturating_add(1),
                rotulo: if atual.is_empty() {
                    "sem-rotulo".to_string()
                } else {
                    atual.clone()
                },
            });
        }
    }
    v
}

fn coletar(dir: &Path, saida: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    let mut e: Vec<_> = rd.filter_map(Result::ok).map(|e| e.path()).collect();
    e.sort();
    for p in e {
        if p.is_dir() {
            coletar(&p, saida);
        } else if p.extension().is_some_and(|x| x == "asm") {
            saida.push(p);
        }
    }
}

/// Etapa-alvo sugerida pela pasta ou pelo arquivo (heurística; o Marcelo pode ajustar).
pub fn etapa_sugerida(arquivo: &str) -> &'static str {
    if arquivo.contains("sounddriver") || arquivo.starts_with("_inc/Sound") {
        "6"
    } else if arquivo.starts_with("_incObj/") {
        "9 a 11"
    } else if arquivo.starts_with("sonic.asm") {
        "7 a 12"
    } else {
        "9 a 12"
    }
}

pub fn renderizar(itens: &[Ocorrencia]) -> String {
    let mut md = String::from("# Bugs conhecidos do original (lista de verificação)\n\n");
    md.push_str("Gerado por `cargo xtask bugs-conhecidos` a partir dos `.asm` de texto do disassembly fixado. Só arquivo, linha e rótulo global mais próximo acima. Nada de código nem de comentário.\n\n");
    md.push_str("Cada item é uma linha de condição que cita `FixBugs`. O núcleo reproduz o comportamento SEM a correção (a ROM de referência). Cada item precisa de pelo menos um teste cujo nome contenha o id com sublinhado (por exemplo `BK_0001`). A coluna etapa é uma sugestão pela pasta.\n\n");
    md.push_str(&format!("{} itens.\n\n", itens.len()));
    md.push_str("| id | arquivo | linha | rótulo | etapa | estado |\n| --- | --- | --- | --- | --- | --- |\n");
    for (i, o) in itens.iter().enumerate() {
        md.push_str(&format!(
            "| BK-{:04} | {} | {} | {} | {} | pendente |\n",
            i.saturating_add(1),
            o.arquivo,
            o.linha,
            o.rotulo,
            etapa_sugerida(&o.arquivo)
        ));
    }
    md
}

pub fn executar(args: &[String]) -> Result<(), String> {
    let opc = |n: &str| {
        args.iter()
            .position(|a| a == n)
            .and_then(|i| args.get(i + 1))
            .map(String::as_str)
    };
    let fonte: PathBuf = match opc("--fonte") {
        Some(f) => f.into(),
        None => {
            let ext = std::env::var("SONIC1_EXT_DIR").unwrap_or_else(|_| {
                format!("{}/sonic1-ext", std::env::var("HOME").unwrap_or_default())
            });
            let host = Path::new(&ext).join("disasm-texto");
            if host.is_dir() {
                host
            } else {
                PathBuf::from("/mnt/disasm-texto")
            }
        }
    };
    let mut arqs = Vec::new();
    coletar(&fonte, &mut arqs);
    if arqs.is_empty() {
        return Err(format!("nenhum .asm em {}", fonte.display()));
    }
    let mut itens = Vec::new();
    for a in &arqs {
        let rel = a
            .strip_prefix(&fonte)
            .unwrap_or(a)
            .to_string_lossy()
            .replace('\\', "/");
        let bytes = fs::read(a).map_err(|e| format!("{rel}: {e}"))?;
        itens.extend(ocorrencias_do_texto(&rel, &String::from_utf8_lossy(&bytes)));
    }
    let md = renderizar(&itens);
    super::inventario::filtro(&md)?;
    let saida = opc("--saida").unwrap_or("docs/bugs-conhecidos.md");
    fs::write(super::raiz().join(saida), md).map_err(|e| e.to_string())?;
    println!(
        "bugs conhecidos: {} itens em {} arquivos",
        itens.len(),
        arqs.len()
    );
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    const TEXTO: &str = "Rotina_Um:\n\tmoveq\t#0,d0\n\tif FixBugs\n\t\tnop\n\telse\n\t\trts\n\tendif\nOutra:\n.local:\n\tif (FixBugs=0)&(X=0)\n\tnop\n\tendif\nFixBugs = 0\nFoo: equ 1\n\twarning \"FixBugs ligado\"\n\tif FixBugs=0\nfim\n";

    #[test]
    fn acha_so_as_linhas_de_condicao_com_o_rotulo_certo() {
        let o = ocorrencias_do_texto("a.asm", TEXTO);
        let v: Vec<(usize, &str)> = o.iter().map(|x| (x.linha, x.rotulo.as_str())).collect();
        // Linha 3 em Rotina_Um; linha 10 em Outra (o rotulo local .local nao vale); a ultima em Outra
        // tambem: Foo e definicao (equ) e FixBugs = 0 tambem, entao nao trocam o rotulo.
        assert_eq!(v, vec![(3, "Rotina_Um"), (10, "Outra"), (16, "Outra")]);
    }

    #[test]
    fn rotulo_global_ignora_definicoes_diretivas_e_locais() {
        assert_eq!(rotulo_global("KosDec:"), Some("KosDec"));
        assert_eq!(
            rotulo_global("Obj17_Main\tlea\t(a0),a1"),
            Some("Obj17_Main")
        );
        assert_eq!(rotulo_global("FixBugs = 0"), None);
        assert_eq!(rotulo_global("Nome: equ 5"), None);
        assert_eq!(rotulo_global("macro: macro x"), None);
        assert_eq!(rotulo_global(".local:"), None);
        assert_eq!(rotulo_global("\tif FixBugs"), None);
        assert_eq!(rotulo_global("endif"), None);
    }

    #[test]
    fn sem_rotulo_e_marcado_e_nao_inventado() {
        let o = ocorrencias_do_texto("b.asm", "\tif FixBugs\n");
        assert_eq!(o[0].rotulo, "sem-rotulo");
    }

    #[test]
    fn documento_tem_ids_estaveis_e_nenhum_codigo() {
        let o = ocorrencias_do_texto("_incObj/17 Helix.asm", TEXTO);
        let md = renderizar(&o);
        assert!(
            md.contains("| BK-0001 | _incObj/17 Helix.asm | 3 | Rotina_Um | 9 a 11 | pendente |")
        );
        assert!(!md.contains("moveq"));
        assert_eq!(etapa_sugerida("s1.sounddriver.asm"), "6");
    }
}
