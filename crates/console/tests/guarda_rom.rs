//! Guarda de comportamento sem o oráculo: hashes agregados da imagem de 9 bits, da RAM do 68000 e
//! do áudio de saída, quadro a quadro, nas três entradas do corpus. Uma correção de desempenho não
//! pode mudar nenhum deles. Valores atualizados após implementar Ring_Loss (objeto 0x37).
use sonic1_console::Console;

const CORPUS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus");
const FNV_INICIO: u64 = 0xcbf2_9ce4_8422_2325;

/// FNV-1a de 64 bits, o mesmo de `Ponte::hash_da_imagem`.
fn fnv(h: &mut u64, bytes: &[u8]) {
    for &b in bytes {
        *h ^= u64::from(b);
        *h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
}

fn ler_s1in(nome: &str) -> Vec<u8> {
    std::fs::read_to_string(format!("{CORPUS}/{nome}"))
        .expect("entrada do corpus")
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .map(|s| u8::from_str_radix(s, 16).expect("hex"))
        .collect()
}

/// Roda as entradas no caminho de CPU, com a ocupação do worklet fixa no alvo (2400 pares a
/// 48 kHz). Assim o hash de áudio não depende dos ganhos do controlador.
fn hashes(rom: &[u8], entradas: &[u8]) -> (u64, u64, u64) {
    let mut c = Console::com_taxa_audio(rom.to_vec(), 48_000.0);
    let (mut img, mut ram, mut audio) = (FNV_INICIO, FNV_INICIO, FNV_INICIO);
    for &b in entradas {
        c.definir_ocupacao_audio(2400);
        c.definir_botoes(b);
        c.simular_quadro();
        for s in c.audio_slice() {
            fnv(&mut audio, &s.to_bits().to_le_bytes());
        }
        let mut r = FNV_INICIO;
        fnv(&mut r, &c.ab.mem.ram()[..]);
        fnv(&mut ram, &r.to_le_bytes());
        fnv(&mut img, &c.hash_da_imagem().to_le_bytes());
    }
    (img, ram, audio)
}

#[test]
#[ignore = "requer SONIC1_ROM"]
fn imagem_ram_e_audio_nao_mudam() {
    let rom = std::fs::read(std::env::var_os("SONIC1_ROM").expect("SONIC1_ROM")).expect("ROM");
    let casos: [(&str, Vec<u8>, (u64, u64, u64)); 3] = [
        ("boot-demos", vec![0; 7200], (0x996b_d534_0a62_17d1, 0x6943_10d4_5770_69dd, 0x2f96_f7e5_16f5_5c0c)),
        ("partida-ghz1", ler_s1in("partida-ghz1.s1in"), (0xfdd3_b2ed_a83d_bed7, 0x2831_6235_fcd1_5484, 0xc198_a3f5_bac8_e79d)),
        ("lz-agua", ler_s1in("lz-agua.s1in"), (0xb597_b314_aa7a_1bad, 0xe6fe_0adf_7f95_8864, 0x3012_0edb_efc8_e9c8)),
    ];
    for (nome, entradas, esperado) in casos {
        assert_eq!(hashes(&rom, &entradas), esperado, "{nome}: imagem, RAM ou áudio mudou");
    }
}
