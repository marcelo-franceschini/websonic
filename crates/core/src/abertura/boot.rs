//! Inicialização do console (sub-etapas 7.1 e 7.2). Cada função cita o rótulo que reproduz.
//!
//! As tabelas (valores dos registradores da VDP, programa de partida do Z80, volumes do PSG,
//! palavras da soma de verificação) são lidas da ROM do jogador nos endereços dos rótulos.
//! Nenhum dado da ROM está neste arquivo.

use super::ram::*;
use super::{ErroAbertura, rb, rw, wb, wl, ww};
use crate::memoria::{Memoria, Tamanho};
use crate::rotinas;

/// Voltas máximas de um laço que espera um valor gravado, antes de desistir.
const LIMITE_ESPERA: u32 = 1_000_000;

fn rom_w(m: &Memoria, a: u32) -> u16 {
    let i = usize::try_from(a).unwrap_or(usize::MAX);
    match m.rom().get(i..i.saturating_add(2)) {
        Some([a, b]) => u16::from_be_bytes([*a, *b]),
        _ => 0,
    }
}

fn rom_l(m: &Memoria, a: u32) -> u32 {
    (u32::from(rom_w(m, a)) << 16) | u32::from(rom_w(m, a.wrapping_add(2)))
}

fn rom_b(m: &Memoria, a: u32) -> u8 {
    let i = usize::try_from(a).unwrap_or(usize::MAX);
    m.rom().get(i).copied().unwrap_or(0)
}

/// `waitZ80`: repete a leitura do BUSACK até o Z80 ceder o barramento.
fn esperar_z80(m: &mut Memoria) -> Result<(), ErroAbertura> {
    for _ in 0..LIMITE_ESPERA {
        if rb(m, Z80_BUSREQ) & 1 == 0 {
            return Ok(());
        }
    }
    Err(ErroAbertura::EsperaSemFim)
}

/// `EntryPoint`: preparação do console na partida (VDP, Z80, RAM, CRAM, VSRAM e PSG).
pub fn entry_point(m: &mut Memoria) -> Result<(), ErroAbertura> {
    // Porta já inicializada (religou sem apagar): o original pula a preparação.
    let a = m.ler(PORTA1_CONTROLE_ALTO, Tamanho::Long)?;
    let ja = if a != 0 {
        true
    } else {
        m.ler(EXPANSAO_CONTROLE_ALTO, Tamanho::Word)? != 0
    };
    if ja {
        return Ok(());
    }
    let inicio = rom_w(m, SETUP_VALUES);
    let passo = rom_w(m, SETUP_VALUES.wrapping_add(4));
    // Sem TMSS no console de referência (versão com nibble baixo zero): a escrita é pulada.
    if rb(m, VERSAO) & 0xF != 0 {
        wl(m, 0xA1_4000, 0x5345_4741);
    }
    let _ = rw(m, VDP_CONTROLE); // limpa o pedido de escrita pendente da VDP
    let mut d5 = inicio;
    for i in 0..SETUP_VALUES_VDP_END.wrapping_sub(SETUP_VALUES_VDP) {
        d5 = (d5 & 0xFF00) | u16::from(rom_b(m, SETUP_VALUES_VDP.wrapping_add(i)));
        ww(m, VDP_CONTROLE, d5);
        d5 = d5.wrapping_add(passo);
    }
    // Destino do preenchimento de DMA e o valor 0: o preenchimento limpa a VRAM.
    wl(m, VDP_CONTROLE, rom_l(m, SETUP_VALUES_VDP_END));
    ww(m, VDP_DADOS, 0);
    ww(m, Z80_BUSREQ, passo);
    ww(m, Z80_RESET, passo);
    esperar_z80(m)?;
    for i in 0..SETUP_VALUES_Z80_END.wrapping_sub(SETUP_VALUES_Z80) {
        wb(
            m,
            Z80_RAM.wrapping_add(i),
            rom_b(m, SETUP_VALUES_Z80.wrapping_add(i)),
        );
    }
    ww(m, Z80_RESET, 0);
    ww(m, Z80_BUSREQ, 0);
    ww(m, Z80_RESET, passo);
    // Limpa a RAM inteira do 68000.
    m.ram_mut().fill(0);
    let mut p = SETUP_VALUES_Z80_END;
    wl(m, VDP_CONTROLE, rom_l(m, p)); // modo da VDP e incremento
    p = p.wrapping_add(4);
    wl(m, VDP_CONTROLE, rom_l(m, p)); // escrita na CRAM
    p = p.wrapping_add(4);
    for _ in 0..32 {
        wl(m, VDP_DADOS, 0);
    }
    wl(m, VDP_CONTROLE, rom_l(m, p)); // escrita na VSRAM
    for _ in 0..20 {
        wl(m, VDP_DADOS, 0);
    }
    for i in 0..SETUP_VALUES_PSG_END.wrapping_sub(SETUP_VALUES_PSG) {
        wb(m, PSG, rom_b(m, SETUP_VALUES_PSG.wrapping_add(i)));
    }
    ww(m, Z80_RESET, 0);
    Ok(())
}

/// `GameProgram` e `CheckSumCheck`: confere a soma das palavras da ROM depois do cabeçalho.
/// O laço soma as palavras de `$200` até o último endereço da ROM do cabeçalho, em 16 bits.
pub fn game_program(m: &mut Memoria) -> Result<(), ErroAbertura> {
    let _ = rw(m, VDP_CONTROLE);
    let porta_c = rb(m, PORTA3_CONTROLE);
    if porta_c & 0x40 != 0 {
        // Religar sem apagar o console é o botão reset: fora do escopo (exclusão aprovada).
        return Err(ErroAbertura::NaoImplementado(
            "GameProgram: porta C ja inicializada",
        ));
    }
    let fim = rom_l(m, ROM_END_LOC);
    let mut a0 = END_OF_HEADER;
    let mut soma: u16 = 0;
    loop {
        soma = soma.wrapping_add(rom_w(m, a0));
        a0 = a0.wrapping_add(2);
        if fim < a0 {
            break;
        }
    }
    if soma != rom_w(m, CHECKSUM) {
        return Err(ErroAbertura::ChecksumErrado);
    }
    Ok(())
}

/// `CheckSumOk`: limpa a RAM que sobrevive ao reset, guarda a região e marca `init`.
pub fn check_sum_ok(m: &mut Memoria) -> Result<(), ErroAbertura> {
    let ini = usize::try_from(V_CROSSRESETRAM & 0xFFFF).unwrap_or(0);
    if let Some(r) = m.ram_mut().get_mut(ini..) {
        r.fill(0);
    }
    let v = rb(m, VERSAO) & 0xC0;
    wb(m, V_MEGADRIVE, v);
    wl(m, V_INIT, 0x696E_6974); // 'init'
    Ok(())
}

/// `GameInit` (primeira parte): limpa a RAM que o reset não preserva. As partes seguintes
/// (`VDPSetupGame`, `DACDriverLoad`, `JoypadInit`) são chamadas uma a uma pelo laço de quadro,
/// cada uma na marca do rótulo da rotina seguinte.
pub fn game_init_limpar(m: &mut Memoria) -> Result<(), ErroAbertura> {
    let fim = usize::try_from(V_CROSSRESETRAM & 0xFFFF).unwrap_or(0);
    if let Some(r) = m.ram_mut().get_mut(..fim) {
        r.fill(0);
    }
    Ok(())
}

/// `GameInit` (última parte): escolhe o modo Sega e entra em `MainGameLoop`.
pub fn game_init_fim(m: &mut Memoria) -> Result<(), ErroAbertura> {
    joypad_init(m)?;
    wb(m, V_GAMEMODE, 0); // id_Sega
    Ok(())
}

/// `fillVRAM` (macro do disassembly): preenchimento de VRAM por DMA e espera do fim.
pub fn fill_vram(m: &mut Memoria, byte: u8, inicio: u32, fim: u32) -> Result<(), ErroAbertura> {
    ww(m, VDP_CONTROLE, 0x8F01);
    let n = fim.wrapping_sub(inicio).wrapping_sub(1);
    wl(
        m,
        VDP_CONTROLE,
        0x9400_9300 | ((n & 0xFF00) << 8) | (n & 0xFF),
    );
    ww(m, VDP_CONTROLE, 0x9780);
    wl(
        m,
        VDP_CONTROLE,
        0x4000_0080u32
            .wrapping_add((inicio & 0x3FFF) << 16)
            .wrapping_add((inicio & 0xC000) >> 14),
    );
    ww(m, VDP_DADOS, u16::from_be_bytes([byte, byte]));
    let mut pronto = false;
    for _ in 0..LIMITE_ESPERA {
        if rw(m, VDP_CONTROLE) & 2 == 0 {
            pronto = true;
            break;
        }
    }
    if !pronto {
        return Err(ErroAbertura::EsperaSemFim);
    }
    ww(m, VDP_CONTROLE, 0x8F02);
    Ok(())
}

/// `VDPSetupGame`: registradores da VDP do jogo, CRAM e VRAM limpas.
pub fn vdp_setup_game(m: &mut Memoria) -> Result<(), ErroAbertura> {
    let n = VDP_SETUP_ARRAY_END.wrapping_sub(VDP_SETUP_ARRAY) / 2;
    for i in 0..n {
        let w = rom_w(m, VDP_SETUP_ARRAY.wrapping_add(i.wrapping_mul(2)));
        ww(m, VDP_CONTROLE, w);
    }
    let r1 = rom_w(m, VDP_SETUP_ARRAY.wrapping_add(2));
    ww(m, V_VDP_BUFFER1, r1);
    ww(m, V_HBLANK_HREG, 0x8A00 | 223);
    wl(m, VDP_CONTROLE, 0xC000_0000);
    for _ in 0..0x40 {
        ww(m, VDP_DADOS, 0);
    }
    wl(m, V_SCRPOSY_VDP, 0);
    wl(m, V_SCRPOSX_VDP, 0);
    fill_vram(m, 0, 0, 0x1_0000)
}

/// `DACDriverLoad`: descomprime o programa do Z80 na RAM dele.
pub fn dac_driver_load(m: &mut Memoria) -> Result<(), ErroAbertura> {
    ww(m, Z80_BUSREQ, 0x100);
    ww(m, Z80_RESET, 0x100);
    rotinas::kos_dec(m, DAC_DRIVER, Z80_RAM).map_err(ErroAbertura::Rotina)?;
    ww(m, Z80_RESET, 0);
    ww(m, Z80_RESET, 0x100);
    ww(m, Z80_BUSREQ, 0);
    Ok(())
}

/// `JoypadInit`: põe as três portas de controle em modo de leitura por TH.
pub fn joypad_init(m: &mut Memoria) -> Result<(), ErroAbertura> {
    ww(m, Z80_BUSREQ, 0x100);
    esperar_z80(m)?;
    wb(m, PORTA1_CONTROLE, 0x40);
    wb(m, PORTA2_CONTROLE, 0x40);
    wb(m, PORTA3_CONTROLE, 0x40);
    ww(m, Z80_BUSREQ, 0);
    Ok(())
}
