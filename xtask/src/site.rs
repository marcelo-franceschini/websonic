//! `cargo xtask site`: compila o crate web para WebAssembly e monta `dist/`.
use crate::{raiz, rodar};
use std::fs;
use std::process::Command;

pub fn montar() -> Result<(), String> {
    let raiz = raiz();
    let dist = raiz.join("dist");
    let pkg = dist.join("pkg");

    // Começa sempre de uma pasta limpa, para não publicar sobras.
    if dist.exists() {
        fs::remove_dir_all(&dist).map_err(|e| format!("limpar dist/: {e}"))?;
    }
    fs::create_dir_all(&pkg).map_err(|e| format!("criar dist/pkg: {e}"))?;

    // 1. Compila o crate web para WebAssembly.
    rodar(
        Command::new("cargo")
            .current_dir(&raiz)
            .args(["build", "--release", "-p", "sonic1-web"])
            .args(["--target", "wasm32-unknown-unknown"]),
    )?;

    // 2. Gera o JavaScript de cola. O wasm-bindgen-cli tem a versão fixada na imagem.
    let wasm = raiz.join("target/wasm32-unknown-unknown/release/sonic1_web.wasm");
    rodar(
        Command::new("wasm-bindgen")
            .current_dir(&raiz)
            .args(["--target", "web", "--no-typescript", "--out-dir"])
            .arg(&pkg)
            .arg(&wasm),
    )?;

    // 3. Copia a página. O .nojekyll evita que o Pages processe os arquivos.
    for nome in ["index.html", "app.js"] {
        fs::copy(raiz.join("site").join(nome), dist.join(nome))
            .map_err(|e| format!("copiar site/{nome}: {e}"))?;
    }
    fs::write(dist.join(".nojekyll"), "").map_err(|e| format!("criar .nojekyll: {e}"))?;

    println!("site montado em dist/");
    Ok(())
}
