//! O comando `diferencial rotina` ponta a ponta, com ROM sintética (nenhum dado da ROM do jogo).
//! A rotina "KosDec" da ROM sintética só faz RTS: o oráculo não escreve nada, e o núcleo
//! descomprime de verdade, então o caso tem de sair DIFERE (prova que o comando enxerga a
//! diferença de RAM) e o resto do pipeline tem de rodar sem erro.

use sonic1_diferencial::rom_com_trecho;
use std::process::Command;

#[test]
fn comando_rotina_roda_e_acusa_diferenca_de_ram() {
    let dir = std::env::temp_dir().join(format!("websonic-cli-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("meta")).unwrap();
    // Fluxo Kosinski mínimo: descrição 0xFFFF (literais), 3 literais e depois o fim.
    // Palavra de descrição little-endian: bits 1 = literal. Três literais, depois bit 0 (cópia),
    // bit 1 (separada) e o terceiro byte 0 termina: bits 1,1,1,0,1 => 0b10111 = 0x17.
    let fluxo = [0x17u8, 0x00, 0xAA, 0xBB, 0xCC, 0x00, 0xF0, 0x00];
    let mut rom = rom_com_trecho(&[]);
    rom[0x500..0x500 + fluxo.len()].copy_from_slice(&fluxo);
    std::fs::write(dir.join("rom.bin"), &rom).unwrap();
    std::fs::write(
        dir.join("rotulos.txt"),
        "KosDec\trotina\t400\nBlk\trotina\t500\n",
    )
    .unwrap();
    std::fs::write(dir.join("manifesto.txt"), "Blk\tkosinski\t-\n").unwrap();
    let todos = "d0 d1 d2 d3 d4 d5 d6 d7 a0 a1 a2 a3 a4 a5 a6 ccr";
    std::fs::write(
        dir.join("meta/KosDec.txt"),
        format!("rotulo: KosDec\nusados_depois:\nignorados: {todos}\n"),
    )
    .unwrap();
    let saida = Command::new(env!("CARGO_BIN_EXE_diferencial"))
        .args(["rotina", "KosDec", "--rom"])
        .arg(dir.join("rom.bin"))
        .arg("--rotulos")
        .arg(dir.join("rotulos.txt"))
        .arg("--manifesto")
        .arg(dir.join("manifesto.txt"))
        .arg("--metadados")
        .arg(dir.join("meta"))
        .output()
        .unwrap();
    let texto = String::from_utf8_lossy(&saida.stdout).into_owned();
    assert!(
        texto.contains("KosDec Blk tile=0000 DIFERE ram_difere=3"),
        "{texto}"
    );
    assert!(
        texto.contains("KosDec: 1 casos, 1 falhas, 0 ausentes"),
        "{texto}"
    );
    assert!(!saida.status.success());
    let _ = std::fs::remove_dir_all(&dir);
}
