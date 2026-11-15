//! Compila o Rust emitido e exercita os dois caminhos de um programa sintético.
#![cfg(not(target_arch = "wasm32"))]

use sonic1_recomp::descobrir;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn codigo_gerado_compila_e_executa_ramos() {
    let bytes = [
        0x70, 0, 0x67, 4, 0x72, 1, 0x4e, 0x75, 0x72, 0xff, 0x4e, 0x75,
    ];
    let programa = descobrir(&bytes, 0, 0, bytes.len() as u32).unwrap();
    let bytes2 = [
        0x20, 0x7c, 0x00, 0xff, 0x00, 0x00, 0x10, 0xfc, 0x00, 0x80, 0x61, 0x04, 0x70, 0x07, 0x4e,
        0x75, 0x52, 0x00, 0x4e, 0x75,
    ];
    let programa2 = descobrir(&bytes2, 0, 0, bytes2.len() as u32).unwrap();
    let bytes3 = [0x4e, 0x73];
    let programa3 = descobrir(&bytes3, 0, 0, bytes3.len() as u32).unwrap();
    let sufixo = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("sonic1-recomp-{}-{sufixo}", std::process::id()));
    std::fs::create_dir_all(dir.join("src")).unwrap();
    let path = env!("CARGO_MANIFEST_DIR")
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    let manifesto = format!(
        "[package]\nname = \"recomp-prova\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[dependencies]\nsonic1-recomp = {{ path = \"{path}\" }}\n"
    );
    std::fs::write(dir.join("Cargo.toml"), manifesto).unwrap();
    let principal = format!(
        r#"{}
mod segundo {{ {} }}
mod terceiro {{ {} }}
fn main() {{
    let mut e = Estado {{ ccr: 16, ..Estado::default() }};
    let mut m = sonic1_recomp::runtime::Memoria::nova(vec![0; 32]);
    e.a[7] = 0xff0000;
    Barramento::escrever(&mut m, 0xff0000, Tamanho::Long, 0x400).unwrap();
    assert_eq!(passo(&mut e, &mut m).unwrap(), Parada::Continua);
    assert_eq!(e.ccr, 20);
    assert_eq!(e.ciclos_base, Some(4));
    assert_eq!(passo(&mut e, &mut m).unwrap(), Parada::Continua);
    assert_eq!(e.pc, 8);
    assert_eq!(e.ciclos_base, Some(14));
    assert_eq!(passo(&mut e, &mut m).unwrap(), Parada::Continua);
    assert_eq!(e.d[1], u32::MAX);
    assert_eq!(passo(&mut e, &mut m).unwrap(), Parada::Retorno);
    assert_eq!(e.pc, 0x400);
    assert_eq!(e.a[7], 0xff0004);
    assert_eq!(e.ciclos_base, Some(34));
    e.pc = 2; e.ccr = 0;
    assert_eq!(passo(&mut e, &mut m).unwrap(), Parada::Continua);
    assert_eq!(e.pc, 4);
    assert_eq!(passo(&mut e, &mut m).unwrap(), Parada::Continua);
    assert_eq!(e.d[1], 1);
    e.pc = 100;
    assert_eq!(passo(&mut e, &mut m).unwrap(), Parada::EnderecoNaoGerado(100));
    let mut e = Estado {{ ccr: 0x10, ..Estado::default() }};
    e.a[7] = 0xff0100;
    Barramento::escrever(&mut m, 0xff0100, Tamanho::Long, 0x800).unwrap();
    assert_eq!(segundo::passo(&mut e, &mut m).unwrap(), Parada::Continua);
    assert_eq!(e.a[0], 0xff0000);
    assert_eq!(e.ciclos_base, None);
    assert_eq!(segundo::passo(&mut e, &mut m).unwrap(), Parada::Continua);
    assert_eq!(Barramento::ler(&mut m, 0xff0000, Tamanho::Byte).unwrap(), 0x80);
    assert_eq!(e.a[0], 0xff0001);
    assert_eq!(e.ccr, 0x18);
    assert_eq!(segundo::passo(&mut e, &mut m).unwrap(), Parada::Continua);
    assert_eq!(e.pc, 16);
    assert_eq!(e.ultimo_custo_base, Some(18));
    assert_eq!(e.a[7], 0xff00fc);
    assert_eq!(Barramento::ler(&mut m, e.a[7], Tamanho::Long).unwrap(), 12);
    assert_eq!(segundo::passo(&mut e, &mut m).unwrap(), Parada::Continua);
    assert_eq!(e.d[0], 1);
    assert_eq!(segundo::passo(&mut e, &mut m).unwrap(), Parada::Retorno);
    assert_eq!(e.pc, 12);
    assert_eq!(e.a[7], 0xff0100);
    assert_eq!(segundo::passo(&mut e, &mut m).unwrap(), Parada::Continua);
    assert_eq!(e.d[0], 7);
    assert_eq!(segundo::passo(&mut e, &mut m).unwrap(), Parada::Retorno);
    assert_eq!(e.pc, 0x800);
    let mut e = Estado::default();
    e.a[7] = 0xff0200;
    Barramento::escrever(&mut m, 0xff0200, Tamanho::Word, 0x2700).unwrap();
    Barramento::escrever(&mut m, 0xff0202, Tamanho::Long, 0x9876).unwrap();
    assert_eq!(terceiro::passo(&mut e, &mut m).unwrap(), Parada::RetornoInterrupcao);
    assert_eq!(e.pc, 0x9876);
    assert_eq!(e.ciclos_base, Some(20));
}}
"#,
        programa.gerar_rust(),
        programa2.gerar_rust(),
        programa3.gerar_rust()
    );
    std::fs::write(dir.join("src/main.rs"), principal).unwrap();
    let resultado = Command::new("cargo")
        .args(["run", "--quiet", "--offline", "--manifest-path"])
        .arg(dir.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", dir.join("target"))
        .output()
        .expect("cargo para compilar código gerado");
    assert!(
        resultado.status.success(),
        "{}",
        String::from_utf8_lossy(&resultado.stderr)
    );
    std::fs::remove_dir_all(dir).unwrap();
}
