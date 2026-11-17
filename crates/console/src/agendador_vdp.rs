//! Coordenação experimental de escritas da CPU com a disponibilidade da VDP.
//!
//! O dispositivo deve compartilhar o estado e o relógio do renderizador. Esta interface não
//! inventa ciclos de instrução nem slots de FIFO; o chamador fornece o instante em que a CPU
//! escreve, e o dispositivo informa quando o bloqueio da CPU termina.
use crate::Ponte;
use sonic1_core::memoria::{Memoria, Saida};
use sonic1_video::tempo::{LINHAS_POR_QUADRO, MCLK_POR_LINHA};

/// Posição no quadro de um instante do relógio mestre.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Posicao {
    pub quadro: u64,
    pub linha: u16,
    pub coluna: u64,
}

pub fn posicao(mclk: u64) -> Posicao {
    let linha_absoluta = mclk / MCLK_POR_LINHA;
    let linhas = u64::from(LINHAS_POR_QUADRO);
    Posicao {
        quadro: linha_absoluta / linhas,
        linha: u16::try_from(linha_absoluta % linhas).unwrap_or(0),
        coluna: mclk % MCLK_POR_LINHA,
    }
}

/// Adaptador para a VDP e o renderizador que compartilham o mesmo relógio.
pub trait Dispositivo {
    fn avancar_ate(&mut self, mclk: u64);
    fn emitir_escrita(&mut self, escrita: &Saida);
    fn cpu_bloqueada(&self) -> bool;
}

/// Adaptador experimental de escritas da VDP, sem modelar a execução da CPU.
pub struct FifoVideo<'a> {
    ponte: &'a mut Ponte,
    memoria: &'a mut Memoria,
    instante: u64,
    dma_do_trace: bool,
}

impl<'a> FifoVideo<'a> {
    pub fn nova(ponte: &'a mut Ponte, memoria: &'a mut Memoria) -> Self {
        Self {
            ponte,
            memoria,
            instante: 0,
            dma_do_trace: false,
        }
    }

    /// Usa palavras de DMA fornecidas por um trace em vez de reler a memória.
    pub fn nova_com_dma_do_trace(ponte: &'a mut Ponte, memoria: &'a mut Memoria) -> Self {
        let mut dispositivo = Self::nova(ponte, memoria);
        dispositivo.dma_do_trace = true;
        dispositivo
    }

    pub fn entregar_dma(&mut self, palavras: &[u16]) {
        self.ponte.rend.entregar_dma(palavras);
    }

    pub fn emitir_capturada(&mut self, c: &crate::Capturada) {
        let p = posicao(self.instante);
        self.ponte.aplicar_capturada(c, p.linha, p.coluna);
    }
}

impl Dispositivo for FifoVideo<'_> {
    fn avancar_ate(&mut self, mclk: u64) {
        let quadro_atual = posicao(self.instante).quadro;
        let p = posicao(mclk);
        for _ in quadro_atual..p.quadro {
            self.ponte
                .rend
                .avancar_ate(LINHAS_POR_QUADRO - 1, MCLK_POR_LINHA - 1);
            self.ponte.rend.avancar_ate(0, 0);
        }
        self.ponte.rend.avancar_ate(p.linha, p.coluna);
        self.instante = mclk;
    }

    fn emitir_escrita(&mut self, escrita: &Saida) {
        assert!(
            matches!(escrita, Saida::Vdp { .. }),
            "FifoVideo aceita apenas escritas da VDP"
        );
        let p = posicao(self.instante);
        if self.dma_do_trace {
            use sonic1_core::memoria::{PortaVdp, Tamanho};
            use sonic1_video::renderizador::Escrita;
            if let Saida::Vdp {
                porta,
                valor,
                tamanho,
            } = *escrita
            {
                let evento = match porta {
                    PortaVdp::Controle => Escrita::Controle(valor),
                    PortaVdp::Dados => Escrita::Dados {
                        valor,
                        tamanho: if tamanho == Tamanho::Byte { 1 } else { 2 },
                    },
                };
                self.ponte.rend.aplicar(p.linha, p.coluna, evento);
            }
        } else {
            self.ponte.aplicar(self.memoria, escrita, p.linha, p.coluna);
        }
    }

    fn cpu_bloqueada(&self) -> bool {
        self.ponte.rend.vdp().cpu_bloqueada()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErroAgendamento {
    RelogioRegrediu,
    EsperaSemProgresso,
}

/// O relógio da CPU e do dispositivo só avançam; o estado real da FIFO pertence ao dispositivo.
pub struct Agendador<D> {
    dispositivo: D,
    mclk: u64,
}

impl<D: Dispositivo> Agendador<D> {
    pub fn novo(dispositivo: D, mclk: u64) -> Self {
        Self { dispositivo, mclk }
    }

    pub fn instante(&self) -> u64 {
        self.mclk
    }

    pub fn dispositivo(&self) -> &D {
        &self.dispositivo
    }

    pub fn dispositivo_mut(&mut self) -> &mut D {
        &mut self.dispositivo
    }

    /// Emite uma palavra e retorna quando a CPU é liberada pela VDP.
    pub fn escrever(&mut self, escrita: &Saida, instante_cpu: u64) -> Result<u64, ErroAgendamento> {
        self.escrever_lote(std::slice::from_ref(escrita), instante_cpu)
    }

    /// Emite as palavras de uma instrução antes de esperar a liberação do barramento.
    pub fn escrever_lote(
        &mut self,
        escritas: &[Saida],
        instante_cpu: u64,
    ) -> Result<u64, ErroAgendamento> {
        if instante_cpu < self.mclk {
            return Err(ErroAgendamento::RelogioRegrediu);
        }
        self.mclk = instante_cpu;
        self.dispositivo.avancar_ate(self.mclk);
        for escrita in escritas {
            self.dispositivo.emitir_escrita(escrita);
        }
        let mut iteracoes = 0usize;
        while self.dispositivo.cpu_bloqueada() && iteracoes < 100_000 {
            iteracoes = iteracoes.saturating_add(1);
            let proximo = self.mclk.saturating_add(1);
            if proximo == self.mclk {
                return Err(ErroAgendamento::EsperaSemProgresso);
            }
            self.mclk = proximo;
            self.dispositivo.avancar_ate(self.mclk);
        }
        Ok(self.mclk)
    }
}

impl<'a> Agendador<FifoVideo<'a>> {
    /// Emite uma escrita capturada e aguarda liberacao da CPU caso haja bloqueio de DMA ou FIFO.
    pub fn escrever_capturada(
        &mut self,
        c: &crate::Capturada,
        instante_cpu: u64,
    ) -> Result<u64, ErroAgendamento> {
        if instante_cpu < self.mclk {
            return Err(ErroAgendamento::RelogioRegrediu);
        }
        self.mclk = instante_cpu;
        self.dispositivo.avancar_ate(self.mclk);
        self.dispositivo.emitir_capturada(c);
        let mut iteracoes = 0usize;
        while self.dispositivo.cpu_bloqueada() && iteracoes < 100_000 {
            iteracoes = iteracoes.saturating_add(1);
            let proximo = self.mclk.saturating_add(16);
            if proximo <= self.mclk {
                return Err(ErroAgendamento::EsperaSemProgresso);
            }
            self.mclk = proximo;
            self.dispositivo.avancar_ate(self.mclk);
        }
        Ok(self.mclk)
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use sonic1_core::memoria::{PortaVdp, Tamanho};

    #[test]
    fn adaptador_real_aguarda_liberacao_da_fifo() {
        let mut ponte = Ponte::nova();
        let mut memoria = Memoria::nova(vec![0; 0x100]);
        for valor in [0x8F02, 0xC000, 0] {
            let escrita = Saida::Vdp {
                porta: PortaVdp::Controle,
                valor,
                tamanho: Tamanho::Word,
            };
            ponte.aplicar(&mut memoria, &escrita, 0, 0);
        }
        let dispositivo = FifoVideo::nova(&mut ponte, &mut memoria);
        let mut agendador = Agendador::novo(dispositivo, 0);
        for valor in 0..4 {
            let escrita = Saida::Vdp {
                porta: PortaVdp::Dados,
                valor,
                tamanho: Tamanho::Word,
            };
            assert_eq!(agendador.escrever(&escrita, 0), Ok(0));
        }
        let escrita = Saida::Vdp {
            porta: PortaVdp::Dados,
            valor: 4,
            tamanho: Tamanho::Word,
        };
        let admitida = agendador.escrever(&escrita, 0).unwrap();
        assert!(admitida > 0);
        assert_eq!(agendador.dispositivo().ponte.rend.vdp().ocupacao_fifo(), 4);
        assert!(!agendador.dispositivo().ponte.rend.vdp().cpu_bloqueada());
    }

    #[test]
    fn duas_palavras_da_mesma_instrucao_entram_antes_da_espera() {
        let mut ponte = Ponte::nova();
        let mut memoria = Memoria::nova(vec![]);
        let mut a = Agendador::novo(FifoVideo::nova(&mut ponte, &mut memoria), 0);
        let palavra = |valor| Saida::Vdp {
            porta: PortaVdp::Dados,
            valor,
            tamanho: Tamanho::Word,
        };
        for valor in 0..3 {
            assert_eq!(a.escrever(&palavra(valor), 0), Ok(0));
        }
        let liberacao = a.escrever_lote(&[palavra(3), palavra(4)], 0).unwrap();
        assert!(liberacao > 0);
        assert_eq!(a.dispositivo().ponte.rend.vdp().ocupacao_fifo(), 4);
        assert!(!a.dispositivo().ponte.rend.vdp().cpu_bloqueada());
    }

    #[test]
    fn comando_dma_captura_ram_e_bloqueia_cpu_ate_conclusao() {
        let mut ponte = Ponte::nova();
        let mut memoria = Memoria::nova(vec![0; 0x100]);
        memoria.ram_mut()[0xFB00..0xFB04].copy_from_slice(&[0x00, 0x0E, 0x0E, 0xE0]);
        let mut a = Agendador::novo(FifoVideo::nova(&mut ponte, &mut memoria), 0);
        for valor in [
            0x8174, 0x8F02, 0x9302, 0x9400, 0x95_80, 0x96_FD, 0x97_7F, 0xC000,
        ] {
            let escrita = Saida::Vdp {
                porta: PortaVdp::Controle,
                valor,
                tamanho: Tamanho::Word,
            };
            assert_eq!(a.escrever(&escrita, 0).unwrap(), 0);
        }
        let comando = Saida::Vdp {
            porta: PortaVdp::Controle,
            valor: 0x0080,
            tamanho: Tamanho::Word,
        };
        let liberacao = a.escrever(&comando, 0).unwrap();
        assert!(liberacao > 0);
        assert!(!a.dispositivo().ponte.rend.vdp().cpu_bloqueada());
        drop(a);
        ponte.fechar_quadro();
        assert_eq!(&ponte.renderizador().vdp().cram()[..2], &[0x000E, 0x0EE0]);
    }

    #[test]
    fn converte_virada_de_linha_e_quadro() {
        assert_eq!(
            posicao(MCLK_POR_LINHA - 1),
            Posicao {
                quadro: 0,
                linha: 0,
                coluna: MCLK_POR_LINHA - 1
            }
        );
        assert_eq!(
            posicao(MCLK_POR_LINHA * u64::from(LINHAS_POR_QUADRO)),
            Posicao {
                quadro: 1,
                linha: 0,
                coluna: 0
            }
        );
    }
}
