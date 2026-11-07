# Corpus e traces (Etapa 2)

## Formato `.s1in` e `.meta` (2.3)

- `corpus/<nome>.s1in`: uma linha por quadro do emulador, mascara de 2 digitos hexadecimais, `#` comenta. Bits: Cima 01, Baixo 02, Esquerda 04, Direita 08, B 10, C 20, A 40, Start 80. Leitor estrito: recusa linha com tamanho diferente de 2.
- `corpus/<nome>.meta`: `chave: valor` com `fonte`, `licenca`, `rom_sha1`, `quadros`, `entradas_sha1`, `autor`. O `corpus-trace` recusa o caso se `quadros` ou `entradas_sha1` nao baterem com o `.s1in`.
- Quadro de lag: a linha vale para um quadro de video. O jogo ignora a linha sozinho. Nenhuma conversao corrige lag.
- Codigo: `crates/corpus` (MIT). Prova: `cargo test -p sonic1-corpus`, 9 testes com dados sinteticos (ida e volta, malformados, GMV inventado).

## GMV (conversor feito) e BK2 (NAO feito)

- Fonte do GMV: https://tasvideos.org/EmulatorResources/Gens/GMV, lida em 06/11/2026. Cabecalho de 64 bytes, 3 bytes por quadro, bits ativos em zero, A=10, B=20, C=40, Start=80. A pagina nao separa 3 e 6 botoes: so aceitamos controle 1 de 3 botoes ('3') e 60 Hz.
- Risco aberto: o mapa A/B/C vem so dessa pagina. Um TAS real precisa confirmar (a conferencia contra a REV01 da sub-etapa 2.5 mostraria dessincronia).
- BK2: a pagina https://tasvideos.org/BizHawk/BK2Format (lida em 06/11/2026) diz que e um zip com Input Log, mas nao da o nome exato do arquivo nem a ordem dos botoes do Mega Drive. Pela regra do plano ("se um formato nao for confirmavel, nao entra"), o BK2 ficou fora. Backlog.

## Traces do corpus (2.4)

- `cargo xtask corpus-trace --rom ROM --oraculo BIN --rotulos refs/rotulos.txt --saida DIR [--corpus corpus] [--intervalo 600] [--caso NOME]`.
- Casos: `corpus/demos.txt` (`nome<TAB>quadros`, sem botoes) e cada `corpus/*.s1in`.
- Por caso: duas execucoes (cmp byte a byte e hash) e retomada do ultimo estado salvo antes do fim, comparando os quadros restantes com o trace completo. Imprime `nome hash quadros` e `determinismo: PASSOU`.
- Saida recusada se estiver dentro do repositorio.
- Mudanca no oraculo (congelado), entregue como patch: `docs/pedidos/estado-a-cada.patch` com `--estado-a-cada N`, `--carregar-estado ARQ` (`--quadros` vira o quadro final absoluto) e `--perfil-a-cada N` (imprime modo, zona, ato e demo, so numeros).
- Evidencia (06/11/2026, ROM sintetica `vblank`, 45 quadros, K=10): `determinismo: PASSOU`, retomada sem divergencia. Testes: `cargo test -p xtask` (25), `comparar_cauda` com quadro plantado, trace curto e longo.
- A evidencia com a ROM real ainda nao existe (so o Marcelo roda).

## Manifesto, ocultos e inventario (2.6 a 2.8, lado da IA)

- `cargo xtask manifesto gerar|verificar`: `arquivo/trace/zona/oculto/teste` com SHA-256 (implementacao propria, vetores do padrao conferidos com `sha256sum`). Assinatura SSHSIG por `ssh-keygen -Y`, namespace `websonic-manifesto`. Prova (06/11/2026, chave descartavel): `assinatura: OK`, `referencias: OK`, `testes: OK`; os casos plantados (assinatura adulterada ou ausente, arquivo mudado ou ausente, teste ignorado ou faltando, teste novo "nao assinado") estao em `cargo test -p xtask` (40 testes).
- `cargo xtask ocultos cifrar|auditar|chaves`: crate `age` 0.12.1 (so a chave publica cifra). A auditoria decifra para `/dev/shm`, apaga e imprime so `zona: passou|falhou (N casos)`; erros viram `erro: E1|E2|E3`. Nao existe `decifrar` de proposito. A identidade e um arquivo 0600 sem senha (decisao: o crate so oferece senha com a feature `cli-common`; protecao em repouso fica por conta do disco do Marcelo). Custo: o xtask passou a ter ~340 crates na arvore de dependencias.
- `cargo xtask inventario --traces DIR`: decodifica os eventos de VDP (registradores, comandos, DMA pelo reg 23), YM, PSG, nivel de interrupcao e BUSREQ/RESET. Nao cobre DMA como evento (R6) nem E/S do 68000 (P2). Teste: 4 casos sinteticos.
- Casos de lag e DMA do VDP que o renderizador da Etapa 4 vai precisar: o trace so tem as escritas nas portas, nao o conteudo dos DMAs. Registrado para o plano da Etapa 4.

## Rodada do host (06/11/2026)

- Perfil das demos do `boot-demos` (quadro: modo/zona): Sega 0 a 360, titulo 360 a 960, demo da zona 0 (GHZ) de 960 a 3000, titulo, demo da zona 2 (MZ) 3900 a 5940, titulo, demo da zona 4 (SYZ) 6900 a 8880, titulo, special stage (modo 16) 9840 a 11400. O caso atual (7200 quadros) cobre duas demos e o inicio da terceira. Cobrir as quatro pede 11700 quadros (trace de cerca de 3,4 GB), pendente.
- `boot-demos`, 7200 quadros: `hash_do_trace` `d0acd7feb9e29c0688da54ce42b74c50bcbb81e9` (hash antigo, de antes dos patches 0002 e 0003; o vigente está em `refs/manifesto.txt`), `determinismo: PASSOU` (duas execucoes e retomada do ultimo estado). Trace de 2,0 GB.
- Inventario de hardware (`docs/inventario-hardware.md`): 47 pares registrador/valor da VDP; DMA da 68000 para a VDP 20617 vezes e preenchimento de VRAM 26; DAC com 796736 escritas; so interrupcao de nivel 6 (VBlank). **Nenhuma interrupcao de nivel 4 (HBlank) no corpus das demos**: a agua da Labyrinth e as mudancas de paleta no meio do quadro nao aparecem aqui. A Etapa 4 precisa de um caso na Labyrinth (partida gravada ou TAS) para provar o meio do quadro.
- Manifesto assinado com ssh-keygen ed25519; `manifesto verificar`: assinatura OK, referencias OK, testes OK.

## Fechamento da Etapa 2 (06/11/2026, lado da IA)

- **Mapa de acessos à RAM por contexto (P2).** Patch 0003 do jgenesis (`oracle/patches/`), flag `oraculo rodar --acessos ARQ`. Formato: `FFxxxx<TAB>BITS`, bit `contexto*3+tipo` (contexto: 0 principal, 1 VBlank, 2 HBlank, 3 outro; tipo: 0 leitura, 1 escrita, 2 leitura por DMA). O contexto vem da pilha de interrupções tomadas (evento INT68) e dos RTE (desvio RTE). Saída só com endereços e bits.
- **Armadilha achada na prova sintética:** o jgenesis adia as escritas do 68000 (`pending_writes`) e as aplica depois da instrução. Marcar o acesso na aplicação atribuía as escritas da pilha de exceção ao contexto errado. O gancho de escrita fica em `write_byte`/`write_word` do barramento do 68000. Prova: `oracle/runner/tests/acessos.rs` (contador do VBlank só no contexto VBlank; quadro de exceção escrito no principal e lido no VBlank).
- **Inventário:** `cargo xtask inventario-memoria --acessos A.acc --acessos B.acc` gera `docs/inventario-memoria.md`. Compartilhada = tocada no laço principal e numa interrupção, com ao menos uma escrita (leitura por DMA conta como acesso de interrupção). Variáveis vêm de `refs/rotulos.txt`; um byte pertence à variável de maior endereço menor ou igual a ele. Pilha e áreas sem rótulo caem na última variável anterior, com alcance grande (coluna "alcance"). Limite: o mapa de rótulos não tem tamanhos. A lista estática (nomes que as rotinas de interrupção citam nos `.asm`) ficou no backlog.
- **Gravador (2.2), opção A com variação:** em vez de editar o frontend, o patch grava no núcleo. `WEBSONIC_GRAVAR=BASE` faz o núcleo gravar `BASE.s1in` (máscara do primeiro passo de cada quadro) e `BASE.ram` (RAM do 68000 do último quadro fechado). Vale para qualquer frontend que use o núcleo, inclusive o `jgenesis` com janela. Prova sintética: `oracle/runner/tests/gravador.rs`. Prova com janela (host): o oráculo sem janela, com as entradas gravadas, tem de gerar a mesma RAM (`cmp`). Se a janela estiver com região, temporização ou controle diferentes do oráculo, a prova acusa `DIFERE` e nada é cifrado. Limite: se o botão mudar no meio de um quadro, a gravação guarda só o valor do início; a prova pega essa divergência.
- **Testes ocultos, decisão:** o mínimo para fechar a Etapa 2 passa a ser um caso por zona (`ghz mz syz lz slz sbz`). Os atos completos de cada zona valem até a Etapa 14. Motivo: o requisito diz "primeiros testes ocultos, por zona". O texto claro vive só em `/dev/shm` durante `etapa2-gravar.sh`.
- **`cargo xtask fidelidade`:** manifesto (assinatura, referências, testes), traces do corpus contra o manifesto assinado (mais determinismo) e diferenças do segundo oráculo com decisão registrada em `oraculos.md`. Não roda o segundo oráculo; isso é `scripts/etapa2-oraculo2.sh`. Prova: 3 testes em `cargo test -p xtask`.
- **D1:** `oraculo2 diferencas` compara os despejos de RAM dos dois oráculos no quadro N e imprime só faixas de endereço, nome da variável mais próxima e contagens. Roda em `scripts/etapa2-final.sh`. O resultado foi lido nas rodadas seguintes, ver `oraculos.md`.
- **Hash do trace:** o cabeçalho entra no hash e carrega o hash dos patches. Depois do patch 0003 os `hash_do_trace` mudam e o manifesto é regerado (`scripts/etapa2-fechar.sh`).

## Rodada do host (07/11/2026): gravador e ocultos

- Prova do gravador com janela: `gravacao: REPRODUZ` em 2526 quadros (RAM do último quadro idêntica ao oráculo sem janela). Configuração forçada por `--config-override` (`genesis.forced_region=Japan`, `genesis.forced_timing_mode=Ntsc`, `input.genesis.p1_type=ThreeButton`).
- Cinco zonas gravadas e cifradas (ghz, syz, lz, slz, sbz), cada uma com `REPRODUZ`. A auditoria decifra em `/dev/shm` e imprime `zona: passou (1 casos)` para as cinco.
- A zona mz deu `DIFERE`. Causa provável: um botão que mudou no meio de um quadro (o gravador guardava só o valor do início, mas a janela usava o valor corrente). Correção: com o gravador ligado, o núcleo usa os botões do início do quadro durante o quadro inteiro (patch `docs/pedidos/etapa2-oraculo-2.patch`, teste `oracle/runner/tests/gravador.rs`, que falha sem a correção). A mz foi regravada na rodada 4.
- Aviso do jgenesis: `Invalid keyboard input string` para a tecla de rebobinar (crase) na configuração do usuário. Não afeta a gravação.

## Revisão da Etapa 2 (sessão nova, 07/11/2026)

Provas rodadas pela revisão, sem ROM (contêiner de trabalho, sem o oráculo):
- `cargo fmt --check`: sem saída. `cargo clippy --workspace --all-targets -- -D warnings`: exit 0. `cargo test --workspace`: todos `ok` (xtask 48, corpus 9, fidelidade 19, video 44+4+1, rom 13).
- `cargo xtask anti-rom`: `anti-rom: nenhum arquivo suspeito`. Nenhum `.s1t`, `.s1e`, `.ram`, `.acc`, `.age`, `.bin` ou `.gen` rastreado (`git ls-files`).
- `cargo xtask manifesto verificar`: `assinatura: OK`, `referências: OK`, `testes: OK (0 não assinados)`.
- Núcleo: nenhum `HashMap`, `HashSet`, `f32`, `f64`, `#[allow]` nem cast com `as` em `crates/core/src`; lints `deny` no `Cargo.toml` do crate.
- Patches: para cada commit que tocou `oracle/` desde c188a83 (a33b3c0, 6121249, 15e4e2d, 633007d, e93909d), toda linha adicionada em `oracle/runner` e `oracle/patches` está em algum arquivo de `docs/pedidos/*.patch`. Zero linhas fora dos patches. O git não distingue quem aplicou (autor único), então a prova é a coincidência com os patches.
- Inventários: sem sequência hexadecimal longa nem mnemônico. Os únicos hexadecimais longos do repositório fora do manifesto são hashes SHA-1 em `corpus/lz-agua.meta`, neste arquivo e em `oraculos.md`.

Afirmações desta base sem comando e resultado registrados (marcadas sem prova):
- Perfil das demos (quadro e modo/zona) e o tamanho de 3,4 GB do trace de 11700 quadros: sem comando nem saída literal.
- Prova do gravador (`REPRODUZ`, 2526 quadros) e das cinco zonas: o comando é `scripts/etapa2-gravar.sh`, mas a saída literal não está aqui. A regravação da zona mz depois da correção (rodada 4, `scripts/etapa2-rodada4.sh`) só aparece em `docs/estado.md`, sem saída.
- "Testes do oráculo OK, incluindo o que falha sem a correção" (`oracle/scripts/testa.sh`): depende do host, a revisão não rodou.
- `age` só oferece senha com a feature `cli-common` e a árvore tem cerca de 340 crates: sem comando.
- Mapa A/B/C do GMV: só a página do TASVideos, nenhum TAS real conferido.
- Contagens de testes antigas (25 e 40 do xtask) valiam na época. Hoje são 48.
