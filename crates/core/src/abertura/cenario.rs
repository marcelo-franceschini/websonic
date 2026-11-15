//! Cenário da fase e do título (sub-etapa 7.5, parte B): `LevelSizeLoad`, `BgScrollSpeed`,
//! `DeformLayers` (fundos da GHZ e MZ), `ScrollHoriz`, `ScrollVertical`, `DynamicLevelEvents` e
//! `LevelLayoutLoad`. Cada função cita o rótulo que reproduz. Referência REV01, com `FixBugs = 0`.
//!
//! As contas seguem o 68000 em 16 e 32 bits: `wrapping_*` onde o processador dá a volta, e
//! comparações com ou sem sinal conforme o desvio que o original usa. Outras zonas devolvem
//! [`ErroAbertura::NaoImplementado`].

use super::boot::{rom_b, rom_w};
use super::ram::*;
use super::{ErroAbertura, objetos, plc, rb, rl, rw, wb, wl, ww};
use crate::memoria::Memoria;

/// Reinterpreta uma palavra como com sinal, sem mudar nenhum bit.
fn com_sinal(v: u16) -> i16 {
    i16::from_be_bytes(v.to_be_bytes())
}

/// Reinterpreta uma longa como com sinal, sem mudar nenhum bit.
fn longa_com_sinal(v: u32) -> i32 {
    i32::from_be_bytes(v.to_be_bytes())
}

/// Reinterpreta uma longa com sinal como sem sinal, sem mudar nenhum bit.
fn longa_sem_sinal(v: i32) -> u32 {
    u32::from_be_bytes(v.to_be_bytes())
}

/// A palavra baixa de uma longa.
fn baixa(v: u32) -> u16 {
    u16::try_from(v & 0xFFFF).unwrap_or(0)
}

/// `bset` num byte de memória: o número do bit vale módulo 8.
fn bset(m: &mut Memoria, endereco: u32, bit: u32) {
    let b = rb(m, endereco);
    wb(m, endereco, b | 1u8.wrapping_shl(bit & 7));
}

/// O byte de um bit de `andi.w #$10` depois de `move.b`/`eor.b`: 0 ou $10.
fn bit_de_bloco(v: u16) -> u8 {
    u8::try_from(v & 0x10).unwrap_or(0)
}

/// Zona (byte alto de `v_zone_act`) e ato (dois bits baixos do byte baixo).
fn zona_e_ato(m: &mut Memoria) -> (u32, u32) {
    let za = rw(m, V_ZONE_ACT);
    (u32::from(za >> 8), u32::from(za & 0xFF) & 3)
}

/// `LevelSizeLoad`: limites, posição de partida de Sonic, câmera, fundo e bloco de laço da fase.
/// Só o caminho sem lampião e sem créditos, que é o do título e da primeira demo.
pub fn level_size_load(m: &mut Memoria) -> Result<(), ErroAbertura> {
    for v in [V_UNUSED7, V_UNUSED8, V_UNUSED9, V_UNUSED10, V_DLE_ROUTINE] {
        wb(m, v, 0);
    }
    // `f_nobgscroll` só é limpo com FixBugs, que a referência não usa.
    let (zona, ato) = zona_e_ato(m);
    // Entradas de 12 bytes, quatro atos por zona: índice = zona * 48 + ato * 12.
    let e = LEVEL_SIZE_ARRAY
        .wrapping_add(zona.wrapping_mul(48))
        .wrapping_add(ato.wrapping_mul(12));
    let v11 = rom_w(m, e);
    ww(m, V_UNUSED11, v11);
    let esquerda = rom_w(m, e.wrapping_add(2));
    let direita = rom_w(m, e.wrapping_add(4));
    let topo = rom_w(m, e.wrapping_add(6));
    let base = rom_w(m, e.wrapping_add(8));
    let olhar = rom_w(m, e.wrapping_add(10));
    let par_x = (u32::from(esquerda)).wrapping_shl(16) | u32::from(direita);
    let par_y = (u32::from(topo)).wrapping_shl(16) | u32::from(base);
    wl(m, V_LIMITLEFT2, par_x);
    wl(m, V_LIMITLEFT1, par_x);
    wl(m, V_LIMITTOP2, par_y);
    wl(m, V_LIMITTOP1, par_y);
    ww(m, V_LIMITLEFT3, esquerda.wrapping_add(0x240));
    // Dispara o desenho de uma coluna e de uma linha no primeiro quadro.
    ww(m, V_FG_XBLOCK, 0x1010);
    ww(m, V_LOOKSHIFT, olhar);

    let (x, y) = if rb(m, V_LASTLAMP) != 0 {
        super::lamppost::carregar(m);
        (rw(m, V_PLAYER + OB_X), rw(m, V_PLAYER + OB_Y))
    } else {
        if rw(m, F_DEMO) & 0x8000 != 0 {
            return Err(ErroAbertura::NaoImplementado("posição dos créditos"));
        }
        // `.getStartLocEntry`: quatro bytes por zona e ato (X e Y de Sonic).
        let loc = START_LOC_ARRAY
            .wrapping_add(zona.wrapping_mul(16))
            .wrapping_add(ato.wrapping_mul(4));
        let x = rom_w(m, loc);
        let y = rom_w(m, loc.wrapping_add(2));
        ww(m, V_PLAYER + OB_X, x);
        ww(m, V_PLAYER + OB_Y, y);
        (x, y)
    };

    // `LevSz_InitCameraPositions`: Sonic no centro, sem passar dos limites.
    let cx = if x >= 160 { x.wrapping_sub(160) } else { 0 };
    let direita_limite = rw(m, V_LIMITRIGHT2);
    let cx = if cx < direita_limite {
        cx
    } else {
        direita_limite
    };
    ww(m, V_SCREENPOSX, cx);
    let cy = if y >= 96 { y.wrapping_sub(96) } else { 0 };
    let fundo_limite = rw(m, V_LIMITBTM2);
    let cy = if com_sinal(cy) < com_sinal(fundo_limite) {
        cy
    } else {
        fundo_limite
    };
    ww(m, V_SCREENPOSY, cy);

    bg_scroll_speed(m, cy, cx)?;
    // `LoopChunkNums`: quatro bytes por zona, lidos como uma longa.
    let loop_zona = rl(m, LOOP_CHUNK_NUMS.wrapping_add(zona.wrapping_mul(4)));
    wl(m, V_256LOOP1, loop_zona);
    Ok(())
}

/// `BgScrollSpeed` (REV01): posição inicial do fundo e zeragem da rolagem da GHZ.
fn bg_scroll_speed(m: &mut Memoria, cy: u16, cx: u16) -> Result<(), ErroAbertura> {
    if rb(m, V_LASTLAMP) == 0 {
        ww(m, V_BGSCREENPOSY, cy);
        ww(m, V_BG2SCREENPOSY, cy);
        ww(m, V_BG3SCREENPOSY, cy);
        ww(m, V_BGSCREENPOSX, cx);
        ww(m, V_BG2SCREENPOSX, cx);
        ww(m, V_BG3SCREENPOSX, cx);
    }
    if rb(m, V_ZONE) == ID_LZ {
        super::labirinto_cenario::iniciar_fundo(m, cy);
        return Ok(());
    }
    if rb(m, V_ZONE) == ID_MZ {
        return Ok(());
    }
    if rb(m, V_ZONE) != 0 {
        return Err(ErroAbertura::NaoImplementado("fundo de outra zona"));
    }
    // `BgScroll_GHZ`: força as posições do fundo a zero e limpa o buffer das nuvens.
    wl(m, V_BGSCREENPOSX, 0);
    wl(m, V_BGSCREENPOSY, 0);
    wl(m, V_BG2SCREENPOSY, 0);
    wl(m, V_BG3SCREENPOSY, 0);
    for k in 0..3u32 {
        wl(m, V_BG_SCROLL_BUFFER.wrapping_add(k.wrapping_mul(4)), 0);
    }
    Ok(())
}

/// `DeformLayers` (REV01): rolagem da câmera, limites da fase e deformação do fundo.
pub fn deform_layers(m: &mut Memoria) -> Result<(), ErroAbertura> {
    if rb(m, F_NOBGSCROLL) != 0 {
        return Ok(());
    }
    for v in [
        V_FG_SCROLL_FLAGS,
        V_BG1_SCROLL_FLAGS,
        V_BG2_SCROLL_FLAGS,
        V_BG3_SCROLL_FLAGS,
    ] {
        ww(m, v, 0);
    }
    scroll_horiz(m);
    scroll_vertical(m);
    dynamic_level_events(m)?;
    let y = rw(m, V_SCREENPOSY);
    ww(m, V_SCRPOSY_VDP, y);
    let by = rw(m, V_BGSCREENPOSY);
    ww(m, V_BGSCRPOSY_VDP, by);
    match rb(m, V_ZONE) {
        0 => deform_ghz(m),
        ID_LZ => super::labirinto_cenario::deformar(m),
        ID_MZ => deform_mz(m),
        _ => return Err(ErroAbertura::NaoImplementado("deformação de outra zona")),
    }
    Ok(())
}

/// `ScrollHoriz`: move a câmera com Sonic e marca a coluna nova do bloco quando ela cruza uma.
fn scroll_horiz(m: &mut Memoria) {
    let antes = rw(m, V_SCREENPOSX);
    move_screen_horiz(m);
    let depois = rw(m, V_SCREENPOSX);
    let bloco = rb(m, V_FG_XBLOCK);
    // `eor.b` com `v_fg_xblock`: só há cruzamento quando o bit bate com o esperado.
    if bit_de_bloco(depois) != bloco {
        return;
    }
    wb(m, V_FG_XBLOCK, bloco ^ 0x10);
    if com_sinal(depois.wrapping_sub(antes)) >= 0 {
        bset(m, V_FG_SCROLL_FLAGS, 3);
    } else {
        bset(m, V_FG_SCROLL_FLAGS, 2);
    }
}

/// `MoveScreenHoriz` (sem FixBugs): a câmera segue Sonic fora da zona central de 32 pixels.
fn move_screen_horiz(m: &mut Memoria) {
    let scr = rw(m, V_SCREENPOSX);
    let distancia = rw(m, V_PLAYER + OB_X).wrapping_sub(scr);
    // `subi.w #(320/2)-16` com o desvio sem sinal `bcs`.
    if distancia < 144 {
        return move_camera_left(m, distancia.wrapping_sub(144));
    }
    let d0 = distancia.wrapping_sub(144);
    // `subi.w #16` e `bcc`: dentro da zona central a câmera não anda.
    if d0 < 16 {
        ww(m, V_SCRSHIFTX, 0);
        return;
    }
    move_camera_right(m, d0.wrapping_sub(16));
}

/// `SH_MoveCameraRight`: anda até 16 pixels para a direita, sem passar do limite.
fn move_camera_right(m: &mut Memoria, d0: u16) {
    let passo = if d0 < 16 { d0 } else { 16 };
    let nova = passo.wrapping_add(rw(m, V_SCREENPOSX));
    let limite = rw(m, V_LIMITRIGHT2);
    let nova = if com_sinal(nova) < com_sinal(limite) {
        nova
    } else {
        limite
    };
    ajustar_camera_x(m, nova);
}

/// `SH_MoveCameraLeft`: anda para a esquerda sem passar do limite, sem limite de passo.
fn move_camera_left(m: &mut Memoria, d0: u16) {
    let nova = d0.wrapping_add(rw(m, V_SCREENPOSX));
    let limite = rw(m, V_LIMITLEFT2);
    let nova = if com_sinal(nova) > com_sinal(limite) {
        nova
    } else {
        limite
    };
    ajustar_camera_x(m, nova);
}

/// `SH_SetScreen`: grava a posição nova e o deslocamento do quadro, em 8.8.
fn ajustar_camera_x(m: &mut Memoria, nova: u16) {
    let desloc = nova.wrapping_sub(rw(m, V_SCREENPOSX)).wrapping_shl(8);
    ww(m, V_SCREENPOSX, nova);
    ww(m, V_SCRSHIFTX, desloc);
}

/// `ScrollVertical`: a câmera acompanha Sonic com zonas mortas de 6, 2 ou 16 pixels e no ar.
fn scroll_vertical(m: &mut Memoria) {
    let status = rb(m, V_PLAYER + OB_STATUS);
    let mut d0 = rw(m, V_PLAYER + OB_Y).wrapping_sub(rw(m, V_SCREENPOSY));
    // `subq.w #sonic_height-sonic_roll_height`: 19 - 14 pixels ao rolar.
    if status & 4 != 0 {
        d0 = d0.wrapping_sub(5);
    }
    if status & 2 != 0 {
        // No ar, a zona central é de 32 pixels acima e abaixo do deslocamento `v_lookshift`.
        d0 = d0.wrapping_add(32);
        let olhar = rw(m, V_LOOKSHIFT);
        let abaixo = d0 < olhar;
        d0 = d0.wrapping_sub(olhar);
        if abaixo {
            return variante_de_rolagem(m, d0, 16, 16);
        }
        let dentro = d0 < 64;
        d0 = d0.wrapping_sub(64);
        if !dentro {
            return variante_de_rolagem(m, d0, 16, 16);
        }
    } else {
        d0 = d0.wrapping_sub(rw(m, V_LOOKSHIFT));
        if d0 != 0 {
            return fora_do_meio(m, d0);
        }
    }
    if rb(m, F_BGSCROLLVERT) != 0 {
        // `SV_BottomBoundaryMoving`: o limite de baixo está se mexendo, então Sonic fica no meio.
        wb(m, F_BGSCROLLVERT, 0);
        return ponto_doce(m, 0);
    }
    // `SV_NoUpdate`
    ww(m, V_SCRSHIFTY, 0);
}

/// `SV_OutsideMid`: escolhe a velocidade da câmera pelo deslocamento e pela velocidade de Sonic.
fn fora_do_meio(m: &mut Memoria, d0: u16) {
    if rw(m, V_LOOKSHIFT) != 0x60 {
        return variante_de_rolagem(m, d0, 2, 2);
    }
    let mut velocidade = rw(m, V_PLAYER + OB_INERTIA);
    if com_sinal(velocidade) < 0 {
        velocidade = 0u16.wrapping_sub(velocidade);
    }
    if velocidade >= 0x800 {
        return variante_de_rolagem(m, d0, 16, 16);
    }
    variante_de_rolagem(m, d0, 6, 6);
}

/// `SV_ScrollSlow`, `SV_ScrollMedium` e `SV_ScrollFast`: rola `passo` pixels se Sonic sair de
/// `limite` pixels do meio. Dentro do limite a câmera fica como está (`SV_SweetSpot`).
fn variante_de_rolagem(m: &mut Memoria, d0: u16, passo: u16, limite: i16) {
    let velocidade = passo.wrapping_shl(8);
    let distancia = com_sinal(d0);
    if distancia > limite {
        return descer(m, velocidade);
    }
    if distancia < limite.wrapping_neg() {
        return subir(m, velocidade);
    }
    ponto_doce(m, d0);
}

/// `SV_MoveCameraDown`: soma a velocidade à câmera e segue para o limite de baixo.
fn descer(m: &mut Memoria, velocidade: u16) {
    let passo = longa_sem_sinal(i32::from(com_sinal(velocidade)).wrapping_shl(8));
    let soma = passo.wrapping_add(rl(m, V_SCREENPOSY));
    limite_inferior(m, soma.rotate_left(16));
}

/// `SV_MoveCameraUp`: subtrai a velocidade da câmera e segue para o limite de cima.
fn subir(m: &mut Memoria, velocidade: u16) {
    let negativo = 0u16.wrapping_sub(velocidade);
    let passo = longa_sem_sinal(i32::from(com_sinal(negativo)).wrapping_shl(8));
    let soma = passo.wrapping_add(rl(m, V_SCREENPOSY));
    limite_superior(m, soma.rotate_left(16));
}

/// `SV_SweetSpot`: a câmera vai para `d0` mais a posição atual, com o limite do lado de Sonic.
fn ponto_doce(m: &mut Memoria, d0: u16) {
    let d1 = u32::from(d0.wrapping_add(rw(m, V_SCREENPOSY)));
    if com_sinal(d0) >= 0 {
        limite_inferior(m, d1);
    } else {
        limite_superior(m, d1);
    }
}

/// `SV_TopBoundary`: não sobe acima do limite de cima. A fase pode dar a volta (`-$100`).
fn limite_superior(m: &mut Memoria, d1: u32) {
    let topo = rw(m, V_LIMITTOP2);
    let mut baixa_d1 = baixa(d1);
    if com_sinal(baixa_d1) > com_sinal(topo) {
        return ajustar_y(m, d1);
    }
    if com_sinal(baixa_d1) > -0x100 {
        // `.noWrap`: prende no limite de cima.
        return ajustar_y(m, substituir_baixa(d1, topo));
    }
    // A fase dá a volta verticalmente: `andi.w #$7FF` nas três posições.
    baixa_d1 &= 0x7FF;
    let y_sonic = rw(m, V_PLAYER + OB_Y) & 0x7FF;
    ww(m, V_PLAYER + OB_Y, y_sonic);
    let y_camera = rw(m, V_SCREENPOSY) & 0x7FF;
    ww(m, V_SCREENPOSY, y_camera);
    let y_fundo = rw(m, V_BGSCREENPOSY) & 0x3FF;
    ww(m, V_BGSCREENPOSY, y_fundo);
    ajustar_y(m, substituir_baixa(d1, baixa_d1))
}

/// `SV_BottomBoundary`: não desce abaixo do limite de baixo. A fase dá a volta em $800.
fn limite_inferior(m: &mut Memoria, d1: u32) {
    let fundo = rw(m, V_LIMITBTM2);
    let baixa_d1 = baixa(d1);
    if com_sinal(baixa_d1) < com_sinal(fundo) {
        return ajustar_y(m, d1);
    }
    if baixa_d1 < 0x800 {
        // `.noWrap`: prende no limite de baixo.
        return ajustar_y(m, substituir_baixa(d1, fundo));
    }
    let nova = baixa_d1.wrapping_sub(0x800);
    let y_sonic = rw(m, V_PLAYER + OB_Y) & 0x7FF;
    ww(m, V_PLAYER + OB_Y, y_sonic);
    let y_camera = rw(m, V_SCREENPOSY).wrapping_sub(0x800);
    ww(m, V_SCREENPOSY, y_camera);
    let y_fundo = rw(m, V_BGSCREENPOSY) & 0x3FF;
    ww(m, V_BGSCREENPOSY, y_fundo);
    ajustar_y(m, substituir_baixa(d1, nova))
}

/// Troca a palavra baixa de uma longa, como `move.w` sobre `d1`.
fn substituir_baixa(d1: u32, nova: u16) -> u32 {
    (d1 & 0xFFFF_0000) | u32::from(nova)
}

/// `SV_SetScreen`: grava a posição vertical da câmera e marca a linha nova do bloco, se houver.
fn ajustar_y(m: &mut Memoria, d1: u32) {
    let antes = rw(m, V_SCREENPOSY);
    let nova = d1.rotate_left(16);
    let diferenca = nova.wrapping_sub(rl(m, V_SCREENPOSY));
    let desloc = baixa(diferenca.rotate_right(8));
    ww(m, V_SCRSHIFTY, desloc);
    wl(m, V_SCREENPOSY, nova);
    let agora = rw(m, V_SCREENPOSY);
    let bloco = rb(m, V_FG_YBLOCK);
    if bit_de_bloco(agora) != bloco {
        return;
    }
    wb(m, V_FG_YBLOCK, bloco ^ 0x10);
    if com_sinal(agora.wrapping_sub(antes)) >= 0 {
        bset(m, V_FG_SCROLL_FLAGS, 1);
    } else {
        bset(m, V_FG_SCROLL_FLAGS, 0);
    }
}

/// `DynamicLevelEvents`: atualiza os eventos da GHZ e aproxima o limite vertical do alvo.
fn dynamic_level_events(m: &mut Memoria) -> Result<(), ErroAbertura> {
    let zona = rb(m, V_ZONE);
    if zona == ID_LZ {
        super::labirinto::eventos(m)?;
    } else if zona == ID_MZ {
        eventos_mz(m)?;
    } else if zona != 0 {
        return Err(ErroAbertura::NaoImplementado("eventos de outra zona"));
    } else {
        let x = rw(m, V_SCREENPOSX);
        match rb(m, V_ACT) {
            0 => ww(m, V_LIMITBTM1, if x >= 0x1780 { 0x400 } else { 0x300 }),
            1 => {
                let limite = if !(0xED0..0x1D60).contains(&x) {
                    0x300
                } else if x < 0x1600 {
                    0x200
                } else {
                    0x400
                };
                ww(m, V_LIMITBTM1, limite);
            }
            2 => match rb(m, V_DLE_ROUTINE) {
                0 => {
                    let mut limite = if x < 0x380 { 0x300 } else { 0x310 };
                    if x >= 0x960 {
                        if rw(m, V_SCREENPOSY) < 0x280 || x >= 0x1700 {
                            limite = 0x300;
                            wb(m, V_DLE_ROUTINE, 2);
                        } else if x < 0x1380 {
                            limite = 0x4C0;
                            ww(m, V_LIMITBTM2, limite);
                        } else {
                            limite = 0x400;
                        }
                    }
                    ww(m, V_LIMITBTM1, limite);
                }
                2 => {
                    if x < 0x960 {
                        wb(m, V_DLE_ROUTINE, 0);
                    }
                    if x >= 0x2960 {
                        if let Some(a) = objetos::find_free_obj(m) {
                            wb(m, a.wrapping_add(OB_ID), 0x3D);
                            ww(m, a.wrapping_add(OB_X), 0x2A60);
                            ww(m, a.wrapping_add(OB_Y), 0x280);
                        }
                        wb(m, V_SOUNDQUEUE0, 0x8C);
                        wb(m, F_LOCKSCREEN, 1);
                        let rotina = rb(m, V_DLE_ROUTINE).wrapping_add(2);
                        wb(m, V_DLE_ROUTINE, rotina);
                        plc::add_plc(m, 0x11)?;
                    }
                }
                4 => ww(m, V_LIMITLEFT2, x),
                _ => return Err(ErroAbertura::NaoImplementado("rotina de eventos da GHZ3")),
            },
            _ => return Err(ErroAbertura::NaoImplementado("ato da GHZ")),
        }
    }
    let alvo = rw(m, V_LIMITBTM1);
    let atual = rw(m, V_LIMITBTM2);
    if alvo == atual {
        return Ok(());
    }
    if alvo > atual {
        // `.move_boundary_down`: 8 pixels no ar quando a câmera já está perto do limite.
        let mut passo: u16 = 2;
        let y_mais = rw(m, V_SCREENPOSY).wrapping_add(8);
        if y_mais >= atual && rb(m, V_PLAYER + OB_STATUS) & 2 != 0 {
            passo = passo.wrapping_shl(2);
        }
        ww(m, V_LIMITBTM2, atual.wrapping_add(passo));
    } else {
        // `.move_boundary_down` ao contrário: sobe 2 pixels e encosta na câmera se ela passou.
        let y = rw(m, V_SCREENPOSY);
        let mut nova = atual;
        if y > alvo {
            nova = y & 0xFFFE;
        }
        ww(m, V_LIMITBTM2, nova.wrapping_sub(2));
    }
    wb(m, F_BGSCROLLVERT, 1);
    Ok(())
}

/// Limites e transições dos eventos dinâmicos da Marble Zone.
fn eventos_mz(m: &mut Memoria) -> Result<(), ErroAbertura> {
    let x = rw(m, V_SCREENPOSX);
    let y = rw(m, V_SCREENPOSY);
    let rotina = rb(m, V_DLE_ROUTINE);
    match rb(m, V_ACT) {
        0 => match rotina {
            0 => {
                let limite = if x < 0x700 {
                    0x1D0
                } else if x < 0xD00 {
                    0x220
                } else {
                    0x340
                };
                ww(m, V_LIMITBTM1, limite);
                if x >= 0xD00 && y >= 0x340 {
                    wb(m, V_DLE_ROUTINE, 2);
                }
            }
            2 => {
                if y < 0x340 {
                    wb(m, V_DLE_ROUTINE, 0);
                } else {
                    ww(m, V_LIMITTOP2, if x < 0xE00 { 0x340 } else { 0 });
                    if x < 0xE00 {
                        ww(m, V_LIMITBTM1, if x < 0xA90 { 0x500 } else { 0x340 });
                        if x < 0xA90 && y >= 0x370 {
                            wb(m, V_DLE_ROUTINE, 4);
                        }
                    }
                }
            }
            4 => {
                if y < 0x370 {
                    wb(m, V_DLE_ROUTINE, 2);
                } else if y >= 0x500 && x >= 0xB80 {
                    ww(m, V_LIMITTOP2, 0x500);
                    wb(m, V_DLE_ROUTINE, 6);
                }
            }
            6 => {
                if x < 0xB80 {
                    let topo = rw(m, V_LIMITTOP2);
                    if topo != 0x340 {
                        ww(m, V_LIMITTOP2, topo.wrapping_sub(2));
                    }
                } else {
                    if rw(m, V_LIMITTOP2) != 0x500 {
                        if y < 0x500 {
                            return Ok(());
                        }
                        ww(m, V_LIMITTOP2, 0x500);
                    }
                    if x >= 0xE70 {
                        ww(m, V_LIMITTOP2, 0);
                        ww(m, V_LIMITBTM1, if x < 0x1430 { 0x500 } else { 0x210 });
                    }
                }
            }
            _ => return Err(ErroAbertura::NaoImplementado("rotina de eventos da MZ1")),
        },
        1 => ww(m, V_LIMITBTM1, if x < 0x1700 { 0x520 } else { 0x200 }),
        2 => match rotina {
            0 => {
                ww(m, V_LIMITBTM1, if x < 0x1560 { 0x720 } else { 0x210 });
                if x >= 0x17F0 {
                    if let Some(a) = objetos::find_free_obj(m) {
                        wb(m, a.wrapping_add(OB_ID), 0x73);
                        ww(m, a.wrapping_add(OB_X), 0x19F0);
                        ww(m, a.wrapping_add(OB_Y), 0x22C);
                    }
                    wb(m, V_SOUNDQUEUE0, 0x8C);
                    wb(m, F_LOCKSCREEN, 1);
                    wb(m, V_DLE_ROUTINE, 2);
                    plc::add_plc(m, 0x11)?;
                }
            }
            2 => ww(m, V_LIMITLEFT2, x),
            _ => return Err(ErroAbertura::NaoImplementado("rotina de eventos da MZ3")),
        },
        _ => return Err(ErroAbertura::NaoImplementado("ato da MZ")),
    }
    Ok(())
}

/// `Deform_GHZ` (REV01): o fundo da GHZ, com as nuvens que rolam sozinhas e a água.
fn deform_ghz(m: &mut Memoria) {
    let desloc = i32::from(com_sinal(rw(m, V_SCRSHIFTX)));
    // Bloco 3 (montanhas distantes) anda $60 vezes o deslocamento; o bloco 2 (colinas), $80.
    bloco_de_fundo(
        m,
        V_BG3SCREENPOSX,
        V_BG3_XBLOCK,
        V_BG3_SCROLL_FLAGS,
        desloc.wrapping_mul(96),
        0,
    );
    bloco_de_fundo(
        m,
        V_BG2SCREENPOSX,
        V_BG2_XBLOCK,
        V_BG2_SCROLL_FLAGS,
        desloc.wrapping_mul(128),
        0,
    );

    // Posição vertical do fundo, limitada a zero.
    let mut dy = (rw(m, V_SCREENPOSY) & 0x7FF) >> 5;
    dy = 0u16.wrapping_sub(dy).wrapping_add(0x20);
    if com_sinal(dy) < 0 {
        dy = 0;
    }
    ww(m, V_BGSCRPOSY_VDP, dy);

    // A palavra alta de cada linha é a rolagem da frente (zero na tela do título).
    let frente = if rb(m, V_GAMEMODE) == GM_TITLE_ID {
        0
    } else {
        rw(m, V_SCREENPOSX)
    };
    let alta = u32::from(0u16.wrapping_sub(frente)).wrapping_shl(16);

    // Nuvens: cada uma anda pelo próprio passo a cada quadro.
    let buffer = V_BG_SCROLL_BUFFER;
    somar_longa(m, buffer, 0x1_0000);
    somar_longa(m, buffer.wrapping_add(4), 0xC000);
    somar_longa(m, buffer.wrapping_add(8), 0x8000);

    let bg3 = rw(m, V_BG3SCREENPOSX);
    let bg2 = rw(m, V_BG2SCREENPOSX);
    let mut a = V_HSCROLLTABLEBUFFER;
    // Nuvem superior: 32 linhas menos as que o fundo já ocupa. Ausente se `dy` passa de 31.
    if dy <= 31 {
        let base = 0u16.wrapping_sub(rw(m, buffer).wrapping_add(bg3));
        for _ in 0..32u16.wrapping_sub(dy) {
            a = linha(m, a, alta, base);
        }
    }
    let base = 0u16.wrapping_sub(rw(m, buffer.wrapping_add(4)).wrapping_add(bg3));
    for _ in 0..16 {
        a = linha(m, a, alta, base);
    }
    let base = 0u16.wrapping_sub(rw(m, buffer.wrapping_add(8)).wrapping_add(bg3));
    for _ in 0..16 {
        a = linha(m, a, alta, base);
    }
    // Montanhas (48 linhas) e colinas (40 linhas).
    let base = 0u16.wrapping_sub(bg3);
    for _ in 0..48 {
        a = linha(m, a, alta, base);
    }
    let base = 0u16.wrapping_sub(bg2);
    for _ in 0..40 {
        a = linha(m, a, alta, base);
    }

    // Água: 72 linhas mais `dy`, com a posição do fundo andando um passo fracionado por linha.
    let diferenca = rw(m, V_SCREENPOSX).wrapping_sub(bg2);
    let dividendo = i32::from(com_sinal(diferenca)).wrapping_shl(8);
    // `divs.w #$68`: se o quociente não cabe em 16 bits, o 68000 deixa o dividendo como está.
    let d2 = match dividendo
        .checked_div(104)
        .and_then(|q| i16::try_from(q).ok())
    {
        Some(q) => i32::from(q),
        None => i32::from(com_sinal(baixa(longa_sem_sinal(dividendo)))),
    }
    .wrapping_shl(8);
    let mut d3 = u32::from(bg2);
    for _ in 0..72u32.wrapping_add(u32::from(dy)) {
        let base = 0u16.wrapping_sub(baixa(d3));
        a = linha(m, a, alta, base);
        d3 = d3
            .rotate_left(16)
            .wrapping_add(longa_sem_sinal(d2))
            .rotate_left(16);
    }
}

/// `Deform_MZ`: três velocidades horizontais e faixas de fundo de 16 linhas.
fn deform_mz(m: &mut Memoria) {
    let desloc = i32::from(com_sinal(rw(m, V_SCRSHIFTX)));
    for (pos, bloco, flags, fator, bit) in [
        (V_BGSCREENPOSX, 0xFF_F74C, V_BG1_SCROLL_FLAGS, 192, 2),
        (V_BG3SCREENPOSX, V_BG3_XBLOCK, V_BG3_SCROLL_FLAGS, 64, 6),
        (V_BG2SCREENPOSX, V_BG2_XBLOCK, V_BG2_SCROLL_FLAGS, 128, 4),
    ] {
        bloco_de_fundo(m, pos, bloco, flags, desloc.wrapping_mul(fator), bit);
    }
    let y = rw(m, V_SCREENPOSY);
    let fundo_y = if y < 456 {
        512
    } else {
        let diferenca = y.wrapping_sub(456);
        let triplo = diferenca.wrapping_mul(3);
        512u16.wrapping_add(u16::from_be_bytes((com_sinal(triplo) >> 2).to_be_bytes()))
    };
    ww(m, V_BG2SCREENPOSY, fundo_y);
    ww(m, V_BG3SCREENPOSY, fundo_y);
    let anterior = rw(m, V_BGSCREENPOSY);
    ww(m, V_BGSCREENPOSY, fundo_y);
    let bloco = rb(m, 0xFF_F74D);
    if bit_de_bloco(fundo_y) == bloco {
        wb(m, 0xFF_F74D, bloco ^ 0x10);
        bset(
            m,
            V_BG1_SCROLL_FLAGS,
            if com_sinal(fundo_y.wrapping_sub(anterior)) < 0 {
                0
            } else {
                1
            },
        );
    }
    ww(m, V_BGSCRPOSY_VDP, fundo_y);
    let flags = rb(m, V_BG1_SCROLL_FLAGS) | rb(m, V_BG2_SCROLL_FLAGS) | rb(m, V_BG3_SCROLL_FLAGS);
    wb(m, V_BG3_SCROLL_FLAGS, flags);
    wb(m, V_BG1_SCROLL_FLAGS, 0);
    wb(m, V_BG2_SCROLL_FLAGS, 0);

    let camera = rw(m, V_SCREENPOSX);
    let negativo = 0u16.wrapping_sub(camera);
    let quarto = com_sinal(negativo) >> 2;
    let delta = u16::from_be_bytes(quarto.to_be_bytes()).wrapping_sub(negativo);
    let passo = i32::from(com_sinal(delta))
        .wrapping_shl(3)
        .wrapping_div(5)
        .wrapping_shl(12);
    let mut d3 = u32::from(u16::from_be_bytes((com_sinal(negativo) >> 1).to_be_bytes()));
    let mut a = V_BG_SCROLL_BUFFER;
    for _ in 0..5 {
        ww(m, a, baixa(d3));
        a = a.wrapping_add(2);
        d3 = d3
            .rotate_left(16)
            .wrapping_add(longa_sem_sinal(passo))
            .rotate_left(16);
    }
    for (pos, quantidade) in [
        (V_BG3SCREENPOSX, 2),
        (V_BG2SCREENPOSX, 9),
        (V_BGSCREENPOSX, 16),
    ] {
        let valor = 0u16.wrapping_sub(rw(m, pos));
        for _ in 0..quantidade {
            ww(m, a, valor);
            a = a.wrapping_add(2);
        }
    }
    let desloc_y = fundo_y.wrapping_sub(512);
    let indice = desloc_y.min(0x100) & 0x1F0;
    let inicio = V_BG_SCROLL_BUFFER.wrapping_add(u32::from(indice >> 3));
    let alta = u32::from(negativo).wrapping_shl(16);
    let mut destino = V_HSCROLLTABLEBUFFER;
    let linhas = 240u16.wrapping_sub(desloc_y & 15);
    for i in 0..linhas {
        let faixa = u32::from((i.wrapping_add(desloc_y & 15)) >> 4);
        let valor = rw(m, inicio.wrapping_add(faixa.wrapping_mul(2)));
        destino = linha(m, destino, alta, valor);
    }
}

/// `addi.l #valor,(endereço)`: soma a longa da RAM.
fn somar_longa(m: &mut Memoria, endereco: u32, valor: u32) {
    let atual = rl(m, endereco);
    wl(m, endereco, atual.wrapping_add(valor));
}

/// Uma linha do buffer de rolagem: palavra alta (frente) e palavra baixa (fundo).
/// Devolve o endereço da linha seguinte.
fn linha(m: &mut Memoria, a: u32, alta: u32, baixa_linha: u16) -> u32 {
    wl(m, a, alta | u32::from(baixa_linha));
    a.wrapping_add(4)
}

/// `BGScroll_Block2` e `BGScroll_Block3` (REV01): soma `d4` à posição do bloco de fundo e marca
/// a direção da coluna nova quando o bit 4 do inteiro muda.
fn bloco_de_fundo(m: &mut Memoria, posicao: u32, bloco_x: u32, flags: u32, d4: i32, bit_base: u32) {
    let antiga = rl(m, posicao);
    let nova = antiga.wrapping_add(longa_sem_sinal(d4));
    wl(m, posicao, nova);
    let bit = u8::try_from((nova >> 16) & 0x10).unwrap_or(0);
    let bloco = rb(m, bloco_x);
    if bit != bloco {
        return;
    }
    wb(m, bloco_x, bloco ^ 0x10);
    // `d6` é 0 nas duas chamadas da GHZ: o bit 0 marca a esquerda e o bit 1, a direita.
    if longa_com_sinal(nova.wrapping_sub(antiga)) >= 0 {
        bset(m, flags, bit_base.wrapping_add(1));
    } else {
        bset(m, flags, bit_base);
    }
}

/// `LevelLayoutLoad` (sem FixBugs): limpa $800 bytes do layout (um a mais que o tamanho real,
/// como no original) e carrega o layout de frente e o de fundo da fase.
/// Devolve o `a0` que sobra no fim da carga do layout de fundo (o fim da última linha copiada),
/// que `LoadTilesFromStart` ainda usa num chunk vazio.
pub fn level_layout_load(m: &mut Memoria) -> Result<u32, ErroAbertura> {
    let mut a = V_LVLLAYOUT;
    for _ in 0..0x200 {
        wl(m, a, 0);
        a = a.wrapping_add(4);
    }
    level_layout_load2(m, V_LVLLAYOUT_FG, 0)?;
    level_layout_load2(m, V_LVLLAYOUT_BG, 2)
}

/// `LevelLayoutLoad2`: uma linha de chunks por vez, com largura e altura da tabela `Level_Index`.
fn level_layout_load2(
    m: &mut Memoria,
    destino: u32,
    deslocamento: u32,
) -> Result<u32, ErroAbertura> {
    let (zona, ato) = zona_e_ato(m);
    // Tabela `Level_Index`: quatro atos por zona, seis bytes por ato (palavras de frente e de
    // fundo). Índice = zona * 24 + ato * 6 + deslocamento (0 para a frente, 2 para o fundo).
    let indice = zona
        .wrapping_mul(24)
        .wrapping_add(ato.wrapping_mul(6))
        .wrapping_add(deslocamento);
    let ponteiro = rom_w(m, LEVEL_INDEX.wrapping_add(indice));
    let mut a1 = LEVEL_INDEX.wrapping_add(u32::from(ponteiro));
    let largura = u32::from(rom_b(m, a1));
    let altura = u32::from(rom_b(m, a1.wrapping_add(1)));
    a1 = a1.wrapping_add(2);
    let mut inicio_linha = destino;
    let mut a0 = destino;
    for _ in 0..=altura {
        a0 = inicio_linha;
        for _ in 0..=largura {
            let b = rom_b(m, a1);
            wb(m, a0, b);
            a0 = a0.wrapping_add(1);
            a1 = a1.wrapping_add(1);
        }
        inicio_linha = inicio_linha.wrapping_add(LAYOUT_ROW);
    }
    Ok(a0)
}

#[cfg(test)]
mod testes_eventos {
    use super::*;

    #[test]
    fn lz_inicializa_fundo_e_deforma_cenario() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        wb(&mut m, V_ZONE, ID_LZ);
        bg_scroll_speed(&mut m, 0x200, 0x100).unwrap();
        assert_eq!(rw(&mut m, V_BGSCREENPOSY), 0x100);
        ww(&mut m, V_PLAYER.wrapping_add(OB_X), 160);
        ww(&mut m, V_PLAYER.wrapping_add(OB_Y), 96);
        ww(&mut m, V_LIMITBTM1, 0x300);
        ww(&mut m, V_LIMITBTM2, 0x300);
        ww(&mut m, 0xFF_F646, 0x200);
        deform_layers(&mut m).unwrap();
        assert_eq!(rw(&mut m, 0xFF_F7D8), 0x80);
        let fundo = rw(&mut m, V_BGSCREENPOSY);
        assert_eq!(rw(&mut m, V_BGSCRPOSY_VDP), fundo);
    }

    #[test]
    fn ghz2_altera_limite_nas_fronteiras() {
        let mut m = Memoria::nova(vec![0; 0x10000]);
        wb(&mut m, V_ACT, 1);
        for (x, esperado) in [
            (0xECF, 0x300),
            (0xED0, 0x200),
            (0x1600, 0x400),
            (0x1D60, 0x300),
        ] {
            ww(&mut m, V_SCREENPOSX, x);
            dynamic_level_events(&mut m).unwrap();
            assert_eq!(rw(&mut m, V_LIMITBTM1), esperado);
        }
    }

    #[test]
    fn ghz3_passagem_subterranea_e_retorno_da_camera() {
        let mut m = Memoria::nova(vec![0; 0x10000]);
        wb(&mut m, V_ACT, 2);
        ww(&mut m, V_SCREENPOSX, 0x960);
        ww(&mut m, V_SCREENPOSY, 0x280);
        dynamic_level_events(&mut m).unwrap();
        assert_eq!(rw(&mut m, V_LIMITBTM2), 0x4C0);
        ww(&mut m, V_SCREENPOSX, 0x1700);
        dynamic_level_events(&mut m).unwrap();
        assert_eq!(rb(&mut m, V_DLE_ROUTINE), 2);
        ww(&mut m, V_SCREENPOSX, 0x95F);
        dynamic_level_events(&mut m).unwrap();
        assert_eq!(rb(&mut m, V_DLE_ROUTINE), 0);
    }

    #[test]
    fn ghz3_aciona_arena_e_acompanha_camera() {
        let mut m = Memoria::nova(vec![0; 0x10000]);
        wb(&mut m, V_ACT, 2);
        wb(&mut m, V_DLE_ROUTINE, 2);
        ww(&mut m, V_SCREENPOSX, 0x2960);
        dynamic_level_events(&mut m).unwrap();
        assert_eq!(rb(&mut m, V_LVLOBJSPACE), 0x3D);
        assert_eq!(rw(&mut m, V_LVLOBJSPACE.wrapping_add(OB_X)), 0x2A60);
        assert_eq!(rw(&mut m, V_LVLOBJSPACE.wrapping_add(OB_Y)), 0x280);
        assert_eq!(rb(&mut m, F_LOCKSCREEN), 1);
        assert_eq!(rb(&mut m, V_SOUNDQUEUE0), 0x8C);
        assert_eq!(rb(&mut m, V_DLE_ROUTINE), 4);
        ww(&mut m, V_SCREENPOSX, 0x2970);
        dynamic_level_events(&mut m).unwrap();
        assert_eq!(rw(&mut m, V_LIMITLEFT2), 0x2970);
    }

    #[test]
    fn mz1_transita_entre_limites_subterraneos() {
        let mut m = Memoria::nova(vec![0; 0x10000]);
        wb(&mut m, V_ZONE, ID_MZ);
        ww(&mut m, V_SCREENPOSX, 0xD00);
        ww(&mut m, V_SCREENPOSY, 0x340);
        dynamic_level_events(&mut m).unwrap();
        assert_eq!(rw(&mut m, V_LIMITBTM1), 0x340);
        assert_eq!(rb(&mut m, V_DLE_ROUTINE), 2);
        ww(&mut m, V_SCREENPOSX, 0xA80);
        ww(&mut m, V_SCREENPOSY, 0x370);
        dynamic_level_events(&mut m).unwrap();
        assert_eq!(rw(&mut m, V_LIMITTOP2), 0x340);
        assert_eq!(rw(&mut m, V_LIMITBTM1), 0x500);
        assert_eq!(rb(&mut m, V_DLE_ROUTINE), 4);
        ww(&mut m, V_SCREENPOSX, 0xB80);
        ww(&mut m, V_SCREENPOSY, 0x500);
        dynamic_level_events(&mut m).unwrap();
        assert_eq!(rw(&mut m, V_LIMITTOP2), 0x500);
        assert_eq!(rb(&mut m, V_DLE_ROUTINE), 6);
    }

    #[test]
    fn mz_fundo_atualiza_posicoes_e_tabela_de_rolagem() {
        let mut m = Memoria::nova(vec![0; 0x10000]);
        ww(&mut m, V_SCREENPOSY, 520);
        ww(&mut m, V_SCREENPOSX, 0x100);
        ww(&mut m, V_SCRSHIFTX, 0x100);
        deform_mz(&mut m);
        assert_eq!(rw(&mut m, V_BGSCREENPOSY), 560);
        assert_eq!(rw(&mut m, V_BGSCRPOSY_VDP), 560);
        assert_eq!(rl(&mut m, V_BGSCREENPOSX), 0xC000);
        assert_eq!(rl(&mut m, V_BG2SCREENPOSX), 0x8000);
        assert_eq!(rl(&mut m, V_BG3SCREENPOSX), 0x4000);
        assert_eq!(rw(&mut m, V_HSCROLLTABLEBUFFER), 0xFF00);
        assert_eq!(rb(&mut m, V_BG1_SCROLL_FLAGS), 0);
        assert_eq!(rb(&mut m, V_BG2_SCROLL_FLAGS), 0);
        assert_ne!(rb(&mut m, V_BG3_SCROLL_FLAGS), 0);
    }
}
