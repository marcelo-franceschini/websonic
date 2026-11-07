//! O renderizador: a VDP com tempo por linha e o desenho do quadro (sub-etapas 4.3 a 4.6).
//!
//! Recebe as escritas nas portas com a linha e a coluna (ciclos mestres desde o início da linha)
//! em que o oráculo as aplicou. Entre uma escrita e a próxima, dispara os eventos de cada linha
//! na ordem da agenda do oráculo (`tempo::AGENDA`), e desenha cada linha no seu evento.
//! Referência de comportamento: jgenesis `386db855`, `vdp.rs` e `vdp/render.rs`.

use crate::camadas::{
    LARGURA, ParametrosDosPlanos, PontoDeTile, Sprite, SpriteEmCache, desenhar_planos,
    desenhar_sprites, ler_rolagem_h, resolver_ponto, varrer_sprites,
};
use crate::cor::da_palavra_cram;
use crate::portas::{Efeitos, PontoDaCram, Vdp};
use crate::quadro::Quadro;
use crate::registros::Registros;
use crate::tempo::{
    AGENDA, Evento, FIM_ATIVO, INICIO_ATIVO, LINHA_DO_FIM_DO_QUADRO, LINHAS_POR_QUADRO,
    PONTOS_POR_LINHA, mclk_para_ponto, ponto_para_h,
};

/// Primeiro ponto em que o oráculo aceita um ponto da CRAM (início ativo menos a borda esquerda
/// mais 5) e o fim (fim ativo mais a borda direita mais 5), em 40 colunas.
const INICIO_PONTOS_CRAM: u16 = 18;
const FIM_PONTOS_CRAM: u16 = 365;
const PONTOS_DA_CRAM: usize = 347;
/// Borda esquerda em 40 colunas.
const BORDA_ESQUERDA: i32 = 13;

/// Linhas ativas (28 linhas de células).
const LINHAS_ATIVAS: u16 = 224;

/// Uma escrita numa porta da VDP.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Escrita {
    Controle(u16),
    Dados { valor: u16, tamanho: u8 },
}

/// Estado dos sprites que depende do tempo em que a imagem ficou desligada no HBlank.
#[derive(Clone, Debug, Default)]
struct EstadoDeSprites {
    estouro_de_pontos_anterior: bool,
    pontos_desligados: u16,
    imagem_ligada: bool,
    ponto_da_imagem: u16,
}

impl EstadoDeSprites {
    fn somar_desligado(&mut self, ponto: u16) {
        let dif = if ponto >= self.ponto_da_imagem {
            ponto.saturating_sub(self.ponto_da_imagem)
        } else {
            ponto
                .saturating_add(PONTOS_POR_LINHA)
                .saturating_sub(self.ponto_da_imagem)
        };
        self.pontos_desligados = self.pontos_desligados.saturating_add(dif);
    }

    fn inicio_do_hblank(&mut self, imagem_ligada: bool) {
        self.pontos_desligados = 0;
        self.imagem_ligada = imagem_ligada;
        self.ponto_da_imagem = FIM_ATIVO.saturating_sub(16);
    }

    fn escrita_do_registrador_1(&mut self, imagem_ligada: bool, ponto: u16) {
        if (INICIO_ATIVO.saturating_sub(16)..FIM_ATIVO.saturating_sub(16)).contains(&ponto) {
            // Escrita antes do HBlank, valendo para a próxima linha: ignorada.
            return;
        }
        if !self.imagem_ligada {
            self.somar_desligado(ponto);
        }
        self.imagem_ligada = imagem_ligada;
        self.ponto_da_imagem = ponto;
    }

    fn fim_da_linha(&mut self, ponto: u16) {
        if !self.imagem_ligada {
            self.somar_desligado(ponto);
        }
    }
}

/// Renderizador completo.
pub struct Renderizador {
    vdp: Vdp,
    quadro: Quadro,
    // Tempo.
    linha: u16,
    ponto: u16,
    proximo_evento: usize,
    // Registradores travados e rolagens.
    travados: Registros,
    rolagem_h: (u16, u16),
    v_tela_cheia: (u16, u16),
    // Sprites.
    cache: Vec<SpriteEmCache>,
    cache_travado: Vec<SpriteEmCache>,
    varridos: Vec<u8>,
    sprites: Vec<Sprite>,
    pontos_de_sprite: Vec<PontoDeTile>,
    estado_sprites: EstadoDeSprites,
    // Pontos coloridos da CRAM: da linha atual e da anterior (índice 0 = ponto 18).
    pontos_atual: Vec<Option<u16>>,
    pontos_anterior: Vec<Option<u16>>,
    // Planos.
    buf_a: Vec<PontoDeTile>,
    buf_b: Vec<PontoDeTile>,
}

impl Default for Renderizador {
    fn default() -> Self {
        Self::novo()
    }
}

impl Renderizador {
    pub fn novo() -> Self {
        Self {
            vdp: Vdp::nova(),
            quadro: Quadro::novo(),
            linha: 0,
            ponto: 0,
            proximo_evento: 0,
            travados: Registros::novo(),
            rolagem_h: (0, 0),
            v_tela_cheia: (0, 0),
            cache: vec![SpriteEmCache::default(); 80],
            cache_travado: vec![SpriteEmCache::default(); 80],
            varridos: Vec::new(),
            sprites: Vec::new(),
            pontos_de_sprite: vec![PontoDeTile::default(); LARGURA],
            estado_sprites: EstadoDeSprites::default(),
            pontos_atual: vec![None; PONTOS_DA_CRAM],
            pontos_anterior: vec![None; PONTOS_DA_CRAM],
            buf_a: vec![PontoDeTile::default(); LARGURA],
            buf_b: vec![PontoDeTile::default(); LARGURA],
        }
    }

    pub fn vdp(&self) -> &Vdp {
        &self.vdp
    }

    pub fn quadro(&self) -> &Quadro {
        &self.quadro
    }

    /// Posição atual (linha, ponto).
    pub fn posicao(&self) -> (u16, u16) {
        (self.linha, self.ponto)
    }

    /// Entrega as palavras de DMA do quadro que vai começar.
    pub fn entregar_dma(&mut self, palavras: &[u16]) {
        self.vdp.entregar_dma(palavras);
    }

    /// Deixa a FIFO e o DMA correrem até acabar, sem andar o tempo. Só para cenas estáticas de
    /// teste: no oráculo a FIFO esvazia nos slots, e é isso que `avancar_ate` reproduz.
    pub fn drenar_vdp(&mut self) {
        self.vdp.drenar();
        self.drenar_cache_de_sprites();
    }

    /// Avança o tempo até (`linha`, `coluna`) e aplica a escrita.
    pub fn aplicar(&mut self, linha: u16, coluna: u64, escrita: Escrita) {
        self.avancar_ate(linha, coluna);
        let antes = self.vdp.registros.brutos;
        match escrita {
            Escrita::Controle(v) => self.vdp.escrever_controle(v),
            Escrita::Dados { valor, tamanho } => self.vdp.escrever_dados(valor, tamanho),
        }
        let mudou = self
            .vdp
            .registros
            .brutos
            .iter()
            .zip(antes.iter())
            .position(|(a, b)| a != b);
        self.depois_da_escrita(mudou);
    }

    /// Termina o quadro do oráculo (linha 232) e devolve a imagem.
    pub fn fechar_quadro(&mut self) -> &Quadro {
        // Se o quadro anterior acabou de fechar neste mesmo ponto, um quadro inteiro se passou.
        if self.linha == LINHA_DO_FIM_DO_QUADRO && self.ponto == 0 {
            self.terminar_linha();
        }
        self.avancar_ate(LINHA_DO_FIM_DO_QUADRO, 0);
        self.vdp.conferir_dma_pendente();
        &self.quadro
    }

    // ---- Tempo ----

    /// Avança até a linha e a coluna dadas, disparando os eventos que passam.
    pub fn avancar_ate(&mut self, linha: u16, coluna: u64) {
        let alvo = mclk_para_ponto(coluna);
        // No máximo um quadro de linhas por chamada (protege contra entrada fora de ordem).
        let mut passos = 0u16;
        while self.linha != linha && passos <= LINHAS_POR_QUADRO {
            self.terminar_linha();
            passos = passos.saturating_add(1);
        }
        self.avancar_pontos(alvo);
    }

    fn terminar_linha(&mut self) {
        self.avancar_pontos(PONTOS_POR_LINHA);
        self.trocar_pontos_da_cram();
        self.linha = (self.linha.saturating_add(1)) % LINHAS_POR_QUADRO;
        self.ponto = 0;
        self.proximo_evento = 0;
    }

    /// Avança os pontos até `fim`. Com a FIFO ou o DMA ativos o tempo anda slot a slot.
    fn avancar_pontos(&mut self, fim: u16) {
        if fim <= self.ponto {
            return;
        }
        if self.vdp.precisa_de_slots() {
            self.avancar_devagar(fim);
        } else {
            self.avancar_rapido(fim);
        }
    }

    /// Dispara os eventos com H interno menor que o do ponto `fim` (sem FIFO nem DMA).
    fn avancar_rapido(&mut self, fim: u16) {
        let h_fim = ponto_para_h(fim.saturating_sub(1));
        while let Some(e) = AGENDA.get(self.proximo_evento).copied() {
            if e.h > h_fim {
                break;
            }
            self.proximo_evento = self.proximo_evento.saturating_add(1);
            self.ponto = self.ponto.max(e.h.min(PONTOS_POR_LINHA));
            self.disparar(e.evento);
        }
        self.ponto = fim;
    }

    /// Um ponto por vez: processa o slot (FIFO e DMA) e depois os eventos da agenda.
    fn avancar_devagar(&mut self, fim: u16) {
        let mut p = self.ponto;
        while p < fim {
            self.ponto = p;
            if p.is_multiple_of(2) {
                let ef = self.vdp.passo_de_slot(usize::from(p / 2));
                self.efeitos_do_slot(ef);
            }
            let h = ponto_para_h(p);
            while let Some(e) = AGENDA.get(self.proximo_evento).copied() {
                if h < e.h {
                    break;
                }
                self.proximo_evento = self.proximo_evento.saturating_add(1);
                self.disparar(e.evento);
            }
            p = p.saturating_add(1);
        }
        self.ponto = fim;
    }

    /// O que a saída de uma entrada da FIFO pede ao renderizador.
    fn efeitos_do_slot(&mut self, ef: Efeitos) {
        self.drenar_cache_de_sprites();
        if ef.imagem_habilitada {
            if let Some(t) = self.travados.brutos.get_mut(1) {
                *t |= 0x40;
            }
            self.estado_sprites
                .escrita_do_registrador_1(true, self.ponto);
        }
        if let Some(p) = ef.ponto_da_cram {
            self.marcar_ponto_da_cram(p);
        }
        let dois = self.travados.rolagem_v() == crate::registros::RolagemV::DuasCelulas;
        if ef.imagem_habilitada || ef.cram || (ef.vsram && dois) {
            self.redesenhar_a_partir_de(self.ponto);
        }
    }

    fn drenar_cache_de_sprites(&mut self) {
        let gravados = std::mem::take(&mut self.vdp.ao_gravar_vram);
        for (endereco, valor) in gravados {
            self.atualizar_cache_de_sprites(endereco, valor);
        }
    }

    fn disparar(&mut self, evento: Evento) {
        match evento {
            Evento::InterrupcaoDeVBlank => {}
            Evento::LerRolagemH => {
                if !self.vdp.imagem_ligada() || self.linha >= LINHAS_ATIVAS {
                    return;
                }
                self.rolagem_h =
                    ler_rolagem_h(&self.vdp.vram, &self.vdp.registros, self.linha & 0xff);
            }
            Evento::RenderizarLinha => {
                self.estado_sprites.fim_da_linha(self.ponto);
                self.varrer_uma_linha_a_frente();
                self.desenhar_linha(self.linha, 0);
                self.aplicar_pontos_da_linha_anterior();
            }
            Evento::BuscarAtributosDeSprites => self.buscar_atributos(),
            Evento::InicioDoHBlank => {
                let ligada = self.vdp.imagem_ligada();
                self.estado_sprites.inicio_do_hblank(ligada);
                // O VBlank começa no HBlank da última linha ativa e acaba no da penúltima do quadro.
                if self.linha == LINHAS_ATIVAS.saturating_sub(1) {
                    self.vdp.em_vblank = true;
                } else if self.linha == LINHAS_POR_QUADRO.saturating_sub(2) {
                    self.vdp.em_vblank = false;
                }
            }
            Evento::InterrupcaoDeLinha => {
                self.cache_travado.clone_from(&self.cache);
            }
            Evento::TravarRegistradores => {
                self.travados = self.vdp.registros.clone();
                self.v_tela_cheia = (
                    self.vdp.vsram.first().copied().unwrap_or(0),
                    self.vdp.vsram.get(1).copied().unwrap_or(0),
                );
            }
        }
    }

    // ---- Efeitos de uma escrita ----

    fn depois_da_escrita(&mut self, registrador: Option<usize>) {
        self.drenar_cache_de_sprites();
        if let Some(reg) = registrador {
            self.registrador_mudou(reg);
        }
    }

    fn atualizar_cache_de_sprites(&mut self, endereco: u32, valor: u8) {
        if endereco & 4 != 0 {
            // Os 4 bytes finais de cada sprite não ficam em cache.
            return;
        }
        let tabela = self.vdp.registros.tabela_de_sprites();
        let fim = tabela.saturating_add(8 * 80);
        if endereco < tabela || endereco >= fim {
            return;
        }
        let idx = usize::try_from(endereco.saturating_sub(tabela) / 8).unwrap_or(usize::MAX);
        if let Some(s) = self.cache.get_mut(idx) {
            s.atualizar(endereco & 3, valor);
        }
    }

    /// Efeitos de escrever um registrador: travar alguns campos na hora e redesenhar a linha.
    fn registrador_mudou(&mut self, reg: usize) {
        let vivo = self.vdp.registros.brutos.get(reg).copied().unwrap_or(0);
        let mudou = match reg {
            1 => {
                let antes = self.travados.brutos.get(1).copied().unwrap_or(0);
                let ligada = if self.vdp.imagem_ligada() { 0x40 } else { 0 };
                let novo = (antes & !0x40) | ligada;
                if let Some(t) = self.travados.brutos.get_mut(1) {
                    *t = novo;
                }
                self.estado_sprites
                    .escrita_do_registrador_1(self.vdp.imagem_ligada(), self.ponto);
                novo != antes
            }
            2 | 3 | 4 | 7 => {
                let antes = self.travados.brutos.get(reg).copied().unwrap_or(0);
                if let Some(t) = self.travados.brutos.get_mut(reg) {
                    *t = vivo;
                }
                antes != vivo
            }
            _ => return,
        };
        if !mudou {
            return;
        }
        let inicio = INICIO_ATIVO.saturating_sub(16);
        if self.ponto < inicio {
            return;
        }
        let relativo = self.ponto.saturating_sub(inicio);
        let afetado = |plano_a: bool, rolagem: u16| -> u16 {
            let inicio_do_ciclo = (relativo / 16).saturating_mul(16);
            let primeiro = if plano_a || relativo % 16 >= 8 {
                inicio_do_ciclo.saturating_add(16)
            } else {
                inicio_do_ciclo
            };
            inicio
                .saturating_add(primeiro)
                .saturating_add(rolagem % 16)
                .saturating_add(16)
        };
        let ponto = match reg {
            2 | 3 => afetado(true, self.rolagem_h.0),
            4 => afetado(false, self.rolagem_h.1),
            _ => self.ponto,
        };
        self.redesenhar_a_partir_de(ponto);
    }

    /// Redesenha o resto da linha a partir de um ponto de vídeo (H interno da área ativa).
    fn redesenhar_a_partir_de(&mut self, ponto: u16) {
        if self.linha >= LINHAS_ATIVAS {
            return;
        }
        if !(INICIO_ATIVO..FIM_ATIVO).contains(&ponto) {
            return;
        }
        self.desenhar_linha(self.linha, ponto.saturating_sub(INICIO_ATIVO));
    }

    // ---- Pontos coloridos da CRAM ----

    fn marcar_ponto_da_cram(&mut self, p: PontoDaCram) {
        let ponto = self.ponto;
        self.pontos_no_intervalo(ponto, p.cor);
        // Truque do oráculo: com a imagem desligada e a cor de fundo mudando, os pontos
        // seguintes também recebem a cor nova.
        let (paleta, cor_id) = self.vdp.registros.fundo();
        let indice_fundo = (u32::from(paleta) << 4) | u32::from(cor_id);
        if !self.vdp.imagem_ligada() && p.indice == indice_fundo {
            let n = if p.tamanho_da_fifo > 1 {
                3
            } else {
                FIM_PONTOS_CRAM.saturating_sub(ponto)
            };
            for i in 1..=n {
                self.pontos_no_intervalo(ponto.saturating_add(i), p.cor);
            }
        }
    }

    fn pontos_no_intervalo(&mut self, ponto: u16, cor: u16) {
        if !(INICIO_PONTOS_CRAM..FIM_PONTOS_CRAM).contains(&ponto) {
            return;
        }
        let idx = usize::from(ponto.saturating_sub(INICIO_PONTOS_CRAM));
        if let Some(c) = self.pontos_atual.get_mut(idx) {
            *c = Some(cor);
        }
    }

    /// No fim de cada linha, os pontos dela passam a ser os da "linha anterior".
    fn trocar_pontos_da_cram(&mut self) {
        let algum = self.pontos_atual.iter().any(Option::is_some)
            || self.pontos_anterior.iter().any(Option::is_some);
        if !algum {
            return;
        }
        std::mem::swap(&mut self.pontos_atual, &mut self.pontos_anterior);
        self.pontos_atual.fill(None);
    }

    /// Desenha na imagem os pontos da linha anterior (como o oráculo, no início da linha atual).
    fn aplicar_pontos_da_linha_anterior(&mut self) {
        if self.linha == 0 || self.linha > LINHAS_ATIVAS {
            return;
        }
        let y = usize::from(self.linha.saturating_sub(1));
        for (idx, cor) in self.pontos_anterior.iter().enumerate() {
            let Some(cor) = cor else { continue };
            let x = i32::try_from(idx)
                .unwrap_or(0)
                .saturating_sub(BORDA_ESQUERDA);
            let Ok(x) = usize::try_from(x) else { continue };
            self.quadro.definir(x, y, da_palavra_cram(*cor));
        }
    }

    // ---- Sprites ----

    fn varrer_uma_linha_a_frente(&mut self) {
        let proxima = if self.linha >= LINHAS_POR_QUADRO.saturating_sub(1) {
            0
        } else {
            self.linha.saturating_add(1)
        };
        // Só varre as linhas ativas e a seguinte (o oráculo não varre a borda).
        if proxima > LINHAS_ATIVAS {
            return;
        }
        let pular = u32::from(self.estado_sprites.pontos_desligados);
        let (ids, estourou) = varrer_sprites(&self.cache_travado, proxima, pular);
        let _ = estourou;
        self.varridos = ids;
    }

    fn buscar_atributos(&mut self) {
        let tabela = self.vdp.registros.tabela_de_sprites() & 0xffff;
        self.sprites.clear();
        for id in &self.varridos {
            let base = usize::try_from(tabela)
                .unwrap_or(0)
                .saturating_add(8 * usize::from(*id));
            let ler = |d: usize| {
                self.vdp
                    .vram
                    .get(base.saturating_add(d))
                    .copied()
                    .unwrap_or(0)
            };
            let bytes = [ler(4), ler(5), ler(6), ler(7)];
            let cache = self
                .cache
                .get(usize::from(*id))
                .copied()
                .unwrap_or_default();
            self.sprites.push(Sprite::montar(cache, bytes));
        }
    }

    // ---- Desenho de uma linha ----

    fn cor_do_fundo(&self) -> u16 {
        let idx = usize::from(self.vdp.registros.indice_do_fundo());
        da_palavra_cram(self.vdp.cram.get(idx).copied().unwrap_or(0))
    }

    fn desenhar_linha(&mut self, linha: u16, ponto_inicial: u16) {
        if linha > LINHAS_ATIVAS {
            return;
        }
        // Não redesenha perto do fim da área ativa (o oráculo evita falhas visuais ali).
        if ponto_inicial >= 304 {
            return;
        }
        let y = usize::from(linha);
        if !self.vdp.imagem_ligada() {
            let fundo = self.cor_do_fundo();
            if linha < LINHAS_ATIVAS {
                for x in usize::from(ponto_inicial)..LARGURA {
                    self.quadro.definir(x, y, fundo);
                }
            }
            self.pontos_de_sprite.fill(PontoDeTile::default());
            return;
        }
        if ponto_inicial == 0 {
            let pular_tiles = self.estado_sprites.pontos_desligados.div_ceil(2);
            let r = desenhar_sprites(
                &self.vdp.vram,
                &self.sprites,
                linha,
                pular_tiles,
                self.estado_sprites.estouro_de_pontos_anterior,
            );
            self.pontos_de_sprite = r.pontos;
            self.estado_sprites.estouro_de_pontos_anterior = r.estouro_de_pontos;
        }
        if linha >= LINHAS_ATIVAS {
            return;
        }
        let par = ParametrosDosPlanos {
            vram: &self.vdp.vram,
            vsram: &self.vdp.vsram,
            reg: &self.travados,
            linha,
            rolagem_h: self.rolagem_h,
            v_tela_cheia: self.v_tela_cheia,
            ini: i32::from(ponto_inicial),
            fim: 320,
        };
        desenhar_planos(&par, &mut self.buf_a, &mut self.buf_b);
        let fundo = self.vdp.registros.indice_do_fundo();
        for x in usize::from(ponto_inicial)..LARGURA {
            let s = self.pontos_de_sprite.get(x).copied().unwrap_or_default();
            let a = self.buf_a.get(x).copied().unwrap_or_default();
            let b = self.buf_b.get(x).copied().unwrap_or_default();
            let idx = usize::from(resolver_ponto(s, a, b, fundo));
            let cor = da_palavra_cram(self.vdp.cram.get(idx).copied().unwrap_or(0));
            self.quadro.definir(x, y, cor);
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn controle(r: &mut Renderizador, linha: u16, w: u16) {
        r.aplicar(linha, 0, Escrita::Controle(w));
    }

    /// Mesmas escritas do programa de video sintetico do oraculo (sem DMA).
    fn fundo_azul() -> Renderizador {
        let mut r = Renderizador::novo();
        for w in [0x8F02u16, 0x8004, 0x8C81, 0x8700, 0x8174] {
            controle(&mut r, 0, w);
        }
        controle(&mut r, 0, 0xC000);
        controle(&mut r, 0, 0x0000);
        r.aplicar(
            0,
            0,
            Escrita::Dados {
                valor: 0x0E00,
                tamanho: 2,
            },
        );
        r
    }

    #[test]
    fn imagem_so_com_o_fundo() {
        let mut r = fundo_azul();
        r.avancar_ate(LINHA_DO_FIM_DO_QUADRO, 0);
        r.avancar_ate(LINHA_DO_FIM_DO_QUADRO.saturating_sub(1), 0);
        assert!(
            r.quadro().pontos().iter().all(|p| *p == 448),
            "fundo azul em todo o quadro"
        );
    }

    #[test]
    fn imagem_desligada_mostra_o_fundo_da_cor_zero() {
        let mut r = fundo_azul();
        controle(&mut r, 0, 0x8134); // imagem desligada
        r.avancar_ate(LINHA_DO_FIM_DO_QUADRO, 0);
        assert!(r.quadro().pontos().iter().all(|p| *p == 448));
    }

    #[test]
    fn mudanca_de_cram_no_meio_da_linha_redesenha_o_resto() {
        let mut r = fundo_azul();
        // Primeiro quadro completo para a CRAM e os registradores assentarem.
        r.avancar_ate(LINHA_DO_FIM_DO_QUADRO, 0);
        r.avancar_ate(10, 0);
        // Na linha 10, no ponto 100 (mclk 800), a cor 0 vira vermelho maximo.
        r.avancar_ate(10, 800);
        controle(&mut r, 10, 0xC000);
        controle(&mut r, 10, 0x0000);
        r.aplicar(
            10,
            800,
            Escrita::Dados {
                valor: 0x000E,
                tamanho: 2,
            },
        );
        r.avancar_ate(LINHA_DO_FIM_DO_QUADRO, 0);
        let q = r.quadro();
        // Antes do ponto 100 a linha 10 continua azul; do ponto 100 em diante fica vermelha.
        assert_eq!(q.ponto(0, 10), Some(448));
        assert_eq!(q.ponto(99 - 26, 10), Some(448));
        assert_eq!(q.ponto(319, 10), Some(7));
        // As linhas seguintes usam a cor nova inteira; as anteriores nao mudam.
        assert_eq!(q.ponto(0, 11), Some(7));
        assert_eq!(q.ponto(0, 9), Some(448));
    }

    /// Vetor tirado do oráculo com a ROM sintética de vídeo (escritas nas colunas reais do
    /// trace): na linha 0 do primeiro quadro, a cor 0 da CRAM é gravada no ponto 145. O oráculo
    /// mostra o azul a partir do ponto de vídeo 130, com um ponto da CRAM azul em x = 125
    /// e quatro pontos pretos entre os dois (x = 126 a 129).
    #[test]
    fn vetor_do_oraculo_para_cram_no_meio_da_linha() {
        let mut r = Renderizador::novo();
        let ev: [(u64, Escrita); 8] = [
            (252, Escrita::Controle(0x8F02)),
            (392, Escrita::Controle(0x8004)),
            (532, Escrita::Controle(0x8C81)),
            (672, Escrita::Controle(0x8700)),
            (812, Escrita::Controle(0x8174)),
            (1008, Escrita::Controle(0xC000)),
            (1008, Escrita::Controle(0x0000)),
            (
                1162,
                Escrita::Dados {
                    valor: 0x0E00,
                    tamanho: 2,
                },
            ),
        ];
        for (col, e) in ev {
            r.aplicar(0, col, e);
        }
        // O ponto da CRAM é desenhado quando a linha 1 começa a ser renderizada.
        r.avancar_ate(2, 0);
        let q = r.quadro();
        let linha: Vec<u16> = (0..320).map(|x| q.ponto(x, 0).unwrap_or(0)).collect();
        assert_eq!(linha[124], 0);
        assert_eq!(linha[125], 448, "ponto da CRAM");
        assert_eq!(&linha[126..130], &[0, 0, 0, 0]);
        assert_eq!(linha[130], 448);
        assert_eq!(linha[319], 448);
        assert_eq!(linha.iter().filter(|p| **p != 448).count(), 129);
    }
}
