//! Modelo independente do barramento do Z80, escrito só a partir da tabela da seção 3 do plano
//! da Etapa 6, sem usar o código do núcleo. Serve para cruzar com o núcleo.
//!
//! Regras: BUSACK vale quando BUSREQ está pedido e o RESET está solto. O registrador de banco
//! guarda os nove últimos bits escritos, o mais antigo no bit 0.

/// Estado do barramento.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Modelo {
    pub busreq: bool,
    /// `true` enquanto o Z80 está em reset. O console liga com reset ativo.
    pub reset: bool,
    bits_de_banco: Vec<bool>,
}

impl Modelo {
    pub fn novo() -> Self {
        Self {
            busreq: false,
            reset: true,
            bits_de_banco: Vec::new(),
        }
    }

    pub fn busack(&self) -> bool {
        self.busreq && !self.reset
    }

    /// O Z80 escreveu um bit no registrador de banco.
    pub fn escrever_bit_de_banco(&mut self, bit: bool) {
        self.bits_de_banco.push(bit);
    }

    /// Os nove últimos bits, o mais antigo em primeiro lugar no bit 0.
    pub fn banco(&self) -> u16 {
        let n = self.bits_de_banco.len();
        let inicio = n.saturating_sub(9);
        let recentes = &self.bits_de_banco[inicio..];
        // Com menos de nove escritas o registrador anda para a direita: o primeiro bit escrito
        // chega ao bit (9 - n) e o resto fica em zero.
        let deslocamento = 9usize.saturating_sub(recentes.len());
        recentes
            .iter()
            .enumerate()
            .map(|(j, b)| u16::from(*b) << (j + deslocamento))
            .sum()
    }
}
