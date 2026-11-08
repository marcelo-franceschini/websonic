//! Endereços da abertura contra o mapa de rótulos (`refs/rotulos.txt`, só nomes e endereços).

use sonic1_core::abertura::MARCAS;
use sonic1_core::abertura::ram::*;
use std::collections::BTreeMap;

fn mapa() -> BTreeMap<String, u64> {
    include_str!("../../../refs/rotulos.txt")
        .lines()
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| {
            let mut c = l.split('\t');
            let nome = c.next()?.to_string();
            let _tipo = c.next()?;
            Some((nome, u64::from_str_radix(c.next()?, 16).ok()?))
        })
        .collect()
}

#[test]
fn constantes_da_abertura_batem_com_o_mapa() {
    let m = mapa();
    let ram = [
        ("v_gamemode", V_GAMEMODE),
        ("v_vdp_buffer1", V_VDP_BUFFER1),
        ("v_hblank_hreg", V_HBLANK_HREG),
        ("v_scrposy_vdp", V_SCRPOSY_VDP),
        ("v_scrposx_vdp", V_SCRPOSX_VDP),
        ("v_crossresetram", V_CROSSRESETRAM),
        ("v_megadrive", V_MEGADRIVE),
        ("v_init", V_INIT),
        ("v_systemstack_end", PILHA_INICIO),
        ("v_systemstack", PILHA_FIM),
        ("v_vblank_routine", V_VBLANK_ROUTINE),
        ("v_vblank_count", V_VBLANK_COUNT),
        ("v_vdp_buffer2", V_VDP_BUFFER2),
        ("f_hblank_pal", F_HBLANK_PAL),
        ("f_wtr_state", F_WTR_STATE),
        ("v_generictimer", V_GENERICTIMER),
        ("v_pcyc_num", V_PCYC_NUM),
        ("v_pcyc_time", V_PCYC_TIME),
        ("v_pfade_start", V_PFADE_START),
        ("v_jpadhold1", V_JPADHOLD1),
        ("v_plc_buffer", V_PLC_BUFFER),
        ("v_plc_buffer_end", V_PLC_BUFFER_END),
        ("v_palette", V_PALETTE),
        ("v_palette_line_2", V_PALETTE_LINE_2),
        ("v_palette_water", V_PALETTE_WATER),
        ("v_palette_fading", V_PALETTE_FADING),
        ("v_spritetablebuffer", V_SPRITETABLEBUFFER),
        ("v_spritetablebuffer_end", V_SPRITETABLEBUFFER_END),
        ("v_hscrolltablebuffer", V_HSCROLLTABLEBUFFER),
        (
            "v_hscrolltablebuffer_end_padded",
            V_HSCROLLTABLEBUFFER_END_PADDED,
        ),
    ];
    for (nome, valor) in ram {
        let esperado = m[nome] & 0xFF_FFFF;
        assert_eq!(u64::from(valor), esperado, "{nome}");
    }
    let rom = [
        ("SetupValues", SETUP_VALUES),
        ("SetupValues_VDP", SETUP_VALUES_VDP),
        ("SetupValues_VDP_End", SETUP_VALUES_VDP_END),
        ("SetupValues_Z80", SETUP_VALUES_Z80),
        ("SetupValues_Z80_End", SETUP_VALUES_Z80_END),
        ("SetupValues_PSG", SETUP_VALUES_PSG),
        ("SetupValues_PSG_End", SETUP_VALUES_PSG_END),
        ("VDPSetupArray", VDP_SETUP_ARRAY),
        ("VDPSetupArray_End", VDP_SETUP_ARRAY_END),
        ("DACDriver", DAC_DRIVER),
        ("Checksum", CHECKSUM),
        ("RomEndLoc", ROM_END_LOC),
        ("EndOfHeader", END_OF_HEADER),
        ("Nem_SegaLogo", NEM_SEGA_LOGO),
        ("Eni_SegaLogo", ENI_SEGA_LOGO),
        ("Pal_Index", PAL_INDEX),
        ("Pal_Sega1", PAL_SEGA1),
        ("Pal_Sega1_end", PAL_SEGA1_END),
        ("Pal_Sega2", PAL_SEGA2),
        ("bgm_Stop", u32::from(BGM_STOP)),
        ("sfx_Sega", u32::from(SFX_SEGA)),
    ];
    for (nome, valor) in rom {
        assert_eq!(u64::from(valor), m[nome], "{nome}");
    }
    for (_, nome, endereco) in MARCAS {
        assert_eq!(u64::from(endereco), m[nome], "marca {nome}");
    }
}
