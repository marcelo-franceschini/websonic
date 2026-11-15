use sonic1_audio_saida::Reamostrador;

fn rms_depois_de_reamostrar(frequencia: f64) -> f64 {
    let entrada = 53_693_175.0 / 240.0;
    let mut reamostrador = Reamostrador::novo(entrada, 48_000.0);
    let amostras: Vec<f32> = (0..16_000)
        .map(|i| (std::f64::consts::TAU * frequencia * f64::from(i) / entrada).sin() as f32)
        .collect();
    reamostrador.alimentar_mono(&amostras);
    let mut saida = Vec::new();
    reamostrador.processar(&mut saida);
    let saida = &saida[64..];
    (saida.iter().map(|s| f64::from(s.0).powi(2)).sum::<f64>() / saida.len() as f64).sqrt()
}

#[test]
fn reduz_componentes_acima_do_nyquist_de_saida() {
    let audivel = rms_depois_de_reamostrar(5_000.0);
    let fora_da_faixa = rms_depois_de_reamostrar(80_000.0);
    assert!(audivel > 0.6, "sinal audível: {audivel}");
    assert!(fora_da_faixa < audivel * 0.05, "alias: {fora_da_faixa}");
}
