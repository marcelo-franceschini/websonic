//! Comparação local da sequência AOT com as rotinas já existentes do núcleo.
//! A ROM e o código gerado ficam fora do repositório.
#![cfg(not(target_arch = "wasm32"))]

use sonic1_recomp::descobrir_fragmento;
use sonic1_rom::{Veredito, identificar};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
#[ignore = "requer SONIC1_ROM apontando para a ROM REV01"]
fn boot_gerado_equivale_as_rotinas_manuais() {
    let rom_path = std::env::var_os("SONIC1_ROM").expect("defina SONIC1_ROM");
    let rom = std::fs::read(&rom_path).expect("leitura da ROM");
    assert_eq!(identificar(&rom).veredito, Veredito::Rev01);
    let grupos = [(0x206, 0x300), (0x300, 0x33e), (0x33e, 0x388)];
    let programas: Vec<_> = grupos
        .into_iter()
        .map(|(ini, fim)| descobrir_fragmento(&rom, ini, ini, fim).expect("geração do fragmento"))
        .collect();
    assert!(programas[0].saidas().any(|p| p == 0x300));
    assert!(programas[1].saidas().any(|p| p == 0x33e));
    let sufixo = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "sonic1-recomp-boot-{}-{sufixo}",
        std::process::id()
    ));
    std::fs::create_dir_all(dir.join("src")).unwrap();
    for (i, p) in programas.iter().enumerate() {
        std::fs::write(dir.join(format!("src/p{i}.rs")), p.gerar_rust()).unwrap();
    }
    let recomp = env!("CARGO_MANIFEST_DIR")
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    let core = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("core");
    let core = core
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    std::fs::write(dir.join("Cargo.toml"), format!("[package]\nname = \"prova-boot\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[dependencies]\nsonic1-recomp = {{ path = \"{recomp}\", default-features = false }}\nsonic1-core = {{ path = \"{core}\" }}\n")).unwrap();
    std::fs::write(
        dir.join("src/main.rs"),
        r#"
mod p0 { include!("p0.rs"); }
mod p1 { include!("p1.rs"); }
mod p2 { include!("p2.rs"); }
use sonic1_recomp::runtime::{Estado, Memoria, Parada};
fn main() {
    let rom = std::fs::read(std::env::var_os("SONIC1_ROM").unwrap()).unwrap();
    let mut manual = Memoria::nova(rom.clone());
    let mut recomp = Memoria::nova(rom);
    manual.ram_mut().fill(0xa5);
    recomp.ram_mut().fill(0xa5);
    sonic1_core::abertura::boot::entry_point(&mut manual).unwrap();
    sonic1_core::abertura::boot::game_program(&mut manual).unwrap();
    sonic1_core::abertura::boot::check_sum_ok(&mut manual).unwrap();
    sonic1_core::abertura::boot::game_init_limpar(&mut manual).unwrap();
    let mut e = Estado { pc: 0x206, ..Estado::default() };
    let mut passos = 0;
    while e.pc != 0x376 {
        passos += 1;
        assert!(passos < 2_000_000, "limite de instruções no endereço {:x}", e.pc);
        let r = if e.pc < 0x300 { p0::passo(&mut e, &mut recomp) }
                else if e.pc < 0x33e { p1::passo(&mut e, &mut recomp) }
                else { p2::passo(&mut e, &mut recomp) };
        assert!(matches!(r, Ok(Parada::Continua)), "parada no endereço {:x}", e.pc);
    }
    assert!(manual.ram() == recomp.ram(), "RAM 68000 divergiu");
    assert!(manual.ram_z80() == recomp.ram_z80(), "RAM Z80 divergiu");
    assert!(manual.saidas() == recomp.saidas(), "saídas divergiram");
    println!("passos={passos}, saidas={}", manual.saidas().len());
}
"#,
    )
    .unwrap();
    let resultado = Command::new("cargo")
        .args([
            "run",
            "--release",
            "--quiet",
            "--offline",
            "--manifest-path",
        ])
        .arg(dir.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", dir.join("target"))
        .env("SONIC1_ROM", rom_path)
        .output()
        .expect("compilação do programa gerado");
    assert!(
        resultado.status.success(),
        "prova falhou em {}: {}",
        dir.display(),
        String::from_utf8_lossy(&resultado.stderr)
    );
    println!("{}", String::from_utf8_lossy(&resultado.stdout));
    std::fs::remove_dir_all(dir).unwrap();
}
