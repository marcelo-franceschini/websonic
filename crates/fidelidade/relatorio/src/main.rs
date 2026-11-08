//! Relatório de fidelidade. Uso: `cargo fidelidade <comando>`.
//!
//! Este pacote mede se o jogo está certo, por isso fica em `crates/fidelidade/` (pasta congelada).
//! Mudar qualquer arquivo daqui exige aprovação e patch do Marcelo.
//!
//! - `fidelidade`: o comando único (manifesto, determinismo, traces e segundo oráculo).
//! - `manifesto`: gera e verifica o manifesto assinado.
//! - `ocultos`: cifra, protege a chave e audita os testes ocultos.
//! - `corpus-trace`: traces do corpus e prova de determinismo.
use std::path::PathBuf;
use std::process::ExitCode;

mod corpus;
mod fidelidade;
mod manifesto;
mod ocultos;
mod sha256;

/// Raiz do repositório (o pacote fica em `<raiz>/crates/fidelidade/relatorio`).
pub fn raiz() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .map(PathBuf::from)
        .unwrap_or_default()
}

/// Valor da opção `--nome VALOR` na linha de comando.
pub fn opcao<'a>(args: &'a [String], nome: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == nome)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let resto = args.get(1..).unwrap_or_default();
    let resultado = match args.first().map(String::as_str) {
        Some("fidelidade") => fidelidade::executar(resto),
        Some("manifesto") => manifesto::executar(resto),
        Some("ocultos") => ocultos::executar(resto),
        Some("corpus-trace") => corpus::executar(resto),
        _ => {
            eprintln!("uso: cargo fidelidade <fidelidade|manifesto|ocultos|corpus-trace>");
            return ExitCode::from(2);
        }
    };
    match resultado {
        Ok(()) => ExitCode::SUCCESS,
        Err(erro) => {
            eprintln!("relatorio: {erro}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn raiz_tem_o_cargo_toml_do_jogo() {
        assert!(
            raiz()
                .join("crates/fidelidade/relatorio/Cargo.toml")
                .is_file()
        );
    }

    #[test]
    fn opcao_acha_o_valor() {
        let a: Vec<String> = ["--x", "1", "--y"].map(String::from).into();
        assert_eq!(opcao(&a, "--x"), Some("1"));
        assert_eq!(opcao(&a, "--y"), None);
        assert_eq!(opcao(&a, "--z"), None);
    }
}
