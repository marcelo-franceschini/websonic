# Backlog

Uma linha por item: o que é, dono, etapa-alvo. Só entra aqui o que não bloqueia a etapa atual.

- N8: checksums dos downloads no Dockerfile (rustup, wasmtime, instalador do Claude Code). Dono: Marcelo (Dockerfile é congelado). Etapa-alvo: 2.
- N9: a lista de extensões bloqueadas está repetida em três lugares (`hooks/lib.sh`, `.gitignore`, `xtask`). Dono: Marcelo para `hooks` e `.gitignore`; IA para `xtask`. Etapa-alvo: 2.
- S1b com hook em `/home/dev/.claude/settings.json`: não foi feita (risco de estragar o settings do volume). Dono: Marcelo. Etapa-alvo: 2.
- Proteção de branch em `main` no GitHub. Dono: Marcelo. Etapa-alvo: qualquer.
- Parser de rótulos achou só 2 variáveis (esperado centenas de `v_...`). Conferir classificação em `refs/rotulos.txt` e no xtask. Dono: IA. Etapa-alvo: 2 (antes dos inventários).
- Prova 1.9 mostrou `eventos_int80=0` e `eventos_int68=293` em 600 quadros. Confirmar que são esperados (VBlank mascarado no boot; Z80 sem INT). Dono: Marcelo e IA. Etapa-alvo: 2.
