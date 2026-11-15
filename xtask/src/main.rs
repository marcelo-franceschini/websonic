//! Comandos do projeto. Uso: `cargo xtask <comando>`.
//!
//! - `site`: monta o site em `dist/`.
//! - `anti-rom`: procura ROM no repositório (extensão, tamanho e hash).
//! - `rotulos`: gera `refs/rotulos.txt` (só o Marcelo roda; ver `rotulos.rs`).
//! - `video-estado` e `video-comparar`: renderizador contra o oráculo (ver `video.rs`).
//! - `inventario`: inventário de hardware a partir dos traces (ver `inventario.rs`).
//! - `inventario-memoria`: inventário da memória compartilhada (ver `inventario_memoria.rs`).
//! - `bugs-conhecidos`: lista de verificação dos trechos `FixBugs` (ver `bugs_conhecidos.rs`).
//! - `inventario-68000`, `inventario-barramento`, `inventario-interrupcoes`: ver `inventarios5.rs`.
//! - `inventario-som`, `ids-som`: inventário do som e lista de IDs (Etapa 6; ver `inventario_som.rs` e `ids_som.rs`).
//! - `corpus-meta`: escreve o `.meta` de um caso do corpus (ver `corpus.rs`).
//! - `tas`: converte e confere TAS do TASVideos (ver `tas.rs`).
//! - `descompressao-relatorio`: compara os descompressores com o oráculo (ver `descompressao.rs`).
//!
//! O comando de fidelidade, o manifesto, os testes ocultos e os traces do corpus saíram daqui na
//! 2.10: medem o que está certo e ficam no pacote congelado `crates/fidelidade/relatorio`
//! (`cargo fidelidade`).
use std::path::PathBuf;
use std::process::{Command, ExitCode};

mod anti_rom;
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
    let resto = || std::env::args().skip(2).collect::<Vec<_>>();
    let resultado = match comando.as_deref() {
        Some("site") => site::montar(),
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
                "uso: cargo xtask <site|anti-rom|rotulos|descompressao-relatorio|corpus-meta|tas|inventario|inventario-memoria|inventario-68000|inventario-barramento|inventario-interrupcoes|inventario-som|ids-som|bugs-conhecidos|video-estado|video-comparar>"
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
