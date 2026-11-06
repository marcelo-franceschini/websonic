//! Testes dos descompressores com dados sintéticos. Nada daqui vem da ROM.
//! Os compressores de teste foram escritos a partir da descrição do formato; a prova de
//! fidelidade contra o jogo é o relatório do oráculo (`cargo xtask descompressao-relatorio`).

use sonic1_core::descompressao::{Erro, enigma, kosinski, nemesis};
use sonic1_teste_descompressao::{Gerador, enigma as ce, kosinski as ck, nemesis as cn};

const LIMITE: usize = 1 << 20;

fn entradas_kos() -> Vec<Vec<u8>> {
    let mut g = Gerador::novo(7);
    let mut v = vec![
        vec![],
        vec![0x42],
        vec![0; 300],
        vec![1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3],
        g.bytes(1000),
        g.bytes_repetitivos(5000),
        g.bytes_repetitivos(40_000),
    ];
    // Tamanhos ao redor das fronteiras de 16 bits de descrição.
    for n in 14..40 {
        v.push(g.bytes_repetitivos(n));
    }
    v
}

#[test]
fn kosinski_ida_e_volta() {
    for dados in entradas_kos() {
        let c = ck::comprimir(&dados);
        let r = kosinski::descomprimir(&c, LIMITE).expect("deve descomprimir");
        assert_eq!(r.saida, dados);
        assert!(r.consumidos <= c.len());
    }
}

#[test]
fn kosinski_copia_com_sobreposicao_e_distancias_limite() {
    // Distancia 1 (corrida) e distancia 8192 (maximo da janela).
    let mut d = vec![9u8; 50];
    let mut g = Gerador::novo(3);
    let ruido = g.bytes(8192);
    d.extend_from_slice(&ruido);
    d.extend_from_slice(&ruido[..40]);
    let c = ck::comprimir(&d);
    assert_eq!(kosinski::descomprimir(&c, LIMITE).unwrap().saida, d);
    // Cópia longa: 256 bytes iguais.
    let d = vec![5u8; 600];
    let c = ck::comprimir(&d);
    assert_eq!(kosinski::descomprimir(&c, LIMITE).unwrap().saida, d);
}

#[test]
fn kosinski_vetor_manual_com_contagem_um_sem_saida() {
    // Descricao: literal, depois copia separada longa com contagem 1 (sem saida), depois
    // terminador. Bits (LSB primeiro): 1, 0,1, 0,1 -> 0b10101 = 0x15.
    let fluxo = [0x15, 0x00, 0x41, 0x00, 0xF0, 0x01, 0x00, 0xF0, 0x00];
    let r = kosinski::descomprimir(&fluxo, 16).unwrap();
    assert_eq!(r.saida, vec![0x41]);
    assert_eq!(r.consumidos, fluxo.len());
}

#[test]
fn kosinski_erros() {
    assert_eq!(kosinski::descomprimir(&[], 10), Err(Erro::EntradaTruncada));
    let c = ck::comprimir(&[7u8; 100]);
    // Limite menor que a saida.
    assert_eq!(kosinski::descomprimir(&c, 10), Err(Erro::SaidaExcedeLimite));
    // Distancia maior que a saida: copia inline logo no inicio (bits 0,0,0,0 e distancia 1).
    let ruim = [0x00, 0x00, 0xFF];
    assert!(matches!(
        kosinski::descomprimir(&ruim, 10),
        Err(Erro::FormatoInvalido(_))
    ));
}

fn palavras_aleatorias(g: &mut Gerador, n: usize, modo: u64) -> Vec<u16> {
    let mut v: Vec<u16> = Vec::new();
    while v.len() < n {
        match g.ate(5) {
            0 => v.push(g.proximo() as u16 & if modo == 0 { 0x7FF } else { 0xFFFF }),
            1 => {
                let w = g.proximo() as u16 & 0x7FF;
                v.extend(std::iter::repeat_n(w, g.ate(20) as usize + 1));
            }
            2 => {
                let w = g.proximo() as u16 & 0x7FF;
                for k in 0..g.ate(20) as u16 + 1 {
                    v.push(w.wrapping_add(k));
                }
            }
            3 => {
                let w = g.proximo() as u16 & 0x7FF;
                for k in 0..g.ate(20) as u16 + 1 {
                    v.push(w.wrapping_sub(k));
                }
            }
            _ => v.push(0x0100),
        }
    }
    v.truncate(n);
    v
}

fn bytes_be(p: &[u16], inicio: u16) -> Vec<u8> {
    p.iter()
        .flat_map(|w| w.wrapping_add(inicio).to_be_bytes())
        .collect()
}

#[test]
fn enigma_ida_e_volta_com_tile_inicial() {
    let mut g = Gerador::novo(11);
    for caso in 0..60u64 {
        let n = [0usize, 1, 15, 16, 17, 100, 1000][(caso % 7) as usize];
        let palavras = palavras_aleatorias(&mut g, n, caso % 2);
        let p = ce::parametros_para(&palavras, 0x0100, 0x0000);
        let c = ce::comprimir(&palavras, &p);
        for inicio in [0u16, 0x2345, 0xFFFF] {
            let r = enigma::descomprimir(&c, inicio, LIMITE).expect("deve descomprimir");
            assert_eq!(r.saida, bytes_be(&palavras, inicio), "caso {caso}");
            assert!(r.consumidos <= c.len());
        }
    }
}

#[test]
fn enigma_erros() {
    assert_eq!(enigma::descomprimir(&[], 0, 10), Err(Erro::EntradaTruncada));
    assert_eq!(
        enigma::descomprimir(&[0, 0, 0, 0, 0], 0, 10),
        Err(Erro::EntradaTruncada)
    );
    assert!(matches!(
        enigma::descomprimir(&[17, 0, 0, 0, 0, 0, 0xFF], 0, 10),
        Err(Erro::FormatoInvalido(_))
    ));
    assert!(matches!(
        enigma::descomprimir(&[5, 0xE0, 0, 0, 0, 0, 0xFF], 0, 10),
        Err(Erro::FormatoInvalido(_))
    ));
    let p = ce::parametros_para(&[1, 1, 1, 1], 0, 0);
    let c = ce::comprimir(&[1, 1, 1, 1], &p);
    assert_eq!(enigma::descomprimir(&c, 0, 3), Err(Erro::SaidaExcedeLimite));
}

fn tiles_aleatorios(g: &mut Gerador, tiles: usize) -> Vec<u8> {
    let mut v = g.bytes_repetitivos(tiles * 32);
    // Mais nibbles altos e zeros, para usar todos os tipos de código.
    for (i, b) in v.iter_mut().enumerate() {
        if i % 7 == 0 {
            *b = 0;
        }
    }
    v
}

#[test]
fn nemesis_ida_e_volta_com_e_sem_xor() {
    let mut g = Gerador::novo(21);
    for tiles in [0usize, 1, 2, 5, 40, 300] {
        for xor in [false, true] {
            let dados = tiles_aleatorios(&mut g, tiles);
            let c = cn::comprimir(&dados, xor);
            let r = nemesis::descomprimir(&c, LIMITE).expect("deve descomprimir");
            assert_eq!(r.saida, dados, "tiles={tiles} xor={xor}");
            assert!(r.consumidos <= c.len());
        }
    }
}

#[test]
fn nemesis_retomavel_da_o_mesmo_resultado() {
    let mut g = Gerador::novo(5);
    let dados = tiles_aleatorios(&mut g, 64);
    for xor in [false, true] {
        let c = cn::comprimir(&dados, xor);
        let inteiro = nemesis::descomprimir(&c, LIMITE).unwrap().saida;
        let mut n = nemesis::Nemesis::novo(&c, LIMITE).unwrap();
        assert_eq!(n.tiles_total(), 64);
        let mut feito = 0;
        while !n.terminou() {
            feito += 3;
            n.executar_ate(feito).unwrap();
            // O que já saiu é sempre um prefixo do resultado final.
            assert_eq!(n.saida(), &inteiro[..n.saida().len()]);
        }
        assert_eq!(n.saida(), &inteiro[..]);
        assert_eq!(inteiro, dados);
    }
}

#[test]
fn nemesis_erros() {
    assert_eq!(nemesis::descomprimir(&[], 10), Err(Erro::EntradaTruncada));
    // Contagem de tiles no maximo (0x7FFF) com limite menor: recusa antes de decodificar.
    assert_eq!(
        nemesis::descomprimir(&[0x7F, 0xFF, 0xFF], 1000),
        Err(Erro::SaidaExcedeLimite)
    );
    // Tabela sem terminador.
    assert_eq!(
        nemesis::descomprimir(&[0x00, 0x01, 0x80, 0x12, 0x00], 100),
        Err(Erro::EntradaTruncada)
    );
    // Tamanho de codigo zero.
    assert!(matches!(
        nemesis::descomprimir(&[0x00, 0x01, 0x80, 0x00, 0x00, 0xFF], 100),
        Err(Erro::FormatoInvalido(_))
    ));
    // Codigo sem entrada: tabela vazia e dados qualquer.
    assert!(matches!(
        nemesis::descomprimir(&[0x00, 0x01, 0xFF, 0x00, 0x00], 100),
        Err(Erro::FormatoInvalido(_))
    ));
    // Zero tiles: saida vazia.
    assert_eq!(
        nemesis::descomprimir(&[0x00, 0x00], 0).unwrap().saida,
        Vec::<u8>::new()
    );
}

/// Propriedades que valem para qualquer entrada: sem panico, saida limitada, consumo dentro
/// da entrada, determinismo.
fn confere(entrada: &[u8], limite: usize) {
    type R = Result<sonic1_core::descompressao::Resultado, Erro>;
    type Decodificador<'a> = &'a dyn Fn(&[u8]) -> R;
    let fs: [Decodificador<'_>; 4] = [
        &|e| kosinski::descomprimir(e, limite),
        &|e| enigma::descomprimir(e, 0, limite),
        &|e| enigma::descomprimir(e, 0x1234, limite),
        &|e| nemesis::descomprimir(e, limite),
    ];
    for f in fs {
        let a = f(entrada);
        assert_eq!(a, f(entrada), "determinismo");
        if let Ok(r) = a {
            assert!(r.saida.len() <= limite);
            assert!(r.consumidos <= entrada.len());
        }
    }
}

#[test]
fn entrada_aleatoria_nunca_panica() {
    let mut g = Gerador::novo(99);
    for _ in 0..3000 {
        let n = g.ate(200) as usize;
        let e = g.bytes(n);
        confere(&e, 4096);
    }
}

#[test]
fn entrada_truncada_e_corrompida_nunca_panica() {
    let mut g = Gerador::novo(1234);
    let dados = g.bytes_repetitivos(800);
    let palavras = palavras_aleatorias(&mut g, 300, 1);
    let p = ce::parametros_para(&palavras, 0x0100, 0);
    let tiles = tiles_aleatorios(&mut g, 10);
    let validos = [
        ck::comprimir(&dados),
        ce::comprimir(&palavras, &p),
        cn::comprimir(&tiles, true),
        cn::comprimir(&tiles, false),
    ];
    for v in &validos {
        // Todos os prefixos.
        for corte in 0..v.len() {
            confere(&v[..corte], 1 << 16);
        }
        // Um bit invertido em cada posicao.
        for byte in 0..v.len() {
            for bit in 0..8 {
                let mut m = v.clone();
                m[byte] ^= 1 << bit;
                confere(&m, 1 << 16);
            }
        }
    }
}
