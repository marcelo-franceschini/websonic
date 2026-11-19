//! Teste de integridade visual do Special Stage (Etapa 11).
use sonic1_console::Console;
use std::fs;

#[test]
#[ignore = "requer SONIC1_ROM"]
fn inspecionar_special_stage_quadro() {
    let caminho = std::env::var_os("SONIC1_ROM").expect("SONIC1_ROM");
    let rom = fs::read(caminho).expect("ler ROM");
    let mut c = Console::novo(rom);
    c.forcar_special_stage().expect("forcar special stage");
    for i in 0..120 {
        let rgba = c.executar_quadro();
        assert!(c.ab.erro.is_none(), "erro na abertura no quadro {i}");
        if i == 119 {
            assert_eq!(c.ab.modo_nome(), "Especial(Jogando)");
            let sprites = c.ab.mem.ram()[0xF62C_usize];
            assert!(
                sprites >= 20,
                "esperado pelo menos 20 sprites no Special Stage, obtido {sprites}"
            );
            let nao_zeros = c.imagem_9bits().iter().filter(|&&p| p != 0).count();
            assert_eq!(
                nao_zeros, 71650,
                "esperado 71650 pontos coloridos e 30 pontos pretos"
            );
            assert_eq!(c.imagem_9bits().len(), 71680, "total de 320x224 pontos");
            assert_eq!(rgba.len(), 320 * 224 * 4);
        }
    }
}
