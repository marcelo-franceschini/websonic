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
#[test]
fn taxa_do_dispositivo_controla_quantidade_de_amostras() {
    let contar = |taxa| {
        let mut console = sonic1_console::Console::com_taxa_audio(vec![0; 0x4000], taxa);
        (0..12)
            .map(|_| {
                console.executar_quadro();
                console.extrair_audio().len()
            })
            .sum::<usize>()
    };
    let a = contar(44_100.0);
    let b = contar(48_000.0);
    assert!(a > 0 && b > a);
    assert!((a as f64 / b as f64 - 44_100.0 / 48_000.0).abs() < 0.02);
}

#[test]
fn fila_cheia_reduz_a_quantidade_de_audio_produzido() {
    let contar = |ocupacao| {
        let mut console = Console::com_taxa_audio(vec![0; 0x4000], 48_000.0);
        console.definir_ocupacao_audio(ocupacao);
        (0..30)
            .map(|_| {
                console.executar_quadro();
                console.extrair_audio().len()
            })
            .sum::<usize>()
    };
    assert!(contar(0) > contar(16_000));
}
