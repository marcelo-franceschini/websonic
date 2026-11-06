//! `cargo xtask manifesto <gerar|verificar>` (sub-etapa 2.7).
//!
//! O manifesto (`refs/manifesto.txt`) lista `tipo<TAB>nome<TAB>sha256`: arquivos de referência,
//! hash dos traces do corpus, hash dos testes ocultos cifrados e a lista de testes. Só hashes,
//! nomes e listas. A assinatura é SSHSIG (`ssh-keygen -Y`), criada pelo Marcelo com a chave dele.
//! A IA nunca assina.
//!
//! `verificar` falha em três casos e avisa num quarto:
//! 1. assinatura inválida ou ausente;
//! 2. arquivo referenciado difere do hash assinado;
//! 3. teste do manifesto ausente ou ignorado na execução;
//! 4. teste novo na execução: não falha, sai "não assinado" e não conta para o pronto.
//!
//! Uso:
//! `cargo xtask manifesto gerar [--traces ARQ] [--ocultos DIR] [--saida refs/manifesto.txt]`
//! `cargo xtask manifesto verificar [--manifesto M] [--sig S] [--assinante PUB] [--pular-testes]`
use crate::sha256::{hex, sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const NAMESPACE: &str = "websonic-manifesto";

/// Pastas e arquivos de referência (relativos à raiz). Pasta entra recursivamente.
const REFERENCIAS: &[&str] = &[
    "refs",
    "oracle/patches",
    "oracle/runner/src",
    "oracle/scripts",
    "crates/fidelidade/src",
    "corpus",
    "descompressao",
];
/// Fora do manifesto: ele mesmo e a assinatura.
const EXCLUIDOS: &[&str] = &["refs/manifesto.txt", "refs/manifesto.txt.sig"];

fn coletar(raiz: &Path, rel: &str, saida: &mut Vec<String>) -> Result<(), String> {
    let caminho = raiz.join(rel);
    if caminho.is_file() {
        if !EXCLUIDOS.contains(&rel) {
            saida.push(rel.to_string());
        }
        return Ok(());
    }
    if !caminho.is_dir() {
        return Ok(());
    }
    let mut nomes: Vec<String> = fs::read_dir(&caminho)
        .map_err(|e| format!("{rel}: {e}"))?
        .filter_map(Result::ok)
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    nomes.sort();
    for n in nomes {
        coletar(raiz, &format!("{rel}/{n}"), saida)?;
    }
    Ok(())
}

/// Linhas dos arquivos de referência: `arquivo<TAB>caminho<TAB>sha256`.
pub fn linhas_de_arquivos(raiz: &Path) -> Result<Vec<String>, String> {
    let mut caminhos = Vec::new();
    for r in REFERENCIAS {
        coletar(raiz, r, &mut caminhos)?;
    }
    caminhos
        .iter()
        .map(|c| {
            let dados = fs::read(raiz.join(c)).map_err(|e| format!("{c}: {e}"))?;
            Ok(format!("arquivo\t{c}\t{}", hex(&sha256(&dados))))
        })
        .collect()
}

/// Confere os arquivos do manifesto contra o disco. Devolve um problema por linha.
pub fn conferir_arquivos(manifesto: &str, raiz: &Path) -> Vec<String> {
    let mut problemas = Vec::new();
    for linha in manifesto.lines().filter(|l| l.starts_with("arquivo\t")) {
        let campos: Vec<&str> = linha.split('\t').collect();
        let [_, nome, hash] = campos[..] else {
            problemas.push(format!("linha malformada: {linha}"));
            continue;
        };
        match fs::read(raiz.join(nome)) {
            Ok(d) if hex(&sha256(&d)) == hash => {}
            Ok(_) => problemas.push(format!("{nome}: hash difere do manifesto assinado")),
            Err(_) => problemas.push(format!("{nome}: ausente")),
        }
    }
    problemas
}

/// Resultado de um teste na saída do `cargo test`.
#[derive(Debug, PartialEq, Eq)]
pub struct Teste {
    pub nome: String,
    /// `None` na listagem (`--list`); `Some("ok")`, `Some("ignored")` ou `Some("FAILED")` na execução.
    pub resultado: Option<String>,
}

/// Lê a saída de `cargo test` (execução ou `--list`). O nome leva o binário: `binario::teste`.
pub fn ler_testes(saida: &str) -> Vec<Teste> {
    let mut binario = String::from("?");
    let mut testes = Vec::new();
    for l in saida.lines() {
        let l = l.trim();
        if let Some(r) = l
            .strip_prefix("Running ")
            .or_else(|| l.strip_prefix("Doc-tests "))
        {
            // "unittests src/lib.rs (target/debug/deps/sonic1_core-abc123)" ou "sonic1_core".
            binario = match r.rsplit_once('(') {
                Some((_, p)) => {
                    let base = p.trim_end_matches(')').rsplit('/').next().unwrap_or("?");
                    base.rsplit_once('-').map_or(base, |(n, _)| n).to_string()
                }
                None => format!("doc-{r}"),
            };
        } else if let Some(r) = l.strip_prefix("test ") {
            if let Some((nome, res)) = r.rsplit_once(" ... ") {
                let res = res
                    .split(|c: char| c.is_whitespace() || c == ',')
                    .next()
                    .unwrap_or("");
                testes.push(Teste {
                    nome: format!("{binario}::{nome}"),
                    resultado: Some(res.to_string()),
                });
            }
        } else if let Some(nome) = l.strip_suffix(": test") {
            testes.push(Teste {
                nome: format!("{binario}::{nome}"),
                resultado: None,
            });
        }
    }
    testes
}

/// Compara os testes do manifesto com os executados. `(faltando_ou_ignorados, nao_assinados)`.
pub fn conferir_testes(manifesto: &str, executados: &[Teste]) -> (Vec<String>, Vec<String>) {
    let assinados: BTreeSet<&str> = manifesto
        .lines()
        .filter_map(|l| l.strip_prefix("teste\t"))
        .filter_map(|r| r.split('\t').next())
        .collect();
    let mut falhas = Vec::new();
    for nome in &assinados {
        match executados.iter().find(|t| t.nome == *nome) {
            Some(t) if t.resultado.as_deref() == Some("ok") => {}
            Some(t) => falhas.push(format!("{nome}: {}", t.resultado.as_deref().unwrap_or("?"))),
            None => falhas.push(format!("{nome}: não apareceu na execução")),
        }
    }
    let novos = executados
        .iter()
        .filter(|t| !assinados.contains(t.nome.as_str()))
        .map(|t| t.nome.clone())
        .collect();
    (falhas, novos)
}

/// Verifica a assinatura SSHSIG com `ssh-keygen -Y verify`.
pub fn verificar_assinatura(manifesto: &Path, sig: &Path, publica: &Path) -> Result<(), String> {
    let chave = fs::read_to_string(publica).map_err(|e| format!("chave pública: {e}"))?;
    let chave = chave.trim();
    if chave.is_empty() || chave.contains('\n') {
        return Err("chave pública deve ter uma linha".into());
    }
    let permitidos =
        std::env::temp_dir().join(format!("websonic-permitidos-{}", std::process::id()));
    fs::write(&permitidos, format!("websonic {chave}\n")).map_err(|e| e.to_string())?;
    let entrada = fs::File::open(manifesto).map_err(|e| format!("manifesto: {e}"))?;
    let s = Command::new("ssh-keygen")
        .args(["-Y", "verify", "-f"])
        .arg(&permitidos)
        .args(["-I", "websonic", "-n", NAMESPACE, "-s"])
        .arg(sig)
        .stdin(Stdio::from(entrada))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = fs::remove_file(&permitidos);
    match s {
        Ok(s) if s.success() => Ok(()),
        Ok(_) => Err("assinatura inválida ou ausente".into()),
        Err(e) => Err(format!("ssh-keygen: {e}")),
    }
}

fn opcao<'a>(args: &'a [String], nome: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == nome)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

fn cargo_test(extra: &[&str]) -> Result<String, String> {
    // stderr e stdout vão para o mesmo cano (`2>&1`), então a ordem é a cronológica:
    // o cargo escreve "Running" antes de cada binário de teste.
    let comando = format!("cargo test --workspace {} 2>&1", extra.join(" "));
    let s = Command::new("sh")
        .args(["-c", &comando])
        .output()
        .map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&s.stdout).into_owned())
}

pub fn executar(args: &[String]) -> Result<(), String> {
    let raiz = super::raiz();
    match args.first().map(String::as_str) {
        Some("gerar") => gerar(&raiz, &args[1..]),
        Some("verificar") => verificar(&raiz, &args[1..]),
        _ => Err("uso: cargo xtask manifesto <gerar|verificar>".into()),
    }
}

fn gerar(raiz: &Path, args: &[String]) -> Result<(), String> {
    let mut linhas = linhas_de_arquivos(raiz)?;
    if let Some(t) = opcao(args, "--traces") {
        // Saída do corpus-trace: `nome<TAB>hash<TAB>quadros`.
        for l in fs::read_to_string(t).map_err(|e| e.to_string())?.lines() {
            let c: Vec<&str> = l.split('\t').collect();
            if let [nome, hash, _] = c[..]
                && hash.len() == 40
                && hash.bytes().all(|b| b.is_ascii_hexdigit())
            {
                linhas.push(format!("trace\t{nome}\t{hash}"));
            }
        }
    }
    if let Some(d) = opcao(args, "--ocultos") {
        // Só hashes dos arquivos cifrados e a lista de zonas.
        let mut zonas: Vec<String> = fs::read_dir(d)
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();
        zonas.sort();
        for z in zonas {
            linhas.push(format!("zona\t{z}\t-"));
            let mut arqs: Vec<PathBuf> = fs::read_dir(Path::new(d).join(&z))
                .map_err(|e| e.to_string())?
                .filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "age"))
                .collect();
            arqs.sort();
            for a in arqs {
                let nome = a.file_name().and_then(|n| n.to_str()).unwrap_or_default();
                let dados = fs::read(&a).map_err(|e| e.to_string())?;
                linhas.push(format!("oculto\t{z}/{nome}\t{}", hex(&sha256(&dados))));
            }
        }
    }
    let lista = ler_testes(&cargo_test(&["--", "--list"])?);
    let mut nomes: Vec<String> = lista.into_iter().map(|t| t.nome).collect();
    nomes.sort();
    nomes.dedup();
    for n in nomes {
        linhas.push(format!("teste\t{n}\t-"));
    }
    let saida = opcao(args, "--saida").unwrap_or("refs/manifesto.txt");
    fs::write(raiz.join(saida), linhas.join("\n") + "\n").map_err(|e| e.to_string())?;
    println!("manifesto: {} linhas em {saida}", linhas.len());
    println!("assine com: ssh-keygen -Y sign -f <chave> -n {NAMESPACE} {saida}");
    Ok(())
}

fn verificar(raiz: &Path, args: &[String]) -> Result<(), String> {
    let manifesto = raiz.join(opcao(args, "--manifesto").unwrap_or("refs/manifesto.txt"));
    let sig = raiz.join(opcao(args, "--sig").unwrap_or("refs/manifesto.txt.sig"));
    let pub_ = raiz.join(opcao(args, "--assinante").unwrap_or("refs/assinante.pub"));
    verificar_assinatura(&manifesto, &sig, &pub_)?;
    println!("assinatura: OK");
    let texto = fs::read_to_string(&manifesto).map_err(|e| e.to_string())?;
    let problemas = conferir_arquivos(&texto, raiz);
    if !problemas.is_empty() {
        for p in &problemas {
            println!("referência: {p}");
        }
        return Err("referência mudou".into());
    }
    println!("referências: OK");
    if !args.iter().any(|a| a == "--pular-testes") {
        let exec = ler_testes(&cargo_test(&[])?);
        let (falhas, novos) = conferir_testes(&texto, &exec);
        for n in &novos {
            println!("não assinado: {n}");
        }
        if !falhas.is_empty() {
            for f in &falhas {
                println!("teste: {f}");
            }
            return Err("testes do manifesto faltando, ignorados ou falhando".into());
        }
        println!("testes: OK ({} não assinados)", novos.len());
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    fn temp(nome: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("websonic-{nome}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn arquivo_mudado_ou_ausente() {
        let d = temp("arq");
        fs::create_dir_all(d.join("refs")).unwrap();
        fs::write(d.join("refs/a.txt"), "um").unwrap();
        let linhas = linhas_de_arquivos(&d).unwrap().join("\n");
        assert!(conferir_arquivos(&linhas, &d).is_empty());
        fs::write(d.join("refs/a.txt"), "dois").unwrap();
        assert_eq!(conferir_arquivos(&linhas, &d).len(), 1);
        fs::remove_file(d.join("refs/a.txt")).unwrap();
        assert!(conferir_arquivos(&linhas, &d)[0].contains("ausente"));
    }

    #[test]
    fn manifesto_nao_entra_nele_mesmo() {
        let d = temp("proprio");
        fs::create_dir_all(d.join("refs")).unwrap();
        fs::write(d.join("refs/manifesto.txt"), "x").unwrap();
        fs::write(d.join("refs/manifesto.txt.sig"), "x").unwrap();
        assert!(linhas_de_arquivos(&d).unwrap().is_empty());
    }

    #[test]
    fn testes_ausente_ignorado_e_novo() {
        let m = "teste\ta::um\t-\nteste\ta::dois\t-\nteste\ta::tres\t-\n";
        let exec = vec![
            Teste {
                nome: "a::um".into(),
                resultado: Some("ok".into()),
            },
            Teste {
                nome: "a::dois".into(),
                resultado: Some("ignored".into()),
            },
            Teste {
                nome: "a::novo".into(),
                resultado: Some("ok".into()),
            },
        ];
        let (falhas, novos) = conferir_testes(m, &exec);
        assert_eq!(falhas.len(), 2);
        assert!(falhas[0].contains("dois") || falhas[1].contains("dois"));
        assert_eq!(novos, vec!["a::novo".to_string()]);
    }

    #[test]
    fn le_saida_do_cargo() {
        let s = "     Running unittests src/lib.rs (target/debug/deps/sonic1_core-0123abcd)\nrunning 2 tests\ntest x::a ... ok\ntest x::b ... ignored, motivo\n\n   Doc-tests sonic1_core\nrunning 1 test\ntest d ... FAILED\n";
        let t = ler_testes(s);
        assert_eq!(
            t[0],
            Teste {
                nome: "sonic1_core::x::a".into(),
                resultado: Some("ok".into())
            }
        );
        assert_eq!(t[1].resultado.as_deref(), Some("ignored"));
        assert_eq!(t[2].nome, "doc-sonic1_core::d");
        assert_eq!(t[2].resultado.as_deref(), Some("FAILED"));
        let l =
            ler_testes("     Running unittests src/lib.rs (target/debug/deps/k-1)\nx::a: test\n");
        assert_eq!(
            l[0],
            Teste {
                nome: "k::x::a".into(),
                resultado: None
            }
        );
    }

    /// Prova da assinatura com chave descartável gerada aqui. Pula se não há ssh-keygen.
    #[test]
    fn assinatura_valida_e_adulterada() {
        if Command::new("ssh-keygen")
            .arg("-?")
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .status()
            .is_err()
        {
            return;
        }
        let d = temp("sig");
        let chave = d.join("k");
        let g = Command::new("ssh-keygen")
            .args(["-q", "-t", "ed25519", "-N", "", "-f"])
            .arg(&chave)
            .status()
            .unwrap();
        assert!(g.success());
        let m = d.join("manifesto.txt");
        fs::write(&m, "arquivo\ta\t00\n").unwrap();
        let s = Command::new("ssh-keygen")
            .args(["-Y", "sign", "-q", "-f"])
            .arg(&chave)
            .args(["-n", NAMESPACE])
            .arg(&m)
            .status()
            .unwrap();
        assert!(s.success());
        let sig = d.join("manifesto.txt.sig");
        let publica = d.join("k.pub");
        assert!(verificar_assinatura(&m, &sig, &publica).is_ok());
        // 1. adulterado
        fs::write(&m, "arquivo\ta\t01\n").unwrap();
        assert!(verificar_assinatura(&m, &sig, &publica).is_err());
        // 1. assinatura ausente
        fs::write(&m, "arquivo\ta\t00\n").unwrap();
        fs::remove_file(&sig).unwrap();
        assert!(verificar_assinatura(&m, &sig, &publica).is_err());
    }
}
