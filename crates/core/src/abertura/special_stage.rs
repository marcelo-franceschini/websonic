//! Preparação dos dados de Special Stage. O laço e a renderização ainda não estão ligados.
use super::boot::{rom_b, rom_l, rom_w};
use super::ram::*;
use super::{ErroAbertura, rb, wb, wl, ww};
use crate::memoria::Memoria;
use crate::rotinas;

const LISTA_ESMERALDAS: u32 = 0xFF_FE58;
const INDICE_LAYOUT: u32 = 0x1BE02;
const POSICOES_INICIAIS: u32 = 0x1BE1A;
const MAPAS: u32 = 0x1BEEE;
const MAPAS_FIM: u32 = 0x1C0C2;
const LAYOUT_BASE: u32 = 0xFF_0000;
const LAYOUT_REAL: u32 = 0xFF_1020;
const TEMPORARIO: u32 = 0xFF_4000;
const ANIMACOES: u32 = 0xFF_4400;
const PAL_SS_NUM: u32 = 0xFF_F79A;
const PAL_SS_TIME: u32 = 0xFF_F79C;
const PAL_SS_INDEX: u32 = 0xFF_F79E;
const SS_BG_ANIM: u32 = 0xFF_F7A0;
const SCRPOSY_VDP: u32 = 0xFF_F616;

fn agendar_animacao(m: &mut Memoria, id: u8, celula: u32) -> Option<u32> {
    for i in 0..32u32 {
        let slot = ANIMACOES.wrapping_add(i.wrapping_mul(8));
        if rb(m, slot) == 0 {
            wb(m, slot, id);
            wl(m, slot.wrapping_add(4), celula);
            return Some(slot);
        }
    }
    None
}

/// `SS_ExecuteAnimationQueue`: atualiza os blocos usando scripts da ROM.
fn animar_itens(m: &mut Memoria) -> Result<(), ErroAbertura> {
    for i in 0..32u32 {
        let slot = ANIMACOES.wrapping_add(i.wrapping_mul(8));
        let id = rb(m, slot);
        if id == 0 {
            continue;
        }
        let (dados, recarga) = match id {
            1 => (0x1BCE8u32, 5),
            2 => (0x1BD1E, 7),
            3 => (0x1BD4E, 5),
            4 => (0x1BD84, 7),
            5 => (0x1BDC4, 5),
            6 => (0x1BDF8, 1),
            _ => return Err(ErroAbertura::NaoImplementado("animacao da Special Stage")),
        };
        let atraso = rb(m, slot.wrapping_add(2)).wrapping_sub(1);
        wb(m, slot.wrapping_add(2), atraso);
        if atraso & 0x80 == 0 {
            continue;
        }
        wb(m, slot.wrapping_add(2), recarga);
        let quadro = rb(m, slot.wrapping_add(3));
        wb(m, slot.wrapping_add(3), quadro.wrapping_add(1));
        let ponteiro = super::rl(m, slot.wrapping_add(4));
        let celula = ponteiro & 0xFF_FFFF;
        let novo = rom_b(m, dados.wrapping_add(u32::from(quadro)));
        if novo == 0 {
            let ultimo = match id {
                2 => 0x25,
                4 => 0x2B,
                6 => rb(m, slot.wrapping_add(4)),
                _ => 0,
            };
            wb(m, celula, ultimo);
            wl(m, slot, 0);
            wl(m, slot.wrapping_add(4), 0);
            if id == 5 {
                wb(m, V_PLAYER.wrapping_add(OB_ROUTINE), 4);
                wb(m, V_SOUNDQUEUE1, 0xA8);
            }
        } else {
            wb(m, celula, novo);
        }
    }
    Ok(())
}

/// `SS_BGLoad`: carrega os quadros de fundo e os mapas de nuvens e bolhas.
pub fn carregar_fundo(m: &mut Memoria) -> Result<(), ErroAbertura> {
    rotinas::eni_dec(m, 0x2D3C6, V_RAM_START, 0x4051).map_err(ErroAbertura::Rotina)?;
    for quadro in 0..7u32 {
        let origem_animal = V_RAM_START
            .wrapping_add(128)
            .wrapping_add(quadro.wrapping_mul(128));
        for linha in 0..4u32 {
            for coluna in 0..8u32 {
                let animal = (quadro < 4) ^ ((linha.wrapping_add(coluna)) & 1 != 0);
                if !animal && quadro != 0 {
                    continue;
                }
                let origem = if animal { origem_animal } else { V_RAM_START };
                let destino = 0x5000_u32
                    .wrapping_add(quadro.wrapping_mul(0x1000))
                    .wrapping_add(linha.wrapping_mul(0x400))
                    .wrapping_add(coluna.wrapping_mul(0x10));
                super::sega::copy_tilemap(m, origem, destino, 8, 8);
            }
        }
    }
    rotinas::eni_dec(m, 0x2E18A, V_RAM_START, 0x4000).map_err(ErroAbertura::Rotina)?;
    super::sega::copy_tilemap(m, V_RAM_START, 0xC000, 64, 32);
    super::sega::copy_tilemap(m, V_RAM_START, 0xD000, 64, 64);
    Ok(())
}

/// Prepara os mapas, arte, estado e áudio antes do aparecimento da Special Stage.
pub fn preparar(m: &mut Memoria) -> Result<(), ErroAbertura> {
    for comando in [0x8B03u16, 0x8004, 0x9011] {
        ww(m, VDP_CONTROLE, comando);
    }
    ww(m, V_HBLANK_HREG, 0x8AAF);
    let registro = super::rw(m, V_VDP_BUFFER1);
    ww(m, VDP_CONTROLE, registro & 0xFFBF);
    super::sega::clear_screen(m)?;
    super::boot::fill_vram(m, 0, 0x5000, 0xC000)?;
    carregar_fundo(m)?;
    let offset = rom_w(m, ART_LOAD_CUES.wrapping_add(0x28));
    let mut entrada = ART_LOAD_CUES.wrapping_add(u32::from(offset));
    let quantidade = rom_w(m, entrada);
    entrada = entrada.wrapping_add(2);
    for _ in 0..=u32::from(quantidade) {
        let arte = rom_l(m, entrada);
        let destino = rom_w(m, entrada.wrapping_add(4));
        wl(
            m,
            VDP_CONTROLE,
            super::titulo::loc_vram(u32::from(destino) / 32),
        );
        rotinas::nem_dec(m, arte).map_err(ErroAbertura::Rotina)?;
        entrada = entrada.wrapping_add(6);
    }
    super::titulo::zerar(m, V_OBJSPACE, V_OBJSPACE_END);
    super::titulo::zerar(m, V_LEVELVARIABLES, V_LEVELVARIABLES_END);
    super::titulo::zerar(m, 0xFF_FE60, 0xFF_FF00);
    super::titulo::zerar(m, 0xFF_AA00, 0xFF_AC00);
    wb(m, F_WTR_STATE, 0);
    ww(m, F_RESTART, 0);
    super::sega::pal_load_para(m, 0x0A, 0x80);
    carregar(m)?;
    wl(m, V_SCREENPOSX, 0);
    wl(m, V_SCREENPOSY, 0);
    wb(m, V_PLAYER.wrapping_add(OB_ID), 9);
    ciclo_paleta(m);
    ww(m, 0xFF_F780, 0);
    ww(m, 0xFF_F782, 0x40);
    wb(m, V_SOUNDQUEUE0, 0x89);
    ww(m, V_BTNPUSHTIME1, 0);
    let demo = rom_l(m, DEMO_DATA_PTR.wrapping_add(24));
    wb(
        m,
        V_BTNPUSHTIME2,
        rom_b(m, demo.wrapping_add(1)).wrapping_sub(1),
    );
    ww(m, V_RINGS, 0);
    wb(m, 0xFF_FE1B, 0);
    ww(m, V_DEBUGUSE, 0);
    ww(m, V_GENERICTIMER, 1800);
    ww(m, VDP_CONTROLE, registro | 0x40);
    Ok(())
}

/// `PalCycle_SS`: atualiza os registradores de fundo e as cores da próxima imagem.
pub fn ciclo_paleta(m: &mut Memoria) {
    if super::rw(m, F_PAUSE) != 0 {
        return;
    }
    let tempo = super::rw(m, PAL_SS_TIME).wrapping_sub(1);
    ww(m, PAL_SS_TIME, tempo);
    if i16::from_be_bytes(tempo.to_be_bytes()) >= 0 {
        return;
    }
    let indice = super::rw(m, PAL_SS_NUM);
    ww(m, PAL_SS_NUM, indice.wrapping_add(1));
    let entrada = 0x4A48_u32.wrapping_add(u32::from(indice & 0x1F).wrapping_mul(4));
    let espera = rom_b(m, entrada);
    ww(
        m,
        PAL_SS_TIME,
        if espera & 0x80 != 0 {
            0x1FF
        } else {
            u16::from(espera)
        },
    );
    let animacao = rom_b(m, entrada.wrapping_add(1));
    ww(m, SS_BG_ANIM, u16::from(animacao));
    let modo = 0x4AC8_u32.wrapping_add(u32::from(animacao));
    ww(m, VDP_CONTROLE, 0x8200 | u16::from(rom_b(m, modo)));
    let y = rom_b(m, modo.wrapping_add(1));
    wb(m, SCRPOSY_VDP, y);
    let fundo = rom_b(m, entrada.wrapping_add(2));
    ww(m, VDP_CONTROLE, 0x8400 | u16::from(fundo));
    wl(m, VDP_CONTROLE, 0x4000_0010);
    let posicao = super::rl(m, SCRPOSY_VDP);
    wl(m, VDP_DADOS, posicao);
    let mut deslocamento = rom_b(m, entrada.wrapping_add(3));
    if deslocamento & 0x80 == 0 {
        let origem = 0x4AD6_u32.wrapping_add(u32::from(deslocamento));
        for i in 0..3u32 {
            let valor = rom_l(m, origem.wrapping_add(i.wrapping_mul(4)));
            wl(m, 0xFF_FB4E_u32.wrapping_add(i.wrapping_mul(4)), valor);
        }
        return;
    }
    let mut grupo = super::rw(m, PAL_SS_INDEX);
    if deslocamento >= 0x8A {
        grupo = grupo.wrapping_add(1);
    }
    let origem = 0x4B1E_u32.wrapping_add(u32::from(grupo).wrapping_mul(0x2A));
    if deslocamento & 1 != 0 {
        for i in 0..3u32 {
            let valor = rom_l(m, origem.wrapping_add(i.wrapping_mul(4)));
            wl(m, 0xFF_FB6E_u32.wrapping_add(i.wrapping_mul(4)), valor);
        }
    }
    deslocamento &= 0x7E;
    let destino = if deslocamento < 10 {
        0xFF_FB5A_u32
    } else {
        deslocamento = deslocamento.wrapping_sub(10);
        0xFF_FB7A_u32
    };
    let dados = origem
        .wrapping_add(12)
        .wrapping_add(u32::from(deslocamento).wrapping_mul(3));
    let primeira = rom_l(m, dados);
    let segunda = rom_w(m, dados.wrapping_add(4));
    wl(m, destino, primeira);
    ww(m, destino.wrapping_add(4), segunda);
}

/// `SS_BGAnimate`: movimenta as camadas e preenche a tabela de rolagem horizontal.
pub fn animar_fundo(m: &mut Memoria) {
    let modo = super::rw(m, SS_BG_ANIM);
    if modo == 0 {
        ww(m, V_BGSCREENPOSY, 0);
        ww(m, 0xFF_F618, 0);
    }
    let (buffer, tabela) = if modo < 8 {
        if modo == 6 {
            let x = super::rw(m, V_BG3SCREENPOSX).wrapping_add(1);
            ww(m, V_BG3SCREENPOSX, x);
            let y = super::rw(m, V_BGSCREENPOSY).wrapping_add(1);
            ww(m, V_BGSCREENPOSY, y);
            ww(m, 0xFF_F618, y);
        }
        for i in 0..10u32 {
            let slot = 0xFF_AA00_u32.wrapping_add(i.wrapping_mul(4));
            let angulo = u8::try_from(super::rw(m, slot.wrapping_add(2)) & 0xFF).unwrap_or(0);
            let (seno, _) = super::fisica::calc_sine(m, angulo);
            let forca = i32::from(rom_b(m, 0x4CD8_u32.wrapping_add(i.wrapping_mul(2))));
            let oscilacao = i32::from(seno).wrapping_mul(forca) >> 8;
            ww(
                m,
                slot,
                u16::try_from(u32::from_be_bytes(oscilacao.to_be_bytes()) & 0xFFFF).unwrap_or(0),
            );
            let delta = i16::from(i8::from_be_bytes([rom_b(
                m,
                0x4CD9_u32.wrapping_add(i.wrapping_mul(2)),
            )]));
            let fase = super::rw(m, slot.wrapping_add(2))
                .wrapping_add(u16::from_be_bytes(delta.to_be_bytes()));
            ww(m, slot.wrapping_add(2), fase);
        }
        (0xFF_AA00u32, 0x4CC4u32)
    } else {
        if modo == 12 {
            let x = super::rw(m, V_BG3SCREENPOSX).wrapping_sub(1);
            ww(m, V_BG3SCREENPOSX, x);
            let mut passo = 0x18000u32;
            for i in 0..7u32 {
                let endereco = 0xFF_AB00_u32.wrapping_add(i.wrapping_mul(4));
                let posicao = super::rl(m, endereco).wrapping_sub(passo);
                wl(m, endereco, posicao);
                passo = passo.wrapping_sub(0x2000);
            }
        }
        (0xFF_AB00u32, 0x4CD0u32)
    };
    let primeiro_plano = 0u16.wrapping_sub(super::rw(m, V_BG3SCREENPOSX));
    let y = super::rw(m, V_BGSCREENPOSY);
    let mut destino = u32::from(0u16.wrapping_sub(y) & 0xFF).wrapping_mul(4);
    let blocos = u32::from(rom_b(m, tabela)).wrapping_add(1);
    for i in 0..blocos {
        let deslocamento = super::rw(m, buffer.wrapping_add(i.wrapping_mul(4)));
        let valor = (u32::from(primeiro_plano) << 16) | u32::from(deslocamento);
        let linhas = u32::from(rom_b(m, tabela.wrapping_add(1).wrapping_add(i)));
        for _ in 0..linhas {
            wl(m, V_HSCROLLTABLEBUFFER.wrapping_add(destino), valor);
            destino = destino.wrapping_add(4) & 0x3FC;
        }
    }
}

/// Seleciona a próxima fase que não teve sua esmeralda coletada, como `SS_Load`.
pub fn selecionar(m: &mut Memoria) -> Result<u8, ErroAbertura> {
    let total = rb(m, V_EMERALDS);
    for _ in 0..6 {
        let etapa = rb(m, V_LASTSPECIAL);
        if etapa >= 6 {
            return Err(ErroAbertura::NaoImplementado(
                "indice invalido de Special Stage",
            ));
        }
        let proxima = etapa.wrapping_add(1);
        wb(m, V_LASTSPECIAL, if proxima >= 6 { 0 } else { proxima });
        if total == 6 {
            return Ok(etapa);
        }
        let mut encontrada = false;
        for i in 0..u32::from(total) {
            if rb(m, LISTA_ESMERALDAS.wrapping_add(i)) == etapa {
                encontrada = true;
                break;
            }
        }
        if !encontrada {
            return Ok(etapa);
        }
    }
    Err(ErroAbertura::NaoImplementado(
        "lista invalida de esmeraldas",
    ))
}

/// Carrega a posição, o layout Enigma e as definições de sprites a partir da ROM do jogador.
pub fn carregar(m: &mut Memoria) -> Result<u8, ErroAbertura> {
    let etapa = selecionar(m)?;
    let deslocamento = u32::from(etapa).wrapping_mul(4);
    let x = rom_w(m, POSICOES_INICIAIS.wrapping_add(deslocamento));
    let y = rom_w(
        m,
        POSICOES_INICIAIS.wrapping_add(deslocamento).wrapping_add(2),
    );
    ww(m, V_PLAYER.wrapping_add(OB_X), x);
    ww(m, V_PLAYER.wrapping_add(OB_Y), y);
    let layout = rom_l(m, INDICE_LAYOUT.wrapping_add(deslocamento));
    rotinas::eni_dec(m, layout, TEMPORARIO, 0).map_err(ErroAbertura::Rotina)?;
    for desloc in (0..0x4000u32).step_by(4) {
        wl(m, LAYOUT_BASE.wrapping_add(desloc), 0);
    }
    for linha in 0..64u32 {
        for coluna in 0..64u32 {
            let origem = TEMPORARIO
                .wrapping_add(linha.wrapping_mul(64))
                .wrapping_add(coluna);
            let destino = LAYOUT_REAL
                .wrapping_add(linha.wrapping_mul(128))
                .wrapping_add(coluna);
            let celula = rb(m, origem);
            wb(m, destino, celula);
        }
    }
    let n = (MAPAS_FIM.wrapping_sub(MAPAS)) / 6;
    for i in 0..n {
        let origem = MAPAS.wrapping_add(i.wrapping_mul(6));
        let destino = TEMPORARIO.wrapping_add(8).wrapping_add(i.wrapping_mul(8));
        let mapa = rom_l(m, origem);
        let frame = rom_b(m, origem);
        let tile = rom_w(m, origem.wrapping_add(4));
        wl(m, destino, mapa);
        ww(m, destino.wrapping_add(4), u16::from(frame));
        ww(m, destino.wrapping_add(6), tile);
    }
    for desloc in (0..0x100u32).step_by(4) {
        wl(m, ANIMACOES.wrapping_add(desloc), 0);
    }
    Ok(etapa)
}

/// Parte de `SonicSS_Move` que atualiza a inércia a partir do controle.
/// A aplicação da rotação e a verificação contra paredes devem ocorrer em seguida.
pub fn atualizar_inercia(m: &mut Memoria, a: u32) {
    let botoes = rb(m, V_JPADHOLD2);
    let mut inercia = i16::from_be_bytes(super::rw(m, a.wrapping_add(OB_INERTIA)).to_be_bytes());
    if botoes & 4 != 0 {
        let status = rb(m, a.wrapping_add(OB_STATUS)) | 1;
        wb(m, a.wrapping_add(OB_STATUS), status);
        if inercia <= 0 {
            inercia = inercia.wrapping_sub(12).max(-0x800);
        } else {
            inercia = inercia.wrapping_sub(0x40);
        }
    }
    if botoes & 8 != 0 {
        let status = rb(m, a.wrapping_add(OB_STATUS)) & !1;
        wb(m, a.wrapping_add(OB_STATUS), status);
        if inercia >= 0 {
            inercia = inercia.wrapping_add(12).min(0x800);
        } else {
            inercia = inercia.wrapping_add(0x40);
        }
    }
    if botoes & 12 == 0 {
        if inercia > 0 {
            inercia = inercia.saturating_sub(12).max(0);
        } else if inercia < 0 {
            inercia = inercia.saturating_add(12).min(0);
        }
    }
    ww(
        m,
        a.wrapping_add(OB_INERTIA),
        u16::from_be_bytes(inercia.to_be_bytes()),
    );
}

/// Centraliza a câmera em Sonic quando a subtração não ultrapassa o início do mapa.
pub fn fixar_camera(m: &mut Memoria, a: u32) {
    let x = super::rw(m, a.wrapping_add(OB_X));
    let y = super::rw(m, a.wrapping_add(OB_Y));
    if let Some(x) = x.checked_sub(160) {
        ww(m, V_SCREENPOSX, x);
    }
    if let Some(y) = y.checked_sub(112) {
        ww(m, V_SCREENPOSY, y);
    }
}

/// Testa os quatro blocos próximos de uma posição 16.16 e registra o último bloco sólido.
pub fn encontrar_parede(m: &mut Memoria, a: u32, x: u32, y: u32) -> bool {
    let pixel_x = u16::try_from(x >> 16).unwrap_or(0);
    let pixel_y = u16::try_from(y >> 16).unwrap_or(0);
    let linha = u32::from(pixel_y.wrapping_add(68) / 24).wrapping_mul(128);
    let coluna = u32::from(pixel_x.wrapping_add(20) / 24);
    let inicio = LAYOUT_BASE.wrapping_add(linha).wrapping_add(coluna);
    let mut solido = false;
    for deslocamento in [0u32, 1, 128, 129] {
        let endereco = inicio.wrapping_add(deslocamento);
        let bloco = rb(m, endereco);
        if bloco != 0 && bloco != 0x28 && !(0x3A..0x4B).contains(&bloco) {
            wb(m, a.wrapping_add(0x30), bloco);
            wl(m, a.wrapping_add(0x32), endereco.wrapping_add(1));
            solido = true;
        }
    }
    solido
}

/// Aplica a inércia ao movimento no referencial rotacionado do cenário.
pub fn mover_sonic(m: &mut Memoria, a: u32) {
    atualizar_inercia(m, a);
    let angulo = rb(m, 0xFF_F780).wrapping_add(0x20) & 0xC0;
    let (seno, cosseno) = super::fisica::calc_sine(m, angulo.wrapping_neg());
    let inercia = i32::from(i16::from_be_bytes(
        super::rw(m, a.wrapping_add(OB_INERTIA)).to_be_bytes(),
    ));
    let dx = i32::from(cosseno).wrapping_mul(inercia);
    let dy = i32::from(seno).wrapping_mul(inercia);
    let original_x = super::rl(m, a.wrapping_add(OB_X));
    let original_y = super::rl(m, a.wrapping_add(OB_Y));
    let x = original_x.wrapping_add(u32::from_be_bytes(dx.to_be_bytes()));
    let y = original_y.wrapping_add(u32::from_be_bytes(dy.to_be_bytes()));
    wl(m, a.wrapping_add(OB_X), x);
    wl(m, a.wrapping_add(OB_Y), y);
    if encontrar_parede(m, a, x, y) {
        wl(m, a.wrapping_add(OB_X), original_x);
        wl(m, a.wrapping_add(OB_Y), original_y);
        ww(m, a.wrapping_add(OB_INERTIA), 0);
    }
}

/// Aplica gravidade e verifica os eixos separadamente contra o layout.
pub fn gravidade(m: &mut Memoria, a: u32) {
    let x = super::rl(m, a.wrapping_add(OB_X));
    let y = super::rl(m, a.wrapping_add(OB_Y));
    let angulo = rb(m, 0xFF_F780) & 0xFC;
    let (seno, cosseno) = super::fisica::calc_sine(m, angulo);
    let vx = i32::from(i16::from_be_bytes(
        super::rw(m, a.wrapping_add(OB_VELX)).to_be_bytes(),
    ));
    let vy = i32::from(i16::from_be_bytes(
        super::rw(m, a.wrapping_add(OB_VELY)).to_be_bytes(),
    ));
    let mut dx = vx
        .wrapping_shl(8)
        .wrapping_add(i32::from(seno).wrapping_mul(0x2A));
    let mut dy = vy
        .wrapping_shl(8)
        .wrapping_add(i32::from(cosseno).wrapping_mul(0x2A));
    let x_alvo = x.wrapping_add(u32::from_be_bytes(dx.to_be_bytes()));
    let parede_x = encontrar_parede(m, a, x_alvo, y);
    let x_teste = if parede_x { x } else { x_alvo };
    if parede_x {
        dx = 0;
    }
    let y_alvo = y.wrapping_add(u32::from_be_bytes(dy.to_be_bytes()));
    let parede_y = encontrar_parede(m, a, x_teste, y_alvo);
    if parede_y {
        dy = 0;
    }
    let status = rb(m, a.wrapping_add(OB_STATUS));
    wb(
        m,
        a.wrapping_add(OB_STATUS),
        if parede_x || parede_y {
            status & !STATUS_IN_AIR
        } else {
            status | STATUS_IN_AIR
        },
    );
    let vx = u16::try_from(u32::from_be_bytes((dx >> 8).to_be_bytes()) & 0xFFFF).unwrap_or(0);
    let vy = u16::try_from(u32::from_be_bytes((dy >> 8).to_be_bytes()) & 0xFFFF).unwrap_or(0);
    ww(m, a.wrapping_add(OB_VELX), vx);
    ww(m, a.wrapping_add(OB_VELY), vy);
}

/// Executa a inicialização e o movimento de Sonic na Special Stage.
/// Os blocos de ação ainda devolvem erro explícito até terem suas rotinas próprias.
pub fn executar_sonic(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    if super::rw(m, V_DEBUGUSE) != 0 {
        return Err(ErroAbertura::NaoImplementado("debug da Special Stage"));
    }
    if rb(m, a.wrapping_add(OB_ROUTINE)) == 0 {
        wb(m, a.wrapping_add(OB_ROUTINE), 2);
        wb(m, a.wrapping_add(OB_HEIGHT), 14);
        wb(m, a.wrapping_add(OB_WIDTH), 7);
        wl(m, a.wrapping_add(OB_MAP), MAP_SONIC);
        ww(m, a.wrapping_add(OB_GFX), ARTTILE_SONIC);
        wb(m, a.wrapping_add(OB_RENDER), 4);
        wb(m, a.wrapping_add(OB_PRIORITY), 0);
        wb(m, a.wrapping_add(OB_ANIM), ANIM_ROLL);
        let status = rb(m, a.wrapping_add(OB_STATUS));
        wb(m, a.wrapping_add(OB_STATUS), status | STATUS_IN_AIR | 4);
    }
    if rb(m, a.wrapping_add(OB_ROUTINE)) == 4 {
        let velocidade = super::rw(m, 0xFF_F782).wrapping_add(0x40);
        ww(m, 0xFF_F782, velocidade);
        if velocidade == 0x1800 {
            wb(m, V_GAMEMODE, ID_LEVEL);
        }
        let angulo = super::rw(m, 0xFF_F780).wrapping_add(velocidade);
        ww(m, 0xFF_F780, angulo);
        super::fisica::sonic_animate(m, a)?;
        super::fisica::sonic_load_gfx(m, a);
        fixar_camera(m, a);
        super::objetos::display_sprite(m, a);
        return Ok(());
    }
    if rb(m, a.wrapping_add(OB_ROUTINE)) != 2 {
        return Err(ErroAbertura::NaoImplementado("saida da Special Stage"));
    }
    wb(m, a.wrapping_add(0x30), 0);
    if rb(m, a.wrapping_add(OB_STATUS)) & STATUS_IN_AIR == 0 {
        pular(m, a);
    }
    mover_sonic(m, a);
    gravidade(m, a);
    let x = super::rw(m, a.wrapping_add(OB_X));
    let y = super::rw(m, a.wrapping_add(OB_Y));
    let celula = LAYOUT_BASE
        .wrapping_add(u32::from(y.wrapping_add(80) / 24).wrapping_mul(128))
        .wrapping_add(u32::from(x.wrapping_add(32) / 24));
    let item = rb(m, celula);
    let contato = rb(m, a.wrapping_add(0x30));
    match item {
        0x3A => {
            agendar_animacao(m, 1, celula);
            super::objetos::collect_ring(m);
            let flags = rb(m, 0xFF_FE1B);
            if super::rw(m, V_RINGS) >= 50 && flags & 1 == 0 {
                wb(m, 0xFF_FE1B, flags | 1);
                let creditos = rb(m, V_CONTINUES).wrapping_add(1);
                wb(m, V_CONTINUES, creditos);
                wb(m, V_SOUNDQUEUE0, 0xBF);
            }
        }
        0x28 => {
            agendar_animacao(m, 3, celula);
            let vidas = rb(m, V_LIVES).wrapping_add(1);
            wb(m, V_LIVES, vidas);
            let atualizacao = rb(m, F_LIFECOUNT).wrapping_add(1);
            wb(m, F_LIFECOUNT, atualizacao);
            wb(m, V_SOUNDQUEUE0, 0x88);
        }
        0x3B..=0x40 => {
            agendar_animacao(m, 5, celula);
            let total = rb(m, V_EMERALDS);
            if total != 6 {
                wb(
                    m,
                    LISTA_ESMERALDAS.wrapping_add(u32::from(total)),
                    item.wrapping_sub(0x3B),
                );
                wb(m, V_EMERALDS, total.wrapping_add(1));
            }
            wb(m, V_SOUNDQUEUE1, 0x93);
        }
        0x41 => wb(m, a.wrapping_add(0x3A), 1),
        0x4A if rb(m, a.wrapping_add(0x3A)) == 1 => wb(m, a.wrapping_add(0x3A), 2),
        0 if rb(m, a.wrapping_add(0x3A)) != 0 => {
            if rb(m, a.wrapping_add(0x3A)) == 2 {
                for linha in 0..64u32 {
                    for coluna in 0..64u32 {
                        let endereco = LAYOUT_REAL
                            .wrapping_add(linha.wrapping_mul(128))
                            .wrapping_add(coluna);
                        if rb(m, endereco) == 0x41 {
                            wb(m, endereco, 0x2C);
                        }
                    }
                }
            }
            wb(m, a.wrapping_add(0x3A), 0);
        }
        _ => {}
    }
    let bloco = super::rl(m, a.wrapping_add(0x32)).wrapping_sub(1);
    match contato {
        0 => {
            for campo in [0x36u32, 0x37] {
                let tempo = rb(m, a.wrapping_add(campo)).wrapping_sub(1);
                wb(
                    m,
                    a.wrapping_add(campo),
                    if tempo & 0x80 != 0 { 0 } else { tempo },
                );
            }
        }
        0x27 => {
            wb(m, a.wrapping_add(OB_ROUTINE), 4);
            wb(m, V_SOUNDQUEUE1, 0xA8);
        }
        0x29 | 0x2A if rb(m, a.wrapping_add(0x36)) == 0 => {
            wb(m, a.wrapping_add(0x36), 30);
            let velocidade = super::rw(m, 0xFF_F782);
            if contato == 0x29 && velocidade & 0x40 != 0 {
                ww(m, 0xFF_F782, velocidade.wrapping_shl(1));
                wb(m, bloco, 0x2A);
            } else if contato == 0x2A && velocidade & 0x40 == 0 {
                let reduzido = i16::from_be_bytes(velocidade.to_be_bytes()) >> 1;
                ww(m, 0xFF_F782, u16::from_be_bytes(reduzido.to_be_bytes()));
                wb(m, bloco, 0x29);
            }
            wb(m, V_SOUNDQUEUE1, 0xA9);
        }
        0x2B if rb(m, a.wrapping_add(0x37)) == 0 => {
            wb(m, a.wrapping_add(0x37), 30);
            agendar_animacao(m, 4, bloco);
            let velocidade = super::rw(m, 0xFF_F782);
            ww(m, 0xFF_F782, 0u16.wrapping_sub(velocidade));
            wb(m, V_SOUNDQUEUE1, 0xA9);
        }
        0x2D..=0x30 => {
            if let Some(slot) = agendar_animacao(m, 6, bloco) {
                wb(
                    m,
                    slot.wrapping_add(4),
                    if contato == 0x30 {
                        0
                    } else {
                        contato.wrapping_add(1)
                    },
                );
            }
            wb(m, V_SOUNDQUEUE1, 0xBA);
        }
        0x25 => {
            let indice = bloco.wrapping_sub(LAYOUT_BASE);
            let coluna = u16::try_from(indice & 0x7F).unwrap_or(0);
            let linha = u16::try_from((indice >> 7) & 0x7F).unwrap_or(0);
            let dx = coluna.wrapping_mul(24).wrapping_sub(20).wrapping_sub(x);
            let dy = linha.wrapping_mul(24).wrapping_sub(68).wrapping_sub(y);
            let angulo = super::fisica::calc_angle(m, dx, dy);
            let (seno, cosseno) = super::fisica::calc_sine(m, angulo);
            let vx = i32::from(cosseno).wrapping_mul(-0x700) >> 8;
            let vy = i32::from(seno).wrapping_mul(-0x700) >> 8;
            ww(
                m,
                a.wrapping_add(OB_VELX),
                u16::try_from(u32::from_be_bytes(vx.to_be_bytes()) & 0xFFFF).unwrap_or(0),
            );
            ww(
                m,
                a.wrapping_add(OB_VELY),
                u16::try_from(u32::from_be_bytes(vy.to_be_bytes()) & 0xFFFF).unwrap_or(0),
            );
            let status = rb(m, a.wrapping_add(OB_STATUS));
            wb(m, a.wrapping_add(OB_STATUS), status | STATUS_IN_AIR);
            agendar_animacao(m, 2, bloco);
            wb(m, V_SOUNDQUEUE1, 0xB4);
        }
        _ => {}
    }
    super::fisica::speed_to_pos(m, a);
    fixar_camera(m, a);
    let angulo = super::rw(m, 0xFF_F780);
    let velocidade = super::rw(m, 0xFF_F782);
    ww(m, 0xFF_F780, angulo.wrapping_add(velocidade));
    super::fisica::sonic_animate(m, a)?;
    super::fisica::sonic_load_gfx(m, a);
    super::objetos::display_sprite(m, a);
    Ok(())
}

/// Um quadro após o VBlank especial; o fundo animado ainda depende de implementação.
pub fn passo(m: &mut Memoria) -> Result<(), ErroAbertura> {
    if super::rw(m, F_DEMO) != 0 {
        return Err(ErroAbertura::NaoImplementado("demo da Special Stage"));
    }
    let botoes = super::rw(m, V_JPADHOLD1);
    ww(m, V_JPADHOLD2, botoes);
    executar_sonic(m, V_PLAYER)?;
    super::objetos::build_sprites(m)?;
    animar_itens(m)?;
    desenhar_layout(m);
    animar_fundo(m);
    Ok(())
}

/// Inicia o salto perpendicular ao cenário quando um botão de pulo é pressionado.
pub fn pular(m: &mut Memoria, a: u32) {
    if rb(m, V_JPADPRESS2) & 0x70 == 0 {
        return;
    }
    let angulo = (rb(m, 0xFF_F780) & 0xFC).wrapping_neg().wrapping_sub(0x40);
    let (seno, cosseno) = super::fisica::calc_sine(m, angulo);
    let vx = i32::from(cosseno).wrapping_mul(0x680) >> 8;
    let vy = i32::from(seno).wrapping_mul(0x680) >> 8;
    let vx = u16::try_from(u32::from_be_bytes(vx.to_be_bytes()) & 0xFFFF).unwrap_or(0);
    let vy = u16::try_from(u32::from_be_bytes(vy.to_be_bytes()) & 0xFFFF).unwrap_or(0);
    ww(m, a.wrapping_add(OB_VELX), vx);
    ww(m, a.wrapping_add(OB_VELY), vy);
    let status = rb(m, a.wrapping_add(OB_STATUS)) | STATUS_IN_AIR;
    wb(m, a.wrapping_add(OB_STATUS), status);
    wb(m, V_SOUNDQUEUE1, 0xA0);
}

/// Calcula a matriz 16 por 16 de posições rotacionadas usada por `SS_ShowLayout`.
pub fn calcular_matriz(m: &mut Memoria) {
    let angulo = rb(m, 0xFF_F780) & 0xFC;
    let (seno, cosseno) = super::fisica::calc_sine(m, angulo);
    let x = i32::from(super::rw(m, V_SCREENPOSX) % 24)
        .wrapping_neg()
        .wrapping_sub(180);
    let mut y = i32::from(super::rw(m, V_SCREENPOSY) % 24)
        .wrapping_neg()
        .wrapping_sub(180);
    let seno = i32::from(seno);
    let cosseno = i32::from(cosseno);
    let passo_x = cosseno.wrapping_mul(24);
    let passo_y = seno.wrapping_mul(24);
    let mut destino = 0xFF_8000u32;
    for _ in 0..16 {
        let mut dx = x.wrapping_mul(cosseno).wrapping_sub(y.wrapping_mul(seno));
        let mut dy = x.wrapping_mul(seno).wrapping_add(y.wrapping_mul(cosseno));
        for _ in 0..16 {
            let px =
                u16::try_from(u32::from_be_bytes((dx >> 8).to_be_bytes()) & 0xFFFF).unwrap_or(0);
            let py =
                u16::try_from(u32::from_be_bytes((dy >> 8).to_be_bytes()) & 0xFFFF).unwrap_or(0);
            ww(m, destino, px);
            ww(m, destino.wrapping_add(2), py);
            destino = destino.wrapping_add(4);
            dx = dx.wrapping_add(passo_x);
            dy = dy.wrapping_add(passo_y);
        }
        y = y.wrapping_add(24);
    }
}

/// Acrescenta os blocos visíveis à tabela de sprites já montada para os objetos.
pub fn desenhar_layout(m: &mut Memoria) {
    animar_blocos(m);
    calcular_matriz(m);
    let camera_x = u32::from(super::rw(m, V_SCREENPOSX) / 24);
    let camera_y = u32::from(super::rw(m, V_SCREENPOSY) / 24);
    let base = LAYOUT_BASE
        .wrapping_add(camera_y.wrapping_mul(128))
        .wrapping_add(camera_x);
    let mut quantidade = rb(m, V_SPRITECOUNT);
    let mut destino = V_SPRITETABLEBUFFER.wrapping_add(u32::from(quantidade).wrapping_mul(8));
    for linha in 0..16u32 {
        for coluna in 0..16u32 {
            let celula = base
                .wrapping_add(linha.wrapping_mul(128))
                .wrapping_add(coluna);
            let id = rb(m, celula);
            if id == 0 || id > 0x4E {
                continue;
            }
            let matriz = 0xFF_8000u32
                .wrapping_add(linha.wrapping_mul(64))
                .wrapping_add(coluna.wrapping_mul(4));
            let x = super::rw(m, matriz).wrapping_add(288);
            let y = super::rw(m, matriz.wrapping_add(2)).wrapping_add(240);
            if !(112..464).contains(&x) || !(112..368).contains(&y) {
                continue;
            }
            let definicao = TEMPORARIO.wrapping_add(u32::from(id).wrapping_mul(8));
            let mapa = super::rl(m, definicao) & 0xFF_FFFF;
            let frame = super::rw(m, definicao.wrapping_add(4));
            let tile = super::rw(m, definicao.wrapping_add(6));
            let offset = rom_w(m, mapa.wrapping_add(u32::from(frame).wrapping_mul(2)));
            let deslocamento = i32::from(i16::from_be_bytes(offset.to_be_bytes()));
            let mut peca = mapa.wrapping_add(u32::from_be_bytes(deslocamento.to_be_bytes()));
            let n = rom_b(m, peca);
            peca = peca.wrapping_add(1);
            for _ in 0..n {
                if quantidade == 80 {
                    break;
                }
                let py = i16::from(i8::from_be_bytes([rom_b(m, peca)]));
                let px = i16::from(i8::from_be_bytes([rom_b(m, peca.wrapping_add(4))]));
                ww(
                    m,
                    destino,
                    y.wrapping_add(u16::from_be_bytes(py.to_be_bytes())),
                );
                let tamanho = rom_b(m, peca.wrapping_add(1));
                wb(m, destino.wrapping_add(2), tamanho);
                quantidade = quantidade.wrapping_add(1);
                wb(m, destino.wrapping_add(3), quantidade);
                let padrao = rom_w(m, peca.wrapping_add(2)).wrapping_add(tile);
                ww(m, destino.wrapping_add(4), padrao);
                let px = x.wrapping_add(u16::from_be_bytes(px.to_be_bytes())) & 0x1FF;
                ww(m, destino.wrapping_add(6), if px == 0 { 1 } else { px });
                destino = destino.wrapping_add(8);
                peca = peca.wrapping_add(5);
            }
        }
    }
    wb(m, V_SPRITECOUNT, quantidade);
    if quantidade == 80 {
        wb(m, destino.wrapping_sub(5), 0);
    } else {
        wl(m, destino, 0);
    }
}

fn animar_blocos(m: &mut Memoria) {
    let rotacao = u16::from((rb(m, 0xFF_F780) >> 2) & 15);
    for bloco in 1..=0x24u32 {
        ww(
            m,
            TEMPORARIO
                .wrapping_add(bloco.wrapping_mul(8))
                .wrapping_add(4),
            rotacao,
        );
    }
    for (tempo, quadro, recarga, mascara, reverso) in [
        (V_ANI1_TIME, V_ANI1_FRAME, 7, 3, false),
        (V_ANI2_TIME, V_ANI2_FRAME, 7, 1, false),
        (V_ANI3_TIME, V_ANI3_FRAME, 4, 3, false),
        (V_ANI0_TIME, V_ANI0_FRAME, 7, 7, true),
    ] {
        let t = rb(m, tempo).wrapping_sub(1);
        wb(m, tempo, t);
        if i8::from_be_bytes([t]) < 0 {
            wb(m, tempo, recarga);
            let frame = rb(m, quadro);
            let frame = if reverso {
                frame.wrapping_sub(1)
            } else {
                frame.wrapping_add(1)
            };
            wb(m, quadro, frame & mascara);
        }
    }
    let ring = rb(m, V_ANI1_FRAME);
    wb(m, TEMPORARIO.wrapping_add(0x3A * 8 + 5), ring);
    let alternado = rb(m, V_ANI2_FRAME);
    for id in [
        0x27u32, 0x2C, 0x29, 0x2A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x40,
    ] {
        wb(
            m,
            TEMPORARIO.wrapping_add(id.wrapping_mul(8)).wrapping_add(5),
            alternado,
        );
    }
    let vidro = rb(m, V_ANI3_FRAME);
    for id in 0x2D..=0x30u32 {
        wb(
            m,
            TEMPORARIO.wrapping_add(id.wrapping_mul(8)).wrapping_add(5),
            vidro,
        );
    }
    let ciclo = u32::from(rb(m, V_ANI0_FRAME)).wrapping_mul(2);
    for grupo in 0..4u32 {
        for i in 0..8u32 {
            let origem = 0x1BBF0u32
                .wrapping_add(ciclo)
                .wrapping_add(grupo.wrapping_mul(32))
                .wrapping_add(i.wrapping_mul(2));
            let tile = rom_w(m, origem);
            let id = 2u32.wrapping_add(grupo.wrapping_mul(9)).wrapping_add(i);
            ww(
                m,
                TEMPORARIO.wrapping_add(id.wrapping_mul(8)).wrapping_add(6),
                tile,
            );
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn fundo_copia_faixas_de_rolagem_a_partir_da_rom() {
        let mut rom = vec![0; 0x10000];
        rom[0x4CD0] = 0;
        rom[0x4CD1] = 2;
        let mut m = Memoria::nova(rom);
        ww(&mut m, SS_BG_ANIM, 8);
        ww(&mut m, V_BG3SCREENPOSX, 16);
        ww(&mut m, 0xFF_AB00, 0x1234);
        animar_fundo(&mut m);
        assert_eq!(super::super::rl(&mut m, V_HSCROLLTABLEBUFFER), 0xFFF0_1234);
        assert_eq!(
            super::super::rl(&mut m, V_HSCROLLTABLEBUFFER.wrapping_add(4)),
            0xFFF0_1234
        );
    }

    #[test]
    fn fila_de_animacao_le_script_e_libera_slot() {
        let mut rom = vec![0; 0x20000];
        rom[0x1BCE8] = 0x44;
        rom[0x1BCE9] = 0;
        let mut m = Memoria::nova(rom);
        let celula = LAYOUT_REAL;
        let slot = agendar_animacao(&mut m, 1, celula).unwrap();
        animar_itens(&mut m).unwrap();
        assert_eq!(rb(&mut m, celula), 0x44);
        for _ in 0..6 {
            animar_itens(&mut m).unwrap();
        }
        assert_eq!(rb(&mut m, celula), 0);
        assert_eq!(rb(&mut m, slot), 0);
    }

    #[test]
    fn saida_acelera_rotacao_ate_retornar_ao_nivel() {
        let mut m = Memoria::nova(vec![0; 0x40000]);
        let a = V_PLAYER;
        wb(&mut m, a.wrapping_add(OB_ROUTINE), 4);
        ww(&mut m, 0xFF_F782, 0x17C0);
        executar_sonic(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, V_GAMEMODE), ID_LEVEL);
        assert_eq!(super::super::rw(&mut m, 0xFF_F782), 0x1800);
    }

    #[test]
    fn ciclo_de_paleta_respeita_pausa_e_le_parametros_da_rom() {
        let mut rom = vec![0; 0x10000];
        rom[0x4A48..0x4A4C].copy_from_slice(&[5, 2, 3, 4]);
        rom[0x4ACA..0x4ACC].copy_from_slice(&[0x28, 7]);
        rom[0x4ADA..0x4ADE].copy_from_slice(&0x1234_5678u32.to_be_bytes());
        let mut m = Memoria::nova(rom);
        ww(&mut m, F_PAUSE, 1);
        ciclo_paleta(&mut m);
        assert_eq!(super::super::rw(&mut m, PAL_SS_NUM), 0);
        ww(&mut m, F_PAUSE, 0);
        ciclo_paleta(&mut m);
        assert_eq!(super::super::rw(&mut m, PAL_SS_NUM), 1);
        assert_eq!(super::super::rw(&mut m, PAL_SS_TIME), 5);
        assert_eq!(super::super::rw(&mut m, SS_BG_ANIM), 2);
        assert_eq!(rb(&mut m, SCRPOSY_VDP), 7);
        assert_eq!(super::super::rl(&mut m, 0xFF_FB4E), 0x1234_5678);
        ciclo_paleta(&mut m);
        assert_eq!(super::super::rw(&mut m, PAL_SS_TIME), 4);
    }

    #[test]
    fn inercia_e_camera_seguem_os_limites_do_estagio() {
        let mut m = Memoria::nova(vec![]);
        wb(&mut m, V_JPADHOLD2, 8);
        atualizar_inercia(&mut m, V_PLAYER);
        assert_eq!(
            super::super::rw(&mut m, V_PLAYER.wrapping_add(OB_INERTIA)),
            12
        );
        wb(&mut m, V_JPADHOLD2, 4);
        atualizar_inercia(&mut m, V_PLAYER);
        assert_eq!(
            super::super::rw(&mut m, V_PLAYER.wrapping_add(OB_INERTIA)),
            0xFFCC
        );
        ww(&mut m, V_PLAYER.wrapping_add(OB_X), 200);
        ww(&mut m, V_PLAYER.wrapping_add(OB_Y), 150);
        fixar_camera(&mut m, V_PLAYER);
        assert_eq!(super::super::rw(&mut m, V_SCREENPOSX), 40);
        assert_eq!(super::super::rw(&mut m, V_SCREENPOSY), 38);
    }

    #[test]
    fn parede_considera_os_quatro_blocos_e_ignora_itens() {
        let mut m = Memoria::nova(vec![]);
        let inicio = LAYOUT_BASE.wrapping_add(2u32.wrapping_mul(128));
        wb(&mut m, inicio, 0x3A);
        wb(&mut m, inicio.wrapping_add(1), 0x28);
        assert!(!encontrar_parede(&mut m, V_PLAYER, 0, 0));
        wb(&mut m, inicio.wrapping_add(129), 0x4B);
        assert!(encontrar_parede(&mut m, V_PLAYER, 0, 0));
        assert_eq!(rb(&mut m, V_PLAYER.wrapping_add(0x30)), 0x4B);
        assert_eq!(
            super::super::rl(&mut m, V_PLAYER.wrapping_add(0x32)),
            inicio.wrapping_add(130)
        );
    }

    #[test]
    fn gravidade_aplica_forca_vertical_sem_parede() {
        let mut rom = vec![0; 0x10000];
        let cosseno = usize::try_from(SINE_DATA.wrapping_add(0x80)).unwrap();
        rom[cosseno..cosseno.wrapping_add(2)].copy_from_slice(&256i16.to_be_bytes());
        let mut m = Memoria::nova(rom);
        gravidade(&mut m, V_PLAYER);
        assert_eq!(super::super::rw(&mut m, V_PLAYER.wrapping_add(OB_VELX)), 0);
        assert_eq!(
            super::super::rw(&mut m, V_PLAYER.wrapping_add(OB_VELY)),
            0x2A
        );
        assert_ne!(
            rb(&mut m, V_PLAYER.wrapping_add(OB_STATUS)) & STATUS_IN_AIR,
            0
        );
    }

    #[test]
    fn matriz_sem_rotacao_preserva_distancia_entre_celulas() {
        let mut rom = vec![0; 0x10000];
        let cosseno = usize::try_from(SINE_DATA.wrapping_add(0x80)).unwrap();
        rom[cosseno..cosseno.wrapping_add(2)].copy_from_slice(&256i16.to_be_bytes());
        let mut m = Memoria::nova(rom);
        calcular_matriz(&mut m);
        assert_eq!(
            super::super::rw(&mut m, 0xFF_8000),
            u16::from_be_bytes((-180i16).to_be_bytes())
        );
        assert_eq!(
            super::super::rw(&mut m, 0xFF_8004),
            u16::from_be_bytes((-156i16).to_be_bytes())
        );
        assert_eq!(
            super::super::rw(&mut m, 0xFF_8000u32.wrapping_add(64).wrapping_add(2)),
            u16::from_be_bytes((-156i16).to_be_bytes())
        );
    }

    #[test]
    fn desenho_converte_celula_visivel_em_sprite() {
        let mut rom = vec![0; 0x10000];
        let cosseno = usize::try_from(SINE_DATA.wrapping_add(0x80)).unwrap();
        rom[cosseno..cosseno.wrapping_add(2)].copy_from_slice(&256i16.to_be_bytes());
        rom[0x9000..0x9008].copy_from_slice(&[0, 2, 1, 0xFC, 0, 0, 1, 0xFC]);
        let mut m = Memoria::nova(rom);
        wb(&mut m, V_ANI1_TIME, 1);
        let definicao = TEMPORARIO.wrapping_add(0x3A * 8);
        wl(&mut m, definicao, 0x9000);
        ww(&mut m, definicao.wrapping_add(6), 0x100);
        wb(
            &mut m,
            LAYOUT_BASE.wrapping_add(7 * 128).wrapping_add(7),
            0x3A,
        );
        desenhar_layout(&mut m);
        assert_eq!(rb(&mut m, V_SPRITECOUNT), 1);
        assert_eq!(super::super::rw(&mut m, V_SPRITETABLEBUFFER), 224);
        assert_eq!(
            super::super::rw(&mut m, V_SPRITETABLEBUFFER.wrapping_add(4)),
            0x101
        );
        assert_eq!(
            super::super::rw(&mut m, V_SPRITETABLEBUFFER.wrapping_add(6)),
            272
        );
    }

    #[test]
    #[ignore = "requer SONIC1_ROM apontando para REV01"]
    fn carrega_as_seis_fases_da_rom() {
        let caminho = std::env::var_os("SONIC1_ROM").expect("SONIC1_ROM");
        let rom = std::fs::read(caminho).expect("ler ROM");
        for etapa in 0..6u8 {
            let mut m = Memoria::nova(rom.clone());
            wb(&mut m, V_LASTSPECIAL, etapa);
            assert_eq!(carregar(&mut m).unwrap(), etapa);
            let mut ocupadas = 0u32;
            for i in 0..64u32 {
                for j in 0..64u32 {
                    if rb(
                        &mut m,
                        LAYOUT_REAL
                            .wrapping_add(i.wrapping_mul(128))
                            .wrapping_add(j),
                    ) != 0
                    {
                        ocupadas = ocupadas.wrapping_add(1);
                    }
                }
            }
            assert!(ocupadas > 0);
            assert_eq!(rb(&mut m, LAYOUT_BASE), 0);
        }
    }

    #[test]
    #[ignore = "requer SONIC1_ROM apontando para REV01"]
    fn prepara_arte_e_estado_com_rom_real() {
        let caminho = std::env::var_os("SONIC1_ROM").expect("SONIC1_ROM");
        let mut m = Memoria::nova(std::fs::read(caminho).expect("ler ROM"));
        preparar(&mut m).unwrap();
        assert_eq!(rb(&mut m, V_PLAYER.wrapping_add(OB_ID)), 9);
        assert_eq!(super::super::rw(&mut m, 0xFF_F782), 0x40);
        assert_eq!(rb(&mut m, V_SOUNDQUEUE0), 0x89);
        assert_eq!(super::super::rw(&mut m, V_GENERICTIMER), 1800);
        passo(&mut m).unwrap();
        assert_eq!(rb(&mut m, V_PLAYER.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(rb(&mut m, V_PLAYER.wrapping_add(OB_ANIM)), ANIM_ROLL);
    }

    #[test]
    fn selecao_pula_estagios_concluidos_e_reinicia_ciclo() {
        let mut m = Memoria::nova(vec![]);
        wb(&mut m, V_LASTSPECIAL, 4);
        wb(&mut m, V_EMERALDS, 2);
        wb(&mut m, LISTA_ESMERALDAS, 4);
        wb(&mut m, LISTA_ESMERALDAS.wrapping_add(1), 5);
        assert_eq!(selecionar(&mut m).unwrap(), 0);
        assert_eq!(rb(&mut m, V_LASTSPECIAL), 1);
    }

    #[test]
    fn selecao_com_todas_esmeraldas_mantem_fallback_original() {
        let mut m = Memoria::nova(vec![]);
        wb(&mut m, V_LASTSPECIAL, 5);
        wb(&mut m, V_EMERALDS, 6);
        assert_eq!(selecionar(&mut m).unwrap(), 5);
        assert_eq!(rb(&mut m, V_LASTSPECIAL), 0);
    }
}
