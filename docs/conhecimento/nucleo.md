# Núcleo base (Etapa 5)

Plano: `docs/plano-etapa-5.md`. Cada afirmação cita o teste ou o comando. Nada aqui tem dado da ROM. Estado de cada prova no fim do arquivo.

## Onde cada coisa mora

| O quê | Onde | Observação |
| --- | --- | --- |
| Memória, portas, saídas de hardware | `crates/core/src/memoria/` | lints do núcleo valem |
| Operações do 68000 com flags | `crates/core/src/ops68k/` | funções puras, sem registradores |
| Laço de quadro, tempo gravado, pontos de checagem | `crates/core/src/quadro/` | infraestrutura, sem par no original |
| Registro de decisões e bugs injetados | `crates/core/src/cobertura/` | features `cobertura` e `mutacao` |
| Funções por rotina do original (piloto) | `crates/core/src/rotinas/` | `KosDec`, `EniDec`, `NemDecToRAM` |
| Vetores, gabarito por fórmulas, harness | `crates/teste-nucleo/` | metadados em `rotinas/*.txt` |
| Oráculo contra núcleo | `diferencial/` | workspace GPLv3 próprio; nunca no build do jogo |
| Cobertura e mutantes, relatórios | `crates/fidelidade/relatorio/` (patch P5-2) | congelado |
| Metadado da cobertura | `cobertura/` | mapa, executados, equivalências, exclusões propostas |

## Memória (5.1)

- Mapa e comportamento tirados do oráculo (`backend/genesis-core/src/bus.rs`, só nomes de função) e de fontes públicas: [Genesis Programming, 68K Memory map](https://en.wikibooks.org/wiki/Genesis_Programming/68K_Memory_map/). Larguras exatas divergem entre as fontes, por isso a regra é o oráculo.
- A RAM do 68000 repete a cada 64 KiB em `E00000` a `FFFFFF`. A janela do Z80 (`A00000` a `A0FFFF`) só responde com BUSREQ pedido e o Z80 fora do reset; `8000` a `FFFF` espelha `0000` a `7FFF`; a RAM do Z80 de 8 KiB repete em `2000`. Escrita em palavra na janela grava o byte alto e leitura em palavra repete o byte nos dois lados.
- ROM além do arquivo, e qualquer região sem dispositivo, leem o **barramento aberto**: o valor é a última palavra que passou pelo barramento do 68000, que no oráculo inclui a busca de instruções. O núcleo não busca instruções, então guarda o último valor lido e oferece `definir_barramento_aberto` para os testes. Escrita em ROM e em região sem dispositivo é ignorada e contada como aviso.
- Versão do console de referência (Japão, NTSC, sem Sega CD): `$A10001` = `0x20` (bit 7 região, bit 6 PAL, bit 5 sem Sega CD), igual ao oráculo.
- Portas de controle: modelo de pinos do controle de 3 botões na porta 1 e nada na 2 e na extensão. Direções opostas apertadas juntas se anulam (o oráculo não as permite); a prova achou isso (`mascara 03` e `0C`). Limite: a volta do pino TH a 1 por tempo, quando o jogo deixa TH como entrada, é do nível 2.
- Palavra ou longa em endereço ímpar: o núcleo devolve `ErroAcesso::EnderecoImpar`; o oráculo toma a exceção de erro de endereço (vetor 3) e a rotina não retorna (teste `endereco_impar_em_palavra_e_excecao_no_oraculo_e_erro_no_nucleo`).
- Leituras de BUSACK, estado da VDP e estado do YM2612 vêm do tempo gravado (`ClasseLeitura`), na ordem. Sem valor gravado, repete o último e conta aviso.
- Saídas de hardware (`Saida`): escritas do 68000 na VDP (porta, valor, tamanho), PSG, YM pela janela do Z80, BUSREQ e RESET, no mesmo sentido dos eventos 5 a 8 do trace. Longa vira duas palavras, a alta primeiro.
- Evidência: `diferencial/tests/memoria.rs`, 6 testes que comparam o núcleo com o oráculo em ROM sintética (leitura em três tamanhos em 40 endereços, com o Z80 cedido e sem ceder; escrita e leitura de volta; saídas de hardware; as 256 combinações de botões). Todos passam. `cargo test -p sonic1-core`: 24 testes.

## Operações do 68000 (5.2)

- Funções: ADD, ADDX, SUB, SUBX, NEG, NEGX, CMP, CMPA, ADDA, SUBA, CLR, TST, AND, OR, EOR, NOT, MOVE, EXT, SWAP, MULU, MULS, DIVU, DIVS, ABCD, SBCD, NBCD, as oito rotações e deslocamentos, BTST, BSET, BCLR, BCHG, as 16 condições, Scc e DBcc. A lista fechada "usada pelo jogo" sai do inventário `docs/inventario-68000.md` (bloco A); até lá, as famílias estão completas.
- Fonte das flags: Motorola, M68000 Family Programmer's Reference Manual ([NXP](https://nxp.com/docs/en/reference-manual/M68000PM-PART2.pdf)). O gabarito independente usa as fórmulas booleanas de V e C do manual (`crates/teste-nucleo/src/ops.rs`); a implementação usa aritmética em tipo mais largo.
- Flags indefinidas (N e V em ABCD, SBCD e NBCD; N, V, Z e C em DIV com estouro): o núcleo segue o oráculo, medido. Estouro de DIVU e DIVS: destino intacto, N e V ligados, C e Z desligados.
- **O jgenesis em build de depuração entra em pânico em `i32::MIN / -1` (DIVS).** O caso `0x80000000 / 0xFFFF` fica fora do diferencial; o quociente não cabe em word de qualquer jeito (V ligado). Em build de release o oráculo não entra em pânico.
- Evidência (07/11/2026, ROM sintética, uma instrução por caso, comparando D0 inteiro e os 5 bits do CCR): `diferencial/tests/ops.rs`, 6 testes, todos passam com `WEBSONIC_OPS_CASOS=1500`: aritmética e lógica nos três tamanhos (14 operações), EXT, SWAP, MULU, MULS, DIVU e DIVS (inclusive divisão por zero: o oráculo toma a exceção e o núcleo devolve `DivisaoPorZero`), ABCD e SBCD nos 65536 pares com os 32 CCRs, NBCD nos 256 valores com os 32 CCRs, os oito deslocamentos e rotações com contagem de 0 a 70, BTST, BCHG, BCLR, BSET, Scc e DBcc nas 16 condições com os 32 CCRs, ADDA, SUBA e CMPA. `crates/teste-nucleo/tests/ops68k.rs`: 12 testes (vetores, byte exaustivo contra as fórmulas do manual, propriedades).
- O diferencial roda em paralelo (uma `Bancada` por thread, porque o estado de observação do oráculo é por thread). Em build de depuração, uma chamada leva ~3 ms; em release, ~0,4 ms. O bloco B usa release.

## Tempo gravado e laço de quadro (5.3)

- Arquivo `.s1tempo` (formato em `crates/core/src/quadro/tempo.rs`): por quadro, `mclk` de início e de fim (o orçamento), e os eventos de tempo na ordem: interrupção (nível, PC interrompido, PC do tratador, `mclk`, linha, coluna), leitura de hardware (classe, valor lido, `mclk`) e espera do Z80 consumida. O oráculo o escreve com `oraculo rodar --tempo` (patch 0004). Os arquivos reais ficam em `privado/`, fora do git.
- `Nucleo::executar_quadro(botoes, passos)`: aplica a linha de entrada, enfileira as leituras gravadas, roda os passos e entrega as interrupções. **Quadro de lag** = VBlank (nível 6) gravado com o PC fora da rotina de espera (`WaitForVBlank`, `refs/rotulos.txt`); nesse quadro a linha de entrada é ignorada. Quadro sem VBlank gravado não é lag (o VBlank fica mascarado no boot).
- **Desvio do plano:** o estouro de orçamento só é registrado (`estourou`); adiar o resto do trabalho para o quadro seguinte exigiria custo em ciclos, que é o nível 2 (Etapa 15). Os passos de brinquedo dos testes declaram um custo para provar o registro.
- Fica para o nível 2: prever orçamento, ponto das interrupções, laços de espera e esperas do Z80; custo em ciclos.
- Evidência: testes de `crates/core/src/quadro/`: ida e volta e arquivo ruim recusado (magia, versão, truncado, marca, tipo), lag, entrada ignorada em lag, leituras gravadas na ordem, orçamento.

## Pontos de checagem (5.4)

- `Ponto { id, rotulo, inicio, fim, endereco, posicao }`. Uma interrupção gravada com PC na faixa da rotina é entregue no primeiro ponto que a cobre: `Antes` se o PC ainda não passou do acesso, `Depois` se já passou. Só a primeira pendente pode ser entregue (interrupções não se reordenam). Sem ponto que cubra, é entregue no fim dos passos e marcada `ponto: None`.
- `conferir_entregas` compara as entregas do núcleo com o tempo gravado, quadro a quadro: interrupção perdida, extra, fora de ordem, ou ponto que não cobre o PC.
- Evidência: testes `interrupcao_antes_e_depois_do_acesso_ve_valores_diferentes` (o tratador vê a variável antes ou depois da escrita, conforme o PC gravado), `interrupcao_em_rotina_sem_ponto_e_entregue_no_fim`, `duas_interrupcoes_no_mesmo_quadro_nao_se_reordenam`, `conferencia_acusa_perdida_extra_ordem_e_ponto_errado`.
- A lista de pontos reais depende de `docs/inventario-interrupcoes.md` (bloco A) e de `docs/inventario-memoria.md` (546 variáveis compartilhadas de 751). Rotinas reais entram nas Etapas 6 a 12.

## Patch 0004 do oráculo (5.0)

- Três saídas novas, só com nomes, endereços e contagens: `--instrucoes` (classe e tamanho; a classe é o nome da variante do decodificador do jgenesis), `--barramento` (região, operação, bytes; mais a contagem de exceções por vetor) e `--tempo` (`.s1tempo`). Também troca a leitura da RAM do 68000 por uma cópia direta (a chamada de rotina isolada ficou ~8x mais rápida nos testes de operações).
- Evidência: `oracle/runner/tests/inventarios.rs` (3 testes) e a suíte antiga do oráculo, todos passam numa cópia com o patch aplicado (`docs/pedidos/etapa5-oraculo.patch`); clippy `-D warnings` limpo.
- Os leitores `cargo xtask inventario-68000`, `inventario-barramento` e `inventario-interrupcoes` têm 3 testes.

## Teste por rotina (5.5)

- Metadado `crates/teste-nucleo/rotinas/NOME.txt`: `rotulo`, `entrada`, `usados_depois` e `ignorados`. Todo registrador (d0 a d7, a0 a a6, ccr) tem de estar numa das duas listas; o analisador recusa o resto. Para `KosDec`, `EniDec` e `NemDecToRAM`, `usados_depois` é **vazio**: lendo os pontos de chamada nos `.asm` de texto (conferidos com `grep` em 08/11/2026: `sonic.asm`, `LevelLayoutLoad.asm` e o fundo da special stage; as 3 linhas depois de cada chamada só recarregam A0 e A1 com `lea` ou `movea`), nenhum chamador usa A0 ou A1 depois, e `EniDec` restaura A1 com `movem`. Só a RAM importa.
- O núcleo não guarda registradores. O adaptador devolve só os valores de `usados_depois`. **Desvio do plano:** a comparação de "todos os registradores na primeira rodada" não se aplica ao lado do núcleo; em troca, o relatório lista os registradores `ignorados` que o ORÁCULO mexeu (entrada contra saída), para quem revisa conferir se algum chamador os lê. O bloco B imprime esse conjunto por rotina.
- `diferencial rotina NOME`: para cada bloco do manifesto de descompressão, parte do mesmo estado, roda a rotina no oráculo (`chamar_rotina`) e a função do núcleo, e compara a RAM inteira (menos a área excluída) e os registradores usados. A área excluída é a pilha da chamada (512 bytes abaixo do SP de entrada), **proposta** em `cobertura/exclusoes-propostas.txt` e que só vale com a aprovação do Marcelo.
- Mapa de decisões **inferido por dados:** o `diferencial rotina` compara, para cada decisão do núcleo, a sequência de resultados com a de cada desvio condicional do original que o oráculo executou, nos dois sentidos (direto ou inverso), em todos os casos. Só uma decisão com um único candidato vira linha de `cobertura/mapa-decisoes.txt`. A conferência seguinte usa o mapa declarado e acusa qualquer caso em que a sequência difere. Assim o mapeamento não depende de a IA ler o assembly com atenção: o erro aparece como divergência.
- Evidência com dados sintéticos (ROM de 12 palavras escrita por nós, `diferencial/tests/rotina.rs`): núcleo certo passa e o mapa `Limita:alto` → desvio `406` (inverso) é inferido; byte de RAM errado, registrador usado errado e polaridade trocada são acusados; diferença dentro da área excluída não reprova. `diferencial/tests/cli.rs` roda o comando inteiro com ROM sintética. Com a ROM real: pendente (bloco B).

## Cobertura (5.6)

- Fonte da lista de decisões do núcleo: as chamadas `decisao!("Rotina:nome", cond)` no código-fonte. Piloto anotado: `KosDec` (5 decisões) e `EniDec` (4). `NemDecToRAM` ainda não tem decisões anotadas (backlog).
- Medida no núcleo: `crates/teste-descompressao/tests/cobertura.rs` roda entradas sintéticas e exige que TODA decisão do código-fonte vá nos dois caminhos. Resultado: as 9 decisões, nos dois caminhos (07/11/2026).
- Medida no original: `cargo fidelidade cobertura extrair` soma os desvios condicionais (Bcc e DBcc) dos traces; o `diferencial rotina --executados-saida` soma os dos testes por rotina. `cargo fidelidade cobertura relatorio` cruza com o mapa. Reprova: decisão sem teste, decisão de um caminho só, desvio mapeado que o oráculo nunca executou, mapa que cita decisão inexistente.
- **Desvio do plano (limite registrado):** o universo é "o que o oráculo executou", que é o que o requisito diz ("cada desvio do código original ... executado pelo corpus e pelos testes por rotina"). Listar desvios que ninguém executou exigiria decodificar o código sem executar, e o decodificador do jgenesis não é público. Fica no backlog; a Etapa 14 precisa disso para o critério 2 do pronto.
- Evidência do relatório: 6 testes em `crates/fidelidade/relatorio/src/cobertura.rs` (45 no pacote).

## Bugs injetados (5.7)

- Cada `decisao!` é um ponto de injeção: com `mutacao`, `WEBSONIC_MUTANTE=Rotina:nome` inverte essa decisão. Decisões de infraestrutura não usam `decisao!` e não são mutadas.
- `cargo fidelidade mutantes`: roda `cargo test -p sonic1-core -p sonic1-teste-nucleo -p sonic1-teste-descompressao` por mutante, com limite por mutante e orçamento total (padrão 60 s e 600 s; `--orcamento 0` é a rodada completa da auditoria). Classes: `detectado`, `tempo`, `sobrevivente` (reprova), `equivalente` (só com justificativa em `cobertura/equivalentes.txt`, que só o Marcelo edita) e `nao_rodado`.
- Evidência (07/11/2026): os 9 mutantes do piloto, rodados à mão com `WEBSONIC_MUTANTE=<id> cargo test -p sonic1-teste-descompressao`, foram todos detectados (2 testes falharam em cada). Testes do `mutantes.rs`: classificação, orçamento, e a suíte substituta que distingue passou, falhou e estourou o limite.

## Bugs conhecidos (5.8)

- `cargo xtask bugs-conhecidos` lê os `.asm` de texto (permitido) e extrai, por linha de condição que cita `FixBugs`, só arquivo, linha e o rótulo global mais próximo acima. **315 itens em 455 arquivos** (o requisito diz "cerca de 300"), `docs/bugs-conhecidos.md`, ids estáveis `BK-0001` a `BK-0315`, 0 sem rótulo. A coluna etapa é uma sugestão pela pasta.
- Interpretação: `FixBugs` marca o código que corrige. O núcleo reproduz o comportamento sem a correção (a ROM de referência). Cada item precisa de um teste com `BK_nnnn` no nome.
- `cargo fidelidade bugs-conhecidos` lê os nomes dos testes (`cargo test --workspace -- --list`) e diz quantos itens estão cobertos, por etapa. Hoje: 0 de 315, porque os trechos moram em rotinas das Etapas 6 a 12. A Etapa 14 usa `--exigir-todos`.

## Achados no caminho

- Pré-existente (Etapa 2): `.github/esperado-cargo-config.txt` não tem o atalho `fidelidade` que a 2.10 pôs em `.cargo/config.toml`, então `sh .github/checa-core.sh` falha com "difere de esperado-cargo-config.txt". O patch `etapa5-congelar.patch` atualiza o arquivo esperado.
- Armadilha do oráculo na Etapa 2 que se repetiu aqui: o jgenesis adia as escritas do 68000 (`pending_writes`); qualquer observação de escrita tem de ficar em `write_byte` e `write_word`, não onde a escrita é aplicada.
- `cargo test` de depuração é 8x mais lento que release para o diferencial; o bloco B roda em release.

## Estado das provas

| Item | Estado |
| --- | --- |
| 5.1 memória | provado contra o oráculo (ROM sintética) |
| 5.2 operações | provado contra o oráculo (ROM sintética, 1500 casos por operação e tamanho; bloco B repete com 6000 em release) |
| 5.3 e 5.4 | testes sintéticos; inventários reais pendentes (bloco A) |
| 5.5 | harness provado com rotinas sintéticas; pilotos com a ROM pendentes (bloco B) |
| 5.6 e 5.7 | provados no piloto com dados sintéticos; relatório com a ROM pendente (bloco B) |
| 5.8 | lista gerada; relatório roda |
| WebAssembly | não rodou nesta sessão (o host não tem o alvo); bloco B roda na imagem |
| Congelamento e CI | patches prontos (`etapa5-relatorio`, `etapa5-congelar`); aplicar e reconstruir a imagem |
