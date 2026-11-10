//! FM do Mega Drive (YM2612), em inteiros, um ciclo interno por chamada (mestre/42).
//!
//! Estrutura (manual do YM2608 e pesquisa pública sobre o YM2612): 6 canais de 4 operadores,
//! 24 "slots" processados um por ciclo, 24 ciclos por amostra. Cada slot passa pelo gerador de
//! fase, pelo envelope e pelo operador (log-seno mais exponencial). O ritmo de cada estágio e do
//! barramento foi medido por experimento contra o chip de referência (`sintese.md`, D8.2), com as
//! sondas de `diferencial-sintese/examples/`. Nenhum código de emulador foi copiado.
//!
//! Slot `i` = operador (ordem de endereço: S1, S3, S2, S4) vezes 6, mais o canal.

use crate::tabelas::{DETUNE, EG_INC_ALTO, EG_INC_BAIXO, EXP, LOGSIN};

/// Ciclos internos por amostra.
pub const CICLOS_POR_AMOSTRA: u32 = 24;

/// Parâmetros de um operador (registradores `$30` a `$8F`).
#[derive(Clone, Copy, Default, Debug)]
struct Operador {
    dt: u8,
    mul: u8,
    tl: u8,
    ks: u8,
    ar: u8,
    d1r: u8,
    d2r: u8,
    sl: u8,
    rr: u8,
}

/// Parâmetros de um canal.
#[derive(Clone, Copy, Debug)]
struct Canal {
    fnum: u16,
    bloco: u8,
    algoritmo: u8,
    realimentacao: u8,
    esquerda: bool,
    direita: bool,
}

impl Canal {
    /// Depois do reset os dois lados do panning estão ligados (medido).
    const PADRAO: Self = Self { fnum: 0, bloco: 0, algoritmo: 0, realimentacao: 0, esquerda: true, direita: true };
}

/// Fases do envelope de um slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Eg {
    Ataque,
    Decai1,
    Decai2,
    Libera,
}

#[derive(Clone, Debug)]
pub struct Fm {
    ciclo: u32,
    tempo: u64,
    // Barramento: trava de endereço, dado de FM pendente e escritas de modo esperando um ciclo.
    parte: usize,
    endereco: u8,
    dado_fm: Option<u8>,
    fila_modo: Vec<(u64, u8, u8)>,
    reg_a4: u8,
    // Registradores gravados.
    op: [[Operador; 4]; 6],
    canal: [Canal; 6],
    // Cópia que cada slot faz dos seus parâmetros antes de ser processado.
    c_op: [Operador; 24],
    c_fnum: [u16; 24],
    c_bloco: [u8; 24],
    c_algo: [u8; 24],
    c_fb: [u8; 24],
    // Tecla: histórico curto do registrador $28 (tempo, canal, bits) e estado por slot.
    tecla_hist: Vec<(u64, usize, u8)>,
    tecla_pedida: [bool; 24],
    tecla_ligada: [bool; 24],
    // Fase e envelope por slot.
    fase: [u32; 24],
    zera_fase: [bool; 24],
    nivel: [u16; 24],
    eg_saida: [u16; 24],
    eg_estado: [Eg; 24],
    eg_visto: [u16; 24],
    eg_timer: u16,
    // Operadores e canais.
    saida_op: [i32; 24],
    op_anterior: [i32; 24],
    // Se a saída do slot entra na soma do canal, decidido quando o slot é processado.
    portador: [bool; 24],
    portador_ant: [bool; 24],
    realim: [[i32; 2]; 6],
    saida_canal: [i32; 6],
    pino_valor: [i32; 6],
    pino_lr: [(i32, i32); 6],
    dac_ligado: bool,
    dac_valor: i32,
    /// Variante de experimento da calibração (bits). Zero é o modelo adotado.
    pub variante: u32,
}

impl Default for Fm {
    fn default() -> Self {
        Self::novo()
    }
}

/// Ciclo em que o valor de cada canal aparece no pino (medido).
const CICLO_PINO: [u32; 6] = [15, 3, 23, 11, 19, 7];

impl Fm {
    pub fn novo() -> Self {
        Self {
            ciclo: 0,
            tempo: 0,
            parte: 0,
            endereco: 0,
            dado_fm: None,
            fila_modo: Vec::new(),
            reg_a4: 0,
            op: [[Operador::default(); 4]; 6],
            canal: [Canal::PADRAO; 6],
            c_op: [Operador::default(); 24],
            c_fnum: [0; 24],
            c_bloco: [0; 24],
            c_algo: [0; 24],
            c_fb: [0; 24],
            tecla_hist: Vec::new(),
            tecla_pedida: [false; 24],
            tecla_ligada: [false; 24],
            fase: [0; 24],
            zera_fase: [false; 24],
            nivel: [1023; 24],
            eg_saida: [1023; 24],
            eg_estado: [Eg::Libera; 24],
            eg_visto: [0; 24],
            eg_timer: 0,
            saida_op: [0; 24],
            op_anterior: [0; 24],
            portador: [false; 24],
            portador_ant: [false; 24],
            realim: [[0; 2]; 6],
            saida_canal: [0; 6],
            pino_valor: [0; 6],
            pino_lr: [(0, 0); 6],
            dac_ligado: false,
            dac_valor: 0,
            variante: 0,
        }
    }

    /// Escreve numa porta do chip (0 e 2 = endereço, 1 e 3 = dado).
    ///
    /// Medido: uma escrita de endereço cancela na hora o dado de FM que ainda não foi gravado.
    /// Registradores abaixo de `$30` valem um ciclo depois; a tecla (`$28`) é gravada na hora e
    /// chega a cada slot com atraso próprio.
    pub fn escreve(&mut self, porta: u32, valor: u8) {
        if porta & 1 == 0 {
            self.parte = usize::from(porta & 2 != 0);
            self.endereco = valor;
            self.dado_fm = None;
        } else if self.endereco >= 0x30 {
            self.dado_fm = Some(valor);
        } else if self.endereco == 0x28 {
            let c = usize::from(valor & 3);
            if c != 3 {
                let canal = if valor & 4 != 0 { c.wrapping_add(3) } else { c };
                self.tecla_hist.push((self.tempo, canal, valor >> 4));
            }
        } else {
            self.fila_modo.push((self.tempo.wrapping_add(1), self.endereco, valor));
        }
    }

    /// Grava o dado de FM pendente quando o ciclo do seu registrador passa. Medido: registradores
    /// de operador a cada 12 ciclos, de canal a cada 6 (`examples/sonda11.rs`).
    fn captura(&mut self, c: u32) {
        let Some(d) = self.dado_fm else { return };
        let a = self.endereco;
        let sub = u32::from(a & 3);
        if sub == 3 {
            return;
        }
        let ch = usize::from(a & 3).wrapping_add(self.parte.wrapping_mul(3));
        let o = usize::from((a >> 2) & 3);
        if a < 0xA0 {
            let slot = u32::from((a >> 2) & 3).wrapping_mul(6).wrapping_add(sub).wrapping_add(if self.parte == 1 { 3 } else { 0 });
            if c % 12 != slot.wrapping_add(11) % 12 {
                return;
            }
            let p = &mut self.op[ch][o];
            match a >> 4 {
                3 => {
                    p.dt = (d >> 4) & 7;
                    p.mul = d & 15;
                }
                4 => p.tl = d & 0x7F,
                5 => {
                    p.ks = d >> 6;
                    p.ar = d & 31;
                }
                6 => p.d1r = d & 31,
                7 => p.d2r = d & 31,
                8 => {
                    p.sl = d >> 4;
                    p.rr = d & 15;
                }
                _ => {}
            }
        } else {
            let chu = u32::try_from(ch).unwrap_or(0);
            if c % 6 != chu.wrapping_add(5) % 6 {
                return;
            }
            match a & 0xFC {
                // O byte alto da frequência fica numa trava e só vale quando o byte baixo chega.
                0xA0 => {
                    self.canal[ch].fnum = (u16::from(self.reg_a4 & 7) << 8) | u16::from(d);
                    self.canal[ch].bloco = (self.reg_a4 >> 3) & 7;
                }
                0xA4 => self.reg_a4 = d,
                0xB0 => {
                    self.canal[ch].algoritmo = d & 7;
                    self.canal[ch].realimentacao = (d >> 3) & 7;
                }
                0xB4 => {
                    self.canal[ch].esquerda = d & 0x80 != 0;
                    self.canal[ch].direita = d & 0x40 != 0;
                }
                _ => {}
            }
        }
    }

    /// Código de tecla (5 bits) a partir do bloco e dos bits altos do F-number (manual do YM2608).
    fn kcode(fnum: u16, bloco: u8) -> u32 {
        let f = u32::from(fnum);
        let n4 = (f >> 10) & 1;
        let n3 = if n4 == 1 { u32::from(f & 0x380 != 0) } else { u32::from(f & 0x380 == 0x380) };
        (u32::from(bloco) << 2) | (n4 << 1) | n3
    }

    /// Incremento de fase do slot (frequência, detune e multiplicador).
    fn incremento(&self, i: usize) -> u32 {
        let (fnum, bloco, p) = (self.c_fnum[i], self.c_bloco[i], &self.c_op[i]);
        let base = (u32::from(fnum) << bloco) >> 1;
        let desl = u32::from(DETUNE[usize::from(p.dt & 3)][Self::kcode(fnum, bloco) as usize]);
        let base = if p.dt & 4 != 0 { base.wrapping_sub(desl) } else { base.wrapping_add(desl) } & 0x1_FFFF;
        let mul = u32::from(p.mul);
        let r = if mul == 0 { base >> 1 } else { base.wrapping_mul(mul) };
        r & 0xF_FFFF
    }

    /// Saída do operador para uma fase de 10 bits e uma atenuação de 10 bits.
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
        let agora = self.tempo;
        let c = self.ciclo;
        // Escritas de modo cujo ciclo de espera acabou.
        let mut k = 0;
        while k < self.fila_modo.len() {
            if self.fila_modo[k].0 <= agora {
                let (_, a, d) = self.fila_modo.remove(k);
                match a {
                    0x2A => self.dac_valor = i32::from(d).wrapping_sub(128).wrapping_mul(2),
                    0x2B => self.dac_ligado = d & 0x80 != 0,
                    _ => {}
                }
            } else {
                k = k.wrapping_add(1);
            }
        }
        // O slot decide se é portador com o algoritmo de antes da gravação deste ciclo (medido:
        // `examples/sonda12.rs`).
        let algo_antes = [0usize, 1, 2, 3, 4, 5].map(|k| self.canal[k].algoritmo);
        self.captura(c);
        self.tempo = agora.wrapping_add(1);
        // O contador do envelope anda uma vez a cada 3 amostras (medido: o valor que os slots usam muda no ciclo 98, depois do slot 23).
        if agora >= 98 && agora.wrapping_sub(98).is_multiple_of(72) {
            self.eg_timer = self.eg_timer.wrapping_add(1) & 0xFFF;
        }
        // Cópias: cada slot lê seus parâmetros 12 ciclos antes de ser processado, e o algoritmo
        // e a realimentação 17 ciclos antes (medido pela calibração das classes).
        let ia = usize::try_from(c.wrapping_add(12) % 24).unwrap_or(0);
        self.c_algo[ia] = self.canal[ia % 6].algoritmo;
        self.c_fb[ia] = self.canal[ia % 6].realimentacao;
        let ip = usize::try_from(c.wrapping_add(7) % 24).unwrap_or(0);
        self.c_op[ip] = self.op[ip % 6][ip / 6];
        self.c_fnum[ip] = self.canal[ip % 6].fnum;
        self.c_bloco[ip] = self.canal[ip % 6].bloco;

        // Estágio do envelope do slot `ie`.
        let ie = usize::try_from(c.wrapping_add(22) % 24).unwrap_or(0);
        self.passo_envelope(ie);
        self.aplica_tecla(ie, agora);
        if self.tecla_pedida[ie] != self.tecla_ligada[ie] {
            self.tecla_ligada[ie] = self.tecla_pedida[ie];
            if self.tecla_ligada[ie] {
                self.eg_estado[ie] = Eg::Ataque;
                self.zera_fase[ie] = true;
                // Taxa de ataque 62 ou 63 leva o nível a zero na hora.
                if self.taxa_atual(ie) >= 62 {
                    self.nivel[ie] = 0;
                    self.eg_estado[ie] = Eg::Decai1;
                }
            } else {
                self.eg_estado[ie] = Eg::Libera;
            }
        }

        // Operador do slot `i`.
        let i = usize::try_from(c.wrapping_add(19) % 24).unwrap_or(0);
        let ch = i % 6;
        let m = self.modulacao(i);
        let eg = u32::from(self.c_op[i].tl).wrapping_mul(8).wrapping_add(u32::from(self.eg_saida[i])).min(1023);
        let fase10 = (self.fase[i] >> 10).wrapping_add(u32::try_from((m >> 1) & 0x3FF).unwrap_or(0)) & 0x3FF;
        self.op_anterior[i] = self.saida_op[i];
        self.portador_ant[i] = self.portador[i];
        // Portadores de cada algoritmo (diagramas do manual do YM2608), na ordem S1, S3, S2, S4.
        self.portador[i] = matches!((algo_antes[ch], i / 6), (_, 3) | (7, _) | (4, 2) | (5 | 6, 1 | 2));
        self.saida_op[i] = Self::operador(fase10, eg);
        self.eg_saida[i] = self.nivel[i];
        // A fase só avança depois de a saída usar o valor antigo.
        if self.zera_fase[i] {
            self.zera_fase[i] = false;
            self.fase[i] = 0;
        } else {
            self.fase[i] = self.fase[i].wrapping_add(self.incremento(i)) & 0xF_FFFF;
        }
        if i < 6 {
            self.realim[ch] = [self.saida_op[i], self.realim[ch][0]];
        }

        // Canal: soma um ciclo depois do slot do primeiro operador.
        let c6 = usize::try_from(c.wrapping_add(18) % 24).unwrap_or(99);
        if c6 < 6 {
            self.saida_canal[c6] = self.soma_canal(c6);
        }

        // Pino: cada canal aparece num ciclo próprio; fora deles, o repouso.
        let mut pino = (3i16, 3i16);
        for (k, &cp) in CICLO_PINO.iter().enumerate() {
            // O valor do canal e o panning são copiados três ciclos antes de aparecer (medido).
            if cp.wrapping_add(21) % 24 == c {
                let v = self.saida_canal[k];
                self.pino_valor[k] = v;
                self.pino_lr[k] = self.com_panning(k, v);
            }
            // O DAC não passa por essa cópia: o pino lê o valor ao vivo.
            let dac_vivo = k == 5 && self.dac_ligado;
            let falta = cp.wrapping_add(24).wrapping_sub(c) % 24;
            if (1..=3).contains(&falta) {
                // Nos três ciclos antes do valor, o pino mostra só o sinal do canal.
                let v = if dac_vivo { self.dac_valor } else { self.pino_valor[k] };
                let s = if v < 0 { -3 } else { 3 };
                pino = (s, s);
            }
            if cp == c {
                let (l, r) = if dac_vivo { self.com_panning(k, self.dac_valor) } else { self.pino_lr[k] };
                pino = (Self::escada(l), Self::escada(r));
            }
        }
        self.ciclo = c.wrapping_add(1) % 24;
        pino
    }

    /// Canal desligado de um lado deixa passar só o sinal (-1 ou 0).
    fn com_panning(&self, k: usize, v: i32) -> (i32, i32) {
        (if self.canal[k].esquerda { v } else { v >> 31 }, if self.canal[k].direita { v } else { v >> 31 })
    }

    /// Modulação que entra no slot `i`, segundo o algoritmo (diagramas do manual do YM2608).
    fn modulacao(&self, i: usize) -> i32 {
        let (g, ch) = (i / 6, i % 6);
        // Um operador só enxerga a saída do operador imediatamente anterior na ordem dos slots
        // com uma amostra de atraso (medido: 6 ciclos não bastam, 12 bastam).
        let s = |produtor: usize| {
            let j = produtor.wrapping_mul(6).wrapping_add(ch);
            if produtor < g && g.wrapping_sub(produtor) < 2 { self.op_anterior[j] } else { self.saida_op[j] }
        };
        let (s1, s3, s2) = (s(0), s(1), s(2));
        let a = self.c_algo[i];
        match g {
            0 => {
                let fb = self.c_fb[i];
                if fb == 0 { 0 } else { (self.realim[ch][0].wrapping_add(self.realim[ch][1])) >> 9_u32.wrapping_sub(u32::from(fb)) }
            }
            2 => match a {
                0 | 3..=6 => s1,
                _ => 0,
            },
            1 => match a {
                0 | 2 => s2,
                1 => s1.wrapping_add(s2),
                5 => s1,
                _ => 0,
            },
            _ => match a {
                0 | 1 | 4 => s3,
                2 => s1.wrapping_add(s3),
                3 => s2.wrapping_add(s3),
                5 => s1,
                _ => 0,
            },
        }
    }

    /// Soma dos operadores portadores, com saturação em 9 bits a cada parcela (medido).
    fn soma_canal(&self, ch: usize) -> i32 {
        let s = |g: usize| self.saida_op[g.wrapping_mul(6).wrapping_add(ch)];
        // O primeiro operador entra com o valor da amostra anterior.
        let (s1, s3, s2, s4) = (self.op_anterior[ch], s(1), s(2), s(3));
        let pt = |g: usize| self.portador[g.wrapping_mul(6).wrapping_add(ch)];
        let portadores = [self.portador_ant[ch], pt(1), pt(2), pt(3)];
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

    /// A tecla chega a cada slot com atraso de `3 + 6 * g` ciclos (o primeiro operador conta
    /// como 4). O slot vê o registrador `$28` como estava nesse instante passado; se o canal do
    /// registrador é o seu, copia o bit (`examples/sonda7.rs` e `sonda10.rs`).
    fn aplica_tecla(&mut self, slot: usize, agora: u64) {
        let (g, ch) = (slot / 6, slot % 6);
        let atraso = 3u64.wrapping_add(6u64.wrapping_mul(if g == 0 { 4 } else { u64::try_from(g).unwrap_or(0) }));
        // Bit do registrador de cada operador na ordem dos slots: S1, S3, S2, S4.
        let bit = [0usize, 2, 1, 3][g];
        if let Some(&(_, _, bits)) = self.tecla_hist.iter().rev().find(|e| e.1 == ch && e.0.wrapping_add(atraso) <= agora) {
            self.tecla_pedida[slot] = (bits >> bit) & 1 != 0;
        }
        // Guarda só o que ainda pode ser visto: as escritas recentes e a última antiga.
        if self.tecla_hist.len() > 1 {
            let limite = agora.saturating_sub(64);
            let antigas = self.tecla_hist.iter().filter(|e| e.0 < limite).count();
            if antigas > 1 {
                self.tecla_hist.drain(0..antigas.wrapping_sub(1));
            }
        }
    }

    /// Taxa efetiva (0 a 63) da fase atual do envelope do slot.
    fn taxa_atual(&self, i: usize) -> u32 {
        let ch = i % 6;
        let p = &self.op[ch][i / 6];
        let r5 = match self.eg_estado[i] {
            Eg::Ataque => u32::from(p.ar),
            Eg::Decai1 => u32::from(p.d1r),
            Eg::Decai2 => u32::from(p.d2r),
            Eg::Libera => u32::from(p.rr).wrapping_mul(2).wrapping_add(1),
        };
        if r5 == 0 {
            return 0;
        }
        let ksv = Self::kcode(self.canal[ch].fnum, self.canal[ch].bloco) >> 3_u32.wrapping_sub(u32::from(p.ks));
        r5.wrapping_mul(2).wrapping_add(ksv).min(63)
    }

    /// Passo do envelope: só age quando o contador mudou desde o último passo do slot.
    /// Tabelas de incremento do manual do YM2608; posição dentro do ciclo de 8 passos medida.
    fn passo_envelope(&mut self, i: usize) {
        // Os 16 últimos níveis viram silêncio total no passo seguinte (medido).
        if self.nivel[i] >= 1008 {
            self.nivel[i] = 1023;
        }
        if self.eg_visto[i] == self.eg_timer {
            return;
        }
        self.eg_visto[i] = self.eg_timer;
        // A fase do envelope é decidida com o nível do passo anterior, antes de escolher a taxa.
        let sl = u32::from(self.op[i % 6][i / 6].sl);
        let limite = if sl == 15 { 0x3E0 } else { sl << 5 };
        if self.eg_estado[i] == Eg::Ataque && self.nivel[i] == 0 {
            self.eg_estado[i] = Eg::Decai1;
        }
        if self.eg_estado[i] == Eg::Decai1 && u32::from(self.nivel[i]) >= limite {
            self.eg_estado[i] = Eg::Decai2;
        }
        let taxa = self.taxa_atual(i);
        if taxa == 0 {
            return;
        }
        let timer = u32::from(self.eg_timer);
        let inc = if taxa >= 48 {
            let pos = timer.wrapping_sub(if taxa & 3 == 3 { 3 } else { 1 }) & 7;
            u32::from(EG_INC_ALTO[taxa.wrapping_sub(48) as usize][pos as usize])
        } else {
            let sh = 11_u32.wrapping_sub(taxa >> 2);
            if timer & ((1u32 << sh).wrapping_sub(1)) != 0 {
                return;
            }
            u32::from(EG_INC_BAIXO[(taxa & 3) as usize][((timer >> sh) & 7) as usize])
        };
        let nivel = i32::from(self.nivel[i]);
        let inc_i = i32::try_from(inc).unwrap_or(0);
        let novo = if self.eg_estado[i] == Eg::Ataque {
            // Ataque: curva exponencial, o passo é proporcional ao que falta.
            nivel.wrapping_add((nivel.wrapping_neg().wrapping_sub(1).wrapping_mul(inc_i)) >> 4)
        } else {
            nivel.wrapping_add(inc_i)
        };
        self.nivel[i] = u16::try_from(novo.clamp(0, 1023)).unwrap_or(1023);
    }

    // Acesso de diagnóstico ao estado interno (para o comparador em camadas).
    pub fn diag_fase(&self) -> [u32; 24] {
        self.fase
    }
    pub fn diag_saida_op(&self) -> [i32; 24] {
        self.saida_op
    }
    pub fn diag_saida_canal(&self) -> [i32; 6] {
        self.saida_canal
    }
    pub fn diag_eg(&self) -> [u16; 24] {
        self.nivel
    }

}
