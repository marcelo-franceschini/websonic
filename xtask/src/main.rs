//! Comandos do projeto. Uso: `cargo xtask <comando>`.
//!
//! - `site`: monta o site em `dist/`.
//! - `anti-rom`: procura ROM no repositório (extensão, tamanho e hash).
use std::path::PathBuf;
use std::process::{Command, ExitCode};

mod anti_rom;
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
        _ => {
            eprintln!("uso: cargo xtask <site|anti-rom>");
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
