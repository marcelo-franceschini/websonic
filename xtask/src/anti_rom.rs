//! `cargo xtask anti-rom`: procura ROM em tudo o que o repositório guarda.
//!
//! Mesma regra dos hooks (`hooks/lib.sh`): hash do blob, tamanho e extensão.
//! Olha todos os objetos alcançáveis (`rev-list --objects --all`), então pega também
//! um arquivo que existiu só no histórico. Não lê a ROM: compara identificadores.
use crate::raiz;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// Extensões bloqueadas. `state*` vale para qualquer extensão que comece com "state".
/// Um teste confere que esta lista é igual à de `hooks/lib.sh`.
const EXTENSOES: &[&str] = &[
    "gen", "bin", "smd", "rom", "32x", "sms", "iso", "zip", "7z", "rar", "tar", "gz", "bz2", "xz",
    "sav", "srm", "state*", "trace", "gst", "wav", "mp3", "ogg", "flac", "vgm",
];

fn extensao_proibida(caminho: &str) -> bool {
    let base = caminho.rsplit('/').next().unwrap_or(caminho);
    let Some((_, ext)) = base.rsplit_once('.') else {
        return false;
    };
    let ext = ext.to_ascii_lowercase();
    EXTENSOES
        .iter()
        .any(|padrao| match padrao.strip_suffix('*') {
            Some(prefixo) => ext.starts_with(prefixo),
            None => ext == *padrao,
        })
}

/// Lê `chave: valor` de um arquivo de `refs/`.
fn ler_ref(texto: &str, chave: &str) -> Option<String> {
    let prefixo = format!("{chave}:");
    texto
        .lines()
        .find_map(|l| l.strip_prefix(&prefixo))
        .map(|v| v.trim().to_string())
}

/// Analisa a saída de `git cat-file --batch-check` e devolve uma linha por problema.
/// Cada linha de entrada é `<id> <tipo> <tamanho> <caminho>`.
fn analisar(saida: &str, max: u64, blob_rom: &str, excecoes: &[&str]) -> Vec<String> {
    let mut problemas = Vec::new();
    for linha in saida.lines() {
        let mut partes = linha.splitn(4, ' ');
        let (Some(id), Some(tipo), Some(tam)) = (partes.next(), partes.next(), partes.next())
        else {
            continue;
        };
        if tipo != "blob" {
            continue;
        }
        let caminho = partes.next().unwrap_or("");
        if id == blob_rom {
            problemas.push(format!("{caminho}: é a ROM de referência (blob {id})"));
            continue;
        }
        // A exceção vale para extensão e tamanho. Nunca para o hash.
        if excecoes.contains(&caminho) {
            continue;
        }
        if let Ok(tam) = tam.parse::<u64>()
            && tam >= max
        {
            problemas.push(format!("{caminho}: {tam} bytes (limite {max})"));
        }
        if extensao_proibida(caminho) {
            problemas.push(format!("{caminho}: extensão proibida"));
        }
    }
    problemas
}

/// Roda o git na raiz e devolve a saída.
fn git(raiz: &Path, args: &[&str], entrada: Option<&[u8]>) -> Result<String, String> {
    let mut filho = Command::new("git")
        .current_dir(raiz)
        .args(args)
        .stdin(if entrada.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("git {args:?}: {e}"))?;
    // A entrada vai de uma thread, para não travar se a saída encher o buffer.
    let escritor = entrada.map(|dados| {
        let dados = dados.to_vec();
        let mut stdin = filho.stdin.take();
        std::thread::spawn(move || {
            if let Some(s) = stdin.as_mut() {
                let _ = s.write_all(&dados);
            }
        })
    });
    let saida = filho
        .wait_with_output()
        .map_err(|e| format!("git {args:?}: {e}"))?;
    if let Some(t) = escritor {
        let _ = t.join();
    }
    if !saida.status.success() {
        return Err(format!("git {args:?} terminou com {}", saida.status));
    }
    Ok(String::from_utf8_lossy(&saida.stdout).into_owned())
}

pub fn executar() -> Result<(), String> {
    let raiz = raiz();
    let ler = |nome: &str| {
        std::fs::read_to_string(raiz.join("refs").join(nome))
            .map_err(|e| format!("refs/{nome} ausente ou ilegível: {e}"))
    };
    // Falha fechada: sem os refs, o comando falha em vez de liberar.
    let max: u64 = ler_ref(&ler("limite-tamanho.txt")?, "max_bytes")
        .and_then(|v| v.parse().ok())
        .ok_or("max_bytes ausente em refs/limite-tamanho.txt")?;
    let blob_rom = ler_ref(&ler("rom-hashes.txt")?, "blob_id")
        .ok_or("blob_id ausente em refs/rom-hashes.txt")?;
    let texto_excecoes = ler("excecoes-extensoes.txt")?;
    let excecoes: Vec<&str> = texto_excecoes
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .collect();

    let objetos = git(&raiz, &["rev-list", "--objects", "--all"], None)?;
    let saida = git(
        &raiz,
        &[
            "cat-file",
            "--batch-check=%(objectname) %(objecttype) %(objectsize) %(rest)",
        ],
        Some(objetos.as_bytes()),
    )?;

    let problemas = analisar(&saida, max, &blob_rom, &excecoes);
    if problemas.is_empty() {
        println!("anti-rom: nenhum arquivo suspeito.");
        Ok(())
    } else {
        for p in &problemas {
            eprintln!("  {p}");
        }
        Err(format!(
            "{} problema(s) de ROM no repositório",
            problemas.len()
        ))
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    const ROM: &str = "f254c3aed45c21bb14a4c95b3514029dddcd25a4";

    #[test]
    fn extensoes() {
        assert!(extensao_proibida("a.gen"));
        assert!(extensao_proibida("pasta/B.GEN"));
        assert!(extensao_proibida("x.state1"));
        assert!(extensao_proibida("Sonic [!] (W).gen"));
        assert!(!extensao_proibida("README.md"));
        assert!(!extensao_proibida("semextensao"));
        assert!(!extensao_proibida("gen"));
    }

    #[test]
    fn a_lista_e_a_mesma_dos_hooks() {
        let lib = include_str!("../../hooks/lib.sh");
        let linha = lib
            .lines()
            .map(str::trim)
            .find(|l| l.starts_with("gen|bin|"))
            .expect("lista de extensões não achada em hooks/lib.sh");
        let dos_hooks: Vec<&str> = linha.trim_end_matches(')').split('|').collect();
        assert_eq!(dos_hooks, EXTENSOES);
    }

    #[test]
    fn rom_pelo_hash_mesmo_com_nome_inocente() {
        let saida = format!("{ROM} blob 524288 docs/nota.txt\n");
        assert_eq!(analisar(&saida, 262_144, ROM, &[]).len(), 1);
    }

    #[test]
    fn excecao_nao_libera_o_hash() {
        let saida = format!("{ROM} blob 524288 docs/nota.txt\n");
        assert_eq!(analisar(&saida, 262_144, ROM, &["docs/nota.txt"]).len(), 1);
    }

    #[test]
    fn tamanho_usa_maior_ou_igual() {
        let saida = "aaaa blob 262144 a.txt\nbbbb blob 262143 b.txt\n";
        let p = analisar(saida, 262_144, ROM, &[]);
        assert_eq!(p.len(), 1);
        assert!(p[0].starts_with("a.txt"));
    }

    #[test]
    fn excecao_libera_extensao_e_tamanho() {
        let saida = "aaaa blob 300000 grande.bin\n";
        assert!(analisar(saida, 262_144, ROM, &["grande.bin"]).is_empty());
    }

    #[test]
    fn ignora_o_que_nao_e_blob() {
        let saida = "cccc commit 200 \ntttt tree 100 pasta\n";
        assert!(analisar(saida, 10, ROM, &[]).is_empty());
    }

    #[test]
    fn le_ref() {
        assert_eq!(
            ler_ref("a: 1\nmax_bytes: 262144\n", "max_bytes").as_deref(),
            Some("262144")
        );
        assert_eq!(ler_ref("a: 1\n", "max_bytes"), None);
    }
}
