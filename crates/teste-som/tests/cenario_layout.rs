//! Endereços do cenário e do título (parte B da Etapa 7) contra o mapa de rótulos
//! (`refs/rotulos.txt`, só nomes e endereços). Mesmo padrão de `abertura_layout.rs`.

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
fn enderecos_do_cenario_batem_com_o_mapa() {
    let m = mapa();
    let ram = [
        ("v_lastlamp", V_LASTLAMP),
        ("v_debuguse", V_DEBUGUSE),
        ("v_zone_act", V_ZONE_ACT),
        ("v_act", V_ACT),
        ("f_demo", F_DEMO),
        ("v_unused2", V_UNUSED2),
        ("v_unused7", V_UNUSED7),
        ("v_unused8", V_UNUSED8),
        ("v_dle_routine", V_DLE_ROUTINE),
        ("f_nobgscroll", F_NOBGSCROLL),
        ("v_unused9", V_UNUSED9),
        ("v_unused10", V_UNUSED10),
        ("v_unused11", V_UNUSED11),
        ("v_fg_xblock", V_FG_XBLOCK),
        ("v_fg_yblock", V_FG_YBLOCK),
        ("v_bg2_xblock", V_BG2_XBLOCK),
        ("v_bg3_xblock", V_BG3_XBLOCK),
        ("v_fg_scroll_flags", V_FG_SCROLL_FLAGS),
        ("v_bg1_scroll_flags", V_BG1_SCROLL_FLAGS),
        ("v_bg2_scroll_flags", V_BG2_SCROLL_FLAGS),
        ("v_bg3_scroll_flags", V_BG3_SCROLL_FLAGS),
        ("f_bgscrollvert", F_BGSCROLLVERT),
        ("v_scrshiftx", V_SCRSHIFTX),
        ("v_scrshifty", V_SCRSHIFTY),
        ("v_lookshift", V_LOOKSHIFT),
        ("v_limitleft1", V_LIMITLEFT1),
        ("v_limitright1", V_LIMITRIGHT1),
        ("v_limittop1", V_LIMITTOP1),
        ("v_limitbtm1", V_LIMITBTM1),
        ("v_limitleft2", V_LIMITLEFT2),
        ("v_limitright2", V_LIMITRIGHT2),
        ("v_limittop2", V_LIMITTOP2),
        ("v_limitbtm2", V_LIMITBTM2),
        ("v_limitleft3", V_LIMITLEFT3),
        ("v_screenposy", V_SCREENPOSY),
        ("v_bgscreenposx", V_BGSCREENPOSX),
        ("v_bgscreenposy", V_BGSCREENPOSY),
        ("v_bg2screenposx", V_BG2SCREENPOSX),
        ("v_bg2screenposy", V_BG2SCREENPOSY),
        ("v_bg3screenposx", V_BG3SCREENPOSX),
        ("v_bg3screenposy", V_BG3SCREENPOSY),
        ("v_bgscrposy_vdp", V_BGSCRPOSY_VDP),
        ("v_256loop1", V_256LOOP1),
        ("v_bgscroll_buffer", V_BG_SCROLL_BUFFER),
        ("v_16x16", V_16X16),
        ("v_256x256", V_256X256),
        ("v_lvllayout", V_LVLLAYOUT),
        ("v_lvllayout_fg", V_LVLLAYOUT_FG),
        ("v_lvllayout_bg", V_LVLLAYOUT_BG),
        ("v_zone", V_ZONE),
        ("v_gamemode", V_GAMEMODE),
    ];
    for (nome, valor) in ram {
        assert_eq!(u64::from(valor), m[nome] & 0xFF_FFFF, "{nome}");
    }
    let rom = [
        ("LevelSizeArray", LEVEL_SIZE_ARRAY),
        ("StartLocArray", START_LOC_ARRAY),
        ("LoopChunkNums", LOOP_CHUNK_NUMS),
        ("Level_Index", LEVEL_INDEX),
        ("Blk16_GHZ", BLK16_GHZ),
        ("Blk256_GHZ", BLK256_GHZ),
        ("Nem_TitleFg", NEM_TITLE_FG),
        ("Nem_TitleSonic", NEM_TITLE_SONIC),
        ("Nem_TitleTM", NEM_TITLE_TM),
        ("Art_Text", ART_TEXT),
        ("Art_Text_end", ART_TEXT_END),
    ];
    for (nome, valor) in rom {
        assert_eq!(u64::from(valor), m[nome], "{nome}");
    }
}
