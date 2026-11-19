//! Guarda da síntese e das taxas do site, sem o oráculo. (1) Amostras nativas de FM (misturado) e
//! de PSG (empacotado), quadro a quadro, nas três entradas do corpus. (2) Áudio de saída nas taxas
//! que o site passa ao jogo a 48 kHz e a 44,1 kHz (taxa do contexto x 59,92 / 60), na partida-ghz1.
//! Uma correção de desempenho não pode mudar nenhum desses hashes.
use sonic1_console::Console;

const CORPUS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus");
const FNV_INICIO: u64 = 0xcbf2_9ce4_8422_2325;
/// Fator de `site/app.js` (`iniciar`): relógio NTSC por quadro sobre 60 Hz exatos.
const FATOR_SITE: f64 = 53_693_175.0 / 896_040.0 / 60.0;

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

/// Roda as entradas e devolve (hash do FM nativo, hash do PSG nativo, hash do áudio de saída).
fn hashes(rom: &[u8], entradas: &[u8], taxa: f64) -> (u64, u64, u64) {
    let mut c = Console::com_taxa_audio(rom.to_vec(), taxa);
    let (mut fm, mut psg, mut audio) = (FNV_INICIO, FNV_INICIO, FNV_INICIO);
    for &b in entradas {
        // Ocupação fixa no alvo: o hash não depende dos ganhos do controlador.
        c.definir_ocupacao_audio((taxa / 20.0) as usize);
        c.definir_botoes(b);
        c.simular_quadro();
        let (f, p) = c.amostras_nativas();
        for &(l, r) in f {
            fnv(&mut fm, &l.to_le_bytes());
            fnv(&mut fm, &r.to_le_bytes());
        }
        for &x in p {
            fnv(&mut psg, &x.to_le_bytes());
        }
        for s in c.audio_slice() {
            fnv(&mut audio, &s.to_bits().to_le_bytes());
        }
    }
    (fm, psg, audio)
}

#[test]
#[ignore = "requer SONIC1_ROM"]
fn sintese_nativa_e_taxas_do_site_nao_mudam() {
    let rom = std::fs::read(std::env::var_os("SONIC1_ROM").expect("SONIC1_ROM")).expect("ROM");
    let partida = ler_s1in("partida-ghz1.s1in");
    // Valores medidos em 18/11/2026 no HEAD 2786b6e (docs/relatorio-performance-2.md, seção 5.3).
    let nativos: [(&str, Vec<u8>, (u64, u64)); 3] = [
        (
            "boot-demos",
            vec![0; 7200],
            (0x8561_d2b4_79ba_2f15, 0xbeb5_8e4c_f534_4dce),
        ),
        (
            "partida-ghz1",
            partida.clone(),
            (0xc0c7_6ef6_f6d5_2543, 0x2d52_01b2_8f03_05d7),
        ),
        (
            "lz-agua",
            ler_s1in("lz-agua.s1in"),
            (0x8481_756c_8d4d_d831, 0x0ec9_1ce6_b13d_347d),
        ),
    ];
    for (nome, entradas, esperado) in nativos {
        let (fm, psg, _) = hashes(&rom, &entradas, 48_000.0);
        assert_eq!(
            (fm, psg),
            esperado,
            "{nome}: amostras nativas de FM ou PSG mudaram"
        );
    }
    let taxas = [
        (48_000.0 * FATOR_SITE, 0xb668_4b4f_abae_80e3),
        (44_100.0 * FATOR_SITE, 0x0f33_1b5b_6cc8_31bb),
    ];
    for (taxa, esperado) in taxas {
        let (_, _, audio) = hashes(&rom, &partida, taxa);
        assert_eq!(
            audio, esperado,
            "partida-ghz1 a {taxa} Hz: áudio de saída mudou"
        );
    }
}
