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
