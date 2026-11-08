#![allow(non_snake_case)] // nomes BK_nnnn da lista de bugs conhecidos
//! Driver do 68000 com uma ROM sintética escrita por nós: tabelas, uma música mínima e
//! efeitos de brinquedo. O formato de música e de voz segue a documentação pública do SMPS
//! (Sonic Retro, "SMPS" e "SMPS Music Format"). Nada vem da ROM do jogo.

use sonic1_core::memoria::{Memoria, Saida, Tamanho};
use sonic1_core::som::driver::{Driver, layout::*};

const TAMANHO_ROM: usize = 0x8_0000;
const MUSICA: usize = 0x7_5000;
const EFEITO: usize = 0x7_6000;

fn p32(rom: &mut [u8], a: usize, v: u32) {
    rom[a..a + 4].copy_from_slice(&v.to_be_bytes());
}

fn p16(rom: &mut [u8], a: usize, v: u16) {
    rom[a..a + 2].copy_from_slice(&v.to_be_bytes());
}

/// Tabelas do driver nos endereços dos rótulos, com valores inventados.
fn rom_base() -> Vec<u8> {
    let mut rom = vec![0u8; TAMANHO_ROM];
    p32(&mut rom, 0x7_1990, 0x7_1AE8); // Go_SoundPriorities
    p32(&mut rom, 0x7_1994, 0x7_7000); // Go_SpecSoundIndex
    p32(&mut rom, 0x7_1998, 0x7_1A9C); // Go_MusicIndex
    p32(&mut rom, 0x7_199C, 0x7_8B44); // Go_SoundIndex
    p32(&mut rom, 0x7_19A0, 0x7_1A94); // Go_SpeedUpIndex
    p32(&mut rom, 0x7_19A4, 0x7_19A8); // Go_PSGIndex
    p32(&mut rom, 0x7_19A8, 0x7_1A00); // PSG1
    rom[0x7_1A00..0x7_1A05].copy_from_slice(&[0, 1, 2, 3, 0x80]);
    rom[0x7_1A94] = 7; // SpeedUpIndex[0]
    p32(&mut rom, 0x7_1A9C, u32::try_from(MUSICA).unwrap()); // MusicIndex[0] = $81
    for i in 0..0x80 {
        rom[0x7_1AE8 + i] = 0x90; // prioridades
    }
    rom[0x7_1AE8 + 0x20] = 0x80; // $A1: prioridade menor
    p32(&mut rom, 0x7_8B44, u32::try_from(EFEITO).unwrap()); // SoundIndex[0] = $A0
    rom[0x7_21BA..0x7_21C1].copy_from_slice(&[6, 0, 1, 2, 4, 5, 6]);
    rom[0x7_21C2..0x7_21C5].copy_from_slice(&[0x80, 0xA0, 0xC0]);
    // SFX_BGMChannelRAM e SFX_SFXChannelRAM: FM3, vazio, FM4, FM5, PSG1..3, ruído.
    let bgm = [0xD0u32, 0, 0x100, 0x130, 0x190, 0x1C0, 0x1F0, 0x1F0];
    let sfx = [0x220u32, 0, 0x250, 0x280, 0x2B0, 0x2E0, 0x310, 0x310];
    for i in 0..8 {
        p32(&mut rom, 0x7_22CC + i * 4, 0xFF_F000 + bgm[i]);
        p32(&mut rom, 0x7_22EC + i * 4, 0xFF_F000 + sfx[i]);
    }
    // FMFrequencies: uma frequência distinta por nota.
    for i in 0..96usize {
        p16(
            &mut rom,
            0x7_2790 + i * 2,
            u16::try_from(0x200 + i * 7).unwrap(),
        );
    }
    rom[0x7_2CAC..0x7_2CB4].copy_from_slice(&[8, 8, 8, 8, 0xA, 0xE, 0xE, 0xF]);
    let ops = [
        0x30u8, 0x38, 0x34, 0x3C, 0x50, 0x58, 0x54, 0x5C, 0x60, 0x68, 0x64, 0x6C, 0x70, 0x78, 0x74,
        0x7C, 0x80, 0x88, 0x84, 0x8C,
    ];
    rom[0x7_2D18..0x7_2D2C].copy_from_slice(&ops);
    rom[0x7_2D2C..0x7_2D30].copy_from_slice(&[0x40, 0x48, 0x44, 0x4C]);
    // Música mínima: uma trilha de DAC e uma de FM, sem PSG.
    let m = MUSICA;
    p16(&mut rom, m, 0x40); // voz
    rom[m + 2] = 2; // DAC + FM1
    rom[m + 3] = 0; // sem PSG
    rom[m + 4] = 1; // divisor de tempo
    rom[m + 5] = 3; // tempo
    p16(&mut rom, m + 6, 0x20); // DAC
    p16(&mut rom, m + 8, 0);
    p16(&mut rom, m + 10, 0x28); // FM1
    p16(&mut rom, m + 12, 0);
    rom[m + 0x20..m + 0x23].copy_from_slice(&[0x81, 0x03, 0xF2]); // amostra 81, duração 3, parar
    rom[m + 0x28..m + 0x2F].copy_from_slice(&[0xEF, 0x00, 0x85, 0x04, 0x80, 0x02, 0xF2]);
    // Voz 0: algoritmo 7, 20 bytes de operadores, 4 de TL.
    rom[m + 0x40] = 0x07;
    for i in 0..20 {
        rom[m + 0x41 + i] = u8::try_from(i).unwrap();
    }
    rom[m + 0x55..m + 0x59].copy_from_slice(&[0x10, 0x11, 0x12, 0x13]);
    rom
}

fn memoria() -> Memoria {
    let mut m = Memoria::nova(rom_base());
    // Z80 fora do reset, como no jogo, para a janela responder quando o driver pedir o barramento.
    m.escrever(0xA1_1200, Tamanho::Word, 0x100).unwrap();
    m
}

fn ym_e_psg(m: &mut Memoria) -> Vec<Saida> {
    m.tirar_saidas()
        .into_iter()
        .filter(|s| matches!(s, Saida::Ym { .. } | Saida::Psg(_)))
        .collect()
}

fn ram(m: &Memoria, off: u32) -> u8 {
    m.ram()[usize::try_from(0xF000 + off).unwrap()]
}

#[test]
fn fila_escreve_no_byte_certo() {
    let mut m = memoria();
    {
        let mut d = Driver::novo(&mut m);
        d.queue_sound1(0x81);
        d.queue_sound2(0xA0);
        d.queue_sound3(0xB5);
    }
    assert_eq!(ram(&m, V_SOUNDQUEUE0), 0x81);
    assert_eq!(ram(&m, V_SOUNDQUEUE1), 0xA0);
    assert_eq!(ram(&m, V_SOUNDQUEUE2), 0xB5);
}

#[test]
fn BK_0276_a_terceira_fila_e_ignorada_pelo_update() {
    // Bug conhecido do original: `UpdateMusic` só testa as duas primeiras filas.
    let mut m = memoria();
    m.ram_mut()[0xF000 + usize::try_from(V_SOUND_ID).unwrap()] = 0x80;
    Driver::novo(&mut m).queue_sound3(0xA0);
    Driver::novo(&mut m).update_music();
    assert_eq!(ram(&m, V_SOUNDQUEUE2), 0xA0, "a fila 3 não foi lida");
    assert_eq!(ram(&m, V_SOUND_ID), 0x80);
}

#[test]
fn musica_minima_carrega_as_trilhas_como_o_formato_publico() {
    let mut m = memoria();
    m.ram_mut()[0xF000 + usize::try_from(V_SOUND_ID).unwrap()] = 0x80;
    Driver::novo(&mut m).queue_sound1(0x81);
    Driver::novo(&mut m).update_music();
    // DAC: tocando, voz 6, ponteiro = início da música + 0x20.
    let dac = V_MUSIC_DAC_TRACK;
    assert_eq!(ram(&m, dac + T_PLAYBACK_CONTROL) & 0x80, 0x80);
    assert_eq!(ram(&m, dac + T_VOICE_CONTROL), 6);
    let ponteiro = u32::from_be_bytes([
        ram(&m, dac + T_DATA_POINTER),
        ram(&m, dac + T_DATA_POINTER + 1),
        ram(&m, dac + T_DATA_POINTER + 2),
        ram(&m, dac + T_DATA_POINTER + 3),
    ]);
    // O driver já consumiu o primeiro byte da trilha no mesmo update, então o ponteiro andou.
    assert!(ponteiro >= u32::try_from(MUSICA).unwrap() + 0x20);
    // FM1: voz 0, painel $C0, divisor de tempo 1.
    let fm1 = V_MUSIC_FM_TRACKS;
    assert_eq!(ram(&m, fm1 + T_VOICE_CONTROL), 0);
    assert_eq!(ram(&m, fm1 + T_TEMPO_DIVIDER), 1);
    // Tempo da música.
    assert_eq!(ram(&m, V_TEMPO_MOD), 3);
    assert_eq!(ram(&m, V_SPEEDUPTEMPO), 7);
}

#[test]
fn nota_de_fm_escreve_voz_frequencia_e_note_on_nessa_ordem() {
    let mut m = memoria();
    m.ram_mut()[0xF000 + usize::try_from(V_SOUND_ID).unwrap()] = 0x80;
    Driver::novo(&mut m).queue_sound1(0x81);
    let mut escritas = Vec::new();
    for _ in 0..8 {
        Driver::novo(&mut m).update_music();
        escritas.extend(ym_e_psg(&mut m));
    }
    // Há escritas de endereço e de dado no YM2612, e o note-on do canal 0 ($28 com $F0) vem
    // depois de ao menos uma escrita das frequências $A4 e $A0.
    let ym: Vec<(u8, u8)> = escritas
        .iter()
        .filter_map(|s| match s {
            Saida::Ym { porta, valor } => Some((*porta, *valor)),
            _ => None,
        })
        .collect();
    let pos_a4 = ym.iter().position(|e| *e == (0, 0xA4));
    let pos_on = ym.iter().position(|e| *e == (1, 0xF0));
    assert!(pos_a4.is_some(), "{ym:?}");
    assert!(pos_on.is_some(), "{ym:?}");
    assert!(pos_a4.unwrap() < pos_on.unwrap());
}

#[test]
fn efeito_de_prioridade_menor_nao_toma_a_vez_de_um_mais_alto() {
    let mut m = memoria();
    m.ram_mut()[0xF000 + usize::try_from(V_SOUND_ID).unwrap()] = 0x80;
    m.ram_mut()[0xF000 + usize::try_from(V_SNDPRIO).unwrap()] = 0x90;
    // $A1 tem prioridade $80 (menor que $90): é descartado; o contador de prioridade fica.
    Driver::novo(&mut m).queue_sound2(0xA1);
    Driver::novo(&mut m).update_music();
    assert_eq!(ram(&m, V_SOUND_ID), 0x80);
    assert_eq!(ram(&m, V_SNDPRIO), 0x90);
    assert_eq!(ram(&m, V_SOUNDQUEUE1), 0, "a fila foi esvaziada");
}

#[test]
fn execucoes_iguais_dao_saidas_iguais() {
    let rodar = || {
        let mut m = memoria();
        m.ram_mut()[0xF000 + usize::try_from(V_SOUND_ID).unwrap()] = 0x80;
        Driver::novo(&mut m).queue_sound1(0x81);
        let mut tudo = Vec::new();
        for _ in 0..20 {
            Driver::novo(&mut m).update_music();
            tudo.extend(m.tirar_saidas());
        }
        (tudo, m.ram().to_vec())
    };
    assert_eq!(rodar(), rodar());
}

#[test]
fn BK_0286_parar_tudo_limpa_a_ram_do_driver_menos_os_ultimos_16_bytes() {
    // Bug conhecido: `StopAllSound` deixa de apagar os últimos $10 bytes antes da cópia do 1UP.
    let mut m = memoria();
    for i in 0..usize::try_from(V_1UP_RAM_COPY).unwrap() {
        m.ram_mut()[0xF000 + i] = 0xAA;
    }
    m.ram_mut()[0xF000 + usize::try_from(V_SOUND_ID).unwrap()] = 0xE4;
    // v_sound_id é lido por PlaySoundID; o byte de pausa fica zerado.
    m.ram_mut()[0xF000 + usize::try_from(F_PAUSEMUSIC).unwrap()] = 0;
    // O tempo principal tem de expirar ou o update segue para o som novo do mesmo jeito.
    Driver::novo(&mut m).update_music();
    let fim_limpo = usize::try_from(V_1UP_RAM_COPY).unwrap() - 0x10;
    assert!(
        m.ram()[0xF000 + 0x10..0xF000 + fim_limpo]
            .iter()
            .all(|b| *b == 0 || *b == 0x80)
    );
    assert!(
        m.ram()[0xF000 + fim_limpo + 4..0xF000 + fim_limpo + 0x10]
            .iter()
            .all(|b| *b == 0xAA)
    );
}

fn poke(m: &mut Memoria, off: u32, v: u8) {
    m.ram_mut()[0xF000 + usize::try_from(off).unwrap()] = v;
}

#[test]
fn BK_0277_musica_aceita_ids_ate_9f_e_trata_94_como_musica() {
    // O original testa `<= $9F` e não `<= $93`: o ID $94 entra em `Sound_PlayBGM`.
    let mut m = memoria();
    poke(&mut m, F_1UP_PLAYING, 0x55);
    poke(&mut m, V_SOUND_ID, 0x94);
    Driver::novo(&mut m).update_music();
    assert_eq!(ram(&m, F_1UP_PLAYING), 0, "o caminho de música rodou");
}

#[test]
fn BK_0278_especial_aceita_ids_ate_df_e_trata_d5_como_especial() {
    let mut m = memoria();
    for i in 0..4 {
        poke(&mut m, V_SPECIAL_VOICE_PTR + i, 0xEE);
    }
    poke(&mut m, V_SOUND_ID, 0xD5);
    Driver::novo(&mut m).update_music();
    assert_ne!(
        ram(&m, V_SPECIAL_VOICE_PTR),
        0xEE,
        "o caminho do especial rodou"
    );
}

#[test]
fn BK_0287_inicio_de_musica_perde_a_terceira_fila() {
    // `InitMusicPlayback` salva e restaura só as filas 0 e 1: a fila 3 é apagada.
    let mut m = memoria();
    poke(&mut m, V_SOUND_ID, 0x81);
    poke(&mut m, V_SOUNDQUEUE2, 0x77);
    Driver::novo(&mut m).update_music();
    assert_eq!(ram(&m, V_SOUNDQUEUE2), 0);
}

#[test]
fn BK_0290_fim_do_fade_in_nao_reescreve_o_registrador_2b() {
    let mut m = memoria();
    poke(&mut m, F_FADEIN_FLAG, 0x80);
    poke(&mut m, V_FADEIN_COUNTER, 0);
    poke(&mut m, V_SOUND_ID, 0x80);
    Driver::novo(&mut m).update_music();
    let ym: Vec<Saida> = m
        .tirar_saidas()
        .into_iter()
        .filter(|s| matches!(s, Saida::Ym { .. }))
        .collect();
    assert!(
        ym.is_empty(),
        "sem o conserto, o fim do fade não escreve no YM: {ym:?}"
    );
    assert_eq!(ram(&m, F_FADEIN_FLAG), 0);
}

#[test]
fn BK_0292_parar_psg3_nao_silencia_o_ruido() {
    let mut m = memoria();
    let a5 = 0xFF_F000 + V_MUSIC_PSG3_TRACK;
    m.escrever(a5 + T_VOICE_CONTROL, Tamanho::Byte, 0xC0)
        .unwrap();
    Driver::novo(&mut m).psg_note_off(a5);
    let psg: Vec<u8> = m
        .tirar_saidas()
        .into_iter()
        .filter_map(|s| if let Saida::Psg(v) = s { Some(v) } else { None })
        .collect();
    assert_eq!(psg, vec![0xDF], "só o canal 3, sem o byte $FF do ruído");
}

#[test]
fn BK_0291_envelope_de_psg_so_trata_o_terminador_80() {
    // Envelope com o valor $E0 (negativo, mas não o terminador).
    let env = 0x7_1A00usize;
    // O teste monta a ROM base e troca o primeiro byte do envelope 1.
    let mut rom = rom_base();
    rom[env] = 0xE0;
    let mut m = Memoria::nova(rom);
    m.escrever(0xA1_1200, Tamanho::Word, 0x100).unwrap();
    let a5 = 0xFF_F000 + V_MUSIC_PSG_TRACKS;
    m.escrever(a5 + T_VOICE_CONTROL, Tamanho::Byte, 0x80)
        .unwrap();
    m.escrever(a5 + T_VOICE_INDEX, Tamanho::Byte, 1).unwrap();
    m.escrever(a5 + T_VOLUME, Tamanho::Byte, 2).unwrap();
    Driver::novo(&mut m).psg_do_vol_fx(a5);
    let psg: Vec<u8> = m
        .tirar_saidas()
        .into_iter()
        .filter_map(|s| if let Saida::Psg(v) = s { Some(v) } else { None })
        .collect();
    // Volume 2 + $E0 passa de $0F: o original limita a $0F e envia (canal $80 | $0F) + $10.
    assert_eq!(psg, vec![0x9F]);
    assert_eq!(
        ram(&m, V_MUSIC_PSG_TRACKS + T_VOL_ENV_INDEX),
        1,
        "o índice andou"
    );
}
