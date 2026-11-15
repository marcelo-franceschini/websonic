#![cfg(not(target_arch = "wasm32"))]

use std::process::Command;

#[test]
fn console_hashes_sinaliza_falha_do_nucleo_no_codigo_de_saida() {
    let caminho =
        std::env::temp_dir().join(format!("websonic-rom-sintetica-{}", std::process::id()));
    let mut rom = vec![0u8; 0x4000];
    rom[0x201] = 1;
    std::fs::write(&caminho, rom).unwrap();
    let saida = Command::new(env!("CARGO_BIN_EXE_console_hashes"))
        .arg(&caminho)
        .arg("10")
        .output()
        .unwrap();
    std::fs::remove_file(caminho).unwrap();
    assert!(!saida.status.success());
    assert!(String::from_utf8_lossy(&saida.stdout).contains("ChecksumErrado"));
}
