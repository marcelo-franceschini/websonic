//! `cargo xtask fidelidade`: o comando único de fidelidade (sub-etapa 2.9, esqueleto).
//!
//! Roda só no computador do Marcelo (usa a ROM). Faz três coisas e dá um veredito:
//! 1. `manifesto verificar`: assinatura, arquivos de referência e testes;
//! 2. `corpus-trace`: refaz os traces do corpus e compara o hash de cada caso com o do
//!    manifesto assinado, além da prova de determinismo;
//! 3. segundo oráculo: confere em `docs/conhecimento/oraculos.md` que cada diferença registrada
//!    tem decisão. Este passo NÃO roda o segundo oráculo: quem o roda é `scripts/etapa2-oraculo2.sh`.
//!    O que ele afirma é só "cada diferença conhecida tem decisão registrada".
//!
//! A saída tem só nomes, hashes, contagens e vereditos.
//!
//! Uso: `cargo xtask fidelidade --rom ROM --oraculo BIN --saida DIR [--pular-testes]`
use std::collections::BTreeMap;
use std::fs;
use std::process::Command;

/// Linhas `trace<TAB>nome<TAB>hash` do manifesto.
pub fn traces_do_manifesto(manifesto: &str) -> BTreeMap<String, String> {
    manifesto
        .lines()
        .filter_map(|l| {
            let c: Vec<&str> = l.split('\t').collect();
            match c[..] {
                ["trace", nome, hash] => Some((nome.to_string(), hash.to_string())),
                _ => None,
            }
        })
        .collect()
}

/// Linhas `nome<TAB>hash<TAB>quadros` da saída do `corpus-trace` (as outras são ignoradas).
pub fn traces_da_saida(saida: &str) -> BTreeMap<String, String> {
    saida
        .lines()
        .filter_map(|l| {
            let c: Vec<&str> = l.split('\t').collect();
            match c[..] {
                [nome, hash, q]
                    if hash.len() == 40
                        && hash.bytes().all(|b| b.is_ascii_hexdigit())
                        && q.bytes().all(|b| b.is_ascii_digit()) =>
                {
                    Some((nome.to_string(), hash.to_string()))
                }
                _ => None,
            }
        })
        .collect()
}

/// Problemas entre o manifesto assinado e a execução. Um por linha.
pub fn conferir_traces(
    manifesto: &BTreeMap<String, String>,
    obtidos: &BTreeMap<String, String>,
) -> Vec<String> {
    let mut p = Vec::new();
    for (nome, hash) in manifesto {
        match obtidos.get(nome) {
            None => p.push(format!("{nome}: caso do manifesto não rodou")),
            Some(h) if h != hash => p.push(format!("{nome}: hash do trace difere do manifesto")),
            Some(_) => {}
        }
    }
    for nome in obtidos.keys() {
        if !manifesto.contains_key(nome) {
            p.push(format!("{nome}: caso novo, não assinado"));
        }
    }
    p
}

/// Diferenças do segundo oráculo registradas em `oraculos.md`: `(total, sem_decisao)`.
/// Uma linha de tabela cujo primeiro campo é `D<número>` é uma diferença; a decisão é o último
/// campo e não pode ficar vazio.
pub fn diferencas_registradas(md: &str) -> (usize, usize) {
    let (mut total, mut sem) = (0, 0);
    for l in md.lines().filter(|l| l.starts_with("| D")) {
        let c: Vec<&str> = l.trim_matches('|').split('|').map(str::trim).collect();
        let eh_d = c
            .first()
            .is_some_and(|id| id.len() > 1 && id[1..].bytes().all(|b| b.is_ascii_digit()));
        if !eh_d {
            continue;
        }
        total += 1;
        if c.last().is_none_or(|d| d.is_empty()) {
            sem += 1;
        }
    }
    (total, sem)
}

fn opcao<'a>(args: &'a [String], nome: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == nome)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

pub fn executar(args: &[String]) -> Result<(), String> {
    let raiz = super::raiz();
    let rom = opcao(args, "--rom").ok_or("falta --rom")?;
    let oraculo = opcao(args, "--oraculo").ok_or("falta --oraculo")?;
    let saida = opcao(args, "--saida").ok_or("falta --saida")?;
    let mut falhas = 0usize;

    // 1. Manifesto.
    let mut args_man = vec!["verificar".to_string()];
    if args.iter().any(|a| a == "--pular-testes") {
        args_man.push("--pular-testes".into());
    }
    match super::manifesto::executar(&args_man) {
        Ok(()) => println!("manifesto: PASSOU"),
        Err(e) => {
            println!("manifesto: FALHOU ({e})");
            falhas += 1;
        }
    }

    // 2. Traces do corpus contra o manifesto assinado.
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let s = Command::new(exe)
        .args([
            "corpus-trace",
            "--rom",
            rom,
            "--oraculo",
            oraculo,
            "--rotulos",
            "refs/rotulos.txt",
            "--saida",
            saida,
        ])
        .current_dir(&raiz)
        .output()
        .map_err(|e| e.to_string())?;
    let texto = String::from_utf8_lossy(&s.stdout).into_owned();
    let determinismo = s.status.success() && texto.contains("determinismo: PASSOU");
    println!(
        "determinismo: {}",
        if determinismo { "PASSOU" } else { "FALHOU" }
    );
    if !determinismo {
        falhas += 1;
    }
    let manifesto =
        fs::read_to_string(raiz.join("refs/manifesto.txt")).map_err(|e| e.to_string())?;
    let problemas = conferir_traces(&traces_do_manifesto(&manifesto), &traces_da_saida(&texto));
    for p in &problemas {
        println!("trace: {p}");
    }
    println!(
        "traces: {}",
        if problemas.is_empty() {
            "PASSOU"
        } else {
            "FALHOU"
        }
    );
    if !problemas.is_empty() {
        falhas += 1;
    }

    // 3. Diferenças do segundo oráculo.
    let md = fs::read_to_string(raiz.join("docs/conhecimento/oraculos.md"))
        .map_err(|e| e.to_string())?;
    let (total, sem) = diferencas_registradas(&md);
    println!("segundo oráculo: {total} diferença(s) registrada(s), {sem} sem decisão");
    if sem > 0 {
        falhas += 1;
    }

    if falhas == 0 {
        println!("veredito: PASSOU");
        Ok(())
    } else {
        println!("veredito: FALHOU ({falhas} verificação(ões))");
        Err("fidelidade falhou".into())
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    const H1: &str = "0123456789abcdef0123456789abcdef01234567";
    const H2: &str = "ffffffffffffffffffffffffffffffffffffffff";

    #[test]
    fn manifesto_e_saida() {
        let m = format!("arquivo\tx\tabc\ntrace\tboot\t{H1}\nteste\ta::b\t-\n");
        let mm = traces_do_manifesto(&m);
        assert_eq!(mm.len(), 1);
        let saida = format!("boot\t{H1}\t7200\ndeterminismo: PASSOU\nlixo\tcurto\t3\n");
        let so = traces_da_saida(&saida);
        assert_eq!(so.len(), 1);
        assert!(conferir_traces(&mm, &so).is_empty());
    }

    #[test]
    fn trace_diferente_ausente_e_novo() {
        let mm: BTreeMap<_, _> = [
            ("a".to_string(), H1.to_string()),
            ("b".to_string(), H1.to_string()),
        ]
        .into();
        let so: BTreeMap<_, _> = [
            ("a".to_string(), H2.to_string()),
            ("c".to_string(), H1.to_string()),
        ]
        .into();
        let p = conferir_traces(&mm, &so);
        assert_eq!(p.len(), 3);
        assert!(p[0].contains("difere"));
        assert!(p[1].contains("não rodou"));
        assert!(p[2].contains("não assinado"));
    }

    #[test]
    fn diferencas_com_e_sem_decisao() {
        let md = "| id | caso | decisao |\n| --- | --- | --- |\n| D1 | x | vale o principal |\n| D2 | y |  |\n| texto | z | w |\n";
        assert_eq!(diferencas_registradas(md), (2, 1));
    }
}
