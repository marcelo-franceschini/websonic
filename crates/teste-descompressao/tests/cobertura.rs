//! Cobertura das decisões do núcleo nos descompressores (sub-etapa 5.6).
//!
//! Roda entradas sintéticas e confere que TODA decisão anotada com `decisao!` no código dos
//! descompressores foi tomada nos dois caminhos. A lista de decisões vem da leitura do próprio
//! código-fonte (as chamadas `decisao!("Rotina:nome", ...)`), então uma decisão nova sem teste
//! reprova este teste. Com `WEBSONIC_COBERTURA_ARQ=ARQ`, grava `id<TAB>verdadeiras<TAB>falsas`.

use sonic1_core::cobertura;
use sonic1_core::descompressao::{enigma, kosinski, nemesis};
use sonic1_teste_descompressao::{Gerador, enigma as ce, kosinski as ck, nemesis as cn};

const LIMITE: usize = 1 << 20;

/// Ids literais de `decisao!("...")` num texto de código-fonte.
fn ids_do_fonte(fonte: &str) -> Vec<String> {
    let mut v = Vec::new();
    for parte in fonte.split("decisao!(\"").skip(1) {
        if let Some(fim) = parte.find('"') {
            v.push(parte[..fim].to_string());
        }
    }
    v
}

#[test]
fn toda_decisao_dos_descompressores_vai_nos_dois_caminhos() {
    cobertura::zerar();
    let mut g = Gerador::novo(7);
    // Kosinski: dados variados, e o vetor manual com contagem 1 (a decisão "pulo").
    for dados in [
        vec![],
        vec![0x42],
        vec![0; 300],
        g.bytes(1000),
        g.bytes_repetitivos(5000),
    ] {
        let c = ck::comprimir(&dados);
        kosinski::descomprimir(&c, LIMITE).unwrap();
    }
    let fluxo = [0x15, 0x00, 0x41, 0x00, 0xF0, 0x01, 0x00, 0xF0, 0x00];
    kosinski::descomprimir(&fluxo, 16).unwrap();
    // Enigma: palavras com corridas, incrementos, decrementos e valores soltos.
    for caso in 0..30u64 {
        let n = [1usize, 15, 100, 1000][(caso % 4) as usize];
        let palavras: Vec<u16> = (0..n)
            .map(|i| match (i / 7 + usize::try_from(caso).unwrap()) % 4 {
                0 => 0x0100,
                1 => u16::try_from(i & 0x7FF).unwrap(),
                2 => 0x0200,
                _ => g.proximo() as u16 & 0x7FF,
            })
            .collect();
        let p = ce::parametros_para(&palavras, 0x0100, 0x0200);
        let c = ce::comprimir(&palavras, &p);
        enigma::descomprimir(&c, 0x2345, LIMITE).unwrap();
    }
    // Nemesis não tem decisões anotadas ainda (backlog); roda só para manter o par.
    let tiles = g.bytes(32 * 8);
    nemesis::descomprimir(&cn::comprimir(&tiles, false), LIMITE).unwrap();

    let contagens = cobertura::contagens();
    let mut ids = Vec::new();
    for fonte in [
        include_str!("../../core/src/descompressao/kosinski.rs"),
        include_str!("../../core/src/descompressao/enigma.rs"),
        include_str!("../../core/src/descompressao/nemesis.rs"),
    ] {
        ids.extend(ids_do_fonte(fonte));
    }
    assert!(!ids.is_empty());
    let mut ruins = Vec::new();
    for id in &ids {
        match contagens.get(id.as_str()) {
            None => ruins.push(format!("{id}: nunca executada")),
            Some((v, f)) if *v == 0 || *f == 0 => {
                ruins.push(format!("{id}: so um caminho ({v} verdadeiras, {f} falsas)"))
            }
            Some(_) => {}
        }
    }
    if let Ok(arq) = std::env::var("WEBSONIC_COBERTURA_ARQ") {
        let linhas: String = contagens
            .iter()
            .map(|(k, (v, f))| format!("{k}\t{v}\t{f}\n"))
            .collect();
        std::fs::write(arq, linhas).unwrap();
    }
    assert!(ruins.is_empty(), "{}", ruins.join("\n"));
}

#[test]
fn leitor_de_ids_acha_as_decisoes() {
    assert_eq!(
        ids_do_fonte("if decisao!(\"A:x\", y) { decisao!(\"B:z\", w) }"),
        vec!["A:x", "B:z"]
    );
    assert!(ids_do_fonte("nada aqui").is_empty());
}
