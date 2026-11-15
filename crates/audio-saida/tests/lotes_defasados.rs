use sonic1_audio::{MCLK_POR_QUADRO_NTSC, Sintetizador};
use sonic1_audio_saida::MisturadorSaida;

#[test]
fn preserva_amostras_quando_chips_entregam_em_lotes_separados() {
    let mut sintetizador = Sintetizador::novo();
    let mut fm = Vec::new();
    let mut psg = Vec::new();
    sintetizador.avancar_ate(MCLK_POR_QUADRO_NTSC, &mut fm, &mut psg);

    let mut misturador = MisturadorSaida::novo(48_000.0);
    misturador.definir_ocupacao_externa(2400);
    misturador.alimentar_quadro(&fm, &[]);
    assert_eq!(misturador.ocupacao_buffer(), 0);
    misturador.alimentar_quadro(&[], &psg);
    let n = misturador.ocupacao_buffer();
    assert!(n >= 790);
    let mut atrasada = vec![0.0; n * 2];
    misturador.extrair_bloco_worklet(&mut atrasada);

    let mut junta = MisturadorSaida::novo(48_000.0);
    junta.definir_ocupacao_externa(2400);
    junta.alimentar_quadro(&fm, &psg);
    assert_eq!(junta.ocupacao_buffer(), n);
    let mut esperada = vec![0.0; n * 2];
    junta.extrair_bloco_worklet(&mut esperada);
    assert_eq!(atrasada, esperada);
}
