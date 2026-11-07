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
    pub dma_sem_dados: u32,
    pub dados_em_bytes: u32,
    pub modo4: u32,
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
            self.dma_sem_dados,
            self.dados_em_bytes,
            self.modo4,
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

/// Uma escrita já decodificada, pronta para ir à memória (a "entrada da FIFO" do oráculo).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntradaFifo {
    pub sentido: Sentido,
    pub alvo: Alvo,
    pub endereco: u32,
    pub palavra: u16,
}

/// DMA de memória para VDP em andamento.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DmaMemoria {
    restante: u32,
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
    dma_ativo: bool,
    dma_memoria: Option<DmaMemoria>,
    /// Palavras que o DMA de memória ainda pode ler, na ordem.
    fila_dma: VecDeque<u16>,
    /// Escritas que o jogo fez nas memórias e que o renderizador precisa notar (sprites).
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
            dma_ativo: false,
            dma_memoria: None,
            fila_dma: VecDeque::new(),
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

    /// Entrega as palavras que o DMA de memória leu (vindas do `.s1v` ou do núcleo).
    pub fn entregar_dma(&mut self, palavras: &[u16]) {
        self.fila_dma.extend(palavras.iter().copied());
        self.continuar_dma();
    }

    // ---- Porta de controle ----

    pub fn escrever_controle(&mut self, valor: u16) {
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
                self.iniciar_dma();
            }
        }
    }

    fn escrever_registrador(&mut self, valor: u16) {
        let reg = usize::from((valor >> 8) & 0x1f);
        if reg >= QUANTIDADE {
            return;
        }
        // Em modo 4 do Master System, registradores acima de 10 não mudam (como no oráculo).
        if self.registros.modo4() && reg > 10 {
            self.avisos.modo4 = self.avisos.modo4.saturating_add(1);
            return;
        }
        let byte = valor.to_le_bytes()[0];
        if let Some(r) = self.registros.brutos.get_mut(reg) {
            *r = byte;
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
        // Todo acesso à porta de dados zera a marca de "segunda palavra".
        self.segunda_palavra = false;
        let entrada = EntradaFifo {
            sentido: self.sentido,
            alvo: self.alvo,
            endereco: self.endereco_de_dados,
            palavra,
        };
        self.endereco_de_dados = self
            .endereco_de_dados
            .wrapping_add(u32::from(self.registros.incremento()));
        self.aplicar(entrada);
        // O preenchimento começa depois da primeira escrita; o dado é a palavra dessa escrita.
        if self.dma_ativo && self.registros.modo_dma() == ModoDma::PreenchimentoDeVram {
            self.preencher(palavra);
        }
    }

    /// Grava uma entrada na memória (o "pop" da FIFO do oráculo).
    pub(crate) fn aplicar(&mut self, e: EntradaFifo) {
        if e.sentido != Sentido::Escrita {
            return;
        }
        match e.alvo {
            Alvo::Vram => {
                let a = usize::from(u16::try_from(e.endereco & 0xffff).unwrap_or(0));
                let [hi, lo] = e.palavra.to_be_bytes();
                self.gravar_byte_vram(a, hi);
                self.gravar_byte_vram(a ^ 1, lo);
            }
            Alvo::Cram => {
                let i = usize::try_from((e.endereco & 0x7f) >> 1).unwrap_or(0);
                if let Some(c) = self.cram.get_mut(i) {
                    *c = e.palavra;
                }
            }
            Alvo::Vsram => {
                let i = usize::try_from((e.endereco & 0x7f) >> 1).unwrap_or(usize::MAX);
                if let Some(v) = self.vsram.get_mut(i) {
                    *v = e.palavra;
                }
            }
            Alvo::Vram8Bits | Alvo::Invalido => {}
        }
    }

    fn gravar_byte_vram(&mut self, endereco: usize, valor: u8) {
        if let Some(b) = self.vram.get_mut(endereco) {
            *b = valor;
        }
        if let Ok(a) = u32::try_from(endereco) {
            self.ao_gravar_vram.push((a, valor));
        }
    }

    // ---- DMA ----

    fn iniciar_dma(&mut self) {
        match self.registros.modo_dma() {
            ModoDma::MemoriaParaVdp => {
                self.dma_memoria = Some(DmaMemoria {
                    restante: self.registros.comprimento_dma(),
                });
                self.continuar_dma();
            }
            ModoDma::PreenchimentoDeVram => {
                // Espera a próxima escrita na porta de dados.
            }
            ModoDma::CopiaDeVram => {
                self.avisos.dma_copia = self.avisos.dma_copia.saturating_add(1);
                self.dma_ativo = false;
            }
        }
    }

    /// Transfere as palavras disponíveis do DMA de memória para o alvo.
    fn continuar_dma(&mut self) {
        while let Some(DmaMemoria { restante }) = self.dma_memoria {
            let Some(palavra) = self.fila_dma.pop_front() else {
                return;
            };
            let entrada = EntradaFifo {
                sentido: self.sentido,
                alvo: self.alvo,
                endereco: self.endereco_de_dados,
                palavra,
            };
            self.endereco_de_dados = self
                .endereco_de_dados
                .wrapping_add(u32::from(self.registros.incremento()));
            self.aplicar(entrada);
            let resto = restante.saturating_sub(1);
            if resto == 0 {
                self.dma_memoria = None;
                self.dma_ativo = false;
            } else {
                self.dma_memoria = Some(DmaMemoria { restante: resto });
            }
        }
    }

    /// Fim do quadro de DMA: se ainda faltavam palavras, conta como aviso (sem dados).
    pub fn conferir_dma_pendente(&mut self) {
        if self.dma_memoria.is_some() {
            self.avisos.dma_sem_dados = self.avisos.dma_sem_dados.saturating_add(1);
        }
    }

    fn preencher(&mut self, palavra: u16) {
        if self.alvo != Alvo::Vram {
            self.avisos.dma_preenchimento_fora_da_vram =
                self.avisos.dma_preenchimento_fora_da_vram.saturating_add(1);
            self.dma_ativo = false;
            return;
        }
        let byte = palavra.to_be_bytes()[0];
        let mut restante = self.registros.comprimento_dma();
        while restante > 0 {
            let a = usize::try_from((self.endereco_de_dados ^ 1) & 0xffff).unwrap_or(0);
            self.gravar_byte_vram(a, byte);
            self.endereco_de_dados = self
                .endereco_de_dados
                .wrapping_add(u32::from(self.registros.incremento()));
            restante = restante.saturating_sub(1);
        }
        self.dma_ativo = false;
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
        assert_eq!(&v.vram()[0..4], &[0x12, 0x34, 0xab, 0xcd]);
    }

    #[test]
    fn palavra_em_endereco_impar_troca_os_bytes() {
        let mut v = pronta();
        v.escrever_controle(0x4001); // VRAM, endereco 1
        v.escrever_controle(0x0000);
        v.escrever_dados(0x1234, 2);
        assert_eq!(v.vram()[1], 0x12);
        assert_eq!(v.vram()[0], 0x34);
    }

    #[test]
    fn cram_e_vsram() {
        let mut v = pronta();
        v.escrever_controle(0xC004); // CRAM, endereco 4 (cor 2)
        v.escrever_controle(0x0000);
        v.escrever_dados(0x0EEE, 2);
        assert_eq!(v.cram()[2], 0x0EEE);
        v.escrever_controle(0x4002); // VSRAM, endereco 2
        v.escrever_controle(0x0010);
        v.escrever_dados(0x0155, 2);
        assert_eq!(v.vsram()[1], 0x0155);
        // A VSRAM tem 40 palavras: o indice 40 (endereco $50 depois da mascara $7F) nao grava nada.
        v.escrever_controle(0x4050);
        v.escrever_controle(0x0010);
        v.escrever_dados(0x0777, 2);
        assert!(v.vsram().iter().all(|x| *x != 0x0777));
    }

    #[test]
    fn leitura_nao_grava() {
        let mut v = pronta();
        v.escrever_controle(0x0000); // VRAM, leitura
        v.escrever_controle(0x0000);
        v.escrever_dados(0x1111, 2);
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
        assert_eq!(
            &v.vram()[0x8000..0x8008],
            &[0x11, 0x11, 0x22, 0x22, 0x33, 0x33, 0x44, 0x44]
        );
        assert_eq!(v.avisos.total(), 0);
    }

    #[test]
    fn dma_continua_no_quadro_seguinte() {
        let mut v = pronta();
        for w in [0x9304u16, 0x9400, 0x9500, 0x9608, 0x9700, 0x4000, 0x0082] {
            v.escrever_controle(w);
        }
        v.entregar_dma(&[0xaaaa, 0xbbbb]);
        assert_eq!(&v.vram()[0x8000..0x8004], &[0xaa, 0xaa, 0xbb, 0xbb]);
        assert_eq!(v.vram()[0x8004], 0);
        v.conferir_dma_pendente();
        assert_eq!(v.avisos.dma_sem_dados, 1);
        v.entregar_dma(&[0xcccc, 0xdddd]);
        assert_eq!(&v.vram()[0x8004..0x8008], &[0xcc, 0xcc, 0xdd, 0xdd]);
    }

    #[test]
    fn dma_sem_permissao_nao_dispara() {
        let mut v = Vdp::nova();
        for w in [0x8164u16, 0x8F02, 0x9304, 0x9400, 0x4000, 0x0082] {
            v.escrever_controle(w);
        }
        v.entregar_dma(&[0x1111, 0x2222, 0x3333, 0x4444]);
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
        assert!(v.vram().iter().all(|b| *b == 0));
    }
}
