//! Comandos do projeto. Uso: `cargo xtask <comando>`.
//!
//! - `site`: monta o site em `dist/` com Wasm SIMD, `wasm-opt` (se instalado), PWA e cache busting.
//! - `verify-js`: executa a suite de testes unitarios do frontend em Node.js.
//! - `bench`: monta o site, confere o tamanho do `.wasm` e roda testes de desempenho nativos e JS.
//! - `anti-rom`: procura ROM no repositorio (extensao, tamanho e hash).
//! - `rotulos`: gera `refs/rotulos.txt` (so o Marcelo roda; ver `rotulos.rs`).
//! - `video-estado` e `video-comparar`: renderizador contra o oraculo (ver `video.rs`).
//! - `inventario`: inventario de hardware a partir dos traces (ver `inventario.rs`).
//! - `inventario-memoria`: inventario da memoria compartilhada (ver `inventario_memoria.rs`).
//! - `bugs-conhecidos`: lista de verificacao dos trechos `FixBugs` (ver `bugs_conhecidos.rs`).
//! - `inventario-68000`, `inventario-barramento`, `inventario-interrupcoes`: ver `inventarios5.rs`.
//! - `inventario-som`, `ids-som`: inventario do som e lista de IDs (Etapa 6; ver `inventario_som.rs` e `ids_som.rs`).
//! - `corpus-meta`: escreve o `.meta` de um caso do corpus (ver `corpus.rs`).
//! - `tas`: converte e confere TAS do TASVideos (ver `tas.rs`).
//! - `descompressao-relatorio`: compara os descompressores com o oraculo (ver `descompressao.rs`).
//!
//! O comando de fidelidade, o manifesto, os testes ocultos e os traces do corpus sairam daqui na
//! 2.10: medem o que esta certo e ficam no pacote congelado `crates/fidelidade/relatorio`
//! (`cargo fidelidade`).
use std::path::PathBuf;
use std::process::{Command, ExitCode};

mod anti_rom;
mod bench;
mod bugs_conhecidos;
mod corpus;
mod descompressao;
mod gpu_comparar;
mod ids_som;
mod inventario;
mod inventario_memoria;
mod inventario_som;
mod inventarios5;
mod rotulos;
mod site;
mod tas;
mod video;

/// Raiz do repositorio (o xtask fica em `<raiz>/xtask`).
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
        .map_err(|e| format!("nao consegui rodar {cmd:?}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{cmd:?} terminou com {status}"))
    }
}

fn main() -> ExitCode {
    let comando = std::env::args().nth(1);
    let resto = || std::env::args().skip(2).collect::<Vec<_>>();
    let resultado = match comando.as_deref() {
        Some("site") => site::montar(),
        Some("verify-js") => bench::verificar_js(),
        Some("bench") => bench::benchmark(),
        Some("gpu-comparar") => gpu_comparar::executar(&resto()),
        Some("anti-rom") => anti_rom::executar(),
        Some("rotulos") => rotulos::executar(&resto()),
        Some("descompressao-relatorio") => descompressao::executar(&resto()),
        Some("corpus-meta") => corpus::escrever_meta(&resto()),
        Some("tas") => tas::executar(&resto()),
        Some("video-estado") => video::estado(&resto()),
        Some("video-comparar") => video::comparar(&resto()),
        Some("inventario") => inventario::executar(&resto()),
        Some("inventario-memoria") => inventario_memoria::executar(&resto()),
        Some("bugs-conhecidos") => bugs_conhecidos::executar(&resto()),
        Some("inventario-68000") => inventarios5::instrucoes(&resto()),
        Some("inventario-barramento") => inventarios5::barramento(&resto()),
        Some("inventario-interrupcoes") => inventarios5::interrupcoes(&resto()),
        Some("inventario-som") => inventario_som::executar(&resto()),
        Some("ids-som") => ids_som::executar(&resto()),
        _ => {
            eprintln!(
                "uso: cargo xtask <site|verify-js|bench|anti-rom|rotulos|descompressao-relatorio|corpus-meta|tas|inventario|inventario-memoria|inventario-68000|inventario-barramento|inventario-interrupcoes|inventario-som|ids-som|bugs-conhecidos|video-estado|video-comparar>"
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
