//! `cargo xtask corpus-meta`: escreve o `.meta` de um caso do corpus (sub-etapa 2.3).
//!
//! Os traces do corpus (`corpus-trace`) saíram daqui na 2.10: medem a fidelidade e ficam no
//! pacote congelado `crates/fidelidade/relatorio` (`cargo fidelidade corpus-trace`).
use sonic1_corpus::{ler_s1in, meta, sha1_das_entradas};
use sonic1_fidelidade::hash::em_hex;
use std::fs;
use std::path::Path;

fn opcao<'a>(args: &'a [String], nome: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == nome)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

/// `cargo xtask corpus-meta --entradas corpus/X.s1in --fonte TEXTO --licenca TEXTO --autor TEXTO`:
/// valida as entradas e escreve o `.meta` ao lado, com os hashes calculados.
pub fn escrever_meta(args: &[String]) -> Result<(), String> {
    let arq = opcao(args, "--entradas").ok_or("falta --entradas")?;
    let q = ler_s1in(&fs::read_to_string(arq).map_err(|e| e.to_string())?)
        .map_err(|e| format!("{arq}: {e:?}"))?;
    let rom =
        fs::read_to_string(super::raiz().join("refs/rom-hashes.txt")).map_err(|e| e.to_string())?;
    let rom_sha1 = rom
        .lines()
        .find_map(|l| l.strip_prefix("sha1:"))
        .map(str::trim)
        .ok_or("refs/rom-hashes.txt sem sha1")?;
    let texto = format!(
        "fonte: {}\nlicenca: {}\nrom_sha1: {}\nquadros: {}\nentradas_sha1: {}\nautor: {}\n",
        opcao(args, "--fonte").ok_or("falta --fonte")?,
        opcao(args, "--licenca").ok_or("falta --licenca")?,
        rom_sha1,
        q.len(),
        em_hex(&sha1_das_entradas(&q)),
        opcao(args, "--autor").ok_or("falta --autor")?,
    );
    meta::ler(&texto).map_err(|e| format!("meta invalido: {e:?}"))?;
    fs::write(Path::new(arq).with_extension("meta"), texto).map_err(|e| e.to_string())?;
    println!("meta: {} quadros", q.len());
    Ok(())
}
