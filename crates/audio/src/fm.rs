//! FM do Mega Drive (YM2612), em inteiros, um ciclo interno por chamada (mestre/42).
//!
//! Estrutura (manual do YM2608 e pesquisa pública sobre o YM2612): 6 canais de 4 operadores,
//! 24 "slots" processados um por ciclo, 24 ciclos por amostra. Cada slot passa pelo gerador de
//! fase, pelo envelope e pelo operador (log-seno mais exponencial). O ritmo exato de cada
//! estágio foi medido por experimento contra o Nuked-OPN2 (`sintese.md`, D8.2).

use crate::tabelas::{EXP, LOGSIN};

/// Ciclos internos por amostra.
pub const CICLOS_POR_AMOSTRA: u32 = 24;

/// Parâmetros de um operador (registradores `$30` a `$9F`).
#[derive(Clone, Copy, Default, Debug)]
struct Operador {
    dt: u8,
    mul: u8,
    tl: u8,
    ks: u8,
    ar: u8,
    am: bool,
    d1r: u8,
    d2r: u8,
    sl: u8,
    rr: u8,
    ssg: u8,
}

/// Parâmetros de um canal.
#[derive(Clone, Copy, Default, Debug)]
struct Canal {
    fnum: u16,
    bloco: u8,
    algoritmo: u8,
    realimentacao: u8,
    esquerda: bool,
    direita: bool,
    ams: u8,
    pms: u8,
}

#[derive(Clone, Debug)]
pub struct Fm {
    ciclo: u32,
    endereco: [u8; 2],
    op: [[Operador; 4]; 6],
    canal: [Canal; 6],
    // Estado por slot (indice = grupo * 6 + canal).
    fase: [u32; 24],
    nivel: [u16; 24],
    saida_op: [i32; 24],
    // Realimentacao do operador 1 (duas saidas anteriores) por canal.
    fb: [[i32; 2]; 6],
    saida_canal: [i32; 6],
    pino: (i16, i16),
}

/// Ordem dos grupos de slots no pipeline para o operador de endereço `o` (0 a 3).
const GRUPO_DO_OP: [usize; 4] = [0, 1, 2, 3];

impl Default for Fm {
    fn default() -> Self {
        Self::novo()
    }
}

impl Fm {
    pub fn novo() -> Self {
        Self {
            ciclo: 0,
            endereco: [0; 2],
            op: [[Operador::default(); 4]; 6],
            canal: [Canal::default(); 6],
            fase: [0; 24],
            nivel: [1023; 24],
            saida_op: [0; 24],
            fb: [[0; 2]; 6],
            saida_canal: [0; 6],
            pino: (3, 3),
        }
    }

    /// Escreve numa porta do chip (0 e 2 = endereço, 1 e 3 = dado).
    pub fn escreve(&mut self, porta: u32, valor: u8) {
        let parte = usize::from(porta & 2 != 0);
        if porta & 1 == 0 {
            self.endereco[parte] = valor;
        } else {
            let a = self.endereco[parte];
            self.registrador(parte, a, valor);
        }
    }

    fn registrador(&mut self, parte: usize, a: u8, d: u8) {
        let sub = usize::from(a & 3);
        if sub == 3 && a >= 0x30 {
            return;
        }
        let ch = sub.wrapping_add(parte.wrapping_mul(3));
        let o = usize::from((a >> 2) & 3);
        match a {
            0x28 => {
                let c = usize::from(d & 3);
                let canal = if d & 4 != 0 { c.wrapping_add(3) } else { c };
                if c == 3 {
                    return;
                }
                let on = d >> 4;
                self.tecla(canal, on);
            }
            0x30..=0x3F => {
                self.op[ch][o].dt = (d >> 4) & 7;
                self.op[ch][o].mul = d & 15;
            }
            0x40..=0x4F => self.op[ch][o].tl = d & 0x7F,
            0x50..=0x5F => {
                self.op[ch][o].ks = d >> 6;
                self.op[ch][o].ar = d & 31;
            }
            0x60..=0x6F => {
                self.op[ch][o].am = d & 0x80 != 0;
                self.op[ch][o].d1r = d & 31;
            }
            0x70..=0x7F => self.op[ch][o].d2r = d & 31,
            0x80..=0x8F => {
                self.op[ch][o].sl = d >> 4;
                self.op[ch][o].rr = d & 15;
            }
            0x90..=0x9F => self.op[ch][o].ssg = d & 15,
            0xA0..=0xA2 => self.canal[ch].fnum = (self.canal[ch].fnum & 0x700) | u16::from(d),
            0xA4..=0xA6 => {
                self.canal[ch].fnum = (self.canal[ch].fnum & 0xFF) | (u16::from(d & 7) << 8);
                self.canal[ch].bloco = (d >> 3) & 7;
            }
            0xB0..=0xB2 => {
                self.canal[ch].algoritmo = d & 7;
                self.canal[ch].realimentacao = (d >> 3) & 7;
            }
            0xB4..=0xB6 => {
                self.canal[ch].esquerda = d & 0x80 != 0;
                self.canal[ch].direita = d & 0x40 != 0;
                self.canal[ch].ams = (d >> 4) & 3;
                self.canal[ch].pms = d & 7;
            }
            _ => {}
        }
    }

    /// Liga ou desliga os operadores de um canal (bits 4 a 7: S1, S2, S3, S4).
    fn tecla(&mut self, canal: usize, bits: u8) {
        // Mapa do bit do registrador $28 para o endereço do operador: S1=0, S2=2, S3=1, S4=3.
        for (bit, o) in [(0usize, 0usize), (1, 2), (2, 1), (3, 3)] {
            let i = GRUPO_DO_OP[o].wrapping_mul(6).wrapping_add(canal);
            let ligado = (bits >> bit) & 1 != 0;
            if ligado {
                self.fase[i] = 0;
                self.nivel[i] = 0;
            } else {
                self.nivel[i] = 1023;
            }
        }
    }

    /// Incremento de fase do slot de um canal e operador (sem LFO, sem detune ainda).
    fn incremento(&self, ch: usize, o: usize) -> u32 {
        let c = &self.canal[ch];
        let base = (u32::from(c.fnum) << c.bloco) >> 1;
        let mul = u32::from(self.op[ch][o].mul);
        let r = if mul == 0 { base >> 1 } else { base.wrapping_mul(mul) };
        r & 0xF_FFFF
    }

    /// Saída do operador para uma fase de 10 bits e uma atenuação.
    fn operador(fase10: u32, atenuacao: u32) -> i32 {
        let mut idx = fase10 & 0xFF;
        if fase10 & 0x100 != 0 {
            idx ^= 0xFF;
        }
        let x = u32::from(LOGSIN[idx as usize]).wrapping_add(atenuacao << 2);
        let e = u32::from(EXP[(x & 0xFF) as usize]);
        let sh = x >> 8;
        let v = if sh >= 13 { 0 } else { ((e | 0x400) << 2) >> sh };
        let v = i32::try_from(v).unwrap_or(0);
        if fase10 & 0x200 != 0 { v.wrapping_neg() } else { v }
    }

    /// Um ciclo interno. Devolve o valor do pino (esquerda, direita).
    pub fn ciclo(&mut self) -> (i16, i16) {
        let c = self.ciclo;
        // Slot processado neste ciclo: indice i = (ciclo - 5) mod 24.
        let i = usize::try_from(c.wrapping_add(19) % 24).unwrap_or(0);
        let (g, ch) = (i / 6, i % 6);
        let o = GRUPO_DO_OP.iter().position(|&x| x == g).unwrap_or(0);
        // Fase.
        let inc = self.incremento(ch, o);
        self.fase[i] = self.fase[i].wrapping_add(inc) & 0xF_FFFF;
        // Entrada de modulação segundo o algoritmo.
        let m = self.modulacao(ch, o);
        let eg = u32::from(self.op[ch][o].tl).wrapping_mul(8).wrapping_add(u32::from(self.nivel[i])).min(1023);
        let fase10 = (self.fase[i] >> 10).wrapping_add(u32::try_from((m >> 1) & 0x3FF).unwrap_or(0)) & 0x3FF;
        self.saida_op[i] = Self::operador(fase10, eg);
        if o == 0 {
            self.fb[ch] = [self.saida_op[i], self.fb[ch][0]];
        }
        // Canal: latch um ciclo depois do slot do grupo 0.
        let c6 = usize::try_from(c.wrapping_add(18) % 24).unwrap_or(99); // canal cujo latch ocorre neste ciclo, se < 6
        if c6 < 6 {
            self.saida_canal[c6] = self.soma_canal(c6);
        }
        // Pino: canal k aparece em um ciclo proprio.
        const CICLO_PINO: [u32; 6] = [15, 3, 23, 11, 19, 7];
        let mut pino = (3i16, 3i16);
        for (k, &cp) in CICLO_PINO.iter().enumerate() {
            if cp == c {
                let v = self.saida_canal[k];
                let l = if self.canal[k].esquerda { v } else { 0 };
                let r = if self.canal[k].direita { v } else { 0 };
                pino = (Self::escada(l), Self::escada(r));
            }
        }
        self.pino = pino;
        self.ciclo = c.wrapping_add(1) % 24;
        pino
    }

    /// Modulação que entra no operador `o` do canal `ch`.
    fn modulacao(&self, ch: usize, o: usize) -> i32 {
        let s = |op_addr: usize| self.saida_op[GRUPO_DO_OP[op_addr].wrapping_mul(6).wrapping_add(ch)];
        let (s1, s3, s2) = (s(0), s(1), s(2));
        let a = self.canal[ch].algoritmo;
        match o {
            0 => {
                let fb = self.canal[ch].realimentacao;
                if fb == 0 { 0 } else { (self.fb[ch][0].wrapping_add(self.fb[ch][1])) >> 10_u32.wrapping_sub(u32::from(fb)) }
            }
            2 => if a <= 3 || a == 4 || a == 5 || a == 6 { s1 } else { 0 }, // S2 recebe S1 (algoritmos 0, 3 a 6)
            1 => match a { 0 => s2, 1 | 2 => s2.wrapping_add(if a == 1 { s1 } else { 0 }), 5 => s1, _ => 0 }, // S3
            _ => match a { 0..=2 => s3, 3 => s3.wrapping_add(s2), 4 => s3, 5 => s1, _ => 0 }, // S4
        }
    }

    /// Soma das saídas portadoras segundo o algoritmo, limitada a 14 bits e reduzida a 9.
    fn soma_canal(&self, ch: usize) -> i32 {
        let s = |op_addr: usize| self.saida_op[GRUPO_DO_OP[op_addr].wrapping_mul(6).wrapping_add(ch)];
        let (s1, s3, s2, s4) = (s(0), s(1), s(2), s(3));
        let t = match self.canal[ch].algoritmo {
            0..=3 => s4,
            4 => s2.wrapping_add(s4),
            5 | 6 => s2.wrapping_add(s3).wrapping_add(s4),
            _ => s1.wrapping_add(s2).wrapping_add(s3).wrapping_add(s4),
        };
        t.clamp(-8192, 8191) >> 5
    }

    /// Escada do DAC do modelo 1 (modo YM2612): três vezes o valor, mais 3 se não negativo.
    fn escada(v: i32) -> i16 {
        let r = v.wrapping_mul(3).wrapping_add(if v >= 0 { 3 } else { 0 });
        i16::try_from(r).unwrap_or(0)
    }
}
