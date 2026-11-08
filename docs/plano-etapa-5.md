# Plano da Etapa 5: Núcleo base

Escrito em 08/11/2026. Aguarda aprovação do Marcelo. Nada foi implementado.
Fonte: `docs/requisitos.md` (sem conflito achado), `docs/estado.md`, `docs/backlog.md`, `docs/conhecimento/` (oráculo, corpus, oráculos, descompressão, vídeo), `docs/plano-fechamento-etapa-2.md`, `docs/inventario-hardware.md` e `docs/inventario-memoria.md`.

Decisões já tomadas pela IA, como o Marcelo pediu (nenhuma pergunta). Cada uma entra em `docs/conhecimento/nucleo.md` com o motivo.

## 0. Pré-condição e o que mudou desde a Etapa 4

- Etapa 2: o Marcelo declarou concluída em 08/11/2026 (reconstruiu a imagem). A 5.0 registra isso no `estado.md`. Limites herdados da Etapa 2: o segundo oráculo só concorda fora de D1a, D1b e D1c; o corpus não tem TAS aceito; os ocultos são um caso por zona.
- **Congelados que a Etapa 5 não edita** (mudança só por patch aprovado): `oracle/`, `oracle2/`, `crates/corpus/`, `crates/fidelidade/` (inclui o pacote `relatorio`), `refs/`, `hooks/`, `.github/`, `.devcontainer/`, `.cargo/`. Os testes, traces, exclusões e a lista de bugs injetados que já estão congelados não são tocados.
- **Onde cada coisa nova mora**, para respeitar o congelamento:
  - `crates/core` (livre): memória, operações, laço de quadro, pontos de checagem, cobertura e mutação. Lints do núcleo valem, sem `#[allow]`.
  - `crates/teste-nucleo` (novo, MIT, lints comuns): vetores públicos, propriedades e o harness de teste por rotina. Fora do núcleo, como o `teste-descompressao`, porque os lints do núcleo tornariam os testes ilegíveis.
  - `diferencial/` (novo, workspace próprio, GPLv3): único lugar que fala com o oráculo e com o núcleo ao mesmo tempo. Não está congelado, então a IA o escreve. Entra no congelamento no patch de fechamento.
  - `crates/fidelidade/relatorio` (congelado): os comandos que **medem** (cobertura, mutantes, bugs conhecidos) entram por patch, como na Etapa 2 (D-R5).
  - `xtask` (livre): só geradores de metadado (endereços, inventários, lista de FixBugs). Nada que decida aprovação.

## 1. Sub-etapas

Legenda de "roda": **IA** roda com dados sintéticos, no contêiner; **Marcelo** roda no host, com a ROM. Haverá **dois blocos de comandos** para o Marcelo: o bloco A (depois da 5.3) e o bloco B (fecho). Cada um vem pronto, num script só.

| # | Entrega | Prova (comando e resultado esperado) | Roda |
| --- | --- | --- | --- |
| 5.0 | Registro da Etapa 2 no `estado.md`. Patch 0004 do oráculo (seção 2.3): histograma de instruções do 68000 por classe e tamanho, mapa de acessos ao barramento fora da RAM com os acessos inválidos, e o arquivo de tempo `.s1tempo`. Leitores no `xtask`: `inventario-68000`, `inventario-barramento`. Pedido `docs/pedidos/etapa5-oraculo.patch` | IA: `git apply --check` numa cópia limpa; na cópia, `sh oracle/scripts/testa.sh` passa com testes novos em ROM sintética (histograma conta `ADD.B` e `BEQ` de uma ROM de 6 instruções; acesso a endereço sem dispositivo, a ROM em escrita e palavra em endereço ímpar aparecem nas contagens; `.s1tempo` tem um registro por quadro). `cargo test -p xtask` com histograma sintético | IA; Marcelo aplica o patch |
| 5.1 | Memória espelhada (seção 2): mapa, espelhos, leitura e escrita por tamanho e alinhamento, acesso inválido, saídas de hardware. `crates/core/src/memoria/` | IA: testes de unidade e de propriedade (espelho da RAM, alinhamento, big-endian, ROM somente leitura, janela do Z80 só com o barramento cedido). `cargo test --workspace` e `cargo test --target wasm32-wasip1 -p sonic1-core`. Diferencial: `diferencial/` roda a mesma sequência de acessos numa ROM sintética no oráculo e no núcleo e compara valores lidos e saídas | IA |
| 5.2 | Operações do 68000 com resultado e flags (seção 3). `crates/core/src/ops68k/` | IA: vetores derivados do manual público, propriedades, e o diferencial com o oráculo em ROM sintética (uma instrução por caso). Resultado esperado: `ops: N casos, 0 divergências` por operação e tamanho | IA (diferencial roda sem ROM real); Marcelo repete no bloco B |
| 5.3 | Laço de quadro com o tempo gravado (seção 4). `crates/core/src/quadro/`, `tempo` | IA: testes com `.s1tempo` sintético: quadro normal, quadro de lag, interrupção de linha, orçamento estourado. Teste de propriedade: ler e escrever `.s1tempo` volta igual. **Bloco A** (Marcelo): gera os `.s1tempo` do corpus e os inventários 68000 e de barramento | IA; Marcelo (bloco A) |
| 5.4 | Pontos de checagem de interrupção (seção 5), mais `docs/inventario-interrupcoes.md` (que rotinas o corpus interrompe, e onde) | IA: teste sintético com um laço principal de brinquedo e interrupção gravada em três posições. Marcelo (bloco A): o inventário com o corpus | IA; Marcelo (bloco A) |
| 5.5 | Harness de teste por rotina (seção 6), com metadado de registradores usados depois. Piloto: `KosDec`, `EniDec`, `NemDecToRAM` | IA: harness contra rotinas sintéticas de 68000 escritas por nós (acerta, e acusa registrador e byte de RAM errados plantados). Marcelo (bloco B): as três rotinas piloto com estado do corpus: `rotina: N casos, 0 divergências` | IA; Marcelo (bloco B) |
| 5.6 | Cobertura (seção 7): universo de desvios do 68000, executados no oráculo, mapa para as decisões do núcleo, relatório | IA: teste sintético de cobertura (ROM de 3 desvios; um nunca tomado aparece como "descoberto"). Marcelo (bloco B): `cobertura/desvios-68k.txt` e o relatório do corpus | IA; Marcelo (bloco B) |
| 5.7 | Bugs injetados (seção 8) | IA: gerador inverte cada decisão do piloto; um teste plantado que não acusa vira "sobrevivente" no relatório; um que acusa vira "detectado". Orçamento de tempo respeitado | IA; Marcelo (rodada completa fica para a auditoria) |
| 5.8 | Bugs conhecidos (seção 9): lista de verificação com os trechos marcados `FixBugs` | IA: `cargo xtask bugs-conhecidos` gera `docs/bugs-conhecidos.md` a partir dos `.asm` de texto (só rótulo, arquivo e número de linha). Relatório: 0 cobertos de N, cada pendente com etapa-alvo | IA |
| 5.9 | Fechamento: pacote `relatorio` com `cobertura`, `mutantes`, `bugs-conhecidos` e manifesto ampliado (por patch), CI e congelamento (por patch), base de conhecimento, `estado.md`, `backlog.md`, resumo. Bloco B, assinatura e `cargo fidelidade` | `cargo fidelidade` com `veredito: PASSOU` e as seções novas; testes wasm do núcleo | IA; Marcelo (bloco B) |

Ordem de trabalho: 5.0, 5.1, 5.2, 5.3, 5.4, **parar para o bloco A**, 5.5 (parte sintética) em paralelo à espera, 5.6, 5.7, 5.8, 5.9, **bloco B**.

## 2. Memória espelhada

### 2.1 Mapa do console (referência: Mega Drive japonês, modelo 1, NTSC)

Fontes públicas, consultadas em 08/11/2026: Genesis Programming, "68K Memory map" ([Wikibooks](https://en.wikibooks.org/wiki/Genesis_Programming/68K_Memory_map/)); Sega Genesis Technical Overview (cópia na [Scribd](https://www.scribd.com/document/492434018/GenesisTechnicalOverview)); comportamento do oráculo no código do jgenesis (`backend/genesis-core/src/bus.rs`, nomes de funções apenas). Larguras exatas dos registradores do Z80 divergem entre fontes: a regra é o oráculo, medido na 5.0 (patch 0004), e cada linha abaixo marcada "a medir" só vira fato com o resultado do bloco A.

| Faixa do 68000 | O que é | No núcleo | Evidência |
| --- | --- | --- | --- |
| `000000` a `3FFFFF` | ROM do cartucho | `Vec<u8>` carregada do arquivo do jogador. Leitura só. Fora do tamanho da ROM: espelha, a medir | bus.rs (cartucho); medir |
| `A00000` a `A0FFFF` | Espaço do Z80, só com o barramento cedido (BUSACK) | RAM do Z80 de 8 KiB (`A00000` a `A01FFF`, com espelho), mais a janela do YM2612 em `A04000` a `A04003`. Sem BUSACK: leitura devolve o barramento aberto, escrita é ignorada, a medir | bus.rs (bloco `A00000`); inventário de hardware (BUSREQ) |
| `A10000` a `A1001F` | E/S: versão, dados e controle das portas 1, 2 e EXT | Registrador de versão constante do console de referência (medido), portas de controle de 3 botões na porta 1, nada na 2. Endereço par espelha o ímpar | bus.rs (E/S); medir |
| `A11100` e `A11200` | BUSREQ e RESET do Z80 | Estado em dois bits. A leitura do BUSACK vem do tempo gravado | bus.rs; inventário |
| `A14000` | TMSS | Fora (console de referência sem TMSS). Acesso vira aviso | requisitos |
| `C00000` a `C0001F` | VDP (dados, controle, contador HV, PSG em `C00011`) | Não há VDP no núcleo. Escrita vira **evento de saída** (tipo VDP ou PSG, com tamanho e valor). Leitura de estado (`C00004`) vem do tempo gravado. Leitura do HV: aviso (o jogo não lê) | requisitos (tempo do processador); inventário |
| `E00000` a `FFFFFF` | RAM do 68000, 64 KiB, repetida a cada 64 KiB | `Box<[u8; 65536]>`. Endereço baixo de 16 bits | Wikibooks e Technical Overview dizem `FF0000`; o espelho vem do oráculo (bus.rs, bloco `E00000`) |
| qualquer outra | sem dispositivo | Leitura: barramento aberto. Escrita: ignorada. Ambos viram aviso e entram na contagem | bus.rs; 5.0 mede |

### 2.2 Como o núcleo expõe a memória

- API única, por tamanho: `ler_u8`, `ler_u16`, `ler_u32`, `escrever_u8`, `escrever_u16`, `escrever_u32`, todas com endereço de 32 bits, big-endian, devolvendo `Result<_, Acesso>`.
- **Alinhamento:** palavra ou longa em endereço ímpar é `Err(Acesso::EnderecoImpar)`. No 68000 isso é a exceção de erro de endereço (vetor 3); o oráculo a trata assim (`cpu/m68000-emu/src/core.rs`, constante do vetor). O núcleo não executa a exceção: devolve o erro, e o chamador (laço de quadro) registra aviso e para o caso. Como o jogo original não faz isso, qualquer ocorrência em teste é falha.
- **Acesso inválido, regra do oráculo:** barramento aberto na leitura, escrita ignorada. Isso é reproduzido, com aviso. No site vira só aviso (regra do requisitos: "um uso fora do inventário falha nos testes e, no site, só registra um aviso"). A contagem por região vai no relatório.
- **Saídas de hardware:** `Memoria::saidas()` devolve a fila de eventos do quadro (VDP dados e controle, PSG, YM pelo Z80, BUSREQ, RESET), no mesmo formato lógico dos eventos do trace (tipos 5 a 8), com a linha e a coluna vindas do tempo gravado. O renderizador da Etapa 4 e o som futuro só consomem essa fila.
- Sem registradores do 68000 como estado. Portas de controle são estado de **memória** (o que o hardware guarda), não de CPU.
- Nenhum dado da ROM no código: o conteúdo da ROM entra por `Memoria::nova(rom)` em tempo de execução. Constantes de endereço vêm do `xtask gerar-enderecos` (seção 10).

### 2.3 Patch 0004 do oráculo (pedido para o Marcelo)

Só observa, como o 0001 a 0003. Três saídas, todas só com nomes, endereços, bits e contagens:
1. `--instrucoes ARQ`: contagem por (classe da instrução, tamanho), onde a classe é o **nome da variante** do decodificador do próprio jgenesis (por exemplo `Add`, `Bcc`). Sem opcode, sem operandos, sem endereço de efeito. É o "inventário de operações do 68000".
2. `--barramento ARQ`: contagem por (região, leitura ou escrita, tamanho) para o que não é RAM, com linhas próprias para: sem dispositivo, escrita em ROM, endereço ímpar, instrução ilegal, divisão por zero.
3. `--tempo ARQ`: o arquivo `.s1tempo` (seção 4).

## 3. Operações do 68000

### 3.1 Lista fechada

A lista **não** é adivinhada: sai do histograma da 5.0 (`docs/inventario-68000.md`), que cobre o corpus inteiro (quatro demos, Labyrinth, seis partidas). Regra: o que o histograma lista entra; o que não lista não entra. Para a IA não esperar o bloco A, a 5.2 já implementa as **famílias aritméticas e lógicas completas** que o manual público define, por tamanho byte, word e long:

- soma e subtração: ADD, ADDX, ADDQ, SUB, SUBX, SUBQ, NEG, NEGX, CMP, CLR, TST; as versões em endereço (ADDA, SUBA, CMPA) sem flags;
- lógicas: AND, OR, EOR, NOT;
- deslocamentos e rotações: ASL, ASR, LSL, LSR, ROL, ROR, ROXL, ROXR (contagem em registrador, imediata e por 1);
- multiplicação e divisão: MULU, MULS, DIVU, DIVS (com estouro e divisão por zero);
- BCD: ABCD, SBCD, NBCD;
- bits: BTST, BSET, BCLR, BCHG;
- extensão e troca: EXT, SWAP;
- MOVE e MOVEQ, só pelo efeito nas flags;
- condições: as 16 de `Bcc`, `Scc` e `DBcc` (avaliação a partir do CCR).

Depois do bloco A, o relatório de operações marca cada uma como `usada` (no histograma) ou `extra`. Operação `usada` sem implementação faz o teste falhar. As extras ficam, porque são pequenas e já provadas.

### 3.2 Tabela de flags

Fonte: Motorola, M68000 Family Programmer's Reference Manual ([cópia NXP](https://nxp.com/docs/en/reference-manual/M68000PM-PART2.pdf), [cópia Emory](https://cs.emory.edu/~cheung/Courses/255/Syllabus/Old-7-M68000/Docs/68K-manual.pdf)). A tabela por instrução é transcrita para `docs/conhecimento/nucleo.md` (sem texto copiado: só `*` afetado, `-` intacto, `U` indefinido, `0` zera, `1` liga) na 5.2, depois de a IA abrir o PDF. **Esta sessão de planejamento ainda não leu a tabela por instrução**, só a definição de cada flag: X vale o "estende" (espelha C nas somas, subtrações e deslocamentos que o afetam), N é o bit mais alto do resultado, Z é resultado zero, V é estouro com sinal, C é vai-um ou empresta. As fórmulas booleanas de V e C por operação saem do manual e viram o **gabarito independente** dos testes (a implementação usa aritmética em tipo mais largo, o gabarito usa as fórmulas de bits; se os dois discordam, falha).

Flags "indefinidas" (`U`): o núcleo escolhe o que o oráculo produz, medido pelo diferencial, e o teste só exige igualdade com o oráculo (o manual não fixa o valor).

### 3.3 Como cada operação é provada

1. **Vetores do manual:** casos nas bordas (zero, máximo, mínimo com sinal, vai-um, estouro) calculados à mão a partir das fórmulas do manual, citados por página.
2. **Propriedades:** byte exaustivo (65536 pares por operação e por X); word e long por amostragem determinística (borda mais gerador congruencial fixo). Propriedades: gabarito de bits igual à implementação; ADD seguido de SUB devolve o original; Z e N consistentes com o resultado; deslocamento por zero não mexe em C (e X fica intacto); MUL e DIV contra aritmética de 64 bits.
3. **Diferencial com o oráculo:** `diferencial ops`. Para cada caso, monta uma **ROM sintética** com uma instrução (codificada por nós a partir do manual), roda no oráculo com `chamar_rotina` (registradores e SR de entrada definidos), lê registradores e SR de saída, e compara com a função do núcleo. Não usa a ROM do jogo, então a IA roda. O Marcelo repete no bloco B para o registro. Orçamento: até 200 mil casos por operação e tamanho, deterministas.
4. Divisão por zero: o oráculo toma a exceção (vetor 5); o núcleo devolve `Excecao::DivisaoPorZero`. O teste confere que as duas coisas acontecem juntas.

## 4. Laço de quadro e tempo gravado

### 4.1 O que o oráculo grava (`.s1tempo`)

Por quadro: número; `mclk` de início e de fim (orçamento = diferença, margem = quanto sobrou antes do VBlank); lista ordenada de eventos de tempo: interrupção tomada (nível, PC interrompido, `mclk`, linha, coluna), leitura do BUSACK (bit lido, `mclk`), leitura do estado da VDP (valor lido, `mclk`), espera acumulada e consumida do Z80. Formato binário de tamanho fixo, little-endian, magia e versão, definido em `crates/core/src/quadro/tempo.rs` (leitor e escritor, sem E/S). Os arquivos reais ficam em `privado/`, fora do git, como os traces.

### 4.2 O que o núcleo faz com ele (nível 1)

- `Quadro::executar(entrada, tempo_do_quadro)` roda os **passos** do jogo registrados (hoje só passos de brinquedo nos testes; as rotinas reais entram nas etapas 6 a 12) e entrega as interrupções nos **pontos de checagem** (seção 5) segundo o tempo gravado.
- **Quadro de lag:** o quadro é de lag quando a interrupção de VBlank gravada chega com o PC interrompido **fora** da rotina de espera do VBlank (nome tirado do mapa de rótulos na 5.3 e conferido pelo `v_vblank_routine`). Num quadro de lag o núcleo não lê o controle e **ignora** a linha de entrada daquele quadro, como no original. O contador de quadros que as rotinas leem sobe na hora gravada.
- **Estouro de orçamento:** se os passos pedem mais do que o orçamento gravado, o núcleo adia o resto para o quadro seguinte e marca o quadro como lag. Se o tempo gravado e os passos discordam além disso, o relatório mostra os dois números.
- **Interrupção de linha (HBlank):** entregue pelo mesmo mecanismo, no nível 4, nos quadros da Labyrinth.
- **Leituras de estado de hardware** (BUSACK, estado da VDP) devolvem o valor gravado, na ordem.
- **Fica para o nível 2 (Etapa 15):** prever o orçamento, o ponto das interrupções, o fim dos laços de espera e as esperas do Z80 sem o arquivo gravado; custo em ciclos; refresh, DMA e uso do barramento pelo Z80. A Etapa 5 não calcula ciclo nenhum.

## 5. Pontos de checagem de interrupção

- **Formato:** `Ponto { id, rotulo, endereco, posicao }`, tabela estática no núcleo. `rotulo` é a rotina do original onde a interrupção pode cair; `endereco` é o acesso de memória compartilhada que o ponto protege; `posicao` é `Antes` ou `Depois` desse acesso. Só nome e endereço, nada copiado.
- **Onde o núcleo os cruza:** o código Rust de cada rotina chama `quadro.checagem(&PONTO)` imediatamente antes ou depois do acesso a variável compartilhada (a lista vem de `docs/inventario-memoria.md`: 546 variáveis hoje, de 751). O laço de quadro é infraestrutura e não tem par no original (requisitos).
- **Regra de entrega:** uma interrupção gravada com PC na rotina R é entregue no primeiro ponto de R cruzado depois do início do trecho; se o PC gravado é menor ou igual ao `endereco` do ponto, entrega `Antes`, senão `Depois`. Interrupção em rotina **sem ponto** é entregue no fim do passo e entra em `Entrega::SemPonto`. Essa contagem é o inventário da lista de pontos que faltam e orienta as Etapas 6 a 12.
- **Como o comparador os confere com o trace:** `diferencial pontos` recebe o `.s1tempo`, o mapa de rótulos e o log de entregas do núcleo (`quadro nivel pc_gravado rotina ponto posicao`), e verifica, por entrega: mesma rotina do PC gravado, posição coerente com o PC, ordem igual à do trace e nenhuma interrupção perdida ou duplicada. Divergência para na primeira e diz quadro e interrupção.
- `docs/inventario-interrupcoes.md` (bloco A): por rotina, quantas interrupções de cada nível o corpus tem, e o menor e o maior deslocamento dentro dela. Só nomes e números.

## 6. Teste por rotina

### 6.1 Harness

`diferencial rotina NOME --estado ARQ.s1e --casos N` (Marcelo, com a ROM) e o equivalente sintético (IA). Para cada caso:
1. Carrega no oráculo o estado salvo de um quadro do corpus, define os registradores de entrada e os `pokes` de RAM, e chama a rotina isolada (`chamar_rotina`, com endereço sentinela e interrupções mascaradas).
2. Monta no núcleo uma `Memoria` com a mesma ROM e a mesma RAM de partida (copiada do oráculo), e chama a função Rust da rotina com os argumentos que o adaptador extrai dos registradores.
3. Compara **toda a RAM do 68000** (menos as áreas de exclusão aprovadas, como a pilha) e os **registradores que o chamador usa depois**.
4. Compara também a **sequência de decisões** (seção 7): para cada endereço de desvio citado pelo núcleo, a lista de resultados (tomado ou não) tem de ser igual à que o oráculo registrou nesse endereço.

### 6.2 Metadado por rotina (só nomes)

Arquivo `crates/teste-nucleo/rotinas/NOME.txt`, uma linha por campo:

```
rotulo: KosDec
entrada: a0 = origem
entrada: a1 = destino
usados_depois: a1
usados_depois: ccr
ignorados: d0 d1 d2 d3 d4 d5 a0
```

- Quem declara `usados_depois`: a IA, lendo os pontos de chamada nos `.asm` de texto (só para saber que registrador o chamador consulta). Registrador que não está nem em `usados_depois` nem em `ignorados` faz o metadado ser recusado.
- Prova de que o metadado não esconde nada: na primeira execução o harness compara **todos** os registradores e imprime os que diferem e não estão em `usados_depois`. Se um deles aparece como lido por algum chamador no corpus (o trace de desvios mostra a rotina chamadora), o relatório marca `risco de registrador sobrando`, caso do requisito "rotinas usam valores que sobraram em registradores, inclusive bugs".
- O núcleo **não** tem registradores como estado: o adaptador mora em `crates/teste-nucleo`, traduz registrador do oráculo em argumento Rust e devolve só o que a função Rust retorna.

### 6.3 Piloto

`KosDec`, `EniDec` e `NemDecToRAM`, que já têm o descompressor provado byte a byte (Etapa 3). A Etapa 5 acrescenta a função de rotina no núcleo (lê da `Memoria`, escreve na `Memoria`, devolve o destino final), as marcas de decisão e o teste por rotina. Fecha o backlog "Provar `NemDec` além de `NemDecToRAM`" só se o tempo permitir; senão fica registrado.

## 7. Cobertura

- **Universo de desvios do 68000:** `diferencial desvios-estaticos` (Marcelo, bloco B) decodifica, **sem executar**, o código de cada rotina do mapa de rótulos (do rótulo até o próximo), com o decodificador do jgenesis, e lista os endereços de desvio condicional e de laço com a classe (`BRA`, `Bcc`, `DBcc`, `BSR`, `JSR`, `JMP`, `RTS`...). Saída em `cobertura/desvios-68k.txt` (endereço e classe, metadado, vai para o git). Risco conhecido: dados no meio do código podem decodificar como desvio falso. Mitigação: só entra desvio cujo endereço aparece em algum trace; os demais ficam em `cobertura/duvidosos.txt` para a decisão de exclusão.
- **Executados:** vêm dos eventos `desvio68` do trace (PC, tomado ou não), do corpus inteiro e dos testes por rotina. União deterministica em `cobertura/executados-68k.txt` (endereço e dois contadores: tomado e não tomado).
- **Z80:** o formato é o mesmo (`desvio80` do trace), mas o universo só existe quando o programa do Z80 estiver no núcleo (Etapa 6). A Etapa 5 entrega o formato e o relatório com o Z80 marcado "universo pendente (Etapa 6)".
- **Mapa desvio para rotina e decisão do núcleo:** cada decisão do núcleo que espelha um desvio usa `decisao!(ENDERECO_DO_DESVIO, condicao)`. Com a feature `cobertura` ligada, o macro grava (endereço, resultado) num registro determinístico (`BTreeMap`). Sem a feature, vira só `condicao`. O endereço é o do desvio no original, tirado do universo acima, e a rotina vem do mapa de rótulos (o maior rótulo menor ou igual). Verificações do relatório:
  1. toda decisão citada pelo núcleo existe no universo (pega endereço errado de digitação);
  2. toda rotina marcada "implementada" no núcleo tem todos os seus desvios do universo citados por alguma decisão (pega desvio esquecido);
  3. desvio do original que nenhum caso fez tomar **e** não tomar: `descoberto`, para virar caso novo ou exclusão;
  4. decisão do núcleo que nenhum teste fez ir nos dois caminhos: `descoberta no núcleo`.
- **Formato do relatório:** texto, uma linha por rotina: `rotina desvios total ambos_caminhos so_um nenhum nucleo_cita nucleo_ambos`, mais as listas dos endereços descobertos. Só nomes, endereços e contagens. Exclusões (código morto, reset, PAL, outras regiões) são grupos aprovados pelo Marcelo; a IA propõe a lista em `cobertura/exclusoes-propostas.txt` e **nunca** a aplica sozinha.

## 8. Bugs injetados

- **Gerador:** o próprio `decisao!` é o ponto de injeção. Com a feature `mutacao`, um `Mutante { alvo: Option<Endereco> }` global (por thread) inverte o resultado da decisão cujo endereço é o alvo. Um mutante por decisão do núcleo que espelha o original. Decisões de infraestrutura (laço de quadro, pontos de checagem) usam `infra!` e **não** são mutadas (não têm par no original, requisitos).
- **Execução:** o comando `mutantes` (no `relatorio`, por patch) lista as decisões, roda a suíte do piloto (testes de `sonic1-core`, `teste-nucleo` e o diferencial sintético) uma vez por mutante com `WEBSONIC_MUTANTE=ENDERECO`, em ordem determinística, e classifica:
  - `detectado`: algum teste falhou;
  - `sobrevivente`: nada falhou. Vira pedido de teste novo ou de justificativa de equivalência;
  - `equivalente`: sobrevivente com justificativa **aprovada pelo Marcelo** (nunca pela IA);
  - `tempo`: estourou o limite por mutante (conta como detectado, mas é listado à parte, porque pode ser laço infinito).
- **Orçamento por etapa:** 10 minutos no total e 20 segundos por mutante, no contêiner. O que não coube sai como `nao_rodado`, com a contagem. A **rodada completa**, sem orçamento, é do Marcelo na auditoria.
- A lista de bugs injetados e as exclusões, depois de assinadas, são congeladas: a IA só acrescenta.

## 9. Bugs conhecidos

- **Fonte da lista:** o mapa de rótulos não marca `FixBugs`. Os `.asm` de texto marcam: 325 ocorrências de `FixBugs` em 88 arquivos (conferido em 08/11/2026 com `grep -c`, só contagem). A Etapa 5 lê esses arquivos **como texto** (permitido pelo requisitos) e extrai, por ocorrência, **só**: arquivo, número da linha e o rótulo global mais próximo acima. Nada de código, comentário ou tabela. `cargo xtask bugs-conhecidos` gera `docs/bugs-conhecidos.md`. Se uma ocorrência não tiver rótulo identificável, ela sai com a marca `sem-rotulo` e **não** é adivinhada.
- **Lista de verificação:** uma linha por ocorrência: id estável (`BK-0001`...), arquivo, linha, rótulo, etapa-alvo (pela pasta: objetos, física, telas...), estado (`pendente`, `coberto`, `excluido`), nome do caso de teste. O requisito é: pelo menos um caso por item.
- **O que a Etapa 5 entrega:** a lista, o formato do caso (um teste de rotina ou de trecho cujo nome cita o id `BK-nnnn`) e o relatório `cobertos X de N`, que lê os nomes dos testes do manifesto. Hoje será `0 de N`: os trechos moram em rotinas das Etapas 6 a 12, e cada etapa fecha os seus. A Etapa 14 exige `N de N`.
- **Cuidado de interpretação:** `FixBugs` marca o código que corrige. O núcleo reproduz o comportamento **sem** a correção (REV01 com `FixBugs` desligado, o que é a ROM de referência). O caso de teste prova que o núcleo se comporta como o ramo sem correção.
- Se a extração por texto se mostrar ambígua, o limite fica registrado e o bloco B traz um comando para o Marcelo (só nomes e linhas).

## 10. Estrutura de crates e módulos; o que roda em WebAssembly

```
crates/core/
  src/memoria/      mapa, espelhos, acesso, saídas de hardware, avisos
  src/ops68k/       operações e flags, por tamanho
  src/quadro/       laço de quadro, tempo gravado (.s1tempo), pontos de checagem
  src/cobertura/    decisao!, infra!, registro (features cobertura e mutacao)
  src/enderecos.rs  GERADO por `cargo xtask gerar-enderecos` a partir de refs/rotulos.txt e de uma lista de nomes; só nome e endereço
  src/rotinas/      funções do núcleo por rotina do original (piloto: KosDec, EniDec, NemDecToRAM), citam o rótulo
  src/descompressao/ (existe)
crates/teste-nucleo/  vetores públicos, propriedades, harness e adaptadores de rotina, metadado em rotinas/*.txt
diferencial/          GPLv3, workspace próprio: ops, rotina, pontos, desvios-estaticos
xtask/                gerar-enderecos, inventario-68000, inventario-barramento, inventario-interrupcoes, bugs-conhecidos
crates/fidelidade/relatorio/  (por patch) cobertura, mutantes, bugs-conhecidos
cobertura/            universo e executados (metadado), exclusões propostas
```

- **Lints do núcleo:** os mesmos. Sem float, aritmética com estouro explícito (`wrapping_*`, `checked_*`, `overflowing_*` escolhidos de propósito, como o `aritmetica.rs` já faz), sem `HashMap`/`HashSet` (só `BTreeMap`), sem cast que trunca (`u8::try_from`, `from`), sem `#[allow]`. As features `cobertura` e `mutacao` não mudam o comportamento com elas desligadas (teste compara o resultado dos dois modos).
- **WebAssembly:** `memoria`, `ops68k`, `quadro`, `cobertura` e `rotinas` rodam em `wasm32-wasip1` nos testes de unidade (`cargo test --target wasm32-wasip1 -p sonic1-core`), e o CI roda o mesmo. `crates/teste-nucleo` roda em nativo; as propriedades pesadas têm versão curta para wasm. `diferencial/` e o oráculo **nunca** entram no wasm nem no build do site. O resultado tem de ser igual no nativo e no wasm: um teste de propriedade grava o hash de uma sequência fixa de operações e o CI compara.

## 11. Riscos e o que NÃO será feito

Riscos:
- O decodificador do jgenesis pode ter `Instruction` sem nome estável: o histograma usa o nome de variante do `Debug`. Mitigação: teste do patch fixa os nomes das classes usadas.
- Dado no meio do código gera desvio falso no universo estático (seção 7). Mitigação: só vale desvio visto em trace; o resto vai para decisão do Marcelo.
- Flags `U` (indefinidas) dependem do oráculo. Mitigação: o teste exige igualdade ao jgenesis, e o segundo oráculo não cobre instruções. Fica registrado.
- Mapear decisão Rust para endereço de desvio é trabalho manual da IA a partir dos `.asm` de texto. Mitigação: a comparação de sequência de decisões (6.1, item 4) acusa mapeamento errado.
- O piloto pode não exercitar todas as famílias de operação: as outras só se provam em ROM sintética até as Etapas 6 a 12.
- Dois blocos de comandos para o Marcelo (A e B). O bloco B depende do A.
- A sessão roda no host se o Marcelo não abrir o contêiner (item de backlog); o plano assume o contêiner.

Não será feito:
- Renderizador e som (Etapas 4, 6 e 8). Rotinas específicas do jogo (Etapas 6 a 12), fora o piloto de descompressão.
- Calcular ciclos, prever interrupção ou tempo do Z80 (nível 2, Etapa 15).
- Interpretador, tradutor automático, registradores do 68000 ou do Z80 como estado, execução de código da ROM.
- Ler a ROM ou binários do disassembly. Rodar qualquer coisa com a ROM pela IA.
- Aprovar exclusões, equivalências de mutantes ou editar itens congelados.
- Comparar eventos e ciclos exatos com o segundo oráculo.

## 12. Itens que exigem mexer em arquivo congelado (patches prontos, entregues na sub-etapa indicada)

| Patch | Onde | Conteúdo | Sub-etapa |
| --- | --- | --- | --- |
| P5-1 `docs/pedidos/etapa5-oraculo.patch` | `oracle/` | patch 0004 do jgenesis, `SHA1` novo, flags `--instrucoes`, `--barramento`, `--tempo`, testes | 5.0 |
| P5-2 `docs/pedidos/etapa5-relatorio.patch` | `crates/fidelidade/relatorio/` | subcomandos `cobertura`, `mutantes`, `bugs-conhecidos`; manifesto cobrindo `diferencial/`, `crates/teste-nucleo`, `cobertura/` | 5.9 |
| P5-3 `docs/pedidos/etapa5-congelar.patch` | `.devcontainer/managed-settings.json`, `verifica-config.sh`, `.github/` | congela `diferencial/`; CI: `fmt`, `clippy` e `test` de `relatorio` e de `teste-nucleo`, testes wasm do núcleo | 5.9 |

Comando de aplicar, igual ao da Etapa 2: `patch -p1 < docs/pedidos/NOME.patch` e depois o comando de prova indicado dentro de cada bloco. Se o P5-3 mudar `.devcontainer/` ou `.github/`, a revisão final olha só essas mudanças; senão não reaudita sandbox, hooks nem CI.

## 13. Backlog gerado por este plano

Vai para `docs/backlog.md` na 5.0 (uma linha cada): universo de desvios do Z80 (Etapa 6); exclusões propostas aguardando o Marcelo; piloto sem `NemDec` da porta da VDP, se não couber; flags indefinidas medidas só no jgenesis; lista de pontos de checagem que faltam (do `inventario-interrupcoes.md`).

## 14. Critério de pronto da Etapa 5, ponto a ponto

| "Pronto quando" | Onde se prova |
| --- | --- |
| Memória espelhada | 5.1 |
| Operações com o mesmo resultado e flags do 68000, sem interpretar | 5.2 (diferencial e vetores) |
| Laço de quadro com o tempo gravado | 5.3 |
| Pontos de checagem de interrupção | 5.4 |
| Testes por rotina | 5.5 |
| Bugs injetados | 5.7 |
| Relatório de cobertura | 5.6 |
| Relatório de bugs conhecidos | 5.8 |
