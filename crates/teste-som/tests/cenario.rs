//! Cenário da fase e do título (parte B da Etapa 7), com ROM sintética. Os valores esperados são
//! contas feitas à mão a partir da lógica de cada rotina (rótulo citado), não dados da ROM real.
//! A prova com a ROM real é o comparador `diferencial-abertura`.

use sonic1_core::abertura::cenario;
use sonic1_core::abertura::ram::*;
use sonic1_core::memoria::Memoria;

/// ROM de 512 KiB com os bytes dados nos endereços dados.
fn rom_com(trechos: &[(u32, &[u8])]) -> Vec<u8> {
    let mut rom = vec![0u8; 0x8_0000];
    for (pos, bytes) in trechos {
        let i = usize::try_from(*pos).unwrap_or(0);
        rom[i..i.wrapping_add(bytes.len())].copy_from_slice(bytes);
    }
    rom
}

fn indice_ram(a: u32) -> usize {
    usize::try_from(a.wrapping_sub(0xFF_0000)).unwrap_or(0)
}

fn ram_b(m: &Memoria, a: u32) -> u8 {
    m.ram()[indice_ram(a)]
}

fn ram_w(m: &Memoria, a: u32) -> u16 {
    let i = indice_ram(a);
    u16::from_be_bytes([m.ram()[i], m.ram()[i.wrapping_add(1)]])
}

fn ram_l(m: &Memoria, a: u32) -> u32 {
    let i = indice_ram(a);
    let r = m.ram();
    u32::from_be_bytes([r[i], r[i + 1], r[i + 2], r[i + 3]])
}

/// Tabelas da GHZ1 (primeiro ato da zona 0) com um ponto de partida e um layout mínimo.
fn rom_ghz1() -> Vec<u8> {
    rom_com(&[
        // `LevelSizeArray`, entrada 0: não usada, esquerda 0, direita $800, topo 0, base $300,
        // deslocamento do olhar $60.
        (
            0x5F2A,
            &[
                0x00, 0x04, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x60,
            ],
        ),
        // `StartLocArray`, entrada 0: X $50, Y $3B0.
        (0x611E, &[0x00, 0x50, 0x03, 0xB0]),
        // `LoopChunkNums`, zona 0: $B5, $7F, $1F, $20.
        (0x618E, &[0xB5, 0x7F, 0x1F, 0x20]),
        // `Level_Index`: ponteiro da frente (índice 0) aponta para $10; do fundo (índice 2), $20.
        (0x68B96, &[0x00, 0x10, 0x00, 0x20]),
        // Layout da frente: largura 1, altura 0, dois chunks $AB e $CD.
        (0x68B96 + 0x10, &[0x01, 0x00, 0xAB, 0xCD]),
        // Layout do fundo: largura 0, altura 0, um chunk $EE.
        (0x68B96 + 0x20, &[0x00, 0x00, 0xEE]),
    ])
}

#[test]
fn level_size_load_le_limites_camera_e_inicio() {
    let mut m = Memoria::nova(rom_ghz1());
    cenario::level_size_load(&mut m).unwrap();
    assert_eq!(ram_w(&m, V_UNUSED11), 4);
    assert_eq!(ram_w(&m, V_LIMITRIGHT2), 0x0800);
    assert_eq!(ram_w(&m, V_LIMITBTM2), 0x0300);
    assert_eq!(ram_l(&m, V_LIMITLEFT1), 0x0000_0800);
    assert_eq!(ram_w(&m, V_LIMITLEFT3), 0x0240);
    assert_eq!(ram_w(&m, V_LOOKSHIFT), 0x0060);
    assert_eq!(ram_b(&m, V_FG_XBLOCK), 0x10);
    assert_eq!(ram_b(&m, V_FG_YBLOCK), 0x10);
    // Sonic em X $50 e Y $3B0. A câmera fica em X 0 (X menor que 160) e em Y no limite de baixo.
    assert_eq!(ram_w(&m, V_PLAYER + OB_X), 0x50);
    assert_eq!(ram_w(&m, V_PLAYER + OB_Y), 0x03B0);
    assert_eq!(ram_w(&m, V_SCREENPOSX), 0);
    assert_eq!(ram_w(&m, V_SCREENPOSY), 0x0300);
    // `BgScroll_GHZ`: posições do fundo forçadas a zero.
    assert_eq!(ram_l(&m, V_BGSCREENPOSX), 0);
    assert_eq!(ram_l(&m, V_BGSCREENPOSY), 0);
    assert_eq!(ram_l(&m, V_256LOOP1), 0xB57F_1F20);
}

#[test]
fn deform_ghz_sem_movimento_preenche_224_linhas() {
    let mut m = Memoria::nova(rom_ghz1());
    cenario::level_size_load(&mut m).unwrap();
    cenario::deform_layers(&mut m).unwrap();
    // Y da câmera $300: `(Y & $7FF) >> 5` é $18, e $20 menos isso dá 8 linhas de deslocamento.
    assert_eq!(ram_w(&m, V_SCREENPOSY), 0x0300);
    assert_eq!(ram_w(&m, V_BGSCRPOSY_VDP), 8);
    // Nuvem superior: 24 linhas com a palavra baixa $FFFF (o buffer de nuvem vale 1 na palavra alta).
    for i in 0..24u32 {
        assert_eq!(
            ram_l(&m, V_HSCROLLTABLEBUFFER + 4 * i),
            0x0000_FFFF,
            "linha {i}"
        );
    }
    // Nuvens do meio e de baixo, montanhas, colinas e água: sem deslocamento, tudo zero.
    for i in 24..224u32 {
        assert_eq!(ram_l(&m, V_HSCROLLTABLEBUFFER + 4 * i), 0, "linha {i}");
    }
}

#[test]
fn level_layout_load_limpa_e_copia_as_duas_camadas() {
    let mut m = Memoria::nova(rom_ghz1());
    // Sujeira num lugar que o `LevelLayoutLoad` limpa ($800 bytes a partir de `v_lvllayout`).
    m.escrever(
        V_LVLLAYOUT + 0x7F0,
        sonic1_core::memoria::Tamanho::Byte,
        0x55,
    )
    .unwrap();
    cenario::level_size_load(&mut m).unwrap();
    cenario::level_layout_load(&mut m).unwrap();
    assert_eq!(ram_b(&m, V_LVLLAYOUT_FG), 0xAB);
    assert_eq!(ram_b(&m, V_LVLLAYOUT_FG + 1), 0xCD);
    assert_eq!(ram_b(&m, V_LVLLAYOUT_FG + 2), 0);
    assert_eq!(ram_b(&m, V_LVLLAYOUT_FG + LAYOUT_ROW), 0);
    assert_eq!(ram_b(&m, V_LVLLAYOUT_BG), 0xEE);
    assert_eq!(ram_b(&m, V_LVLLAYOUT + 0x7F0), 0);
}
