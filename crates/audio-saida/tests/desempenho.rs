//! Medida de desempenho do sintetizador e saida de audio (sub-etapa 8.9).
//!
//! Nao faz parte do conjunto normal de testes:
//! `WEBSONIC_BENCH=1 cargo test -p sonic1-audio-saida --release --test desempenho -- --nocapture`
//!
//! Orçamento (plano da Etapa 8, secao 6):
//! 3 ms por quadro em wasm (3000 µs) e < 1 ms no nativo (1000 µs).

use sonic1_audio::{Chip, EscritaSom, MCLK_POR_QUADRO_NTSC, Sintetizador};
use sonic1_audio_saida::MisturadorSaida;
use std::time::Instant;

struct Rng(u64);
impl Rng {
    fn proximo(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.0
    }
    fn ate(&mut self, n: u64) -> u64 {
        self.proximo().checked_rem(n).unwrap_or(0)
    }
}

/// Gera escritas sinteticas densas simulando o pior caso de carga de audio em um quadro:
/// 6 canais FM ativos, 4 operadores por canal, LFO, modulacao de fase, DAC e 4 canais de PSG.
fn gerar_escritas_quadro_pesado(quadro: u64, rng: &mut Rng) -> Vec<EscritaSom> {
    let mut e = Vec::new();
    let mclk_base = quadro.wrapping_mul(MCLK_POR_QUADRO_NTSC);

    // Varias escritas em FM (frequencia, envelopes, DAC)
    for i in 0..15u64 {
        let t = mclk_base.wrapping_add(i.wrapping_mul(50_000).wrapping_add(rng.ate(2000)));
        let ch = u8::try_from(rng.ate(6)).unwrap();
        let (p, off) = if ch < 3 {
            (0u32, ch)
        } else {
            (2u32, ch.wrapping_sub(3))
        };
        // F-Number e bloco
        e.push(EscritaSom {
            mclk: t,
            chip: Chip::Fm,
            porta: p,
            valor: 0xA0 + off,
        });
        e.push(EscritaSom {
            mclk: t.wrapping_add(42),
            chip: Chip::Fm,
            porta: p + 1,
            valor: u8::try_from(rng.ate(256)).unwrap(),
        });

        // DAC
        e.push(EscritaSom {
            mclk: t.wrapping_add(84),
            chip: Chip::Fm,
            porta: 0,
            valor: 0x2A,
        });
        e.push(EscritaSom {
            mclk: t.wrapping_add(126),
            chip: Chip::Fm,
            porta: 1,
            valor: u8::try_from(rng.ate(256)).unwrap(),
        });
    }

    // Escritas de PSG (tons e ruido)
    for i in 0..10u64 {
        let t = mclk_base.wrapping_add(i.wrapping_mul(80_000).wrapping_add(rng.ate(1000)));
        e.push(EscritaSom {
            mclk: t,
            chip: Chip::Psg,
            porta: 0,
            valor: u8::try_from(rng.ate(256)).unwrap(),
        });
    }

    e
}

#[test]
fn desempenho_do_sintetizador_e_saida() {
    if std::env::var_os("WEBSONIC_BENCH").is_none() {
        return;
    }

    let mut sint = Sintetizador::novo();
    let mut misturador = MisturadorSaida::novo(48_000.0);
    let mut rng = Rng(0xDEAD_BEEF_CAFE_BABE);

    let mut fm_buf = Vec::new();
    let mut psg_buf = Vec::new();
    let mut worklet_buf = [0.0f32; 256];

    let mut tempos_us: Vec<u128> = Vec::with_capacity(600);

    for q in 1..=600u64 {
        let escritas = gerar_escritas_quadro_pesado(q, &mut rng);
        sint.agendar_lote(&escritas);

        let t0 = Instant::now();
        let mclk_alvo = q.wrapping_mul(MCLK_POR_QUADRO_NTSC);

        fm_buf.clear();
        psg_buf.clear();
        sint.avancar_ate(mclk_alvo, &mut fm_buf, &mut psg_buf);
        misturador.alimentar_quadro(&fm_buf, &psg_buf);

        while misturador.ocupacao_buffer() >= 128 {
            misturador.extrair_bloco_worklet(&mut worklet_buf);
        }

        tempos_us.push(t0.elapsed().as_micros());
    }

    // Descarta os primeiros 20 quadros de aquecimento
    let mut medidos = tempos_us.split_off(20);
    medidos.sort_unstable();

    let mediana = medidos[medidos.len() / 2];
    let p95 = medidos[(medidos.len() * 95) / 100];
    let maximo = medidos[medidos.len().saturating_sub(1)];

    let orcamento = 3000;
    let veredito = if mediana <= orcamento {
        "dentro"
    } else {
        "ESTOUROU"
    };

    println!(
        "audio-bench: mediana={mediana}us p95={p95}us maximo={maximo}us orcamento={orcamento}us {veredito}"
    );
}
