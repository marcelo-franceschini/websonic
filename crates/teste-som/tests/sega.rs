//! Sub-etapa 7.3: tela da Sega com memória e ROM sintéticas (nada da ROM do jogo).
//! Fonte do comportamento: o efeito de cada rotina, descrito nos testes por fórmula direta.

use sonic1_core::abertura::ram::*;
use sonic1_core::abertura::{sega, vblank};
use sonic1_core::memoria::{Memoria, PortaVdp, Saida};

fn mem() -> Memoria {
    let mut rom = vec![0x11u8; 0x2200];
    // Pal_Sega1 e Pal_Sega2 sintéticas: bytes distintos por posição.
    for i in 0..0x30usize {
        rom[usize::try_from(PAL_SEGA1).unwrap() + i] = u8::try_from(i).unwrap();
    }
    Memoria::nova(rom)
}

#[test]
fn desvanecimento_leva_cada_cor_a_preto_em_tantos_passos_quanto_a_soma_dos_canais() {
    // Modelo direto: cada passo tira uma unidade (2, 0x20 ou 0x200) de um canal, o vermelho
    // primeiro, depois o verde, depois o azul. Uma cor 0x0EEE tem 7 + 7 + 7 unidades.
    let mut m = Memoria::nova(vec![0; 0x100]);
    m.ram_mut()[0xFB00..0xFB02].copy_from_slice(&[0x0E, 0xEE]);
    m.ram_mut()[0xFA80..0xFA82].copy_from_slice(&[0x02, 0x04]); // paleta de água: 1 + 2 unidades
    m.ram_mut()[0xF626] = 0; // início
    m.ram_mut()[0xF627] = 0x3F; // tamanho
    for passo in 1..=21 {
        sega::fade_out_to_black(&mut m);
        let cor = u16::from_be_bytes([m.ram()[0xFB00], m.ram()[0xFB01]]);
        assert_eq!(cor == 0, passo == 21, "passo {passo}");
    }
    let agua = u16::from_be_bytes([m.ram()[0xFA80], m.ram()[0xFA81]]);
    assert_eq!(agua, 0);
}

#[test]
fn ciclo_de_paleta_da_sega_termina_e_e_deterministico() {
    let rodar = || {
        let mut m = mem();
        m.ram_mut()[0xF632..0xF634].copy_from_slice(&0xFFF6u16.to_be_bytes());
        let mut n = 0u32;
        while sega::pal_cycle_sega(&mut m) {
            n += 1;
            assert!(n < 200, "o ciclo não terminou");
        }
        (n, m.ram()[0xFB00..0xFB80].to_vec())
    };
    let (n1, p1) = rodar();
    let (n2, p2) = rodar();
    assert_eq!((n1, &p1), (n2, &p2));
    assert!(n1 > 20);
}

#[test]
fn vblank_da_sega_transfere_paleta_sprites_e_rolagem_por_dma() {
    let mut m = mem();
    m.ram_mut()[0xF62A] = vblank::ROTINA_SEGA;
    m.ram_mut()[0xF614..0xF616].copy_from_slice(&5u16.to_be_bytes());
    vblank::tratar(&mut m).unwrap();
    // O temporizador desceu um e a rotina foi zerada.
    assert_eq!(u16::from_be_bytes([m.ram()[0xF614], m.ram()[0xF615]]), 4);
    assert_eq!(m.ram()[0xF62A], 0);
    // Três DMAs: o último comando de cada um é a palavra de disparo ($0080 para a CRAM, $0083
    // para os destinos em $F800 e $FC00, cujos bits altos do endereço entram no comando).
    let disparos = m
        .saidas()
        .iter()
        .filter(|s| {
            matches!(s, Saida::Vdp { porta: PortaVdp::Controle, valor, .. } if *valor == 0x0080 || *valor == 0x0083)
        })
        .count();
    assert_eq!(disparos, 3);
    // O contador de VBlank subiu.
    assert_eq!(&m.ram()[0xFE0C..0xFE10], &[0, 0, 0, 1]);
}

#[test]
fn rotina_de_vblank_desconhecida_e_erro_e_nunca_panico() {
    let mut m = mem();
    m.ram_mut()[0xF62A] = 0x3E;
    assert!(vblank::tratar(&mut m).is_err());
}
