//! Objeto de anéis espalhados (`Ring_Loss`, objeto `0x37`).
//!
//! Quando Sonic sofre dano tendo anéis, até 32 anéis são espalhados ao redor dele.
//! Os anéis recebem velocidades angulares em arco, quicam no solo e somem após 256 quadros.
//! Podem ser recolhidos pelo Sonic após o tempo inicial de imunidade piscar.

use super::fisica::{calc_sine, obj_floor_dist, speed_to_pos};
use super::objetos::{collect_ring, delete_object, display_sprite, find_free_obj, ring_sparkle};
use super::ram::{
    ARTTILE_RING, ID_RING_LOSS, MAP_RING, OB_ACTWID, OB_COLTYPE, OB_FRAME, OB_GFX, OB_HEIGHT,
    OB_ID, OB_MAP, OB_PRIORITY, OB_RENDER, OB_ROUTINE, OB_VELX, OB_VELY, OB_WIDTH, OB_X, OB_Y,
    V_ANI3_FRAME, V_ANI3_TIME, V_LIMITBTM2, V_PLAYER, V_RINGS,
};
use super::{ErroAbertura, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;

/// Colisão tipo item 12x12 (`0x47`).
const COL_ANEL: u8 = 0x47;

/// Rotinas do objeto RingLoss.
const ROTINA_QUICAR: u8 = 2;
const ROTINA_COLETAR: u8 = 4;
const ROTINA_BRILHO: u8 = 6;
const ROTINA_APAGAR: u8 = 8;

fn s16(w: u16) -> i16 {
    i16::from_be_bytes(w.to_be_bytes())
}

fn u16_de(i: i16) -> u16 {
    u16::from_be_bytes(i.to_be_bytes())
}

/// Espalha os anéis do Sonic quando ele sofre dano.
pub fn soltar_aneis(m: &mut Memoria) {
    let total_aneis = rw(m, V_RINGS);
    if total_aneis == 0 {
        return;
    }

    let qtd = total_aneis.min(32);
    // Reinicia o temporizador global de anéis espalhados para 255 quadros.
    wb(m, V_ANI3_TIME, 255);

    let sonic_x = rw(m, V_PLAYER.wrapping_add(OB_X));
    let sonic_y = rw(m, V_PLAYER.wrapping_add(OB_Y));

    // Valor inicial do ângulo e boost de velocidade ($288 no original).
    // Byte alto (2) = deslocamento para esquerda da velocidade.
    // Byte baixo ($88) = ângulo angular inicial.
    let mut espalhamento: i16 = 0x288;
    let mut vel_x_anterior: i16 = 0;
    let mut vel_y_anterior: i16 = 0;

    for _ in 0..qtd {
        let Some(a1) = find_free_obj(m) else {
            break;
        };

        // Inicializa o objeto de anel quicando.
        wb(m, a1.wrapping_add(OB_ID), ID_RING_LOSS);
        wb(m, a1.wrapping_add(OB_ROUTINE), ROTINA_QUICAR);
        wb(m, a1.wrapping_add(OB_HEIGHT), 8);
        wb(m, a1.wrapping_add(OB_WIDTH), 8);
        ww(m, a1.wrapping_add(OB_X), sonic_x);
        ww(m, a1.wrapping_add(OB_Y), sonic_y);
        wl(m, a1.wrapping_add(OB_MAP), MAP_RING);
        ww(m, a1.wrapping_add(OB_GFX), ARTTILE_RING | 0x2000);
        wb(m, a1.wrapping_add(OB_RENDER), 4);
        wb(m, a1.wrapping_add(OB_PRIORITY), 3);
        wb(m, a1.wrapping_add(OB_COLTYPE), COL_ANEL);
        wb(m, a1.wrapping_add(OB_ACTWID), 8);

        let (vx, vy) = if espalhamento < 0 {
            // Repete as velocidades anteriores invertendo X.
            let vx = vel_x_anterior.wrapping_neg();
            let vy = vel_y_anterior;
            espalhamento = espalhamento.wrapping_neg();
            (vx, vy)
        } else {
            let angulo = (espalhamento & 0xFF).to_be_bytes()[1];
            let (seno, cosseno) = calc_sine(m, angulo);
            let boost = (u16_de(espalhamento) >> 8) & 0xFF;

            let vx = s16(u16_de(seno).wrapping_shl(u32::from(boost)));
            let vy = s16(u16_de(cosseno).wrapping_shl(u32::from(boost)));
            vel_x_anterior = vx;
            vel_y_anterior = vy;

            // Avança o ângulo para o próximo par de anéis.
            let baixo = (espalhamento & 0xFF).to_be_bytes()[1];
            let (novo_baixo, estourou) = baixo.overflowing_add(0x10);
            if estourou {
                // Estouro de byte: recua ângulo e reduz o boost.
                espalhamento = (espalhamento & !0xFF)
                    .wrapping_add(i16::from(novo_baixo))
                    .wrapping_sub(0x80);
            } else {
                espalhamento = (espalhamento & !0xFF) | i16::from(novo_baixo);
            }

            // Inverte para a próxima iteração espelhar o anel par.
            espalhamento = espalhamento.wrapping_neg();
            (vx, vy)
        };

        ww(m, a1.wrapping_add(OB_VELX), u16_de(vx));
        ww(m, a1.wrapping_add(OB_VELY), u16_de(vy));
    }
}

/// Executa a lógica de um anel quicando.
pub fn executar(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    match rb(m, a0.wrapping_add(OB_ROUTINE)) {
        ROTINA_QUICAR => quicar(m, a0),
        ROTINA_COLETAR => {
            wb(m, a0.wrapping_add(OB_ROUTINE), ROTINA_BRILHO);
            wb(m, a0.wrapping_add(OB_COLTYPE), 0);
            wb(m, a0.wrapping_add(OB_PRIORITY), 1);
            collect_ring(m);
            ring_sparkle(m, a0)
        }
        ROTINA_BRILHO => ring_sparkle(m, a0),
        ROTINA_APAGAR => {
            delete_object(m, a0);
            Ok(())
        }
        _ => {
            delete_object(m, a0);
            Ok(())
        }
    }
}

fn quicar(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    // Sincroniza o quadro de animação com o temporizador global.
    let quadro = rb(m, V_ANI3_FRAME);
    wb(m, a0.wrapping_add(OB_FRAME), quadro);

    speed_to_pos(m, a0);

    // Gravidade: adiciona 0x18 à velocidade vertical.
    let vy = s16(rw(m, a0.wrapping_add(OB_VELY))).wrapping_add(0x18);
    ww(m, a0.wrapping_add(OB_VELY), u16_de(vy));

    // Se estiver caindo (vy >= 0), verifica colisão com o solo.
    if vy >= 0 {
        let dist = rw(m, a0.wrapping_add(OB_Y));
        let dist_piso = s16(obj_floor_dist(m, a0));
        if dist_piso < 0 {
            // Ajusta posição no piso e quica com 75% da velocidade.
            let novo_y = dist.wrapping_add(u16_de(dist_piso));
            ww(m, a0.wrapping_add(OB_Y), novo_y);
            let amortecido = vy.wrapping_sub(vy >> 2);
            let novo_vy = amortecido.wrapping_neg();
            ww(m, a0.wrapping_add(OB_VELY), u16_de(novo_vy));
        }
    }

    // Se o temporizador global expirou, apaga o anel.
    let tempo = rb(m, V_ANI3_TIME);
    if tempo == 0 {
        delete_object(m, a0);
        return Ok(());
    }

    // Se caiu abaixo do limite inferior da fase, apaga o anel.
    let limite_inferior = rw(m, V_LIMITBTM2).wrapping_add(224);
    let y = rw(m, a0.wrapping_add(OB_Y));
    if y > limite_inferior {
        delete_object(m, a0);
        return Ok(());
    }

    // Pisca nos últimos 64 quadros alternando a visibilidade.
    if tempo >= 64 || (tempo & 4) != 0 {
        display_sprite(m, a0);
    }

    Ok(())
}

#[cfg(test)]
mod testes {
    use super::super::ram::{OBJECT_SIZE, SINE_DATA, V_LVLOBJEND, V_LVLOBJSPACE};
    use super::*;

    fn criar_memoria_com_seno() -> Memoria {
        let mut rom = vec![0; 0x10000];
        let base = usize::try_from(SINE_DATA).unwrap();
        for i in 0..256 {
            let offset = base + i * 2;
            let val = if i < 128 { 0x0100i16 } else { -0x0100i16 };
            rom[offset..offset + 2].copy_from_slice(&val.to_be_bytes());
        }
        Memoria::nova(rom)
    }

    #[test]
    fn soltar_aneis_cria_objetos_quicando_e_limita_a_32() {
        let mut m = criar_memoria_com_seno();
        // Configura Sonic com 10 aneis na posicao (200, 150)
        ww(&mut m, V_RINGS, 10);
        ww(&mut m, V_PLAYER.wrapping_add(OB_X), 200);
        ww(&mut m, V_PLAYER.wrapping_add(OB_Y), 150);

        soltar_aneis(&mut m);

        // Deve reiniciar V_ANI3_TIME para 255
        assert_eq!(rb(&mut m, V_ANI3_TIME), 255);

        // Confere que foram criados 10 aneis quicando
        let mut contagem = 0;
        let mut a1 = V_LVLOBJSPACE;
        while a1 < V_LVLOBJEND {
            if rb(&mut m, a1.wrapping_add(OB_ID)) == ID_RING_LOSS {
                contagem += 1;
                assert_eq!(rb(&mut m, a1.wrapping_add(OB_ROUTINE)), ROTINA_QUICAR);
                assert_eq!(rw(&mut m, a1.wrapping_add(OB_X)), 200);
                assert_eq!(rw(&mut m, a1.wrapping_add(OB_Y)), 150);
                assert_eq!(rb(&mut m, a1.wrapping_add(OB_COLTYPE)), COL_ANEL);
                // Velocidade nao deve ser zero
                let vx = rw(&mut m, a1.wrapping_add(OB_VELX));
                let vy = rw(&mut m, a1.wrapping_add(OB_VELY));
                assert!(vx != 0 || vy != 0);
            }
            a1 = a1.wrapping_add(OBJECT_SIZE);
        }
        assert_eq!(contagem, 10);
    }

    #[test]
    fn aneis_sao_limitados_a_32_mesmo_com_mais_aneis() {
        let mut m = criar_memoria_com_seno();
        ww(&mut m, V_RINGS, 85);
        soltar_aneis(&mut m);

        let mut contagem = 0;
        let mut a1 = V_LVLOBJSPACE;
        while a1 < V_LVLOBJEND {
            if rb(&mut m, a1.wrapping_add(OB_ID)) == ID_RING_LOSS {
                contagem += 1;
            }
            a1 = a1.wrapping_add(OBJECT_SIZE);
        }
        assert_eq!(contagem, 32);
    }
}
