//! Plataformas de grama e marcadores de lava da Marble Zone (objetos 2F, 35 e 54).
use super::boot::{rom_b, rom_w};
use super::objetos::{animate_sprite, delete_object, display_sprite, fora_da_janela, solid_object};
use super::ram::*;
use super::{ErroAbertura, rb, rl, rw, wb, wl, ww};
use crate::memoria::Memoria;

const ORIG_X: u32 = 0x2A;
const ORIG_Y: u32 = 0x2C;
const COLISAO: u32 = 0x30;
const PRESSAO: u32 = 0x34;
const QUEIMANDO: u32 = 0x35;
const FILHOS: u32 = 0x36;
const PAI: u32 = 0x38;
const AJUSTE: u32 = 0x3C;

/// Marcador invisível de colisão com lava.
pub fn lava(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    if rb(m, a.wrapping_add(OB_ROUTINE)) == 0 {
        wb(m, a.wrapping_add(OB_ROUTINE), 2);
        let tipo = rb(m, a.wrapping_add(OB_SUBTYPE));
        let colisao = rom_b(m, 0xF856u32.wrapping_add(u32::from(tipo)));
        wb(m, a.wrapping_add(OB_COLTYPE), colisao);
        wl(m, a.wrapping_add(OB_MAP), 0xF89C);
        wb(m, a.wrapping_add(OB_RENDER), 0x84);
    }
    let x = rw(m, a.wrapping_add(OB_X));
    if fora_da_janela(m, x) {
        delete_object(m, a);
    }
    Ok(())
}

/// Plataforma móvel com altura obtida da tabela original de colisão.
pub fn plataforma(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    if rb(m, a.wrapping_add(OB_ROUTINE)) == 0 {
        wb(m, a.wrapping_add(OB_ROUTINE), 2);
        wl(m, a.wrapping_add(OB_MAP), 0xB95E);
        ww(m, a.wrapping_add(OB_GFX), 0xC000);
        wb(m, a.wrapping_add(OB_RENDER), 0x14);
        wb(m, a.wrapping_add(OB_PRIORITY), 5);
        let x = rw(m, a.wrapping_add(OB_X));
        let y = rw(m, a.wrapping_add(OB_Y));
        ww(m, a.wrapping_add(ORIG_X), x);
        ww(m, a.wrapping_add(ORIG_Y), y);
        let subtipo = rb(m, a.wrapping_add(OB_SUBTYPE));
        let tabela = 0xB554u32.wrapping_add(u32::from((subtipo >> 4) & 7).wrapping_mul(4));
        let desloc = rom_w(m, tabela);
        wl(
            m,
            a.wrapping_add(COLISAO),
            0xB554u32.wrapping_add(u32::from(desloc)),
        );
        wb(
            m,
            a.wrapping_add(OB_FRAME),
            rom_b(m, tabela.wrapping_add(2)),
        );
        wb(
            m,
            a.wrapping_add(OB_ACTWID),
            rom_b(m, tabela.wrapping_add(3)),
        );
        wb(m, a.wrapping_add(OB_SUBTYPE), subtipo & 15);
        wb(m, a.wrapping_add(OB_HEIGHT), 64);
    }
    let tipo = rb(m, a.wrapping_add(OB_SUBTYPE));
    match tipo & 7 {
        1..=4 => {
            let (offset, amplitude) = match tipo & 7 {
                1 => (2, 32u16),
                2 => (6, 48),
                3 => (10, 64),
                _ => (14, 96),
            };
            let mut delta = u16::from(rb(m, V_OSCILLATE.wrapping_add(offset)));
            if tipo & 8 != 0 {
                delta = amplitude.wrapping_sub(delta);
            }
            let y = rw(m, a.wrapping_add(ORIG_Y)).wrapping_sub(delta);
            ww(m, a.wrapping_add(OB_Y), y);
        }
        5 => pressao(m, a),
        _ => {}
    }
    let largura = u16::from(rb(m, a.wrapping_add(OB_ACTWID))).wrapping_add(11);
    let px = rw(m, V_PLAYER.wrapping_add(OB_X));
    let x = rw(m, a.wrapping_add(OB_X));
    let mut indice = px.wrapping_sub(x).wrapping_add(largura);
    if rb(m, a.wrapping_add(OB_RENDER)) & 1 != 0 {
        indice = (!indice).wrapping_add(largura.wrapping_mul(2));
    }
    let tabela = rl(m, a.wrapping_add(COLISAO));
    let base = rom_b(m, tabela);
    let altura = rom_b(m, tabela.wrapping_add(u32::from(indice >> 1)));
    let ajuste = u16::from(altura.wrapping_sub(base));
    let y = rw(m, a.wrapping_add(OB_Y));
    ww(m, a.wrapping_add(OB_Y), y.wrapping_sub(ajuste));
    let h = if rb(m, a.wrapping_add(OB_FRAME)) == 2 {
        48
    } else {
        32
    };
    let resultado = solid_object(m, a, largura, h, h);
    ww(m, a.wrapping_add(OB_Y), y);
    resultado?;
    display_sprite(m, a);
    if rb(m, a.wrapping_add(QUEIMANDO)) != 0 && rb(m, a.wrapping_add(OB_RENDER)) & 0x80 == 0 {
        let n = rb(m, a.wrapping_add(FILHOS));
        for i in 0..n {
            let endereco = a
                .wrapping_add(FILHOS)
                .wrapping_add(1)
                .wrapping_add(u32::from(i));
            let idx = rb(m, endereco);
            wb(m, endereco, 0);
            delete_object(
                m,
                V_OBJSPACE.wrapping_add(u32::from(idx).wrapping_mul(OBJECT_SIZE)),
            );
        }
        wb(m, a.wrapping_add(FILHOS), 0);
        wb(m, a.wrapping_add(QUEIMANDO), 0);
        wb(m, a.wrapping_add(PRESSAO), 0);
    } else {
        let origem = rw(m, a.wrapping_add(ORIG_X));
        if fora_da_janela(m, origem) {
            delete_object(m, a);
        }
    }
    Ok(())
}

fn proximo_livre(m: &mut Memoria, a: u32) -> Option<u32> {
    let mut candidato = a.wrapping_add(OBJECT_SIZE);
    while candidato < V_OBJSPACE_END {
        if rb(m, candidato.wrapping_add(OB_ID)) == 0 {
            return Some(candidato);
        }
        candidato = candidato.wrapping_add(OBJECT_SIZE);
    }
    None
}

fn registrar_filho(m: &mut Memoria, pai: u32, filho: u32) {
    let n = rb(m, pai.wrapping_add(FILHOS));
    wb(m, pai.wrapping_add(FILHOS), n.wrapping_add(1));
    let indice = u8::try_from((filho.wrapping_sub(V_OBJSPACE) / OBJECT_SIZE) & 0x7F).unwrap_or(0);
    wb(
        m,
        pai.wrapping_add(FILHOS)
            .wrapping_add(1)
            .wrapping_add(u32::from(n)),
        indice,
    );
}

fn pressao(m: &mut Memoria, a: u32) {
    let valor = rb(m, a.wrapping_add(PRESSAO));
    let valor = if rb(m, a.wrapping_add(OB_ROUTINE_2)) != 0 {
        valor.saturating_add(4).min(0x40)
    } else {
        valor.saturating_sub(2)
    };
    wb(m, a.wrapping_add(PRESSAO), valor);
    let seno = rom_w(m, SINE_DATA.wrapping_add(u32::from(valor).wrapping_mul(2)));
    let desloc = seno >> 4;
    let y = rw(m, a.wrapping_add(ORIG_Y)).wrapping_add(desloc);
    ww(m, a.wrapping_add(OB_Y), y);
    if valor == 0x20 && rb(m, a.wrapping_add(QUEIMANDO)) == 0 {
        wb(m, a.wrapping_add(QUEIMANDO), 1);
        if let Some(filho) = proximo_livre(m, a) {
            wb(m, filho.wrapping_add(OB_ID), 0x35);
            let x = rw(m, a.wrapping_add(OB_X)).wrapping_sub(64);
            ww(m, filho.wrapping_add(OB_X), x);
            let y = rw(m, a.wrapping_add(ORIG_Y)).wrapping_add(5);
            ww(m, filho.wrapping_add(ORIG_Y), y);
            let mapa = rl(m, a.wrapping_add(COLISAO));
            wl(m, filho.wrapping_add(COLISAO), mapa);
            wl(m, filho.wrapping_add(PAI), a);
            registrar_filho(m, a, filho);
        }
    }
    let n = rb(m, a.wrapping_add(FILHOS));
    for i in 0..n {
        let indice = rb(
            m,
            a.wrapping_add(FILHOS)
                .wrapping_add(1)
                .wrapping_add(u32::from(i)),
        );
        let filho = V_OBJSPACE.wrapping_add(u32::from(indice).wrapping_mul(OBJECT_SIZE));
        ww(m, filho.wrapping_add(AJUSTE), desloc);
    }
}

/// Chama que se espalha pela plataforma e seus filhos estacionários.
pub fn fogo(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    if rb(m, a.wrapping_add(OB_ROUTINE)) == 0 {
        let rotina = if rb(m, a.wrapping_add(OB_SUBTYPE)) == 0 {
            2
        } else {
            4
        };
        wb(m, a.wrapping_add(OB_ROUTINE), rotina);
        wl(m, a.wrapping_add(OB_MAP), 0xB9F8);
        ww(m, a.wrapping_add(OB_GFX), 0x345);
        let x = rw(m, a.wrapping_add(OB_X));
        ww(m, a.wrapping_add(ORIG_X), x);
        wb(m, a.wrapping_add(OB_RENDER), 4);
        wb(m, a.wrapping_add(OB_PRIORITY), 1);
        wb(m, a.wrapping_add(OB_COLTYPE), 0x8B);
        wb(m, a.wrapping_add(OB_ACTWID), 8);
        wb(m, V_SOUNDQUEUE1, 0xC8);
    }
    let ajuste = rw(m, a.wrapping_add(AJUSTE));
    if rb(m, a.wrapping_add(OB_ROUTINE)) == 2 {
        let distancia = rw(m, a.wrapping_add(OB_X))
            .wrapping_sub(rw(m, a.wrapping_add(ORIG_X)))
            .wrapping_add(12);
        let tabela = rl(m, a.wrapping_add(COLISAO));
        let altura = u16::from(rom_b(m, tabela.wrapping_add(u32::from(distancia >> 1))));
        let y = rw(m, a.wrapping_add(ORIG_Y)).wrapping_sub(altura);
        ww(m, a.wrapping_add(OB_Y), y.wrapping_add(ajuste));
        if distancia < 132 {
            let x = rl(m, a.wrapping_add(OB_X)).wrapping_add(0x10000);
            wl(m, a.wrapping_add(OB_X), x);
            if distancia < 128
                && x.wrapping_add(0x80000) & 0xFFFFF == 0
                && let Some(filho) = proximo_livre(m, a)
            {
                wb(m, filho.wrapping_add(OB_ID), 0x35);
                ww(
                    m,
                    filho.wrapping_add(OB_X),
                    u16::try_from(x >> 16).unwrap_or(0),
                );
                ww(m, filho.wrapping_add(ORIG_Y), y);
                ww(m, filho.wrapping_add(AJUSTE), ajuste);
                wb(m, filho.wrapping_add(OB_SUBTYPE), 1);
                let pai = rl(m, a.wrapping_add(PAI));
                registrar_filho(m, pai, filho);
            }
        }
    } else {
        let y = rw(m, a.wrapping_add(ORIG_Y)).wrapping_add(ajuste);
        ww(m, a.wrapping_add(OB_Y), y);
    }
    animate_sprite(m, a, 0xB956)?;
    display_sprite(m, a);
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn lava_inicializa_colisao_e_some_fora_da_janela() {
        let mut rom = vec![0; 0x10000];
        rom[0xF856..0xF859].copy_from_slice(&[0x96, 0x94, 0x95]);
        let mut m = Memoria::nova(rom);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x54);
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 1);
        ww(&mut m, a.wrapping_add(OB_X), 0x100);
        lava(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_COLTYPE)), 0x94);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_RENDER)), 0x84);
        ww(&mut m, V_SCREENPOSX, 0x1000);
        lava(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ID)), 0);
    }

    #[test]
    fn plataforma_inicia_e_movimenta_com_oscilador() {
        let mut rom = vec![0; 0x10000];
        rom[0xB554..0xB558].copy_from_slice(&[0, 0x10, 0, 64]);
        rom[0xB564..0xB5C0].fill(32);
        let mut m = Memoria::nova(rom);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x2F);
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 1);
        ww(&mut m, a.wrapping_add(OB_X), 0x100);
        ww(&mut m, a.wrapping_add(OB_Y), 0x200);
        wb(&mut m, V_OSCILLATE.wrapping_add(2), 12);
        plataforma(&mut m, a).unwrap();
        assert_eq!(rw(&mut m, a.wrapping_add(OB_Y)), 0x1F4);
        assert_eq!(rw(&mut m, a.wrapping_add(ORIG_Y)), 0x200);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_ACTWID)), 64);
        assert_eq!(rl(&mut m, a.wrapping_add(COLISAO)), 0xB564);
    }

    #[test]
    fn plataforma_pressionada_cria_chama_e_atualiza_sua_altura() {
        let mut rom = vec![0; 0x10000];
        let seno = usize::try_from(SINE_DATA.wrapping_add(0x40)).unwrap();
        rom[seno..seno.wrapping_add(2)].copy_from_slice(&0x80u16.to_be_bytes());
        let mut m = Memoria::nova(rom);
        let a = V_LVLOBJSPACE;
        let filho = a.wrapping_add(OBJECT_SIZE);
        wb(&mut m, a.wrapping_add(OB_ID), 0x2F);
        wb(&mut m, a.wrapping_add(OB_ROUTINE_2), 2);
        wb(&mut m, a.wrapping_add(PRESSAO), 0x1C);
        ww(&mut m, a.wrapping_add(OB_X), 0x200);
        ww(&mut m, a.wrapping_add(ORIG_Y), 0x300);
        wl(&mut m, a.wrapping_add(COLISAO), 0xB7A2);
        pressao(&mut m, a);
        assert_eq!(rb(&mut m, filho.wrapping_add(OB_ID)), 0x35);
        assert_eq!(rb(&mut m, a.wrapping_add(FILHOS)), 1);
        assert_eq!(rw(&mut m, filho.wrapping_add(OB_X)), 0x1C0);
        assert_eq!(rw(&mut m, filho.wrapping_add(AJUSTE)), 8);
        assert_eq!(rl(&mut m, filho.wrapping_add(PAI)), a);
    }
}
