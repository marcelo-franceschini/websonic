//! Coordenação experimental de escritas da CPU com a disponibilidade da VDP.
//!
//! O dispositivo deve compartilhar o estado e o relógio do renderizador. Esta interface não
//! inventa ciclos de instrução nem slots de FIFO; o chamador fornece o instante em que a CPU
//! tenta escrever, e o dispositivo informa quando pode tentar novamente.
use sonic1_core::memoria::Saida;
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

/// Resultado de uma tentativa atômica. A falha não pode enfileirar a escrita.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Admissao {
    Aceita,
    /// Próximo instante em que o dispositivo pode voltar a aceitar a escrita.
    AguardarAte(u64),
}

/// Adaptador para a VDP e o renderizador que compartilham o mesmo relógio.
pub trait Dispositivo {
    /// Avança o hardware e processa seus slots até o instante fornecido.
    fn avancar_ate(&mut self, mclk: u64);
    /// Tenta emitir uma escrita no instante atual, sem duplicá-la se a FIFO estiver cheia.
    fn tentar_escrita(&mut self, escrita: &Saida) -> Admissao;
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

    /// Admite a escrita e retorna seu instante real, incluindo a espera indicada pelo dispositivo.
    pub fn escrever(&mut self, escrita: &Saida, instante_cpu: u64) -> Result<u64, ErroAgendamento> {
        if instante_cpu < self.mclk {
            return Err(ErroAgendamento::RelogioRegrediu);
        }
        self.mclk = instante_cpu;
        loop {
            self.dispositivo.avancar_ate(self.mclk);
            match self.dispositivo.tentar_escrita(escrita) {
                Admissao::Aceita => return Ok(self.mclk),
                Admissao::AguardarAte(proximo) if proximo > self.mclk => self.mclk = proximo,
                Admissao::AguardarAte(_) => return Err(ErroAgendamento::EsperaSemProgresso),
            }
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use sonic1_core::memoria::{PortaVdp, Tamanho};
    use std::collections::VecDeque;

    #[derive(Default)]
    struct FifoSintetica {
        agora: u64,
        saidas: VecDeque<u64>,
        recebidas: Vec<(u64, u16)>,
    }

    impl Dispositivo for FifoSintetica {
        fn avancar_ate(&mut self, mclk: u64) {
            self.agora = mclk;
            while self.saidas.front().is_some_and(|fim| *fim <= mclk) {
                self.saidas.pop_front();
            }
        }
        fn tentar_escrita(&mut self, escrita: &Saida) -> Admissao {
            if self.saidas.len() == 4 {
                return Admissao::AguardarAte(*self.saidas.front().unwrap());
            }
            if let Saida::Vdp { valor, .. } = escrita {
                self.recebidas.push((self.agora, *valor));
            }
            self.saidas.push_back(self.agora.saturating_add(20));
            Admissao::Aceita
        }
    }

    #[test]
    fn fifo_cheia_adia_cpu_sem_duplicar_escritas() {
        let mut a = Agendador::novo(FifoSintetica::default(), 0);
        for valor in 0..5 {
            let escrita = Saida::Vdp {
                porta: PortaVdp::Dados,
                valor,
                tamanho: Tamanho::Word,
            };
            let quando = a.escrever(&escrita, 0).unwrap();
            assert_eq!(quando, if valor == 4 { 20 } else { 0 });
        }
        assert_eq!(a.dispositivo().recebidas.len(), 5);
        assert_eq!(a.dispositivo().recebidas[4], (20, 4));
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
