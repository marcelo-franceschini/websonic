//! Guarda de comportamento sem o oráculo: hashes agregados da imagem de 9 bits, da RAM do 68000 e
//! do áudio de saída, quadro a quadro, nas três entradas do corpus. Uma correção de desempenho não
//! pode mudar nenhum deles. Os valores só mudam por uma mudança de comportamento aprovada pelo
//! Marcelo: em 17/11/2026, o RingLoss e o HurtSonic fiéis ao disassembly (revisão R6); em
//! 19/11/2026, o DMA Fill síncrono no VDP corrigindo pixels espúrios da abertura e transições.
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
    type Caso = (&'static str, Vec<u8>, (u64, u64, u64));
    let casos: [Caso; 3] = [
        (
            "boot-demos",
            vec![0; 7200],
            (
                0x31ff_a080_6c6b_f089,
                0x53dc_aed7_42e5_4cba,
                0xa566_fb94_9619_7a58,
            ),
        ),
        (
            "partida-ghz1",
            ler_s1in("partida-ghz1.s1in"),
            (
                0x3b7f_c415_150a_6e91,
                0x5068_4ee5_3074_a4d8,
                0x0cdc_57e4_c3cf_af1b,
            ),
        ),
        (
            "lz-agua",
            ler_s1in("lz-agua.s1in"),
            (
                0x7fe1_8c6c_4143_34d9,
                0x41b8_ca12_0d65_ba8e,
                0xc6f7_7b45_fbd5_4140,
            ),
        ),
    ];
    for (nome, entradas, esperado) in casos {
        assert_eq!(
            hashes(&rom, &entradas),
            esperado,
            "{nome}: imagem, RAM ou áudio mudou"
        );
    }
}
