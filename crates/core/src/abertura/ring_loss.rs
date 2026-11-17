//! Anéis espalhados quando o Sonic sofre dano (`RingLoss`, objeto `0x37`).
//!
//! Espelha `RingLoss` do disassembly (REV01, sem `FixBugs`). O `HurtSonic` só cria o objeto
//! na posição do Sonic. A rotina 0 (`RLoss_Count`) transforma o próprio objeto no primeiro
//! anel, cria os outros com `FindFreeObj` (no máximo 32 no total) e segue direto para a
//! rotina 2 (`RLoss_Bounce`), porque o original não tem `rts` entre as duas.

use super::fisica::{calc_sine, obj_floor_dist, speed_to_pos};
use super::objetos::{collect_ring, delete_object, display_sprite, find_free_obj, ring_sparkle};
use super::ram::{
    ARTTILE_RING, F_RINGCOUNT, ID_RING_LOSS, MAP_RING, OB_ACTWID, OB_COLTYPE, OB_FRAME, OB_GFX,
    OB_HEIGHT, OB_ID, OB_MAP, OB_PRIORITY, OB_RENDER, OB_ROUTINE, OB_VELX, OB_VELY, OB_WIDTH, OB_X,
    OB_Y, OBJECT_SIZE, SFX_RINGLOSS, V_ANI3_FRAME, V_ANI3_TIME, V_LIFECOUNT, V_LIMITBTM2,
    V_OBJSPACE, V_RINGS, V_SOUNDQUEUE1, V_VBLANK_BYTE,
};
use super::{ErroAbertura, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;

/// `rloss_spread`: aumento de velocidade (2) no byte alto e ângulo inicial (`$88`) no baixo.
const ESPALHAMENTO_INICIAL: u16 = 0x288;
/// `col_12x12|col_item`: o anel pode ser recolhido.
const COL_ANEL: u8 = 0x47;
/// `Tile_Pal2`: linha de paleta 1 no padrão do sprite.
const PALETA_2: u16 = 0x2000;
/// `sprite_cam_field`: posição relativa à câmera.
const RENDER_CAMERA: u8 = 4;

/// `RingLoss`: despacha pela rotina do objeto em `a0`.
pub fn executar(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    match rb(m, a0.wrapping_add(OB_ROUTINE)) {
        0 => {
            contar(m, a0);
            // Sem `rts` no original: o primeiro anel já quica neste quadro.
            quicar(m, a0);
            Ok(())
        }
        2 => {
            quicar(m, a0);
            Ok(())
        }
        4 => {
            // `RLoss_Collect`, que cai direto em `RLoss_Sparkle`.
            let rotina = rb(m, a0.wrapping_add(OB_ROUTINE)).wrapping_add(2);
            wb(m, a0.wrapping_add(OB_ROUTINE), rotina);
            wb(m, a0.wrapping_add(OB_COLTYPE), 0);
            wb(m, a0.wrapping_add(OB_PRIORITY), 1);
            collect_ring(m);
            ring_sparkle(m, a0)
        }
        6 => ring_sparkle(m, a0),
        8 => {
            delete_object(m, a0);
            Ok(())
        }
        _ => Err(ErroAbertura::NaoImplementado("rotina de RingLoss")),
    }
}

/// `RLoss_Count`: cria os anéis em pares espelhados, zera os anéis do Sonic e toca o som.
fn contar(m: &mut Memoria, a0: u32) {
    // `d5`: anéis a criar menos 1, para o `dbf`. A comparação do original é sem sinal.
    let mut d5 = rw(m, V_RINGS).min(32).wrapping_sub(1);
    let mut d4 = ESPALHAMENTO_INICIAL;
    let mut vel_x: u16 = 0;
    let mut vel_y: u16 = 0;
    let x = rw(m, a0.wrapping_add(OB_X));
    let y = rw(m, a0.wrapping_add(OB_Y));
    // O primeiro anel ocupa o próprio slot do objeto (`movea.l a0,a1`).
    let mut a1 = a0;
    loop {
        // `.makerings`
        wb(m, a1.wrapping_add(OB_ID), ID_RING_LOSS);
        let rotina = rb(m, a1.wrapping_add(OB_ROUTINE)).wrapping_add(2);
        wb(m, a1.wrapping_add(OB_ROUTINE), rotina);
        wb(m, a1.wrapping_add(OB_HEIGHT), 8);
        wb(m, a1.wrapping_add(OB_WIDTH), 8);
        ww(m, a1.wrapping_add(OB_X), x);
        ww(m, a1.wrapping_add(OB_Y), y);
        wl(m, a1.wrapping_add(OB_MAP), MAP_RING);
        ww(m, a1.wrapping_add(OB_GFX), ARTTILE_RING | PALETA_2);
        wb(m, a1.wrapping_add(OB_RENDER), RENDER_CAMERA);
        wb(m, a1.wrapping_add(OB_PRIORITY), 3);
        wb(m, a1.wrapping_add(OB_COLTYPE), COL_ANEL);
        wb(m, a1.wrapping_add(OB_ACTWID), 8);
        // Sem `FixBugs`, cada anel novo reinicia o temporizador de todos.
        wb(m, V_ANI3_TIME, 255);
        // Com `d4` negativo, o anel repete a velocidade anterior com X invertido.
        if d4 & 0x8000 == 0 {
            let [aumento, angulo] = d4.to_be_bytes();
            let (seno, cosseno) = calc_sine(m, angulo);
            vel_x = u16::from_be_bytes(seno.to_be_bytes()).wrapping_shl(u32::from(aumento));
            vel_y = u16::from_be_bytes(cosseno.to_be_bytes()).wrapping_shl(u32::from(aumento));
            // `addi.b #$10,d4`: espalha o ângulo sem mexer no byte de aumento.
            let (baixo, vai_um) = angulo.overflowing_add(0x10);
            d4 = u16::from_be_bytes([aumento, baixo]);
            if vai_um {
                // A cada 8 pares, recua o ângulo e reduz o aumento.
                let (recuado, emprestou) = d4.overflowing_sub(0x80);
                d4 = if emprestou {
                    ESPALHAMENTO_INICIAL
                } else {
                    recuado
                };
            }
        }
        // `.setRingSpeed`
        ww(m, a1.wrapping_add(OB_VELX), vel_x);
        ww(m, a1.wrapping_add(OB_VELY), vel_y);
        vel_x = vel_x.wrapping_neg();
        d4 = d4.wrapping_neg();
        // `dbf d5,.loop`
        d5 = d5.wrapping_sub(1);
        if d5 == u16::MAX {
            break;
        }
        // `.loop`: sem slot livre, para de criar anéis.
        match find_free_obj(m) {
            Some(novo) => a1 = novo,
            None => break,
        }
    }
    // `.resetcounter`
    ww(m, V_RINGS, 0);
    // `$80`: o HUD redesenha todos os dígitos dos anéis.
    wb(m, F_RINGCOUNT, 0x80);
    wb(m, V_LIFECOUNT, 0);
    // `QueueSound2`.
    wb(m, V_SOUNDQUEUE1, SFX_RINGLOSS);
}

/// `RLoss_Bounce`: move o anel, quica no chão a cada 4 quadros e apaga quando o tempo acaba.
fn quicar(m: &mut Memoria, a0: u32) {
    let quadro = rb(m, V_ANI3_FRAME);
    wb(m, a0.wrapping_add(OB_FRAME), quadro);
    speed_to_pos(m, a0);
    let vel_y = rw(m, a0.wrapping_add(OB_VELY)).wrapping_add(0x18);
    ww(m, a0.wrapping_add(OB_VELY), vel_y);
    // Subindo, não testa o chão (`bmi.s .chkdel`).
    if vel_y & 0x8000 == 0 {
        // O original espalha o teste de chão pelos quadros com o `d7` do laço de objetos.
        let fase = rb(m, V_VBLANK_BYTE).wrapping_add(d7_do_slot(a0)) & 3;
        if fase == 0 {
            let distancia = obj_floor_dist(m, a0);
            if distancia & 0x8000 != 0 {
                // Bateu no chão: alinha à superfície e quica com 3/4 da velocidade.
                let y = rw(m, a0.wrapping_add(OB_Y)).wrapping_add(distancia);
                ww(m, a0.wrapping_add(OB_Y), y);
                let v = i16::from_be_bytes(rw(m, a0.wrapping_add(OB_VELY)).to_be_bytes());
                let amortecido = v.wrapping_sub(v >> 2);
                ww(
                    m,
                    a0.wrapping_add(OB_VELY),
                    u16::from_be_bytes(amortecido.wrapping_neg().to_be_bytes()),
                );
            }
        }
    }
    // `.chkdel`
    if rb(m, V_ANI3_TIME) == 0 {
        delete_object(m, a0);
        return;
    }
    // Abaixo do limite inferior da fase mais uma tela: comparação sem sinal (`blo`).
    let limite = rw(m, V_LIMITBTM2).wrapping_add(224);
    if limite < rw(m, a0.wrapping_add(OB_Y)) {
        delete_object(m, a0);
        return;
    }
    display_sprite(m, a0);
}

/// `d7` de `ExecuteObjects` ao rodar o objeto em `a0`: `$7F` menos o índice do slot.
fn d7_do_slot(a0: u32) -> u8 {
    let indice = a0.wrapping_sub(V_OBJSPACE) / OBJECT_SIZE;
    let d7 = 127u32.wrapping_sub(indice);
    u8::try_from(d7 & 0xFF).unwrap_or(0)
}

#[cfg(test)]
mod testes {
    use super::super::ram::{SINE_DATA, V_LVLOBJEND, V_LVLOBJSPACE};
    use super::*;

    /// Memória com uma tabela de seno sintética: 0x100 na primeira metade, -0x100 na segunda.
    fn memoria_com_seno() -> Memoria {
        let mut rom = vec![0; 0x10000];
        let base = usize::try_from(SINE_DATA).unwrap();
        for i in 0..320 {
            let valor: i16 = if i < 128 { 0x100 } else { -0x100 };
            let posicao = base + i * 2;
            rom[posicao..posicao + 2].copy_from_slice(&valor.to_be_bytes());
        }
        Memoria::nova(rom)
    }

    fn aneis_criados(m: &mut Memoria) -> Vec<u32> {
        let mut slots = Vec::new();
        let mut a = V_LVLOBJSPACE;
        while a < V_LVLOBJEND {
            if rb(m, a.wrapping_add(OB_ID)) == ID_RING_LOSS {
                slots.push(a);
            }
            a = a.wrapping_add(OBJECT_SIZE);
        }
        slots
    }

    fn objeto_contador(m: &mut Memoria, aneis: u16) -> u32 {
        let a0 = V_LVLOBJSPACE;
        wb(m, a0.wrapping_add(OB_ID), ID_RING_LOSS);
        ww(m, a0.wrapping_add(OB_X), 200);
        ww(m, a0.wrapping_add(OB_Y), 150);
        ww(m, V_RINGS, aneis);
        // Fundo da fase bem abaixo, para nenhum anel ser apagado pelo limite.
        ww(m, V_LIMITBTM2, 0x700);
        a0
    }

    #[test]
    fn primeiro_anel_reaproveita_o_slot_e_o_total_e_limitado_a_32() {
        let mut m = memoria_com_seno();
        let a0 = objeto_contador(&mut m, 40);
        executar(&mut m, a0).unwrap();
        let slots = aneis_criados(&mut m);
        assert_eq!(slots.len(), 32);
        assert_eq!(slots[0], a0);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(rw(&mut m, V_RINGS), 0);
        assert_eq!(rb(&mut m, F_RINGCOUNT), 0x80);
        assert_eq!(rb(&mut m, V_SOUNDQUEUE1), SFX_RINGLOSS);
        assert_eq!(rb(&mut m, V_ANI3_TIME), 255);
    }

    #[test]
    fn aneis_saem_em_pares_com_x_espelhado() {
        let mut m = memoria_com_seno();
        let a0 = objeto_contador(&mut m, 2);
        executar(&mut m, a0).unwrap();
        let slots = aneis_criados(&mut m);
        assert_eq!(slots.len(), 2);
        // O segundo anel ainda não rodou a rotina 2: a velocidade é a que o contador gravou.
        let vx1 = rw(&mut m, slots[1].wrapping_add(OB_VELX));
        let vy1 = rw(&mut m, slots[1].wrapping_add(OB_VELY));
        // Ângulo inicial `$88`: seno e cosseno sintéticos de -0x100, com aumento de 2 (-0x400).
        // O par repete a velocidade com X invertido: (+0x400, -0x400).
        assert_eq!(vx1, 0x400);
        assert_eq!(vy1, 0x400u16.wrapping_neg());
    }

    #[test]
    fn d7_e_127_menos_o_indice_do_slot() {
        assert_eq!(d7_do_slot(V_OBJSPACE), 127);
        assert_eq!(d7_do_slot(V_LVLOBJSPACE), 127 - 32);
    }

    #[test]
    fn temporizador_zerado_apaga_o_anel() {
        let mut m = memoria_com_seno();
        let a0 = V_LVLOBJSPACE;
        wb(&mut m, a0.wrapping_add(OB_ID), ID_RING_LOSS);
        wb(&mut m, a0.wrapping_add(OB_ROUTINE), 2);
        ww(&mut m, V_LIMITBTM2, 0x700);
        wb(&mut m, V_ANI3_TIME, 0);
        executar(&mut m, a0).unwrap();
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ID)), 0);
    }
}
