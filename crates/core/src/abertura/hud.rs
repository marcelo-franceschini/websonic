//! HUD (`_inc/HUD Update.asm`): os números de pontos, anéis, tempo e vidas na VRAM. O modo de
//! depuração ainda devolve `NaoImplementado`.

use super::boot::{rom_b, rom_l};
use super::ram::*;
use super::titulo::loc_vram;
use super::{ErroAbertura, rb, rl, rw, wb, wl};
use crate::memoria::Memoria;

/// Tabelas de inicialização dos dígitos (`Hud_Base_Score`, a partir dela até `Hud_Base_End`).
const HUD_BASE_SCORE: u32 = 0x1_D036;
const HUD_BASE_RINGS: u32 = 0x1_D042;
const HUD_BASE_END: u32 = 0x1_D045;
/// Soma que `addi.l #$400000,d0` faz: avança 2 tiles na palavra alta do comando.
const AVANCO_DE_DIGITO: u32 = 0x40_0000;
/// Divisores decimais (`Hud_100000` a `Hud_1`).
const CENTENAS_DE_MILHAR: [u32; 6] = [100_000, 10_000, 1000, 100, 10, 1];

/// `ext.w` de um byte para o índice do dígito, e `lsl.w #5`: o endereço da arte do dígito.
fn arte_do_digito(d0: u8) -> u32 {
    let w = u16::from_be_bytes(i16::from(i8::from_be_bytes([d0])).to_be_bytes());
    ART_HUD.wrapping_add(u32::from(w.wrapping_shl(5)))
}

/// Escreve `longas` longas da ROM em `origem` na porta de dados.
fn escrever_longas(m: &mut Memoria, origem: u32, longas: u32) {
    for k in 0..longas {
        let v = rom_l(m, origem.wrapping_add(k.wrapping_mul(4)));
        wl(m, VDP_DADOS, v);
    }
}

/// `Hud_Init_8x16Digits`: `quantos` dígitos da tabela em `a2`; negativo escreve tiles em branco.
fn hud_init_8x16_digits(m: &mut Memoria, mut a2: u32, quantos: u32) {
    for _ in 0..quantos {
        let d0 = rom_b(m, a2);
        a2 = a2.wrapping_add(1);
        if d0 & 0x80 != 0 {
            for _ in 0..16 {
                wl(m, VDP_DADOS, 0);
            }
        } else {
            escrever_longas(m, arte_do_digito(d0), 16);
        }
    }
}

/// `Hud_Base`: vidas, e os dígitos iniciais de pontos, tempo e anéis.
pub fn hud_base(m: &mut Memoria) {
    hud_lives(m);
    wl(m, VDP_CONTROLE, loc_vram(ARTTILE_HUDSCORE_E));
    hud_init_8x16_digits(m, HUD_BASE_SCORE, HUD_BASE_END.wrapping_sub(HUD_BASE_SCORE));
}

/// `Hud_ResetRings`: anéis voltam a `__0`.
fn hud_reset_rings(m: &mut Memoria) {
    wl(m, VDP_CONTROLE, loc_vram(ARTTILE_HUDRINGS));
    hud_init_8x16_digits(m, HUD_BASE_RINGS, HUD_BASE_END.wrapping_sub(HUD_BASE_RINGS));
}

/// Acha o dígito de `valor` para o divisor `d3`: quantas vezes cabe, e o resto.
fn digito(valor: u32, d3: u32) -> (u16, u32) {
    let mut d2 = 0u16;
    let mut v = valor;
    while v >= d3 {
        v = v.wrapping_sub(d3);
        d2 = d2.wrapping_add(1);
    }
    (d2, v)
}

/// `Hud_Write_8x16Digits_SkipLeading` e `_WithLeading`: `quantos` dígitos a partir do comando
/// `d0`. Sem `com_zeros`, os zeros à esquerda não se escrevem (mas o endereço anda).
fn escrever_8x16(m: &mut Memoria, mut d0: u32, valor: u32, quantos: usize, com_zeros: bool) {
    let divisores = CENTENAS_DE_MILHAR
        .get(CENTENAS_DE_MILHAR.len().saturating_sub(quantos)..)
        .unwrap_or(&[]);
    let mut v = valor;
    let mut escrevendo = com_zeros;
    for d3 in divisores {
        let (d2, resto) = digito(v, *d3);
        v = resto;
        if d2 != 0 {
            escrevendo = true;
        }
        if escrevendo {
            wl(m, VDP_CONTROLE, d0);
            let origem = ART_HUD.wrapping_add(u32::from(d2).wrapping_mul(0x40));
            escrever_longas(m, origem, 16);
        }
        d0 = d0.wrapping_add(AVANCO_DE_DIGITO);
    }
}

/// `Hud_Lives`: dois dígitos 8x8; o zero à esquerda fica em branco, menos o último.
pub fn hud_lives(m: &mut Memoria) {
    let mut d0 = loc_vram(ARTTILE_LIVES_NUM);
    let mut v = u32::from(rb(m, V_LIVES));
    let mut escrevendo = false;
    for (k, d3) in [10u32, 1].iter().enumerate() {
        wl(m, VDP_CONTROLE, d0);
        let (d2, resto) = digito(v, *d3);
        v = resto;
        if d2 != 0 {
            escrevendo = true;
        }
        let ultimo = k == 1;
        if escrevendo || ultimo {
            let origem = ART_LIVESNUMS.wrapping_add(u32::from(d2).wrapping_mul(32));
            escrever_longas(m, origem, 8);
        } else {
            for _ in 0..8 {
                wl(m, VDP_DADOS, 0);
            }
        }
        d0 = d0.wrapping_add(AVANCO_DE_DIGITO);
    }
}

/// `HUD_Update` (sem o modo de depuração): atualiza os números que as flags pedem.
pub fn hud_update(m: &mut Memoria) -> Result<(), ErroAbertura> {
    if rw(m, F_DEBUGMODE) != 0 {
        return Err(ErroAbertura::NaoImplementado("HUD do modo de depuração"));
    }
    if rb(m, F_SCORECOUNT) != 0 {
        wb(m, F_SCORECOUNT, 0);
        let pontos = rl(m, V_SCORE);
        escrever_8x16(m, loc_vram(ARTTILE_HUDSCORE), pontos, 6, false);
    }
    let anel = rb(m, F_RINGCOUNT);
    if anel != 0 {
        if anel & 0x80 != 0 {
            hud_reset_rings(m);
        }
        wb(m, F_RINGCOUNT, 0);
        let aneis = u32::from(rw(m, V_RINGS));
        escrever_8x16(m, loc_vram(ARTTILE_HUDRINGS), aneis, 3, false);
    }
    if atualizar_tempo(m) {
        return Ok(());
    }
    if rb(m, F_LIFECOUNT) != 0 {
        wb(m, F_LIFECOUNT, 0);
        hud_lives(m);
    }
    if rb(m, F_ENDACTBONUS) != 0 {
        wb(m, F_ENDACTBONUS, 0);
        wl(m, VDP_CONTROLE, loc_vram(0x570));
        let tempo = u32::from(rw(m, V_TIMEBONUS));
        escrever_bonus(m, tempo);
        let aneis = u32::from(rw(m, V_RINGBONUS));
        escrever_bonus(m, aneis);
    }
    Ok(())
}

/// A parte do tempo de `HUD_Update`: um quadro soma 1/60 s, e os números só se escrevem quando
/// os segundos mudam.
fn atualizar_tempo(m: &mut Memoria) -> bool {
    if rb(m, F_TIMECOUNT) == 0 || rw(m, F_PAUSE) != 0 {
        return false;
    }
    if rl(m, V_TIME) == (9 * 0x10000) + (59 * 0x100) + 59 {
        wb(m, F_TIMECOUNT, 0);
        super::fisica::kill_sonic(m);
        wb(m, F_TIMEOVER, 1);
        return true;
    }
    let centesimos = V_TIME.wrapping_add(3);
    let segundos = V_TIME.wrapping_add(2);
    let minutos = V_TIME.wrapping_add(1);
    let c = rb(m, centesimos).wrapping_add(1);
    wb(m, centesimos, c);
    if c < 60 {
        return false;
    }
    wb(m, centesimos, 0);
    let s = rb(m, segundos).wrapping_add(1);
    wb(m, segundos, s);
    if s >= 60 {
        wb(m, segundos, 0);
        let mi = rb(m, minutos).wrapping_add(1);
        wb(m, minutos, mi);
        if mi >= 9 {
            wb(m, minutos, 9);
        }
    }
    let mi = u32::from(rb(m, V_TIMEMIN));
    escrever_8x16(m, loc_vram(ARTTILE_HUDTIMEMINS), mi, 1, true);
    let sec = u32::from(rb(m, segundos));
    escrever_8x16(m, loc_vram(ARTTILE_HUDTIMESECS), sec, 2, true);
    false
}

/// `Hud_TimeRingBonus`: quatro dígitos contíguos, limpando zeros à esquerda.
fn escrever_bonus(m: &mut Memoria, mut valor: u32) {
    let mut escrevendo = false;
    for divisor in [1000, 100, 10, 1] {
        let (digito, resto) = digito(valor, divisor);
        valor = resto;
        escrevendo |= digito != 0;
        if escrevendo {
            escrever_longas(
                m,
                ART_HUD.wrapping_add(u32::from(digito).wrapping_mul(0x40)),
                16,
            );
        } else {
            for _ in 0..16 {
                wl(m, VDP_DADOS, 0);
            }
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::memoria::{PortaVdp, Saida};

    #[test]
    fn tempo_esgotado_mata_sonic_e_interrompe_o_hud() {
        let mut m = Memoria::nova(vec![]);
        wb(&mut m, F_TIMECOUNT, 1);
        wl(&mut m, V_TIME, 0x0009_3B3B);
        wb(&mut m, F_LIFECOUNT, 1);
        hud_update(&mut m).unwrap();
        assert_eq!(rb(&mut m, F_TIMECOUNT), 0);
        assert_eq!(rb(&mut m, F_TIMEOVER), 1);
        assert_eq!(rb(&mut m, V_PLAYER.wrapping_add(OB_ROUTINE)), 6);
        assert_eq!(rb(&mut m, F_LIFECOUNT), 1);
    }

    #[test]
    fn bonus_escreve_oito_digitos_e_limpa_flag() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        wb(&mut m, F_ENDACTBONUS, 1);
        super::super::ww(&mut m, V_TIMEBONUS, 123);
        super::super::ww(&mut m, V_RINGBONUS, 45);
        hud_update(&mut m).unwrap();
        assert_eq!(rb(&mut m, F_ENDACTBONUS), 0);
        let dados = m
            .saidas()
            .iter()
            .filter(|s| {
                matches!(
                    s,
                    Saida::Vdp {
                        porta: PortaVdp::Dados,
                        ..
                    }
                )
            })
            .count();
        assert_eq!(dados, 8 * 16 * 2);
    }
}
