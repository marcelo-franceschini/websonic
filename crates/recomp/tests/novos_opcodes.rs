//! Prova de compilacao e execucao dos novos opcodes e da fisica de objetos.
#![cfg(not(target_arch = "wasm32"))]

use sonic1_recomp::descobrir;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn novos_opcodes_compilam_e_executam() {
    let bytes_novos = [
        0x42, 0x00, // clr.b d0
        0x44, 0x41, // neg.w d1
        0x48, 0xc2, // ext.l d2
        0x48, 0x68, 0x00, 0x04, // pea 4(a0)
        0x43, 0xe8, 0x00, 0x0a, // lea $a(a0), a1
        0xd0, 0xfc, 0x00, 0x10, // adda.w #$10, a0
        0x91, 0xfc, 0x00, 0x00, 0x00, 0x04, // suba.l #4, a0
        0xb1, 0xfc, 0x00, 0x00, 0xff, 0x0c, // cmpa.l #$ff0c, a0
        0x52, 0x50, // addq.w #1, (a0)
        0x53, 0x50, // subq.w #1, (a0)
        0xe1, 0x88, // lsl.l #8, d0
        0xe2, 0x49, // lsr.w #1, d1
        0xe7, 0x5a, // rol.w #3, d2
        0xe6, 0x5a, // ror.w #3, d2
        0xe1, 0xd0, // asl.w (a0)
        0xb3, 0x50, // eor.w d1, (a0)
        0x4e, 0x75, // rts
    ];
    let prog_novos = descobrir(&bytes_novos, 0, 0, bytes_novos.len() as u32).unwrap();

    let sufixo = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "sonic1-recomp-opcodes-{}-{sufixo}",
        std::process::id()
    ));
    std::fs::create_dir_all(dir.join("src")).unwrap();
    let path = env!("CARGO_MANIFEST_DIR")
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    let manifesto = format!(
        "[package]\nname = \"recomp-opcodes-prova\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[dependencies]\nsonic1-recomp = {{ path = \"{path}\" }}\n"
    );
    std::fs::write(dir.join("Cargo.toml"), manifesto).unwrap();
    let principal = format!(
        r#"mod novos {{ {} }}
use sonic1_recomp::runtime::{{Estado, Memoria, Parada, Barramento, Tamanho}};

fn main() {{
    let mut m = Memoria::nova(vec![0; 0x100]);
    let mut e2 = Estado::default();
    e2.d[0] = 0xff;
    e2.d[1] = 0x0004;
    e2.d[2] = 0x8000;
    e2.a[0] = 0xff0000;
    e2.a[7] = 0xff0080;
    Barramento::escrever(&mut m, 0xff0080, Tamanho::Long, 0xbeef).unwrap();
    Barramento::escrever(&mut m, 0xff000c, Tamanho::Word, 0x0002).unwrap();

    loop {{
        let res = novos::passo(&mut e2, &mut m).unwrap();
        if res == Parada::Retorno {{
            break;
        }}
    }}
    assert_eq!(e2.d[0], 0);
    assert_eq!(e2.d[1] & 0xffff, 0x7ffe);
    assert_eq!(e2.d[2], 0xffff_8000);
    assert_eq!(e2.a[1], 0xff000a);
    assert_eq!(e2.a[0], 0xff000c);
    assert_eq!(Barramento::ler(&mut m, 0xff000c, Tamanho::Word).unwrap(), 0x7ffa);
    assert_eq!(e2.pc, 0xff0004);
    assert_eq!(e2.a[7], 0xff0080);
}}
"#,
        prog_novos.gerar_rust()
    );
    std::fs::write(dir.join("src/main.rs"), &principal).unwrap();
    let resultado = Command::new("cargo")
        .args(["run", "--quiet", "--offline", "--manifest-path"])
        .arg(dir.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", dir.join("target"))
        .output()
        .expect("cargo para compilar codigo gerado dos novos opcodes");
    if !resultado.status.success() {
        for (i, linha) in principal.lines().enumerate() {
            eprintln!("{:3}: {}", i + 1, linha);
        }
    }
    assert!(
        resultado.status.success(),
        "falha: {}",
        String::from_utf8_lossy(&resultado.stderr)
    );
    std::fs::remove_dir_all(dir).unwrap();
}
