//! `cargo xtask verify-js` e `cargo xtask bench`: verificacao do frontend e guardrail de tamanho/performance.
use crate::{raiz, rodar, site};
use std::fs;
use std::process::Command;

/// Limite superior de alerta para o binario Wasm otimizado (em bytes).
const LIMITE_WASM_BYTES: u64 = 1_500_000;

/// Roda a suite unitaria do frontend com `node --test`.
pub fn verificar_js() -> Result<(), String> {
    let raiz = raiz();
    rodar(Command::new("node").current_dir(&raiz).args([
        "--test",
        "site/tests/relogio.test.mjs",
        "site/tests/ritmo-60hz.test.mjs",
        "site/tests/audio-worklet.test.cjs",
        "site/tests/audio-worklet-carga.test.cjs",
        "site/tests/audio-worklet-canal.test.cjs",
        "site/tests/entrada.test.mjs",
        "site/tests/saves.test.mjs",
        "site/tests/save-ui.test.mjs",
        "site/tests/buffers.test.mjs",
        "site/tests/hud.test.mjs",
        "site/tests/video.test.mjs",
        "site/tests/audio.test.mjs",
        "site/tests/preferencias.test.mjs",
        "site/tests/server.test.mjs",
    ]))
}

/// Monta o site em release Wasm, confere o orcamento de tamanho do `.wasm` e roda benchmarks nativos.
pub fn benchmark() -> Result<(), String> {
    let raiz = raiz();
    site::montar()?;

    let wasm_path = raiz.join("dist/pkg/sonic1_web_bg.wasm");
    let meta = fs::metadata(&wasm_path)
        .map_err(|e| format!("ler tamanho de {}: {e}", wasm_path.display()))?;
    let bytes = meta.len();
    let kib = bytes as f64 / 1024.0;
    println!("bench: sonic1_web_bg.wasm = {bytes} bytes ({kib:.1} KiB)");
    if bytes > LIMITE_WASM_BYTES {
        return Err(format!(
            "tamanho do Wasm ({bytes} bytes) ultrapassou o limite de {LIMITE_WASM_BYTES} bytes"
        ));
    }

    // Benchmarks deterministicos nativos em release (video e audio-saida).
    rodar(Command::new("cargo").current_dir(&raiz).args([
        "test",
        "--release",
        "-p",
        "sonic1-video",
        "--test",
        "desempenho",
        "--",
        "--nocapture",
    ]))?;

    rodar(Command::new("cargo").current_dir(&raiz).args([
        "test",
        "--release",
        "-p",
        "sonic1-audio-saida",
        "--test",
        "desempenho",
        "--",
        "--nocapture",
    ]))?;

    verificar_js()?;
    println!("bench: verificacoes de tamanho, video, audio e JS concluidas.");
    Ok(())
}
