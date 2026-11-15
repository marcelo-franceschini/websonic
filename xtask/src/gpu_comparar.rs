//! Compara no navegador a composição GPU com o renderizador CPU em quadros iguais.
//! A ROM é servida apenas em loopback durante este processo e não é gravada no site.
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

const HTML: &str = "<!doctype html><meta charset=utf-8><pre id=resultado>AGUARDANDO</pre><script type=module src=/teste.js></script><img hidden src=/esperar>";

struct Arquivos {
    rom: Vec<u8>,
    js: Vec<u8>,
    wasm: Vec<u8>,
    gpu: Vec<u8>,
    finalizado: AtomicBool,
}

fn responder(mut conexao: TcpStream, arquivos: &Arquivos) {
    let _ = conexao.set_read_timeout(Some(Duration::from_secs(2)));
    let mut requisicao = Vec::with_capacity(4096);
    let mut bloco = [0u8; 1024];
    loop {
        let Ok(n) = conexao.read(&mut bloco) else {
            return;
        };
        if n == 0 {
            return;
        }
        requisicao.extend_from_slice(&bloco[..n]);
        if requisicao.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
        if requisicao.len() >= 16_384 {
            return;
        }
    }
    let linha = String::from_utf8_lossy(&requisicao);
    let caminho = linha
        .split_whitespace()
        .nth(1)
        .unwrap_or("")
        .split('?')
        .next()
        .unwrap_or("");
    let (status, tipo, dados): (&str, &str, &[u8]) = match caminho {
        "/" => ("200 OK", "text/html; charset=utf-8", HTML.as_bytes()),
        "/teste.js" => ("200 OK", "text/javascript", include_bytes!("gpu_teste.js")),
        "/gpu.js" => ("200 OK", "text/javascript", &arquivos.gpu),
        "/pkg/sonic1_web.js" => ("200 OK", "text/javascript", &arquivos.js),
        "/pkg/sonic1_web_bg.wasm" => ("200 OK", "application/wasm", &arquivos.wasm),
        "/rom" => ("200 OK", "application/octet-stream", &arquivos.rom),
        "/terminou" => {
            arquivos.finalizado.store(true, Ordering::Release);
            ("200 OK", "text/plain", b"ok")
        }
        "/esperar" => {
            let inicio = std::time::Instant::now();
            while !arquivos.finalizado.load(Ordering::Acquire)
                && inicio.elapsed() < Duration::from_secs(120)
            {
                std::thread::sleep(Duration::from_millis(20));
            }
            (
                "200 OK",
                "image/svg+xml",
                b"<svg xmlns='http://www.w3.org/2000/svg'/>",
            )
        }
        "/favicon.ico" => ("204 No Content", "text/plain", b""),
        _ => {
            eprintln!("rota de teste não reconhecida: {caminho}");
            ("404 Not Found", "text/plain", b"nao encontrado")
        }
    };
    let cabecalho = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {tipo}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        dados.len()
    );
    let _ = conexao.write_all(cabecalho.as_bytes());
    let _ = conexao.write_all(dados);
}

pub fn executar(args: &[String]) -> Result<(), String> {
    let caminho = args
        .first()
        .ok_or("uso: cargo xtask gpu-comparar ROM [QUADROS] [INTERVALO]")?;
    let quadros: u32 = args
        .get(1)
        .map_or(Ok(1000), |v| v.parse())
        .map_err(|_| "quadros inválidos")?;
    let intervalo: u32 = args
        .get(2)
        .map_or(Ok(60), |v| v.parse())
        .map_err(|_| "intervalo inválido")?;
    if quadros == 0 || quadros > 100_000 || intervalo == 0 {
        return Err("quadros ou intervalo fora da faixa".into());
    }
    let rom = std::fs::read(caminho).map_err(|e| format!("não li a ROM: {e}"))?;
    if sonic1_rom::identificar(&rom).veredito != sonic1_rom::Veredito::Rev01 {
        return Err("ROM não é a REV01".into());
    }
    let pkg = std::env::var_os("WEBSONIC_GPU_PKG")
        .map(PathBuf::from)
        .unwrap_or_else(|| crate::raiz().join("dist/pkg"));
    let ler = |nome: &str| {
        std::fs::read(pkg.join(nome)).map_err(|e| {
            format!(
                "não li {}: {e}; monte o site ou defina WEBSONIC_GPU_PKG",
                pkg.join(nome).display()
            )
        })
    };
    let arquivos = Arquivos {
        rom,
        js: ler("sonic1_web.js")?,
        wasm: ler("sonic1_web_bg.wasm")?,
        gpu: std::fs::read(crate::raiz().join("site/gpu.js"))
            .map_err(|e| format!("não li gpu.js: {e}"))?,
        finalizado: AtomicBool::new(false),
    };
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| format!("servidor local: {e}"))?;
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let porta = listener.local_addr().map_err(|e| e.to_string())?.port();
    let ativo = Arc::new(AtomicBool::new(true));
    let ativo_servidor = Arc::clone(&ativo);
    let arquivos = Arc::new(arquivos);
    let servidor = std::thread::spawn(move || {
        while ativo_servidor.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((conexao, _)) => {
                    let arquivos = Arc::clone(&arquivos);
                    std::thread::spawn(move || responder(conexao, &arquivos));
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(5))
                }
                Err(_) => break,
            }
        }
    });
    let navegador =
        std::env::var_os("WEBSONIC_NAVEGADOR").unwrap_or_else(|| "google-chrome-stable".into());
    let perfil = std::env::temp_dir().join(format!("websonic-gpu-{}", std::process::id()));
    let url = format!("http://127.0.0.1:{porta}/?quadros={quadros}&passo={intervalo}");
    let mut processo = Command::new(navegador);
    processo.args([
        "--headless",
        "--disable-dev-shm-usage",
        "--no-first-run",
        "--disable-background-networking",
        "--timeout=120000",
        "--dump-dom",
    ]);
    processo.arg(format!("--user-data-dir={}", perfil.display()));
    if std::env::var_os("WEBSONIC_GPU_SOFTWARE").is_some() {
        processo.args([
            "--use-gl=angle",
            "--use-angle=swiftshader",
            "--enable-unsafe-swiftshader",
        ]);
    } else {
        processo.args(["--use-gl=angle", "--use-angle=gl", "--enable-gpu"]);
    }
    let saida = processo.arg(url).output();
    ativo.store(false, Ordering::Relaxed);
    let _ = servidor.join();
    let saida = saida.map_err(|e| format!("não consegui abrir o navegador: {e}"))?;
    if !saida.status.success() {
        return Err(format!(
            "navegador falhou: {}",
            String::from_utf8_lossy(&saida.stderr)
        ));
    }
    let html = String::from_utf8_lossy(&saida.stdout);
    let resultado = html
        .split("<pre id=\"resultado\">")
        .nth(1)
        .and_then(|v| v.split("</pre>").next())
        .ok_or_else(|| {
            format!(
                "navegador não retornou resultado: {}",
                html.chars().take(500).collect::<String>()
            )
        })?;
    println!("{resultado}");
    if resultado.starts_with("GPU_TESTE_PASSOU ") {
        Ok(())
    } else {
        Err("comparação GPU não passou".into())
    }
}
