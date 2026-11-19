//! Desenho de uma linha: planos A e B, sprites e prioridade (sub-etapas 4.3 e 4.4).
//!
//! Funções puras sobre a VRAM, a CRAM e a VSRAM. O comportamento é o do oráculo (jgenesis
//! `386db855`, `vdp/render.rs` e `vdp/sprites.rs`), em 40 colunas, sem entrelaçamento, sem janela
//! e sem sombra e realce.

use crate::registros::{Registros, RolagemH, RolagemV, TamanhoPlano};

/// Largura nominal da área ativa.
pub const LARGURA: usize = 320;
/// Altura nominal da área ativa.
pub const ALTURA: usize = 224;
/// Colunas de 16 pontos na área ativa.
const COLUNAS: i32 = 20;
/// Início da área de exibição dos sprites (coordenada de sprite que cai no ponto 0).
pub const INICIO_SPRITES: u16 = 0x80;

/// Ponto de uma camada: índice de cor (0 é transparente), paleta e prioridade.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PontoDeTile {
    pub cor: u8,
    pub paleta: u8,
    pub prioridade: bool,
}

/// Os dois planos de rolagem.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Plano {
    A,
    B,
}

fn be16(vram: &[u8], a: u16) -> u16 {
    let hi = vram.get(usize::from(a)).copied().unwrap_or(0);
    let lo = vram
        .get(usize::from(a.wrapping_add(1)))
        .copied()
        .unwrap_or(0);
    u16::from_be_bytes([hi, lo])
}

/// Rolagem horizontal de uma linha, lida da tabela na VRAM (`read_h_scroll` do oráculo).
pub fn ler_rolagem_h(vram: &[u8], reg: &Registros, linha: u16) -> (u16, u16) {
    let base = reg.tabela_de_rolagem_h();
    let endereco = match reg.rolagem_h() {
        RolagemH::Tela => base,
        RolagemH::Celula => base.wrapping_add(32u16.wrapping_mul(linha / 8)),
        RolagemH::Linha => base.wrapping_add(4u16.wrapping_mul(linha)),
        RolagemH::Invalida => base.wrapping_add(4u16.wrapping_mul(linha & 7)),
    };
    (
        be16(vram, endereco) & 0x03ff,
        be16(vram, endereco.wrapping_add(2)) & 0x03ff,
    )
}

fn rolagem_v(
    vsram: &[u16],
    reg: &Registros,
    plano: Plano,
    coluna: i32,
    tela_cheia: (u16, u16),
) -> u16 {
    let v = match reg.rolagem_v() {
        RolagemV::Tela => match plano {
            Plano::A => tela_cheia.0,
            Plano::B => tela_cheia.1,
        },
        RolagemV::DuasCelulas => {
            if coluna < 0 {
                // A coluna -1 usa VSRAM[$4C] & VSRAM[$4E] nos dois planos (40 colunas).
                let a = vsram.get(0x4c >> 1).copied().unwrap_or(0);
                let b = vsram.get(0x4e >> 1).copied().unwrap_or(0);
                a & b
            } else if coluna < COLUNAS {
                let desloc = usize::from(plano == Plano::B);
                let i = usize::try_from(coluna)
                    .unwrap_or(0)
                    .saturating_mul(2)
                    .saturating_add(desloc);
                vsram.get(i).copied().unwrap_or(0)
            } else {
                let col = usize::try_from(coluna).unwrap_or(0);
                let modulo = usize::try_from(COLUNAS).unwrap_or(20);
                let col_efetiva = col.checked_rem(modulo).unwrap_or(0);
                let desloc = usize::from(plano == Plano::B);
                let i = col_efetiva.saturating_mul(2).saturating_add(desloc);
                vsram.get(i).copied().unwrap_or(0)
            }
        }
    };
    v & 0x03ff
}

#[derive(Clone, Copy, Default)]
struct Nome {
    prioridade: bool,
    paleta: u8,
    v_flip: bool,
    h_flip: bool,
    padrao: u16,
}

fn ler_nome(vram: &[u8], base: u16, largura: u16, linha: u16, coluna: u16) -> Nome {
    // A tabela de nomes tem no máximo 8 KiB: acima disso os endereços dão a volta.
    let rel = 2u16.wrapping_mul(linha.wrapping_mul(largura).wrapping_add(coluna)) & 0x1fff;
    let w = be16(vram, base.wrapping_add(rel));
    Nome {
        prioridade: w & 0x8000 != 0,
        paleta: u8::try_from((w >> 13) & 3).unwrap_or(0),
        v_flip: w & 0x1000 != 0,
        h_flip: w & 0x0800 != 0,
        padrao: w & 0x07ff,
    }
}

/// Uma linha de 8 pontos de um tile de 8 por 8 (4 bits por ponto).
fn linha_do_tile(vram: &[u8], padrao: u16, linha: u16, v_flip: bool, h_flip: bool) -> [u8; 8] {
    let l = if v_flip {
        7u16.saturating_sub(linha & 7)
    } else {
        linha & 7
    };
    let celula = 32u16.wrapping_mul(padrao);
    let base = usize::from(celula.wrapping_add(4u16.wrapping_mul(l)));
    let Some(b) = vram.get(base..base.saturating_add(4)) else {
        return [0; 8];
    };
    let mut c = [
        b[0] >> 4,
        b[0] & 15,
        b[1] >> 4,
        b[1] & 15,
        b[2] >> 4,
        b[2] & 15,
        b[3] >> 4,
        b[3] & 15,
    ];
    if h_flip {
        c.reverse();
    }
    c
}

/// Entradas do desenho dos planos de uma linha.
pub struct ParametrosDosPlanos<'a> {
    pub vram: &'a [u8],
    pub vsram: &'a [u16],
    pub reg: &'a Registros,
    pub linha: u16,
    pub rolagem_h: (u16, u16),
    pub v_tela_cheia: (u16, u16),
    /// Primeiro ponto a desenhar.
    pub ini: i32,
    /// Um depois do último ponto a desenhar.
    pub fim: i32,
}

/// Desenha os planos A e B da linha nos pontos `[ini, fim)` dos buffers.
pub fn desenhar_planos(
    p: &ParametrosDosPlanos<'_>,
    buf_a: &mut [PontoDeTile],
    buf_b: &mut [PontoDeTile],
) {
    let ParametrosDosPlanos {
        vram,
        vsram,
        reg,
        linha,
        rolagem_h,
        v_tela_cheia,
        ini,
        fim,
    } = *p;
    let (larg, alt) = (reg.largura_do_plano(), reg.altura_do_plano());
    let px = |t: TamanhoPlano| t.celulas().wrapping_mul(8);
    let (h_px, v_px) = match (larg, alt) {
        (TamanhoPlano::Invalido, _) => (256u16, 8u16),
        (_, TamanhoPlano::Invalido) => (px(larg), 256),
        _ => (px(larg), px(alt)),
    };
    let h_celulas = h_px / 8;
    let largura_nome: u16 = if larg == TamanhoPlano::Invalido {
        1
    } else {
        larg.celulas()
    };
    let mascara_v = v_px.wrapping_sub(1);
    for plano in [Plano::A, Plano::B] {
        let (base, h_scroll, buf) = match plano {
            Plano::A => (reg.nome_a(), rolagem_h.0, &mut *buf_a),
            Plano::B => (reg.nome_b(), rolagem_h.1, &mut *buf_b),
        };
        let grosso = h_scroll & !15;
        let fino = i32::from(h_scroll & 15);
        let primeira: i32 = if fino != 0 { -1 } else { 0 };
        let ultima_coluna = fim
            .saturating_add(15)
            .checked_div(16)
            .unwrap_or(COLUNAS)
            .max(COLUNAS);
        for coluna in primeira..ultima_coluna {
            let v = rolagem_v(vsram, reg, plano, coluna, v_tela_cheia);
            let linha_rolada = linha.wrapping_add(v) & mascara_v;
            let celula_v = linha_rolada >> 3;
            // 16 * coluna dá a volta em 16 bits quando a coluna é -1, como no oráculo.
            let pixel_rolado = u16::try_from((coluna.wrapping_mul(16)) & 0xffff)
                .unwrap_or(0)
                .wrapping_sub(grosso);
            let celula_h_base = pixel_rolado / 8;
            for desloc in 0..2i32 {
                let col_fb = coluna
                    .wrapping_mul(16)
                    .wrapping_add(desloc.wrapping_mul(8))
                    .wrapping_add(fino);
                if col_fb.wrapping_add(8) <= ini || col_fb >= fim {
                    continue;
                }
                let celula_h = celula_h_base.wrapping_add(u16::try_from(desloc).unwrap_or(0))
                    & h_celulas.wrapping_sub(1);
                let nome = ler_nome(vram, base, largura_nome, celula_v, celula_h);
                let cores =
                    linha_do_tile(vram, nome.padrao, linha_rolada, nome.v_flip, nome.h_flip);
                let ponto = |cor: u8| PontoDeTile {
                    cor,
                    paleta: nome.paleta,
                    prioridade: nome.prioridade,
                };
                // Caminho comum: a célula inteira cabe no que vai ser desenhado.
                if col_fb >= ini.max(0) && col_fb.wrapping_add(8) <= fim {
                    let inicio = usize::try_from(col_fb).unwrap_or(0);
                    if let Some(dest) = buf.get_mut(inicio..inicio.saturating_add(8)) {
                        for (d, c) in dest.iter_mut().zip(cores.iter()) {
                            *d = ponto(*c);
                        }
                        continue;
                    }
                }
                for (k, cor) in cores.iter().enumerate() {
                    let x = col_fb.wrapping_add(i32::try_from(k).unwrap_or(0));
                    if x < ini || x >= fim {
                        continue;
                    }
                    if let Some(p) = usize::try_from(x).ok().and_then(|i| buf.get_mut(i)) {
                        *p = ponto(*cor);
                    }
                }
            }
        }
    }
}

// ---- Sprites ----

/// Os 4 primeiros bytes de cada sprite, que o oráculo guarda em cache a cada escrita na VRAM.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SpriteEmCache {
    pub v_pos: u16,
    pub larg: u8,
    pub alt: u8,
    pub elo: u8,
}

impl SpriteEmCache {
    /// Atualiza o byte `indice` (0 a 3) do sprite.
    pub fn atualizar(&mut self, indice: u32, valor: u8) {
        match indice & 3 {
            0 => self.v_pos = (self.v_pos & 0x00ff) | (u16::from(valor & 3) << 8),
            1 => self.v_pos = (self.v_pos & 0xff00) | u16::from(valor),
            2 => {
                self.larg = ((valor >> 2) & 3).wrapping_add(1);
                self.alt = (valor & 3).wrapping_add(1);
            }
            _ => self.elo = valor & 0x7f,
        }
    }
}

/// Sprite completo, com os 4 bytes finais lidos da VRAM.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sprite {
    pub padrao: u16,
    pub v_pos: u16,
    pub h_pos: u16,
    pub larg: u8,
    pub alt: u8,
    pub paleta: u8,
    pub v_flip: bool,
    pub h_flip: bool,
    pub prioridade: bool,
}

impl Sprite {
    pub fn montar(cache: SpriteEmCache, bytes: [u8; 4]) -> Self {
        Self {
            padrao: u16::from_be_bytes([bytes[0] & 7, bytes[1]]),
            v_pos: cache.v_pos,
            h_pos: u16::from_be_bytes([bytes[2] & 1, bytes[3]]),
            larg: cache.larg,
            alt: cache.alt,
            paleta: (bytes[0] >> 5) & 3,
            v_flip: bytes[0] & 0x10 != 0,
            h_flip: bytes[0] & 0x08 != 0,
            prioridade: bytes[0] & 0x80 != 0,
        }
    }
}

/// Fase 1: quais sprites tocam a `linha`. `limite_pular` reduz quantos sprites são varridos
/// (o oráculo faz isso quando a imagem ficou desligada durante o HBlank).
/// Devolve os índices e se a linha estourou o limite de 20.
pub fn varrer_sprites(
    travados: &[SpriteEmCache],
    linha: u16,
    limite_pular: u32,
    ids: &mut Vec<u8>,
) -> bool {
    ids.clear();
    let total: u32 = 80;
    let maximo = total.saturating_sub(limite_pular);
    let linha_sprite = INICIO_SPRITES.wrapping_add(linha) & 0x1ff;
    let mut estourou = false;
    let mut idx: usize = 0;
    for _ in 0..maximo {
        let Some(s) = travados.get(idx) else { break };
        let topo = s.v_pos & 0x1ff;
        let base = topo.wrapping_add(8u16.wrapping_mul(u16::from(s.alt)));
        if topo <= linha_sprite && linha_sprite < base {
            if ids.len() == 20 {
                estourou = true;
                break;
            }
            ids.push(u8::try_from(idx).unwrap_or(0));
        }
        idx = usize::from(s.elo);
        if idx == 0 || idx >= 80 {
            break;
        }
    }
    estourou
}

/// Fase 3: desenha os sprites já buscados da `linha`. `pular_tiles` reduz o limite de 40 tiles.
/// `estouro_anterior`: a linha anterior estourou o limite de pontos (afeta a máscara por x = 0).
///
/// Escreve em `pontos` (limpa antes) e devolve se a linha estourou o limite de 40 tiles.
pub fn desenhar_sprites(
    vram: &[u8],
    sprites: &[Sprite],
    linha: u16,
    pular_tiles: u16,
    estouro_anterior: bool,
    pontos: &mut [PontoDeTile],
) -> bool {
    pontos.fill(PontoDeTile::default());
    let max_tiles = 40u16.saturating_sub(pular_tiles);
    let linha_sprite = INICIO_SPRITES.wrapping_add(linha);
    let mut buscados: u16 = 0;
    let mut estouro = false;
    let mut achou_nao_zero = estouro_anterior;
    let mut mascara = false;
    let inicio = INICIO_SPRITES;
    let largura = pontos.len().max(1);
    let fim = INICIO_SPRITES.wrapping_add(u16::try_from(largura).unwrap_or(320));
    'fora: for s in sprites {
        if s.h_pos == 0 && achou_nao_zero {
            mascara = true;
        } else if s.h_pos != 0 {
            achou_nao_zero = true;
        }
        let alt = u16::from(s.alt);
        let larg = u16::from(s.larg);
        let linha_no_sprite = linha_sprite.wrapping_sub(s.v_pos & 0x1ff) & 0x1f;
        let linha_no_sprite = if s.v_flip {
            ((alt << 3).wrapping_sub(1)).wrapping_sub(linha_no_sprite) & 0x1f
        } else {
            linha_no_sprite
        };
        for h in 0..larg {
            if buscados == max_tiles {
                estouro = true;
                break 'fora;
            }
            let coluna = if s.h_flip {
                larg.wrapping_sub(1).wrapping_sub(h)
            } else {
                h
            };
            let offset = coluna.wrapping_mul(alt).wrapping_add(linha_no_sprite >> 3);
            let padrao = s.padrao.wrapping_add(offset);
            let linha_do_tile_ = linha_no_sprite & 7;
            buscados = buscados.wrapping_add(1);
            if mascara {
                continue;
            }
            let esq = s.h_pos.wrapping_add(h.wrapping_mul(8));
            let dir = esq.wrapping_add(8);
            if esq >= fim || dir <= inicio {
                continue;
            }
            let cores = linha_do_tile(vram, padrao, linha_do_tile_, false, s.h_flip);
            for (k, cor) in cores.iter().enumerate() {
                let h_pos = esq.wrapping_add(u16::try_from(k).unwrap_or(0));
                if h_pos < inicio || h_pos >= fim {
                    continue;
                }
                let x = usize::from(h_pos.wrapping_sub(inicio));
                if let Some(p) = pontos.get_mut(x)
                    && p.cor == 0
                {
                    *p = PontoDeTile {
                        cor: *cor,
                        paleta: s.paleta,
                        prioridade: s.prioridade,
                    };
                }
            }
        }
    }
    estouro
}

/// Escolhe o índice de cor de 6 bits do ponto, pela prioridade (sem sombra nem realce).
pub fn resolver_ponto(
    sprite: PontoDeTile,
    a: PontoDeTile,
    b: PontoDeTile,
    indice_do_fundo: u8,
) -> u8 {
    let idx = |p: PontoDeTile| (p.paleta << 4) | p.cor;
    let ordem: [PontoDeTile; 3] = match (sprite.prioridade, a.prioridade, b.prioridade) {
        (false, false, false) | (true, false, false) | (true, true, false) | (true, true, true) => {
            [sprite, a, b]
        }
        (false, true, false) => [a, sprite, b],
        (false, false, true) => [b, sprite, a],
        (true, false, true) => [sprite, b, a],
        (false, true, true) => [a, b, sprite],
    };
    for p in ordem {
        if p.cor != 0 {
            return idx(p);
        }
    }
    indice_do_fundo
}

#[cfg(test)]
mod testes {
    use super::*;

    fn p(cor: u8, paleta: u8, prio: bool) -> PontoDeTile {
        PontoDeTile {
            cor,
            paleta,
            prioridade: prio,
        }
    }

    #[test]
    fn prioridade_sem_flags_sprite_a_b_fundo() {
        let fundo = 0x05;
        assert_eq!(
            resolver_ponto(p(1, 0, false), p(2, 1, false), p(3, 2, false), fundo),
            0x01
        );
        assert_eq!(
            resolver_ponto(p(0, 0, false), p(2, 1, false), p(3, 2, false), fundo),
            0x12
        );
        assert_eq!(
            resolver_ponto(p(0, 0, false), p(0, 1, false), p(3, 2, false), fundo),
            0x23
        );
        assert_eq!(
            resolver_ponto(p(0, 0, false), p(0, 1, false), p(0, 2, false), fundo),
            fundo
        );
    }

    #[test]
    fn prioridade_do_plano_sobe_acima_do_sprite() {
        let fundo = 0;
        // A com prioridade vence o sprite sem prioridade.
        assert_eq!(
            resolver_ponto(p(1, 0, false), p(2, 1, true), p(3, 2, false), fundo),
            0x12
        );
        // B com prioridade vence o sprite e o A sem prioridade.
        assert_eq!(
            resolver_ponto(p(1, 0, false), p(2, 1, false), p(3, 2, true), fundo),
            0x23
        );
        // Os dois com prioridade: A, B, depois o sprite.
        assert_eq!(
            resolver_ponto(p(1, 0, false), p(2, 1, true), p(3, 2, true), fundo),
            0x12
        );
        // Sprite com prioridade vence tudo.
        assert_eq!(
            resolver_ponto(p(1, 0, true), p(2, 1, true), p(3, 2, true), fundo),
            0x01
        );
    }

    #[test]
    fn linha_do_tile_nibbles_e_flips() {
        let mut vram = vec![0u8; 65536];
        // Tile 1: linha 0 = 1 2 3 4 5 6 7 8.
        vram[32..36].copy_from_slice(&[0x12, 0x34, 0x56, 0x78]);
        assert_eq!(
            linha_do_tile(&vram, 1, 0, false, false),
            [1, 2, 3, 4, 5, 6, 7, 8]
        );
        assert_eq!(
            linha_do_tile(&vram, 1, 0, false, true),
            [8, 7, 6, 5, 4, 3, 2, 1]
        );
        // Com v_flip a linha 0 vem da linha 7 do tile (zeros).
        assert_eq!(linha_do_tile(&vram, 1, 0, true, false), [0; 8]);
        assert_eq!(
            linha_do_tile(&vram, 1, 7, true, false),
            [1, 2, 3, 4, 5, 6, 7, 8]
        );
    }

    #[test]
    fn cache_de_sprite_atualiza_por_byte() {
        let mut c = SpriteEmCache::default();
        c.atualizar(0, 0x01);
        c.atualizar(1, 0x23);
        c.atualizar(2, 0b0000_1110); // 4 de largura menos 1 = 3 -> larg 4? bits 3:2 = 3 -> 4
        c.atualizar(3, 0xff);
        assert_eq!(c.v_pos, 0x123);
        assert_eq!(c.larg, 4);
        assert_eq!(c.alt, 3);
        assert_eq!(c.elo, 0x7f);
    }

    #[test]
    fn varredura_respeita_o_limite_de_20_por_linha() {
        // 25 sprites encadeados, todos na mesma linha (topo 0x80, 1 tile de altura).
        let mut c = vec![SpriteEmCache::default(); 80];
        for (i, sprite) in c.iter_mut().enumerate().take(25) {
            *sprite = SpriteEmCache {
                v_pos: 0x80,
                larg: 1,
                alt: 1,
                elo: u8::try_from(i.saturating_add(1)).unwrap(),
            };
        }
        c[24].elo = 0;
        let mut ids = Vec::new();
        let estouro = varrer_sprites(&c, 0, 0, &mut ids);
        assert_eq!(ids.len(), 20);
        assert!(estouro);
        // Com 20 exatos nao estoura.
        c[19].elo = 0;
        let estouro = varrer_sprites(&c, 0, 0, &mut ids);
        assert_eq!(ids.len(), 20);
        assert!(!estouro);
    }

    #[test]
    fn varredura_so_pega_sprites_que_tocam_a_linha() {
        let mut c = vec![SpriteEmCache::default(); 80];
        c[0] = SpriteEmCache {
            v_pos: 0x80 + 8,
            larg: 1,
            alt: 2,
            elo: 0,
        }; // linhas 8 a 23
        let mut ids = Vec::new();
        varrer_sprites(&c, 7, 0, &mut ids);
        assert_eq!(ids.len(), 0);
        varrer_sprites(&c, 8, 0, &mut ids);
        assert_eq!(ids.len(), 1);
        varrer_sprites(&c, 23, 0, &mut ids);
        assert_eq!(ids.len(), 1);
        varrer_sprites(&c, 24, 0, &mut ids);
        assert_eq!(ids.len(), 0);
    }

    struct Resultado {
        pontos: Vec<PontoDeTile>,
        estouro_de_pontos: bool,
    }

    fn sprites_da_linha(vram: &[u8], s: &[Sprite], anterior: bool) -> Resultado {
        let mut pontos = vec![PontoDeTile::default(); LARGURA];
        let estouro_de_pontos = desenhar_sprites(vram, s, 0, 0, anterior, &mut pontos);
        Resultado {
            pontos,
            estouro_de_pontos,
        }
    }

    fn sprite_simples(h: u16) -> Sprite {
        Sprite {
            padrao: 1,
            v_pos: 0x80,
            h_pos: h,
            larg: 1,
            alt: 1,
            ..Sprite::default()
        }
    }

    #[test]
    fn sprite_aparece_na_posicao_certa() {
        let mut vram = vec![0u8; 65536];
        vram[32..36].copy_from_slice(&[0x11, 0x11, 0x11, 0x11]);
        let r = sprites_da_linha(&vram, &[sprite_simples(0x80 + 10)], false);
        assert_eq!(r.pontos[9].cor, 0);
        assert_eq!(r.pontos[10].cor, 1);
        assert_eq!(r.pontos[17].cor, 1);
        assert_eq!(r.pontos[18].cor, 0);
    }

    #[test]
    fn mascara_por_x_zero_precisa_de_sprite_anterior_com_x_diferente_de_zero() {
        let mut vram = vec![0u8; 65536];
        vram[32..36].copy_from_slice(&[0x11, 0x11, 0x11, 0x11]);
        vram[64..68].copy_from_slice(&[0x22, 0x22, 0x22, 0x22]);
        let mut mascara = sprite_simples(0);
        mascara.padrao = 1;
        let mut atras = sprite_simples(0x80 + 40);
        atras.padrao = 2;
        // Sem sprite anterior com x diferente de zero: a máscara não vale e o sprite aparece.
        let r = sprites_da_linha(&vram, &[mascara, atras], false);
        assert_eq!(r.pontos[40].cor, 2);
        // Com um sprite anterior fora do zero, o x = 0 mascara os seguintes.
        let antes = sprite_simples(0x80 + 100);
        let r = sprites_da_linha(&vram, &[antes, mascara, atras], false);
        assert_eq!(r.pontos[100].cor, 1);
        assert_eq!(r.pontos[40].cor, 0);
        // Estouro de pontos na linha anterior troca o requisito.
        let r = sprites_da_linha(&vram, &[mascara, atras], true);
        assert_eq!(r.pontos[40].cor, 0);
    }

    #[test]
    fn limite_de_40_tiles_por_linha() {
        let mut vram = vec![0u8; 65536];
        vram[32..36].copy_from_slice(&[0x11, 0x11, 0x11, 0x11]);
        // 11 sprites de 4 tiles de largura = 44 tiles.
        let s: Vec<Sprite> = (0..11u16)
            .map(|i| Sprite {
                padrao: 1,
                v_pos: 0x80,
                h_pos: 0x80 + i * 8,
                larg: 4,
                alt: 1,
                ..Sprite::default()
            })
            .collect();
        let r = sprites_da_linha(&vram, &s, false);
        assert!(r.estouro_de_pontos);
        let r = sprites_da_linha(&vram, &s[..10], false);
        assert!(!r.estouro_de_pontos);
    }

    #[test]
    fn planos_com_rolagem_e_tabela_simples() {
        let mut vram = vec![0u8; 65536];
        // Plano A em 0xC000: célula (0,0) usa o tile 1 (cor 3 na linha 0), paleta 2.
        vram[32..36].copy_from_slice(&[0x33, 0x33, 0x33, 0x33]);
        vram[0xc000..0xc002].copy_from_slice(&[0x40, 0x01]); // paleta 2, tile 1
        let mut reg = Registros::novo();
        reg.brutos[2] = 0x30; // A em 0xC000
        reg.brutos[4] = 0x07; // B em 0xE000
        reg.brutos[16] = 0x01; // 64 x 32
        let mut a = vec![PontoDeTile::default(); LARGURA];
        let mut b = vec![PontoDeTile::default(); LARGURA];
        let vsram = vec![0u16; 40];
        let par = ParametrosDosPlanos {
            vram: &vram,
            vsram: &vsram,
            reg: &reg,
            linha: 0,
            rolagem_h: (0, 0),
            v_tela_cheia: (0, 0),
            ini: 0,
            fim: 320,
        };
        desenhar_planos(&par, &mut a, &mut b);
        assert_eq!(a[0], p(3, 2, false));
        assert_eq!(a[7], p(3, 2, false));
        assert_eq!(a[8].cor, 0);
        assert_eq!(b[0].cor, 0);
        // Rolagem horizontal de 8 pontos para a direita: a célula passa a começar no ponto 8.
        let mut a = vec![PontoDeTile::default(); LARGURA];
        let par = ParametrosDosPlanos {
            rolagem_h: (8, 0),
            ..par
        };
        desenhar_planos(&par, &mut a, &mut b);
        assert_eq!(a[8].cor, 3);
        assert_eq!(a[0].cor, 0);
    }
}
