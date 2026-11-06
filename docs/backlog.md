# Backlog

Uma linha por item: o que é, dono, etapa-alvo. Só entra aqui o que não bloqueia a etapa atual.

- N8: checksums dos downloads no Dockerfile (rustup, wasmtime, instalador do Claude Code). Dono: Marcelo (Dockerfile é congelado). Etapa-alvo: 2.
- N9: a lista de extensões bloqueadas está repetida em três lugares (`hooks/lib.sh`, `.gitignore`, `xtask`). Dono: Marcelo para `hooks` e `.gitignore`; IA para `xtask`. Etapa-alvo: 2.
- S1b com hook em `/home/dev/.claude/settings.json`: não foi feita (risco de estragar o settings do volume). Dono: Marcelo. Etapa-alvo: 2.
- Proteção de branch em `main` no GitHub. Dono: Marcelo. Etapa-alvo: qualquer.
- Parser de rótulos achou só 2 variáveis (esperado centenas de `v_...`). Conferir classificação em `refs/rotulos.txt` e no xtask. Dono: IA. Etapa-alvo: 2 (antes dos inventários).
- Prova 1.9 mostrou `eventos_int80=0` e `eventos_int68=293` em 600 quadros. Confirmar que são esperados (VBlank mascarado no boot; Z80 sem INT). Dono: Marcelo e IA. Etapa-alvo: 2.
- R3 a R6, R8, R12 da revisão da Etapa 1 (ROM real só na prova de determinismo; tempo no início da instrução; estado inicial zerado; sem evento de DMA; wasm do `fidelidade` não rodou; oráculo construído no host). Dono: IA e Marcelo. Etapa-alvo: 2 a 5, ver `docs/conhecimento/oraculo.md`.
- Licença dos TAS do TASVideos não verificada. Dono: Marcelo. Etapa-alvo: 2 (antes do primeiro download).
- Comparar eventos e ciclos exatos com o segundo oráculo (nível 2). Dono: IA. Etapa-alvo: 15.
- Confirmar o formato GMV e BK2 e a API do crate `age` na documentação. Dono: IA. Etapa-alvo: 2 (2.3 e 2.8).
