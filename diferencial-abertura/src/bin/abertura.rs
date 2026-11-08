//! `abertura comparar ROM QUADROS [rotulos]`: oráculo contra núcleo na abertura (Etapa 7).
use sonic1_diferencial_abertura::{Nomes, comparar, imprimir};
use std::process::ExitCode;

fn principal() -> Result<bool, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) != Some("comparar") || args.len() < 3 {
        return Err("uso: abertura comparar ROM QUADROS [REFS/ROTULOS.TXT]".into());
    }
    let rom = std::fs::read(&args[1]).map_err(|e| format!("nao li a ROM: {e}"))?;
    let quadros: u32 = args[2].parse().map_err(|_| "QUADROS invalido")?;
    let texto = match args.get(3) {
        Some(a) => std::fs::read_to_string(a).map_err(|e| e.to_string())?,
        None => String::new(),
    };
    let nomes = Nomes::carregar(&texto);
    // Os contadores do tempo do oráculo são por thread: uma thread nova, com pilha grande.
    let rel = std::thread::Builder::new()
        .stack_size(64 << 20)
        .spawn(move || comparar(rom, quadros, &nomes))
        .map_err(|e| e.to_string())?
        .join()
        .map_err(|_| "pânico na comparacao".to_string())??;
    imprimir(&rel);
    Ok(rel.zero())
}

fn main() -> ExitCode {
    match principal() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(e) => {
            eprintln!("erro: {e}");
            ExitCode::from(2)
        }
    }
}
