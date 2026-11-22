//! `cargo xtask site`: compila o crate web para WebAssembly (SIMD128 + wasm-opt) e monta `dist/`.
use crate::{raiz, rodar};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Flags de compilacao Wasm modernas (suportadas em todos os navegadores atuais).
const WASM_RUSTFLAGS: &str = "-C target-feature=+simd128,+bulk-memory,+nontrapping-fptoint";

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

fn otimizar_wasm_se_disponivel(wasm_bg: &Path) -> Result<(), String> {
    let antes = fs::metadata(wasm_bg)
        .map_err(|e| format!("ler tamanho de {}: {e}", wasm_bg.display()))?
        .len();
    let tentativa = Command::new("wasm-opt")
        .args([
            "-O3",
            "--enable-simd",
            "--enable-bulk-memory",
            "--enable-nontrapping-float-to-int",
            "-o",
        ])
        .arg(wasm_bg)
        .arg(wasm_bg)
        .status();

    match tentativa {
        Ok(st) if st.success() => {
            let depois = fs::metadata(wasm_bg)
                .map_err(|e| format!("ler tamanho otimizado: {e}"))?
                .len();
            println!(
                "wasm-opt: {:.1} KiB -> {:.1} KiB",
                antes as f64 / 1024.0,
                depois as f64 / 1024.0
            );
            Ok(())
        }
        Ok(st) => Err(format!("wasm-opt terminou com {st}")),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!(
                "wasm-opt nao encontrado no PATH; mantendo binario ({:.1} KiB).",
                antes as f64 / 1024.0
            );
            Ok(())
        }
        Err(e) => Err(format!("falha ao rodar wasm-opt: {e}")),
    }
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

    // 1. Compila o crate web para WebAssembly com perfil wasm-release e SIMD128.
    let rustflags_atual = std::env::var("CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS")
        .map(|v| format!("{v} {WASM_RUSTFLAGS}"))
        .unwrap_or_else(|_| WASM_RUSTFLAGS.to_string());
    rodar(
        Command::new("cargo")
            .current_dir(&raiz)
            .env(
                "CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS",
                rustflags_atual,
            )
            .args(["build", "--profile", "wasm-release", "-p", "sonic1-web"])
            .args(["--target", "wasm32-unknown-unknown"]),
    )?;

    // 2. Gera o JavaScript de cola. O wasm-bindgen-cli tem a versao fixada na imagem.
    let wasm = raiz.join("target/wasm32-unknown-unknown/wasm-release/sonic1_web.wasm");
    rodar(
        Command::new("wasm-bindgen")
            .current_dir(&raiz)
            .args(["--target", "web", "--no-typescript", "--out-dir"])
            .arg(&pkg)
            .arg(&wasm),
    )?;

    // 2b. Otimiza o binario Wasm com wasm-opt (se disponivel no ambiente).
    let wasm_bg = pkg.join("sonic1_web_bg.wasm");
    otimizar_wasm_se_disponivel(&wasm_bg)?;

    // 3. Copia a pagina, modulos JS, PWA e arquivos estaticos.
    for nome in [
        "index.html",
        "style.css",
        "app.js",
        "audio.js",
        "video.js",
        "preferencias.js",
        "modal-controles.js",
        "entrada.js",
        "hud.js",
        "saves.js",
        "save-ui.js",
        "buffers.js",
        "pwa.js",
        "sw.js",
        "sim-worker.js",
        "audio-worklet.js",
        "gpu.js",
        "relogio.js",
        "manifest.webmanifest",
        "favicon.svg",
        "icon-192.png",
        "icon-512.png",
        "icon-maskable-512.png",
        "apple-touch-icon.png",
        "robots.txt",
        "sitemap.xml",
    ] {
        fs::copy(raiz.join("site").join(nome), dist.join(nome))
            .map_err(|e| format!("copiar site/{nome}: {e}"))?;
    }
    fs::write(dist.join(".nojekyll"), "").map_err(|e| format!("criar .nojekyll: {e}"))?;

    // 4. Cache busting automatico por hash de conteudo em ordem topologica de dependencias.
    let h_wasm_bg = hash_arquivo(&wasm_bg)?;
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
    let h_saves = hash_arquivo(&dist.join("saves.js"))?;
    let h_buffers = hash_arquivo(&dist.join("buffers.js"))?;
    let h_preferencias = hash_arquivo(&dist.join("preferencias.js"))?;
    let h_pwa = hash_arquivo(&dist.join("pwa.js"))?;
    let h_worklet = hash_arquivo(&dist.join("audio-worklet.js"))?;

    reescrever_referencias(
        &dist.join("save-ui.js"),
        &[("\"./saves.js\"", format!("\"./saves.js?h={h_saves}\""))],
    )?;
    let h_save_ui = hash_arquivo(&dist.join("save-ui.js"))?;

    reescrever_referencias(
        &dist.join("hud.js"),
        &[(
            "\"./relogio.js\"",
            format!("\"./relogio.js?h={h_relogio}\""),
        )],
    )?;
    let h_hud = hash_arquivo(&dist.join("hud.js"))?;

    reescrever_referencias(
        &dist.join("audio.js"),
        &[(
            "\"audio-worklet.js\"",
            format!("\"audio-worklet.js?h={h_worklet}\""),
        )],
    )?;
    let h_audio = hash_arquivo(&dist.join("audio.js"))?;

    reescrever_referencias(
        &dist.join("video.js"),
        &[("\"./gpu.js\"", format!("\"./gpu.js?h={h_gpu}\""))],
    )?;
    let h_video = hash_arquivo(&dist.join("video.js"))?;

    reescrever_referencias(
        &dist.join("modal-controles.js"),
        &[(
            "\"./entrada.js\"",
            format!("\"./entrada.js?h={h_entrada}\""),
        )],
    )?;
    let h_modal_controles = hash_arquivo(&dist.join("modal-controles.js"))?;

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
            (
                "\"./entrada.js\"",
                format!("\"./entrada.js?h={h_entrada}\""),
            ),
            ("\"./gpu.js\"", format!("\"./gpu.js?h={h_gpu}\"")),
            ("\"./saves.js\"", format!("\"./saves.js?h={h_saves}\"")),
            (
                "\"./buffers.js\"",
                format!("\"./buffers.js?h={h_buffers}\""),
            ),
        ],
    )?;
    let h_worker = hash_arquivo(&dist.join("sim-worker.js"))?;

    reescrever_referencias(
        &dist.join("app.js"),
        &[
            ("\"./audio.js\"", format!("\"./audio.js?h={h_audio}\"")),
            ("\"./video.js\"", format!("\"./video.js?h={h_video}\"")),
            (
                "\"./preferencias.js\"",
                format!("\"./preferencias.js?h={h_preferencias}\""),
            ),
            (
                "\"./modal-controles.js\"",
                format!("\"./modal-controles.js?h={h_modal_controles}\""),
            ),
            (
                "\"./entrada.js\"",
                format!("\"./entrada.js?h={h_entrada}\""),
            ),
            ("\"./hud.js\"", format!("\"./hud.js?h={h_hud}\"")),
            (
                "\"./save-ui.js\"",
                format!("\"./save-ui.js?h={h_save_ui}\""),
            ),
            ("\"./pwa.js\"", format!("\"./pwa.js?h={h_pwa}\"")),
            (
                "\"./sim-worker.js\"",
                format!("\"./sim-worker.js?h={h_worker}\""),
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

    // 5. Atualiza versao do cache do Service Worker e reescreve URLs com hash de conteudo.
    let versao_sw = format!("websonic-{h_app}-{h_wasm_bg}");
    reescrever_referencias(
        &dist.join("sw.js"),
        &[
            ("__WEBSONIC_CACHE_VERSION__", versao_sw),
            ("\"./style.css\"", format!("\"./style.css?h={h_style}\"")),
            ("\"./app.js\"", format!("\"./app.js?h={h_app}\"")),
            ("\"./audio.js\"", format!("\"./audio.js?h={h_audio}\"")),
            ("\"./video.js\"", format!("\"./video.js?h={h_video}\"")),
            (
                "\"./preferencias.js\"",
                format!("\"./preferencias.js?h={h_preferencias}\""),
            ),
            ("\"./gpu.js\"", format!("\"./gpu.js?h={h_gpu}\"")),
            (
                "\"./modal-controles.js\"",
                format!("\"./modal-controles.js?h={h_modal_controles}\""),
            ),
            (
                "\"./entrada.js\"",
                format!("\"./entrada.js?h={h_entrada}\""),
            ),
            ("\"./hud.js\"", format!("\"./hud.js?h={h_hud}\"")),
            ("\"./saves.js\"", format!("\"./saves.js?h={h_saves}\"")),
            (
                "\"./save-ui.js\"",
                format!("\"./save-ui.js?h={h_save_ui}\""),
            ),
            (
                "\"./buffers.js\"",
                format!("\"./buffers.js?h={h_buffers}\""),
            ),
            ("\"./pwa.js\"", format!("\"./pwa.js?h={h_pwa}\"")),
            (
                "\"./relogio.js\"",
                format!("\"./relogio.js?h={h_relogio}\""),
            ),
            (
                "\"./sim-worker.js\"",
                format!("\"./sim-worker.js?h={h_worker}\""),
            ),
            (
                "\"./audio-worklet.js\"",
                format!("\"./audio-worklet.js?h={h_worklet}\""),
            ),
            (
                "\"./pkg/sonic1_web.js\"",
                format!("\"./pkg/sonic1_web.js?h={h_wasm_js}\""),
            ),
            (
                "\"./pkg/sonic1_web_bg.wasm\"",
                format!("\"./pkg/sonic1_web_bg.wasm?h={h_wasm_bg}\""),
            ),
        ],
    )?;

    println!("site montado em dist/");
    Ok(())
}
