//! PSG do Mega Drive (bloco de som dentro do chip de vídeo, "YM7101 PSG", compatível com o SN76489).
//!
//! Fontes públicas: documentação do SN76489 (Maxim, SMS Power, "SN76489 notes") para os registradores
//! (3 tons de 10 bits, 1 ruído, atenuação de 4 bits em passos de 2 dB, ruído periódico ou branco com
//! retroalimentação pelos bits 15 e 12 de um deslocador de 16 bits, limpeza do deslocador ao escrever
//! no registrador de ruído). O Sega Retro descreve o PSG do console como parte do VDP.
//! O ritmo interno (qual ciclo atualiza o quê) não está na documentação textual: foi medido por
//! experimento contra o Nuked-PSG com vetores sintéticos (ver `docs/conhecimento/sintese.md`, D8.2).
//!
//! Modelo: um ciclo de entrada do chip por chamada de [`Psg::ciclo`] (mestre/15). O chip trabalha
//! em um ritmo de 4 ciclos: no ciclo 1 de cada 4 ele serve um dos quatro canais (contador), no
//! ciclo 3 de cada 4 ele aplica os registradores e anda para o canal seguinte. Os quatro canais
//! passam uma vez a cada 16 ciclos, que é uma amostra.

/// Atenuação de saída de um canal quando mudo.
const MUDO: u8 = 15;

#[derive(Clone, Debug)]
pub struct Psg {
    // Entrada de dados do barramento.
    dado: u8,
    escrita_pendente: bool,
    escrita_aplicar: bool,
    registrador: u8,
    // Pino de reset (IC) e seus atrasos internos.
    ic: bool,
    ic_atraso_prescaler: u8,
    ic_atraso: u8,
    mascara_dado: bool,
    reset_registradores: bool,
    // Divisor de relógio de 4 fases.
    prescaler_a: u8,
    prescaler_b: u8,
    // Registradores.
    volume: [u8; 4],
    frequencia: [u16; 3],
    ruido_dado: u8,
    ruido_gatilho: bool,
    ruido_gatilho_atraso: bool,
    // Estado dos canais.
    canal_sel: u8,
    rotacao: usize,
    contador: [u16; 4],
    estouros: u8,
    sinal: u8,
    ruido: u16,
    ruido_estouro_anterior: bool,
    ruido_sinal: bool,
}

impl Default for Psg {
    fn default() -> Self {
        Self::novo()
    }
}

impl Psg {
    /// Chip depois de um pulso de reset de 16 ciclos (o estado de partida de todos os testes).
    pub fn novo() -> Self {
        let mut c = Self {
            dado: 0,
            escrita_pendente: false,
            escrita_aplicar: false,
            registrador: 0,
            ic: true,
            ic_atraso_prescaler: 0,
            ic_atraso: 0,
            mascara_dado: false,
            reset_registradores: false,
            prescaler_a: 0,
            prescaler_b: 0,
            volume: [0; 4],
            frequencia: [0; 3],
            ruido_dado: 0,
            ruido_gatilho: false,
            ruido_gatilho_atraso: false,
            canal_sel: 0,
            rotacao: 0,
            contador: [0; 4],
            estouros: 0,
            sinal: 0,
            ruido: 0,
            ruido_estouro_anterior: false,
            ruido_sinal: false,
        };
        for _ in 0..16 {
            c.ciclo();
        }
        c.ic = false;
        c
    }

    /// Escreve um byte no barramento do PSG. Vale a partir do próximo ciclo.
    pub fn escreve(&mut self, dado: u8) {
        self.dado = dado;
        self.escrita_pendente = true;
    }

    /// Um ciclo do relógio de entrada (mestre/15).
    pub fn ciclo(&mut self) {
        let prescaler_b_antes = self.prescaler_b;
        self.prescaler_b = (self.prescaler_b ^ self.prescaler_a) & 1;
        self.prescaler_a ^= 1;
        // A borda de descida do IC zera o divisor (alinha a fase).
        if (self.ic_atraso_prescaler & 3) == 2 {
            self.prescaler_a = 0;
            self.prescaler_b = 0;
        }
        self.ic_atraso_prescaler = (self.ic_atraso_prescaler << 1) | u8::from(self.ic);
        self.atualiza_registradores();
        self.escrita_aplicar = false;
        if self.escrita_pendente {
            // Um byte com o bit 7 ligado escolhe o registrador (latch).
            let d = if self.mascara_dado { 0 } else { self.dado };
            if d & 0x80 != 0 {
                self.registrador = (d >> 4) & 7;
            }
            self.escrita_pendente = false;
            self.escrita_aplicar = true;
        }
        if self.prescaler_a != 0 {
            if prescaler_b_antes == 0 {
                self.serve_canal();
            } else {
                self.fim_da_volta();
            }
        }
    }

    /// Aplica a escrita pendente (ou o reset) aos registradores.
    fn atualiza_registradores(&mut self) {
        if !self.reset_registradores && !self.escrita_aplicar {
            return;
        }
        let d = if self.mascara_dado { 0 } else { self.dado };
        let reset = self.reset_registradores;
        let alvo = |r: u8, s: &Self| reset || (s.escrita_aplicar && s.registrador == r);
        // Volumes (registradores 1, 3, 5, 7).
        for (i, r) in [1u8, 3, 5, 7].into_iter().enumerate() {
            if alvo(r, self) {
                self.volume[i] = if self.mascara_dado { MUDO } else { d & 15 };
            }
        }
        // Períodos dos tons (registradores 0, 2, 4): byte de latch traz os 4 bits baixos,
        // byte de dado traz os 6 bits altos.
        for (i, r) in [0u8, 2, 4].into_iter().enumerate() {
            if alvo(r, self) {
                if (d & 0x80) != 0 || reset {
                    self.frequencia[i] = (self.frequencia[i] & 0x3F0) | u16::from(d & 15);
                }
                if d & 0x80 == 0 {
                    self.frequencia[i] = (self.frequencia[i] & 15) | ((u16::from(d) << 4) & 0x3F0);
                }
            }
        }
        // Controle do ruído (registrador 6): limpa o deslocador pelo gatilho.
        if alvo(6, self) {
            self.ruido_dado = d & 7;
            self.ruido_gatilho = true;
        }
    }

    /// Serve o canal da vez: compara o contador com o período e atualiza o deslocador de ruído.
    fn serve_canal(&mut self) {
        let taxa = self.ruido_dado & 3;
        // O ruído de taxa 3 usa a saída do tom 3 como relógio.
        let relogio_ruido = if taxa == 3 {
            (self.sinal >> 1) & 1
        } else {
            self.sinal & 1
        } != 0;
        if self.ruido_gatilho_atraso || (self.ic_atraso & 1) != 0 {
            self.ruido = 0;
        } else if relogio_ruido && !self.ruido_estouro_anterior {
            let tap = ((self.ruido >> 15) ^ (self.ruido >> 12)) & 1 != 0;
            let branco = (self.ruido_dado & 4) != 0;
            // Um deslocador zerado recebe 1 para nunca travar.
            let entra = (tap && branco) || (self.ruido & 0x7FFF) == 0;
            self.ruido = (self.ruido << 1) | u16::from(entra);
        }
        self.ruido_estouro_anterior = relogio_ruido;

        let em_reset = (self.ic_atraso & 2) != 0;
        let canal_sel = if em_reset { 0 } else { self.canal_sel };
        let mut periodo: u16 = 0;
        for (i, frequencia) in self.frequencia.iter().enumerate() {
            if (canal_sel >> i) & 1 != 0 {
                periodo |= frequencia;
            }
        }
        if canal_sel & 8 != 0 {
            periodo |= match taxa {
                0 => 16,
                1 => 32,
                2 => 64,
                _ => 0,
            };
        }
        // No começo de cada volta (canal 0 da vez), os estouros dos quatro canais da volta anterior
        // invertem os bits de sinal.
        if self.canal_sel & 1 != 0 {
            self.sinal ^= self.estouros & 15;
        }
        if em_reset {
            self.sinal = 0;
        }
        self.estouros <<= 1;
        let r = self.rotacao;
        if self.contador[r] >= periodo {
            self.estouros |= 1;
            self.contador[r] = 0;
        }
        if em_reset {
            self.contador[r] = 0;
        }
        self.contador[r] = self.contador[r].wrapping_add(1) & 1023;
        // Seleção do canal: um bit que anda; reinjetado quando some.
        if (self.ic_atraso & 1) != 0 || (self.canal_sel & 7) != 0 {
            self.canal_sel <<= 1;
        } else {
            self.canal_sel = (self.canal_sel << 1) | 1;
        }
        self.ic_atraso = (self.ic_atraso << 1) | u8::from(self.ic);
        self.ruido_gatilho_atraso = self.ruido_gatilho;
    }

    /// Segunda metade do ritmo: aplica máscaras, anda para o canal seguinte e guarda o sinal do ruído.
    fn fim_da_volta(&mut self) {
        self.mascara_dado = (self.ic_atraso >> 1) & 1 != 0;
        self.reset_registradores = self.ic_atraso & 1 != 0;
        if self.ruido_gatilho_atraso {
            self.ruido_gatilho = false;
        }
        self.atualiza_registradores();
        self.rotacao = self.rotacao.wrapping_add(1) & 3;
        self.ruido_sinal = (self.ruido >> 14) & 1 != 0;
    }

    /// Saída digital: a atenuação de cada canal (0 = mais alto, 15 = mudo), tons 0 a 2 e ruído.
    pub fn saida(&self) -> [u8; 4] {
        let mut sinal = self.sinal & 14;
        sinal |= u8::from(self.ruido_sinal);
        let mut s = [MUDO; 4];
        for (i, v) in s.iter_mut().enumerate() {
            if (sinal >> 3_usize.wrapping_sub(i)) & 1 != 0 {
                *v = self.volume[i];
            }
        }
        s
    }

    /// A saída empacotada em 16 bits, 4 bits por canal (canal 0 nos bits altos).
    pub fn saida_empacotada(&self) -> u16 {
        let sinal = (self.sinal & 14) | u8::from(self.ruido_sinal);
        let c0 = if (sinal & 8) != 0 {
            self.volume[0]
        } else {
            MUDO
        };
        let c1 = if (sinal & 4) != 0 {
            self.volume[1]
        } else {
            MUDO
        };
        let c2 = if (sinal & 2) != 0 {
            self.volume[2]
        } else {
            MUDO
        };
        let c3 = if (sinal & 1) != 0 {
            self.volume[3]
        } else {
            MUDO
        };
        (u16::from(c0) << 12) | (u16::from(c1) << 8) | (u16::from(c2) << 4) | u16::from(c3)
    }

    /// Roda 16 ciclos e devolve uma amostra.
    pub fn amostra(&mut self) -> u16 {
        for _ in 0..16 {
            self.ciclo();
        }
        self.saida_empacotada()
    }
}
