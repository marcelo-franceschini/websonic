//! FM do Mega Drive (YM2612), em inteiros, um ciclo interno por chamada (mestre/42).
//!
//! Estrutura (manual do YM2608 e pesquisa pública sobre o YM2612): 6 canais de 4 operadores,
//! 24 "slots" processados um por ciclo, 24 ciclos por amostra. Cada slot passa pelo gerador de
//! fase, pelo envelope e pelo operador (log-seno mais exponencial). O ritmo exato de cada
//! estágio foi medido por experimento contra o Nuked-OPN2 (`sintese.md`, D8.2).

use crate::tabelas::{DETUNE, EXP, LOGSIN};

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
    fb: [[i32; 3]; 6],
    saida_canal: [i32; 6],
    pino: (i16, i16),
    // Tempo absoluto em ciclos e escritas esperando para valer (atraso do barramento).
    tempo: u64,
    fila: Vec<(u64, usize, u8, u8)>,
    // Tecla pedida por slot e zeragem de fase pedida pelo envelope.
    tecla_pedida: [bool; 24],
    // Teclas escritas e ainda nao vistas pelo envelope: (tempo da escrita, canal, bits, aplicado por grupo).
    kon_fila: Vec<(u64, usize, u8, [bool; 4])>,
    zera_fase: [bool; 24],
    tecla_ligada: [bool; 24],
    eg_saida: [u16; 24],
    // Saida do grupo 0 da amostra anterior, para o latch do canal.
    g0_anterior: [i32; 6],
    pino_valor: [i32; 6],
    op_anterior: [i32; 24],
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
            fb: [[0; 3]; 6],
            saida_canal: [0; 6],
            pino: (3, 3),
            tempo: 0,
            fila: Vec::new(),
            tecla_pedida: [false; 24],
            kon_fila: Vec::new(),
            zera_fase: [false; 24],
            tecla_ligada: [false; 24],
            eg_saida: [1023; 24],
            g0_anterior: [0; 6],
            pino_valor: [0; 6],
            op_anterior: [0; 24],
        }
    }

    /// Escreve numa porta do chip (0 e 2 = endereço, 1 e 3 = dado).
    pub fn escreve(&mut self, porta: u32, valor: u8) {
        let parte = usize::from(porta & 2 != 0);
        if porta & 1 == 0 {
            self.endereco[parte] = valor;
        } else {
            let a = self.endereco[parte];
            if a == 0x28 {
                let c = usize::from(valor & 3);
                if c != 3 {
                    let canal = if valor & 4 != 0 { c.wrapping_add(3) } else { c };
                    self.kon_fila.push((self.tempo, canal, valor >> 4, [false; 4]));
                }
            } else {
                self.fila.push((self.tempo, parte, a, valor));
            }
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
            self.tecla_pedida[i] = ligado;
        }
    }

    /// Incremento de fase do slot de um canal e operador (sem LFO, sem detune ainda).
    fn incremento(&self, ch: usize, o: usize) -> u32 {
        let c = &self.canal[ch];
        let base = (u32::from(c.fnum) << c.bloco) >> 1;
        let f = u32::from(c.fnum);
        let n4 = (f >> 10) & 1;
        let n3 = if n4 == 1 { u32::from(f & 0x380 != 0) } else { u32::from(f & 0x380 == 0x380) };
        let kcode = (u32::from(c.bloco) << 2) | (n4 << 1) | n3;
        let dt = self.op[ch][o].dt;
        let desl = u32::from(DETUNE[usize::from(dt & 3)][kcode as usize]);
        let base = if dt & 4 != 0 { base.wrapping_sub(desl) } else { base.wrapping_add(desl) } & 0x1_FFFF;
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
        // Escritas cujo atraso acabou valem agora, na ordem em que chegaram.
        let agora = self.tempo;
        let mut k = 0;
        while k < self.fila.len() {
            let (w, p, a, _) = self.fila[k];
            if Self::pronto(w, p, a, agora) {
                let (_, p, a, d) = self.fila.remove(k);
                self.registrador(p, a, d);
            } else {
                k = k.wrapping_add(1);
            }
        }
        self.tempo = agora.wrapping_add(1);
        let c = self.ciclo;
        // Estagio do envelope: o slot cujo ciclo de envelope e este.
        let ie = usize::try_from(c.wrapping_add(22) % 24).unwrap_or(0);
        self.aplica_teclas(ie);
        if self.tecla_pedida[ie] != self.tecla_ligada[ie] {
            self.tecla_ligada[ie] = self.tecla_pedida[ie];
            if self.tecla_ligada[ie] {
                self.nivel[ie] = 0;
                self.zera_fase[ie] = true;
            } else {
                self.nivel[ie] = 1023;
            }
        }
        // Slot processado neste ciclo: indice i = (ciclo - 5) mod 24.
        let i = usize::try_from(c.wrapping_add(19) % 24).unwrap_or(0);
        let (g, ch) = (i / 6, i % 6);
        let o = GRUPO_DO_OP.iter().position(|&x| x == g).unwrap_or(0);
        // Entrada de modulação segundo o algoritmo.
        let m = self.modulacao(ch, o);
        let eg = u32::from(self.op[ch][o].tl).wrapping_mul(8).wrapping_add(u32::from(self.eg_saida[i])).min(1023);
        let fase10 = (self.fase[i] >> 10).wrapping_add(u32::try_from((m >> 1) & 0x3FF).unwrap_or(0)) & 0x3FF;
        self.op_anterior[i] = self.saida_op[i];
        if g == 0 {
            self.g0_anterior[ch] = self.saida_op[i];
        }
        self.saida_op[i] = Self::operador(fase10, eg);
        self.eg_saida[i] = self.nivel[i];
        // A fase so avanca depois de a saida usar o valor antigo.
        if self.zera_fase[i] {
            self.zera_fase[i] = false;
            self.fase[i] = 0;
        } else {
            self.fase[i] = self.fase[i].wrapping_add(self.incremento(ch, o)) & 0xF_FFFF;
        }
        if o == 0 {
            self.fb[ch] = [self.saida_op[i], self.fb[ch][0], self.fb[ch][1]];
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
            // O valor do canal e copiado tres ciclos antes de aparecer no pino (medido).
            if cp.wrapping_add(21) % 24 == c {
                self.pino_valor[k] = self.saida_canal[k];
            }
            let falta = cp.wrapping_add(24).wrapping_sub(c) % 24;
            if falta >= 1 && falta <= 3 {
                // Nos tres ciclos antes do valor, o pino mostra o sinal do canal (-3 ou +3).
                let v = self.pino_valor[k];
                let s = |lado: bool| if lado && v < 0 { -3 } else { 3 };
                pino = (s(self.canal[k].esquerda), s(self.canal[k].direita));
            }
            if cp == c {
                let v = self.pino_valor[k];
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
        // Um operador so enxerga a saida de outro da mesma amostra se ela foi calculada pelo menos
        // 12 ciclos antes (medido: 6 ciclos nao bastam). Senao ele ve o valor da amostra anterior.
        let consumidor = GRUPO_DO_OP[o];
        let s = |op_addr: usize| {
            let produtor = GRUPO_DO_OP[op_addr];
            let i = produtor.wrapping_mul(6).wrapping_add(ch);
            if produtor < consumidor && consumidor.wrapping_sub(produtor) < 2 { self.op_anterior[i] } else { self.saida_op[i] }
        };
        let (s1, s3, s2) = (s(0), s(1), s(2));
        let a = self.canal[ch].algoritmo;
        match o {
            0 => {
                let fb = self.canal[ch].realimentacao;
                if fb == 0 { 0 } else {
                    (self.fb[ch][0].wrapping_add(self.fb[ch][1])) >> 9_u32.wrapping_sub(u32::from(fb))
                }
            }
            2 => match a { 0 | 3..=6 => s1, _ => 0 },
            1 => match a { 0 | 2 => s2, 1 => s1.wrapping_add(s2), 5 => s1, _ => 0 },
            _ => match a { 0 | 1 | 4 => s3, 2 => s1.wrapping_add(s3), 3 => s2.wrapping_add(s3), 5 => s1, _ => 0 },
        }
    }

    /// Soma das saídas portadoras segundo o algoritmo, limitada a 14 bits e reduzida a 9.
    fn soma_canal(&self, ch: usize) -> i32 {
        let s = |op_addr: usize| self.saida_op[GRUPO_DO_OP[op_addr].wrapping_mul(6).wrapping_add(ch)];
        let (s1, s3, s2, s4) = (self.g0_anterior[ch], s(1), s(2), s(3));
        // Operadores portadores de cada algoritmo, na ordem em que o chip os soma (S1, S3, S2, S4).
        let portadores: [bool; 4] = match self.canal[ch].algoritmo {
            0..=3 => [false, false, false, true],
            4 => [false, false, true, true],
            5 | 6 => [false, true, true, true],
            _ => [true, true, true, true],
        };
        // O acumulador satura em 9 bits a cada soma.
        let mut acc: i32 = 0;
        for (p, v) in portadores.iter().zip([s1, s3, s2, s4]) {
            if *p {
                acc = acc.wrapping_add(v >> 5).clamp(-256, 255);
            }
        }
        acc
    }

    /// Escada do DAC do modelo 1 (modo YM2612): três vezes o valor, mais 3 se não negativo.
    fn escada(v: i32) -> i16 {
        let r = v.wrapping_mul(3).wrapping_add(if v >= 0 { 3 } else { 0 });
        i16::try_from(r).unwrap_or(0)
    }
}

/// Uma escrita so vale quando passaram `atraso` ciclos e o ciclo coincide com o ponto onde o
/// registrador e gravado (medido por varredura, ver sintese.md): tecla no ciclo 8, operador e canal
/// no ciclo do proprio slot.
impl Fm {
    fn pronto(w: u64, parte: usize, a: u8, agora: u64) -> bool {
        let (fase, atraso, desloc) = match a {
            0x28 => (8u64, 12u64, 0u64),
            0x30..=0x9F => {
                let ch = usize::from(a & 3).wrapping_add(parte.wrapping_mul(3));
                let o = usize::from((a >> 2) & 3);
                (0, 13, u64::try_from(GRUPO_DO_OP[o].wrapping_mul(6).wrapping_add(ch)).unwrap_or(0))
            }
            0xA0..=0xB6 => (0, 7, u64::from(a & 3).wrapping_add(u64::try_from(parte.wrapping_mul(3)).unwrap_or(0))),
            _ => (0, 0, 0),
        };
        agora >= w.wrapping_add(atraso) && agora.wrapping_add(48).wrapping_sub(fase).wrapping_sub(desloc) % 24 == 0
    }
}

/// Acesso de diagnostico ao estado interno (para o comparador em camadas; nao altera nada).
impl Fm {
    pub fn diag_fase(&self) -> [u32; 24] { self.fase }
    pub fn diag_saida_op(&self) -> [i32; 24] { self.saida_op }
    pub fn diag_saida_canal(&self) -> [i32; 6] { self.saida_canal }
    pub fn diag_eg(&self) -> [u16; 24] { self.nivel }
}

impl Fm {
    /// A tecla escrita chega a cada slot do canal depois de `3 + 6 * g` ciclos, onde `g` e o grupo
    /// do slot (o grupo 0 conta como 4, isto e, 27 ciclos). Medido por varredura da fase da escrita
    /// (`diferencial-sintese/examples/sonda7.rs`).
    fn aplica_teclas(&mut self, slot: usize) {
        let (g, ch) = (slot / 6, slot % 6);
        let atraso = 3u64.wrapping_add(6u64.wrapping_mul(if g == 0 { 4 } else { u64::try_from(g).unwrap_or(0) }));
        // Bit do registrador $28 de cada grupo: S1 (g0), S3 (g1), S2 (g2), S4 (g3).
        let bit = [0usize, 2, 1, 3][g];
        let agora = self.tempo.wrapping_sub(1);
        for k in 0..self.kon_fila.len() {
            let (w, c, bits, ap) = self.kon_fila[k];
            if c == ch && !ap[g] && agora >= w.wrapping_add(atraso) {
                self.tecla_pedida[slot] = (bits >> bit) & 1 != 0;
                self.kon_fila[k].3[g] = true;
            }
        }
        self.kon_fila.retain(|e| !e.3.iter().all(|&x| x));
    }
}
