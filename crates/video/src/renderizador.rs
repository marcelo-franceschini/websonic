//! O renderizador: a VDP com tempo por linha e o desenho do quadro (sub-etapas 4.3 a 4.6).
//!
//! Recebe as escritas nas portas com a linha e a coluna (ciclos mestres desde o início da linha)
//! em que o oráculo as aplicou. Entre uma escrita e a próxima, dispara os eventos de cada linha
//! na ordem da agenda do oráculo (`tempo::AGENDA`), e desenha cada linha no seu evento.
//! Referência de comportamento: jgenesis `386db855`, `vdp.rs` e `vdp/render.rs`.

use crate::camadas::{
    LARGURA, ParametrosDosPlanos, Sprite, SpriteEmCache, desenhar_planos_compacto,
    desenhar_sprites_compacto, ler_rolagem_h, resolver_ponto_compacto, resolver_ponto_sem_sprite,
    varrer_sprites_limite,
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
    /// O quadro acabou de ser fechado e nada andou desde então.
    fechado: bool,
    linha: u16,
    ponto: u16,
    proximo_evento: usize,
    // Registradores travados e rolagens.
    travados: Registros,
    rolagem_h: (u16, u16),
    v_tela_cheia: (u16, u16),
    // Sprites.
    cache: [SpriteEmCache; 80],
    cache_travado: [SpriteEmCache; 80],
    varridos: Vec<u8>,
    sprites: Vec<Sprite>,
    pontos_de_sprite: Vec<u8>,
    linha_tem_sprite: bool,
    estado_sprites: EstadoDeSprites,
    // Pontos coloridos da CRAM: da linha atual e da anterior (índice 0 = ponto 18).
    pontos_atual: Vec<Option<u16>>,
    pontos_anterior: Vec<Option<u16>>,
    // Há algum ponto em cada buffer? Evita varrer 347 entradas vazias duas ou três vezes por linha.
    marcados_atual: bool,
    marcados_anterior: bool,
    // Planos. `chave_planos` diz para qual estado os buffers valem (linha, gravações de VRAM e
    // versão dos registradores e rolagens), para a troca de CRAM não refazer os planos.
    chave_planos: Option<(u16, u64, u64)>,
    versao_planos: u64,
    buf_a: Vec<u8>,
    buf_b: Vec<u8>,
    /// Emulação dos pontos coloridos de CRAM (analógicos do hardware original).
    pontos_cram_ativos: bool,
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
            fechado: false,
            linha: 0,
            ponto: 0,
            proximo_evento: 0,
            travados: Registros::novo(),
            rolagem_h: (0, 0),
            v_tela_cheia: (0, 0),
            cache: [SpriteEmCache::default(); 80],
            cache_travado: [SpriteEmCache::default(); 80],
            varridos: Vec::with_capacity(40),
            sprites: Vec::with_capacity(40),
            pontos_de_sprite: vec![0; LARGURA],
            linha_tem_sprite: false,
            estado_sprites: EstadoDeSprites::default(),
            pontos_atual: vec![None; PONTOS_DA_CRAM],
            pontos_anterior: vec![None; PONTOS_DA_CRAM],
            marcados_atual: false,
            marcados_anterior: false,
            chave_planos: None,
            versao_planos: 0,
            buf_a: vec![0; LARGURA],
            buf_b: vec![0; LARGURA],
            pontos_cram_ativos: true,
        }
    }

    /// Define a largura da área visível (entre 320 e 640, par). 320 preserva fidelidade clássica.
    pub fn definir_largura(&mut self, largura: usize) {
        self.quadro.redimensionar(largura);
        let nova = self.quadro.largura();
        if self.pontos_de_sprite.len() != nova {
            self.pontos_de_sprite.resize(nova, 0);
            self.pontos_de_sprite.fill(0);
            self.buf_a.resize(nova, 0);
            self.buf_b.resize(nova, 0);
            self.linha_tem_sprite = false;
            self.chave_planos = None;
        }
    }

    /// Largura atual do quadro renderizado em pontos.
    pub fn largura(&self) -> usize {
        self.quadro.largura()
    }

    /// Liga ou desliga a emulação dos pontos de CRAM analógicos do hardware original.
    pub fn definir_pontos_cram(&mut self, ativos: bool) {
        self.pontos_cram_ativos = ativos;
    }

    /// Informa se a emulação dos pontos de CRAM analógicos está ativa.
    pub fn pontos_cram_ativos(&self) -> bool {
        self.pontos_cram_ativos
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
        self.fechado = false;
        self.avancar_ate(linha, coluna);
        let mudou = match escrita {
            Escrita::Controle(v) => self.vdp.escrever_controle_com_reg(v),
            Escrita::Dados { valor, tamanho } => {
                self.vdp.escrever_dados(valor, tamanho);
                None
            }
        };
        self.depois_da_escrita(mudou);
    }

    /// Termina o quadro do oráculo (linha 232, coluna 0) e devolve a imagem.
    pub fn fechar_quadro(&mut self) -> &Quadro {
        self.fechar_quadro_em(0)
    }

    /// Termina o quadro na coluna em que o oráculo o fechou (o fim da instrução que cruzou para a
    /// linha 232). O estado da VRAM e da CRAM do oráculo é o desse instante.
    pub fn fechar_quadro_em(&mut self, coluna: u64) -> &Quadro {
        // Se o quadro anterior acabou de fechar e nada andou, um quadro inteiro se passou.
        if self.fechado {
            self.terminar_linha();
        }
        self.avancar_ate(LINHA_DO_FIM_DO_QUADRO, coluna);
        self.vdp.conferir_dma_pendente();
        self.fechado = true;
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
        self.fechado = false;
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
            if !self.vdp.precisa_de_slots() {
                self.ponto = p;
                if p < fim {
                    self.avancar_rapido(fim);
                }
                return;
            }
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
        if ef.vsram {
            self.versao_planos = self.versao_planos.wrapping_add(1);
        }
        let dois = self.travados.rolagem_v() == crate::registros::RolagemV::DuasCelulas;
        if ef.imagem_habilitada || ef.cram || (ef.vsram && dois) {
            self.redesenhar_a_partir_de(self.ponto);
        }
    }

    fn drenar_cache_de_sprites(&mut self) {
        if self.vdp.ao_gravar_vram.is_empty() {
            return;
        }
        let mut gravados = std::mem::take(&mut self.vdp.ao_gravar_vram);
        for &(endereco, valor) in &gravados {
            self.atualizar_cache_de_sprites(endereco, valor);
        }
        gravados.clear();
        // Devolve a capacidade so se ninguem gravou no campo durante o laco.
        if self.vdp.ao_gravar_vram.is_empty() {
            self.vdp.ao_gravar_vram = gravados;
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
                self.cache_travado = self.cache;
            }
            Evento::TravarRegistradores => {
                self.travados = self.vdp.registros.clone();
                self.versao_planos = self.versao_planos.wrapping_add(1);
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
                self.versao_planos = self.versao_planos.wrapping_add(1);
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
            self.marcados_atual = true;
        }
    }

    /// No fim de cada linha, os pontos dela passam a ser os da "linha anterior".
    fn trocar_pontos_da_cram(&mut self) {
        if !self.marcados_atual && !self.marcados_anterior {
            return;
        }
        std::mem::swap(&mut self.pontos_atual, &mut self.pontos_anterior);
        std::mem::swap(&mut self.marcados_atual, &mut self.marcados_anterior);
        self.pontos_atual.fill(None);
        self.marcados_atual = false;
    }

    /// Desenha na imagem os pontos da linha anterior (como o oráculo, no início da linha atual).
    fn aplicar_pontos_da_linha_anterior(&mut self) {
        if !self.pontos_cram_ativos {
            return;
        }
        if self.linha == 0 || self.linha > LINHAS_ATIVAS {
            return;
        }
        if !self.marcados_anterior {
            return;
        }
        let y = usize::from(self.linha.saturating_sub(1));
        let margem = self.quadro.largura().saturating_sub(LARGURA) / 2;
        for (idx, cor) in self.pontos_anterior.iter().enumerate() {
            let Some(cor) = cor else { continue };
            let x = i32::try_from(idx)
                .unwrap_or(0)
                .saturating_sub(BORDA_ESQUERDA);
            let Ok(x) = usize::try_from(x) else { continue };
            self.quadro
                .definir(x.saturating_add(margem), y, da_palavra_cram(*cor));
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
        let max_sprites = if self.quadro.largura() > LARGURA {
            40
        } else {
            20
        };
        let estourou = varrer_sprites_limite(
            &self.cache_travado,
            proxima,
            pular,
            max_sprites,
            &mut self.varridos,
        );
        let _ = estourou;
    }

    fn buscar_atributos(&mut self) {
        self.sprites.clear();
        if self.varridos.is_empty() {
            return;
        }
        let tabela = usize::try_from(self.vdp.registros.tabela_de_sprites() & 0xffff).unwrap_or(0);
        for &id in &self.varridos {
            let idx = usize::from(id);
            let base = tabela.saturating_add(idx.saturating_mul(8));
            let ini = base.saturating_add(4);
            let bytes = self
                .vdp
                .vram
                .get(ini..ini.saturating_add(4))
                .and_then(|s| <[u8; 4]>::try_from(s).ok())
                .unwrap_or_else(|| {
                    let ler = |d: usize| {
                        self.vdp
                            .vram
                            .get(base.saturating_add(d))
                            .copied()
                            .unwrap_or(0)
                    };
                    [ler(4), ler(5), ler(6), ler(7)]
                });
            let cache = self.cache.get(idx).copied().unwrap_or_default();
            self.sprites.push(Sprite::montar(cache, bytes));
        }
    }

    // ---- Desenho de uma linha ----

    fn cor_do_fundo(&self) -> u16 {
        let idx = usize::from(self.vdp.registros.indice_do_fundo()) & 63;
        self.vdp.cram_9bits[idx]
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
        let larg = self.quadro.largura();
        let margem_usize = larg.saturating_sub(LARGURA) / 2;
        let margem = i32::try_from(margem_usize).unwrap_or(0);
        let inicio_buf = if ponto_inicial == 0 {
            0
        } else {
            margem_usize
                .saturating_add(usize::from(ponto_inicial))
                .min(larg)
        };
        let ini_tela = if ponto_inicial == 0 {
            margem.wrapping_neg()
        } else {
            i32::from(ponto_inicial)
        };
        let fim_tela = i32::try_from(LARGURA).unwrap_or(320).saturating_add(margem);
        if !self.vdp.imagem_ligada() {
            let fundo = self.cor_do_fundo();
            if linha < LINHAS_ATIVAS
                && let Some(fatia) = self
                    .quadro
                    .linha_mut(y)
                    .and_then(|l| l.get_mut(inicio_buf..larg))
            {
                fatia.fill(fundo);
            }
            if self.linha_tem_sprite {
                self.pontos_de_sprite.fill(0);
                self.linha_tem_sprite = false;
            }
            return;
        }
        if ponto_inicial == 0 {
            let pular_tiles = self.estado_sprites.pontos_desligados.div_ceil(2);
            let (estouro, tem_sprite) = desenhar_sprites_compacto(
                &self.vdp.tiles_8bpp,
                &self.sprites,
                linha,
                pular_tiles,
                crate::camadas::OpcoesSprites {
                    estouro_anterior: self.estado_sprites.estouro_de_pontos_anterior,
                    limpar_buffer: self.linha_tem_sprite,
                    margem_esq: margem,
                },
                &mut self.pontos_de_sprite,
            );
            self.estado_sprites.estouro_de_pontos_anterior = estouro;
            self.linha_tem_sprite = tem_sprite;
        }
        if linha >= LINHAS_ATIVAS {
            return;
        }
        let chave = (linha, self.vdp.gravacoes_de_vram, self.versao_planos);
        // Uma troca só de cor (CRAM) não muda os planos: basta compor de novo.
        if ponto_inicial == 0 || self.chave_planos != Some(chave) {
            let par = ParametrosDosPlanos {
                vram: &self.vdp.vram,
                vsram: &self.vdp.vsram,
                reg: &self.travados,
                linha,
                rolagem_h: self.rolagem_h,
                v_tela_cheia: self.v_tela_cheia,
                ini: ini_tela,
                fim: fim_tela,
            };
            desenhar_planos_compacto(
                &par,
                &self.vdp.tiles_8bpp,
                margem,
                &mut self.buf_a,
                &mut self.buf_b,
            );
            // Só o desenho da linha inteira deixa os buffers válidos para a linha toda.
            self.chave_planos = if ponto_inicial == 0 {
                Some(chave)
            } else {
                None
            };
        }
        let fundo = self.vdp.registros.indice_do_fundo();
        let paleta = &self.vdp.cram_9bits;
        if self.linha_tem_sprite {
            if let (Some(destinos), Some(spr), Some(pa), Some(pb)) = (
                self.quadro
                    .linha_mut(y)
                    .and_then(|l| l.get_mut(inicio_buf..larg)),
                self.pontos_de_sprite.get(inicio_buf..larg),
                self.buf_a.get(inicio_buf..larg),
                self.buf_b.get(inicio_buf..larg),
            ) {
                for (destino, ((&s, &a), &b)) in destinos
                    .iter_mut()
                    .zip(spr.iter().zip(pa.iter()).zip(pb.iter()))
                {
                    let idx = usize::from(resolver_ponto_compacto(s, a, b, fundo)) & 63;
                    *destino = paleta[idx];
                }
            }
        } else if let (Some(destinos), Some(pa), Some(pb)) = (
            self.quadro
                .linha_mut(y)
                .and_then(|l| l.get_mut(inicio_buf..larg)),
            self.buf_a.get(inicio_buf..larg),
            self.buf_b.get(inicio_buf..larg),
        ) {
            for (destino, (&a, &b)) in destinos.iter_mut().zip(pa.iter().zip(pb.iter())) {
                let idx = usize::from(resolver_ponto_sem_sprite(a, b, fundo)) & 63;
                *destino = paleta[idx];
            }
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

    #[test]
    fn saida_antecipada_de_avancar_devagar_preserva_eventos_da_linha() {
        let mut r_lote = fundo_azul();
        let mut r_passo = fundo_azul();
        r_lote.avancar_ate(LINHA_DO_FIM_DO_QUADRO, 0);
        r_passo.avancar_ate(LINHA_DO_FIM_DO_QUADRO, 0);

        // Escrita isolada no comeco da linha 12: FIFO drena cedo e o resto da linha usa caminho rapido.
        for r in [&mut r_lote, &mut r_passo] {
            r.avancar_ate(12, 40);
            controle(r, 12, 0xC000);
            controle(r, 12, 0x0000);
            r.aplicar(
                12,
                40,
                Escrita::Dados {
                    valor: 0x00E0,
                    tamanho: 2,
                },
            );
        }

        // Um avanca direto ate o fim do quadro; o outro avanca em passos curtos de 8 mclk.
        r_lote.avancar_ate(14, 0);
        for linha in 12u16..14 {
            let mut col = if linha == 12 { 48u64 } else { 0u64 };
            while col < 3420 {
                r_passo.avancar_ate(linha, col);
                col = col.saturating_add(8);
            }
        }
        r_passo.avancar_ate(14, 0);

        assert_eq!(r_lote.posicao(), r_passo.posicao());
        assert_eq!(r_lote.vdp().cram(), r_passo.vdp().cram());
        assert_eq!(r_lote.quadro().pontos(), r_passo.quadro().pontos());
    }
}
