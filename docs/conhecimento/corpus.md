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

- `cargo fidelidade corpus-trace --rom ROM --oraculo BIN --rotulos refs/rotulos.txt --saida DIR [--corpus corpus] [--intervalo 600] [--caso NOME]`.
- Casos: `corpus/demos.txt` (`nome<TAB>quadros`, sem botoes) e cada `corpus/*.s1in`.
- Por caso: duas execucoes (cmp byte a byte e hash) e retomada do ultimo estado salvo antes do fim, comparando os quadros restantes com o trace completo. Imprime `nome hash quadros` e `determinismo: PASSOU`.
- Saida recusada se estiver dentro do repositorio.
- Mudanca no oraculo (congelado), entregue como patch: `docs/pedidos/estado-a-cada.patch` com `--estado-a-cada N`, `--carregar-estado ARQ` (`--quadros` vira o quadro final absoluto) e `--perfil-a-cada N` (imprime modo, zona, ato e demo, so numeros).
- Evidencia (06/11/2026, ROM sintetica `vblank`, 45 quadros, K=10): `determinismo: PASSOU`, retomada sem divergencia. Testes: `cargo test -p xtask` (25), `comparar_cauda` com quadro plantado, trace curto e longo.
- A evidencia com a ROM real ainda nao existe (so o Marcelo roda).

## Manifesto, ocultos e inventario (2.6 a 2.8, lado da IA)

- `cargo fidelidade manifesto gerar|verificar`: `arquivo/trace/zona/oculto/teste` com SHA-256 (implementacao propria, vetores do padrao conferidos com `sha256sum`). Assinatura SSHSIG por `ssh-keygen -Y`, namespace `websonic-manifesto`. Prova (06/11/2026, chave descartavel): `assinatura: OK`, `referencias: OK`, `testes: OK`; os casos plantados (assinatura adulterada ou ausente, arquivo mudado ou ausente, teste ignorado ou faltando, teste novo "nao assinado") estao em `cargo test -p xtask` (testes do xtask (contagem antiga, hoje 34 no relatorio e 48 no xtask na época)).
- `cargo fidelidade ocultos cifrar|auditar|chaves`: crate `age` 0.12.1 (so a chave publica cifra). A auditoria decifra para `/dev/shm`, apaga e imprime so `zona: passou|falhou (N casos)`; erros viram `erro: E1|E2|E3`. Nao existe `decifrar` de proposito. A identidade era um arquivo 0600 sem senha; isso mudou na 2.10 (ver o fim deste arquivo). A afirmacao antiga de que o `age` so oferece senha com a feature `cli-common` estava errada: `age::scrypt` existe sem features (conferido em 07/11/2026 no codigo do crate 0.12.1). Custo: o xtask passou a ter ~340 crates na arvore de dependencias.
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
- **`cargo fidelidade fidelidade`:** manifesto (assinatura, referências, testes), traces do corpus contra o manifesto assinado (mais determinismo) e diferenças do segundo oráculo com decisão registrada em `oraculos.md`. Não roda o segundo oráculo; isso é `scripts/etapa2-oraculo2.sh`. Prova: 3 testes em `cargo test -p xtask`.
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
- `cargo fidelidade manifesto verificar`: `assinatura: OK`, `referências: OK`, `testes: OK (0 não assinados)`.
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

## Fechamento da Etapa 2, sub-etapa 2.10 (IA, no contêiner, 07/11/2026)

Plano: `docs/plano-fechamento-etapa-2.md`. Tudo com dados sintéticos; a ROM não foi usada.

- **O que mede saiu do `xtask`.** O comando de fidelidade, o manifesto, os ocultos e o `corpus-trace` agora vivem em `crates/fidelidade/relatorio/` (pacote `sonic1-relatorio`, binário `relatorio`, workspace próprio como o `oracle/runner`). Atalho `cargo fidelidade <fidelidade|manifesto|ocultos|corpus-trace>` em `.cargo/config.toml`, com `--locked` e `--target-dir target/relatorio`. O `xtask` não guarda cópia. Entregue como patch (`docs/pedidos/etapa2-relatorio.patch`), porque a pasta é congelada: o Marcelo aplica.
- **Prova do patch.** `git apply --check` e `git apply` numa cópia limpa do HEAD (`git archive`); na cópia, `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings` e `cargo test --locked` do pacote passam (34 testes). `cargo fidelidade nada` na cópia imprime o uso, o que prova o atalho e o `Cargo.lock`.
- **Nada ficou mais fraco.** Comparação linha a linha com o `xtask` antigo: as verificações de assinatura, de arquivo, de teste ausente, ignorado e novo, de caso novo não assinado, de determinismo e de retomada são as mesmas. Mudou para mais forte: (1) falha se algum teste executado der `FAILED`, mesmo sem estar no manifesto, e se o `cargo test` terminar com erro; (2) os testes do pacote `relatorio` entram no manifesto e na execução, então teste faltando ou falhando derruba o comando; (3) o `corpus-trace` confere que o trace tem tantos quadros quanto o caso declara e que o hash lido do arquivo fecha com o hash do oráculo; (4) `--intervalo 0` é recusado; (5) o id de diferença do segundo oráculo aceita `D1a`, `D1b`, `D1c`.
- **Manifesto cobre mais** (F7): `.cargo/config.toml`, `oracle/` inteiro, `oracle2/`, `crates/fidelidade/` inteiro (com o `relatorio`), `crates/corpus/`, `corpus/`, `refs/` e `descompressao/`, sempre sem pastas `target`. Teste `cobre_oracle2_corpus_cargo_e_pacotes_congelados`: mudar um arquivo de `oracle2/`, de `crates/corpus/` ou do `.cargo/config.toml` acusa a referência. Limite que já existia e continua: arquivo NOVO numa pasta de referência não é acusado (só os listados). Vai para o backlog.
- **Trace sem cópias** (F2, D-R6). O `corpus-trace` compara a segunda execução (byte a byte, em blocos de 1 MiB) e a retomada (leitura em sequência), e apaga as duas em seguida, com sucesso ou não. Ficam: o hash, `<nome>.quadros.tsv` (uma linha `quadro<TAB>hash` por quadro, o mesmo cálculo do hash do trace), os estados a cada `--intervalo` quadros e a saída de vídeo. O `.s1t` completo só fica com `--manter-trace` (o `scripts/etapa4-host.sh` usa, por causa do `video-comparar`). Provas: testes com oráculo de mentira (não sobra `.b` nem `.c`; uma linha por quadro; falha na segunda execução e na retomada aponta o quadro e não deixa cópias) e uma rodada com o oráculo real e ROM sintética `vblank` (45 quadros, intervalo 10): ficaram 4 estados, `.s1i`, `.s1v` e `quadros.tsv` com 45 linhas, sem nenhum `.s1t`, e `determinismo: PASSOU`.
- **Chave dos ocultos com senha** (F6, D-R7). `ocultos proteger` cifra a identidade com a senha lida da entrada padrão (scrypt do `age`, custo fixo 2^18) e só termina bem depois de abrir o arquivo cifrado e achar a mesma chave; `ocultos conferir` confere a identidade e a senha sem decifrar nenhum teste; `ocultos auditar` aceita a identidade cifrada e lê a senha da entrada padrão. Em perfil de depuração o scrypt leva uns 20 s por abertura. Provas: testes com chave descartável (abre com a senha certa, falha com a errada e com a vazia, o texto da chave não aparece no arquivo, a auditoria dá o mesmo resultado com a identidade aberta) e uma rodada pela linha de comando (`proteger`, `conferir` certo e errado, `cifrar`, `auditar` com senha certa e errada; a errada dá `erro: E1`; a clara continua funcionando).
- **TAS** (F3). `crates/corpus` agora aceita controle de 6 botões: descarta X, Y, Z, Mode e a porta 2 e conta os quadros descartados. Recusa estado salvo (bit 6 do byte 0x16) e 50 Hz (bit 7). `cargo xtask tas converter` gerou `corpus/tas-3570m.s1in` (51166 quadros, 175 com X/Y/Z/Mode descartados, 0 na porta 2) e `corpus/tas-1937m.s1in` (63395 quadros, 63 e 0), com `.meta` (fonte, autor Aglar, licença CC BY 2.0 do TASVideos e o SHA-1 do GMV de origem). Os números batem com a contagem do plano (145 + 30 e 12 + 51). Que o descarte não muda a partida é hipótese; quem confirma é `cargo xtask tas conferir`, que roda com a ROM no script do host.
- **Conferência** (`tas conferir`). Estende as entradas com 3600 quadros sem botão, roda o oráculo com `--perfil-a-cada 60` e aceita o TAS se o modo (sem o bit 7) chegar a `id_Ending` (0x18) ou `id_Credits` (0x1c). Confirmado no `.asm`: o jogo liga o bit 7 de `v_gamemode` no início de `GM_Level` e o apaga em `Level_StartGame`, por isso a conferência ignora esse bit. O oráculo imprime o modo em decimal. Provas: testes com saída de perfil sintética, um caso aceito e um descartado, e o descarte (`--aplicar`) remove `.s1in` e `.meta` e escreve `corpus/descartados.txt`.
- **Demos**: `corpus/demos.txt` ganhou `demos-quatro` (11700 quadros). O `boot-demos` não mudou.
- **Partida gravada**: `scripts/etapa2-gravar.sh ROM corpus CASO` grava o caso público no `corpus/` (sem cifrar) e escreve o `.meta`. Não foi rodado aqui (precisa de janela e ROM).
- **Segundo oráculo**: ver as linhas D1a, D1b e D1c em `oraculos.md`.
- **Não verificado aqui.** `ssh-keygen` e `ssh-agent` não existem neste contêiner: o teste de assinatura do pacote `relatorio` pula, e o fluxo com `ssh-agent` do `scripts/etapa2-fechamento.sh` só foi lido, não rodado. O host roda os dois.

## Fechamento da Etapa 2, sub-etapa 2.11 (rodada do host, 08/11/2026)

Resumo literal do `scripts/etapa2-fechamento.sh` (`/tmp/etapa2-fechamento.txt`). Só vereditos, contagens, hashes e nomes.

- **Partida gravada.** `partida-ghz1`: 8309 quadros, `gravacao: REPRODUZ (RAM do ultimo quadro identica)`. Entrou em `corpus/` com `.meta` (`entradas_sha1: 21fef791b9c51ee361fa99559a9fb81b3c786716`).
- **TAS conferidos contra a REV01 (`tas conferir --aplicar`): os dois foram DESCARTADOS.**
  - `tas-3570m`: não chegou a `id_Ending` nem a `id_Credits`. Último modo lido 12 (fase) no quadro 54720.
  - `tas-1937m`: não chegou a `id_Ending` nem a `id_Credits`. Último modo lido 8 (demo) no quadro 66960.
  - Os dois saíram do corpus e estão em `corpus/descartados.txt`, com a fonte e o motivo.
  - A causa da perda de sincronia **não foi investigada**. Hipóteses, sem prova: diferença de tempo entre o Gens 11b e o jgenesis, ou entradas do GMV (A/B/C de 6 botões) lidas de outro jeito.
  - O requisito diz "TAS convertidos e conferidos contra a REV01". A conversão e a conferência foram feitas. Nenhum TAS ficou aceito no corpus. Ver backlog.
- **D1 em 7 amostras novas** (quadros 6100, 6600, 7100, 7600, 8400, 10200, 11000): `quadros com variavel fora de pilha e tempo: 0 de 7`. Somados aos 14 anteriores, são 21 quadros sem variável de lógica fora de pilha e tempo. A causa continua hipótese.
- **Traces** (`corpus-trace`): `boot-demos` 52ed49ce85b0fc5d69afb977d6d9470a41035c45 (7200), `demos-quatro` a3b53824de96516e7e118469460c6228a70c4b90 (11700), `lz-agua` 9f8d8be138a8099ec8bee31981fc8fce7ca11f52 (2000), `partida-ghz1` ae0740757eb2558c9935b2c53fb5019f6e9b6e26 (8309). `determinismo: PASSOU`.
- **Manifesto**: 266 linhas, assinado com a chave de assinatura. `assinatura: OK`, `referências: OK`, `testes: OK (0 não assinados)`, `manifesto: PASSOU`.
- **Relatório completo** (`cargo fidelidade fidelidade`): `traces: PASSOU`, `segundo oráculo: 3 diferença(s) registrada(s), 0 sem decisão`, `veredito: PASSOU`.
- **Auditoria dos ocultos**: `ghz`, `lz`, `mz`, `sbz`, `slz`, `syz`: `passou (1 casos)` cada. Limite antigo continua: prova decifração e execução, não fidelidade.
- **Chave dos ocultos**: protegida com senha (`ocultos.key.age`); a cópia sem senha foi apagada.
- **Chave de assinatura**: o Marcelo deixou a senha vazia nas duas rodadas. A chave **continua sem senha**. Pendente (backlog).
- **Falha achada e corrigida na rodada**: o `ssh-keygen -Y sign` perguntava "Overwrite (y/n)?" e parou a primeira tentativa. O script agora apaga a assinatura antiga antes (commit 4722d95).
- **Ainda não feito**: reconstruir a imagem do contêiner (o congelamento só vale na imagem nova).

### Saída literal da conferência dos TAS (rodada de 07/11/2026, registro da correção B2)

Linhas `modos (quadro=modo)` do `tas conferir`. Modo sem o bit 7: 0 Sega, 4 título, 8 demo, 12 fase, 16 special stage. `id_Ending` e `id_Credits` nunca apareceram.

- `tas-3570m`: 60=0 360=4 960=8 3000=0 3300=4 3900=8 5940=0 6300=4 6900=8 7440=4 8100=16 9600=0 9900=4 10500=8 12540=0 12840=4 13440=8 14220=4 14400=12 39120=0 39360=4 39600=12
- `tas-1937m`: 60=0 360=4 960=8 3000=0 3300=4 3900=8 5940=0 6300=4 6900=8 7800=4 8400=16 9960=0 10200=4 10800=8 12840=0 13200=4 13800=8 14940=4 15540=8 17580=0 17880=4 18480=16 20040=0 20340=4 20940=8 22980=0 23280=4 23880=8 25920=0 26220=4 26820=8 28860=0 29220=4 29820=16 30480=4 30720=12 50700=0 51000=4 51600=8 53640=0 53940=4 54540=8 56580=0 56880=4 57480=8 59520=4 60120=16 61680=0 61920=4 62520=8 64560=0 64920=4 65520=8

Leitura (hipótese, sem prova): a sequência é a de um jogo que **reinicia** (0 Sega, 4 título, 8 demo, de novo 0) a cada uns 2900 quadros. Os dois TAS entram na fase (modo 12) uma vez e depois voltam ao ciclo Sega, título, demo. Isso aponta para causa comum: a partida do TAS perde a sincronia logo no começo da primeira fase (fase GHZ aos quadros 14400 e 30720) e o jogo cai no reinício. Candidatos: leitura dos botões do GMV de 6 botões (mapa A/B/C), tempo diferente entre Gens 11b e jgenesis, ou estado inicial. Nenhum foi testado.
