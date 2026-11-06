//! Comandos do projeto. Uso: `cargo xtask <comando>`.
//!
//! - `site`: monta o site em `dist/`.
//! - `anti-rom`: procura ROM no repositório (extensão, tamanho e hash).
//! - `rotulos`: gera `refs/rotulos.txt` (só o Marcelo roda; ver `rotulos.rs`).
//! - `inventario`: inventário de hardware a partir dos traces (ver `inventario.rs`).
//! - `ocultos`: cifra e audita os testes ocultos (ver `ocultos.rs`).
//! - `manifesto`: gera e verifica o manifesto assinado (ver `manifesto.rs`).
//! - `corpus-trace`: traces do corpus e prova de determinismo (ver `corpus.rs`).
//! - `descompressao-relatorio`: compara os descompressores com o oráculo (ver `descompressao.rs`).
use std::path::PathBuf;
use std::process::{Command, ExitCode};

mod anti_rom;
mod corpus;
mod descompressao;
mod inventario;
mod manifesto;
mod ocultos;
mod rotulos;
mod sha256;
mod site;

/// Raiz do repositório (o xtask fica em `<raiz>/xtask`).
fn raiz() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(PathBuf::from)
        .unwrap_or_default()
}

/// Roda um comando e falha com mensagem clara se ele falhar.
fn rodar(cmd: &mut Command) -> Result<(), String> {
    let status = cmd
        .status()
        .map_err(|e| format!("não consegui rodar {cmd:?}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{cmd:?} terminou com {status}"))
    }
}

fn main() -> ExitCode {
    let comando = std::env::args().nth(1);
    let resultado = match comando.as_deref() {
        Some("site") => site::montar(),
        Some("anti-rom") => anti_rom::executar(),
        Some("rotulos") => rotulos::executar(&std::env::args().skip(2).collect::<Vec<_>>()),
        Some("descompressao-relatorio") => {
            descompressao::executar(&std::env::args().skip(2).collect::<Vec<_>>())
        }
        Some("corpus-trace") => corpus::executar(&std::env::args().skip(2).collect::<Vec<_>>()),
        Some("inventario") => inventario::executar(&std::env::args().skip(2).collect::<Vec<_>>()),
        Some("ocultos") => ocultos::executar(&std::env::args().skip(2).collect::<Vec<_>>()),
        Some("manifesto") => manifesto::executar(&std::env::args().skip(2).collect::<Vec<_>>()),
        _ => {
            eprintln!(
                "uso: cargo xtask <site|anti-rom|rotulos|descompressao-relatorio|corpus-trace|manifesto|ocultos|inventario>"
            );
            return ExitCode::from(2);
        }
    };
    match resultado {
        Ok(()) => ExitCode::SUCCESS,
        Err(erro) => {
            eprintln!("xtask: {erro}");
            ExitCode::FAILURE
        }
    }
}
