//! Memórias da VDP, portas de controle e de dados e DMA (sub-etapa 4.2).
//!
//! Reproduz o comportamento do oráculo (jgenesis `386db855`, `vdp.rs`): `write_control`,
//! `write_data`, `pop_fifo` e os DMAs. Nesta camada as escritas valem na hora; o atraso da FIFO
//! entra na camada de tempo (4.6).

use crate::registros::{ModoDma, QUANTIDADE, Registros};
use std::collections::VecDeque;

pub const TAMANHO_VRAM: usize = 0x1_0000;
pub const PALAVRAS_CRAM: usize = 64;
pub const PALAVRAS_VSRAM: usize = 40;

/// Usos de recursos fora do inventário. Cada um conta quantas vezes aconteceu.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Avisos {
    pub janela: u32,
    pub vram128: u32,
    pub h32: u32,
    pub v30: u32,
    pub entrelacado: u32,
    pub sombra_realce: u32,
    pub dma_copia: u32,
    pub dma_preenchimento_fora_da_vram: u32,
    /// Quadros que acabaram com um DMA esperando palavras (acontece quando o DMA atravessa o
    /// fim do quadro; não é recurso fora do inventário e não entra no total).
    pub dma_sem_dados: u32,
    pub dados_em_bytes: u32,
    pub modo4: u32,
    pub coluna_esquerda: u32,
}

impl Avisos {
    /// Soma de todos os avisos.
    pub fn total(&self) -> u32 {
        [
            self.janela,
            self.vram128,
            self.h32,
            self.v30,
            self.entrelacado,
            self.sombra_realce,
            self.dma_copia,
            self.dma_preenchimento_fora_da_vram,
            self.dados_em_bytes,
            self.modo4,
            self.coluna_esquerda,
        ]
        .iter()
        .fold(0u32, |a, b| a.saturating_add(*b))
    }
}

/// Alvo do acesso pela porta de dados.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alvo {
    Vram,
    Vram8Bits,
    Cram,
    Vsram,
    Invalido,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sentido {
    Leitura,
    Escrita,
}

fn alvo_dos_bits(bits: u8, sentido: Sentido) -> Alvo {
    match (bits, sentido) {
        (0b000, _) => Alvo::Vram,
        (0b010, _) => Alvo::Vsram,
        (0b001, Sentido::Escrita) | (0b100, Sentido::Leitura) => Alvo::Cram,
        (0b110, Sentido::Leitura) => Alvo::Vram8Bits,
        _ => Alvo::Invalido,
    }
}

/// Uma escrita já decodificada, esperando na FIFO (a "entrada da FIFO" do oráculo).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntradaFifo {
    pub sentido: Sentido,
    pub alvo: Alvo,
    pub endereco: u32,
    pub palavra: u16,
    /// Slots que ainda faltam até a entrada poder sair da FIFO.
    pub latencia: u8,
    /// Escrita de VRAM em palavra: a primeira metade já saiu.
    pub segunda_metade: bool,
}

/// Latência inicial de uma entrada da FIFO, em slots (oráculo: `INITIAL_FIFO_LATENCY`).
const LATENCIA_INICIAL: u8 = 3;
/// Tamanho da FIFO.
const TAMANHO_FIFO: usize = 4;

/// O que um passo de slot mudou e o renderizador precisa saber.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Efeitos {
    /// A CRAM foi gravada: o renderizador redesenha o resto da linha e marca o "ponto da CRAM".
    pub cram: bool,
    /// Dados da escrita na CRAM, para o ponto colorido (cor, índice e tamanho da FIFO).
    pub ponto_da_cram: Option<PontoDaCram>,
    pub vsram: bool,
    /// A imagem acabou de ser ligada (a espera pela FIFO terminou).
    pub imagem_habilitada: bool,
}

/// Escrita na CRAM que pode virar um ponto colorido na imagem.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PontoDaCram {
    /// Índice da cor na CRAM (0 a 63).
    pub indice: u32,
    pub cor: u16,
    /// Tamanho da FIFO no momento da saída (a entrada ainda conta).
    pub tamanho_da_fifo: usize,
}

/// Escrita guardada enquanto o DMA de memória trava o barramento.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pendente {
    Controle(u16),
    Dados(u16),
}

/// Slots em que a VDP faz a atualização da memória (40 colunas).
fn eh_refresh(slot: usize) -> bool {
    matches!(slot, 26 | 58 | 90 | 122 | 154 | 204)
}

/// Slots de acesso externo durante a imagem ativa (40 colunas).
fn eh_acesso_externo(slot: usize) -> bool {
    matches!(
        slot,
        6 | 14
            | 22
            | 38
            | 46
            | 54
            | 70
            | 78
            | 86
            | 102
            | 110
            | 118
            | 134
            | 142
            | 150
            | 165
            | 166
            | 190
    )
}

/// Estado completo da VDP que as portas alteram.
#[derive(Clone, Debug)]
pub struct Vdp {
    pub registros: Registros,
    pub(crate) vram: Vec<u8>,
    pub(crate) cram: [u16; PALAVRAS_CRAM],
    pub(crate) vsram: [u16; PALAVRAS_VSRAM],
    pub avisos: Avisos,
    // Porta de controle.
    segunda_palavra: bool,
    sentido: Sentido,
    bits_do_alvo: u8,
    alvo: Alvo,
    endereco_de_controle: u32,
    endereco_de_dados: u32,
    // Imagem ligada de fato (o oráculo adia a ligação enquanto a FIFO não esvazia).
    imagem_ligada: bool,
    habilitacao_pendente: bool,
    /// O renderizador marca quando o VDP está em VBlank (muda o conjunto de slots de acesso).
    pub(crate) em_vblank: bool,
    // FIFO.
    fifo: VecDeque<EntradaFifo>,
    fifo_pendentes: VecDeque<EntradaFifo>,
    // DMA.
    dma_ativo: bool,
    dma_comprimento: u16,
    dma_latencia: u8,
    /// Palavras que o DMA de memória ainda pode ler, na ordem.
    fila_dma: VecDeque<u16>,
    preenchimento: Option<u16>,
    pendentes: Vec<Pendente>,
    pendentes_reserva: Vec<Pendente>,
    /// Quantas vezes a VRAM foi gravada (o renderizador usa para saber se os planos ainda valem).
    pub(crate) gravacoes_de_vram: u64,
    /// Bytes gravados na VRAM desde a última vez que o renderizador olhou (cache de sprites).
    pub(crate) ao_gravar_vram: Vec<(u32, u8)>,
}

impl Default for Vdp {
    fn default() -> Self {
        Self::nova()
    }
}

impl Vdp {
    pub fn nova() -> Self {
        Self {
            registros: Registros::novo(),
            vram: vec![0; TAMANHO_VRAM],
            cram: [0; PALAVRAS_CRAM],
            vsram: [0; PALAVRAS_VSRAM],
            avisos: Avisos::default(),
            segunda_palavra: false,
            sentido: Sentido::Leitura,
            bits_do_alvo: 0,
            alvo: Alvo::Vram,
            endereco_de_controle: 0,
            endereco_de_dados: 0,
            imagem_ligada: false,
            habilitacao_pendente: false,
            em_vblank: false,
            fifo: VecDeque::new(),
            fifo_pendentes: VecDeque::new(),
            dma_ativo: false,
            dma_comprimento: 0,
            dma_latencia: 0,
            fila_dma: VecDeque::new(),
            preenchimento: None,
            pendentes: Vec::new(),
            pendentes_reserva: Vec::new(),
            gravacoes_de_vram: 0,
            ao_gravar_vram: Vec::new(),
        }
    }

    pub fn vram(&self) -> &[u8] {
        &self.vram
    }

    pub fn cram(&self) -> &[u16] {
        &self.cram
    }

    pub fn vsram(&self) -> &[u16] {
        &self.vsram
    }

    /// Imagem ligada de fato neste instante.
    pub fn imagem_ligada(&self) -> bool {
        self.imagem_ligada
    }

    /// Quantidade de escritas efetivamente admitidas na FIFO da VDP.
    pub fn ocupacao_fifo(&self) -> usize {
        self.fifo.len()
    }

    /// O barramento da CPU aguarda o DMA de memória ou uma escrita pendente na FIFO.
    pub fn cpu_bloqueada(&self) -> bool {
        (self.dma_de_memoria() && !self.fila_dma.is_empty()) || !self.fifo_pendentes.is_empty()
    }

    /// A FIFO tem entradas ou há DMA em andamento: o tempo precisa andar slot a slot.
    pub fn precisa_de_slots(&self) -> bool {
        self.dma_ativo || !self.fifo.is_empty()
    }

    /// Entrega as palavras que o DMA de memória leu (vindas do `.s1v` ou do núcleo).
    pub fn entregar_dma(&mut self, palavras: &[u16]) {
        self.fila_dma.extend(palavras.iter().copied());
    }

    fn dma_de_memoria(&self) -> bool {
        self.dma_ativo && self.registros.modo_dma() == ModoDma::MemoriaParaVdp
    }

    // ---- Porta de controle ----

    pub fn escrever_controle(&mut self, valor: u16) {
        if self.dma_de_memoria() {
            // O DMA trava o barramento: a escrita espera o fim dele.
            self.pendentes.push(Pendente::Controle(valor));
            return;
        }
        if !self.segunda_palavra {
            // A primeira palavra sempre mexe no endereço, mesmo quando é escrita de registrador.
            self.endereco_de_controle =
                (self.endereco_de_controle & !0x3fff) | u32::from(valor & 0x3fff);
            self.endereco_de_dados = self.endereco_de_controle;
            self.sentido = if valor & 0x4000 != 0 {
                Sentido::Escrita
            } else {
                Sentido::Leitura
            };
            self.bits_do_alvo = (self.bits_do_alvo & !1) | u8::from(valor & 0x8000 != 0);
            self.alvo = alvo_dos_bits(self.bits_do_alvo, self.sentido);
            if valor & 0xc000 == 0x8000 {
                self.escrever_registrador(valor);
            } else {
                self.segunda_palavra = true;
            }
        } else {
            self.endereco_de_controle =
                (self.endereco_de_controle & 0x3fff) | (u32::from(valor & 7) << 14);
            self.endereco_de_dados = self.endereco_de_controle;
            let altos = u8::try_from((valor >> 3) & 0b110).unwrap_or(0);
            self.bits_do_alvo = (self.bits_do_alvo & 1) | altos;
            self.alvo = alvo_dos_bits(self.bits_do_alvo, self.sentido);
            self.segunda_palavra = false;
            // CD5 só vale com o DMA permitido no registrador 1.
            if self.registros.dma_permitido() {
                self.dma_ativo = valor & 0x80 != 0;
            }
            if self.dma_ativo {
                self.preenchimento = None;
                match self.registros.modo_dma() {
                    ModoDma::MemoriaParaVdp => {
                        self.dma_latencia = if self.alvo == Alvo::Vsram { 5 } else { 8 };
                    }
                    ModoDma::PreenchimentoDeVram => {}
                    ModoDma::CopiaDeVram => {
                        self.avisos.dma_copia = self.avisos.dma_copia.saturating_add(1);
                        self.dma_ativo = false;
                    }
                }
            }
        }
    }

    fn escrever_registrador(&mut self, valor: u16) {
        let reg = usize::from((valor >> 8) & 0x1f);
        if reg >= QUANTIDADE {
            return;
        }
        // Em modo 4 do Master System, registradores acima de 10 não mudam (como no oráculo).
        if self.registros.modo4 && reg > 10 {
            self.avisos.modo4 = self.avisos.modo4.saturating_add(1);
            return;
        }
        let byte = valor.to_le_bytes()[0];
        if let Some(r) = self.registros.brutos.get_mut(reg) {
            *r = byte;
        }
        match reg {
            1 => {
                // O bit 2 liga e desliga o modo 4 (comportamento do oráculo).
                self.registros.modo4 = byte & 4 == 0;
                let antes = self.imagem_ligada;
                self.imagem_ligada = byte & 0x40 != 0;
                if !antes && self.imagem_ligada && !self.fifo.is_empty() {
                    // Ligar a imagem com a FIFO cheia espera a FIFO esvaziar.
                    self.imagem_ligada = false;
                    self.habilitacao_pendente = true;
                } else {
                    self.habilitacao_pendente = false;
                }
            }
            19 => self.dma_comprimento = (self.dma_comprimento & 0xff00) | u16::from(byte),
            20 => self.dma_comprimento = (self.dma_comprimento & 0x00ff) | (u16::from(byte) << 8),
            _ => {}
        }
        self.notar_recursos_fora_do_inventario(reg);
    }

    fn notar_recursos_fora_do_inventario(&mut self, reg: usize) {
        let somar = |c: &mut u32| *c = c.saturating_add(1);
        match reg {
            1 => {
                if self.registros.vram128() {
                    somar(&mut self.avisos.vram128);
                }
                if self.registros.v30() {
                    somar(&mut self.avisos.v30);
                }
            }
            12 => {
                if !self.registros.h40() {
                    somar(&mut self.avisos.h32);
                }
                if self.registros.sombra_realce() {
                    somar(&mut self.avisos.sombra_realce);
                }
                if self.registros.entrelacado() {
                    somar(&mut self.avisos.entrelacado);
                }
            }
            17 | 18 if self.registros.janela_ativa() => somar(&mut self.avisos.janela),
            0 if self.registros.coluna_esquerda_apagada() => {
                somar(&mut self.avisos.coluna_esquerda)
            }
            _ => {}
        }
    }

    // ---- Porta de dados ----

    /// Escreve na porta de dados. `tamanho` 1 repete o byte nas duas metades (não visto no corpus).
    pub fn escrever_dados(&mut self, valor: u16, tamanho: u8) {
        let palavra = if tamanho == 1 {
            self.avisos.dados_em_bytes = self.avisos.dados_em_bytes.saturating_add(1);
            let b = valor.to_le_bytes()[0];
            u16::from_be_bytes([b, b])
        } else {
            valor
        };
        self.escrever_palavra_de_dados(palavra);
    }

    fn escrever_palavra_de_dados(&mut self, palavra: u16) {
        if self.dma_de_memoria() {
            self.pendentes.push(Pendente::Dados(palavra));
            return;
        }
        // Todo acesso à porta de dados zera a marca de "segunda palavra".
        self.segunda_palavra = false;
        if self.dma_ativo && self.registros.modo_dma() == ModoDma::PreenchimentoDeVram {
            self.executar_preenchimento_dma(palavra);
            return;
        }
        let entrada = EntradaFifo {
            sentido: self.sentido,
            alvo: self.alvo,
            endereco: self.endereco_de_dados,
            palavra,
            latencia: LATENCIA_INICIAL,
            segunda_metade: false,
        };
        self.endereco_de_dados = self
            .endereco_de_dados
            .wrapping_add(u32::from(self.registros.incremento()));
        if self.fifo.len() >= TAMANHO_FIFO {
            self.fifo_pendentes.push_back(entrada);
        } else {
            self.empurrar(entrada);
        }
    }

    fn esvaziar_fifo(&mut self) {
        let mut ef = Efeitos::default();
        while !self.fifo.is_empty() || !self.fifo_pendentes.is_empty() {
            if self.fifo.is_empty()
                && let Some(p) = self.fifo_pendentes.pop_front()
            {
                self.empurrar(p);
            }
            if let Some(f) = self.fifo.front_mut() {
                f.latencia = 0;
            }
            self.sair_da_fifo(&mut ef);
        }
    }

    fn executar_preenchimento_dma(&mut self, palavra: u16) {
        self.esvaziar_fifo();
        if self.alvo != Alvo::Vram && self.alvo != Alvo::Vram8Bits {
            self.avisos.dma_preenchimento_fora_da_vram =
                self.avisos.dma_preenchimento_fora_da_vram.saturating_add(1);
            self.dma_ativo = false;
            return;
        }
        let a = usize::from(u16::try_from(self.endereco_de_dados & 0xffff).unwrap_or(0));
        let [hi, lo] = palavra.to_be_bytes();
        self.gravar_byte_vram(a, hi);
        self.gravar_byte_vram(a ^ 1, lo);
        self.ao_gravar_vram.push((self.endereco_de_dados, hi));
        self.ao_gravar_vram.push((self.endereco_de_dados ^ 1, lo));
        self.endereco_de_dados = self
            .endereco_de_dados
            .wrapping_add(u32::from(self.registros.incremento()));

        let byte = hi;
        let mut restante = self.dma_comprimento;
        if restante == 0 {
            for _ in 0..0x1_0000 {
                let addr = usize::try_from((self.endereco_de_dados ^ 1) & 0xffff).unwrap_or(0);
                self.gravar_byte_vram(addr, byte);
                self.ao_gravar_vram
                    .push((u32::try_from(addr).unwrap_or(0), byte));
                self.endereco_de_dados = self
                    .endereco_de_dados
                    .wrapping_add(u32::from(self.registros.incremento()));
            }
        } else {
            while restante > 0 {
                let addr = usize::try_from((self.endereco_de_dados ^ 1) & 0xffff).unwrap_or(0);
                self.gravar_byte_vram(addr, byte);
                self.ao_gravar_vram
                    .push((u32::try_from(addr).unwrap_or(0), byte));
                self.endereco_de_dados = self
                    .endereco_de_dados
                    .wrapping_add(u32::from(self.registros.incremento()));
                restante = restante.wrapping_sub(1);
            }
        }
        self.dma_comprimento = 0;
        self.dma_ativo = false;
        self.preenchimento = None;
    }

    /// Põe na FIFO. O cache de sprites vê a escrita neste momento (como no oráculo).
    fn empurrar(&mut self, e: EntradaFifo) {
        if e.sentido == Sentido::Escrita && e.alvo == Alvo::Vram {
            let [hi, lo] = e.palavra.to_be_bytes();
            self.ao_gravar_vram.push((e.endereco, hi));
            self.ao_gravar_vram.push((e.endereco ^ 1, lo));
        }
        self.fifo.push_back(e);
    }

    fn gravar_byte_vram(&mut self, endereco: usize, valor: u8) {
        self.gravacoes_de_vram = self.gravacoes_de_vram.wrapping_add(1);
        if let Some(b) = self.vram.get_mut(endereco) {
            *b = valor;
        }
    }

    // ---- Passo de slot (o "advance_to_pixel" do oráculo, para a FIFO e o DMA) ----

    /// Processa um slot (um ponto par da linha). `slot` vai de 0 a 209.
    pub(crate) fn passo_de_slot(&mut self, slot: usize) -> Efeitos {
        let refresh = eh_refresh(slot);
        let mut ef = Efeitos::default();
        if !refresh {
            self.progredir_dma_de_memoria(slot);
        }
        let em_branco = !self.imagem_ligada || self.em_vblank;
        let acesso = if em_branco {
            !refresh
        } else {
            eh_acesso_externo(slot)
        };
        if acesso {
            self.slot_de_acesso(&mut ef);
        }
        if !refresh {
            for e in self.fifo.iter_mut() {
                e.latencia = e.latencia.saturating_sub(1);
            }
        }
        ef
    }

    fn slot_de_acesso(&mut self, ef: &mut Efeitos) {
        if let Some(frente) = self.fifo.front() {
            if frente.latencia == 0 {
                self.sair_da_fifo(ef);
            }
        } else if self.dma_ativo && self.registros.modo_dma() == ModoDma::PreenchimentoDeVram {
            self.progredir_preenchimento();
        }
    }

    fn progredir_dma_de_memoria(&mut self, slot: usize) {
        if !self.dma_de_memoria() {
            return;
        }
        self.dma_latencia = self.dma_latencia.saturating_sub(1);
        if self.fifo.len() >= TAMANHO_FIFO {
            return;
        }
        // Perde-se uma leitura a cada slot de atualização da memória.
        if slot != 0 && eh_refresh(slot.saturating_sub(1)) {
            return;
        }
        let Some(palavra) = self.fila_dma.pop_front() else {
            // Sem palavras: o DMA espera (as palavras do próximo quadro ainda não chegaram).
            return;
        };
        let entrada = EntradaFifo {
            sentido: self.sentido,
            alvo: self.alvo,
            endereco: self.endereco_de_dados,
            palavra,
            latencia: self.dma_latencia.max(LATENCIA_INICIAL),
            segunda_metade: false,
        };
        self.empurrar(entrada);
        self.endereco_de_dados = self
            .endereco_de_dados
            .wrapping_add(u32::from(self.registros.incremento()));
        self.decrementar_dma();
    }

    fn decrementar_dma(&mut self) {
        self.dma_comprimento = self.dma_comprimento.wrapping_sub(1);
        if self.dma_comprimento != 0 {
            return;
        }
        self.dma_ativo = false;
        // Aplica as escritas que esperaram o fim do DMA.
        self.pendentes_reserva.clear();
        std::mem::swap(&mut self.pendentes, &mut self.pendentes_reserva);
        let len = self.pendentes_reserva.len();
        for i in 0..len {
            let p = self.pendentes_reserva[i];
            match p {
                Pendente::Controle(v) => self.escrever_controle(v),
                Pendente::Dados(v) => self.escrever_palavra_de_dados(v),
            }
        }
        self.pendentes_reserva.clear();
    }

    fn progredir_preenchimento(&mut self) {
        let Some(dado) = self.preenchimento else {
            return;
        };
        if self.alvo != Alvo::Vram && self.alvo != Alvo::Vram8Bits {
            // O preenchimento da CRAM e da VSRAM usa a próxima entrada da FIFO (comportamento
            // com defeito). Fora do inventário: só avisa e encerra.
            self.avisos.dma_preenchimento_fora_da_vram =
                self.avisos.dma_preenchimento_fora_da_vram.saturating_add(1);
            self.dma_ativo = false;
            return;
        }
        let a = usize::try_from((self.endereco_de_dados ^ 1) & 0xffff).unwrap_or(0);
        let byte = dado.to_be_bytes()[0];
        self.gravar_byte_vram(a, byte);
        self.ao_gravar_vram
            .push((u32::try_from(a).unwrap_or(0), byte));
        self.endereco_de_dados = self
            .endereco_de_dados
            .wrapping_add(u32::from(self.registros.incremento()));
        self.decrementar_dma();
    }

    /// Grava a entrada da frente da FIFO na memória e a tira (VRAM em palavra leva dois slots).
    fn sair_da_fifo(&mut self, ef: &mut Efeitos) {
        let Some(frente) = self.fifo.front().copied() else {
            return;
        };
        if frente.sentido == Sentido::Escrita {
            match frente.alvo {
                Alvo::Vram => {
                    let a = usize::from(u16::try_from(frente.endereco & 0xffff).unwrap_or(0));
                    let [hi, lo] = frente.palavra.to_be_bytes();
                    self.gravar_byte_vram(a, hi);
                    self.gravar_byte_vram(a ^ 1, lo);
                }
                Alvo::Cram => {
                    let i = usize::try_from((frente.endereco & 0x7f) >> 1).unwrap_or(0);
                    if let Some(c) = self.cram.get_mut(i) {
                        *c = frente.palavra;
                    }
                    ef.cram = true;
                    ef.ponto_da_cram = Some(PontoDaCram {
                        indice: u32::try_from(i).unwrap_or(0),
                        cor: frente.palavra,
                        tamanho_da_fifo: self.fifo.len(),
                    });
                }
                Alvo::Vsram => {
                    let i = usize::try_from((frente.endereco & 0x7f) >> 1).unwrap_or(usize::MAX);
                    if let Some(v) = self.vsram.get_mut(i) {
                        *v = frente.palavra;
                        ef.vsram = true;
                    }
                }
                Alvo::Vram8Bits | Alvo::Invalido => {}
            }
        }
        // O preenchimento começa quando uma entrada sai da FIFO depois do comando.
        self.preenchimento = Some(frente.palavra);
        let duas_metades =
            matches!(frente.alvo, Alvo::Vram | Alvo::Invalido) && !frente.segunda_metade;
        if duas_metades {
            if let Some(f) = self.fifo.front_mut() {
                f.segunda_metade = true;
            }
        } else {
            self.fifo.pop_front();
            if self.fifo.len() < TAMANHO_FIFO
                && let Some(p) = self.fifo_pendentes.pop_front()
            {
                self.empurrar(p);
            }
        }
        if self.habilitacao_pendente && self.fifo.is_empty() {
            self.habilitacao_pendente = false;
            self.imagem_ligada = true;
            ef.imagem_habilitada = true;
        }
    }

    /// Fim do quadro de DMA: se ainda faltavam palavras, conta como aviso (sem dados).
    pub fn conferir_dma_pendente(&mut self) {
        if self.dma_de_memoria() && self.fila_dma.is_empty() {
            self.avisos.dma_sem_dados = self.avisos.dma_sem_dados.saturating_add(1);
        }
    }

    /// Deixa a FIFO e o DMA correrem até acabar, como num VBlank longo. Serve a testes e a
    /// cenas estáticas: não é o tempo do oráculo.
    pub fn drenar(&mut self) {
        let antes = self.em_vblank;
        self.em_vblank = true;
        for _ in 0..200_000 {
            if !self.precisa_de_slots() {
                break;
            }
            let mut sem_palavras = self.dma_de_memoria() && self.fila_dma.is_empty();
            for slot in 0..210 {
                let ef = self.passo_de_slot(slot);
                if ef.imagem_habilitada {
                    sem_palavras = false;
                }
            }
            if sem_palavras && self.fifo.is_empty() {
                break;
            }
        }
        self.em_vblank = antes;
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    /// Liga o DMA e o incremento 2, como o jogo faz.
    fn pronta() -> Vdp {
        let mut v = Vdp::nova();
        for w in [0x8174u16, 0x8F02, 0x8C81] {
            v.escrever_controle(w);
        }
        v
    }

    #[test]
    fn registrador_so_vale_com_o_padrao_8xxx() {
        let mut v = pronta();
        v.escrever_controle(0x8B03);
        assert_eq!(v.registros.brutos[11], 0x03);
        // 0xC0xx não é escrita de registrador.
        v.escrever_controle(0xC000);
        v.escrever_controle(0x0000);
        assert_eq!(v.registros.brutos[11], 0x03);
    }

    #[test]
    fn modo4_ignora_registradores_acima_de_10() {
        let mut v = Vdp::nova();
        // O modo 4 so liga quando o jogo escreve o registrador 1 sem o bit 2.
        v.escrever_controle(0x8C81);
        assert_eq!(v.registros.brutos[12], 0x81);
        assert_eq!(v.avisos.modo4, 0);
        let mut v = Vdp::nova();
        v.escrever_controle(0x8100);
        v.escrever_controle(0x8C81);
        assert_eq!(v.registros.brutos[12], 0);
        assert_eq!(v.avisos.modo4, 1);
        v.escrever_controle(0x8704);
        assert_eq!(v.registros.brutos[7], 4);
    }

    #[test]
    fn escrita_de_vram_por_palavra_e_incremento() {
        let mut v = pronta();
        v.escrever_controle(0x4000); // VRAM, endereco 0
        v.escrever_controle(0x0000);
        v.escrever_dados(0x1234, 2);
        v.escrever_dados(0xabcd, 2);
        // Ainda na FIFO: a memoria so muda quando os slots andam.
        assert_eq!(&v.vram()[0..4], &[0, 0, 0, 0]);
        v.drenar();
        assert_eq!(&v.vram()[0..4], &[0x12, 0x34, 0xab, 0xcd]);
    }

    #[test]
    fn palavra_em_endereco_impar_troca_os_bytes() {
        let mut v = pronta();
        v.escrever_controle(0x4001); // VRAM, endereco 1
        v.escrever_controle(0x0000);
        v.escrever_dados(0x1234, 2);
        v.drenar();
        assert_eq!(v.vram()[1], 0x12);
        assert_eq!(v.vram()[0], 0x34);
    }

    #[test]
    fn cram_e_vsram() {
        let mut v = pronta();
        v.escrever_controle(0xC004); // CRAM, endereco 4 (cor 2)
        v.escrever_controle(0x0000);
        v.escrever_dados(0x0EEE, 2);
        v.drenar();
        assert_eq!(v.cram()[2], 0x0EEE);
        v.escrever_controle(0x4002); // VSRAM, endereco 2
        v.escrever_controle(0x0010);
        v.escrever_dados(0x0155, 2);
        v.drenar();
        assert_eq!(v.vsram()[1], 0x0155);
        // A VSRAM tem 40 palavras: o indice 40 (endereco $50 depois da mascara $7F) nao grava nada.
        v.escrever_controle(0x4050);
        v.escrever_controle(0x0010);
        v.escrever_dados(0x0777, 2);
        v.drenar();
        assert!(v.vsram().iter().all(|x| *x != 0x0777));
    }

    #[test]
    fn leitura_nao_grava() {
        let mut v = pronta();
        v.escrever_controle(0x0000); // VRAM, leitura
        v.escrever_controle(0x0000);
        v.escrever_dados(0x1111, 2);
        v.drenar();
        assert!(v.vram().iter().all(|b| *b == 0));
    }

    #[test]
    fn dma_de_memoria_para_vram() {
        let mut v = pronta();
        for w in [0x9304u16, 0x9400, 0x9500, 0x9608, 0x9700] {
            v.escrever_controle(w);
        }
        v.escrever_controle(0x4000);
        v.escrever_controle(0x0082); // VRAM $8000 com DMA
        v.entregar_dma(&[0x1111, 0x2222, 0x3333, 0x4444]);
        v.drenar();
        assert_eq!(
            &v.vram()[0x8000..0x8008],
            &[0x11, 0x11, 0x22, 0x22, 0x33, 0x33, 0x44, 0x44]
        );
        assert_eq!(v.avisos.total(), 0);
    }

    #[test]
    fn dma_espera_as_palavras_e_continua() {
        let mut v = pronta();
        for w in [0x9304u16, 0x9400, 0x9500, 0x9608, 0x9700, 0x4000, 0x0082] {
            v.escrever_controle(w);
        }
        v.entregar_dma(&[0xaaaa, 0xbbbb]);
        v.drenar();
        assert_eq!(&v.vram()[0x8000..0x8004], &[0xaa, 0xaa, 0xbb, 0xbb]);
        assert_eq!(v.vram()[0x8004], 0);
        v.conferir_dma_pendente();
        assert_eq!(v.avisos.dma_sem_dados, 1);
        v.entregar_dma(&[0xcccc, 0xdddd]);
        v.drenar();
        assert_eq!(&v.vram()[0x8004..0x8008], &[0xcc, 0xcc, 0xdd, 0xdd]);
    }

    #[test]
    fn dma_sem_permissao_nao_dispara() {
        let mut v = Vdp::nova();
        for w in [0x8164u16, 0x8F02, 0x9304, 0x9400, 0x4000, 0x0082] {
            v.escrever_controle(w);
        }
        v.entregar_dma(&[0x1111, 0x2222, 0x3333, 0x4444]);
        v.drenar();
        assert!(v.vram().iter().all(|b| *b == 0));
    }

    #[test]
    fn preenchimento_de_vram() {
        let mut v = pronta();
        v.escrever_controle(0x8F01); // incremento 1
        for w in [0x9304u16, 0x9400, 0x9780] {
            v.escrever_controle(w);
        }
        v.escrever_controle(0x4000);
        v.escrever_controle(0x0080); // VRAM 0 com DMA (preenchimento)
        v.escrever_dados(0x5a00, 2);
        v.drenar();
        // A escrita normal grava a palavra em 0 e 1 e o endereco passa a 1. O preenchimento grava
        // o byte alto em (endereco ^ 1): 1^1=0, 2^1=3, 3^1=2, 4^1=5. O byte 1 mantem o valor
        // da escrita normal e o byte 4 nao e tocado (comportamento do oraculo com incremento 1).
        assert_eq!(&v.vram()[0..7], &[0x5a, 0x00, 0x5a, 0x5a, 0, 0x5a, 0]);
    }

    #[test]
    fn copia_de_vram_e_aviso() {
        let mut v = pronta();
        for w in [0x9304u16, 0x97c0, 0x4000, 0x0080] {
            v.escrever_controle(w);
        }
        assert_eq!(v.avisos.dma_copia, 1);
    }

    #[test]
    fn dados_em_byte_repetem_e_avisam() {
        let mut v = pronta();
        v.escrever_controle(0x4000);
        v.escrever_controle(0x0000);
        v.escrever_dados(0x0042, 1);
        v.drenar();
        assert_eq!(&v.vram()[0..2], &[0x42, 0x42]);
        assert_eq!(v.avisos.dados_em_bytes, 1);
    }

    #[test]
    fn registrador_mexe_no_endereco_da_primeira_palavra() {
        let mut v = pronta();
        v.escrever_controle(0x4000);
        v.escrever_controle(0x0000);
        // Uma escrita de registrador depois: o modo vira leitura (bit 14 zerado).
        v.escrever_controle(0x8F02);
        v.escrever_dados(0xffff, 2);
        v.drenar();
        assert!(v.vram().iter().all(|b| *b == 0));
    }

    #[test]
    fn a_fifo_atrasa_a_gravacao_e_respeita_a_ordem() {
        // Em imagem ativa so os slots externos servem a FIFO: a escrita demora a chegar.
        let mut v = pronta();
        v.escrever_controle(0xC000);
        v.escrever_controle(0x0000);
        v.escrever_dados(0x0111, 2);
        v.escrever_dados(0x0222, 2);
        // Slots 0 a 5 (antes do primeiro slot externo, o 6): nada saiu.
        for slot in 0..6 {
            v.passo_de_slot(slot);
        }
        assert_eq!(v.cram()[0], 0);
        // O slot 6 ainda esta com latencia: a entrada precisa de 3 slots pares antes de sair.
        v.passo_de_slot(6);
        v.passo_de_slot(7);
        let mut saiu_no = None;
        for slot in 8..60 {
            let ef = v.passo_de_slot(slot);
            if ef.cram && saiu_no.is_none() {
                saiu_no = Some(slot);
            }
        }
        // A primeira cor saiu no slot externo 6 (nos passos de 6 e 7 acima); a segunda sai no
        // proximo slot externo, o 14.
        assert_eq!(saiu_no, Some(14));
        assert_eq!(v.cram()[0], 0x0111);
        assert_eq!(v.cram()[1], 0x0222);
    }

    #[test]
    fn recursos_fora_do_inventario_viram_aviso() {
        // Modo 5 ligado primeiro (senao os registradores acima de 10 sao ignorados).
        let mut v = Vdp::nova();
        v.escrever_controle(0x8104);
        let antes = v.avisos.total();
        assert_eq!(antes, 0);
        v.escrever_controle(0x8C00); // 32 colunas
        assert_eq!(v.avisos.h32, 1);
        v.escrever_controle(0x8C89); // 40 colunas com sombra e realce
        assert_eq!(v.avisos.sombra_realce, 1);
        v.escrever_controle(0x8C87); // entrelacado
        assert_eq!(v.avisos.entrelacado, 1);
        v.escrever_controle(0x910F); // janela cobrindo area
        assert_eq!(v.avisos.janela, 1);
        v.escrever_controle(0x810C); // 30 linhas
        assert_eq!(v.avisos.v30, 1);
        v.escrever_controle(0x8184); // VRAM de 128 KiB
        assert_eq!(v.avisos.vram128, 1);
        v.escrever_controle(0x8024); // coluna da esquerda apagada
        assert_eq!(v.avisos.coluna_esquerda, 1);
    }

    #[test]
    fn valores_do_corpus_nao_geram_aviso() {
        let mut v = Vdp::nova();
        // Sequencia de inicializacao vista no corpus: modo 5, 40 colunas, janela zerada.
        for w in [
            0x8004u16, 0x8174, 0x8C81, 0x8B03, 0x9100, 0x9200, 0x8230, 0x8407,
        ] {
            v.escrever_controle(w);
        }
        assert_eq!(v.avisos.total(), 0);
    }

    #[test]
    fn ligar_a_imagem_com_a_fifo_cheia_espera_a_fifo_esvaziar() {
        let mut v = Vdp::nova();
        v.escrever_controle(0x8004);
        v.escrever_controle(0x8F02);
        v.escrever_controle(0xC000);
        v.escrever_controle(0x0000);
        v.escrever_dados(0x0111, 2);
        v.escrever_controle(0x8174);
        assert!(!v.imagem_ligada());
        v.em_vblank = true;
        let mut ligou = false;
        for slot in 0..30 {
            if v.passo_de_slot(slot).imagem_habilitada {
                ligou = true;
            }
        }
        assert!(ligou);
        assert!(v.imagem_ligada());
    }
}
