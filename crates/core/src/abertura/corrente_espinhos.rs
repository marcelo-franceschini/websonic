//! Bola de espinhos em corrente das zonas Labyrinth e Spring Yard, objeto 0x57.
use super::super::fisica::calc_sine;
use super::super::ram::*;
use super::super::{ErroAbertura, rb, rl, rw, wb, wl, ww};
use super::{delete_object, display_sprite, find_free_obj, fora_da_janela};
use crate::memoria::Memoria;

const FILHOS: u32 = 0x29;
const ORIG_Y: u32 = 0x38;
const ORIG_X: u32 = 0x3a;
const RAIO: u32 = 0x3c;
const VELOCIDADE: u32 = 0x3e;

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => iniciar(m, a),
        2 => {}
        4 => {
            display_sprite(m, a);
            return Ok(());
        }
        _ => {
            return Err(ErroAbertura::NaoImplementado(
                "rotina da corrente de espinhos",
            ));
        }
    }
    girar(m, a);
    let x = rw(m, a.wrapping_add(ORIG_X));
    if fora_da_janela(m, x) {
        let count = rb(m, a.wrapping_add(FILHOS));
        for i in 0..=count {
            let indice = rb(
                m,
                a.wrapping_add(FILHOS)
                    .wrapping_add(1)
                    .wrapping_add(u32::from(i)),
            );
            let objeto = V_OBJSPACE.wrapping_add(u32::from(indice).wrapping_mul(OBJECT_SIZE));
            delete_object(m, objeto);
        }
    } else {
        display_sprite(m, a);
    }
    Ok(())
}

fn indice(a: u32) -> u8 {
    u8::try_from((a.wrapping_sub(V_OBJSPACE) / OBJECT_SIZE) & 0x7f).unwrap_or(0)
}

fn iniciar(m: &mut Memoria, a: u32) {
    wb(m, a.wrapping_add(OB_ROUTINE), 2);
    let labirinto = rb(m, V_ZONE) == ID_LZ;
    wl(
        m,
        a.wrapping_add(OB_MAP),
        if labirinto { 0x11032 } else { 0x1102a },
    );
    ww(
        m,
        a.wrapping_add(OB_GFX),
        if labirinto { 0x310 } else { 0x3ba },
    );
    wb(m, a.wrapping_add(OB_RENDER), 4);
    wb(m, a.wrapping_add(OB_PRIORITY), 4);
    wb(m, a.wrapping_add(OB_ACTWID), 8);
    let x = rw(m, a.wrapping_add(OB_X));
    let y = rw(m, a.wrapping_add(OB_Y));
    ww(m, a.wrapping_add(ORIG_X), x);
    ww(m, a.wrapping_add(ORIG_Y), y);
    wb(
        m,
        a.wrapping_add(OB_COLTYPE),
        if labirinto { 0 } else { 0x98 },
    );
    let subtipo = rb(m, a.wrapping_add(OB_SUBTYPE));
    let velocidade = i16::from(i8::from_be_bytes([subtipo & 0xf0])).wrapping_mul(8);
    ww(
        m,
        a.wrapping_add(VELOCIDADE),
        u16::from_be_bytes(velocidade.to_be_bytes()),
    );
    let estado = rb(m, a.wrapping_add(OB_STATUS));
    wb(
        m,
        a.wrapping_add(OB_ANGLE),
        (estado & 3).rotate_right(2) & 0xc0,
    );
    wb(m, a.wrapping_add(FILHOS), 0);
    let quantidade = subtipo & 7;
    let mut raio = quantidade.wrapping_mul(16);
    wb(m, a.wrapping_add(RAIO), raio);
    let desejados = if subtipo & 8 != 0 {
        quantidade.saturating_sub(1)
    } else {
        quantidade
    };
    let mut criados = 0u8;
    for _ in 0..desejados {
        let Some(filho) = find_free_obj(m) else {
            break;
        };
        criados = criados.wrapping_add(1);
        wb(m, a.wrapping_add(FILHOS), criados);
        wb(
            m,
            a.wrapping_add(FILHOS).wrapping_add(u32::from(criados)),
            indice(filho),
        );
        wb(m, filho.wrapping_add(OB_ROUTINE), 4);
        let id = rb(m, a.wrapping_add(OB_ID));
        wb(m, filho.wrapping_add(OB_ID), id);
        let mapa = rl(m, a.wrapping_add(OB_MAP));
        wl(m, filho.wrapping_add(OB_MAP), mapa);
        let arte = rw(m, a.wrapping_add(OB_GFX));
        ww(m, filho.wrapping_add(OB_GFX), arte);
        for campo in [OB_RENDER, OB_PRIORITY, OB_ACTWID, OB_COLTYPE] {
            let valor = rb(m, a.wrapping_add(campo));
            wb(m, filho.wrapping_add(campo), valor);
        }
        raio = raio.wrapping_sub(16);
        wb(m, filho.wrapping_add(RAIO), raio);
        if labirinto && raio == 0 {
            wb(m, filho.wrapping_add(OB_FRAME), 2);
        }
    }
    wb(
        m,
        a.wrapping_add(FILHOS)
            .wrapping_add(1)
            .wrapping_add(u32::from(criados)),
        indice(a),
    );
    if labirinto {
        wb(m, a.wrapping_add(OB_COLTYPE), 0x8b);
        wb(m, a.wrapping_add(OB_FRAME), 1);
    }
}

fn girar(m: &mut Memoria, a: u32) {
    let angulo = rw(m, a.wrapping_add(OB_ANGLE)).wrapping_add(rw(m, a.wrapping_add(VELOCIDADE)));
    ww(m, a.wrapping_add(OB_ANGLE), angulo);
    let (seno, cosseno) = calc_sine(m, angulo.to_be_bytes()[0]);
    let origem_y = rw(m, a.wrapping_add(ORIG_Y));
    let origem_x = rw(m, a.wrapping_add(ORIG_X));
    let quantidade = rb(m, a.wrapping_add(FILHOS));
    for i in 0..=quantidade {
        let n = rb(
            m,
            a.wrapping_add(FILHOS)
                .wrapping_add(1)
                .wrapping_add(u32::from(i)),
        );
        let objeto = V_OBJSPACE.wrapping_add(u32::from(n).wrapping_mul(OBJECT_SIZE));
        let raio = i32::from(rb(m, objeto.wrapping_add(RAIO)));
        let dx = i32::from(cosseno).wrapping_mul(raio) >> 8;
        let dy = i32::from(seno).wrapping_mul(raio) >> 8;
        let bx = dx.to_be_bytes();
        let by = dy.to_be_bytes();
        ww(
            m,
            objeto.wrapping_add(OB_X),
            origem_x.wrapping_add(u16::from_be_bytes([bx[2], bx[3]])),
        );
        ww(
            m,
            objeto.wrapping_add(OB_Y),
            origem_y.wrapping_add(u16::from_be_bytes([by[2], by[3]])),
        );
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn lz_cria_cinco_elos_e_gira_em_torno_da_base() {
        let mut rom = vec![0; 0x20000];
        rom[0x2a6a..0x2a6c].copy_from_slice(&256u16.to_be_bytes());
        let mut m = Memoria::nova(rom);
        let a = V_LVLOBJSPACE;
        wb(&mut m, V_ZONE, ID_LZ);
        wb(&mut m, a.wrapping_add(OB_ID), 0x57);
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 0x05);
        ww(&mut m, a.wrapping_add(OB_X), 0x200);
        ww(&mut m, a.wrapping_add(OB_Y), 0x300);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(FILHOS)), 5);
        assert_eq!(rw(&mut m, a.wrapping_add(OB_X)), 0x250);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_COLTYPE)), 0x8b);
        let base = a.wrapping_add(5 * OBJECT_SIZE);
        assert_eq!(rb(&mut m, base.wrapping_add(OB_COLTYPE)), 0);
        assert_eq!(rb(&mut m, base.wrapping_add(OB_FRAME)), 2);
        assert_eq!(rw(&mut m, base.wrapping_add(OB_X)), 0x200);
    }
    #[test]
    fn remover_pai_fora_da_tela_remove_todos_os_elos() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x57);
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 2);
        ww(&mut m, a.wrapping_add(OB_X), 0x1000);
        executar(&mut m, a).unwrap();
        for i in 0..3 {
            assert_eq!(
                rb(&mut m, a.wrapping_add(i * OBJECT_SIZE).wrapping_add(OB_ID)),
                0
            );
        }
    }
}
