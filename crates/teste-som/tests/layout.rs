//! As constantes do driver contra o mapa de rótulos (`refs/rotulos.txt`, só nomes e endereços).

use sonic1_core::som::driver::{self, layout::*};
use std::collections::BTreeMap;

fn mapa() -> BTreeMap<String, u64> {
    let texto = include_str!("../../../refs/rotulos.txt");
    texto
        .lines()
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| {
            let mut c = l.split('\t');
            let nome = c.next()?.to_string();
            let _tipo = c.next()?;
            let valor = u64::from_str_radix(c.next()?, 16).ok()?;
            Some((nome, valor))
        })
        .collect()
}

#[test]
fn layout_da_ram_e_das_trilhas_bate_com_o_mapa() {
    let m = mapa();
    let ram = [
        ("v_sndprio", V_SNDPRIO),
        ("v_main_tempo_timeout", V_MAIN_TEMPO_TIMEOUT),
        ("v_main_tempo", V_MAIN_TEMPO),
        ("f_pausemusic", F_PAUSEMUSIC),
        ("v_fadeout_counter", V_FADEOUT_COUNTER),
        ("v_fadeout_delay", V_FADEOUT_DELAY),
        ("v_communication_byte", V_COMMUNICATION_BYTE),
        ("f_updating_dac", F_UPDATING_DAC),
        ("v_sound_id", V_SOUND_ID),
        ("v_soundqueue0", V_SOUNDQUEUE0),
        ("v_soundqueue1", V_SOUNDQUEUE1),
        ("v_soundqueue2", V_SOUNDQUEUE2),
        ("f_voice_selector", F_VOICE_SELECTOR),
        ("v_voice_ptr", V_VOICE_PTR),
        ("v_special_voice_ptr", V_SPECIAL_VOICE_PTR),
        ("f_fadein_flag", F_FADEIN_FLAG),
        ("v_fadein_delay", V_FADEIN_DELAY),
        ("v_fadein_counter", V_FADEIN_COUNTER),
        ("f_1up_playing", F_1UP_PLAYING),
        ("v_tempo_mod", V_TEMPO_MOD),
        ("v_speeduptempo", V_SPEEDUPTEMPO),
        ("f_speedup", F_SPEEDUP),
        ("v_ring_speaker", V_RING_SPEAKER),
        ("f_push_playing", F_PUSH_PLAYING),
        ("v_music_dac_track", V_MUSIC_DAC_TRACK),
        ("v_music_fm_tracks", V_MUSIC_FM_TRACKS),
        ("v_music_fm4_track", V_MUSIC_FM4_TRACK),
        ("v_music_psg_tracks", V_MUSIC_PSG_TRACKS),
        ("v_music_psg3_track", V_MUSIC_PSG3_TRACK),
        ("v_music_track_ram_end", V_MUSIC_TRACK_RAM_END),
        ("v_1up_ram_end", V_1UP_RAM_END),
        ("v_sfx_track_ram", V_SFX_TRACK_RAM),
        ("v_sfx_fm_tracks", V_SFX_FM_TRACKS),
        ("v_sfx_fm4_track", V_SFX_FM4_TRACK),
        ("v_sfx_psg_tracks", V_SFX_PSG_TRACKS),
        ("v_sfx_psg3_track", V_SFX_PSG3_TRACK),
        ("v_spcsfx_fm4_track", V_SPCSFX_FM4_TRACK),
        ("v_spcsfx_psg3_track", V_SPCSFX_PSG3_TRACK),
        ("v_1up_ram_copy", V_1UP_RAM_COPY),
    ];
    for (nome, valor) in ram {
        let chave = format!("SMPS_RAM.{nome}");
        assert_eq!(m.get(&chave), Some(&u64::from(valor)), "{chave}");
    }
    let trilha = [
        ("len", T_LEN),
        ("PlaybackControl", T_PLAYBACK_CONTROL),
        ("VoiceControl", T_VOICE_CONTROL),
        ("TempoDivider", T_TEMPO_DIVIDER),
        ("DataPointer", T_DATA_POINTER),
        ("Transpose", T_TRANSPOSE),
        ("Volume", T_VOLUME),
        ("AMSFMSPan", T_AMSFMSPAN),
        ("VoiceIndex", T_VOICE_INDEX),
        ("VolEnvIndex", T_VOL_ENV_INDEX),
        ("StackPointer", T_STACK_POINTER),
        ("DurationTimeout", T_DURATION_TIMEOUT),
        ("SavedDuration", T_SAVED_DURATION),
        ("Freq", T_FREQ),
        ("SavedDAC", T_SAVED_DAC),
        ("NoteTimeout", T_NOTE_TIMEOUT),
        ("NoteTimeoutMaster", T_NOTE_TIMEOUT_MASTER),
        ("ModulationPtr", T_MODULATION_PTR),
        ("ModulationWait", T_MODULATION_WAIT),
        ("ModulationSpeed", T_MODULATION_SPEED),
        ("ModulationDelta", T_MODULATION_DELTA),
        ("ModulationSteps", T_MODULATION_STEPS),
        ("ModulationVal", T_MODULATION_VAL),
        ("Detune", T_DETUNE),
        ("FeedbackAlgo", T_FEEDBACK_ALGO),
        ("PSGNoise", T_PSG_NOISE),
        ("VoicePtr", T_VOICE_PTR),
        ("LoopCounters", T_LOOP_COUNTERS),
    ];
    for (nome, valor) in trilha {
        let chave = format!("SMPS_Track.{nome}");
        assert_eq!(m.get(&chave), Some(&u64::from(valor)), "{chave}");
    }
    assert_eq!(m.get("v_snddriver_ram"), Some(&0xFFFF_F000));
    assert_eq!(0xFF_F000, BASE);
}

#[test]
fn enderecos_das_tabelas_da_rom_batem_com_o_mapa() {
    let m = mapa();
    let tabelas = [
        ("Go_SoundPriorities", driver::GO_SOUND_PRIORITIES),
        ("Go_SpecSoundIndex", driver::GO_SPEC_SOUND_INDEX),
        ("Go_MusicIndex", driver::GO_MUSIC_INDEX),
        ("Go_SoundIndex", driver::GO_SOUND_INDEX),
        ("Go_SpeedUpIndex", driver::GO_SPEEDUP_INDEX),
        ("Go_PSGIndex", driver::GO_PSG_INDEX),
        ("FMDACInitBytes", driver::FM_DAC_INIT_BYTES),
        ("PSGInitBytes", driver::PSG_INIT_BYTES),
        ("SFX_BGMChannelRAM", driver::SFX_BGM_CHANNEL_RAM),
        ("SFX_SFXChannelRAM", driver::SFX_SFX_CHANNEL_RAM),
        ("FMFrequencies", driver::FM_FREQUENCIES),
        ("PSGFrequencies", driver::PSG_FREQUENCIES),
        ("FMSlotMask", driver::FM_SLOT_MASK),
        (
            "FMInstrumentOperatorTable",
            driver::FM_INSTRUMENT_OPERATOR_TABLE,
        ),
        (
            "FMInstrumentOperatorTable_End",
            driver::FM_INSTRUMENT_OPERATOR_TABLE_END,
        ),
        ("FMInstrumentTLTable", driver::FM_INSTRUMENT_TL_TABLE),
        (
            "FMInstrumentTLTable_End",
            driver::FM_INSTRUMENT_TL_TABLE_END,
        ),
        ("DAC_sample_rate", driver::DAC_SAMPLE_RATE),
    ];
    for (nome, valor) in tabelas {
        assert_eq!(m.get(nome), Some(&u64::from(valor)), "{nome}");
    }
}
