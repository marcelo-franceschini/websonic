//! Água, túneis, escorregadores e eventos dinâmicos de Labyrinth Zone (REV01).
use super::boot::{rom_b, rom_w};
use super::ram::*;
use super::{ErroAbertura, objetos, plc, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;

const WATER1: u32 = 0xFF_F646;
const WATER2: u32 = 0xFF_F648;
const WATER3: u32 = 0xFF_F64A;
const F_WATER: u32 = 0xFF_F64C;
const WATER_ROUTINE: u32 = 0xFF_F64D;
const HBLANK_LINE: u32 = 0xFF_F625;
const SWITCH: u32 = 0xFF_F7E0;
const TUNNEL_DISALLOW: u32 = 0xFF_F7C9;
const WATER_HEIGHT: u32 = 0x3C2C;
const WIND_DATA: u32 = 0x3F0E;
const SLIDE_SPEEDS: u32 = 0x3FC2;
const SLIDE_CHUNKS: u32 = 0x3FCA;

/// Inicializa o nível de água e habilita a interrupção horizontal.
pub fn iniciar_agua(m: &mut Memoria) {
    let ato = u32::from(rb(m, V_ACT));
    let altura = rom_w(m, WATER_HEIGHT.wrapping_add(ato.wrapping_mul(2)));
    ww(m, VDP_CONTROLE, 0x8014);
    for destino in [WATER1, WATER2, WATER3] {
        ww(m, destino, altura);
    }
    wb(m, WATER_ROUTINE, 0);
    wb(m, F_WTR_STATE, 0);
    wb(m, F_WATER, 1);
}

/// Efeitos de nível da LZ, executados antes dos objetos.
pub fn efeitos(m: &mut Memoria) {
    if rb(m, V_ZONE) != ID_LZ {
        return;
    }
    if rb(m, F_NOBGSCROLL) == 0 && rb(m, V_PLAYER.wrapping_add(OB_ROUTINE)) < 6 {
        tunel(m);
        escorregador(m);
        agua_dinamica(m);
    }
    wb(m, F_WTR_STATE, 0);
    let oscilacao = u16::from(rb(m, V_OSCILLATE.wrapping_add(2))) >> 1;
    let altura = rw(m, WATER2).wrapping_add(oscilacao);
    ww(m, WATER1, altura);
    let camera = rw(m, V_SCREENPOSY);
    let diferenca = altura.wrapping_sub(camera);
    if altura < camera && i16::from_be_bytes(diferenca.to_be_bytes()) < 0 {
        wb(m, F_WTR_STATE, 1);
    }
    let linha = if diferenca < 223 {
        diferenca.to_be_bytes()[1]
    } else {
        223
    };
    wb(m, HBLANK_LINE, linha);
}

fn agua_dinamica(m: &mut Memoria) {
    let x = rw(m, V_SCREENPOSX);
    let y = rw(m, V_PLAYER.wrapping_add(OB_Y));
    let nivel = rw(m, WATER2);
    let rotina = rb(m, WATER_ROUTINE);
    let mut alvo = rw(m, WATER3);
    let mut imediato = false;
    match rb(m, V_ACT) {
        0 => {
            if rotina == 0 {
                alvo = 0xB8;
                if x >= 0x600 {
                    alvo = 0x108;
                    if y < 0x200 {
                        if x >= 0xC80 {
                            alvo = 0xE8;
                        }
                        if x >= 0x1500 {
                            alvo = 0x108;
                        }
                    } else {
                        if x >= 0xC00 {
                            alvo = 0x318;
                        }
                        if x >= 0x1080 {
                            wb(m, SWITCH.wrapping_add(5), 0x80);
                            alvo = 0x5C8;
                        }
                        if x >= 0x1380 {
                            alvo = 0x3A8;
                            if nivel == alvo {
                                wb(m, WATER_ROUTINE, 1);
                            }
                        }
                    }
                }
            } else if rotina == 1 && y < 0x2E0 {
                alvo = 0x3A8;
                if x >= 0x1300 {
                    alvo = 0x108;
                    wb(m, WATER_ROUTINE, 2);
                }
            }
        }
        1 => {
            alvo = if x < 0x500 {
                0x328
            } else if x < 0xB00 {
                0x3C8
            } else {
                0x428
            };
        }
        2 => match rotina {
            0 => {
                alvo = 0x900;
                if x >= 0x600 && (0x3C0..0x600).contains(&y) {
                    alvo = 0x4C8;
                    wb(m, V_LVLLAYOUT_FG.wrapping_add(0x106), 0x4B);
                    wb(m, WATER_ROUTINE, 1);
                    wb(m, V_SOUNDQUEUE1, 0xB7);
                }
                imediato = true;
            }
            1 => {
                alvo = if x < 0x770 { 0x4C8 } else { 0x308 };
                if x >= 0x1400 && (rw(m, WATER3) == 0x508 || !(0x280..0x600).contains(&y)) {
                    alvo = 0x508;
                    imediato = true;
                    if x >= 0x1770 {
                        wb(m, WATER_ROUTINE, 2);
                    }
                }
            }
            2 => {
                alvo = if x < 0x1860 { 0x508 } else { 0x188 };
                if x >= 0x1860 && (x >= 0x1AF0 || nivel == alvo) {
                    wb(m, WATER_ROUTINE, 3);
                }
            }
            3 => {
                alvo = if x < 0x1AF0 { 0x188 } else { 0x900 };
                if x >= 0x1BC0 {
                    wb(m, WATER_ROUTINE, 4);
                    alvo = 0x608;
                    ww(m, WATER2, 0x7C0);
                    wb(m, SWITCH.wrapping_add(8), 1);
                } else {
                    imediato = true;
                }
            }
            _ => {
                if x >= 0x1E00 {
                    alvo = 0x128;
                }
            }
        },
        3 => {
            alvo = if x < 0xF00 { 0x228 } else { 0x4C8 };
        }
        _ => return,
    }
    ww(m, WATER3, alvo);
    if imediato {
        ww(m, WATER2, alvo);
    }
    let atual = rw(m, WATER2);
    if alvo != atual {
        let passo = u16::from(rb(m, F_WATER));
        ww(
            m,
            WATER2,
            if alvo > atual {
                atual.wrapping_add(passo)
            } else {
                atual.wrapping_sub(passo)
            },
        );
    }
}

fn tunel(m: &mut Memoria) {
    if rw(m, V_DEBUGUSE) != 0 {
        return;
    }
    let ato = rb(m, V_ACT);
    let indice = if ato == 0 {
        0
    } else {
        u32::from(ato).wrapping_add(1)
    };
    let quantidade = if ato == 0 { 2 } else { 1 };
    let x = rw(m, V_PLAYER.wrapping_add(OB_X));
    let y = rw(m, V_PLAYER.wrapping_add(OB_Y));
    for i in 0..quantidade {
        let tabela = WIND_DATA.wrapping_add(indice.wrapping_add(i).wrapping_mul(8));
        let esquerda = rom_w(m, tabela);
        if x < esquerda
            || x >= rom_w(m, tabela.wrapping_add(4))
            || y < rom_w(m, tabela.wrapping_add(2))
            || y >= rom_w(m, tabela.wrapping_add(6))
        {
            continue;
        }
        let frame = rb(m, V_VBLANK_BYTE) & 0x3F;
        if frame == 0 {
            wb(m, V_SOUNDQUEUE1, 0xD0);
        }
        if rb(m, TUNNEL_DISALLOW) != 0 {
            return;
        }
        if rb(m, V_PLAYER.wrapping_add(OB_ROUTINE)) >= 4 {
            wb(m, F_WTUNNELMODE, 0);
            return;
        }
        wb(m, F_WTUNNELMODE, 1);
        let d0 = (x & 0xFF00) | u16::from(frame);
        let mut novo_y = y;
        if d0.wrapping_sub(128) < esquerda {
            novo_y = if ato == 1 {
                novo_y.wrapping_sub(2)
            } else {
                novo_y.wrapping_add(2)
            };
        }
        let botoes = rb(m, V_JPADHOLD2);
        if botoes & 1 != 0 {
            novo_y = novo_y.wrapping_sub(1);
        }
        if botoes & 2 != 0 {
            novo_y = novo_y.wrapping_add(1);
        }
        ww(m, V_PLAYER.wrapping_add(OB_X), x.wrapping_add(4));
        ww(m, V_PLAYER.wrapping_add(OB_Y), novo_y);
        ww(m, V_PLAYER.wrapping_add(OB_VELX), 0x400);
        ww(m, V_PLAYER.wrapping_add(OB_VELY), 0);
        wb(m, V_PLAYER.wrapping_add(OB_ANIM), 0x0F);
        let status = rb(m, V_PLAYER.wrapping_add(OB_STATUS));
        wb(m, V_PLAYER.wrapping_add(OB_STATUS), status | 2);
        return;
    }
    if rb(m, F_WTUNNELMODE) != 0 {
        wb(m, V_PLAYER.wrapping_add(OB_ANIM), ANIM_WALK);
        wb(m, F_WTUNNELMODE, 0);
    }
}

fn escorregador(m: &mut Memoria) {
    let status = rb(m, V_PLAYER.wrapping_add(OB_STATUS));
    let x = rw(m, V_PLAYER.wrapping_add(OB_X));
    let y = rw(m, V_PLAYER.wrapping_add(OB_Y));
    let indice = (y >> 1 & 0x380).wrapping_add(x >> 8 & 0x7F);
    let chunk = rb(m, V_LVLLAYOUT_FG.wrapping_add(u32::from(indice)));
    let mut encontrado = None;
    if status & 2 == 0 {
        for i in (0..7u32).rev() {
            if rom_b(m, SLIDE_CHUNKS.wrapping_add(i)) == chunk {
                encontrado = Some(i);
                break;
            }
        }
    }
    if let Some(i) = encontrado {
        let velocidade = rom_b(m, SLIDE_SPEEDS.wrapping_add(i));
        wb(
            m,
            V_PLAYER.wrapping_add(OB_STATUS),
            (status & !1) | u8::from(velocidade & 0x80 != 0),
        );
        ww(
            m,
            V_PLAYER.wrapping_add(OB_INERTIA),
            u16::from(velocidade).wrapping_shl(8),
        );
        wb(m, V_PLAYER.wrapping_add(OB_ANIM), 0x1B);
        wb(m, F_SLIDEMODE, 1);
        if rb(m, V_VBLANK_BYTE) & 0x1F == 0 {
            wb(m, V_SOUNDQUEUE1, 0xD0);
        }
    } else if rb(m, F_SLIDEMODE) != 0 {
        ww(m, V_PLAYER.wrapping_add(SON_LOCKTIME), 5);
        wb(m, F_SLIDEMODE, 0);
    }
}

/// Eventos de câmera da LZ. O retorno não altera os limites verticais comuns.
pub fn eventos(m: &mut Memoria) -> Result<(), ErroAbertura> {
    match rb(m, V_ACT) {
        0 | 1 => Ok(()),
        2 => {
            let bloco = V_LVLLAYOUT_FG.wrapping_add(0x106);
            if rb(m, SWITCH.wrapping_add(15)) != 0 && rb(m, bloco) != 7 {
                wb(m, bloco, 7);
                wb(m, V_SOUNDQUEUE1, 0xB7);
            }
            if rb(m, V_DLE_ROUTINE) == 0
                && rw(m, V_SCREENPOSX) >= 0x1CA0
                && rw(m, V_SCREENPOSY) < 0x600
            {
                if let Some(slot) = objetos::find_free_obj(m) {
                    wb(m, slot.wrapping_add(OB_ID), 0x77);
                }
                wb(m, V_SOUNDQUEUE0, 0x8C);
                wb(m, F_LOCKSCREEN, 1);
                let rotina = rb(m, V_DLE_ROUTINE).wrapping_add(2);
                wb(m, V_DLE_ROUTINE, rotina);
                plc::add_plc(m, 0x11)?;
            }
            Ok(())
        }
        3 => {
            if rw(m, V_SCREENPOSX) >= 0xD00 && rw(m, V_PLAYER.wrapping_add(OB_Y)) < 0x18 {
                wb(m, V_LASTLAMP, 0);
                ww(m, F_RESTART, 1);
                ww(m, V_ZONE_ACT, ID_FZ);
                wb(m, F_PLAYERCTRL, 1);
            }
            Ok(())
        }
        _ => Err(ErroAbertura::NaoImplementado("ato da LZ")),
    }
}

/// Objeto 0x1B: duas metades da superfície, com quadros contínuos durante pausa.
pub fn superficie(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        wb(m, a.wrapping_add(OB_ROUTINE), 2);
        wl(m, a.wrapping_add(OB_MAP), 0x11840);
        ww(m, a.wrapping_add(OB_GFX), 0xC300);
        wb(m, a.wrapping_add(OB_RENDER), 4);
        wb(m, a.wrapping_add(OB_ACTWID), 128);
        let x = rw(m, a.wrapping_add(OB_X));
        ww(m, a.wrapping_add(0x30), x);
    } else if rotina != 2 {
        return Err(ErroAbertura::NaoImplementado(
            "rotina da superfície da água",
        ));
    }
    let camera = rw(m, V_SCREENPOSX) & 0xFFE0;
    let origem = rw(m, a.wrapping_add(0x30));
    let x = camera
        .wrapping_add(origem)
        .wrapping_add(if rb(m, V_FRAMEBYTE) & 1 == 0 { 0 } else { 32 });
    ww(m, a.wrapping_add(OB_X), x);
    let y = rw(m, WATER1);
    ww(m, a.wrapping_add(OB_Y), y);
    if rb(m, a.wrapping_add(0x32)) == 0 && rb(m, V_JPADPRESS1) & BTN_START != 0 {
        let quadro = rb(m, a.wrapping_add(OB_FRAME)).wrapping_add(3);
        wb(m, a.wrapping_add(OB_FRAME), quadro);
        wb(m, a.wrapping_add(0x32), 1);
    } else {
        if rb(m, a.wrapping_add(0x32)) != 0 {
            if rw(m, F_PAUSE) != 0 {
                objetos::display_sprite(m, a);
                return Ok(());
            }
            wb(m, a.wrapping_add(0x32), 0);
            let quadro = rb(m, a.wrapping_add(OB_FRAME)).wrapping_sub(3);
            wb(m, a.wrapping_add(OB_FRAME), quadro);
        }
        let tempo = rb(m, a.wrapping_add(OB_TIME_FRAME)).wrapping_sub(1);
        wb(m, a.wrapping_add(OB_TIME_FRAME), tempo);
        if tempo & 0x80 != 0 {
            wb(m, a.wrapping_add(OB_TIME_FRAME), 7);
            let quadro = rb(m, a.wrapping_add(OB_FRAME)).wrapping_add(1);
            wb(
                m,
                a.wrapping_add(OB_FRAME),
                if quadro >= 3 { 0 } else { quadro },
            );
        }
    }
    objetos::display_sprite(m, a);
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn agua_lz1_segue_caminho_baixo_e_nao_regride() {
        let mut m = Memoria::nova(vec![0; 0x10000]);
        wb(&mut m, F_WATER, 1);
        ww(&mut m, V_PLAYER.wrapping_add(OB_Y), 0x300);
        ww(&mut m, V_SCREENPOSX, 0x1080);
        ww(&mut m, WATER2, 0x318);
        agua_dinamica(&mut m);
        assert_eq!(rw(&mut m, WATER3), 0x5C8);
        assert_eq!(rw(&mut m, WATER2), 0x319);
        assert_eq!(rb(&mut m, SWITCH.wrapping_add(5)), 0x80);
        ww(&mut m, V_SCREENPOSX, 0x1380);
        ww(&mut m, WATER2, 0x3A8);
        agua_dinamica(&mut m);
        assert_eq!(rb(&mut m, WATER_ROUTINE), 1);
        ww(&mut m, V_SCREENPOSX, 0x500);
        agua_dinamica(&mut m);
        assert_eq!(rw(&mut m, WATER3), 0x3A8);
    }

    #[test]
    fn agua_lz3_modifica_cenario_e_inunda_arena() {
        let mut m = Memoria::nova(vec![0; 0x10000]);
        wb(&mut m, V_ACT, 2);
        wb(&mut m, F_WATER, 1);
        ww(&mut m, V_SCREENPOSX, 0x600);
        ww(&mut m, V_PLAYER.wrapping_add(OB_Y), 0x400);
        agua_dinamica(&mut m);
        assert_eq!(rw(&mut m, WATER2), 0x4C8);
        assert_eq!(rb(&mut m, V_LVLLAYOUT_FG.wrapping_add(0x106)), 0x4B);
        assert_eq!(rb(&mut m, WATER_ROUTINE), 1);
        wb(&mut m, WATER_ROUTINE, 3);
        ww(&mut m, V_SCREENPOSX, 0x1BC0);
        agua_dinamica(&mut m);
        assert_eq!(rb(&mut m, WATER_ROUTINE), 4);
        assert_eq!(rw(&mut m, WATER3), 0x608);
        assert_eq!(rw(&mut m, WATER2), 0x7BF);
        assert_eq!(rb(&mut m, SWITCH.wrapping_add(8)), 1);
    }

    #[test]
    fn altura_da_agua_determina_linha_e_estado() {
        let mut m = Memoria::nova(vec![0; 0x10000]);
        wb(&mut m, V_ZONE, ID_LZ);
        wb(&mut m, F_NOBGSCROLL, 1);
        ww(&mut m, WATER2, 0x120);
        ww(&mut m, V_SCREENPOSY, 0x100);
        efeitos(&mut m);
        assert_eq!(rb(&mut m, HBLANK_LINE), 32);
        assert_eq!(rb(&mut m, F_WTR_STATE), 0);
        ww(&mut m, V_SCREENPOSY, 0x140);
        efeitos(&mut m);
        assert_eq!(rb(&mut m, HBLANK_LINE), 223);
        assert_eq!(rb(&mut m, F_WTR_STATE), 1);
    }

    #[test]
    fn superficie_fixa_e_retoma_quadro_na_pausa() {
        let mut m = Memoria::nova(vec![0; 0x10000]);
        let a = V_LVLOBJSPACE;
        ww(&mut m, a.wrapping_add(OB_X), 0x60);
        ww(&mut m, WATER1, 0x120);
        wb(&mut m, V_JPADPRESS1, BTN_START);
        superficie(&mut m, a).unwrap();
        assert_eq!(rw(&mut m, a.wrapping_add(OB_Y)), 0x120);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_FRAME)), 3);
        wb(&mut m, V_JPADPRESS1, 0);
        ww(&mut m, F_PAUSE, 1);
        superficie(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_FRAME)), 3);
        ww(&mut m, F_PAUSE, 0);
        superficie(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_FRAME)), 1);
    }
}
