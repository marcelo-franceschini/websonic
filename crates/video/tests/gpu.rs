//! Comparação de quadros sintéticos entre a referência CPU e os dados consumidos pela GPU.
#![allow(clippy::arithmetic_side_effects)]

#[allow(dead_code)]
mod common;
use sonic1_video::camadas::{
    FonteDeTile, LARGURA_WIDESCREEN, ParametrosDosPlanos, PontoDeTile, Sprite, desenhar_sprites,
    fontes_planos, fontes_sprites, resolver_ponto,
};
use sonic1_video::quadro_gpu::QuadroGpu;
use sonic1_video::registros::Registros;
use sonic1_video::renderizador::{Escrita, Renderizador};

fn ponto(byte: u8, atributos: u8) -> PontoDeTile {
    PontoDeTile {
        cor: if atributos & 0x80 != 0 {
            byte >> 4
        } else {
            byte & 15
        },
        paleta: (atributos >> 4) & 3,
        prioridade: atributos & 0x40 != 0,
    }
}

fn comparar(r: &Renderizador) {
    let gpu = r.quadro_gpu().unwrap();
    for y in 0..224usize {
        for x in 0..320usize {
            let i = y * 320 + x;
            let seletor = gpu.seletores()[i];
            let cor = if seletor & 0x8000 != 0 {
                seletor & 0x1ff
            } else {
                let camadas = &gpu.camadas()[i * 4..i * 4 + 4];
                let a = ponto(camadas[0], camadas[1]);
                let b = ponto(camadas[2], camadas[3]);
                let sprite = (0..20usize)
                    .find_map(|s| {
                        let j = ((y * 20 + s) * 320 + x) * 2;
                        let p = ponto(gpu.sprites()[j], gpu.sprites()[j + 1]);
                        (p.cor != 0).then_some(p)
                    })
                    .unwrap_or_default();
                let paleta = usize::from(seletor) * 64;
                let indice = usize::from(resolver_ponto(sprite, a, b, gpu.fundos()[i]));
                gpu.paletas()[paleta + indice]
            };
            assert_eq!(Some(cor), r.quadro().ponto(x, y), "x={x} y={y}");
        }
    }
}

#[test]
fn camadas_e_sprites_de_cenas_sinteticas_coincidem_com_cpu() {
    for semente in [1, 19, 12345] {
        let mut r = common::cena(semente);
        r.ativar_gpu();
        r.avancar_ate(240, 0);
        r.avancar_ate(232, 0);
        comparar(&r);
    }
}

#[test]
fn mudanca_da_cram_no_meio_da_linha_coincide_com_cpu() {
    let mut r = common::cena(81);
    r.ativar_gpu();
    r.avancar_ate(240, 0);
    r.aplicar(40, 500, Escrita::Controle(0xC000));
    r.aplicar(40, 500, Escrita::Controle(0));
    r.aplicar(
        40,
        800,
        Escrita::Dados {
            valor: 0x0e0e,
            tamanho: 2,
        },
    );
    r.avancar_ate(232, 0);
    comparar(&r);
}

#[test]
fn fontes_preservam_sprites_sobrepostos_flips_e_mascaras() {
    let mut sorteio = common::Sorteio(0x12345678);
    let vram: Vec<u8> = (0..65536).map(|_| sorteio.byte()).collect();
    for variante in 0..32u16 {
        let sprites: Vec<Sprite> = (0..20u16)
            .map(|i| Sprite {
                padrao: i * 4,
                v_pos: 128,
                h_pos: if variante & 1 != 0 && i == 3 {
                    0
                } else {
                    128 + (i % 4) * 8
                },
                larg: 1 + u8::try_from(i % 4).unwrap(),
                alt: 2,
                paleta: u8::try_from(i % 4).unwrap(),
                v_flip: (i + variante) & 1 != 0,
                h_flip: (i + variante) & 2 != 0,
                prioridade: i & 1 != 0,
            })
            .collect();
        let mut esperado = vec![PontoDeTile::default(); 320];
        let mut fontes = vec![FonteDeTile::default(); 320 * 20];
        let anterior = variante & 4 != 0;
        let estouro = desenhar_sprites(&vram, &sprites, 5, variante % 5, anterior, &mut esperado);
        assert_eq!(
            estouro,
            fontes_sprites(&vram, &sprites, 5, variante % 5, anterior, &mut fontes)
        );
        for (x, ponto_cpu) in esperado.iter().enumerate() {
            let gpu = (0..20usize)
                .map(|i| fontes[i * 320 + x].decodificar())
                .find(|p| p.cor != 0)
                .unwrap_or_default();
            assert_eq!(ponto_cpu.cor, gpu.cor, "variante={variante} x={x}");
            if gpu.cor != 0 {
                assert_eq!(*ponto_cpu, gpu, "variante={variante} x={x}");
            }
        }
    }
}

#[test]
fn modo_gpu_sem_referencia_produz_as_mesmas_fontes() {
    let mut referencia = common::cena(317);
    let mut somente_gpu = common::cena(317);
    referencia.ativar_gpu();
    somente_gpu.ativar_gpu_sem_referencia();
    for r in [&mut referencia, &mut somente_gpu] {
        r.avancar_ate(240, 0);
        r.aplicar(40, 500, Escrita::Controle(0xC000));
        r.aplicar(40, 500, Escrita::Controle(0));
        r.aplicar(
            40,
            800,
            Escrita::Dados {
                valor: 0x0e0e,
                tamanho: 2,
            },
        );
        r.avancar_ate(232, 0);
    }
    comparar(&referencia);
    let a = referencia.quadro_gpu().unwrap();
    let b = somente_gpu.quadro_gpu().unwrap();
    assert_eq!(a.camadas(), b.camadas());
    assert_eq!(a.sprites(), b.sprites());
    assert_eq!(a.fundos(), b.fundos());
    assert_eq!(a.seletores(), b.seletores());
    assert_eq!(a.paletas(), b.paletas());
}

#[test]
fn compactacao_preserva_sprites_em_linhas_distintas_ate_o_limite() {
    let mut quadro = QuadroGpu::novo();
    let mut fontes = vec![FonteDeTile::default(); 320 * 20];
    for i in 0..20usize {
        fontes[i * 320 + 7] = FonteDeTile {
            byte: u8::try_from(i + 1).unwrap(),
            atributos: 0x80,
        };
    }
    quadro.sprites_da_linha(0, 0, 1, &fontes);
    quadro.sprites_da_linha(1, 0, 3, &fontes);
    let compacto = quadro.sprites_compactos();
    assert_eq!(compacto.len(), 224 * 320 * 3 * 2);
    assert_eq!(compacto[7 * 2], 1);
    assert_eq!(compacto[(320 + 7) * 2], 0);
    assert_eq!(compacto[((3 + 2) * 320 + 7) * 2], 3);
    quadro.sprites_da_linha(2, 0, 20, &fontes);
    let completo = quadro.sprites_compactos();
    assert_eq!(completo.len(), 224 * 320 * 20 * 2);
    assert_eq!(completo[((2 * 20 + 19) * 320 + 7) * 2], 20);
    assert_eq!(completo[((20 + 3) * 320 + 7) * 2], 0);

    // Depois de virar o quadro, uma linha sem sprites não pode reusar dados antigos.
    quadro.iniciar_quadro();
    quadro.sprites_da_linha(1, 0, 3, &fontes);
    let seguinte = quadro.sprites_compactos();
    assert_eq!(seguinte.len(), 224 * 320 * 3 * 2);
    assert_eq!(seguinte[7 * 2], 0);
    assert_eq!(seguinte[((3 + 2) * 320 + 7) * 2], 3);
}

#[test]
fn widescreen_quadro_gpu_configuravel_e_compactacao() {
    let mut quadro = QuadroGpu::widescreen_16_9();
    assert_eq!(quadro.largura(), LARGURA_WIDESCREEN);
    assert_eq!(quadro.altura(), 224);
    assert_eq!(quadro.pontos_total(), 426 * 224);
    assert_eq!(quadro.camadas().len(), 426 * 224 * 4);
    assert_eq!(quadro.fundos().len(), 426 * 224);
    assert_eq!(quadro.seletores().len(), 426 * 224);

    // Grava ponto no extremo direito da área widescreen (x = 425, y = 223).
    quadro.ponto(
        425,
        223,
        FonteDeTile {
            byte: 0x12,
            atributos: 0x80,
        },
        FonteDeTile {
            byte: 0x34,
            atributos: 0x40,
        },
        0x07,
        0x01,
    );
    let idx = 223 * 426 + 425;
    assert_eq!(
        &quadro.camadas()[idx * 4..idx * 4 + 4],
        &[0x12, 0x80, 0x34, 0x40]
    );
    assert_eq!(quadro.fundos()[idx], 0x07);
    assert_eq!(quadro.seletores()[idx], 0x01);

    // Cor direta fora da borda nominal mas dentro de widescreen.
    quadro.cor_direta(420, 10, 0x155);
    assert_eq!(quadro.seletores()[10 * 426 + 420], 0x8000 | 0x155);

    // Sprites compactos com largura expandida.
    let mut fontes = vec![FonteDeTile::default(); 426 * 20];
    fontes[400] = FonteDeTile {
        byte: 0x55,
        atributos: 0x80,
    };
    quadro.sprites_da_linha(0, 0, 2, &fontes);
    let compacto = quadro.sprites_compactos();
    assert_eq!(compacto.len(), 224 * 426 * 2 * 2);
    assert_eq!(compacto[400 * 2], 0x55);
}

#[test]
fn widescreen_fontes_planos_cobrem_alem_de_320_pixels() {
    let mut vram = vec![0u8; 65536];
    // Tile 1 preenchido com nibbles 0x7 na linha 0.
    vram[32..36].copy_from_slice(&[0x77, 0x77, 0x77, 0x77]);
    // Plano A em 0xC000: celula_h 48 usa tile 1 (pixel 48 * 8 = 384).
    // Indice relativo em bytes: 2 * celula_h = 2 * 48 = 96.
    vram[0xc000 + 96..0xc000 + 98].copy_from_slice(&[0x00, 0x01]);

    let mut reg = Registros::novo();
    reg.brutos[2] = 0x30; // A em 0xC000
    reg.brutos[4] = 0x07; // B em 0xE000
    reg.brutos[16] = 0x01; // 64 x 32
    let vsram = vec![0u16; 40];

    let mut buf_a = vec![FonteDeTile::default(); 426];
    let mut buf_b = vec![FonteDeTile::default(); 426];
    let par = ParametrosDosPlanos {
        vram: &vram,
        vsram: &vsram,
        reg: &reg,
        linha: 0,
        rolagem_h: (0, 0),
        v_tela_cheia: (0, 0),
        ini: 0,
        fim: 426,
    };
    fontes_planos(&par, &mut buf_a, &mut buf_b);

    // O tile 1 aparece na coluna 24 (pixels 384 a 391), alem dos 320 nominais.
    let dec = buf_a[384].decodificar();
    assert_eq!(
        dec.cor, 7,
        "Pixel em x=384 deve decodificar a cor do tile 1"
    );
    assert!(!dec.prioridade);
}

#[test]
fn widescreen_fontes_sprites_capturam_alem_de_320_pixels() {
    let mut vram = vec![0u8; 65536];
    vram[32..36].copy_from_slice(&[0x22, 0x22, 0x22, 0x22]);
    // Sprite posicionado em h_pos = 0x80 + 350 (fora dos 320 nominais, dentro dos 426 de widescreen).
    let sprites = vec![Sprite {
        padrao: 1,
        v_pos: 0x80,
        h_pos: 0x80 + 350,
        larg: 1,
        alt: 1,
        paleta: 1,
        v_flip: false,
        h_flip: false,
        prioridade: true,
    }];
    let mut fontes = vec![FonteDeTile::default(); 426 * 20];
    let estouro = fontes_sprites(&vram, &sprites, 0, 0, false, &mut fontes);
    assert!(!estouro);

    // Verifica que o sprite foi gravado em x = 350.
    let ponto = fontes[350].decodificar();
    assert_eq!(ponto.cor, 2);
    assert_eq!(ponto.paleta, 1);
    assert!(ponto.prioridade);
}

#[test]
fn renderizador_modo_widescreen_preenche_viewport_expandido() {
    let mut r = common::cena(42);
    r.ativar_gpu_widescreen();
    r.avancar_ate(240, 0);
    r.avancar_ate(232, 0);

    let gpu = r.quadro_gpu().unwrap();
    assert_eq!(gpu.largura(), 426);
    assert_eq!(gpu.altura(), 224);
    assert_eq!(gpu.camadas().len(), 426 * 224 * 4);
    assert_eq!(gpu.fundos().len(), 426 * 224);
    assert_eq!(gpu.seletores().len(), 426 * 224);

    let compacto = gpu.sprites_compactos();
    assert_eq!(compacto.len() % (426 * 224 * 2), 0);
}
