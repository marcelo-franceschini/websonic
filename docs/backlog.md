# Backlog

Uma linha por item: o que é, dono, etapa-alvo. Só entra aqui o que não bloqueia a etapa atual.

- N8: checksums dos downloads no Dockerfile (rustup, wasmtime, instalador do Claude Code). Dono: Marcelo (Dockerfile é congelado). Etapa-alvo: 5.
- N9: a lista de extensões bloqueadas está repetida em três lugares (`hooks/lib.sh`, `.gitignore`, `xtask`). Dono: Marcelo para `hooks` e `.gitignore`; IA para `xtask`. Etapa-alvo: 5.
- S1b com hook em `/home/dev/.claude/settings.json`: não foi feita (risco de estragar o settings do volume). Dono: Marcelo. Etapa-alvo: 5.
- Proteção de branch em `main` no GitHub. Dono: Marcelo. Etapa-alvo: qualquer.
- Prova 1.9 mostrou `eventos_int80=0` e `eventos_int68=293` em 600 quadros. Confirmar que são esperados (VBlank mascarado no boot; Z80 sem INT). Dono: Marcelo e IA. Etapa-alvo: 5.
- R3 a R6, R8, R12 da revisão da Etapa 1 (ROM real só na prova de determinismo; tempo no início da instrução; estado inicial zerado; sem evento de DMA; wasm do `fidelidade` não rodou; oráculo construído no host). Dono: IA e Marcelo. Etapa-alvo: 2 a 5, ver `docs/conhecimento/oraculo.md`.
- Licença dos TAS do TASVideos não verificada. Dono: Marcelo. Etapa-alvo: 14 (antes do primeiro download).
- Comparar eventos e ciclos exatos com o segundo oráculo (nível 2). Dono: IA. Etapa-alvo: 15.
- Confirmar a especificação dos três formatos em fonte primária (Sonic Retro bloqueou o acesso automático). Dono: Marcelo e IA. Etapa-alvo: 4 (só se o relatório da Etapa 3 mostrar `DIFERE`).
- Provar `NemDec` (porta da VDP) além de `NemDecToRAM`: só a escrita da linha muda. Dono: IA. Etapa-alvo: 5.
- Passos exatos do Nemesis por quadro (fila de arte e `v_plc_*`). Dono: IA. Etapa-alvo: 5.
- Rodar `cargo test --target wasm32-wasip1 -p sonic1-core -p sonic1-teste-descompressao` na imagem. Dono: Marcelo. Etapa-alvo: 5.
- Comparar a saída dos descompressores com o segundo oráculo. Dono: IA. Etapa-alvo: 5.
- Conversor de BK2: o formato (nome do log e ordem dos botoes do Mega Drive) nao foi confirmado em documentacao. Dono: IA. Etapa-alvo: 14, so se houver TAS em BK2.
- Mapa A/B/C do GMV so confere na pagina do TASVideos; confirmar com um TAS real (2.5). Dono: Marcelo e IA. Etapa-alvo: 14.
- Demos do corpus: o caso `boot-demos` cobre 7200 quadros (duas demos e inicio da terceira). Estender para 11700 cobre as quatro. Dono: Marcelo (reassinar o manifesto). Etapa-alvo: 4.
- Desempenho do renderizador: mediana de 0,93 a 1,10 ms no nativo (orcamento 1 ms), sem folga. Otimizar planos e sprites (reuso de linhas sem mudanca). Dono: IA. Etapa-alvo: 7.
- OQ1: o quadro 0 do oraculo e desenhado com largura 256 (esquisitice do jgenesis). Documentado; so importa se o quadro 0 tiver imagem. Dono: IA. Etapa-alvo: 7.
- Sombra e realce, janela, H32, V30 e entrelacamento fora do renderizador. Entram se o corpus ampliado mostrar uso (aviso do `video-estado`). Dono: IA. Etapa-alvo: 10.
- Cobertura do corpus de video: so Sega, titulo, demos GHZ, MZ, SYZ (parcial) e 2000 quadros de LZ1. Faltam special stage, LZ com o Sonic dentro da agua por mais tempo, SLZ, SBZ, FZ, chefes, creditos e telas finais. Dono: Marcelo (gravar/escolher) e IA. Etapa-alvo: 7 a 12, rodar `video-comparar` em cada caso novo.
- Medida em wasm do renderizador (`WEBSONIC_BENCH=1` passado ao wasmtime). Dono: Marcelo. Etapa-alvo: 4 (opcional) ou 7.
- Desempate de cor com o segundo oraculo (`oraculo2 imagens`) nao foi necessario nesta etapa; implementar so se aparecer divergencia de cor. Dono: IA. Etapa-alvo: 14.
- Lista estatica de memoria compartilhada (nomes citados pelas rotinas de interrupcao nos .asm de texto) para cruzar com `docs/inventario-memoria.md`. Dono: IA. Etapa-alvo: 5.
- Atos completos de cada zona nos testes ocultos (a Etapa 2 fecha com um caso por zona). Dono: Marcelo. Etapa-alvo: 14.
- Mapa de rotulos sem tamanhos: o inventario de memoria agrupa pelo rotulo anterior, e pilha e areas sem rotulo caem na ultima variavel. Dono: IA. Etapa-alvo: 5.
- D1 (segundo oráculo): fase de 1 quadro apoiada em 14 quadros do `boot-demos` (até o 5600); falta cobrir 5601 a 7200 (`sh scripts/etapa2-d1c.sh ROM 6100 6600 7100`, um por vez, é lento), olhar o ciclo da interrupção para provar a causa e separar o D1 por grupo em `oraculos.md`. Dono: IA e Marcelo. Etapa-alvo: 5.
- Manifesto sem cobertura de `oracle/runner/tests/*`, `oracle/runner/Cargo.toml`, `crates/fidelidade/Cargo.toml` e de todo o `oracle2/`. Os testes do `oracle/` (workspace separado) também não entram na lista de testes assinada. `oracle2/` não tem regra Edit nem denyWrite no `managed-settings.json`. Dono: IA (`REFERENCIAS` em `xtask/src/manifesto.rs`), Marcelo (congelar `oracle2/` e reassinar). Etapa-alvo: 5.
- TAS: nenhum TAS no corpus e a conferência contra a REV01 (plano 2.5, modo/zona/ato no último quadro) não existe. Corte padrão do plano. Dono: Marcelo e IA. Etapa-alvo: 14.
- Inventário de hardware sem E/S do 68000 (versão, controle, SRAM, TMSS) e sem DMA como evento; o mapa de acessos só cobre a RAM. Dono: IA. Etapa-alvo: 5 (a Etapa 5 precisa para a entrada e a versão).
- A auditoria dos ocultos usa um sistema sob teste mínimo (`scripts/sut-reproduz.sh`: o oráculo roda o caso sem erro). Trocar pelo núcleo comparado com o oráculo. Dono: IA. Etapa-alvo: 5.
- Teste `assinatura_valida_e_adulterada` passa vazio sem `ssh-keygen` (a imagem não tem openssh) e não cobre chave de outro assinante nem namespace errado; não há teste de ponta a ponta de `manifesto verificar` e `fidelidade` falhando. Dono: IA. Etapa-alvo: 5.
- Saída literal da regravação da zona mz (rodada 4) e do `REPRODUZ` das seis zonas não está na base de conhecimento. Dono: Marcelo (colar a saída de `sh scripts/etapa2-rodada4.sh`, só vereditos). Etapa-alvo: 5.
- Identidade `age` dos ocultos sem senha (arquivo 0600). Dono: Marcelo. Etapa-alvo: 14.
