//! `cargo xtask site`: compila o crate web para WebAssembly e monta `dist/`.
use crate::{raiz, rodar};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Hash curto deterministico (FNV-1a 64-bit dobrado em 32 bits) para invalidar cache do navegador.
fn hash_curto(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
    let dobrado = (h as u32) ^ ((h >> 32) as u32);
    format!("{dobrado:08x}")
}

fn hash_arquivo(caminho: &Path) -> Result<String, String> {
    let bytes = fs::read(caminho).map_err(|e| format!("ler {}: {e}", caminho.display()))?;
    Ok(hash_curto(&bytes))
}

fn reescrever_referencias(caminho: &Path, trocas: &[(&str, String)]) -> Result<(), String> {
    let mut texto =
        fs::read_to_string(caminho).map_err(|e| format!("ler {}: {e}", caminho.display()))?;
    for (de, para) in trocas {
        texto = texto.replace(de, para);
    }
    fs::write(caminho, texto).map_err(|e| format!("gravar {}: {e}", caminho.display()))
}

pub fn montar() -> Result<(), String> {
    let raiz = raiz();
    let dist = raiz.join("dist");
    let pkg = dist.join("pkg");

    // Comeca sempre de uma pasta limpa, para nao publicar sobras.
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

    // 2. Gera o JavaScript de cola. O wasm-bindgen-cli tem a versao fixada na imagem.
    let wasm = raiz.join("target/wasm32-unknown-unknown/release/sonic1_web.wasm");
    rodar(
        Command::new("wasm-bindgen")
            .current_dir(&raiz)
            .args(["--target", "web", "--no-typescript", "--out-dir"])
            .arg(&pkg)
            .arg(&wasm),
    )?;

    // 3. Copia a pagina e arquivos estaticos. O .nojekyll evita que o Pages processe os arquivos.
    for nome in [
        "index.html",
        "style.css",
        "app.js",
        "entrada.js",
        "hud.js",
        "sim-worker.js",
        "audio-worklet.js",
        "gpu.js",
        "relogio.js",
        "favicon.svg",
        "robots.txt",
        "sitemap.xml",
    ] {
        fs::copy(raiz.join("site").join(nome), dist.join(nome))
            .map_err(|e| format!("copiar site/{nome}: {e}"))?;
    }
    fs::write(dist.join(".nojekyll"), "").map_err(|e| format!("criar .nojekyll: {e}"))?;

    // 4. Cache busting automatico por hash de conteudo em ordem topologica de dependencias.
    let h_wasm_bg = hash_arquivo(&pkg.join("sonic1_web_bg.wasm"))?;
    reescrever_referencias(
        &pkg.join("sonic1_web.js"),
        &[(
            "'sonic1_web_bg.wasm'",
            format!("'sonic1_web_bg.wasm?h={h_wasm_bg}'"),
        )],
    )?;
    let h_wasm_js = hash_arquivo(&pkg.join("sonic1_web.js"))?;

    let h_style = hash_arquivo(&dist.join("style.css"))?;
    let h_relogio = hash_arquivo(&dist.join("relogio.js"))?;
    let h_gpu = hash_arquivo(&dist.join("gpu.js"))?;
    let h_entrada = hash_arquivo(&dist.join("entrada.js"))?;
    let h_worklet = hash_arquivo(&dist.join("audio-worklet.js"))?;

    reescrever_referencias(
        &dist.join("hud.js"),
        &[(
            "\"./relogio.js\"",
            format!("\"./relogio.js?h={h_relogio}\""),
        )],
    )?;
    let h_hud = hash_arquivo(&dist.join("hud.js"))?;

    reescrever_referencias(
        &dist.join("sim-worker.js"),
        &[
            (
                "\"./pkg/sonic1_web.js\"",
                format!("\"./pkg/sonic1_web.js?h={h_wasm_js}\""),
            ),
            (
                "\"./relogio.js\"",
                format!("\"./relogio.js?h={h_relogio}\""),
            ),
        ],
    )?;
    let h_worker = hash_arquivo(&dist.join("sim-worker.js"))?;

    reescrever_referencias(
        &dist.join("app.js"),
        &[
            ("\"./gpu.js\"", format!("\"./gpu.js?h={h_gpu}\"")),
            (
                "\"./entrada.js\"",
                format!("\"./entrada.js?h={h_entrada}\""),
            ),
            ("\"./hud.js\"", format!("\"./hud.js?h={h_hud}\"")),
            (
                "\"./sim-worker.js\"",
                format!("\"./sim-worker.js?h={h_worker}\""),
            ),
            (
                "\"audio-worklet.js\"",
                format!("\"audio-worklet.js?h={h_worklet}\""),
            ),
        ],
    )?;
    let h_app = hash_arquivo(&dist.join("app.js"))?;

    reescrever_referencias(
        &dist.join("index.html"),
        &[
            (
                "href=\"style.css\"",
                format!("href=\"style.css?h={h_style}\""),
            ),
            ("src=\"app.js\"", format!("src=\"app.js?h={h_app}\"")),
        ],
    )?;

    println!("site montado em dist/");
    Ok(())
}
