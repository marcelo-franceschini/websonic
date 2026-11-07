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
