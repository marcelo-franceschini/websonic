# Plano da Etapa 4: Renderizador fiel

Escrito em 06/11/2026. Aguarda aprovação do Marcelo. Nada foi implementado.
Fonte: `docs/requisitos.md` (sem conflito achado), `docs/estado.md`, `docs/backlog.md`, `docs/conhecimento/` (oraculo, corpus, oraculos) e o código do jgenesis no commit `386db855` (texto do emulador, não da ROM).

## 0. Estado de partida e ressalvas

- **Etapa 2 incompleta.** Faltam o gravador, o inventário de memória, os testes ocultos reais e a revisão. O segundo oráculo **não concorda** no quadro 52 (D1, `docs/conhecimento/oraculos.md`). O Marcelo pediu para seguir com a Etapa 4. A Etapa 3 está feita.
- **Corpus de vídeo hoje:** só `boot-demos` (7200 quadros: Sega, título, demos de GHZ e MZ, início da de SYZ). Ele não tem interrupção de linha (HBlank) nenhuma, então **não prova o meio do quadro**. A Labyrinth precisa de um caso novo (sub-etapa 4.7).
- **O trace não basta para o vídeo.** Tem as escritas nas portas da VDP com linha e coluna (tipo 5), mas não tem o conteúdo dos DMAs (R6) nem a imagem. Por isso a Etapa 4 pede dois patches no oráculo (seção 8).
- **Fato do oráculo que define a regra (código do jgenesis `386db855`, `backend/genesis-components/src/vdp.rs` e `vdp/render.rs`):** ele renderiza a linha e, quando um registrador ou a CRAM muda no meio da linha, **re-renderiza a linha a partir de um pixel calculado por uma heurística própria**. Os comentários do código dizem que o valor pode estar 16 pixels errado e que ele ignora re-renderização perto do fim da área ativa. Logo, "pixel a pixel no meio do quadro" significa **igualar o oráculo**, e o desempate pela documentação do hardware vem depois.

## 1. Sub-etapas

Legenda de "roda": **IA** com dados sintéticos; **Marcelo** no host, com a ROM.

| # | Entrega | Prova (comando e resultado esperado) | Roda |
| --- | --- | --- | --- |
| 4.0 | Pesquisa e base: `docs/conhecimento/video.md` com (a) o que o Sonic 1 usa da VDP e o que fica fora, (b) a regra de renderização do oráculo com arquivo e função citados, (c) fontes públicas citadas. Crate novo `crates/video` (`sonic1-video`, sem dependências) com a tabela de lints do núcleo. Patch do CI para o `lints-core.sh` e o `checa-core.sh` cobrirem o `video`. | IA: `cargo clippy -p sonic1-video -- -D warnings` com os lints do núcleo, e prova com violações plantadas (float, `+` sem checagem, cast, `HashMap`, `#[allow]`). Marcelo: aplica o patch do CI e roda os dois scripts na imagem. | IA e Marcelo |
| 4.1 | Patches do oráculo: **P6** (imagem de 320x224 em 9 bits por quadro, arquivo `.s1i` com o SHA-1 de cada quadro; subcomando `oraculo imagem --estado ARQ --quadro N --saida ARQ.s1p` que regenera o quadro N inteiro) e **P7** (fluxo de DMA, arquivo `.s1v`: para cada DMA de 68000 para VDP, o `quadro`, `seq` e as palavras transferidas). Extensões `.s1i`, `.s1p`, `.s1v` entram na lista de bloqueio do git. | IA: testes do oráculo com ROM sintética (`sh oracle/scripts/testa.sh`): imagem de uma ROM que pinta um quadro conhecido tem o hash calculado por um modelo simples independente; DMA sintético gera as palavras esperadas; duas execuções geram arquivos idênticos. Marcelo: aplica, roda `testa.sh` e `roda-oraculo.sh` (cmp idêntico). | IA e Marcelo |
| 4.2 | Núcleo da VDP: registradores, VRAM (64 KiB), CRAM (64 x 9 bits), VSRAM (40 x 10 bits), decodificação das portas de controle e de dados, incremento automático, DMA (68000 para VDP com fonte injetada, preenchimento, cópia). Sem desenho. | IA: vetores sintéticos por recurso e um modelo de referência simples (seção 4). Marcelo: `cargo xtask video-estado` repete as escritas do `boot-demos` e compara, **quadro a quadro, os hashes de VRAM, CRAM, VSRAM e registradores com o trace** (categorias que já existem). Esperado: `IGUAL em 7200 quadros`. Essa prova independe de pixels. | IA e Marcelo |
| 4.3 | Planos A e B: tamanhos 32/64 x 32/64, rolagem de plano inteiro, por célula de 8 linhas e por linha, rolagem vertical por coluna, flip, paleta, prioridade de tile. Fundo (backdrop). | IA: vetores sintéticos e propriedades. Marcelo: parte da comparação de imagem da 4.8 nos quadros sem sprite (título). | IA |
| 4.4 | Sprites: tabela e ligação, tamanhos, flip, **20 por linha e 80 por quadro**, limite de 320 pontos por linha, **máscara por x = 0**, prioridade contra os planos. | IA: vetores sintéticos de limite (21 sprites numa linha, 81 no quadro, máscara com e sem sprite anterior, ponto 321). Cruza com o comportamento do oráculo na 4.8. | IA |
| 4.5 | Janela, sombra e realce, entrelaçamento, H32 e V30: **fora** (seção 2). Qualquer uso fora do inventário registra aviso no site e **falha no teste**. | IA: teste que liga cada recurso fora do inventário e confere o aviso. | IA |
| 4.6 | Meio do quadro: a lista de escritas entra com `linha` e `coluna`. A regra de em qual pixel cada escrita passa a valer **replica a do oráculo** (re-renderização parcial, FIFO da VDP, janela de exclusão perto do fim da linha). Pontos coloridos da CRAM como camada à parte, ligada à regra do oráculo. | IA: vetores sintéticos de meio de linha (mudar a CRAM, o registrador 7 e a rolagem em colunas diferentes) calculados à mão. Marcelo: confere os vetores no oráculo com ROM sintética (as mesmas escritas, a imagem do oráculo tem o hash esperado). | IA e Marcelo |
| 4.7 | Caso da Labyrinth: `corpus/lz-agua.s1in`, entradas **escritas por mim** (caso feito para cobrir decisões do código, previsto no requisitos): leva do título ao seletor de fases e entra na Labyrinth ato 1. A sequência vem dos `.asm` em texto, só como nomes e lógica. `corpus/lz-agua.meta`. | Marcelo: `oraculo rodar` com `--perfil-a-cada 60` e o `xtask inventario`. Esperado: zona 1, modo 8, **eventos de interrupção nível 4 maiores que zero**. Se a sequência falhar, o perfil diz onde parou e eu corrijo as entradas (mesma rodada). | IA escreve, Marcelo roda |
| 4.8 | Comparação com o corpus: `cargo xtask video-comparar` (seção 5). | Marcelo: relatório com `IGUAL` em todos os quadros do `boot-demos` e do `lz-agua`, ou o primeiro quadro divergente com pixel e linha. | Marcelo |
| 4.9 | Desempenho em nativo e em wasm (seção 6) e fechamento: `estado.md`, base de conhecimento, backlog, e a sessão de revisão só do que a Etapa 4 mudou. | Marcelo: `cargo xtask video-bench`. | IA e Marcelo |

Cada sub-etapa fecha com commit local. Entre elas não paro. Só paro nos blocos de comandos do Marcelo: um depois da 4.1 (aplicar os patches), um depois da 4.7 (todas as provas com ROM de uma vez) e o do fechamento.

## 2. O que o Sonic 1 usa da VDP e o que fica fora

Tudo abaixo vem do `docs/inventario-hardware.md` (comando `cargo xtask inventario`, `boot-demos`, 7200 quadros, rodado em 06/11/2026). "Visto" vale só para esse corpus. O que o corpus não mostra (Labyrinth, special stage completa, créditos) fica **provisório** até a 4.8 rodar no corpus ampliado.

| Recurso | Evidência no inventário | Decisão |
| --- | --- | --- |
| Cor de 9 bits (registrador 0 = `0x04` em todas as 11 escritas) | visto | dentro |
| Modo de 40 colunas, sem entrelaçamento, sem sombra/realce (registrador 12 = `0x81`, 2 escritas, bits 0 e 3 desligados nos dois valores... só o `0x81`) | visto | **H40 dentro; sombra, realce e entrelaçamento fora** |
| Altura de 28 células (224 linhas): registrador 1 com bit 3 desligado nos quatro valores vistos (`0x04`, `0x14`, `0x34`, `0x74`) | visto | V28 dentro; V30 fora |
| Rolagem por linha: registrador 11 = `0x03` (6 escritas) e `0x00` (5 escritas) | visto | **rolagem de plano inteiro e por linha dentro**. Por célula e vertical por coluna: dentro por completude, sem evidência no corpus. Se o corpus ampliado não mostrar, sai da lista de provados e vai para o backlog |
| Tamanho de plano 64x32 (registrador 16 = `0x01`, 8 escritas) | visto | dentro. Outros tamanhos: implementar (são tabelas), marcar "não provado" |
| Janela (registradores 17 e 18 = 0 em todas as escritas) | visto como desligada | **fora**. Aviso se aparecer |
| Sprites (registrador 5 = `0x6c`, `0x7c`) | visto | dentro, com limites |
| DMA: 68000 para VDP 20617 vezes, preenchimento de VRAM 26 | visto | dentro. Cópia de VRAM: não visto, **fora** |
| Interrupção de linha: registrador 0 sem o bit 4, só nível 6 no corpus | **ausente** | dentro por requisito (água da Labyrinth, `docs/requisitos.md`), provada só pela 4.7 |

Sobre limites e máscara de sprite, as fontes públicas (consultadas em 06/11/2026) são:
- Sega Retro, [Sega Mega Drive/Sprites](https://segaretro.org/Sega_Mega_Drive/Sprites): 20 sprites por linha e 80 por quadro em H40, e a máscara por x = 0.
- [Sprite Masking and Overflow Test ROM](https://segaretro.org/Sprite_Masking_and_Overflow_Test_ROM): confere os mesmos limites e o teto de 320 pontos por linha em H40.
- Plutiedev, [VDP registers](https://plutiedev.com/vdp-registers): bits dos registradores 0, 1, 11 e 12. Essa página **não** documenta os limites nem a máscara.
- Há um post de fórum que dá números diferentes (32 por linha). Não adoto: vale a ROM de teste. A 4.0 confirma tudo isso também no comportamento do oráculo (vetor sintético no jgenesis) antes de chamar de provado.

Para o que o oráculo faz de diferente da documentação, a regra é a do requisitos: a documentação do hardware e os testes publicados decidem; sem evidência, vale o oráculo principal e a diferença fica registrada.

## 3. Formatos

**Entrada do renderizador** (alinhada ao trace da Etapa 1):
- Por quadro, uma lista ordenada de escritas: `{ linha: u16, coluna: u16, seq: u32, porta: Dados | Controle, valor: u16, tamanho: 1 | 2 }`. É o evento tipo 5 do trace (`a` = porta, `b` = valor, `c` = tamanho; `linha` e `coluna` do próprio evento).
- Para cada DMA de 68000 para VDP: as palavras transferidas, vindas do `.s1v` (P7), ligadas ao `quadro` e `seq` da escrita de controle que o disparou. No jogo final, o núcleo entrega essas palavras lendo a própria memória, pela mesma interface.
- Estado inicial: tudo zero (o do oráculo, R5).
- O núcleo da VDP **não lê ROM nem arquivo**. Recebe só escritas e palavras.

**Saída:** um quadro de 320 x 224 pontos, cada ponto um `u16` com a cor de 9 bits (`0b000_BBB_GGG_RRR_0`... guardado como valor de 0 a 511, formato de 3 bits por canal). Conversão para RGB acontece só na camada de apresentação.

**Hash da imagem:** SHA-1 dos 71680 valores em 2 bytes little-endian, na ordem de linhas. É o mesmo cálculo do oráculo (P6).

## 4. Estratégia de teste sem ROM

- **Vetores sintéticos por recurso**, escritos por mim, com o quadro esperado calculado à mão ou por um modelo de referência independente.
- **Modelo de referência simples**, em `crates/video/tests/referencia.rs`: um desenhista ponto a ponto, lento e sem otimização, escrito separado do renderizador (busca cada ponto em cada camada, sem cache de linha). Os dois têm de dar o mesmo quadro em todos os vetores e nos quadros gerados por propriedades.
- **Propriedades** (geradas com um gerador pseudoaleatório próprio e semente fixa): determinismo (mesma entrada, mesma saída); nunca entra em pânico com escritas aleatórias; nunca escreve fora de 320 x 224; DMA fora do limite não escreve fora da VRAM; as duas ordens de aplicação de escritas na mesma linha que a documentação diz iguais dão o mesmo quadro.
- **Lints do núcleo no `crates/video`**: sem ponto flutuante, estouro explícito, sem `HashMap`, sem casts que cortam, sem `#[allow]`. Cada função cita o rótulo da rotina do disassembly que reproduz quando houver par, ou a fonte pública do hardware quando não houver (a VDP é hardware, não rotina do jogo). Só o nome.
- **Sem dados da ROM** no crate nem nos testes.
- **Wasm:** os testes do `video` rodam também em `wasm32-wasip1` e os hashes dos vetores são os mesmos.

## 5. Comparação com o corpus

Comando (só o Marcelo, usa a ROM):

```
cargo xtask video-comparar --rom ROM --oraculo BIN --rotulos refs/rotulos.txt \
    --trace DIR/caso.a.s1t --video DIR/caso.s1v --imagens DIR/caso.s1i [--oraculo2 TSV] [--saida RELATORIO.tsv]
```

O comando repete as escritas do trace no renderizador, calcula o hash da imagem de cada quadro e compara com o `.s1i`.

**Relatório** (um arquivo TSV, só nomes, números e hashes):
- Por quadro: `quadro`, `hash_esperado`, `hash_obtido`, `veredito` (`IGUAL` ou `DIFERE`).
- Resumo: quadros comparados, quadros iguais, primeiro quadro que difere.
- Para o **primeiro** quadro que difere: a **linha** e a **coluna (x) do primeiro ponto diferente**, na ordem de leitura. Para achar o ponto, o comando regenera o quadro do oráculo com `oraculo imagem --estado ... --quadro N` (P6), compara ponto a ponto na memória e **imprime só as coordenadas e o número de pontos diferentes, nunca valores de cor**. O quadro regenerado fica em `privado/`, fora do git.
- Veredito geral na última linha: `video: IGUAL em N quadros` ou `video: DIFERE no quadro Q, linha L, x X`.

**Regra de desempate de cor entre os dois oráculos.** Só vale quando o renderizador diverge do oráculo principal em cor, ou os dois oráculos divergem entre si:
1. Comparar tudo em 9 bits (nunca em RGB).
2. Se os oráculos concordam entre si e o renderizador difere, o erro é do renderizador. Corrige-se o jogo.
3. Se os oráculos discordam, vale a documentação pública do hardware citada em `video.md` e as ROMs de teste publicadas.
4. Sem evidência, vale o oráculo principal e a diferença vai para `docs/conhecimento/oraculos.md` com id, caso, quadro, causa e decisão.
5. Como o segundo oráculo (Genesis Plus GX por libretro) entrega pixels pelo `video_refresh`, a 4.8 inclui `oraculo2 imagens` (subcomando novo do hospedeiro, MIT) que converte para 9 bits e gera o mesmo tipo de hash. Limite conhecido: a diferença D1 da RAM (quadro 52) está sem causa, então o segundo oráculo só serve de árbitro para cores depois de a D1 ser explicada ou isolada. Enquanto isso, a coluna `segundo_oraculo` do relatório fica `nao_comparado`.

## 6. Desempenho

- **Orçamento por quadro (320 x 224, pior caso: dois planos com rolagem por linha, 20 sprites por linha, 80 por quadro, 64 trocas de CRAM no meio do quadro):**
  - nativo: **no máximo 1 ms** por quadro, mediana de 200 quadros;
  - wasm (wasmtime, `wasm32-wasip1`): **no máximo 4 ms** por quadro, mediana de 200. É um quarto do orçamento de 16,6 ms, deixando o resto para o núcleo, o som e a apresentação.
- **Como medir:** `cargo xtask video-bench` roda o pior caso sintético e um replay do `boot-demos` (este só no host), nativo e dentro do wasmtime da imagem, e imprime mediana e máximo em microssegundos. A medida usa `Instant` **no xtask**, nunca no crate `video`.
- O orçamento é meta, não critério de pronto da etapa. Se estourar, o relatório registra o número e a sub-etapa 4.9 abre o item de otimização, sem mudar o resultado. O renderizador fica em software e na CPU, sem GPU, como manda o requisitos.

## 7. Riscos e o que NÃO será feito

Riscos:
- **A regra do meio da linha é heurística do oráculo.** Posso igualar o jgenesis e ainda divergir do hardware real. Mitigação: segundo oráculo e a documentação; a diferença é registrada, não escondida.
- **FIFO e tempo da escrita:** o momento real em que uma escrita da CPU chega à CRAM depende da FIFO da VDP e do tempo do 68000 (V5: o `mclk` é o do início da instrução). Pode haver diferença de poucos pontos que só o caso da Labyrinth revela.
- **Pontos coloridos da CRAM** dependem do oráculo (requisitos). Podem ser uma camada à parte e nem sempre comparáveis.
- **Corpus magro:** sem a 4.7, o meio do quadro não é provado. O caso sintético de entradas pode falhar na primeira tentativa.
- **Sem a Etapa 2 completa:** o gravador e o segundo oráculo estão pendentes.
- **Trace grande:** cada caso de 7200 quadros tem cerca de 2 GB. O `.s1v` e o `.s1i` crescem junto. Espaço livre hoje: 41 GB.

Não será feito:
- Som, lógica do jogo, tempo do 68000 e do Z80, driver de som e descompressão (outras etapas).
- Nenhuma leitura da ROM nem dos binários do disassembly pela IA. Nenhum dado, imagem ou trace da ROM no git.
- Editar arquivo congelado: tudo vai como patch.
- GPU, filtros, escala, apresentação no navegador (Etapa 7 em diante).
- Sombra e realce, janela, entrelaçamento, H32, V30, cópia de VRAM por DMA, modo 4 do Master System.
- Resolver a D1 do segundo oráculo (backlog; só entra se bloquear o desempate).

## 8. Itens congelados que a etapa precisa mexer

| Item | Congelado em | Quando | Entrega |
| --- | --- | --- | --- |
| P6 e P7 (imagem por quadro, regeneração do quadro, fluxo de DMA) | `oracle/` | 4.1 | `docs/pedidos/video-oraculo.patch` e o comando `patch -p1 < ...` no bloco da 4.1, mais `sh oracle/scripts/gera-patches.sh` se o patch tocar o jgenesis (o P6 toca; o P7 também) |
| Extensões `.s1i`, `.s1p`, `.s1v` na lista de bloqueio do git | `hooks/lib.sh`, `.gitignore`, `xtask/src/anti_rom.rs` | 4.1 | `docs/pedidos/extensoes-video.patch` (mesmo formato do P0) |
| `crates/video` entrar nos lints do núcleo no CI | `.github/lints-core.sh`, `.github/checa-core.sh`, `.github/deps-core.txt` | 4.0 | `docs/pedidos/ci-video.patch` e os dois scripts rodados na imagem. Sem este patch, o CI não cobre o `video`. Enquanto isso, a prova local é o `clippy` com a mesma tabela |
| Manifesto: testes novos aparecem "não assinados" | `refs/manifesto.txt` | 4.9 | `cargo xtask manifesto gerar` e a assinatura do Marcelo |

Sem mudança prevista em `managed-settings.json`, `Dockerfile`, `dominios.txt` nem `ci.yml`. A revisão final não reaudita sandbox, hooks nem CI, a menos que o patch `ci-video` mude os scripts de lint.

## 9. Decisões tomadas (sem perguntas)

| Decisão | Motivo |
| --- | --- |
| Crate novo `crates/video` (e não um módulo do `core`) | Segue a estrutura de pastas do requisitos. O custo é o patch do CI |
| Renderizador sem dependência alguma e sem ler arquivo | Roda igual em nativo e wasm; entra no build do site |
| Replicar a regra de meio de linha do jgenesis | O critério da etapa é "idêntico ao oráculo"; desempate depois |
| Caso da Labyrinth com entradas escritas por mim | O gravador (2.2) está pendente e as demos não têm HBlank. O requisitos prevê casos feitos para cobrir decisões |
| Comparação por hash de imagem em 9 bits, com detalhe sob demanda | Igual ao que o requisitos manda para a imagem e ao trace (guardar todos os pixels ocuparia gigabytes) |
| Segundo oráculo só como árbitro de cor, depois da D1 | A D1 não tem causa; usá-lo antes geraria falsos veredictos |
| Janela, S/H, entrelaçamento, H32 e V30 fora | O inventário não mostra uso; o requisitos manda implementar só o que o inventário lista |
