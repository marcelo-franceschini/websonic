use sonic1_console::Console;

#[test]
fn console_executa_quadros_e_extrai_audio() {
    let mut c = Console::novo(vec![0u8; 0x4000]);
    let mut total_amostras = 0;
    let mut quadros_com_audio = 0;
    for _ in 0..70 {
        let _img = c.executar_quadro();
        let audio = c.extrair_audio();
        if !audio.is_empty() {
            quadros_com_audio += 1;
            total_amostras += audio.len();
        }
    }
    println!("quadros com audio: {quadros_com_audio}, total amostras: {total_amostras}");
    assert!(quadros_com_audio > 0, "Deveria ter extraido audio");
    assert!(total_amostras > 0, "Deveria ter amostras de audio");
}

#[test]
fn console_reage_a_escrita_de_som() {
    let mut c = Console::novo(vec![0u8; 0x4000]);
    // Agenda um tom no PSG: canal 0 frequência baixa, volume máximo (0x90).
    c.ab.mem
        .escrever(0xC00011, sonic1_core::memoria::Tamanho::Byte, 0x85)
        .unwrap();
    c.ab.mem
        .escrever(0xC00011, sonic1_core::memoria::Tamanho::Byte, 0x01)
        .unwrap();
    c.ab.mem
        .escrever(0xC00011, sonic1_core::memoria::Tamanho::Byte, 0x90)
        .unwrap();

    let _ = c.executar_quadro();
    let audio1 = c.extrair_audio();
    let nao_zeros1 = audio1.iter().filter(|&&x| x.abs() > 0.001).count();

    // Segundo quadro (o tom deve continuar tocando):
    let _ = c.executar_quadro();
    let audio2 = c.extrair_audio();
    let nao_zeros2 = audio2.iter().filter(|&&x| x.abs() > 0.001).count();

    println!("quadro 1: {nao_zeros1} nao-zeros; quadro 2: {nao_zeros2} nao-zeros");
    assert!(nao_zeros1 > 0, "Quadro 1 deveria ter tom");
    assert!(nao_zeros2 > 0, "Quadro 2 deveria continuar tocando o tom");
}
