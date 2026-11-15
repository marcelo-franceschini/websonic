//! `console-hashes ROM QUADROS [--ppm QUADRO ARQUIVO]`: roda o console nominal (o mesmo caminho do
//! navegador) e imprime o hash da imagem de 9 bits de cada quadro. Com `--ppm`, grava a imagem
//! do quadro pedido num arquivo, para o Marcelo ver (o arquivo fica fora do repositório).
//! Serve nativo e em wasm32-wasip1 (a ROM entra por um diretório pré-aberto).
use sonic1_console::Console;

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (Some(caminho), Some(n)) = (
        args.first(),
        args.get(1).and_then(|n| n.parse::<u32>().ok()),
    ) else {
        eprintln!("uso: console-hashes ROM QUADROS [--ppm QUADRO ARQUIVO]");
        return std::process::ExitCode::from(2);
    };
    let ppm = args.iter().position(|a| a == "--ppm").and_then(|i| {
        Some((
            args.get(i + 1)?.parse::<u32>().ok()?,
            args.get(i + 2)?.clone(),
        ))
    });
    let Ok(rom) = std::fs::read(caminho) else {
        eprintln!("nao li a ROM");
        return std::process::ExitCode::from(2);
    };
    let mut c = Console::novo(rom);
    for q in 0..n {
        let rgba = c.executar_quadro();
        println!("quadro {q} {:016x}", c.hash_da_imagem());
        if q % 10 == 0 || q < 5 {
            eprintln!("diag quadro={q} {}", c.diagnostico());
        }
        if let Some((pq, arq)) = &ppm
            && *pq == q
        {
            let mut f = b"P6\n320 224\n255\n".to_vec();
            for p in rgba.chunks(4) {
                f.extend(&p[..3]);
            }
            if std::fs::write(arq, f).is_err() {
                eprintln!("nao gravei o ppm");
            }
        }
        if let Some(e) = c.erro() {
            println!("erro_do_nucleo quadro={q} {e}");
            return std::process::ExitCode::FAILURE;
        }
    }
    std::process::ExitCode::SUCCESS
}
