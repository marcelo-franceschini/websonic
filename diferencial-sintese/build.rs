// Compila os dois chips de referencia (C) e um shim pequeno, sem a crate `cc`.
// O fonte dos Nuked nao esta no repositorio: vem de NUKED_DIR ou de $SONIC1_EXT_DIR/trabalho/nuked
// (preparado por scripts/etapa8-bloco1.sh, em commits fixados).
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let ext = std::env::var("SONIC1_EXT_DIR").unwrap_or_else(|_| {
        format!("{}/sonic1-ext", std::env::var("HOME").expect("HOME"))
    });
    let nuked = PathBuf::from(std::env::var("NUKED_DIR").unwrap_or(format!("{ext}/trabalho/nuked")));
    let opn2 = nuked.join("Nuked-OPN2");
    let psg = nuked.join("Nuked-PSG");
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let fontes = [
        (opn2.join("ym3438.c"), "ym3438.o"),
        (psg.join("ympsg.c"), "ympsg.o"),
        (PathBuf::from("csrc/shim.c"), "shim.o"),
    ];
    let mut objetos = Vec::new();
    for (fonte, obj) in &fontes {
        let o = out.join(obj);
        let st = Command::new("cc")
            .args(["-O2", "-fPIC", "-c", "-w"])
            .arg("-I").arg(&opn2)
            .arg("-I").arg(&psg)
            .arg(fonte)
            .arg("-o").arg(&o)
            .status()
            .expect("cc");
        assert!(st.success(), "falha ao compilar {}", fonte.display());
        objetos.push(o);
        println!("cargo:rerun-if-changed={}", fonte.display());
    }
    let lib = out.join("libnuked.a");
    let _ = std::fs::remove_file(&lib);
    let st = Command::new("ar").arg("crs").arg(&lib).args(&objetos).status().expect("ar");
    assert!(st.success());
    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static=nuked");
    println!("cargo:rerun-if-env-changed=NUKED_DIR");
    println!("cargo:rerun-if-changed=csrc/shim.c");
}
