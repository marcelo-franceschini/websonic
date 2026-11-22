# Base de Conhecimento

Índice dos documentos de conhecimento técnico acumulado no projeto. Cada entrada na base deve conter evidência sustentada por teste, trace ou commit. Não são permitidos códigos ou comentários copiados do disassembly, dados da ROM ou traces brutos.

## Índice

- [ambiente.md](ambiente.md): provas do contêiner, da rede e das permissões, com comando e resultado.
- [ci.md](ci.md): workflow do GitHub Actions, publicação no Pages, segurança e o que só o GitHub prova.
- [corpus.md](corpus.md): formato `.s1in`, GMV, traces do corpus, determinismo e retomada, mapa de acessos a RAM, gravador e `fidelidade`.
- [descompressao.md](descompressao.md): formatos Nemesis, Enigma e Kosinski, fontes, código, manifesto e como o oráculo mede a saída.
- [esqueleto.md](esqueleto.md): workspace Rust, lints do núcleo, crate `rom`, site e `cargo xtask`, com evidência.
- [hooks.md](hooks.md): hooks do git, regras de bloqueio da ROM, ativação, limites e testes.
- [video.md](video.md): o que o Sonic 1 usa da VDP, regra de renderizacao e de tempo do oraculo, provas e desempenho do renderizador.
- [oraculos.md](oraculos.md): segundo oraculo (Genesis Plus GX) e as diferencas D1a, D1b e D1c, com a decisao.
- [oraculo.md](oraculo.md): oráculo (jgenesis modificado), formato do trace e dos eventos, mapa de rótulos, com evidência.
- [nucleo.md](nucleo.md): memoria espelhada, operacoes do 68000, tempo gravado, pontos de checagem, teste por rotina, cobertura, bugs injetados e bugs conhecidos (Etapa 5), com evidencia.
- [recomp.md](recomp.md): referências de recompilação, limites de reaproveitamento e decisões para a geração de Rust.
- [som.md](som.md): pontos do Z80 no tempo gravado, como o Z80 do oráculo avança, IDs e inventário do som (Etapa 6), com evidência.
- [sintese.md](sintese.md): sintetizador unificado FM e PSG (Etapa 8), reamostragem, saida web e medicoes diferenciais.
- [abertura.md](abertura.md): do boot à primeira demo (Etapa 7): medidas do oráculo, decisões D7.9 a D7.34, provas com a ROM (veredito IGUAL do quadro 0 ao 1166, com imagem, VRAM, CRAM e VSRAM), navegador e experimento do nível 2.
- [desempenho-web.md](desempenho-web.md): diagnóstico de performance web, eliminação do compositor de candidatos, alocações zero e medidas em Wasm.
- [frontend.md](frontend.md): arquitetura modular de `site/`, servidor local `server.mjs`, UI responsiva, PWA offline, Save States `.s1sav` e cobertura de testes JS.

