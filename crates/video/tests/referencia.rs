//! Cruza o renderizador com um desenhista de referência simples (ponto a ponto, sem cache de
//! linha) e prova propriedades. Só dados sintéticos gerados aqui. Sem dado da ROM.
//!
//! A referência é escrita separada do renderizador: para cada ponto (x, y) ela vai direto à
//! VRAM, à CRAM e à VSRAM finais. Vale para cenas estáticas (nada muda depois do começo).

use sonic1_video::renderizador::{Escrita, Renderizador};

// ---- Aritmética explícita (os lints do núcleo valem nos testes) ----
fn add(a: i32, b: i32) -> i32 {
    a.wrapping_add(b)
}
fn sub(a: i32, b: i32) -> i32 {
    a.wrapping_sub(b)
}
fn mul(a: i32, b: i32) -> i32 {
    a.wrapping_mul(b)
}
fn modulo(a: i32, m: i32) -> i32 {
    a.rem_euclid(m)
}
fn u(i: i32) -> usize {
    usize::try_from(i).unwrap_or(0)
}

/// Gerador pseudoaleatório xorshift com semente fixa.
struct Sorteio(u64);

impl Sorteio {
    fn proximo(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn byte(&mut self) -> u8 {
        self.proximo().to_le_bytes()[0]
    }
    fn palavra(&mut self) -> u16 {
        u16::from_le_bytes([
            self.proximo().to_le_bytes()[1],
            self.proximo().to_le_bytes()[2],
        ])
    }
    fn ate(&mut self, n: u16) -> u16 {
        u16::try_from(self.proximo().checked_rem(u64::from(n)).unwrap_or(0)).unwrap_or(0)
    }
}

fn reg(r: &mut Renderizador, linha: u16, n: u8, v: u8) {
    let w = 0x8000u16 | (u16::from(n) << 8) | u16::from(v);
    r.aplicar(linha, 0, Escrita::Controle(w));
}

fn comando(r: &mut Renderizador, linha: u16, codigo_alto: u16, endereco: u32) {
    // Primeira palavra: CD1 CD0 e A13..A0. Segunda: CD5..CD2 e A16..A14.
    let primeira = (codigo_alto & 3) << 14 | u16::try_from(endereco & 0x3fff).unwrap_or(0);
    let segunda =
        (((codigo_alto >> 2) & 0xf) << 4) | u16::try_from((endereco >> 14) & 7).unwrap_or(0);
    r.aplicar(linha, 0, Escrita::Controle(primeira));
    r.aplicar(linha, 0, Escrita::Controle(segunda));
}

fn dados(r: &mut Renderizador, linha: u16, v: u16) {
    r.aplicar(
        linha,
        0,
        Escrita::Dados {
            valor: v,
            tamanho: 2,
        },
    );
}

/// Cena aleatória estática. Devolve o renderizador depois de desenhar um quadro completo.
fn cena(semente: u64) -> Renderizador {
    let mut s = Sorteio(semente | 1);
    let mut r = Renderizador::novo();
    let l = 240; // dentro do VBlank do quadro anterior
    r.avancar_ate(l, 0);
    // Modo 5, 40 colunas, imagem ligada, DMA permitido, incremento 2.
    reg(&mut r, l, 1, 0x74);
    reg(&mut r, l, 12, 0x81);
    reg(&mut r, l, 0, 0x04);
    reg(&mut r, l, 15, 2);
    let nome_a: u8 = [0x30, 0x20, 0x10][u(i32::from(s.ate(3)))];
    let nome_b: u8 = [0x07, 0x05, 0x06][u(i32::from(s.ate(3)))];
    reg(&mut r, l, 2, nome_a);
    reg(&mut r, l, 4, nome_b);
    reg(&mut r, l, 5, 0x6c); // tabela de sprites em $D800
    reg(&mut r, l, 13, 0x37); // tabela de rolagem em $DC00
    reg(
        &mut r,
        l,
        16,
        [0x00, 0x01, 0x10, 0x11][u(i32::from(s.ate(4)))],
    );
    let modo_h = [0x00u8, 0x02, 0x03][u(i32::from(s.ate(3)))];
    let modo_v = if s.ate(2) == 0 { 0x00 } else { 0x04 };
    reg(&mut r, l, 11, modo_h | modo_v);
    reg(&mut r, l, 7, s.byte() & 0x3f);
    // VRAM inteira aleatória (por palavras), exceto que mantemos zeros em partes para haver transparência.
    comando(&mut r, l, 0b0001, 0);
    for i in 0..32768u32 {
        let mut w = s.palavra();
        if i % 3 == 0 {
            w = 0;
        }
        dados(&mut r, l, w);
    }
    // Tabela de sprites: 24 sprites encadeados, posições espalhadas.
    comando(&mut r, l, 0b0001, 0xd800);
    for i in 0..80u16 {
        let v = 0x80u16.wrapping_add(s.ate(224)).wrapping_sub(8);
        dados(&mut r, l, v & 0x1ff);
        let tam = u16::from(s.byte() & 0x0f);
        let elo = if i < 23 { i.wrapping_add(1) } else { 0 };
        dados(&mut r, l, (tam << 8) | elo);
        dados(&mut r, l, s.palavra() & 0xe7ff);
        let h = 0x80u16.wrapping_add(s.ate(340)).wrapping_sub(10);
        dados(&mut r, l, if i == 5 { 0 } else { h & 0x1ff });
    }
    // CRAM e VSRAM.
    comando(&mut r, l, 0b0011, 0);
    for _ in 0..64 {
        dados(&mut r, l, s.palavra() & 0x0eee);
    }
    comando(&mut r, l, 0b0101, 0);
    for _ in 0..40 {
        dados(&mut r, l, s.palavra() & 0x03ff);
    }
    r.drenar_vdp();
    // Um quadro inteiro para tudo assentar, depois outro para medir.
    r.avancar_ate(232, 0);
    r.avancar_ate(l, 0);
    r.avancar_ate(232, 0);
    r
}

// ---- Referência ----

fn be(v: &[u8], a: i32) -> u16 {
    let a = u(modulo(a, 65536));
    u16::from_be_bytes([
        v[a],
        v[u(modulo(add(i32::try_from(a).unwrap_or(0), 1), 65536))],
    ])
}

fn pixel_do_tile(vram: &[u8], padrao: i32, x: i32, y: i32, vf: bool, hf: bool) -> u8 {
    let yy = if vf { sub(7, y & 7) } else { y & 7 };
    let xx = if hf { sub(7, x & 7) } else { x & 7 };
    let a = modulo(add(mul(padrao, 32), add(mul(yy, 4), xx / 2)), 65536);
    let b = vram[u(a)];
    if xx % 2 == 0 { b >> 4 } else { b & 15 }
}

#[derive(Clone, Copy, Default)]
struct P {
    cor: u8,
    paleta: u8,
    prio: bool,
}

fn plano(r: &Renderizador, a: bool, x: i32, y: i32) -> P {
    let v = r.vdp();
    let g = &v.registros.brutos;
    let vram = v.vram();
    let linha_h = match g[11] & 3 {
        0 => i32::from(be(vram, i32::from(g[13] & 0x3f) << 10)),
        2 => i32::from(be(vram, add(i32::from(g[13] & 0x3f) << 10, mul(32, y / 8)))),
        _ => i32::from(be(vram, add(i32::from(g[13] & 0x3f) << 10, mul(4, y)))),
    };
    let (hs_a, hs_b) = match g[11] & 3 {
        0 => (
            linha_h,
            i32::from(be(vram, add(i32::from(g[13] & 0x3f) << 10, 2))),
        ),
        2 => (
            linha_h,
            i32::from(be(
                vram,
                add(add(i32::from(g[13] & 0x3f) << 10, mul(32, y / 8)), 2),
            )),
        ),
        _ => (
            linha_h,
            i32::from(be(
                vram,
                add(add(i32::from(g[13] & 0x3f) << 10, mul(4, y)), 2),
            )),
        ),
    };
    let hs = if a { hs_a } else { hs_b } & 0x3ff;
    let fino = hs & 15;
    // Coluna de 16 pontos onde a rolagem vertical por 2 células é lida.
    let coluna = if sub(x, fino) < 0 {
        -1
    } else {
        sub(x, fino) / 16
    };
    let vs = if g[11] & 4 == 0 {
        let i = if a { 0 } else { 1 };
        i32::from(v.vsram()[i])
    } else if coluna < 0 {
        i32::from(v.vsram()[0x4c >> 1] & v.vsram()[0x4e >> 1])
    } else if coluna < 20 {
        i32::from(v.vsram()[u(add(mul(coluna, 2), i32::from(!a)))])
    } else {
        0
    } & 0x3ff;
    let largura = match g[16] & 3 {
        0 => 32,
        1 => 64,
        3 => 128,
        _ => 1,
    };
    let (lp, ap) = match (g[16] & 3, (g[16] >> 4) & 3) {
        (2, _) => (256, 8),
        (h, 2) => (
            match h {
                0 => 256,
                1 => 512,
                _ => 1024,
            },
            256,
        ),
        (h, w) => (
            match h {
                0 => 256,
                1 => 512,
                _ => 1024,
            },
            match w {
                0 => 256,
                1 => 512,
                _ => 1024,
            },
        ),
    };
    let tx = modulo(sub(x, hs), lp);
    let ty = modulo(add(y, vs), ap);
    let base = if a {
        i32::from(g[2] & 0x38) << 10
    } else {
        i32::from(g[4] & 7) << 13
    };
    let rel = (mul(2, add(mul(ty / 8, largura), tx / 8))) & 0x1fff;
    let w = be(vram, add(base, rel));
    let padrao = i32::from(w & 0x7ff);
    let cor = pixel_do_tile(vram, padrao, tx, ty, w & 0x1000 != 0, w & 0x0800 != 0);
    P {
        cor,
        paleta: u8::try_from((w >> 13) & 3).unwrap_or(0),
        prio: w & 0x8000 != 0,
    }
}

/// Sprites da linha `y`, em ordem de ligação, na ordem em que o hardware os varre.
fn sprite(r: &Renderizador, y: i32, x: i32) -> P {
    let v = r.vdp();
    let vram = v.vram();
    let tabela = i32::from(v.registros.brutos[5] & 0x7e) << 9;
    let linha = add(0x80, y);
    // Varredura: até 20 sprites por linha.
    let mut ids = Vec::new();
    let mut i: i32 = 0;
    for _ in 0..80 {
        let b = add(tabela, mul(i, 8));
        let topo = i32::from(be(vram, b) & 0x1ff);
        let alt = add(i32::from(vram[u(add(b, 2))] & 3), 1);
        if topo <= linha && linha < add(topo, mul(alt, 8)) {
            if ids.len() == 20 {
                break;
            }
            ids.push(i);
        }
        i = i32::from(vram[u(add(b, 3))] & 0x7f);
        if i == 0 || i >= 80 {
            break;
        }
    }
    let mut pontos = vec![P::default(); 320];
    let mut tiles = 0;
    let mut achou = false;
    let mut mascara = false;
    'fora: for id in ids {
        let b = add(tabela, mul(id, 8));
        let topo = i32::from(be(vram, b) & 0x1ff);
        let tam = vram[u(add(b, 2))];
        let larg = add(i32::from((tam >> 2) & 3), 1);
        let alt = add(i32::from(tam & 3), 1);
        let attr = be(vram, add(b, 4));
        let hpos = i32::from(be(vram, add(b, 6)) & 0x1ff);
        if hpos == 0 && achou {
            mascara = true;
        } else if hpos != 0 {
            achou = true;
        }
        let vf = attr & 0x1000 != 0;
        let hf = attr & 0x0800 != 0;
        let mut ly = sub(linha, topo) & 0x1f;
        if vf {
            ly = sub(sub(mul(alt, 8), 1), ly) & 0x1f;
        }
        for h in 0..larg {
            if tiles == 40 {
                break 'fora;
            }
            tiles = add(tiles, 1);
            if mascara {
                continue;
            }
            let col = if hf { sub(sub(larg, 1), h) } else { h };
            let padrao = add(i32::from(attr & 0x7ff), add(mul(col, alt), ly / 8)) & 0xffff;
            for k in 0..8 {
                let sx = sub(add(hpos, add(mul(h, 8), k)), 0x80);
                if !(0..320).contains(&sx) {
                    continue;
                }
                let pc = pixel_do_tile(
                    vram,
                    padrao,
                    if hf { sub(7, k) } else { k },
                    ly & 7,
                    false,
                    false,
                );
                let p = &mut pontos[u(sx)];
                if p.cor == 0 {
                    *p = P {
                        cor: pc,
                        paleta: u8::try_from((attr >> 13) & 3).unwrap_or(0),
                        prio: attr & 0x8000 != 0,
                    };
                }
            }
        }
    }
    pontos[u(x)]
}

fn referencia(r: &Renderizador, x: i32, y: i32) -> u16 {
    let s = sprite(r, y, x);
    let a = plano(r, true, x, y);
    let b = plano(r, false, x, y);
    let fundo = r.vdp().registros.indice_do_fundo();
    let ordem = match (s.prio, a.prio, b.prio) {
        (false, false, false) | (true, false, false) | (true, true, false) | (true, true, true) => {
            [s, a, b]
        }
        (false, true, false) => [a, s, b],
        (false, false, true) => [b, s, a],
        (true, false, true) => [s, b, a],
        (false, true, true) => [a, b, s],
    };
    let mut idx = fundo;
    for p in ordem {
        if p.cor != 0 {
            idx = (p.paleta << 4) | p.cor;
            break;
        }
    }
    let w = r.vdp().cram()[usize::from(idx)];
    let (rr, gg, bb) = ((w >> 1) & 7, (w >> 5) & 7, (w >> 9) & 7);
    (bb << 6) | (gg << 3) | rr
}

#[test]
fn cenas_estaticas_batem_com_a_referencia() {
    for semente in 1..=6u64 {
        let r = cena(semente.wrapping_mul(0x9e37_79b9_7f4a_7c15));
        let mut diferentes = 0;
        let mut primeiro = None;
        for y in 0..224i32 {
            for x in 0..320i32 {
                let esperado = referencia(&r, x, y);
                let obtido = r.quadro().ponto(u(x), u(y)).unwrap();
                if esperado != obtido {
                    diferentes = add(diferentes, 1);
                    primeiro.get_or_insert((x, y, esperado, obtido));
                }
            }
        }
        assert_eq!(diferentes, 0, "semente {semente}: primeiro {primeiro:?}");
    }
}

#[test]
fn determinismo_e_nada_fora_do_quadro() {
    let a = cena(77);
    let b = cena(77);
    assert_eq!(a.quadro(), b.quadro());
    assert_eq!(a.quadro().pontos().len(), 320 * 224);
    assert!(a.quadro().pontos().iter().all(|p| *p < 512));
}

#[test]
fn escritas_aleatorias_nunca_dao_panico() {
    for semente in 1..=8u64 {
        let mut s = Sorteio(semente.wrapping_mul(0x2545_f491_4f6c_dd1d) | 1);
        let mut r = Renderizador::novo();
        let mut linha = 0u16;
        for _ in 0..4000 {
            if s.ate(5) == 0 {
                linha = (linha.wrapping_add(s.ate(40))) % 262;
            }
            let coluna = u64::from(s.ate(3420));
            let w = s.palavra();
            let e = match s.ate(4) {
                0 => Escrita::Controle(w),
                1 => Escrita::Controle(0x8000 | (w & 0x1fff)),
                2 => Escrita::Dados {
                    valor: w,
                    tamanho: 2,
                },
                _ => Escrita::Dados {
                    valor: w,
                    tamanho: 1,
                },
            };
            r.aplicar(linha, coluna, e);
            if s.ate(50) == 0 {
                let n = usize::from(s.ate(200));
                let ws: Vec<u16> = (0..n).map(|_| s.palavra()).collect();
                r.entregar_dma(&ws);
            }
        }
        r.fechar_quadro();
        assert_eq!(r.quadro().pontos().len(), 320 * 224);
        assert!(r.quadro().pontos().iter().all(|p| *p < 512));
    }
}

#[test]
fn as_cenas_nao_sao_triviais() {
    let r = cena(1);
    let mut distintas = std::collections::BTreeSet::new();
    for p in r.quadro().pontos() {
        distintas.insert(*p);
    }
    assert!(distintas.len() > 20, "so {} cores", distintas.len());
}
