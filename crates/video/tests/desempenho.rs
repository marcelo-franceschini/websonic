//! Medida de desempenho do renderizador (sub-etapa 4.9). Não faz parte do conjunto normal:
//! `WEBSONIC_BENCH=1 cargo test -p sonic1-video --release --test desempenho -- --nocapture`
//! Em wasm: `WEBSONIC_BENCH=1 cargo test --target wasm32-wasip1 -p sonic1-video --release --test
//! desempenho -- --nocapture` (com a variável passada ao wasmtime).
//! Orçamento (plano da Etapa 4): nativo até 1000 microssegundos por quadro e wasm até 4000.
//! O tempo é medido aqui, nos testes, e nunca no código do crate.

mod common;
use common::*;
use sonic1_video::renderizador::{Escrita, Renderizador};
use std::time::Instant;

/// Pior caso sintético: planos com rolagem por linha, sprites e 64 trocas de CRAM no meio do quadro.
fn quadro_pesado(r: &mut Renderizador, n: u16) {
    // 64 cores gravadas na linha 100, no meio da linha.
    r.avancar_ate(100, 1600);
    comando(r, 100, 0b0011, 0);
    for i in 0..64u16 {
        r.aplicar(
            100,
            1700,
            Escrita::Dados {
                valor: (i << 1) & 0x0eee,
                tamanho: 2,
            },
        );
    }
    let _ = n;
    r.fechar_quadro();
}

#[test]
fn desempenho_do_renderizador() {
    // Sem a variável, o teste não mede nada (assim ele não pesa no conjunto normal).
    if std::env::var_os("WEBSONIC_BENCH").is_none() {
        return;
    }
    let mut r = cena(7);
    // Rolagem por linha e tabela de rolagem cheia.
    reg(&mut r, 240, 11, 0x03);
    let mut tempos: Vec<u128> = Vec::new();
    for n in 0..220u16 {
        let t = Instant::now();
        quadro_pesado(&mut r, n);
        tempos.push(t.elapsed().as_micros());
    }
    // Os 20 primeiros quadros aquecem.
    let mut medidos: Vec<u128> = tempos.split_off(20);
    medidos.sort_unstable();
    let mediana = medidos[medidos.len() / 2];
    let maximo = medidos[medidos.len().saturating_sub(1)];
    let alvo = if cfg!(target_arch = "wasm32") {
        4000
    } else {
        1000
    };
    let veredito = if mediana <= alvo {
        "dentro"
    } else {
        "ESTOUROU"
    };
    println!("video-bench: mediana={mediana}us maximo={maximo}us orcamento={alvo}us {veredito}");
}
