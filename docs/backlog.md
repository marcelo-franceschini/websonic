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
- Confirmar a especificação dos três formatos em fonte primária (Sonic Retro bloqueou o acesso automático). Dono: Marcelo e IA. Etapa-alvo: 4 (só se o relatório da Etapa 3 mostrar `DIFERE`).
- Provar `NemDec` (porta da VDP) além de `NemDecToRAM`: só a escrita da linha muda. Dono: IA. Etapa-alvo: 5.
- Passos exatos do Nemesis por quadro (fila de arte e `v_plc_*`). Dono: IA. Etapa-alvo: 5.
- Rodar `cargo test --target wasm32-wasip1 -p sonic1-core -p sonic1-teste-descompressao` na imagem. Dono: Marcelo. Etapa-alvo: 5.
- Comparar a saída dos descompressores com o segundo oráculo. Dono: IA. Etapa-alvo: 2.
- Conversor de BK2: o formato (nome do log e ordem dos botoes do Mega Drive) nao foi confirmado em documentacao. Dono: IA. Etapa-alvo: 2 ou depois, so se houver TAS em BK2.
- Mapa A/B/C do GMV so confere na pagina do TASVideos; confirmar com um TAS real (2.5). Dono: Marcelo e IA. Etapa-alvo: 2.
- D1: o segundo oraculo (Genesis Plus GX) diverge do jgenesis no quadro 52 do `boot-demos`. Diagnosticar com os enderecos das palavras que diferem. Dono: IA e Marcelo. Etapa-alvo: 4 (antes do desempate de cor) ou 14.
- Etapa 2 incompleta: gravador de partidas (2.2), inventario de memoria compartilhada (pedido P2), testes ocultos reais, `cargo xtask fidelidade` completo e revisao. Dono: IA e Marcelo. Etapa-alvo: antes da 5.
- Demos do corpus: o caso `boot-demos` cobre 7200 quadros (duas demos e inicio da terceira). Estender para 11700 cobre as quatro. Dono: Marcelo (reassinar o manifesto). Etapa-alvo: 4.
