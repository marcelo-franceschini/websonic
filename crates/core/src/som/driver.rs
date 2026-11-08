//! Driver de som do 68000 (sub-etapa 6.3): a variante do SMPS que o Sonic 1 REV01 usa.
//!
//! Cada função cita o rótulo do disassembly que reproduz. A RAM do driver, as trilhas e a
//! `Memoria` seguem o original byte a byte. As tabelas (frequências, vozes, prioridades,
//! envelopes, endereços de trilhas) são lidas da ROM, nos endereços dos rótulos, em tempo de
//! execução. Nada é interpretado: cada rotina é lógica nativa.
//!
//! Registradores do original viram variáveis locais com o papel que têm. Os retornos que o
//! original adultera na pilha (`addq.w #4,sp` e `addq.w #8,sp`) viram valores de retorno
//! ([`Fluxo`] e `Option`).

use crate::decisao;
use crate::memoria::{Memoria, Tamanho};

// --- Endereços de hardware ---
const Z80_RAM: u32 = 0xA0_0000;
const Z80_BUS_REQUEST: u32 = 0xA1_1100;
const YM_A0: u32 = 0xA0_4000;
const YM_D0: u32 = 0xA0_4001;
const YM_A1: u32 = 0xA0_4002;
const YM_D1: u32 = 0xA0_4003;
pub const PSG_INPUT: u32 = 0xC0_0011;
const Z_DAC_STATUS: u32 = 0x1FFD;
const Z_DAC_SAMPLE: u32 = 0x1FFF;
const Z_TIMPANI_PITCH: u32 = 0xEA;

// --- Layout da RAM do driver (rótulos SMPS_RAM.* e SMPS_Track.*, conferidos em teste) ---
pub mod layout {
    pub const BASE: u32 = 0xFF_F000;
    // SMPS_RAM
    pub const V_SNDPRIO: u32 = 0x00;
    pub const V_MAIN_TEMPO_TIMEOUT: u32 = 0x01;
    pub const V_MAIN_TEMPO: u32 = 0x02;
    pub const F_PAUSEMUSIC: u32 = 0x03;
    pub const V_FADEOUT_COUNTER: u32 = 0x04;
    pub const V_FADEOUT_DELAY: u32 = 0x06;
    pub const V_COMMUNICATION_BYTE: u32 = 0x07;
    pub const F_UPDATING_DAC: u32 = 0x08;
    pub const V_SOUND_ID: u32 = 0x09;
    pub const V_SOUNDQUEUE0: u32 = 0x0A;
    pub const V_SOUNDQUEUE1: u32 = 0x0B;
    pub const V_SOUNDQUEUE2: u32 = 0x0C;
    pub const F_VOICE_SELECTOR: u32 = 0x0E;
    pub const V_VOICE_PTR: u32 = 0x18;
    pub const V_SPECIAL_VOICE_PTR: u32 = 0x20;
    pub const F_FADEIN_FLAG: u32 = 0x24;
    pub const V_FADEIN_DELAY: u32 = 0x25;
    pub const V_FADEIN_COUNTER: u32 = 0x26;
    pub const F_1UP_PLAYING: u32 = 0x27;
    pub const V_TEMPO_MOD: u32 = 0x28;
    pub const V_SPEEDUPTEMPO: u32 = 0x29;
    pub const F_SPEEDUP: u32 = 0x2A;
    pub const V_RING_SPEAKER: u32 = 0x2B;
    pub const F_PUSH_PLAYING: u32 = 0x2C;
    pub const V_MUSIC_DAC_TRACK: u32 = 0x40;
    pub const V_MUSIC_FM_TRACKS: u32 = 0x70;
    pub const V_MUSIC_FM4_TRACK: u32 = 0x100;
    pub const V_MUSIC_PSG_TRACKS: u32 = 0x190;
    pub const V_MUSIC_PSG3_TRACK: u32 = 0x1F0;
    pub const V_MUSIC_TRACK_RAM_END: u32 = 0x220;
    pub const V_1UP_RAM_END: u32 = 0x220;
    pub const V_SFX_TRACK_RAM: u32 = 0x220;
    pub const V_SFX_FM_TRACKS: u32 = 0x220;
    pub const V_SFX_FM4_TRACK: u32 = 0x250;
    pub const V_SFX_PSG_TRACKS: u32 = 0x2B0;
    pub const V_SFX_PSG3_TRACK: u32 = 0x310;
    pub const V_SPCSFX_FM4_TRACK: u32 = 0x340;
    pub const V_SPCSFX_PSG3_TRACK: u32 = 0x370;
    pub const V_1UP_RAM_COPY: u32 = 0x3A0;
    // SMPS_Track
    pub const T_LEN: u32 = 0x30;
    pub const T_PLAYBACK_CONTROL: u32 = 0x00;
    pub const T_VOICE_CONTROL: u32 = 0x01;
    pub const T_TEMPO_DIVIDER: u32 = 0x02;
    pub const T_DATA_POINTER: u32 = 0x04;
    pub const T_TRANSPOSE: u32 = 0x08;
    pub const T_VOLUME: u32 = 0x09;
    pub const T_AMSFMSPAN: u32 = 0x0A;
    pub const T_VOICE_INDEX: u32 = 0x0B;
    pub const T_VOL_ENV_INDEX: u32 = 0x0C;
    pub const T_STACK_POINTER: u32 = 0x0D;
    pub const T_DURATION_TIMEOUT: u32 = 0x0E;
    pub const T_SAVED_DURATION: u32 = 0x0F;
    pub const T_FREQ: u32 = 0x10;
    pub const T_SAVED_DAC: u32 = 0x10;
    pub const T_NOTE_TIMEOUT: u32 = 0x12;
    pub const T_NOTE_TIMEOUT_MASTER: u32 = 0x13;
    pub const T_MODULATION_PTR: u32 = 0x14;
    pub const T_MODULATION_WAIT: u32 = 0x18;
    pub const T_MODULATION_SPEED: u32 = 0x19;
    pub const T_MODULATION_DELTA: u32 = 0x1A;
    pub const T_MODULATION_STEPS: u32 = 0x1B;
    pub const T_MODULATION_VAL: u32 = 0x1C;
    pub const T_DETUNE: u32 = 0x1E;
    pub const T_FEEDBACK_ALGO: u32 = 0x1F;
    pub const T_PSG_NOISE: u32 = 0x1F;
    pub const T_VOICE_PTR: u32 = 0x20;
    pub const T_LOOP_COUNTERS: u32 = 0x24;
}
use layout::*;

// --- Endereços de tabelas na ROM (rótulos do disassembly) ---
pub const GO_SOUND_PRIORITIES: u32 = 0x7_1990;
pub const GO_SPEC_SOUND_INDEX: u32 = 0x7_1994;
pub const GO_MUSIC_INDEX: u32 = 0x7_1998;
pub const GO_SOUND_INDEX: u32 = 0x7_199C;
pub const GO_SPEEDUP_INDEX: u32 = 0x7_19A0;
pub const GO_PSG_INDEX: u32 = 0x7_19A4;
pub const FM_DAC_INIT_BYTES: u32 = 0x7_21BA;
pub const PSG_INIT_BYTES: u32 = 0x7_21C2;
pub const SFX_BGM_CHANNEL_RAM: u32 = 0x7_22CC;
pub const SFX_SFX_CHANNEL_RAM: u32 = 0x7_22EC;
pub const FM_FREQUENCIES: u32 = 0x7_2790;
pub const PSG_FREQUENCIES: u32 = 0x7_29CE;
pub const FM_SLOT_MASK: u32 = 0x7_2CAC;
pub const FM_INSTRUMENT_OPERATOR_TABLE: u32 = 0x7_2D18;
pub const FM_INSTRUMENT_OPERATOR_TABLE_END: u32 = 0x7_2D2C;
pub const FM_INSTRUMENT_TL_TABLE: u32 = 0x7_2D2C;
pub const FM_INSTRUMENT_TL_TABLE_END: u32 = 0x7_2D30;
pub const DAC_SAMPLE_RATE: u32 = 0x7_1CC4;

// --- Constantes de IDs (rótulos bgm_*, sfx_*, spec_*, flg_*) ---
const BGM_FIRST: u8 = 0x81;
const BGM_LAST: u8 = 0x93;
const BGM_EXTRA_LIFE: u8 = 0x88;
pub const SFX_FIRST: u8 = 0xA0;
pub const SFX_LAST: u8 = 0xCF;
pub const SFX_RING: u8 = 0xB5;
pub const SFX_RING_LEFT: u8 = 0xCE;
pub const SFX_PUSH: u8 = 0xA7;
const SPEC_FIRST: u8 = 0xD0;
const SPEC_LAST: u8 = 0xD0;
const FLG_FIRST: u8 = 0xE0;
const FLG_LAST: u8 = 0xE4;

/// O que uma rotina do original faz com o retorno ao chamador.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fluxo {
    /// Retorno normal.
    Normal,
    /// O original adultera a pilha e volta direto ao chamador de `UpdateMusic`: o resto da
    /// atualização não roda, nem o `startZ80` do fim.
    SairDoUpdate,
}

/// O que uma flag de coordenação faz com o retorno.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cf {
    Normal,
    /// `addq.w #8,sp`: sai da rotina que chamou a flag e da que a chamou.
    Sair8,
}

fn lo(v: u32) -> u8 {
    v.to_le_bytes()[0]
}

/// O driver sobre uma `Memoria`.
pub struct Driver<'a> {
    pub m: &'a mut Memoria,
    /// Quantas vezes um laço de espera chegou ao limite por falta de tempo gravado.
    pub esperas_estouradas: u64,
}

/// Voltas máximas de um laço de espera antes de desistir (só vale sem tempo gravado).
const LIMITE_ESPERA: u32 = 100_000;

impl<'a> Driver<'a> {
    pub fn novo(m: &'a mut Memoria) -> Self {
        Self {
            m,
            esperas_estouradas: 0,
        }
    }

    // --- Acesso à memória ---

    fn rb(&mut self, a: u32) -> u8 {
        self.m.ler(a, Tamanho::Byte).map_or(0, lo)
    }

    fn rw(&mut self, a: u32) -> u16 {
        u16::from_be_bytes([self.rb(a), self.rb(a.wrapping_add(1))])
    }

    fn rl(&mut self, a: u32) -> u32 {
        u32::from_be_bytes([
            self.rb(a),
            self.rb(a.wrapping_add(1)),
            self.rb(a.wrapping_add(2)),
            self.rb(a.wrapping_add(3)),
        ])
    }

    fn wb(&mut self, a: u32, v: u8) {
        let _ = self.m.escrever(a, Tamanho::Byte, u32::from(v));
    }

    fn ww(&mut self, a: u32, v: u16) {
        let [h, l] = v.to_be_bytes();
        self.wb(a, h);
        self.wb(a.wrapping_add(1), l);
    }

    fn wl(&mut self, a: u32, v: u32) {
        let b = v.to_be_bytes();
        for (i, x) in b.iter().enumerate() {
            self.wb(a.wrapping_add(u32::try_from(i).unwrap_or(0)), *x);
        }
    }

    fn ram(off: u32) -> u32 {
        BASE.wrapping_add(off)
    }

    fn bset(&mut self, a: u32, bit: u8) {
        let v = self.rb(a);
        self.wb(a, v | (1 << bit));
    }

    fn bclr(&mut self, a: u32, bit: u8) {
        let v = self.rb(a);
        self.wb(a, v & !(1 << bit));
    }

    fn btst(&mut self, a: u32, bit: u8) -> bool {
        self.rb(a) & (1 << bit) != 0
    }

    // --- Z80 ---

    /// `stopZ80`.
    fn stop_z80(&mut self) {
        let _ = self.m.escrever(Z80_BUS_REQUEST, Tamanho::Word, 0x100);
    }

    /// `startZ80`.
    fn start_z80(&mut self) {
        let _ = self.m.escrever(Z80_BUS_REQUEST, Tamanho::Word, 0);
    }

    // --- Escritas nos chips ---

    /// `WriteFMI`.
    fn write_fm_i(&mut self, d0: u8, d1: u8) {
        self.write_fm(YM_A0, YM_D0, d0, d1);
    }

    /// `WriteFMII`.
    fn write_fm_ii(&mut self, d0: u8, d1: u8) {
        self.write_fm(YM_A1, YM_D1, d0, d1);
    }

    /// Espera o chip ficar livre, escreve o registrador, espera de novo e escreve o dado.
    fn write_fm(&mut self, endereco: u32, dado: u32, d0: u8, d1: u8) {
        // Os laços leem o bit de ocupado, que vem do tempo gravado.
        self.esperar_ym_livre();
        self.wb(endereco, d0);
        self.esperar_ym_livre();
        self.wb(dado, d1);
    }

    /// Laço de espera do bit de ocupado do YM2612. Com o tempo gravado esgotado o último valor se
    /// repete; o limite evita travar o teste e é contado em `esperas_estouradas`.
    fn esperar_ym_livre(&mut self) {
        for _ in 0..LIMITE_ESPERA {
            if self.rb(YM_A0) & 0x80 == 0 {
                return;
            }
        }
        self.esperas_estouradas = self.esperas_estouradas.saturating_add(1);
    }

    /// `WriteFMIorII`.
    fn write_fm_i_or_ii(&mut self, a5: u32, d0: u8, d1: u8) {
        let vc = self.rb(a5.wrapping_add(T_VOICE_CONTROL));
        if decisao!("WriteFMIorII:parte_ii", vc & 4 != 0) {
            // WriteFMIIPart
            let d2 = vc & !4;
            self.write_fm_ii(d0.wrapping_add(d2), d1);
        } else {
            self.write_fm_i(d0.wrapping_add(vc), d1);
        }
    }

    /// `WriteFMIorIIMain`.
    fn write_fm_i_or_ii_main(&mut self, a5: u32, d0: u8, d1: u8) {
        if decisao!(
            "WriteFMIorIIMain:sfx_sobrepoe",
            self.btst(a5.wrapping_add(T_PLAYBACK_CONTROL), 2)
        ) {
            return;
        }
        self.write_fm_i_or_ii(a5, d0, d1);
    }

    fn psg(&mut self, v: u8) {
        self.wb(PSG_INPUT, v);
    }

    // --- Tabelas na ROM ---

    fn rom_ptr(&mut self, go: u32) -> u32 {
        self.rl(go) & 0xFF_FFFF
    }

    fn canal_bgm(&mut self, indice_x4: u32) -> u32 {
        self.rl(SFX_BGM_CHANNEL_RAM.wrapping_add(indice_x4)) & 0xFF_FFFF
    }

    fn canal_sfx(&mut self, indice_x4: u32) -> u32 {
        self.rl(SFX_SFX_CHANNEL_RAM.wrapping_add(indice_x4)) & 0xFF_FFFF
    }

    // =========================================================================================
    // UpdateMusic
    // =========================================================================================

    /// `UpdateMusic`: roda uma vez por VBlank.
    pub fn update_music(&mut self) {
        loop {
            self.stop_z80();
            // .updateloop: espera o BUSACK (bit 0 do byte lido em A11100 vira zero).
            let mut esperou = 0u32;
            while self.rb(Z80_BUS_REQUEST) & 1 != 0 {
                esperou = esperou.saturating_add(1);
                if esperou >= LIMITE_ESPERA {
                    self.esperas_estouradas = self.esperas_estouradas.saturating_add(1);
                    break;
                }
            }
            if decisao!(
                "UpdateMusic:dac_ocupado",
                self.rb(Z80_RAM + Z_DAC_STATUS) & 0x80 != 0
            ) {
                // O DAC ainda não aceita comandos: solta o Z80 e tenta de novo.
                self.start_z80();
                continue;
            }
            break;
        }
        self.wb(Self::ram(F_VOICE_SELECTOR), 0);
        let pausa = self.rb(Self::ram(F_PAUSEMUSIC));
        if decisao!("UpdateMusic:pausado", pausa != 0) {
            self.pause_music(pausa);
            return;
        }
        let t = self.rb(Self::ram(V_MAIN_TEMPO_TIMEOUT)).wrapping_sub(1);
        self.wb(Self::ram(V_MAIN_TEMPO_TIMEOUT), t);
        if decisao!("UpdateMusic:tempo_zerou", t == 0) {
            self.tempo_wait();
        }
        if decisao!(
            "UpdateMusic:fadeout",
            self.rb(Self::ram(V_FADEOUT_COUNTER)) != 0
        ) {
            self.do_fade_out();
        }
        if decisao!("UpdateMusic:fadein", self.rb(Self::ram(F_FADEIN_FLAG)) != 0) {
            self.do_fade_in();
        }
        // O original testa só `v_soundqueue0` e `v_soundqueue1` (a terceira fila é ignorada).
        if decisao!(
            "UpdateMusic:fila_cheia",
            self.rw(Self::ram(V_SOUNDQUEUE0)) != 0
        ) {
            self.cycle_sound_queue();
        }
        if decisao!(
            "UpdateMusic:som_novo",
            self.rb(Self::ram(V_SOUND_ID)) != 0x80
        ) && self.play_sound_id() == Fluxo::SairDoUpdate
        {
            return;
        }
        // Trilha de DAC.
        let mut a5 = Self::ram(V_MUSIC_DAC_TRACK);
        if decisao!("UpdateMusic:dac_tocando", self.btst(a5, 7))
            && self.dac_update_track(a5) == Fluxo::SairDoUpdate
        {
            return;
        }
        self.wb(Self::ram(F_UPDATING_DAC), 0);
        for _ in 0..6 {
            a5 = a5.wrapping_add(T_LEN);
            if decisao!("UpdateMusic:fm_tocando", self.btst(a5, 7)) {
                self.fm_update_track(a5);
            }
        }
        for _ in 0..3 {
            a5 = a5.wrapping_add(T_LEN);
            if decisao!("UpdateMusic:psg_tocando", self.btst(a5, 7)) {
                self.psg_update_track(a5);
            }
        }
        self.wb(Self::ram(F_VOICE_SELECTOR), 0x80);
        for _ in 0..3 {
            a5 = a5.wrapping_add(T_LEN);
            if decisao!("UpdateMusic:sfx_fm_tocando", self.btst(a5, 7)) {
                self.fm_update_track(a5);
            }
        }
        for _ in 0..3 {
            a5 = a5.wrapping_add(T_LEN);
            if decisao!("UpdateMusic:sfx_psg_tocando", self.btst(a5, 7)) {
                self.psg_update_track(a5);
            }
        }
        self.wb(Self::ram(F_VOICE_SELECTOR), 0x40);
        a5 = a5.wrapping_add(T_LEN);
        if decisao!("UpdateMusic:especial_fm_tocando", self.btst(a5, 7)) {
            self.fm_update_track(a5);
        }
        a5 = a5.wrapping_add(T_LEN);
        if decisao!("UpdateMusic:especial_psg_tocando", self.btst(a5, 7)) {
            self.psg_update_track(a5);
        }
        // DoStartZ80.
        self.start_z80();
    }

    // =========================================================================================
    // Trilha de DAC
    // =========================================================================================

    /// `DACUpdateTrack`.
    fn dac_update_track(&mut self, a5: u32) -> Fluxo {
        let t = self.rb(a5.wrapping_add(T_DURATION_TIMEOUT)).wrapping_sub(1);
        self.wb(a5.wrapping_add(T_DURATION_TIMEOUT), t);
        if decisao!("DACUpdateTrack:ainda_toca", t != 0) {
            return Fluxo::Normal;
        }
        self.wb(Self::ram(F_UPDATING_DAC), 0x80);
        let mut a4 = self.rl(a5.wrapping_add(T_DATA_POINTER));
        let mut d5;
        loop {
            d5 = self.rb(a4);
            a4 = a4.wrapping_add(1);
            if decisao!("DACUpdateTrack:flag_de_coordenacao", d5 >= 0xE0) {
                let (cf, novo_a4) = self.coord_flag(a5, a4, d5);
                a4 = novo_a4;
                if cf == Cf::Sair8 {
                    // Na trilha de DAC o `addq.w #8,sp` desfaz também o retorno de `UpdateMusic`.
                    return Fluxo::SairDoUpdate;
                }
                continue;
            }
            break;
        }
        let duracao_direta;
        if decisao!("DACUpdateTrack:e_amostra", d5 & 0x80 != 0) {
            self.wb(a5.wrapping_add(T_SAVED_DAC), d5);
            let proximo = self.rb(a4);
            a4 = a4.wrapping_add(1);
            d5 = proximo;
            if decisao!("DACUpdateTrack:segue_duracao", d5 & 0x80 == 0) {
                duracao_direta = true;
            } else {
                a4 = a4.wrapping_sub(1);
                let sd = self.rb(a5.wrapping_add(T_SAVED_DURATION));
                self.wb(a5.wrapping_add(T_DURATION_TIMEOUT), sd);
                duracao_direta = false;
            }
        } else {
            duracao_direta = true;
        }
        if duracao_direta {
            self.set_duration(a5, d5);
        }
        self.wl(a5.wrapping_add(T_DATA_POINTER), a4);
        if decisao!(
            "DACUpdateTrack:sobreposta",
            self.btst(a5.wrapping_add(T_PLAYBACK_CONTROL), 2)
        ) {
            return Fluxo::Normal;
        }
        let d0 = self.rb(a5.wrapping_add(T_SAVED_DAC));
        if decisao!("DACUpdateTrack:pausa", d0 == 0x80) {
            return Fluxo::Normal;
        }
        if decisao!("DACUpdateTrack:timpani", d0 & 8 != 0) {
            let i = u32::from(d0.wrapping_sub(0x88));
            let taxa = self.rb(DAC_SAMPLE_RATE.wrapping_add(i));
            self.wb(Z80_RAM + Z_TIMPANI_PITCH, taxa);
            self.wb(Z80_RAM + Z_DAC_SAMPLE, 0x83);
            return Fluxo::Normal;
        }
        self.wb(Z80_RAM + Z_DAC_SAMPLE, d0);
        Fluxo::Normal
    }

    // =========================================================================================
    // Trilhas de FM
    // =========================================================================================

    /// `FMUpdateTrack`.
    fn fm_update_track(&mut self, a5: u32) {
        let t = self.rb(a5.wrapping_add(T_DURATION_TIMEOUT)).wrapping_sub(1);
        self.wb(a5.wrapping_add(T_DURATION_TIMEOUT), t);
        if decisao!("FMUpdateTrack:nota_acabou", t == 0) {
            self.bclr(a5.wrapping_add(T_PLAYBACK_CONTROL), 4);
            if self.fm_do_next(a5) == Cf::Sair8 {
                return;
            }
            self.fm_prepare_note(a5);
            self.fm_note_on(a5);
            return;
        }
        // .notegoing
        if self.note_timeout_update(a5) {
            return;
        }
        if let Some(d6) = self.do_modulation(a5) {
            self.fm_update_freq(a5, d6);
        }
    }

    /// `FMDoNext`. Devolve `Sair8` se uma flag de coordenação encerrou a atualização da trilha.
    fn fm_do_next(&mut self, a5: u32) -> Cf {
        let mut a4 = self.rl(a5.wrapping_add(T_DATA_POINTER));
        self.bclr(a5.wrapping_add(T_PLAYBACK_CONTROL), 1);
        let mut d5;
        loop {
            d5 = self.rb(a4);
            a4 = a4.wrapping_add(1);
            if decisao!("FMDoNext:flag_de_coordenacao", d5 >= 0xE0) {
                let (cf, novo) = self.coord_flag(a5, a4, d5);
                a4 = novo;
                if cf == Cf::Sair8 {
                    return Cf::Sair8;
                }
                continue;
            }
            break;
        }
        self.fm_note_off(a5);
        if decisao!("FMDoNext:e_nota", d5 & 0x80 != 0) {
            self.fm_set_freq(a5, a4, d5);
            let proximo = self.rb(a4);
            a4 = a4.wrapping_add(1);
            d5 = proximo;
            if decisao!("FMDoNext:segue_duracao", d5 & 0x80 == 0) {
                self.set_duration(a5, d5);
            } else {
                a4 = a4.wrapping_sub(1);
            }
        } else {
            self.set_duration(a5, d5);
        }
        self.finish_track_update(a5, a4);
        Cf::Normal
    }

    /// `FMSetFreq`. Para a pausa, o original cai em `FinishTrackUpdate` (com o ponteiro atual) e
    /// volta a quem chamou.
    fn fm_set_freq(&mut self, a5: u32, a4: u32, d5: u8) {
        let d5 = d5.wrapping_sub(0x80);
        if decisao!("FMSetFreq:pausa", d5 == 0) {
            // TrackSetRest
            self.bset(a5.wrapping_add(T_PLAYBACK_CONTROL), 1);
            self.ww(a5.wrapping_add(T_FREQ), 0);
            self.finish_track_update(a5, a4);
            return;
        }
        let d5 = d5.wrapping_add(self.rb(a5.wrapping_add(T_TRANSPOSE)));
        let i = u32::from(d5 & 0x7F).wrapping_mul(2);
        let f = self.rw(FM_FREQUENCIES.wrapping_add(i));
        self.ww(a5.wrapping_add(T_FREQ), f);
    }

    /// `SetDuration`.
    fn set_duration(&mut self, a5: u32, d5: u8) {
        let mut d0 = d5;
        let mut d1 = self.rb(a5.wrapping_add(T_TEMPO_DIVIDER));
        loop {
            d1 = d1.wrapping_sub(1);
            if decisao!("SetDuration:fim_da_multiplicacao", d1 == 0) {
                break;
            }
            d0 = d0.wrapping_add(d5);
        }
        self.wb(a5.wrapping_add(T_SAVED_DURATION), d0);
        self.wb(a5.wrapping_add(T_DURATION_TIMEOUT), d0);
    }

    /// `FinishTrackUpdate`.
    fn finish_track_update(&mut self, a5: u32, a4: u32) {
        self.wl(a5.wrapping_add(T_DATA_POINTER), a4);
        let sd = self.rb(a5.wrapping_add(T_SAVED_DURATION));
        self.wb(a5.wrapping_add(T_DURATION_TIMEOUT), sd);
        if decisao!(
            "FinishTrackUpdate:nao_ataca",
            self.btst(a5.wrapping_add(T_PLAYBACK_CONTROL), 4)
        ) {
            return;
        }
        let nm = self.rb(a5.wrapping_add(T_NOTE_TIMEOUT_MASTER));
        self.wb(a5.wrapping_add(T_NOTE_TIMEOUT), nm);
        self.wb(a5.wrapping_add(T_VOL_ENV_INDEX), 0);
        if decisao!(
            "FinishTrackUpdate:modulacao",
            self.btst(a5.wrapping_add(T_PLAYBACK_CONTROL), 3)
        ) {
            let a0 = self.rl(a5.wrapping_add(T_MODULATION_PTR));
            let w = self.rb(a0);
            self.wb(a5.wrapping_add(T_MODULATION_WAIT), w);
            let s = self.rb(a0.wrapping_add(1));
            self.wb(a5.wrapping_add(T_MODULATION_SPEED), s);
            let d = self.rb(a0.wrapping_add(2));
            self.wb(a5.wrapping_add(T_MODULATION_DELTA), d);
            let passos = self.rb(a0.wrapping_add(3)) >> 1;
            self.wb(a5.wrapping_add(T_MODULATION_STEPS), passos);
            self.ww(a5.wrapping_add(T_MODULATION_VAL), 0);
        }
    }

    /// `NoteTimeoutUpdate`. Devolve `true` se o original adultera o retorno (a trilha termina aqui).
    fn note_timeout_update(&mut self, a5: u32) -> bool {
        if decisao!(
            "NoteTimeoutUpdate:sem_preenchimento",
            self.rb(a5.wrapping_add(T_NOTE_TIMEOUT)) == 0
        ) {
            return false;
        }
        let t = self.rb(a5.wrapping_add(T_NOTE_TIMEOUT)).wrapping_sub(1);
        self.wb(a5.wrapping_add(T_NOTE_TIMEOUT), t);
        if decisao!("NoteTimeoutUpdate:ainda_nao_acabou", t != 0) {
            return false;
        }
        self.bset(a5.wrapping_add(T_PLAYBACK_CONTROL), 1);
        if decisao!(
            "NoteTimeoutUpdate:e_psg",
            self.rb(a5.wrapping_add(T_VOICE_CONTROL)) & 0x80 != 0
        ) {
            self.psg_note_off(a5);
        } else {
            self.fm_note_off(a5);
        }
        true
    }

    /// `DoModulation`. Devolve `Some(frequência)` quando o original volta ao chamador com D6
    /// preenchido, e `None` quando o retorno é adulterado.
    fn do_modulation(&mut self, a5: u32) -> Option<u16> {
        if decisao!(
            "DoModulation:desligada",
            !self.btst(a5.wrapping_add(T_PLAYBACK_CONTROL), 3)
        ) {
            return None;
        }
        if decisao!(
            "DoModulation:espera",
            self.rb(a5.wrapping_add(T_MODULATION_WAIT)) != 0
        ) {
            let w = self.rb(a5.wrapping_add(T_MODULATION_WAIT)).wrapping_sub(1);
            self.wb(a5.wrapping_add(T_MODULATION_WAIT), w);
            return None;
        }
        let s = self.rb(a5.wrapping_add(T_MODULATION_SPEED)).wrapping_sub(1);
        self.wb(a5.wrapping_add(T_MODULATION_SPEED), s);
        if decisao!("DoModulation:velocidade_nao_zerou", s != 0) {
            return None;
        }
        let a0 = self.rl(a5.wrapping_add(T_MODULATION_PTR));
        let v = self.rb(a0.wrapping_add(1));
        self.wb(a5.wrapping_add(T_MODULATION_SPEED), v);
        if decisao!(
            "DoModulation:sem_passos",
            self.rb(a5.wrapping_add(T_MODULATION_STEPS)) == 0
        ) {
            let p = self.rb(a0.wrapping_add(3));
            self.wb(a5.wrapping_add(T_MODULATION_STEPS), p);
            let d = self.rb(a5.wrapping_add(T_MODULATION_DELTA)).wrapping_neg();
            self.wb(a5.wrapping_add(T_MODULATION_DELTA), d);
            return None;
        }
        let p = self.rb(a5.wrapping_add(T_MODULATION_STEPS)).wrapping_sub(1);
        self.wb(a5.wrapping_add(T_MODULATION_STEPS), p);
        let delta = i16::from(i8::from_ne_bytes([
            self.rb(a5.wrapping_add(T_MODULATION_DELTA))
        ]));
        let acumulado = self.rw(a5.wrapping_add(T_MODULATION_VAL));
        let d6 = acumulado.wrapping_add(u16::from_ne_bytes(delta.to_ne_bytes()));
        self.ww(a5.wrapping_add(T_MODULATION_VAL), d6);
        let freq = self.rw(a5.wrapping_add(T_FREQ));
        Some(d6.wrapping_add(freq))
    }

    /// `FMPrepareNote` (e, por queda, `FMUpdateFreq`).
    fn fm_prepare_note(&mut self, a5: u32) {
        if decisao!(
            "FMPrepareNote:em_pausa",
            self.btst(a5.wrapping_add(T_PLAYBACK_CONTROL), 1)
        ) {
            return;
        }
        let d6 = self.rw(a5.wrapping_add(T_FREQ));
        if decisao!("FMPrepareNote:frequencia_zero", d6 == 0) {
            // FMSetRest
            self.bset(a5.wrapping_add(T_PLAYBACK_CONTROL), 1);
            return;
        }
        self.fm_update_freq(a5, d6);
    }

    /// `FMUpdateFreq`.
    fn fm_update_freq(&mut self, a5: u32, d6: u16) {
        let detune = i16::from(i8::from_ne_bytes([self.rb(a5.wrapping_add(T_DETUNE))]));
        let d6 = d6.wrapping_add(u16::from_ne_bytes(detune.to_ne_bytes()));
        if decisao!(
            "FMUpdateFreq:sobreposta",
            self.btst(a5.wrapping_add(T_PLAYBACK_CONTROL), 2)
        ) {
            return;
        }
        let [alto, baixo] = d6.to_be_bytes();
        self.write_fm_i_or_ii(a5, 0xA4, alto);
        self.write_fm_i_or_ii(a5, 0xA0, baixo);
    }

    /// `FMNoteOn`.
    fn fm_note_on(&mut self, a5: u32) {
        if decisao!(
            "FMNoteOn:em_pausa",
            self.btst(a5.wrapping_add(T_PLAYBACK_CONTROL), 1)
        ) {
            return;
        }
        if decisao!(
            "FMNoteOn:sobreposta",
            self.btst(a5.wrapping_add(T_PLAYBACK_CONTROL), 2)
        ) {
            return;
        }
        let d1 = self.rb(a5.wrapping_add(T_VOICE_CONTROL)) | 0xF0;
        self.write_fm_i(0x28, d1);
    }

    /// `FMNoteOff`.
    fn fm_note_off(&mut self, a5: u32) {
        if decisao!(
            "FMNoteOff:nao_ataca",
            self.btst(a5.wrapping_add(T_PLAYBACK_CONTROL), 4)
        ) {
            return;
        }
        if decisao!(
            "FMNoteOff:sobreposta",
            self.btst(a5.wrapping_add(T_PLAYBACK_CONTROL), 2)
        ) {
            return;
        }
        self.send_fm_note_off(a5);
    }

    /// `SendFMNoteOff`.
    fn send_fm_note_off(&mut self, a5: u32) {
        let d1 = self.rb(a5.wrapping_add(T_VOICE_CONTROL));
        self.write_fm_i(0x28, d1);
    }

    // =========================================================================================
    // Trilhas de PSG
    // =========================================================================================

    /// `PSGUpdateTrack`.
    fn psg_update_track(&mut self, a5: u32) {
        let t = self.rb(a5.wrapping_add(T_DURATION_TIMEOUT)).wrapping_sub(1);
        self.wb(a5.wrapping_add(T_DURATION_TIMEOUT), t);
        if decisao!("PSGUpdateTrack:nota_acabou", t == 0) {
            self.bclr(a5.wrapping_add(T_PLAYBACK_CONTROL), 4);
            if self.psg_do_next(a5) == Cf::Sair8 {
                return;
            }
            self.psg_do_note_on(a5);
            self.psg_do_vol_fx(a5);
            return;
        }
        if self.note_timeout_update(a5) {
            return;
        }
        self.psg_update_vol_fx(a5);
        if let Some(d6) = self.do_modulation(a5) {
            self.psg_update_freq(a5, d6);
        }
    }

    /// `PSGDoNext`.
    fn psg_do_next(&mut self, a5: u32) -> Cf {
        self.bclr(a5.wrapping_add(T_PLAYBACK_CONTROL), 1);
        let mut a4 = self.rl(a5.wrapping_add(T_DATA_POINTER));
        let mut d5;
        loop {
            d5 = self.rb(a4);
            a4 = a4.wrapping_add(1);
            if decisao!("PSGDoNext:flag_de_coordenacao", d5 >= 0xE0) {
                let (cf, novo) = self.coord_flag(a5, a4, d5);
                a4 = novo;
                if cf == Cf::Sair8 {
                    return Cf::Sair8;
                }
                continue;
            }
            break;
        }
        if decisao!("PSGDoNext:e_nota", d5 & 0x80 != 0) {
            self.psg_set_freq(a5, a4, d5);
            let proximo = self.rb(a4);
            a4 = a4.wrapping_add(1);
            d5 = proximo;
            if decisao!("PSGDoNext:segue_duracao", d5 & 0x80 == 0) {
                self.set_duration(a5, d5);
            } else {
                a4 = a4.wrapping_sub(1);
            }
        } else {
            self.set_duration(a5, d5);
        }
        self.finish_track_update(a5, a4);
        Cf::Normal
    }

    /// `PSGSetFreq`: termina em `FinishTrackUpdate` com o ponteiro atual.
    fn psg_set_freq(&mut self, a5: u32, a4: u32, d5: u8) {
        if decisao!("PSGSetFreq:pausa", d5 < 0x81) {
            self.bset(a5.wrapping_add(T_PLAYBACK_CONTROL), 1);
            self.ww(a5.wrapping_add(T_FREQ), 0xFFFF);
            self.finish_track_update(a5, a4);
            self.psg_note_off(a5);
            return;
        }
        let d5 = d5
            .wrapping_sub(0x81)
            .wrapping_add(self.rb(a5.wrapping_add(T_TRANSPOSE)));
        let f = self.rw(PSG_FREQUENCIES.wrapping_add(u32::from(d5 & 0x7F).wrapping_mul(2)));
        self.ww(a5.wrapping_add(T_FREQ), f);
        self.finish_track_update(a5, a4);
    }

    /// `PSGDoNoteOn`.
    fn psg_do_note_on(&mut self, a5: u32) {
        let d6 = self.rw(a5.wrapping_add(T_FREQ));
        if decisao!("PSGDoNoteOn:frequencia_invalida", d6 & 0x8000 != 0) {
            // PSGSetRest
            self.bset(a5.wrapping_add(T_PLAYBACK_CONTROL), 1);
            return;
        }
        self.psg_update_freq(a5, d6);
    }

    /// `PSGUpdateFreq`.
    fn psg_update_freq(&mut self, a5: u32, d6: u16) {
        let detune = i16::from(i8::from_ne_bytes([self.rb(a5.wrapping_add(T_DETUNE))]));
        let d6 = d6.wrapping_add(u16::from_ne_bytes(detune.to_ne_bytes()));
        if decisao!(
            "PSGUpdateFreq:sobreposta",
            self.btst(a5.wrapping_add(T_PLAYBACK_CONTROL), 2)
        ) {
            return;
        }
        if decisao!(
            "PSGUpdateFreq:em_pausa",
            self.btst(a5.wrapping_add(T_PLAYBACK_CONTROL), 1)
        ) {
            return;
        }
        let mut d0 = self.rb(a5.wrapping_add(T_VOICE_CONTROL));
        if decisao!("PSGUpdateFreq:ruido", d0 == 0xE0) {
            d0 = 0xC0;
        }
        let baixo = lo(u32::from(d6)) & 0xF;
        d0 |= baixo;
        let d6 = lo(u32::from(d6 >> 4)) & 0x3F;
        self.psg(d0);
        self.psg(d6);
    }

    /// `PSGUpdateVolFX`.
    fn psg_update_vol_fx(&mut self, a5: u32) {
        if decisao!(
            "PSGUpdateVolFX:sem_tom",
            self.rb(a5.wrapping_add(T_VOICE_INDEX)) == 0
        ) {
            return;
        }
        self.psg_do_vol_fx(a5);
    }

    /// `PSGDoVolFX`.
    fn psg_do_vol_fx(&mut self, a5: u32) {
        let mut d6 = self.rb(a5.wrapping_add(T_VOLUME));
        let d0 = self.rb(a5.wrapping_add(T_VOICE_INDEX));
        if decisao!("PSGDoVolFX:sem_envelope", d0 == 0) {
            self.set_psg_volume(a5, d6);
            return;
        }
        let tabela = self.rom_ptr(GO_PSG_INDEX);
        let a0 =
            self.rl(tabela.wrapping_add(u32::from(d0.wrapping_sub(1)).wrapping_mul(4))) & 0xFF_FFFF;
        let indice = self.rb(a5.wrapping_add(T_VOL_ENV_INDEX));
        let valor = self.rb(a0.wrapping_add(u32::from(indice)));
        self.wb(a5.wrapping_add(T_VOL_ENV_INDEX), indice.wrapping_add(1));
        // O original só trata o terminador $80; outros valores negativos entram na soma.
        if decisao!("PSGDoVolFX:valor_negativo", valor & 0x80 != 0)
            && decisao!("PSGDoVolFX:terminador", valor == 0x80)
        {
            // VolEnvHold
            let i = self.rb(a5.wrapping_add(T_VOL_ENV_INDEX)).wrapping_sub(1);
            self.wb(a5.wrapping_add(T_VOL_ENV_INDEX), i);
            return;
        }
        d6 = d6.wrapping_add(valor);
        if decisao!("PSGDoVolFX:limite", d6 >= 0x10) {
            d6 = 0xF;
        }
        self.set_psg_volume(a5, d6);
    }

    /// `SetPSGVolume`.
    fn set_psg_volume(&mut self, a5: u32, d6: u8) {
        if decisao!(
            "SetPSGVolume:em_pausa",
            self.btst(a5.wrapping_add(T_PLAYBACK_CONTROL), 1)
        ) {
            return;
        }
        if decisao!(
            "SetPSGVolume:sobreposta",
            self.btst(a5.wrapping_add(T_PLAYBACK_CONTROL), 2)
        ) {
            return;
        }
        if decisao!(
            "SetPSGVolume:nao_ataca",
            self.btst(a5.wrapping_add(T_PLAYBACK_CONTROL), 4)
        ) {
            // PSGCheckNoteTimeout
            if self.rb(a5.wrapping_add(T_NOTE_TIMEOUT_MASTER)) != 0
                && self.rb(a5.wrapping_add(T_NOTE_TIMEOUT)) == 0
            {
                return;
            }
        }
        // PSGSendVolume
        let v = (d6 | self.rb(a5.wrapping_add(T_VOICE_CONTROL))).wrapping_add(0x10);
        self.psg(v);
    }

    /// `PSGNoteOff`.
    fn psg_note_off(&mut self, a5: u32) {
        if decisao!(
            "PSGNoteOff:sobreposta",
            self.btst(a5.wrapping_add(T_PLAYBACK_CONTROL), 2)
        ) {
            return;
        }
        self.send_psg_note_off(a5);
    }

    /// `SendPSGNoteOff`.
    fn send_psg_note_off(&mut self, a5: u32) {
        let d0 = self.rb(a5.wrapping_add(T_VOICE_CONTROL)) | 0x1F;
        self.psg(d0);
    }

    /// `PSGSilenceAll`.
    fn psg_silence_all(&mut self) {
        for v in [0x9F, 0xBF, 0xDF, 0xFF] {
            self.psg(v);
        }
    }

    // =========================================================================================
    // Vozes de FM
    // =========================================================================================

    /// Soma `indice * 25` ao ponteiro (o tamanho de uma voz).
    fn avancar_voz(a1: u32, indice: u16) -> u32 {
        a1.wrapping_add(u32::from(indice).wrapping_mul(25))
    }

    /// `SetVoice`: envia a voz `d0` apontada por `a1` à trilha `a5`.
    fn set_voice(&mut self, a5: u32, a1: u32, d0: u8) {
        let mut a1 = Self::avancar_voz(a1, u16::from(d0));
        let d1 = self.rb(a1);
        a1 = a1.wrapping_add(1);
        self.wb(a5.wrapping_add(T_FEEDBACK_ALGO), d1);
        self.write_fm_i_or_ii(a5, 0xB0, d1);
        let n_op = FM_INSTRUMENT_OPERATOR_TABLE_END - FM_INSTRUMENT_OPERATOR_TABLE;
        for i in 0..n_op {
            let d0 = self.rb(FM_INSTRUMENT_OPERATOR_TABLE.wrapping_add(i));
            let d1 = self.rb(a1);
            a1 = a1.wrapping_add(1);
            self.write_fm_i_or_ii(a5, d0, d1);
        }
        let mascara_alg = self.rb(FM_SLOT_MASK.wrapping_add(u32::from(d1_alg(d1))));
        let mut d4 = mascara_alg;
        let d3 = self.rb(a5.wrapping_add(T_VOLUME));
        let n_tl = FM_INSTRUMENT_TL_TABLE_END - FM_INSTRUMENT_TL_TABLE;
        for i in 0..n_tl {
            let d0 = self.rb(FM_INSTRUMENT_TL_TABLE.wrapping_add(i));
            let mut d1 = self.rb(a1);
            a1 = a1.wrapping_add(1);
            let carry = d4 & 1 != 0;
            d4 >>= 1;
            if decisao!("SetVoice:operador_na_mascara", carry) {
                d1 = d1.wrapping_add(d3);
            }
            self.write_fm_i_or_ii(a5, d0, d1);
        }
        let pan = self.rb(a5.wrapping_add(T_AMSFMSPAN));
        self.write_fm_i_or_ii(a5, 0xB4, pan);
    }

    /// `SendVoiceTL`.
    fn send_voice_tl(&mut self, a5: u32) {
        if decisao!(
            "SendVoiceTL:sobreposta",
            self.btst(a5.wrapping_add(T_PLAYBACK_CONTROL), 2)
        ) {
            return;
        }
        let d0 = self.rb(a5.wrapping_add(T_VOICE_INDEX));
        let seletor = self.rb(Self::ram(F_VOICE_SELECTOR));
        let mut a1 = self.rl(Self::ram(V_VOICE_PTR)) & 0xFF_FFFF;
        if decisao!("SendVoiceTL:seletor_nao_zero", seletor != 0) {
            // O original lê o ponteiro de voz relativo a A6 (o início da RAM do driver), que cai
            // em `v_special_voice_ptr`, e não relativo à trilha.
            a1 = self.rl(Self::ram(T_VOICE_PTR)) & 0xFF_FFFF;
            if decisao!("SendVoiceTL:seletor_negativo", seletor & 0x80 == 0) {
                a1 = self.rl(Self::ram(V_SPECIAL_VOICE_PTR)) & 0xFF_FFFF;
            }
        }
        a1 = Self::avancar_voz(a1, u16::from(d0));
        a1 = a1.wrapping_add(21);
        let algo = self.rb(a5.wrapping_add(T_FEEDBACK_ALGO)) & 7;
        let mut d4 = self.rb(FM_SLOT_MASK.wrapping_add(u32::from(algo)));
        let d3 = self.rb(a5.wrapping_add(T_VOLUME));
        if decisao!("SendVoiceTL:volume_negativo", d3 & 0x80 != 0) {
            return;
        }
        let n_tl = FM_INSTRUMENT_TL_TABLE_END - FM_INSTRUMENT_TL_TABLE;
        for i in 0..n_tl {
            let d0 = self.rb(FM_INSTRUMENT_TL_TABLE.wrapping_add(i));
            let d1 = self.rb(a1);
            a1 = a1.wrapping_add(1);
            let na_mascara = d4 & 1 != 0;
            d4 >>= 1;
            if !decisao!("SendVoiceTL:operador_na_mascara", na_mascara) {
                continue;
            }
            let (soma, estouro) = d1.overflowing_add(d3);
            if decisao!("SendVoiceTL:estouro", estouro) {
                continue;
            }
            self.write_fm_i_or_ii(a5, d0, soma);
        }
    }

    // =========================================================================================
    // Flags de coordenação
    // =========================================================================================

    /// `CoordFlag`: `d5` é o byte lido (`E0` a `FF`), `a4` o ponteiro depois dele. Devolve o que
    /// fazer com o retorno e o novo `a4`.
    fn coord_flag(&mut self, a5: u32, mut a4: u32, d5: u8) -> (Cf, u32) {
        match d5 {
            0xE0 => {
                // cfPanningAMSFMS
                let d1 = self.rb(a4);
                a4 = a4.wrapping_add(1);
                if decisao!(
                    "cfPanningAMSFMS:e_psg",
                    self.rb(a5.wrapping_add(T_VOICE_CONTROL)) & 0x80 != 0
                ) {
                    return (Cf::Normal, a4);
                }
                let d0 = self.rb(a5.wrapping_add(T_AMSFMSPAN)) & 0x37;
                let d1 = d1 | d0;
                self.wb(a5.wrapping_add(T_AMSFMSPAN), d1);
                self.write_fm_i_or_ii_main(a5, 0xB4, d1);
            }
            0xE1 => {
                let v = self.rb(a4);
                a4 = a4.wrapping_add(1);
                self.wb(a5.wrapping_add(T_DETUNE), v);
            }
            0xE2 => {
                let v = self.rb(a4);
                a4 = a4.wrapping_add(1);
                self.wb(Self::ram(V_COMMUNICATION_BYTE), v);
            }
            0xE3 => {
                // cfJumpReturn
                let sp = u32::from(self.rb(a5.wrapping_add(T_STACK_POINTER)));
                a4 = self.rl(a5.wrapping_add(sp));
                self.wl(a5.wrapping_add(sp), 0);
                a4 = a4.wrapping_add(2);
                self.wb(a5.wrapping_add(T_STACK_POINTER), lo(sp).wrapping_add(4));
            }
            0xE4 => return (Cf::Sair8, a4).and_then_fade(self, a5),
            0xE5 => {
                let v = self.rb(a4);
                a4 = a4.wrapping_add(1);
                self.wb(a5.wrapping_add(T_TEMPO_DIVIDER), v);
            }
            0xE6 => {
                // cfChangeFMVolume
                let d0 = self.rb(a4);
                a4 = a4.wrapping_add(1);
                let v = self.rb(a5.wrapping_add(T_VOLUME)).wrapping_add(d0);
                self.wb(a5.wrapping_add(T_VOLUME), v);
                self.send_voice_tl(a5);
            }
            0xE7 => self.bset(a5.wrapping_add(T_PLAYBACK_CONTROL), 4),
            0xE8 => {
                // cfNoteTimeout
                let v = self.rb(a4);
                a4 = a4.wrapping_add(1);
                self.wb(a5.wrapping_add(T_NOTE_TIMEOUT), v);
                self.wb(a5.wrapping_add(T_NOTE_TIMEOUT_MASTER), v);
            }
            0xE9 => {
                let d0 = self.rb(a4);
                a4 = a4.wrapping_add(1);
                let v = self.rb(a5.wrapping_add(T_TRANSPOSE)).wrapping_add(d0);
                self.wb(a5.wrapping_add(T_TRANSPOSE), v);
            }
            0xEA => {
                let v = self.rb(a4);
                a4 = a4.wrapping_add(1);
                self.wb(Self::ram(V_MAIN_TEMPO), v);
                self.wb(Self::ram(V_MAIN_TEMPO_TIMEOUT), v);
            }
            0xEB => {
                let d0 = self.rb(a4);
                a4 = a4.wrapping_add(1);
                for i in 0..10u32 {
                    self.wb(
                        Self::ram(V_MUSIC_DAC_TRACK)
                            .wrapping_add(i.wrapping_mul(T_LEN))
                            .wrapping_add(T_TEMPO_DIVIDER),
                        d0,
                    );
                }
            }
            0xEC => {
                let d0 = self.rb(a4);
                a4 = a4.wrapping_add(1);
                let v = self.rb(a5.wrapping_add(T_VOLUME)).wrapping_add(d0);
                self.wb(a5.wrapping_add(T_VOLUME), v);
            }
            0xED => self.wb(Self::ram(F_PUSH_PLAYING), 0),
            0xEE => {
                self.cf_stop_special_fm4(a5);
                return (Cf::Sair8, a4);
            }
            0xEF => {
                // cfSetVoice
                let d0 = self.rb(a4);
                a4 = a4.wrapping_add(1);
                self.wb(a5.wrapping_add(T_VOICE_INDEX), d0);
                if decisao!(
                    "cfSetVoice:sobreposta",
                    self.btst(a5.wrapping_add(T_PLAYBACK_CONTROL), 2)
                ) {
                    return (Cf::Normal, a4);
                }
                let seletor = self.rb(Self::ram(F_VOICE_SELECTOR));
                let mut a1 = self.rl(Self::ram(V_VOICE_PTR)) & 0xFF_FFFF;
                if decisao!("cfSetVoice:seletor_nao_zero", seletor != 0) {
                    a1 = self.rl(a5.wrapping_add(T_VOICE_PTR)) & 0xFF_FFFF;
                    if decisao!("cfSetVoice:seletor_negativo", seletor & 0x80 == 0) {
                        a1 = self.rl(Self::ram(V_SPECIAL_VOICE_PTR)) & 0xFF_FFFF;
                    }
                }
                self.set_voice(a5, a1, d0);
            }
            0xF0 => {
                // cfModulation
                self.bset(a5.wrapping_add(T_PLAYBACK_CONTROL), 3);
                self.wl(a5.wrapping_add(T_MODULATION_PTR), a4);
                let w = self.rb(a4);
                self.wb(a5.wrapping_add(T_MODULATION_WAIT), w);
                let s = self.rb(a4.wrapping_add(1));
                self.wb(a5.wrapping_add(T_MODULATION_SPEED), s);
                let d = self.rb(a4.wrapping_add(2));
                self.wb(a5.wrapping_add(T_MODULATION_DELTA), d);
                let p = self.rb(a4.wrapping_add(3)) >> 1;
                self.wb(a5.wrapping_add(T_MODULATION_STEPS), p);
                self.ww(a5.wrapping_add(T_MODULATION_VAL), 0);
                a4 = a4.wrapping_add(4);
            }
            0xF1 => self.bset(a5.wrapping_add(T_PLAYBACK_CONTROL), 3),
            0xF2 => {
                self.cf_stop_track(a5);
                return (Cf::Sair8, a4);
            }
            0xF3 => {
                // cfSetPSGNoise
                self.wb(a5.wrapping_add(T_VOICE_CONTROL), 0xE0);
                let v = self.rb(a4);
                a4 = a4.wrapping_add(1);
                self.wb(a5.wrapping_add(T_PSG_NOISE), v);
                if !decisao!(
                    "cfSetPSGNoise:sobreposta",
                    self.btst(a5.wrapping_add(T_PLAYBACK_CONTROL), 2)
                ) {
                    self.psg(v);
                }
            }
            0xF4 => self.bclr(a5.wrapping_add(T_PLAYBACK_CONTROL), 3),
            0xF5 => {
                let v = self.rb(a4);
                a4 = a4.wrapping_add(1);
                self.wb(a5.wrapping_add(T_VOICE_INDEX), v);
            }
            0xF6 => a4 = self.cf_jump_to(a4),
            0xF7 => {
                // cfRepeatAtPos
                let d0 = u32::from(self.rb(a4));
                let d1 = self.rb(a4.wrapping_add(1));
                a4 = a4.wrapping_add(2);
                let ctr = a5.wrapping_add(T_LOOP_COUNTERS).wrapping_add(d0);
                if decisao!("cfRepeatAtPos:laco_novo", self.rb(ctr) == 0) {
                    self.wb(ctr, d1);
                }
                let n = self.rb(ctr).wrapping_sub(1);
                self.wb(ctr, n);
                if decisao!("cfRepeatAtPos:repete", n != 0) {
                    a4 = self.cf_jump_to(a4);
                } else {
                    a4 = a4.wrapping_add(2);
                }
            }
            0xF8 => {
                // cfJumpToGosub
                let sp = self.rb(a5.wrapping_add(T_STACK_POINTER)).wrapping_sub(4);
                self.wl(a5.wrapping_add(u32::from(sp)), a4);
                self.wb(a5.wrapping_add(T_STACK_POINTER), sp);
                a4 = self.cf_jump_to(a4);
            }
            0xF9 => {
                // cfOpF9
                self.write_fm_i(0x88, 0x0F);
                self.write_fm_i(0x8C, 0x0F);
            }
            _ => {}
        }
        (Cf::Normal, a4)
    }

    /// `cfJumpTo`: lê o deslocamento de 16 bits e salta.
    fn cf_jump_to(&mut self, a4: u32) -> u32 {
        let alto = self.rb(a4);
        let baixo = self.rb(a4.wrapping_add(1));
        let a4 = a4.wrapping_add(2);
        let d = i16::from_be_bytes([alto, baixo]);
        let desloc = u32::from_ne_bytes(i32::from(d).to_ne_bytes());
        a4.wrapping_add(desloc).wrapping_sub(1)
    }

    /// `cfStopTrack`.
    fn cf_stop_track(&mut self, a5: u32) {
        self.bclr(a5.wrapping_add(T_PLAYBACK_CONTROL), 7);
        self.bclr(a5.wrapping_add(T_PLAYBACK_CONTROL), 4);
        let vc = self.rb(a5.wrapping_add(T_VOICE_CONTROL));
        if decisao!("cfStopTrack:e_psg", vc & 0x80 != 0) {
            self.psg_note_off(a5);
        } else {
            if decisao!(
                "cfStopTrack:e_dac",
                self.rb(Self::ram(F_UPDATING_DAC)) & 0x80 != 0
            ) {
                return;
            }
            self.fm_note_off(a5);
        }
        // .stoppedchannel
        if decisao!(
            "cfStopTrack:nao_e_sfx",
            self.rb(Self::ram(F_VOICE_SELECTOR)) & 0x80 == 0
        ) {
            return;
        }
        self.wb(Self::ram(V_SNDPRIO), 0);
        let d0 = self.rb(a5.wrapping_add(T_VOICE_CONTROL));
        if decisao!("cfStopTrack:canal_psg", d0 & 0x80 != 0) {
            // .getpsgptr
            let mut a0 = Self::ram(V_SPCSFX_PSG3_TRACK);
            let tocando = self.btst(a0, 7);
            let usa_especial =
                decisao!("cfStopTrack:especial_psg_tocando", tocando) && (d0 == 0xE0 || d0 == 0xC0);
            if !usa_especial {
                a0 = self.canal_bgm(u32::from(d0 >> 3));
            }
            self.bclr(a0.wrapping_add(T_PLAYBACK_CONTROL), 2);
            self.bset(a0.wrapping_add(T_PLAYBACK_CONTROL), 1);
            if self.rb(a0.wrapping_add(T_VOICE_CONTROL)) == 0xE0 {
                let n = self.rb(a0.wrapping_add(T_PSG_NOISE));
                self.psg(n);
            }
            return;
        }
        let (a5x, a1) = if decisao!("cfStopTrack:fm4", d0 == 4)
            && decisao!(
                "cfStopTrack:especial_fm4_tocando",
                self.btst(Self::ram(V_SPCSFX_FM4_TRACK), 7)
            ) {
            (
                Self::ram(V_SPCSFX_FM4_TRACK),
                self.rl(Self::ram(V_SPECIAL_VOICE_PTR)) & 0xFF_FFFF,
            )
        } else {
            let alvo = self.canal_bgm(u32::from(d0.wrapping_sub(2).wrapping_shl(2)));
            if !decisao!("cfStopTrack:trilha_tocando", self.btst(alvo, 7)) {
                return;
            }
            (alvo, self.rl(Self::ram(V_VOICE_PTR)) & 0xFF_FFFF)
        };
        self.bclr(a5x.wrapping_add(T_PLAYBACK_CONTROL), 2);
        self.bset(a5x.wrapping_add(T_PLAYBACK_CONTROL), 1);
        let vi = self.rb(a5x.wrapping_add(T_VOICE_INDEX));
        self.set_voice(a5x, a1, vi);
    }

    /// `cfStopSpecialFM4`.
    fn cf_stop_special_fm4(&mut self, a5: u32) {
        self.bclr(a5.wrapping_add(T_PLAYBACK_CONTROL), 7);
        self.bclr(a5.wrapping_add(T_PLAYBACK_CONTROL), 4);
        self.fm_note_off(a5);
        if decisao!(
            "cfStopSpecialFM4:sfx_usa_fm4",
            self.btst(Self::ram(V_SFX_FM4_TRACK), 7)
        ) {
            return;
        }
        let alvo = Self::ram(V_MUSIC_FM4_TRACK);
        let a1 = self.rl(Self::ram(V_VOICE_PTR)) & 0xFF_FFFF;
        self.bclr(alvo.wrapping_add(T_PLAYBACK_CONTROL), 2);
        self.bset(alvo.wrapping_add(T_PLAYBACK_CONTROL), 1);
        let vi = self.rb(alvo.wrapping_add(T_VOICE_INDEX));
        self.set_voice(alvo, a1, vi);
    }

    /// `cfFadeInToPrevious`.
    fn cf_fade_in_to_previous(&mut self) {
        // Restaura as variáveis e as trilhas de música da cópia.
        for i in 0..(V_1UP_RAM_END / 4) {
            let v = self.rl(Self::ram(V_1UP_RAM_COPY).wrapping_add(i.wrapping_mul(4)));
            self.wl(Self::ram(i.wrapping_mul(4)), v);
        }
        self.bset(Self::ram(V_MUSIC_DAC_TRACK) + T_PLAYBACK_CONTROL, 2);
        let d6 = 0x28u8.wrapping_sub(self.rb(Self::ram(V_FADEIN_COUNTER)));
        let mut a5 = Self::ram(V_MUSIC_FM_TRACKS);
        for _ in 0..6 {
            if decisao!("cfFadeInToPrevious:fm_tocando", self.btst(a5, 7)) {
                self.bset(a5.wrapping_add(T_PLAYBACK_CONTROL), 1);
                let v = self.rb(a5.wrapping_add(T_VOLUME)).wrapping_add(d6);
                self.wb(a5.wrapping_add(T_VOLUME), v);
                if !decisao!(
                    "cfFadeInToPrevious:fm_sobreposta",
                    self.btst(a5.wrapping_add(T_PLAYBACK_CONTROL), 2)
                ) {
                    let vi = self.rb(a5.wrapping_add(T_VOICE_INDEX));
                    let a1 = self.rl(Self::ram(V_VOICE_PTR)) & 0xFF_FFFF;
                    self.set_voice(a5, a1, vi);
                }
            }
            a5 = a5.wrapping_add(T_LEN);
        }
        for _ in 0..3 {
            if decisao!("cfFadeInToPrevious:psg_tocando", self.btst(a5, 7)) {
                self.bset(a5.wrapping_add(T_PLAYBACK_CONTROL), 1);
                self.psg_note_off(a5);
                let v = self.rb(a5.wrapping_add(T_VOLUME)).wrapping_add(d6);
                self.wb(a5.wrapping_add(T_VOLUME), v);
            }
            a5 = a5.wrapping_add(T_LEN);
        }
        self.wb(Self::ram(F_FADEIN_FLAG), 0x80);
        self.wb(Self::ram(V_FADEIN_COUNTER), 0x28);
        self.wb(Self::ram(F_1UP_PLAYING), 0);
        self.start_z80();
    }

    // =========================================================================================
    // Fila de sons, tocar música, efeitos e comandos
    // =========================================================================================

    /// `QueueSound1`.
    pub fn queue_sound1(&mut self, d0: u8) {
        self.wb(Self::ram(V_SOUNDQUEUE0), d0);
    }

    /// `QueueSound2`.
    pub fn queue_sound2(&mut self, d0: u8) {
        self.wb(Self::ram(V_SOUNDQUEUE1), d0);
    }

    /// `QueueSound3`.
    pub fn queue_sound3(&mut self, d0: u8) {
        self.wb(Self::ram(V_SOUNDQUEUE2), d0);
    }

    /// `CycleSoundQueue`.
    fn cycle_sound_queue(&mut self) {
        let a0 = self.rom_ptr(GO_SOUND_PRIORITIES);
        let mut d3 = self.rb(Self::ram(V_SNDPRIO));
        for i in 0..3u32 {
            let a1 = Self::ram(V_SOUNDQUEUE0).wrapping_add(i);
            let d1 = self.rb(a1);
            self.wb(a1, 0);
            if decisao!("CycleSoundQueue:nao_e_som", d1 < BGM_FIRST) {
                continue;
            }
            let d0 = d1.wrapping_sub(BGM_FIRST);
            if decisao!(
                "CycleSoundQueue:ocupado",
                self.rb(Self::ram(V_SOUND_ID)) != 0x80
            ) {
                self.wb(Self::ram(V_SOUNDQUEUE0), d1);
                continue;
            }
            let d2 = self.rb(a0.wrapping_add(u32::from(d0 & 0x7F)));
            if decisao!("CycleSoundQueue:prioridade_menor", d2 < d3) {
                continue;
            }
            d3 = d2;
            self.wb(Self::ram(V_SOUND_ID), d1);
        }
        if decisao!("CycleSoundQueue:prioridade_negativa", d3 & 0x80 != 0) {
            return;
        }
        self.wb(Self::ram(V_SNDPRIO), d3);
    }

    /// `PlaySoundID`.
    fn play_sound_id(&mut self) -> Fluxo {
        let d7 = self.rb(Self::ram(V_SOUND_ID));
        if decisao!("PlaySoundID:zero", d7 == 0) {
            self.stop_all_sound();
            return Fluxo::Normal;
        }
        if decisao!("PlaySoundID:positivo", d7 & 0x80 == 0) {
            return Fluxo::Normal;
        }
        self.wb(Self::ram(V_SOUND_ID), 0x80);
        // O original aceita músicas até $9F (o certo seria $93).
        if decisao!("PlaySoundID:e_musica", d7 <= BGM_LAST + 0x0C) {
            return self.sound_play_bgm(d7);
        }
        if decisao!("PlaySoundID:antes_dos_efeitos", d7 < SFX_FIRST) {
            return Fluxo::Normal;
        }
        if decisao!("PlaySoundID:e_efeito", d7 <= SFX_LAST) {
            self.sound_play_sfx(d7);
            return Fluxo::Normal;
        }
        if decisao!("PlaySoundID:antes_do_especial", d7 < SPEC_FIRST) {
            return Fluxo::Normal;
        }
        // O original aceita especiais até $DF (o certo seria $D0).
        if decisao!("PlaySoundID:e_especial", d7 < SPEC_LAST + 0x10) {
            self.sound_play_special(d7);
            return Fluxo::Normal;
        }
        if decisao!("PlaySoundID:e_comando", d7 <= FLG_LAST) {
            return match d7.wrapping_sub(FLG_FIRST) {
                0 => {
                    self.fade_out_music();
                    Fluxo::Normal
                }
                1 => self.play_sega_sound(),
                2 => {
                    self.speed_up_music();
                    Fluxo::Normal
                }
                3 => {
                    self.slow_down_music();
                    Fluxo::Normal
                }
                _ => {
                    self.stop_all_sound();
                    Fluxo::Normal
                }
            };
        }
        Fluxo::Normal
    }

    /// `PlaySegaSound`. O laço de espera de cerca de 2 segundos é tempo gravado: sem efeito
    /// observável aqui.
    fn play_sega_sound(&mut self) -> Fluxo {
        self.wb(Z80_RAM + Z_DAC_SAMPLE, 0x88);
        self.start_z80();
        Fluxo::SairDoUpdate
    }

    /// `Sound_PlayBGM`.
    fn sound_play_bgm(&mut self, d7: u8) -> Fluxo {
        if decisao!("Sound_PlayBGM:vida_extra", d7 == BGM_EXTRA_LIFE) {
            if decisao!(
                "Sound_PlayBGM:vida_extra_ja_toca",
                self.rb(Self::ram(F_1UP_PLAYING)) != 0
            ) {
                return Fluxo::SairDoUpdate;
            }
            for i in 0..10u32 {
                self.bclr(
                    Self::ram(V_MUSIC_DAC_TRACK)
                        .wrapping_add(i.wrapping_mul(T_LEN))
                        .wrapping_add(T_PLAYBACK_CONTROL),
                    2,
                );
            }
            for i in 0..6u32 {
                self.bclr(
                    Self::ram(V_SFX_TRACK_RAM)
                        .wrapping_add(i.wrapping_mul(T_LEN))
                        .wrapping_add(T_PLAYBACK_CONTROL),
                    7,
                );
            }
            self.wb(Self::ram(V_SNDPRIO), 0);
            for i in 0..(V_1UP_RAM_END / 4) {
                let v = self.rl(Self::ram(i.wrapping_mul(4)));
                self.wl(Self::ram(V_1UP_RAM_COPY).wrapping_add(i.wrapping_mul(4)), v);
            }
            self.wb(Self::ram(F_1UP_PLAYING), 0x80);
            self.wb(Self::ram(V_SNDPRIO), 0);
        } else {
            self.wb(Self::ram(F_1UP_PLAYING), 0);
            self.wb(Self::ram(V_FADEIN_COUNTER), 0);
        }
        // .bgm_loadMusic
        self.init_music_playback();
        let speedup = self.rom_ptr(GO_SPEEDUP_INDEX);
        let d7 = d7.wrapping_sub(BGM_FIRST);
        let sp = self.rb(speedup.wrapping_add(u32::from(d7)));
        self.wb(Self::ram(V_SPEEDUPTEMPO), sp);
        let indice = self.rom_ptr(GO_MUSIC_INDEX);
        let a4 = self.rl(indice.wrapping_add(u32::from(d7).wrapping_mul(4))) & 0xFF_FFFF;
        let a3 = a4;
        let voz = u32::from(self.rw(a4));
        self.wl(Self::ram(V_VOICE_PTR), voz.wrapping_add(a4));
        let mut d0 = self.rb(a4.wrapping_add(5));
        self.wb(Self::ram(V_TEMPO_MOD), d0);
        if decisao!(
            "Sound_PlayBGM:sapatos_de_velocidade",
            self.rb(Self::ram(F_SPEEDUP)) != 0
        ) {
            d0 = self.rb(Self::ram(V_SPEEDUPTEMPO));
        }
        self.wb(Self::ram(V_MAIN_TEMPO), d0);
        self.wb(Self::ram(V_MAIN_TEMPO_TIMEOUT), d0);
        let mut a4 = a4.wrapping_add(6);
        let n_fm = self.rb(a3.wrapping_add(2));
        let d4 = self.rb(a3.wrapping_add(4));
        let d6 = T_LEN;
        let d5 = 1u8;
        if decisao!("Sound_PlayBGM:tem_fm", n_fm != 0) {
            let mut a1 = Self::ram(V_MUSIC_DAC_TRACK);
            let mut a2 = FM_DAC_INIT_BYTES;
            for _ in 0..n_fm {
                self.bset(a1.wrapping_add(T_PLAYBACK_CONTROL), 7);
                let vc = self.rb(a2);
                a2 = a2.wrapping_add(1);
                self.wb(a1.wrapping_add(T_VOICE_CONTROL), vc);
                self.wb(a1.wrapping_add(T_TEMPO_DIVIDER), d4);
                self.wb(a1.wrapping_add(T_STACK_POINTER), lo(d6));
                self.wb(a1.wrapping_add(T_AMSFMSPAN), 0xC0);
                self.wb(a1.wrapping_add(T_DURATION_TIMEOUT), d5);
                let rel = u32::from(self.rw(a4));
                self.wl(a1.wrapping_add(T_DATA_POINTER), rel.wrapping_add(a3));
                let tr = self.rw(a4.wrapping_add(2));
                self.ww(a1.wrapping_add(T_TRANSPOSE), tr);
                a4 = a4.wrapping_add(4);
                a1 = a1.wrapping_add(d6);
            }
            if decisao!("Sound_PlayBGM:sete_fm", self.rb(a3.wrapping_add(2)) == 7) {
                self.write_fm_i(0x2B, 0);
            } else {
                self.write_fm_i(0x28, 6);
                for (reg, val) in [
                    (0x42, 0x7F),
                    (0x4A, 0x7F),
                    (0x46, 0x7F),
                    (0x4E, 0x7F),
                    (0xB6, 0xC0),
                ] {
                    self.write_fm_ii(reg, val);
                }
            }
        }
        // .bgm_fmdone
        let n_psg = self.rb(a3.wrapping_add(3));
        if decisao!("Sound_PlayBGM:tem_psg", n_psg != 0) {
            let mut a1 = Self::ram(V_MUSIC_PSG_TRACKS);
            let mut a2 = PSG_INIT_BYTES;
            for _ in 0..n_psg {
                self.bset(a1.wrapping_add(T_PLAYBACK_CONTROL), 7);
                let vc = self.rb(a2);
                a2 = a2.wrapping_add(1);
                self.wb(a1.wrapping_add(T_VOICE_CONTROL), vc);
                self.wb(a1.wrapping_add(T_TEMPO_DIVIDER), d4);
                self.wb(a1.wrapping_add(T_STACK_POINTER), lo(d6));
                self.wb(a1.wrapping_add(T_DURATION_TIMEOUT), d5);
                let rel = u32::from(self.rw(a4));
                self.wl(a1.wrapping_add(T_DATA_POINTER), rel.wrapping_add(a3));
                let tr = self.rw(a4.wrapping_add(2));
                self.ww(a1.wrapping_add(T_TRANSPOSE), tr);
                let tom = self.rb(a4.wrapping_add(5));
                self.wb(a1.wrapping_add(T_VOICE_INDEX), tom);
                a4 = a4.wrapping_add(6);
                a1 = a1.wrapping_add(d6);
            }
        }
        // .bgm_psgdone: as trilhas de efeito tomam os canais da música.
        for i in 0..6u32 {
            let a1 = Self::ram(V_SFX_TRACK_RAM).wrapping_add(i.wrapping_mul(T_LEN));
            if !decisao!("Sound_PlayBGM:sfx_tocando", self.btst(a1, 7)) {
                continue;
            }
            let vc = self.rb(a1.wrapping_add(T_VOICE_CONTROL));
            let d0 = if decisao!("Sound_PlayBGM:sfx_psg", vc & 0x80 != 0) {
                vc >> 3
            } else {
                vc.wrapping_sub(2).wrapping_shl(2)
            };
            let a0 = self.canal_bgm(u32::from(d0));
            self.bset(a0.wrapping_add(T_PLAYBACK_CONTROL), 2);
        }
        if decisao!(
            "Sound_PlayBGM:especial_fm4",
            self.btst(Self::ram(V_SPCSFX_FM4_TRACK), 7)
        ) {
            self.bset(Self::ram(V_MUSIC_FM4_TRACK) + T_PLAYBACK_CONTROL, 2);
        }
        if decisao!(
            "Sound_PlayBGM:especial_psg3",
            self.btst(Self::ram(V_SPCSFX_PSG3_TRACK), 7)
        ) {
            self.bset(Self::ram(V_MUSIC_PSG3_TRACK) + T_PLAYBACK_CONTROL, 2);
        }
        let mut a5 = Self::ram(V_MUSIC_FM_TRACKS);
        for _ in 0..6 {
            self.fm_note_off(a5);
            a5 = a5.wrapping_add(d6);
        }
        for _ in 0..3 {
            self.psg_note_off(a5);
            a5 = a5.wrapping_add(d6);
        }
        Fluxo::SairDoUpdate
    }

    /// `Sound_PlaySFX`.
    fn sound_play_sfx(&mut self, d7: u8) {
        let limpa = |s: &mut Self| s.wb(Self::ram(V_SNDPRIO), 0);
        if decisao!(
            "Sound_PlaySFX:vida_extra_toca",
            self.rb(Self::ram(F_1UP_PLAYING)) != 0
        ) {
            limpa(self);
            return;
        }
        if decisao!(
            "Sound_PlaySFX:fadeout",
            self.rb(Self::ram(V_FADEOUT_COUNTER)) != 0
        ) {
            limpa(self);
            return;
        }
        if decisao!(
            "Sound_PlaySFX:fadein",
            self.rb(Self::ram(F_FADEIN_FLAG)) != 0
        ) {
            limpa(self);
            return;
        }
        let mut d7 = d7;
        if decisao!("Sound_PlaySFX:e_anel", d7 == SFX_RING) {
            if decisao!(
                "Sound_PlaySFX:alto_falante_direito",
                self.rb(Self::ram(V_RING_SPEAKER)) == 0
            ) {
                d7 = SFX_RING_LEFT;
            }
            let v = self.rb(Self::ram(V_RING_SPEAKER)) ^ 1;
            self.wb(Self::ram(V_RING_SPEAKER), v);
        }
        if decisao!("Sound_PlaySFX:e_empurrar", d7 == SFX_PUSH) {
            if decisao!(
                "Sound_PlaySFX:empurrar_ja_toca",
                self.rb(Self::ram(F_PUSH_PLAYING)) != 0
            ) {
                return;
            }
            self.wb(Self::ram(F_PUSH_PLAYING), 0x80);
        }
        let a0 = self.rom_ptr(GO_SOUND_INDEX);
        let idx = u32::from(d7.wrapping_sub(SFX_FIRST)).wrapping_mul(4);
        let a3 = self.rl(a0.wrapping_add(idx)) & 0xFF_FFFF;
        let mut a1 = a3;
        let d1 = u32::from(self.rw(a1)).wrapping_add(a3);
        a1 = a1.wrapping_add(2);
        let d5 = self.rb(a1);
        let n = self.rb(a1.wrapping_add(1));
        a1 = a1.wrapping_add(2);
        for _ in 0..n {
            let d4 = self.rb(a1.wrapping_add(1));
            let d3: u32;
            if decisao!("Sound_PlaySFX:canal_psg", d4 & 0x80 != 0) {
                d3 = u32::from(d4) >> 3;
                let a5 = self.canal_bgm(d3);
                self.bset(a5.wrapping_add(T_PLAYBACK_CONTROL), 2);
                if decisao!("Sound_PlaySFX:psg3", d4 == 0xC0) {
                    let d0 = d4 | 0x1F;
                    self.psg(d0);
                    self.psg(d0 ^ 0x20);
                }
            } else {
                d3 = u32::from(d4.wrapping_sub(2)) << 2;
                let a5 = self.canal_bgm(d3);
                self.bset(a5.wrapping_add(T_PLAYBACK_CONTROL), 2);
            }
            let a5 = self.canal_sfx(d3);
            for k in 0..(T_LEN / 4) {
                self.wl(a5.wrapping_add(k.wrapping_mul(4)), 0);
            }
            let pc = self.rw(a1);
            self.ww(a5.wrapping_add(T_PLAYBACK_CONTROL), pc);
            a1 = a1.wrapping_add(2);
            self.wb(a5.wrapping_add(T_TEMPO_DIVIDER), d5);
            let rel = u32::from(self.rw(a1));
            self.wl(a5.wrapping_add(T_DATA_POINTER), rel.wrapping_add(a3));
            a1 = a1.wrapping_add(2);
            let tr = self.rw(a1);
            self.ww(a5.wrapping_add(T_TRANSPOSE), tr);
            a1 = a1.wrapping_add(2);
            self.wb(a5.wrapping_add(T_DURATION_TIMEOUT), 1);
            self.wb(a5.wrapping_add(T_STACK_POINTER), lo(T_LEN));
            if !decisao!("Sound_PlaySFX:inicio_psg", d4 & 0x80 != 0) {
                self.wb(a5.wrapping_add(T_AMSFMSPAN), 0xC0);
                self.wl(a5.wrapping_add(T_VOICE_PTR), d1);
            }
        }
        if decisao!(
            "Sound_PlaySFX:sfx_fm4_tocando",
            self.btst(Self::ram(V_SFX_FM4_TRACK), 7)
        ) {
            self.bset(Self::ram(V_SPCSFX_FM4_TRACK) + T_PLAYBACK_CONTROL, 2);
        }
        if decisao!(
            "Sound_PlaySFX:sfx_psg3_tocando",
            self.btst(Self::ram(V_SFX_PSG3_TRACK), 7)
        ) {
            self.bset(Self::ram(V_SPCSFX_PSG3_TRACK) + T_PLAYBACK_CONTROL, 2);
        }
    }

    /// `Sound_PlaySpecial`.
    fn sound_play_special(&mut self, d7: u8) {
        if decisao!(
            "Sound_PlaySpecial:vida_extra_toca",
            self.rb(Self::ram(F_1UP_PLAYING)) != 0
        ) {
            return;
        }
        if decisao!(
            "Sound_PlaySpecial:fadeout",
            self.rb(Self::ram(V_FADEOUT_COUNTER)) != 0
        ) {
            return;
        }
        if decisao!(
            "Sound_PlaySpecial:fadein",
            self.rb(Self::ram(F_FADEIN_FLAG)) != 0
        ) {
            return;
        }
        let a0 = self.rom_ptr(GO_SPEC_SOUND_INDEX);
        let a3 = self.rl(a0.wrapping_add(u32::from(d7.wrapping_sub(SPEC_FIRST)).wrapping_mul(4)))
            & 0xFF_FFFF;
        let mut a1 = a3;
        let voz = u32::from(self.rw(a1)).wrapping_add(a3);
        self.wl(Self::ram(V_SPECIAL_VOICE_PTR), voz);
        a1 = a1.wrapping_add(2);
        let d5 = self.rb(a1);
        let n = self.rb(a1.wrapping_add(1));
        a1 = a1.wrapping_add(2);
        let mut d4 = 0;
        for _ in 0..n {
            d4 = self.rb(a1.wrapping_add(1));
            let a5 = if decisao!("Sound_PlaySpecial:canal_psg", d4 & 0x80 != 0) {
                self.bset(Self::ram(V_MUSIC_PSG3_TRACK) + T_PLAYBACK_CONTROL, 2);
                Self::ram(V_SPCSFX_PSG3_TRACK)
            } else {
                self.bset(Self::ram(V_MUSIC_FM4_TRACK) + T_PLAYBACK_CONTROL, 2);
                Self::ram(V_SPCSFX_FM4_TRACK)
            };
            for k in 0..(T_LEN / 4) {
                self.wl(a5.wrapping_add(k.wrapping_mul(4)), 0);
            }
            let pc = self.rw(a1);
            self.ww(a5.wrapping_add(T_PLAYBACK_CONTROL), pc);
            a1 = a1.wrapping_add(2);
            self.wb(a5.wrapping_add(T_TEMPO_DIVIDER), d5);
            let rel = u32::from(self.rw(a1));
            self.wl(a5.wrapping_add(T_DATA_POINTER), rel.wrapping_add(a3));
            a1 = a1.wrapping_add(2);
            let tr = self.rw(a1);
            self.ww(a5.wrapping_add(T_TRANSPOSE), tr);
            a1 = a1.wrapping_add(2);
            self.wb(a5.wrapping_add(T_DURATION_TIMEOUT), 1);
            self.wb(a5.wrapping_add(T_STACK_POINTER), lo(T_LEN));
            if !decisao!("Sound_PlaySpecial:inicio_psg", d4 & 0x80 != 0) {
                self.wb(a5.wrapping_add(T_AMSFMSPAN), 0xC0);
            }
        }
        if decisao!(
            "Sound_PlaySpecial:sfx_fm4_tocando",
            self.btst(Self::ram(V_SFX_FM4_TRACK), 7)
        ) {
            self.bset(Self::ram(V_SPCSFX_FM4_TRACK) + T_PLAYBACK_CONTROL, 2);
        }
        if decisao!(
            "Sound_PlaySpecial:sfx_psg3_tocando",
            self.btst(Self::ram(V_SFX_PSG3_TRACK), 7)
        ) {
            self.bset(Self::ram(V_SPCSFX_PSG3_TRACK) + T_PLAYBACK_CONTROL, 2);
            let d4 = d4 | 0x1F;
            self.psg(d4);
            self.psg(d4 ^ 0x20);
        }
    }

    /// `StopSFX`.
    fn stop_sfx(&mut self) {
        self.wb(Self::ram(V_SNDPRIO), 0);
        for i in 0..6u32 {
            let mut a5 = Self::ram(V_SFX_TRACK_RAM).wrapping_add(i.wrapping_mul(T_LEN));
            if !decisao!("StopSFX:tocando", self.btst(a5, 7)) {
                continue;
            }
            self.bclr(a5.wrapping_add(T_PLAYBACK_CONTROL), 7);
            let d3 = self.rb(a5.wrapping_add(T_VOICE_CONTROL));
            if decisao!("StopSFX:canal_psg", d3 & 0x80 != 0) {
                self.psg_note_off(a5);
                let mut a0 = Self::ram(V_SPCSFX_PSG3_TRACK);
                let usa_especial = d3 == 0xE0 || d3 == 0xC0;
                if !usa_especial {
                    a0 = self.canal_bgm(u32::from(d3 >> 3));
                }
                self.bclr(a0.wrapping_add(T_PLAYBACK_CONTROL), 2);
                self.bset(a0.wrapping_add(T_PLAYBACK_CONTROL), 1);
                if self.rb(a0.wrapping_add(T_VOICE_CONTROL)) == 0xE0 {
                    let n = self.rb(a0.wrapping_add(T_PSG_NOISE));
                    self.psg(n);
                }
                continue;
            }
            self.fm_note_off(a5);
            let (alvo, a1);
            if decisao!("StopSFX:fm4", d3 == 4)
                && decisao!(
                    "StopSFX:especial_fm4_tocando",
                    self.btst(Self::ram(V_SPCSFX_FM4_TRACK), 7)
                )
            {
                // O original perde o ponteiro da trilha de efeito aqui (falta `movea.l a5,a3`).
                alvo = Self::ram(V_SPCSFX_FM4_TRACK);
                a1 = self.rl(Self::ram(V_SPECIAL_VOICE_PTR)) & 0xFF_FFFF;
                a5 = alvo;
            } else {
                alvo = self.canal_bgm(u32::from(d3.wrapping_sub(2)) << 2);
                a1 = self.rl(Self::ram(V_VOICE_PTR)) & 0xFF_FFFF;
            }
            self.bclr(alvo.wrapping_add(T_PLAYBACK_CONTROL), 2);
            self.bset(alvo.wrapping_add(T_PLAYBACK_CONTROL), 1);
            let vi = self.rb(alvo.wrapping_add(T_VOICE_INDEX));
            self.set_voice(alvo, a1, vi);
            let _ = a5;
        }
    }

    /// `StopSpecialSFX`.
    fn stop_special_sfx(&mut self) {
        let a5 = Self::ram(V_SPCSFX_FM4_TRACK);
        if decisao!("StopSpecialSFX:fm_tocando", self.btst(a5, 7)) {
            self.bclr(a5.wrapping_add(T_PLAYBACK_CONTROL), 7);
            if !decisao!(
                "StopSpecialSFX:fm_sobreposta",
                self.btst(a5.wrapping_add(T_PLAYBACK_CONTROL), 2)
            ) {
                self.send_fm_note_off(a5);
                let m = Self::ram(V_MUSIC_FM4_TRACK);
                self.bclr(m.wrapping_add(T_PLAYBACK_CONTROL), 2);
                self.bset(m.wrapping_add(T_PLAYBACK_CONTROL), 1);
                if decisao!("StopSpecialSFX:musica_fm4_toca", self.btst(m, 7)) {
                    let a1 = self.rl(Self::ram(V_VOICE_PTR)) & 0xFF_FFFF;
                    let vi = self.rb(m.wrapping_add(T_VOICE_INDEX));
                    self.set_voice(m, a1, vi);
                }
            }
        }
        let a5 = Self::ram(V_SPCSFX_PSG3_TRACK);
        if decisao!("StopSpecialSFX:psg_tocando", self.btst(a5, 7)) {
            self.bclr(a5.wrapping_add(T_PLAYBACK_CONTROL), 7);
            if !decisao!(
                "StopSpecialSFX:psg_sobreposta",
                self.btst(a5.wrapping_add(T_PLAYBACK_CONTROL), 2)
            ) {
                self.send_psg_note_off(a5);
                let m = Self::ram(V_MUSIC_PSG3_TRACK);
                self.bclr(m.wrapping_add(T_PLAYBACK_CONTROL), 2);
                self.bset(m.wrapping_add(T_PLAYBACK_CONTROL), 1);
                if decisao!("StopSpecialSFX:musica_psg3_toca", self.btst(m, 7))
                    && self.rb(m.wrapping_add(T_VOICE_CONTROL)) == 0xE0
                {
                    let n = self.rb(m.wrapping_add(T_PSG_NOISE));
                    self.psg(n);
                }
            }
        }
    }

    /// `FadeOutMusic`.
    fn fade_out_music(&mut self) {
        self.stop_sfx();
        self.stop_special_sfx();
        self.wb(Self::ram(V_FADEOUT_DELAY), 3);
        self.wb(Self::ram(V_FADEOUT_COUNTER), 0x28);
        self.wb(Self::ram(V_MUSIC_DAC_TRACK) + T_PLAYBACK_CONTROL, 0);
        self.wb(Self::ram(F_SPEEDUP), 0);
    }

    /// `DoFadeOut`.
    fn do_fade_out(&mut self) {
        let d0 = self.rb(Self::ram(V_FADEOUT_DELAY));
        if decisao!("DoFadeOut:atraso", d0 != 0) {
            self.wb(Self::ram(V_FADEOUT_DELAY), d0.wrapping_sub(1));
            return;
        }
        let c = self.rb(Self::ram(V_FADEOUT_COUNTER)).wrapping_sub(1);
        self.wb(Self::ram(V_FADEOUT_COUNTER), c);
        if decisao!("DoFadeOut:acabou", c == 0) {
            self.stop_all_sound();
            return;
        }
        self.wb(Self::ram(V_FADEOUT_DELAY), 3);
        let mut a5 = Self::ram(V_MUSIC_FM_TRACKS);
        for _ in 0..6 {
            if decisao!("DoFadeOut:fm_tocando", self.btst(a5, 7)) {
                let v = self.rb(a5.wrapping_add(T_VOLUME)).wrapping_add(1);
                self.wb(a5.wrapping_add(T_VOLUME), v);
                if decisao!("DoFadeOut:volume_ainda_positivo", v & 0x80 == 0) {
                    self.send_voice_tl(a5);
                } else {
                    self.bclr(a5.wrapping_add(T_PLAYBACK_CONTROL), 7);
                }
            }
            a5 = a5.wrapping_add(T_LEN);
        }
        for _ in 0..3 {
            if decisao!("DoFadeOut:psg_tocando", self.btst(a5, 7)) {
                let v = self.rb(a5.wrapping_add(T_VOLUME)).wrapping_add(1);
                self.wb(a5.wrapping_add(T_VOLUME), v);
                if decisao!("DoFadeOut:psg_audivel", v < 0x10) {
                    self.set_psg_volume(a5, v);
                } else {
                    self.bclr(a5.wrapping_add(T_PLAYBACK_CONTROL), 7);
                }
            }
            a5 = a5.wrapping_add(T_LEN);
        }
    }

    /// `FMSilenceAll`.
    fn fm_silence_all(&mut self) {
        for d3 in [2u8, 1, 0] {
            self.write_fm_i(0x28, d3);
            self.write_fm_i(0x28, d3.wrapping_add(4));
        }
        let mut d0 = 0x40u8;
        for _ in 0..3 {
            for _ in 0..4 {
                self.write_fm_i(d0, 0x7F);
                self.write_fm_ii(d0, 0x7F);
                d0 = d0.wrapping_add(4);
            }
            d0 = d0.wrapping_sub(0x0F);
        }
    }

    /// `StopAllSound`.
    fn stop_all_sound(&mut self) {
        self.write_fm_i(0x2B, 0x80);
        self.write_fm_i(0x27, 0);
        // O original apaga 0x390 bytes e deixa de apagar os últimos 0x10.
        for i in 0..((V_1UP_RAM_COPY - 0x10) / 4) {
            self.wl(Self::ram(i.wrapping_mul(4)), 0);
        }
        self.wb(Self::ram(V_SOUND_ID), 0x80);
        self.fm_silence_all();
        self.psg_silence_all();
    }

    /// `InitMusicPlayback`.
    fn init_music_playback(&mut self) {
        let d1 = self.rb(Self::ram(V_SNDPRIO));
        let d2 = self.rb(Self::ram(F_1UP_PLAYING));
        let d3 = self.rb(Self::ram(F_SPEEDUP));
        let d4 = self.rb(Self::ram(V_FADEIN_COUNTER));
        let d5 = self.rw(Self::ram(V_SOUNDQUEUE0));
        for i in 0..(V_1UP_RAM_END / 4) {
            self.wl(Self::ram(i.wrapping_mul(4)), 0);
        }
        self.wb(Self::ram(V_SNDPRIO), d1);
        self.wb(Self::ram(F_1UP_PLAYING), d2);
        self.wb(Self::ram(F_SPEEDUP), d3);
        self.wb(Self::ram(V_FADEIN_COUNTER), d4);
        self.ww(Self::ram(V_SOUNDQUEUE0), d5);
        self.wb(Self::ram(V_SOUND_ID), 0x80);
        self.fm_silence_all();
        self.psg_silence_all();
    }

    /// `TempoWait`.
    fn tempo_wait(&mut self) {
        let t = self.rb(Self::ram(V_MAIN_TEMPO));
        self.wb(Self::ram(V_MAIN_TEMPO_TIMEOUT), t);
        for i in 0..10u32 {
            let a = Self::ram(V_MUSIC_DAC_TRACK)
                .wrapping_add(i.wrapping_mul(T_LEN))
                .wrapping_add(T_DURATION_TIMEOUT);
            let v = self.rb(a).wrapping_add(1);
            self.wb(a, v);
        }
    }

    /// `SpeedUpMusic`.
    fn speed_up_music(&mut self) {
        let deslocamento = if decisao!(
            "SpeedUpMusic:vida_extra",
            self.rb(Self::ram(F_1UP_PLAYING)) != 0
        ) {
            V_1UP_RAM_COPY
        } else {
            0
        };
        let sp = self.rb(Self::ram(deslocamento.wrapping_add(V_SPEEDUPTEMPO)));
        self.wb(Self::ram(deslocamento.wrapping_add(V_MAIN_TEMPO)), sp);
        self.wb(
            Self::ram(deslocamento.wrapping_add(V_MAIN_TEMPO_TIMEOUT)),
            sp,
        );
        self.wb(Self::ram(deslocamento.wrapping_add(F_SPEEDUP)), 0x80);
    }

    /// `SlowDownMusic`.
    fn slow_down_music(&mut self) {
        let deslocamento = if decisao!(
            "SlowDownMusic:vida_extra",
            self.rb(Self::ram(F_1UP_PLAYING)) != 0
        ) {
            V_1UP_RAM_COPY
        } else {
            0
        };
        let t = self.rb(Self::ram(deslocamento.wrapping_add(V_TEMPO_MOD)));
        self.wb(Self::ram(deslocamento.wrapping_add(V_MAIN_TEMPO)), t);
        self.wb(
            Self::ram(deslocamento.wrapping_add(V_MAIN_TEMPO_TIMEOUT)),
            t,
        );
        self.wb(Self::ram(deslocamento.wrapping_add(F_SPEEDUP)), 0);
    }

    /// `DoFadeIn`.
    fn do_fade_in(&mut self) {
        let atraso = self.rb(Self::ram(V_FADEIN_DELAY));
        if decisao!("DoFadeIn:atraso", atraso != 0) {
            self.wb(Self::ram(V_FADEIN_DELAY), atraso.wrapping_sub(1));
            return;
        }
        let c = self.rb(Self::ram(V_FADEIN_COUNTER));
        if decisao!("DoFadeIn:acabou", c == 0) {
            self.bclr(Self::ram(V_MUSIC_DAC_TRACK) + T_PLAYBACK_CONTROL, 2);
            self.wb(Self::ram(F_FADEIN_FLAG), 0);
            return;
        }
        self.wb(Self::ram(V_FADEIN_COUNTER), c.wrapping_sub(1));
        self.wb(Self::ram(V_FADEIN_DELAY), 2);
        let mut a5 = Self::ram(V_MUSIC_FM_TRACKS);
        for _ in 0..6 {
            if decisao!("DoFadeIn:fm_tocando", self.btst(a5, 7)) {
                let v = self.rb(a5.wrapping_add(T_VOLUME)).wrapping_sub(1);
                self.wb(a5.wrapping_add(T_VOLUME), v);
                self.send_voice_tl(a5);
            }
            a5 = a5.wrapping_add(T_LEN);
        }
        for _ in 0..3 {
            if decisao!("DoFadeIn:psg_tocando", self.btst(a5, 7)) {
                let v = self.rb(a5.wrapping_add(T_VOLUME)).wrapping_sub(1);
                self.wb(a5.wrapping_add(T_VOLUME), v);
                let d6 = if decisao!("DoFadeIn:psg_limite", v >= 0x10) {
                    0xF
                } else {
                    v
                };
                self.set_psg_volume(a5, d6);
            }
            a5 = a5.wrapping_add(T_LEN);
        }
    }

    /// `PauseMusic`: `d0` é o byte de `f_pausemusic` (o original usa a flag N dele).
    fn pause_music(&mut self, d0: u8) {
        if decisao!("PauseMusic:retomar", d0 & 0x80 != 0) {
            self.wb(Self::ram(F_PAUSEMUSIC), 0);
            let mut trilhas = vec![];
            for i in 0..7u32 {
                trilhas.push(Self::ram(V_MUSIC_DAC_TRACK).wrapping_add(i.wrapping_mul(T_LEN)));
            }
            for i in 0..3u32 {
                trilhas.push(Self::ram(V_SFX_FM_TRACKS).wrapping_add(i.wrapping_mul(T_LEN)));
            }
            trilhas.push(Self::ram(V_SPCSFX_FM4_TRACK));
            for a5 in trilhas {
                if self.btst(a5, 7) && !self.btst(a5, 2) {
                    let pan = self.rb(a5.wrapping_add(T_AMSFMSPAN));
                    self.write_fm_i_or_ii(a5, 0xB4, pan);
                }
            }
            self.start_z80();
            return;
        }
        if decisao!("PauseMusic:ja_pausado", d0 == 2) {
            self.start_z80();
            return;
        }
        self.wb(Self::ram(F_PAUSEMUSIC), 2);
        for reg in [0xB4u8, 0xB5, 0xB6] {
            self.write_fm_i(reg, 0);
            self.write_fm_ii(reg, 0);
        }
        for d3 in [2u8, 1, 0] {
            self.write_fm_i(0x28, d3);
            self.write_fm_i(0x28, d3.wrapping_add(4));
        }
        self.psg_silence_all();
        self.start_z80();
    }
}

/// Algoritmo de uma voz: os três bits baixos do byte de realimentação.
fn d1_alg(d1: u8) -> u8 {
    d1 & 7
}

trait EntaoFade {
    fn and_then_fade(self, d: &mut Driver<'_>, a5: u32) -> (Cf, u32);
}

impl EntaoFade for (Cf, u32) {
    /// `cfFadeInToPrevious`: o resto da flag roda no driver e a flag sempre sai pelo retorno
    /// adulterado.
    fn and_then_fade(self, d: &mut Driver<'_>, _a5: u32) -> (Cf, u32) {
        d.cf_fade_in_to_previous();
        self
    }
}
