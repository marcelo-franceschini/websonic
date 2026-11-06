# Plano da Etapa 3: Descompressores

Escrito em 06/11/2026. Aguarda aprovação do Marcelo. Nada foi implementado.
Fonte: `docs/requisitos.md` (sem conflito achado), `docs/estado.md`, `docs/backlog.md`, `docs/conhecimento/oraculo.md`, `esqueleto.md`, `docs/plano-etapa-2.md`.
Critério (linha da Etapa 3): a arte, os blocos e os mapas saem idênticos, byte a byte, ao que o oráculo carrega.

## 0. Pré-condição

- Etapa 1: encerrada do lado da IA (1.1 a 1.9 feitas, congelamento commitado em c188a83). Falta só a sessão de revisão, que não bloqueia a Etapa 3.
- Etapa 2: **aberta**, plano aguardando aprovação. Isso não bloqueia. O que a Etapa 3 usaria dela:
  - **Item 2.0 (mapa de rótulos corrigido). É necessário.** O `refs/rotulos.txt` atual tem valores errados (R1). Confirmei só por leitura do arquivo (metadado): `v_ram_end` vale 0, `EntryPoint` vale `ce` e `GameProgram` vale `12c`, e a contagem de `ArtNem_`, `Blk256_` e `Eni_` é 0 ou quase 0. Rótulos de dados com letra hexadecimal no valor foram descartados. Sem a 2.0 a Etapa 3 não acha endereço de nenhum bloco. Ainda é inferência até a 2.0 rodar.
  - Nada do corpus, dos traces, do gravador, dos inventários, do manifesto assinado nem dos testes ocultos.
- Nada nesta sessão rodou sobre a ROM nem sobre binários do disassembly. Nenhum `.asm` foi lido.

## 1. Inventário dos formatos

Regra aplicada: só entra como "confirmado" o que uma fonte pública citável diz. As páginas do Sonic Retro e do Sega Retro (`info.sonicretro.org`, `segaretro.org`) bloquearam o acesso automático (proteção Anubis, "Access Denied"). Por isso **nenhuma especificação bit a bit foi confirmada nesta sessão**. Ver a pergunta Q1.

| Formato | O que comprime no Sonic 1 | Situação | Fonte |
| --- | --- | --- | --- |
| Nemesis | Arte (tiles de 8x8, 32 bytes cada) | Uso em arte: confirmado em fonte secundária. Detalhe do formato: **não confirmado** | s3unlocked (descreve como formato para gráficos do Mega Drive, entropia mais RLE em nibbles, "earlier Sonic games used it for main level art"). Resultado de busca do Sega Retro: Nemesis cobre a arte comprimida do Sonic 1 (não consegui abrir a página) |
| Kosinski | Mapeamentos de blocos de 256x256 | Uso: confirmado em fonte secundária (snippet da busca do Sega Retro). Detalhe do formato: **não confirmado** | s3unlocked (LZSS, descompressão rápida); clownlzss no GitHub (lista Kosinski como formato comum da Sega, sem especificação) |
| Enigma | Mapas de plano (telas estáticas) | Uso geral: confirmado em fonte secundária. **Se os blocos de 16x16 do Sonic 1 são Enigma ou Kosinski: não confirmado** | s3unlocked (mapas de plano, telas estáticas). Um post de fórum sugere Enigma para os 16x16, anedótico |
| Sem compressão | Parte da arte | Os nomes de pasta do disassembly (`artkos`, `artnem`, `artunc`, `map16`, `map256`, `sslayout`) sugerem Kosinski, Nemesis e sem compressão, mas o README não diz o conteúdo | github.com/sonicretro/s1disasm (lista de pastas). Inferência minha |
| Layouts da special stage | ? | Formato **desconhecido** | O rótulo `SS_1` existe no mapa; o formato dos dados não está em fonte consultada |
| Saxman | Nada no Sonic 1 | Pertence ao Sonic 2 (som) segundo o clownlzss | github.com/clownacy/clownlzss. Fora do escopo |

O que o mapa de rótulos já mostra (só nomes, em `refs/rotulos.txt`): as rotinas `NemDec`, `NemDecMain`, `NemDec_BuildCodeTable`, `NemDec_ProcessCompressedData`, `NemDecM_ModeNoXOR`, `EniDec`, `EniDec_Masks`, `EniDec_FetchByte` e `KosDec`; a fila de arte `AddPLC`, `RunPLC` e os ids `plcid_*`; 22 rótulos `Nem_*` (parte dos dados); `Blk16_LZ` e `Blk16_SBZ`. A lista está incompleta por causa do defeito do mapa. Os nomes dizem quais rotinas existem, não o que cada formato faz.

### Perguntas ao Marcelo (não consegui confirmar sem ler o disassembly ou a ROM)

- **Q1.** Preciso da especificação pública de cada formato. Opções: (a) você cola aqui o texto das páginas "Nemesis compression", "Enigma compression" e "Kosinski compression" do Sonic Retro (abrem no seu navegador); (b) você libera um domínio de documentação para a sessão; (c) eu uso o código-fonte do `mdcomp` (flamewing) ou do `clownlzss`, que é público, como descrição do formato. O repositório do `clownlzss` não traz especificação escrita, e o do `mdcomp` tem "Detail compression formats" como item pendente. Recomendo (a).
- **Q2.** Os blocos de 16x16 são Enigma ou Kosinski, e os layouts da special stage usam qual formato? Resolve-se com a especificação (Q1) mais uma lista de formato por rótulo, que só você pode confirmar rodando o oráculo (item 3.5, comando `classificar`).
- **Q3.** Posso ler os `.asm` de texto (`/mnt/disasm-texto`) só para saber qual registrador recebe origem e destino em `NemDec`, `EniDec` e `KosDec`, e quais parâmetros o chamador passa ao `EniDec`? Isso é necessário para chamar a rotina isolada no oráculo (seção 4). Nada seria copiado. A política N4 do `oraculo.md` permite ler para achar nomes de rótulos, e a convenção de registradores é um passo além disso. Por isso pergunto.

## 2. Sub-etapas

Legenda de "roda": **IA** com dados sintéticos; **Marcelo** no host, com a ROM, fora do Claude. Ordem: o mais simples e o que mais ativos cobre primeiro, depois o que depende de especificação.

| # | Entrega | Prova (comando e resultado esperado) | Roda |
| --- | --- | --- | --- |
| 3.0 | Pré-requisito: mapa de rótulos corrigido (item 2.0 da Etapa 2) | Marcelo: `sh oracle/scripts/monta-disasm.sh montar s1built.bin sonic.lst 16`. Esperado: `IDENTICA`, `variaveis` na casa das centenas, linha `v_plc_ptrnemcode` presente, e rótulos de dados (`Nem_`, `Blk16_`, `Eni_`, `Kos_`, `Art_`) com contagem maior que hoje | Marcelo (código do xtask: IA, já planejado na Etapa 2) |
| 3.1 | Kosinski em `crates/core/src/descompressao/kosinski.rs`, mais a base comum (tipo de erro, limite de saída, leitor de bits) em `descompressao/mod.rs`. Cita `KosDec` | IA: `cargo test -p sonic1-core` e `cargo test --target wasm32-wasip1 -p sonic1-core`. Esperado: ida e volta com compressor sintético, vetores de borda e propriedades da seção 3, todos verdes; clippy `-D warnings` sem `#[allow]` | IA |
| 3.2 | Enigma em `descompressao/enigma.rs`. Parâmetros de chamada (deslocamento de tile e bits de paleta) como argumentos explícitos. Cita `EniDec` | Igual à 3.1 | IA |
| 3.3 | Nemesis em `descompressao/nemesis.rs`: tabela de códigos, bits do cabeçalho (modo XOR), saída em bytes de tile. Cita `NemDec` | Igual à 3.1, mais caso de contagem de tiles no limite e tabela de códigos inválida | IA |
| 3.4 | Manifesto de endereços e relatório (seções 5 e 6): `refs/descompressao.txt` e `cargo xtask descompressao-relatorio`. A formatação do relatório fica em `xtask`, não em `crates/fidelidade` (congelado) | IA: `cargo test -p xtask` com ROM sintética de 3 blocos e um manifesto de teste. Esperado: relatório com `IGUAL` nos 3, `DIFERE` quando se adultera 1 byte da saída esperada, `ERRO` quando o rótulo não existe, e nenhuma linha com bytes | IA |
| 3.5 | Lado do oráculo: subcomando `oraculo descomprimir ROM ROTULO FORMATO` (seção 4), entregue como **patch P6** para o `oracle/` congelado, mais o comando do Marcelo. Inclui `classificar` (responde Q2) | IA: teste do runner com ROM sintética, usando blocos sintéticos comprimidos pelo compressor de teste (`sh oracle/scripts/testa.sh` depois do patch aplicado). Marcelo: `cargo xtask descompressao-prova --rom <ROM> --rotulos refs/rotulos.txt`, que roda os dois lados e imprime o relatório. Esperado: toda linha `IGUAL`, ou lista dos `DIFERE` | IA (sintético) e Marcelo (real) |
| 3.6 | Conferência no ponto natural de carga (seção 4, opção B): hashes das regiões de destino depois do carregamento de cada fase. Só se a 3.5 deixar dúvida ou se o Marcelo pedir | Marcelo: `cargo xtask descompressao-prova --ponto-natural`. Esperado: hash da região igual ao do descompressor | Marcelo |
| 3.7 | Fechamento: `docs/estado.md`, `docs/conhecimento/descompressao.md` (com a fonte de cada formato e o resultado do comando), `docs/backlog.md`, revisão só do que a Etapa 3 mudou | A revisão final olha só `crates/core/src/descompressao/`, `xtask/`, `refs/descompressao.txt`, `docs/` e o patch P6. Não reaudita sandbox, hooks nem CI | IA e sessão nova |

Corte padrão se estourar uma sessão: primeiro a 3.6, depois a versão retomável do Nemesis (decisão D3), por último o `classificar`. O "Pronto quando" exige 3.1 a 3.5.

## 3. Estratégia de teste sem a ROM

Tudo em `#[cfg(test)]`, sem dependência nova. Gerador de números pseudoaleatórios próprio (xorshift com semente fixa), para os testes serem determinísticos e rodarem igual em nativo e em WebAssembly.

- **Compressores de teste.** Um por formato, escritos por nós a partir da especificação (Q1), só para gerar entradas válidas. Podem ser simples (só literais, mais repetições gulosas). Não usam conteúdo da ROM.
- **Ida e volta.** Para entradas sintéticas (vazia, 1 byte, aleatória, só zeros, padrão repetido, tamanho máximo do formato), `descomprimir(comprimir(x)) == x`.
- **Vetores de borda.** Entrada vazia. Tamanho máximo. Repetição no comprimento mínimo e máximo, e com distância igual a 1 (sobreposição). Fim exatamente no limite do buffer. Terminador ausente. Cabeçalho truncado. Tabela de códigos do Nemesis incompleta ou com prefixo ambíguo. Contagem de tiles zero e no máximo.
- **Entrada truncada ou corrompida.** Cortar a entrada válida em cada tamanho possível e inverter um bit em cada posição: o resultado é `Err` ou uma saída dentro do limite. Nunca pânico.
- **Propriedades.** (1) Nunca panica, para qualquer entrada aleatória. (2) Nunca lê fora do buffer: o leitor devolve erro em vez de indexar além do fim, e o clippy do núcleo proíbe aritmética sem tratamento. (3) A saída nunca passa do `limite_saida` recebido. (4) `consumidos <= entrada.len()`. (5) Determinismo: duas chamadas dão os mesmos bytes e o mesmo `consumidos`.
- **Lints.** Sem `#[allow]`. Sem `HashMap` e `HashSet`. Sem float. Casts só com `try_from`.
- **WebAssembly.** Os mesmos testes em `wasm32-wasip1` (já funciona para o `core`).
- **Limite honesto.** Compressor e descompressor sintéticos escritos pela mesma sessão podem repetir o mesmo mal-entendido do formato (como aconteceu com a base do mapa de rótulos). Por isso o critério de verdade é o relatório contra o oráculo, rodado pelo Marcelo (3.5). Os testes sintéticos provam robustez, não fidelidade.

## 4. Como o oráculo expõe "o que ele carrega"

O que o oráculo tem hoje (Etapa 1): hashes por quadro de `ram68k`, `ram_z80`, `vram`, `cram`, `vsram`; eventos de VDP com valor e tamanho; parada em endereço, rótulo, quadro e relógio; **rotina isolada** (`chamar_rotina`, com registradores e RAM de partida, retorno por sentinela). O trace só guarda hashes, não bytes, e não tem hash de uma região.

**Opção A (principal): rotina isolada.** Para cada bloco, o oráculo parte do estado depois do boot (ou do estado salvo no quadro N), põe a origem (endereço do rótulo, vindo do mapa) e o destino (um buffer da RAM) nos registradores que a rotina espera, chama a rotina de descompressão e para na sentinela. Depois imprime: rótulo, formato, bytes lidos (diferença do registrador de origem), bytes escritos (diferença do registrador de destino) e o SHA-1 desses bytes. Vantagens: sem depender de ponto de captura no jogo, tamanho exato conhecido, byte a byte.
- Ponto de captura identificado por: **rótulo da rotina** (`NemDec`, `EniDec`, `KosDec`) mais **rótulo do bloco**. Não há quadro.
- Se a rotina de arte só escreve na porta de dados da VDP (sem variante para RAM), o hash sai da concatenação dos valores dos eventos `vdp` (tipo 5, porta de dados) emitidos durante a chamada. O evento já leva valor e tamanho, então não falta dado no trace.
- O estado de partida zerado (R5) não importa aqui: a rotina é função pura da entrada. A confirmar na prova.

**Opção B (conferência cruzada, 3.6): o que o jogo carrega de fato.** Parar o oráculo (`--parar-em` por rótulo, já existe) ao fim do carregamento de uma fase e do carregamento da arte, e imprimir o SHA-1 das regiões de destino: área dos blocos de 16x16 e de 256x256 na RAM e a VRAM nos intervalos de destino do PLC. Prova que o jogo usa o bloco e o endereço que o manifesto diz. Intervalos de destino: nomes e endereços vindos do mapa.

**O que falta nas ferramentas da Etapa 1** (tudo em `oracle/`, congelado, logo vira patch):
1. Subcomando `descomprimir` (A): chamar a rotina com origem e destino e imprimir os resumos. A mecânica de `chamar_rotina` já existe, falta o subcomando e a leitura dos registradores de saída. Depende da Q3.
2. Subcomando `hash-regiao` (B): SHA-1 de um intervalo de RAM, VRAM ou CRAM no momento da parada. Hoje só há hash do banco inteiro.
3. Subcomando `classificar` (Q2): rodar cada candidato de rotina sobre um rótulo de dados e dizer qual formato termina sem erro dentro do espaço até o próximo rótulo. Heurística, só para orientar; o manifesto final é decidido por você.

Nada disso exige ler a ROM pela IA: a IA escreve o código do subcomando e testa com ROM sintética. O Marcelo roda com a ROM.

## 5. De onde vêm endereço e tamanho, sem tabela copiada

- **Onde começa:** o endereço do rótulo do bloco, no `refs/rotulos.txt` corrigido (3.0). O núcleo recebe os bytes da ROM do usuário e o mapa de rótulos já é metadado no git (requisitos, seção de mapa de rótulos).
- **Quanto lê:** cada formato se autodelimita, e o descompressor devolve `consumidos`:
  - Nemesis: pelo cabeçalho (contagem de tiles) e pela tabela de códigos (a confirmar na Q1).
  - Enigma: por um código de fim.
  - Kosinski: por um código de fim.
  O tamanho nunca vem de tabela copiada. Como verificação cruzada, o relatório mostra `consumidos` e a distância até o próximo rótulo de dados; `consumidos` maior que a distância é erro (`ERRO`), e menor é aceito (pode haver alinhamento).
- **Descobrir endereços das tabelas do jogo (PLC, listas de fases):** são dados da ROM. O descompressor em si não precisa delas. Quem monta a lista de blocos de uma fase (Etapa 5 em diante) as lerá da ROM em tempo de execução. Na Etapa 3 o manifesto nomeia os blocos diretamente.
- **Manifesto de endereços `refs/descompressao.txt`** (texto, uma linha por bloco, campos separados por TAB, só metadado):

```text
# bloco<TAB>formato<TAB>rotulo_de_dados<TAB>rotulo_da_rotina
nome_curto	nemesis|enigma|kosinski	<rotulo no mapa>	NemDec|EniDec|KosDec
```

  Mais, só para Enigma, uma coluna de parâmetros numéricos de chamada (deslocamento de tile, bits de paleta), que são parâmetros do chamador e não dados da ROM. Não há endereço no manifesto: ele é resolvido pelo mapa. Isso evita copiar endereço e quebra com clareza se o mapa mudar (`ERRO`: rótulo não existe). Um teste confere que cada linha só tem nomes (mesmo filtro que `Rotulos::analisar`).
- **Quais blocos entram:** todos os rótulos que o mapa corrigido classificar como dados de um dos três formatos e que o oráculo confirmar com `classificar`. A lista é gerada pelo `xtask` e revisada pelo Marcelo. Não copio a lista do disassembly.

## 6. Relatório de comparação

Saída do `cargo xtask descompressao-relatorio` e do `descompressao-prova`. TSV, uma linha por bloco, **nunca bytes**:

```text
bloco  formato  rotulo  endereco_origem  consumidos  tamanho_saida  sha1_esperado  sha1_obtido  veredito
```

- `veredito`: `IGUAL`, `DIFERE`, `ERRO` (descompressor devolveu erro, rótulo ausente ou `consumidos` fora da faixa) ou `SEM_ORACULO` (bloco no manifesto sem resposta do oráculo).
- Em `DIFERE`: acrescenta `primeiro_offset_diferente` e `tamanho_oraculo`. Só números.
- Última linha: contagens por veredito e o SHA-1 do próprio relatório. O comando sai com código diferente de zero se houver qualquer linha que não seja `IGUAL`.
- Regra para o que o Marcelo cola de volta: o relatório inteiro serve, pois só tem rótulos, endereços, tamanhos e hashes.
- Teste que falha se corrige no jogo (no descompressor), nunca no teste nem no manifesto.

## 7. Decisões que dependem do Marcelo

| # | Decisão | Recomendação |
| --- | --- | --- |
| D1 | Como obter a especificação de cada formato (Q1) | (a) você cola o texto das três páginas do Sonic Retro |
| D2 | Posso ler os `.asm` de texto só para a convenção de registradores (Q3)? | Sim, só para isso, sem copiar nada. Alternativa: você me diz os registradores de origem e destino de cada rotina |
| D3 | Interface do Nemesis. Os requisitos dizem que a descompressão da arte roda no núcleo nos mesmos passos do original, porque o progresso dela (por exemplo `v_plc_ptrnemcode`) fica na memória comparada. Função pura agora, ou já uma máquina de passos retomável? | Kosinski e Enigma como funções puras. Nemesis como estrutura com `executar_ate(n_tiles)` que a função pura chama até o fim. Custa pouco agora e evita reescrever na Etapa 5 ou 7. Os passos exatos do original (quantos tiles por quadro, onde para) **não** entram na Etapa 3, pois dependem da fila de arte, que é do núcleo base |
| D4 | Onde fica o código | `crates/core/src/descompressao/` (requisitos: a descompressão da arte roda no núcleo). A prova e o relatório ficam em `xtask` |
| D5 | Hash | SHA-1, como o resto do projeto na Etapa 1 |
| D6 | Fazer o item 2.0 da Etapa 2 antes, mesmo com a Etapa 2 sem aprovação | Sim, só a 2.0 (corrigir o mapa). Sem ela a Etapa 3 trava |
| D7 | O segundo oráculo entra na Etapa 3? | Não. A saída dos descompressores é função pura da entrada. A confirmação independente vem da especificação pública (Q1). Fica no backlog para a Etapa 2 |

## 8. Riscos e o que não será feito

Riscos:
- **Especificação não confirmada** (fonte primária bloqueada). Mitigação: Q1. Sem ela, 3.1 a 3.3 não começam.
- **Mal-entendido repetido** no compressor e no descompressor de teste. Mitigação: o relatório contra o oráculo (3.5) é o juiz.
- **Rotina do oráculo exige contexto** (RAM ou registradores preparados pelo chamador, como parâmetros do Enigma). Mitigação: Q3 e estado de partida salvo após o boot.
- **Nemesis na VDP:** capturar pelos eventos de VDP depende de a rotina escrever só pela porta de dados. A confirmar na 3.5.
- **Mapa de rótulos incompleto** mesmo depois da 2.0 (dados sem rótulo). Mitigação: `ERRO` claro no relatório.
- **Os dados que o jogo realmente carrega** podem ter blocos que o jogo usa de formas que o oráculo isolado não exerce. A opção B (3.6) cobre.
- A etapa pode passar de uma sessão (corte na seção 2).

Não será feito:
- Nenhuma leitura da ROM nem de binários do disassembly pela IA. Nenhum `.asm` lido sem sua resposta à D2.
- Renderização, paletas, tiles na tela e DMA (Etapa 4).
- Fila de arte do jogo, ordem e passos de carga por quadro, `v_plc_*` como memória (Etapa 5 em diante).
- Carregar fases, layouts de nível, special stage além do formato dos dados.
- Compressão no jogo final. Compressores existem só em testes.
- Saxman, Comper e outros formatos que o Sonic 1 não usa.
- Segundo oráculo, corpus, traces, inventários, manifesto assinado.
- Qualquer dado, tabela ou offset da ROM no código, nos docs ou no git.

## 9. Itens congelados que exigem sua mão

- **P6 (3.5):** `oracle/` (congelado). Subcomandos `descomprimir`, `hash-regiao` e `classificar`. O patch será entregue na 3.5, depois que eu ler `oracle/runner/src/oraculo.rs` e a convenção de registradores (D2). Não o escrevo agora porque ele depende de decisões ainda abertas. Aplicar com `patch -p1`, atualizar `oracle/patches/SHA1` pelo `gera-patches.sh`, e rodar `sh oracle/scripts/testa.sh`.
- **Nenhum outro.** Sem mudança em `managed-settings.json`, `Dockerfile`, `.github/`, `hooks/`, `.gitignore`, `crates/fidelidade` ou `refs/` de verificação. O `refs/descompressao.txt` é arquivo novo, de texto e sem hash assinado (o manifesto assinado é da Etapa 2). Logo a revisão final não reaudita sandbox, hooks nem CI.
- Dependência da Etapa 2: patch **P0** (extensões de trace) só importa se a Etapa 3 gravar traces. Não grava.

## 10. Fontes consultadas

- Sonic Retro e Sega Retro (páginas de Nemesis, Enigma e Kosinski): **bloqueadas** (Anubis, "Access Denied"), só vistos os resultados da busca.
- s3unlocked, "Mega Drive compression formats" (2017): https://s3unlocked.blogspot.com/2017/06/mega-drive-compression-formats.html
- clownlzss: https://github.com/clownacy/clownlzss
- mdcomp (flamewing): https://github.com/flamewing/mdcomp
- s1disasm (README e nomes de pasta): https://github.com/sonicretro/s1disasm

## 11. Backlog gerado

- Confirmar a especificação dos três formatos em fonte primária e registrar em `docs/conhecimento/descompressao.md`. Dono: Marcelo (cola o texto) e IA. Etapa-alvo: 3.
- Comparar a saída dos descompressores com o segundo oráculo. Dono: IA. Etapa-alvo: 2 (depois de 2.5).
- Passos exatos do Nemesis por quadro (fila de arte). Dono: IA. Etapa-alvo: 5.
