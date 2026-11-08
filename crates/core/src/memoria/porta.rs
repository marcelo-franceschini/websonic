//! Portas de controle do console (porta 1, porta 2 e porta de extensão).
//!
//! Estado de memória do hardware, não de CPU: o que os pinos mostram e quais pinos são saída.
//! Reproduz o comportamento observado no oráculo para o controle de 3 botões (porta 1) e para
//! "nada ligado" (porta 2 e extensão). Limite do nível 1: a volta do pino TH a 1 depois de
//! alguns ciclos, quando o jogo deixa TH como entrada, depende de tempo e fica para o nível 2.

const TH: u8 = 6;
const TR: u8 = 5;
const TL: u8 = 4;

/// Que dispositivo está ligado numa porta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dispositivo {
    /// Controle de 3 botões. A máscara tem os bits do corpus: Cima 01, Baixo 02, Esquerda 04,
    /// Direita 08, B 10, C 20, A 40, Start 80.
    TresBotoes(u8),
    Nada,
}

/// Pinos de uma porta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Porta {
    /// Bit a 1 = pino é saída. O bit 7 é sempre lido como 1 internamente.
    direcoes: u8,
    pinos: u8,
    bit7_do_controle: bool,
    ultimo_dado: u8,
    tx: u8,
    dispositivo: Dispositivo,
}

fn bit(valor: u8, n: u8) -> bool {
    valor & (1u8.wrapping_shl(u32::from(n))) != 0
}

impl Porta {
    pub fn nova(dispositivo: Dispositivo) -> Self {
        let mut p = Self {
            direcoes: 0,
            pinos: 0xFF,
            bit7_do_controle: false,
            ultimo_dado: 0xFF,
            tx: 0xFF,
            dispositivo,
        };
        p.atualizar_pinos();
        p
    }

    /// Define um pino de entrada, se ele não for saída.
    fn entrada(&mut self, n: u8, nivel: bool) {
        if !bit(self.direcoes, n) {
            let mascara = 1u8.wrapping_shl(u32::from(n));
            self.pinos = (self.pinos & !mascara) | if nivel { mascara } else { 0 };
        }
    }

    fn entrada_nibble(&mut self, pinos: u8) {
        let baixo = (self.pinos & self.direcoes) | (pinos & !self.direcoes);
        self.pinos = (self.pinos & !0x0F) | (baixo & 0x0F);
    }

    fn atualizar_pinos(&mut self) {
        match self.dispositivo {
            Dispositivo::Nada => {
                self.entrada(TH, true);
                self.entrada(TR, true);
                self.entrada(TL, true);
                self.entrada_nibble(0b1111);
            }
            Dispositivo::TresBotoes(m) => {
                let tecla = |bit_da_tecla: u8| m & bit_da_tecla != 0;
                let (cima, baixo, esq, dir) = (tecla(0x01), tecla(0x02), tecla(0x04), tecla(0x08));
                // Direções opostas ao mesmo tempo se anulam (o oráculo não as permite).
                let (cima, baixo) = if cima && baixo {
                    (false, false)
                } else {
                    (cima, baixo)
                };
                let (esq, dir) = if esq && dir {
                    (false, false)
                } else {
                    (esq, dir)
                };
                let (b, c, a, start) = (tecla(0x10), tecla(0x20), tecla(0x40), tecla(0x80));
                if bit(self.pinos, TH) {
                    self.entrada(TR, !c);
                    self.entrada(TL, !b);
                    self.entrada(3, !dir);
                    self.entrada(2, !esq);
                    self.entrada(1, !baixo);
                    self.entrada(0, !cima);
                } else {
                    self.entrada(TR, !start);
                    self.entrada(TL, !a);
                    self.entrada(3, false);
                    self.entrada(2, false);
                    self.entrada(1, !baixo);
                    self.entrada(0, !cima);
                }
            }
        }
    }

    /// Troca os botões apertados (só faz algo se mudaram).
    pub fn definir_botoes(&mut self, mascara: u8) {
        if let Dispositivo::TresBotoes(atual) = self.dispositivo
            && atual != mascara
        {
            self.dispositivo = Dispositivo::TresBotoes(mascara);
            self.atualizar_pinos();
        }
    }

    fn saida(&mut self, valor: u8) {
        self.pinos = (self.pinos & !self.direcoes) | (valor & self.direcoes);
        self.atualizar_pinos();
    }

    pub fn ler_dado(&self) -> u8 {
        self.pinos
    }

    pub fn escrever_dado(&mut self, valor: u8) {
        self.ultimo_dado = valor;
        self.saida(valor);
    }

    pub fn ler_controle(&self) -> u8 {
        (self.direcoes & 0x7F) | if self.bit7_do_controle { 0x80 } else { 0 }
    }

    pub fn escrever_controle(&mut self, valor: u8) {
        self.direcoes = valor | 0x80;
        self.bit7_do_controle = valor & 0x80 != 0;
        self.saida(self.ultimo_dado);
    }

    pub fn ler_tx(&self) -> u8 {
        self.tx
    }

    pub fn escrever_tx(&mut self, valor: u8) {
        self.tx = valor;
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn leitura_do_controle_como_o_jogo_faz() {
        let mut p = Porta::nova(Dispositivo::TresBotoes(0));
        p.escrever_controle(0x40);
        // TH em 1: nenhum botao, tudo em 1 nos bits 0 a 5.
        p.escrever_dado(0x40);
        assert_eq!(p.ler_dado() & 0x3F, 0x3F);
        // Cima (01) e C (20) apertados, TH em 1.
        p.definir_botoes(0x21);
        assert_eq!(p.ler_dado() & 0x3F, 0b01_1110);
        // TH em 0: Start e A nos bits 5 e 4, bits 3 e 2 em 0.
        p.definir_botoes(0xC1);
        p.escrever_dado(0x00);
        assert_eq!(p.ler_dado() & 0x3F, 0b00_0010);
    }

    #[test]
    fn porta_sem_nada_le_tudo_alto() {
        let p = Porta::nova(Dispositivo::Nada);
        assert_eq!(p.ler_dado(), 0xFF);
        assert_eq!(p.ler_controle(), 0x00);
    }

    #[test]
    fn controle_guarda_o_bit_7_a_parte() {
        let mut p = Porta::nova(Dispositivo::Nada);
        p.escrever_controle(0x80 | 0x40);
        assert_eq!(p.ler_controle(), 0xC0);
        p.escrever_controle(0x40);
        assert_eq!(p.ler_controle(), 0x40);
    }
}
