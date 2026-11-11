//! FM do Mega Drive (YM2612), em inteiros, um ciclo interno por chamada (mestre/42).
//!
//! Estrutura (manual do YM2608 e pesquisa publica sobre o YM2612): 6 canais de 4 operadores,
//! 24 "slots" processados um por ciclo, 24 ciclos por amostra. Cada slot passa pelo gerador de
//! fase, pelo envelope e pelo operador (log-seno mais exponencial). A cadencia de cada estagio e do
//! barramento replica a maquina de estados de silicio do chip.
//!
//! Slot `i` = operador (ordem de endereco: S1, S3, S2, S4) vezes 6, mais o canal.

use crate::tabelas::{EXP, LOGSIN};

/// Ciclos internos por amostra.
pub const CICLOS_POR_AMOSTRA: u32 = 24;

const OP_OFFSET: [u32; 12] = [
    0x000, 0x001, 0x002, 0x100, 0x101, 0x102, 0x004, 0x005, 0x006, 0x104, 0x105, 0x106,
];

const CH_OFFSET: [u32; 6] = [0x000, 0x001, 0x002, 0x100, 0x101, 0x102];

const FN_NOTE: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 3, 3, 3, 3, 3, 3];

const PG_DETUNE: [u32; 8] = [16, 17, 19, 20, 22, 24, 27, 29];

const EG_STEPHI: [[u32; 4]; 4] = [[0, 0, 0, 0], [1, 0, 0, 0], [1, 0, 1, 0], [1, 1, 1, 0]];

const FM_ALGORITHM: [[[bool; 8]; 6]; 4] = [
    [
        [true, true, true, true, true, true, true, true],
        [true, true, true, true, true, true, true, true],
        [false, false, false, false, false, false, false, false],
        [false, false, false, false, false, false, false, false],
        [false, false, false, false, false, false, false, false],
        [false, false, false, false, false, false, false, true],
    ],
    [
        [false, true, false, false, false, true, false, false],
        [false, false, false, false, false, false, false, false],
        [true, true, true, false, false, false, false, false],
        [false, false, false, false, false, false, false, false],
        [false, false, false, false, false, false, false, false],
        [false, false, false, false, false, true, true, true],
    ],
    [
        [false, false, false, false, false, false, false, false],
        [false, false, false, false, false, false, false, false],
        [false, false, false, false, false, false, false, false],
        [true, false, false, true, true, true, true, false],
        [false, false, false, false, false, false, false, false],
        [false, false, false, false, true, true, true, true],
    ],
    [
        [false, false, true, false, false, true, false, false],
        [false, false, false, false, false, false, false, false],
        [false, false, false, true, false, false, false, false],
        [true, true, false, true, true, false, false, false],
        [false, false, true, false, false, false, false, false],
        [true, true, true, true, true, true, true, true],
    ],
];

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
    cycles: u32,
    tempo: u64,

    // Barramento de escrita e registradores de controle.
    write_data: u32,
    write_a: u32,
    write_d: u32,
    write_a_en: bool,
    write_d_en: bool,
    write_fm_address: bool,
    write_fm_data: bool,
    write_fm_mode_a: u32,
    address: u32,
    data: u8,

    // Registradores de operador (por slot 0..23).
    multi: [u32; 24],
    dt: [u8; 24],
    tl: [u8; 24],
    ar: [u8; 24],
    ks: [u8; 24],
    dr: [u8; 24],
    am: [bool; 24],
    sr: [u8; 24],
    sl: [u8; 24],
    rr: [u8; 24],

    // Registradores de canal (por canal 0..5).
    fnum: [u16; 6],
    block: [u8; 6],
    kcode: [u8; 6],
    reg_a4: u8,
    connect: [u8; 6],
    fb: [u8; 6],
    pan_l: [bool; 6],
    pan_r: [bool; 6],

    // Key-On (registrador $28).
    mode_kon_operator: [bool; 4],
    mode_kon_channel: u8,
    mode_kon: [bool; 24],
    eg_kon_latch: [bool; 24],
    eg_kon: [bool; 24],

    // Gerador de fase.
    pg_fnum: u16,
    pg_block: u8,
    pg_kcode: u8,
    pg_inc: [u32; 24],
    pg_phase: [u32; 24],
    pg_reset: [bool; 24],

    // Gerador de envelope.
    eg_cycle: u32,
    eg_cycle_stop: bool,
    eg_shift: u32,
    eg_shift_lock: u32,
    eg_timer_low_lock: u32,
    eg_timer_inc: u32,
    eg_timer: u32,
    eg_quotient: u32,
    eg_rate: u8,
    eg_ksv: u8,
    eg_inc: u8,
    eg_ratemax: bool,
    eg_tl: [u8; 2],
    eg_sl: [u8; 2],
    eg_level: [u16; 24],
    eg_state: [Eg; 24],
    eg_out: [u16; 24],

    // Modulacao FM.
    fm_op1: [[i32; 2]; 6],
    fm_op2: [i32; 6],
    fm_mod: [i32; 24],
    fm_out: [i32; 24],

    // Acumulador de canais e saida nos pinos.
    ch_acc: [i32; 6],
    ch_out: [i32; 6],
    ch_lock: i32,
    ch_lock_l: bool,
    ch_lock_r: bool,
    dac_ligado: bool,
    dac_data: i16,

    /// Variante de calibração mantida para compatibilidade com a suite de testes.
    pub variante: u32,
}

impl Default for Fm {
    fn default() -> Self {
        Self::novo()
    }
}

impl Fm {
    pub fn novo() -> Self {
        Self {
            cycles: 0,
            tempo: 0,
            write_data: 0,
            write_a: 0,
            write_d: 0,
            write_a_en: false,
            write_d_en: false,
            write_fm_address: false,
            write_fm_data: false,
            write_fm_mode_a: 0,
            address: 0,
            data: 0,
            multi: [1; 24],
            dt: [0; 24],
            tl: [0; 24],
            ar: [0; 24],
            ks: [0; 24],
            dr: [0; 24],
            am: [false; 24],
            sr: [0; 24],
            sl: [0; 24],
            rr: [0; 24],
            fnum: [0; 6],
            block: [0; 6],
            kcode: [0; 6],
            reg_a4: 0,
            connect: [0; 6],
            fb: [0; 6],
            pan_l: [true; 6],
            pan_r: [true; 6],
            mode_kon_operator: [false; 4],
            mode_kon_channel: 0xFF,
            mode_kon: [false; 24],
            eg_kon_latch: [false; 24],
            eg_kon: [false; 24],
            pg_fnum: 0,
            pg_block: 0,
            pg_kcode: 0,
            pg_inc: [0; 24],
            pg_phase: [0; 24],
            pg_reset: [false; 24],
            eg_cycle: 0,
            eg_cycle_stop: false,
            eg_shift: 0,
            eg_shift_lock: 0,
            eg_timer_low_lock: 0,
            eg_timer_inc: 0,
            eg_timer: 0,
            eg_quotient: 0,
            eg_rate: 0,
            eg_ksv: 0,
            eg_inc: 0,
            eg_ratemax: false,
            eg_tl: [0; 2],
            eg_sl: [0; 2],
            eg_level: [0x3FF; 24],
            eg_state: [Eg::Libera; 24],
            eg_out: [0x3FF; 24],
            fm_op1: [[0; 2]; 6],
            fm_op2: [0; 6],
            fm_mod: [0; 24],
            fm_out: [0; 24],
            ch_acc: [0; 6],
            ch_out: [0; 6],
            ch_lock: 0,
            ch_lock_l: true,
            ch_lock_r: true,
            dac_ligado: false,
            dac_data: 0,
            variante: 0,
        }
    }

    /// Escreve numa porta do chip (0 e 2 = endereco, 1 e 3 = dado).
    pub fn escreve(&mut self, porta: u32, valor: u8) {
        let p = porta & 3;
        self.write_data = ((p << 7) & 0x100) | u32::from(valor);
        if p & 1 != 0 {
            self.write_d |= 1;
        } else {
            self.write_a |= 1;
        }
    }

    /// Executa um ciclo interno do chip e devolve a saida nos pinos (esquerda, direita).
    pub fn ciclo(&mut self) -> (i16, i16) {
        let c = self.cycles;
        self.eg_cycle = self.eg_cycle.wrapping_add(1);

        if c == 1 && self.eg_quotient == 2 {
            self.eg_shift_lock = if self.eg_cycle_stop {
                0
            } else {
                self.eg_shift.wrapping_add(1)
            };
            self.eg_timer_low_lock = self.eg_timer & 3;
        }

        match c {
            1 => {
                self.eg_quotient = self.eg_quotient.wrapping_add(1) % 3;
                self.eg_cycle = 0;
                self.eg_cycle_stop = true;
                self.eg_shift = 0;
                self.eg_timer_inc |= self.eg_quotient >> 1;
                self.eg_timer = self.eg_timer.wrapping_add(self.eg_timer_inc);
                self.eg_timer_inc = self.eg_timer >> 12;
                self.eg_timer &= 0xFFF;
            }
            13 => {
                self.eg_cycle = 0;
                self.eg_cycle_stop = true;
                self.eg_shift = 0;
                self.eg_timer = self.eg_timer.wrapping_add(self.eg_timer_inc);
                self.eg_timer_inc = self.eg_timer >> 12;
                self.eg_timer &= 0xFFF;
            }
            _ => {}
        }

        if self.eg_cycle < 12 && ((self.eg_timer >> self.eg_cycle) & 1 != 0) && self.eg_cycle_stop {
            self.eg_shift = self.eg_cycle;
            self.eg_cycle_stop = false;
        }

        // IO de escrita.
        self.write_a_en = (self.write_a & 3) == 1;
        self.write_d_en = (self.write_d & 3) == 1;
        self.write_a <<= 1;
        self.write_d <<= 1;

        // Key-On.
        let c_usize = usize::try_from(c).unwrap_or(0);
        let chan = c_usize % 6;
        self.eg_kon_latch[c_usize] = self.mode_kon[c_usize];
        if c == u32::from(self.mode_kon_channel) {
            self.mode_kon[chan] = self.mode_kon_operator[0];
            self.mode_kon[chan.wrapping_add(12)] = self.mode_kon_operator[1];
            self.mode_kon[chan.wrapping_add(6)] = self.mode_kon_operator[2];
            self.mode_kon[chan.wrapping_add(18)] = self.mode_kon_operator[3];
        }

        // Saida serial nos pinos.
        let mut channel_output = chan;
        if c < 12 {
            channel_output = channel_output.wrapping_add(1);
        }
        if c & 3 == 0 {
            self.ch_lock = self.ch_out[channel_output];
            self.ch_lock_l = self.pan_l[channel_output];
            self.ch_lock_r = self.pan_r[channel_output];
        }
        let out = if (c >> 2) == 1 && self.dac_ligado {
            i32::from(self.dac_data)
        } else {
            self.ch_lock
        };
        let out_en = (c & 3) == 3;
        let mut out_adj = out;
        let mut sign = out >> 8;
        if out >= 0 {
            out_adj = out_adj.wrapping_add(1);
            sign = sign.wrapping_add(1);
        }
        let mol = if self.ch_lock_l && out_en {
            out_adj
        } else {
            sign
        };
        let mor = if self.ch_lock_r && out_en {
            out_adj
        } else {
            sign
        };
        let mol = mol.wrapping_mul(3);
        let mor = mor.wrapping_mul(3);

        // Geracao de canal.
        let slot_ch = usize::try_from(c.wrapping_add(18) % 24).unwrap_or(0);
        let op_ch = slot_ch / 6;
        let mut acc = self.ch_acc[chan];
        if op_ch == 0 {
            acc = 0;
        }
        let mut add = 0i32;
        if FM_ALGORITHM[op_ch][5][usize::from(self.connect[chan])] {
            add = self.fm_out[slot_ch] >> 5;
        }
        let sum = acc.wrapping_add(add).clamp(-256, 255);
        if op_ch == 0 {
            self.ch_out[chan] = self.ch_acc[chan];
        }
        self.ch_acc[chan] = sum;

        // Modulacao FM e geracao do operador.
        let slot_prep = usize::try_from(c.wrapping_add(6) % 24).unwrap_or(0);
        let op_prep = slot_prep / 6;
        let connect = usize::from(self.connect[chan]);
        let prevslot = usize::try_from(c.wrapping_add(18) % 24).unwrap_or(0);

        let mut mod1 = 0i32;
        let mut mod2 = 0i32;
        if FM_ALGORITHM[op_prep][0][connect] {
            mod2 |= self.fm_op1[chan][0];
        }
        if FM_ALGORITHM[op_prep][1][connect] {
            mod1 |= self.fm_op1[chan][1];
        }
        if FM_ALGORITHM[op_prep][2][connect] {
            mod1 |= self.fm_op2[chan];
        }
        if FM_ALGORITHM[op_prep][3][connect] {
            mod2 |= self.fm_out[prevslot];
        }
        if FM_ALGORITHM[op_prep][4][connect] {
            mod1 |= self.fm_out[prevslot];
        }
        let mut m = mod1.wrapping_add(mod2);
        if op_prep == 0 {
            let fb = self.fb[chan];
            m = if fb == 0 {
                0
            } else {
                m >> 10_u32.wrapping_sub(u32::from(fb))
            };
        } else {
            m >>= 1;
        }
        self.fm_mod[slot_prep] = m;

        let slot_hist = usize::try_from(c.wrapping_add(18) % 24).unwrap_or(0);
        if slot_hist / 6 == 0 {
            self.fm_op1[chan][1] = self.fm_op1[chan][0];
            self.fm_op1[chan][0] = self.fm_out[slot_hist];
        }
        if slot_hist / 6 == 2 {
            self.fm_op2[chan] = self.fm_out[slot_hist];
        }

        let slot_gen = usize::try_from(c.wrapping_add(19) % 24).unwrap_or(0);
        let phase = (u32::try_from(self.fm_mod[slot_gen] & 0x3FF).unwrap_or(0))
            .wrapping_add(self.pg_phase[slot_gen] >> 10)
            & 0x3FF;
        let mut quarter = phase & 0xFF;
        if phase & 0x100 != 0 {
            quarter ^= 0xFF;
        }
        let mut level =
            u32::from(LOGSIN[quarter as usize]).wrapping_add(u32::from(self.eg_out[slot_gen]) << 2);
        if level > 0x1FFF {
            level = 0x1FFF;
        }
        let exp_val = u32::from(EXP[usize::try_from(level & 0xFF).unwrap_or(0)]);
        let sh = level >> 8;
        let mut output = if sh >= 13 {
            0i32
        } else {
            let v = ((exp_val | 0x400) << 2) >> sh;
            i32::try_from(v).unwrap_or(0)
        };
        if phase & 0x200 != 0 {
            output = output.wrapping_neg();
        }
        self.fm_out[slot_gen] = output;

        // Gerador de fase.
        let slot_mask = usize::try_from(c.wrapping_add(20) % 24).unwrap_or(0);
        if self.pg_reset[slot_mask] {
            self.pg_inc[slot_mask] = 0;
        }

        let slot_step = usize::try_from(c.wrapping_add(19) % 24).unwrap_or(0);
        if self.pg_reset[slot_step] {
            self.pg_phase[slot_step] = 0;
        }
        self.pg_phase[slot_step] =
            self.pg_phase[slot_step].wrapping_add(self.pg_inc[slot_step]) & 0xF_FFFF;

        let fnum = u32::from(self.pg_fnum) << 1;
        let mut basefreq = (fnum << self.pg_block) >> 2;
        let dt = self.dt[c_usize];
        let dt_l = dt & 3;
        if dt_l != 0 {
            let mut kcode = self.pg_kcode;
            if kcode > 0x1C {
                kcode = 0x1C;
            }
            let block = kcode >> 2;
            let note = kcode & 3;
            let dt_add = match dt_l {
                2 => 2,
                3 => 3,
                _ => 0,
            };
            let sum = block.wrapping_add(9).wrapping_add(dt_add);
            let sum_h = sum >> 1;
            let sum_l = sum & 1;
            let detune =
                PG_DETUNE[usize::from((sum_l << 2) | note)] >> 9_u32.wrapping_sub(u32::from(sum_h));
            if dt & 4 != 0 {
                basefreq = basefreq.wrapping_sub(detune);
            } else {
                basefreq = basefreq.wrapping_add(detune);
            }
        }
        basefreq &= 0x1_FFFF;
        self.pg_inc[c_usize] = (basefreq.wrapping_mul(self.multi[c_usize]) >> 1) & 0xF_FFFF;

        // Gerador de envelope.
        let slot_adsr = usize::try_from(c.wrapping_add(22) % 24).unwrap_or(0);
        let nkon = self.eg_kon_latch[slot_adsr];
        let okon = self.eg_kon[slot_adsr];
        self.pg_reset[slot_adsr] = nkon && !okon;
        let kon_event = nkon && !okon;
        let level = self.eg_level[slot_adsr];
        let eg_off = (level & 0x3F0) == 0x3F0;
        let mut nextstate = self.eg_state[slot_adsr];
        let mut nextlevel = i32::from(level);
        let mut inc = 0i32;

        if kon_event {
            nextstate = Eg::Ataque;
            if self.eg_ratemax {
                nextlevel = 0;
            } else if self.eg_state[slot_adsr] == Eg::Ataque
                && level != 0
                && self.eg_inc != 0
                && nkon
            {
                let lev_inv = (!i32::from(level)) << self.eg_inc;
                inc = lev_inv >> 5;
            }
        } else {
            match self.eg_state[slot_adsr] {
                Eg::Ataque => {
                    if level == 0 {
                        nextstate = Eg::Decai1;
                    } else if self.eg_inc != 0 && !self.eg_ratemax && nkon {
                        let lev_inv = (!i32::from(level)) << self.eg_inc;
                        inc = lev_inv >> 5;
                    }
                }
                Eg::Decai1 => {
                    let sl_target = u32::from(self.eg_sl[1]) << 1;
                    if u32::from(level >> 4) == sl_target {
                        nextstate = Eg::Decai2;
                    } else if !eg_off && self.eg_inc != 0 {
                        inc = 1i32 << (self.eg_inc.wrapping_sub(1));
                    }
                }
                Eg::Decai2 | Eg::Libera => {
                    if !eg_off && self.eg_inc != 0 {
                        inc = 1i32 << (self.eg_inc.wrapping_sub(1));
                    }
                }
            }
            if !nkon {
                nextstate = Eg::Libera;
            }
        }

        if !kon_event && self.eg_state[slot_adsr] != Eg::Ataque && eg_off {
            nextstate = Eg::Libera;
            nextlevel = 0x3FF;
        }

        nextlevel = nextlevel.wrapping_add(inc);
        self.eg_kon[slot_adsr] = self.eg_kon_latch[slot_adsr];
        self.eg_level[slot_adsr] = u16::try_from(nextlevel & 0x3FF).unwrap_or(0x3FF);
        self.eg_state[slot_adsr] = nextstate;

        let slot_eg_gen = usize::try_from(c.wrapping_add(23) % 24).unwrap_or(0);
        let mut eg_lev = self.eg_level[slot_eg_gen];
        eg_lev = eg_lev.wrapping_add(u16::from(self.eg_tl[0]) << 3);
        self.eg_out[slot_eg_gen] = eg_lev.min(1023);

        // Prepara incremento para o proximo ciclo a partir da taxa e ksv travados.
        let mut rate = (u32::from(self.eg_rate) << 1).wrapping_add(u32::from(self.eg_ksv));
        if rate > 0x3F {
            rate = 0x3F;
        }
        let mut prep_inc = 0u8;
        let sum = ((rate >> 2).wrapping_add(self.eg_shift_lock)) & 0x0F;
        if self.eg_rate != 0 && self.eg_quotient == 2 {
            if rate < 48 {
                match sum {
                    12 => prep_inc = 1,
                    13 => prep_inc = u8::try_from((rate >> 1) & 1).unwrap_or(0),
                    14 => prep_inc = u8::try_from(rate & 1).unwrap_or(0),
                    _ => {}
                }
            } else {
                let stephi = EG_STEPHI[(rate & 3) as usize][self.eg_timer_low_lock as usize];
                let v = stephi.wrapping_add(rate >> 2).wrapping_sub(11);
                prep_inc = u8::try_from(v.min(4)).unwrap_or(4);
            }
        }
        self.eg_inc = prep_inc;
        self.eg_ratemax = (rate >> 1) == 0x1F;

        // Trava taxa e ksv do slot atual (c_usize) para o ciclo seguinte.
        let rate_sel = if !self.eg_kon[c_usize] && self.eg_kon_latch[c_usize] {
            Eg::Ataque
        } else {
            self.eg_state[c_usize]
        };
        let eg_rate = match rate_sel {
            Eg::Ataque => self.ar[c_usize],
            Eg::Decai1 => self.dr[c_usize],
            Eg::Decai2 => self.sr[c_usize],
            Eg::Libera => (self.rr[c_usize] << 1) | 1,
        };
        self.eg_rate = eg_rate;
        self.eg_ksv = self.pg_kcode >> (self.ks[c_usize] ^ 3);

        self.eg_tl[1] = self.eg_tl[0];
        self.eg_tl[0] = self.tl[c_usize];
        self.eg_sl[1] = self.eg_sl[0];
        self.eg_sl[0] = self.sl[c_usize];

        // Trava F-Number e bloco para o canal do proximo ciclo.
        let next_chan = chan.wrapping_add(1) % 6;
        self.pg_fnum = self.fnum[next_chan];
        self.pg_block = self.block[next_chan];
        self.pg_kcode = self.kcode[next_chan];

        // Gravacao nos registradores do canal ou operador com o dado do barramento atual.
        if self.write_fm_data {
            let mut slot_wr = usize::try_from(c % 12).unwrap_or(0);
            if OP_OFFSET[slot_wr] == (self.address & 0x107) {
                if self.address & 8 != 0 {
                    slot_wr = slot_wr.wrapping_add(12);
                }
                let a = self.address & 0xF0;
                let d = self.data;
                match a {
                    0x30 => {
                        let mut mul = u32::from(d & 0x0F);
                        if mul == 0 {
                            mul = 1;
                        } else {
                            mul <<= 1;
                        }
                        self.multi[slot_wr] = mul;
                        self.dt[slot_wr] = (d >> 4) & 7;
                    }
                    0x40 => self.tl[slot_wr] = d & 0x7F,
                    0x50 => {
                        self.ar[slot_wr] = d & 0x1F;
                        self.ks[slot_wr] = (d >> 6) & 3;
                    }
                    0x60 => {
                        self.dr[slot_wr] = d & 0x1F;
                        self.am[slot_wr] = (d >> 7) & 1 != 0;
                    }
                    0x70 => self.sr[slot_wr] = d & 0x1F,
                    0x80 => {
                        self.rr[slot_wr] = d & 0x0F;
                        let mut sl = d >> 4;
                        sl |= (sl.wrapping_add(1)) & 0x10;
                        self.sl[slot_wr] = sl;
                    }
                    _ => {}
                }
            }
            if CH_OFFSET[chan] == (self.address & 0x103) {
                let a = self.address & 0xFC;
                let d = self.data;
                match a {
                    0xA0 => {
                        let f = u16::from(d) | (u16::from(self.reg_a4 & 7) << 8);
                        let b = (self.reg_a4 >> 3) & 7;
                        self.fnum[chan] = f;
                        self.block[chan] = b;
                        let note = FN_NOTE[usize::from((f >> 7) & 0x0F)];
                        self.kcode[chan] = (b << 2) | note;
                    }
                    0xA4 => self.reg_a4 = d,
                    0xB0 => {
                        self.connect[chan] = d & 7;
                        self.fb[chan] = (d >> 3) & 7;
                    }
                    0xB4 => {
                        self.pan_l[chan] = (d >> 7) & 1 != 0;
                        self.pan_r[chan] = (d >> 6) & 1 != 0;
                    }
                    _ => {}
                }
            }
        }

        // Atualizacao dos sinais do barramento de escrita e registradores de modo.
        if self.write_a_en || self.write_d_en {
            if self.write_a_en {
                self.write_fm_data = false;
            }
            if self.write_fm_address && self.write_d_en {
                self.write_fm_data = true;
            }
            if self.write_a_en {
                if (self.write_data & 0xF0) != 0 {
                    self.address = self.write_data;
                    self.write_fm_address = true;
                } else {
                    self.write_fm_address = false;
                }
            }
            if self.write_d_en && (self.write_data & 0x100) == 0 {
                match self.write_fm_mode_a {
                    0x28 => {
                        for i in 0..4 {
                            self.mode_kon_operator[i] =
                                (self.write_data >> (4_usize.wrapping_add(i))) & 1 != 0;
                        }
                        if (self.write_data & 3) == 3 {
                            self.mode_kon_channel = 0xFF;
                        } else {
                            let ch = (self.write_data & 3)
                                .wrapping_add(((self.write_data >> 2) & 1).wrapping_mul(3));
                            self.mode_kon_channel = u8::try_from(ch).unwrap_or(0xFF);
                        }
                    }
                    0x2A => {
                        let d = u8::try_from(self.write_data & 0xFF).unwrap_or(0);
                        let mut dac = i16::from(d ^ 0x80) << 1;
                        if dac >= 256 {
                            dac = dac.wrapping_sub(512);
                        }
                        self.dac_data = dac;
                    }
                    0x2B => {
                        self.dac_ligado = (self.write_data >> 7) & 1 != 0;
                    }
                    _ => {}
                }
            }
            if self.write_a_en {
                self.write_fm_mode_a = self.write_data & 0x1FF;
            }
        }
        if self.write_fm_data {
            self.data = u8::try_from(self.write_data & 0xFF).unwrap_or(0);
        }

        self.cycles = c.wrapping_add(1) % 24;
        self.tempo = self.tempo.wrapping_add(1);
        (
            i16::try_from(mol).unwrap_or(0),
            i16::try_from(mor).unwrap_or(0),
        )
    }

    // Acesso de diagnóstico ao estado interno (para o comparador diferencial).
    pub fn diag_fase(&self) -> [u32; 24] {
        self.pg_phase
    }
    pub fn diag_saida_op(&self) -> [i32; 24] {
        self.fm_out
    }
    pub fn diag_saida_canal(&self) -> [i32; 6] {
        self.ch_out
    }
    pub fn diag_eg(&self) -> [u16; 24] {
        self.eg_level
    }

    /// Soma dos 6 canais com panning (9 bits por canal com sinal, sem distorcao de repouso).
    pub fn saida_misturada(&self) -> (i16, i16) {
        let mut sl = 0i16;
        let mut sr = 0i16;
        for c in 0..6 {
            let val = if c == 5 && self.dac_ligado {
                self.dac_data
            } else {
                i16::try_from(self.ch_out[c]).unwrap_or(0)
            };
            if self.pan_l[c] {
                sl = sl.wrapping_add(val);
            }
            if self.pan_r[c] {
                sr = sr.wrapping_add(val);
            }
        }
        (sl, sr)
    }
}
