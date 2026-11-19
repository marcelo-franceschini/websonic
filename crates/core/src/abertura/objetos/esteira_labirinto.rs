//! Plataformas e rodas das esteiras de Labyrinth, objeto 0x63 (REV01).
use super::super::boot::rom_w;
use super::super::fisica::speed_to_pos;
use super::super::plataforma_oscilante::{detectar_altura, mover_sonic_altura, sair};
use super::super::ram::*;
use super::super::{ErroAbertura, rb, rl, rw, wb, wl, ww};
use super::{delete_object, display_sprite, find_free_obj, fora_da_janela, remember_state};
use crate::memoria::Memoria;

const GRUPO: u32 = 0x2f;
const BASE_X: u32 = 0x30;
const DEST_X: u32 = 0x34;
const DEST_Y: u32 = 0x36;
const INDICE: u32 = 0x38;
const TOTAL: u32 = 0x39;
const PASSO: u32 = 0x3a;
const REVERSO: u32 = 0x3b;
const TABELA: u32 = 0x3c;
const F_REVERSO: u32 = 0xFF_F7C0;
const F_GRUPO: u32 = 0xFF_F7C1;
const F_BOTAO: u32 = 0xFF_F7EE;
const OBJ_INDEX: u32 = 0x6b000;
const OBJ_LZ_INDEX: u32 = 0x6b070;
const DADOS: u32 = 0x12cbc;

pub(super) fn executar(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let mut rotina = rb(m, a.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        let subtipo = rb(m, a.wrapping_add(OB_SUBTYPE));
        if subtipo & 0x80 != 0 {
            gerar(m, a, subtipo);
            return Ok(());
        }
        wb(m, a.wrapping_add(OB_ROUTINE), 2);
        wl(m, a.wrapping_add(OB_MAP), 0x12d50);
        ww(m, a.wrapping_add(OB_GFX), 0x43f6);
        let render = rb(m, a.wrapping_add(OB_RENDER)) | 4;
        wb(m, a.wrapping_add(OB_RENDER), render);
        wb(m, a.wrapping_add(OB_ACTWID), 16);
        wb(m, a.wrapping_add(OB_PRIORITY), 4);
        if subtipo == 0x7f {
            wb(m, a.wrapping_add(OB_ROUTINE), 6);
            ww(m, a.wrapping_add(OB_GFX), 0x03f6);
            wb(m, a.wrapping_add(OB_PRIORITY), 1);
            roda(m, a);
            return Ok(());
        }
        iniciar_plataforma(m, a, subtipo);
        rotina = 2;
    }
    match rotina {
        2 => {
            detectar_altura(m, a, 8);
            atualizar(m, a);
        }
        4 => {
            sair(m, a);
            let x = rw(m, a.wrapping_add(OB_X));
            atualizar(m, a);
            mover_sonic_altura(m, a, x, 9);
        }
        6 => {
            roda(m, a);
            return Ok(());
        }
        _ => {
            return Err(ErroAbertura::NaoImplementado(
                "rotina da esteira de Labyrinth",
            ));
        }
    }
    let base = rw(m, a.wrapping_add(BASE_X));
    let diferenca = (base & 0xff80).wrapping_sub(rw(m, V_SCREENPOSX).wrapping_sub(128) & 0xff80);
    if fora_da_janela(m, base) && !(rb(m, V_ACT) == 2 && diferenca >= 0xff80) {
        let grupo = rb(m, a.wrapping_add(GRUPO));
        if grupo & 0x80 != 0 {
            let endereco = F_GRUPO.wrapping_add(u32::from(grupo & 0x7f));
            let flags = rb(m, endereco) & !1;
            wb(m, endereco, flags);
        }
        delete_object(m, a);
    } else {
        display_sprite(m, a);
    }
    Ok(())
}

fn gerar(m: &mut Memoria, a: u32, subtipo: u8) {
    wb(m, a.wrapping_add(GRUPO), subtipo);
    let grupo = subtipo & 0x7f;
    let flag = F_GRUPO.wrapping_add(u32::from(grupo));
    let estado = rb(m, flag);
    wb(m, flag, estado | 1);
    if estado & 1 != 0 {
        delete_object(m, a);
        return;
    }
    let tabela = OBJ_LZ_INDEX.wrapping_add(u32::from(grupo.wrapping_mul(2) & 0x1e));
    let desloc = i16::from_be_bytes(rom_w(m, tabela).to_be_bytes());
    let mut endereco = OBJ_INDEX.wrapping_add_signed(i32::from(desloc));
    let numero = rom_w(m, endereco);
    endereco = endereco.wrapping_add(2);
    for i in 0..=numero {
        let destino = if i == 0 { Some(a) } else { find_free_obj(m) };
        if let Some(destino) = destino {
            wb(m, destino.wrapping_add(OB_ID), 0x63);
            ww(m, destino.wrapping_add(OB_X), rom_w(m, endereco));
            ww(
                m,
                destino.wrapping_add(OB_Y),
                rom_w(m, endereco.wrapping_add(2)),
            );
            wb(
                m,
                destino.wrapping_add(OB_SUBTYPE),
                rom_w(m, endereco.wrapping_add(4)).to_be_bytes()[1],
            );
            endereco = endereco.wrapping_add(6);
        }
    }
}

fn iniciar_plataforma(m: &mut Memoria, a: u32, subtipo: u8) {
    wb(m, a.wrapping_add(OB_FRAME), 4);
    let indice = u32::from((subtipo >> 4) & 15).wrapping_mul(2);
    let desloc = i16::from_be_bytes(rom_w(m, DADOS.wrapping_add(indice)).to_be_bytes());
    let tabela = DADOS.wrapping_add_signed(i32::from(desloc));
    let total = rom_w(m, tabela).to_be_bytes()[1];
    wb(m, a.wrapping_add(TOTAL), total);
    ww(m, a.wrapping_add(BASE_X), rom_w(m, tabela.wrapping_add(2)));
    wl(m, a.wrapping_add(TABELA), tabela.wrapping_add(4));
    let mut alvo = (subtipo & 15).wrapping_mul(4);
    let reverso = rb(m, F_REVERSO) != 0;
    wb(m, a.wrapping_add(PASSO), if reverso { 0xfc } else { 4 });
    if reverso {
        wb(m, a.wrapping_add(REVERSO), 1);
        alvo = proximo(alvo, 0xfc, total);
    }
    definir_alvo(m, a, alvo);
    mudar_direcao(m, a);
}

fn proximo(atual: u8, passo: u8, total: u8) -> u8 {
    let novo = atual.wrapping_add(passo);
    if novo < total {
        novo
    } else if novo & 0x80 != 0 {
        total.wrapping_sub(4)
    } else {
        0
    }
}
fn definir_alvo(m: &mut Memoria, a: u32, indice: u8) {
    wb(m, a.wrapping_add(INDICE), indice);
    let tabela = rl(m, a.wrapping_add(TABELA)).wrapping_add(u32::from(indice));
    ww(m, a.wrapping_add(DEST_X), rom_w(m, tabela));
    ww(m, a.wrapping_add(DEST_Y), rom_w(m, tabela.wrapping_add(2)));
}
fn atualizar(m: &mut Memoria, a: u32) {
    let reverter = rb(m, F_BOTAO) != 0 && rb(m, a.wrapping_add(REVERSO)) == 0;
    if reverter {
        wb(m, a.wrapping_add(REVERSO), 1);
        wb(m, F_REVERSO, 1);
        let passo = rb(m, a.wrapping_add(PASSO)).wrapping_neg();
        wb(m, a.wrapping_add(PASSO), passo);
    }
    let no_alvo = rw(m, a.wrapping_add(OB_X)) == rw(m, a.wrapping_add(DEST_X))
        && rw(m, a.wrapping_add(OB_Y)) == rw(m, a.wrapping_add(DEST_Y));
    if reverter || no_alvo {
        let indice = proximo(
            rb(m, a.wrapping_add(INDICE)),
            rb(m, a.wrapping_add(PASSO)),
            rb(m, a.wrapping_add(TOTAL)),
        );
        definir_alvo(m, a, indice);
        mudar_direcao(m, a);
    }
    speed_to_pos(m, a);
}
fn mudar_direcao(m: &mut Memoria, a: u32) {
    let x = rw(m, a.wrapping_add(OB_X));
    let y = rw(m, a.wrapping_add(OB_Y));
    let tx = rw(m, a.wrapping_add(DEST_X));
    let ty = rw(m, a.wrapping_add(DEST_Y));
    let dx = i16::from_be_bytes(x.wrapping_sub(tx).to_be_bytes());
    let dy = i16::from_be_bytes(y.wrapping_sub(ty).to_be_bytes());
    let adx = dx.unsigned_abs();
    let ady = dy.unsigned_abs();
    if ady >= adx {
        let (vel, resto) = divisao(dx, ady);
        ww(m, a.wrapping_add(OB_VELX), vel);
        ww(
            m,
            a.wrapping_add(OB_VELY),
            if y < ty { 0x100 } else { 0xff00 },
        );
        ww(m, a.wrapping_add(OB_X).wrapping_add(2), resto);
        ww(m, a.wrapping_add(OB_Y).wrapping_add(2), 0);
    } else {
        let (vel, resto) = divisao(dy, adx);
        ww(m, a.wrapping_add(OB_VELY), vel);
        ww(
            m,
            a.wrapping_add(OB_VELX),
            if x < tx { 0x100 } else { 0xff00 },
        );
        ww(m, a.wrapping_add(OB_Y).wrapping_add(2), resto);
        ww(m, a.wrapping_add(OB_X).wrapping_add(2), 0);
    }
}
fn divisao(delta: i16, divisor: u16) -> (u16, u16) {
    if delta == 0 || divisor == 0 {
        return (0, 0);
    }
    let numerador = i32::from(delta).wrapping_shl(8);
    let divisor = i32::from(i16::from_be_bytes(divisor.to_be_bytes()));
    if divisor == 0 {
        return (0, 0);
    }
    let quociente = numerador.checked_div(divisor).unwrap_or(0);
    let resto = numerador.checked_rem(divisor).unwrap_or(0);
    let q = quociente.wrapping_neg().to_be_bytes();
    let r = resto.to_be_bytes();
    (
        u16::from_be_bytes([q[2], q[3]]),
        u16::from_be_bytes([r[2], r[3]]),
    )
}
fn roda(m: &mut Memoria, a: u32) {
    if rw(m, V_FRAMECOUNT) & 3 == 0 {
        let quadro = rb(m, a.wrapping_add(OB_FRAME));
        let quadro = if rb(m, F_REVERSO) == 0 {
            quadro.wrapping_add(1)
        } else {
            quadro.wrapping_sub(1)
        };
        wb(m, a.wrapping_add(OB_FRAME), quadro & 3);
    }
    remember_state(m, a);
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn roda_gira_na_direcao_da_esteira() {
        let mut m = Memoria::nova(vec![0; 0x70000]);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x63);
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 0x7f);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_FRAME)), 1);
        wb(&mut m, F_REVERSO, 1);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a.wrapping_add(OB_FRAME)), 0);
    }
    #[test]
    fn gerador_le_posicoes_e_eviti_duplicacao_do_grupo() {
        let mut rom = vec![0; 0x70000];
        rom[0x6b070..0x6b072].copy_from_slice(&0x100u16.to_be_bytes());
        rom[0x6b100..0x6b108].copy_from_slice(&[0, 0, 2, 0, 3, 0, 0, 1]);
        let mut m = Memoria::nova(rom);
        let a = V_LVLOBJSPACE;
        wb(&mut m, a.wrapping_add(OB_ID), 0x63);
        wb(&mut m, a.wrapping_add(OB_SUBTYPE), 0x80);
        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, F_GRUPO) & 1, 1);
        assert_eq!(rw(&mut m, a.wrapping_add(OB_X)), 0x200);
        assert_eq!(rb(&mut m, a.wrapping_add(OB_SUBTYPE)), 1);
        let outro = a.wrapping_add(OBJECT_SIZE);
        wb(&mut m, outro.wrapping_add(OB_ID), 0x63);
        wb(&mut m, outro.wrapping_add(OB_SUBTYPE), 0x80);
        executar(&mut m, outro).unwrap();
        assert_eq!(rb(&mut m, outro.wrapping_add(OB_ID)), 0);
    }
    #[test]
    fn velocidade_diagonal_preserva_razao_e_resto() {
        let mut m = Memoria::nova(vec![0; 0x70000]);
        let a = V_LVLOBJSPACE;
        ww(&mut m, a.wrapping_add(OB_X), 100);
        ww(&mut m, a.wrapping_add(OB_Y), 200);
        ww(&mut m, a.wrapping_add(DEST_X), 105);
        ww(&mut m, a.wrapping_add(DEST_Y), 210);
        mudar_direcao(&mut m, a);
        assert_eq!(rw(&mut m, a.wrapping_add(OB_VELX)), 0x80);
        assert_eq!(rw(&mut m, a.wrapping_add(OB_VELY)), 0x100);
    }
}
