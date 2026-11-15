use sonic1_recomp::{descobrir, descobrir_fragmento};
use sonic1_rom::{Veredito, identificar};
use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

fn endereco(s: &str) -> Result<u32, String> {
    u32::from_str_radix(s.strip_prefix("0x").unwrap_or(s), 16)
        .map_err(|_| format!("endereço hexadecimal inválido: {s}"))
}

fn executar() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !(args.len() == 4 || (args.len() == 5 && args[4] == "--fragmento")) {
        return Err(
            "uso: sonic1-recomp ROM INICIO_HEX FIM_EXCLUSIVO_HEX SAIDA.rs [--fragmento]".into(),
        );
    }
    let rom = std::fs::read(&args[0]).map_err(|e| format!("leitura da ROM: {e}"))?;
    if identificar(&rom).veredito != Veredito::Rev01 {
        return Err("ROM recusada: é necessária a REV01 de referência".into());
    }
    let inicio = endereco(&args[1])?;
    let fim = endereco(&args[2])?;
    let programa = if args.len() == 5 {
        descobrir_fragmento(&rom, inicio, inicio, fim)
    } else {
        descobrir(&rom, inicio, inicio, fim)
    }
    .map_err(|e| format!("descoberta: {e:?}"))?;
    let saida = Path::new(&args[3]);
    let mut arquivo = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(saida)
        .map_err(|e| format!("criação da saída: {e}"))?;
    arquivo
        .write_all(programa.gerar_rust().as_bytes())
        .map_err(|e| format!("escrita da saída: {e}"))?;
    println!("instruções geradas: {}", programa.enderecos().count());
    println!("saídas externas: {}", programa.saidas().count());
    Ok(())
}

fn main() -> ExitCode {
    match executar() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
