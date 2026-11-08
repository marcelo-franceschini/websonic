//! O harness do som roda inteiro numa ROM sintética. A rotina "UpdateMusic" da ROM é de brinquedo
//! (pede e devolve o barramento do Z80), então o driver do núcleo, que faz muito mais, tem de
//! DIVERGIR. O teste prova que a máquina de comparação roda sem erro e acusa a diferença.

use sonic1_diferencial_som::harness::{Config, em_thread_nova, preparar_base, rodar_id};
use std::sync::Arc;
use websonic_oracle::rom_sintetica::{RomSintetica, SSP};

fn rom() -> Vec<u8> {
    let mut rom = RomSintetica::nova(SSP, 0x200);
    // Laço principal: chama a rotina em $400 e repete.
    rom.palavras(0x200, &[0x4EB9, 0x0000, 0x0400, 0x60F8]);
    rom.palavras(
        0x400,
        &[
            0x33FC, 0x0100, 0x00A1, 0x1100, // pede o barramento
            0x33FC, 0x0000, 0x00A1, 0x1100, // devolve
            0x4E75,
        ],
    );
    rom.finalizar()
}

#[test]
fn harness_roda_e_acusa_a_diferenca() {
    let cfg = Arc::new(Config {
        rom: rom(),
        update_music: 0x400,
        quadros_base: 2,
        quadros_musica: 6,
        trechos_maximo_efeito: 6,
    });
    let base = {
        let c = Arc::clone(&cfg);
        Arc::new(em_thread_nova(move || preparar_base(&c)).unwrap())
    };
    let r = {
        let (c, b) = (Arc::clone(&cfg), Arc::clone(&base));
        em_thread_nova(move || rodar_id(&c, &b, 0x81, "teste")).unwrap()
    };
    assert_eq!(r.quadros, 6);
    assert!(
        !r.igual(),
        "o driver real não pode igualar uma rotina de brinquedo"
    );
    assert!(r.falha.is_some());
}
