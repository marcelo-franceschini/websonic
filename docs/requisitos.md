# Reconstrução do Sonic 1: Requisitos e Objetivos

Nov 2, 2026 · @Marcelo Majer Franceschini

## Visão e objetivos

O projeto reconstrói o Sonic the Hedgehog (1991) do Mega Drive como um jogo novo em Rust, que se comporta exatamente como o original. É um projeto pessoal de hobby, feito por diversão e para aprender: Marcelo quer entender o pipeline inteiro e aprender Rust lendo e revisando o código que a IA escreve. O projeto não tem prazo. O repositório e o site são públicos só por necessidade técnica (minutos grátis e GitHub Pages), sem divulgação e sem nunca conter a ROM.

- O jogo final não executa o código original. Ele lê a ROM que o jogador fornece, só no navegador, e nada dela entra no repositório nem no site. Dela vêm gráficos, paletas, fases, colisão, posições de objetos, música e tabelas, além do custo em ciclos do código no nível 2. Os endereços vêm do mapa de rótulos, que fica no git.
- A fidelidade é provada por testes automáticos contra a ROM, rodando em emuladores sem tela usados só como ferramenta de teste.
- As melhorias só vêm depois da fidelidade provada, e nenhuma pode quebrá-la.
- **Visão de longo prazo:** reconstruir todos os Sonics de Mega Drive, incluindo o Sonic CD. Este documento cobre só o Sonic 1. Por isso as peças genéricas do console (memória, vídeo, som) ficam separadas do que é específico do jogo, sem aumentar o escopo atual.

## Escopo

O escopo é só o Sonic the Hedgehog (1991). Os outros Sonics de Mega Drive ficam fora deste documento e viram uma decisão nova quando ele estiver pronto.

- **Jogo completo:** jogável do começo ao fim.
- **Inclui:** todas as zonas, chefes, special stages e a Final Zone, além de telas, modo demo, finais, créditos, continues e os quatro códigos secretos do console japonês: seletor de fases, câmera lenta, modo de depuração e créditos japoneses ocultos.

**Depois do Sonic 1:** Sonic 2 e Sonic 3 & Knuckles são os candidatos mais viáveis, porque usam o mesmo hardware e reaproveitam quase tudo. Sonic CD e Knuckles' Chaotix exigem hardware extra e seriam projetos de outra escala.

**Fora do escopo:**

- As revisões REV00 e REVXB, a versão da Sonic Mega Collection.
- Console PAL a 50 Hz.
- O botão reset do console. O código que trata o reset vira exclusão aprovada.
- Outros jogos, até o Sonic 1 estar pronto.
- Touch e celular.
- Divulgação e distribuição.
- Tradução de textos: o texto do jogo é quase todo em inglês. Os créditos ocultos, em japonês, são uma imagem e ficam como no original.

## Fidelidade

Fiel significa produzir o mesmo estado de jogo e as mesmas saídas que a ROM original, quadro a quadro, para as mesmas entradas de controle.

- **Oráculo:** a ROM rodando num emulador de alta precisão, sem tela, só nos testes. Um segundo emulador independente confere o primeiro.
- **Comparação a cada quadro:** a memória do 68000 e a do Z80, os comandos de vídeo, com a linha da tela em que aconteceram, os comandos de som e a imagem. A imagem é comparada na cor de 9 bits do console, na área ativa de 320x224, e não no RGB final, porque cada emulador converte as cores de um jeito.
- **Tempo das escritas de som:** o momento de cada escrita dentro do quadro também é tempo do processador. O nível 1 compara o conteúdo e a ordem das escritas em cada quadro; o nível 2 compara também o ciclo de cada uma.
- **Som sintetizado:** comparado na saída digital de cada chip, na taxa dele, antes de reamostrar. O FM roda a cerca de 53 kHz, e o PSG tem outra taxa. A bateria e a voz "SEGA" saem pelo DAC do FM e entram na mesma comparação. Só a parte analógica do modelo 1, como o filtro e o amplificador, é medida e aparece no relatório como aproximada, nunca como verificada.
- **Bugs e slowdown originais:** ficam. Os bugs conhecidos, cerca de 300 trechos marcados com `FixBugs` no disassembly, viram lista de verificação: cada um tem pelo menos um caso de teste que prova que o núcleo o reproduz. Correções e a opção de desligar o slowdown entram depois, como toggles.
- **Limite:** a confiança é muito alta no que foi testado, mas não é prova formal.

**Console de referência:** Mega Drive japonês, modelo 1, NTSC, sem a tela de licença (TMSS), com o chip de som YM2612, um controle de 3 botões na porta 1 e nenhum na porta 2. O oráculo usa sempre essa configuração. Sem ela, os traces não são reproduzíveis.

**O que o console japonês muda na REV01**, confirmado no código: vida extra a cada 50.000 pontos, o "TM" escondido na tela da Sega e no título, outro código para o seletor de fases e a escolha entre quatro códigos secretos pelo número de toques em C. Em consoles de outras regiões, os créditos japoneses ocultos estão na ROM, mas não podem ser ativados.

**ROM de referência:** REV01, a mesma para todas as regiões, com SHA-1 `69e102855d4389c3fd1a8f3dc7d193f8eee5fe5b` e MD5 `09dadb5071eb35050067a32462e39c5f`. Duas fontes independentes conferem: o TASVideos e o verificador do próprio disassembly.

### Tempo do processador

O original depende do tempo do processador em mais pontos do que o lag. O jogo nunca lê o contador de linha do chip de vídeo e só lê o estado dele para esperar o fim de um preenchimento. Por isso o tempo só entra pelos quatro casos abaixo, todos confirmados no código.

- **Quadro de lag:** a interrupção de VBlank chega no meio do laço principal, roda o driver de som e soma 1 a um contador que mais de dez objetos e rotinas leem. Quando o Sonic perde anéis, cada anel usa esse contador para decidir se testa o chão naquele quadro. Se a interrupção chega no meio da lista, parte dos anéis vê um valor e parte vê outro.
- **Água da Labyrinth:** com a água a menos de 96 pixels do topo, o VBlank adia o vídeo e o som para a interrupção da linha da água, quando a lógica do quadro seguinte já está rodando. Essa interrupção grava a paleta inteira no meio da imagem, o que no console real gera pontos coloridos na tela.
- **Laços de espera:** a voz "SEGA" é um laço fixo de cerca de 2 segundos, e a verificação da ROM no boot também gasta um tempo fixo. A cada troca de tela e na special stage, o jogo limpa a memória de vídeo por DMA e espera o chip de vídeo terminar, o que depende do tempo do chip.
- **Z80:** a cada quadro, o driver de som para o Z80 e espera ele soltar o barramento. Se a bateria estiver no meio de uma escrita, o driver solta o Z80 e tenta de novo, então essa espera depende do ponto em que o laço do Z80 está. Durante a voz "SEGA", o Z80 lê a voz direto da ROM e ocupa o barramento do 68000. Essas paradas também interrompem a bateria a cada quadro, e isso faz parte do som que ela tem no console.

Por isso o trace grava o tempo: os quadros de lag, o ponto exato de cada interrupção (endereço e ciclo), o fim de cada laço de espera, as esperas pelo Z80 e a margem de ciclos de cada quadro.

### Corpus

O corpus reúne as entradas de teste.

- As demos internas do jogo.
- Partidas gravadas por Marcelo com o gravador do projeto, que roda fora do Docker porque precisa de janela, controle e som: o oráculo com janela, ou um emulador que exporte as entradas. A escolha fica para a etapa 2.
- TAS convertidos para o formato do projeto e conferidos contra a REV01.
- Casos feitos para cobrir decisões do código e bugs conhecidos.

As entradas são só botões por quadro e ficam no git, em `corpus/`. Traces nunca entram no git. Num quadro de lag, o jogo não lê o controle, e a entrada daquele quadro é ignorada, como no original. Os testes ocultos seguem o mesmo formato, mas ficam fora do repositório.

### Oráculos

O oráculo principal é o jgenesis, em Rust, com licença GPLv3. Por isso os oráculos ficam num workspace separado e nunca entram no build do site. O README do jgenesis não cita modo sem tela, depurador nem traces, então a etapa 1 provavelmente exige uma versão modificada. Ele só fica como principal se cumprir o critério da etapa 1, na tabela de Etapas, e concordar com o segundo oráculo nas demos, na etapa 2. Os candidatos a segundo oráculo são o BlastEm e o Genesis Plus GX, que é a referência do OpenGGF.

Quando os oráculos discordam, decidem a documentação do hardware e os testes publicados. O Nuked-MD, emulador baseado em fotos dos chips e próximo do modelo 1 VA7, é candidato a árbitro em trechos curtos, mas a velocidade dele não está documentada. Sem evidência, vale o oráculo principal, e a diferença fica registrada.

### Estratégia de testes

- **Níveis de teste:** rápido, a cada mudança, com testes por rotina e casos curtos; completo, no fim de cada etapa; e auditoria, por zona, com os testes ocultos.
- **Testes por rotina:** rodam uma rotina isolada no oráculo e no núcleo, a partir do mesmo estado, e comparam a memória e os registradores que o chamador usa depois.
- **Trace:** hashes por quadro e estados completos a intervalos. O detalhe de qualquer quadro é regenerado sob demanda, porque tudo é determinístico. Guardar todos os pixels de horas de jogo ocuparia gigabytes.
- **Cobertura:** medida no oráculo, que registra cada desvio do código original, nos dois processadores, executado pelo corpus e pelos testes por rotina.
- **Bugs injetados:** gerados no núcleo, invertendo cada decisão, com orçamento de tempo por etapa. A rodada completa fica para a auditoria.
- **Comparador:** fica em `crates/fidelidade/`, junto com o formato do trace e o relatório.
- **Auditoria:** roda num contêiner separado, da mesma imagem, sem iniciar o Claude Code e com os testes ocultos montados. Só Marcelo roda a auditoria.

## Critério de pronto

O Sonic 1 está pronto quando um relatório reproduzível, gerado por um único comando, mostra tudo abaixo. Não há metas percentuais: o tamanho da tarefa vem da própria ROM.

1. Zero divergências em todos os casos de teste.
2. Toda decisão do código original, do 68000 e do Z80, foi testada nos dois caminhos, ou excluída com justificativa aprovada por Marcelo. As exclusões são aprovadas em grupos, como código morto, ramos impossíveis, o tratamento do botão reset e os trechos para outras regiões e para PAL, que nunca rodam no console de referência. As áreas da memória ignoradas na comparação, como a pilha, seguem a mesma aprovação.
3. A suíte detecta todos os bugs injetados de propósito. Um bug que não muda o comportamento do jogo é marcado como equivalente, com justificativa aprovada.
4. Os testes ocultos, nunca vistos pela IA, também passam.
5. Os dois emuladores de referência concordam, ou cada diferença tem decisão registrada.
6. O som sintetizado é idêntico, amostra a amostra, aos núcleos de referência de FM e de PSG, alimentados com as mesmas escritas nos mesmos instantes. Só a parte analógica fica de fora, marcada como aproximada.
7. Cada bug conhecido do original tem pelo menos um caso de teste que prova que o núcleo o reproduz.
8. ROM, emuladores, casos de teste e o próprio relatório são identificados por hash.

**Nível 1:** zero divergências usando o tempo gravado do oráculo: quadros de lag, ponto de cada interrupção, fim de cada laço de espera e esperas pelo Z80. **Nível 2:** o núcleo prevê esse tempo sozinho, inclusive o do Z80 e o ciclo de cada escrita de som, também sem divergências. O nível 2 é o 100% exigido; se ele travar, a decisão sobre as fases futuras é revista.

## Arquitetura

O jogo tem duas partes: um núcleo fiel e uma camada moderna.

- **Núcleo fiel:** espelha o original, com a mesma memória, os mesmos objetos e a mesma ordem de execução. Essa semelhança permite provar a fidelidade, reproduzir glitches e prever o tempo do processador. O espelhamento vale para a memória e as rotinas, não para o estilo: cada rotina é escrita em Rust legível, com nomes e tipos.
- **Decisões ligadas ao original:** toda decisão da lógica do jogo no núcleo corresponde a uma decisão do original, no mesmo endereço, exceto os toggles. O laço de quadro e os pontos de checagem são infraestrutura e não têm par no original. Assim a cobertura, os bugs injetados e o custo em ciclos ficam ligados um a um.
- **Camada moderna:** vídeo, som, entrada e melhorias. Ela só recebe o que o núcleo produz e nunca altera o jogo.
- **Sem emulação:** nenhuma instrução do código original é executada, interpretada ou traduzida automaticamente, nem do 68000 nem do Z80. O núcleo não tem os registradores desses processadores como estado. Vídeo e som são sistemas próprios, guiados pelos mesmos comandos que o jogo mandava ao hardware, e implementam só o que o inventário de hardware lista. Um uso fora do inventário falha nos testes e, no site, só registra um aviso. Depois da fidelidade, vídeo e som ganham modos refinados, e o modo fiel continua como referência.
- **Opções que mudam o jogo:** correções de bugs e desligar o slowdown ficam no núcleo, desligadas por padrão. O relatório de fidelidade sempre roda com tudo desligado.
- **Dentro do núcleo:** o driver de som do 68000, o programa do Z80 (bateria e voz "SEGA") e a descompressão da arte rodam no núcleo, nos mesmos passos do original, porque o progresso deles fica na memória comparada e afeta o tempo do processador. O crate de som só sintetiza, e o crate da ROM só identifica e valida a ROM.
- **Vídeo fiel em software:** o renderizador desenha na resolução original, na CPU, para a comparação pixel a pixel ser determinística. Ele aplica as mudanças feitas no meio do quadro, linha a linha, como a paleta da água da Labyrinth. A GPU só apresenta e amplia a imagem.
- **Tempo do processador:** o núcleo tem pontos de checagem onde o laço principal usa memória que as interrupções também usam. No nível 1, o tempo gravado diz em que ponto cada interrupção chegou. No nível 2, o núcleo calcula o custo em ciclos lendo o código do 68000 e do Z80 na ROM, sem executá-lo, e soma os fatores do hardware: DMA, espera do chip de vídeo, refresh da memória e o uso do barramento pelo Z80. O custo de cada trecho é calculado uma vez, quando a ROM é carregada.
- **Mesmo resultado no nativo e no WebAssembly:** estouro de números tratado de forma explícita, sem ponto flutuante no núcleo e sem depender do tamanho de ponteiro nem da ordem de um HashMap.

### Disassembly e mapa de rótulos

O disassembly fica fora do repositório, como a ROM, porque montá-lo reproduz a ROM byte a byte. Ele fica fixado num commit, porque os rótulos mudam entre versões: a documentação continuou até 25/10/2026, com rótulos renomeados em outubro.

O mapa de rótulos cobre rotinas, variáveis da memória e constantes, como a posição de cada campo de um objeto. Ele tem só nomes, endereços e valores, então é metadado e fica no git. Ele sai de montar esse commit com `Revision = 1` e conferir que o resultado é idêntico à ROM de referência. O núcleo precisa desse mapa para manter a mesma memória do original, e também porque a memória guarda endereços de código: o descompressor grava na RAM o endereço da sua rotina de escrita (`v_plc_ptrnemcode`). A IA lê os arquivos `.asm`, mas nunca os binários do disassembly.

### Pipelines

São dois pipelines: como o jogo roda e como a fidelidade é provada.

&#91;embedded content: pipeline do jogo · da ROM à tela e ao som\]

A ROM chega ao navegador pelo seletor de arquivo. O núcleo recebe os bytes e a entrada, e entrega as saídas do quadro. Vídeo e som só obedecem a essas saídas.

&#91;embedded content: pipeline de fidelidade · oráculo contra núcleo\]

Os dois lados recebem os mesmos insumos. O comparador para na primeira divergência de cada caso. Tudo isso roda no computador de Marcelo, com a ROM fora do repositório.

## Plataforma e restrições

O alvo é WebAssembly no navegador, com aceleração gráfica. Versões nativas para Linux, Mac e Windows só entram se o navegador se mostrar inviável.

- **Linguagem:** Rust. O site usa um mínimo de HTML e JavaScript só para carregar o WebAssembly.
- **Testes:** uma versão nativa sem tela existe desde o início, só para rodar os testes. O CI roda os testes unitários do núcleo também em WebAssembly, e localmente um caso de fidelidade roda nos dois e compara os hashes.
- **Taxa de quadros:** no navegador, o máximo é a taxa do monitor. A lógica do jogo roda sempre a 60 Hz exatos, e não aos 59,92 Hz do console. A diferença de 0,13% não muda nenhum quadro, e a decisão é revista quando a interpolação entrar. O áudio é reamostrado para a taxa do navegador, com ajuste fino de taxa, para acompanhar os 60 Hz sem cortes nem deriva.
- **Entrada:** teclado e gamepad, sem touch.
- **Divulgação:** nenhuma. Código e site são públicos por necessidade técnica (GitHub Pages e minutos grátis do CI), sem ROM, sem dados extraídos e sem traces no git.
- **Navegadores:** as versões atuais de desktop. O GitHub Pages não permite configurar cabeçalhos HTTP. Existe um contorno com service worker, mas o projeto não depende de threads nem de memória compartilhada, e o contorno fica como opção futura. O áudio usa um AudioWorklet alimentado por eventos de escrita nos chips.
- **ROM entre sessões:** o jogador escolhe a ROM a cada sessão. Guardá-la no navegador fica para depois.
- **Escala:** a versão jogável mostra a imagem de 320x224 em escala inteira simples. Filtros e CRT ficam para as fases futuras.

## Ambiente e estrutura

Todo o desenvolvimento roda em Docker, e o código do projeto é só Rust. As exceções são mínimas: HTML e JavaScript para carregar o WebAssembly, algumas linhas de shell nos hooks do git e o gravador de partidas, que roda fora do Docker porque precisa de janela, controle e som.

- **Imagem:** Debian slim, com Rust fixo, o alvo WebAssembly, o gerador de ligação com o navegador, um executor de WebAssembly para os testes, o linker C que o Rust exige no Linux, o Lua que o build do disassembly usa e o Claude Code. Um Dockerfile só, sem Docker Compose.
- **Contêineres:** o contêiner de trabalho é descrito num `devcontainer.json` e abre pelo VS Code, pelo Cursor ou pelo terminal, com a ferramenta de linha de comando de dev containers. A rede dele só alcança o necessário: a API do Claude, os pacotes do Rust e o GitHub para leitura. A auditoria usa um segundo contêiner, da mesma imagem, sem iniciar o Claude Code, configurado em `.devcontainer/auditoria/`.
- **Pastas fora do repositório:** a ROM e o disassembly, montados só para leitura; a pasta de trabalho, com traces e relatórios; e os testes ocultos, cifrados, que só entram no contêiner de auditoria. As chaves ficam no computador de Marcelo.
- **Comandos:** `cargo xtask` roda as tarefas, como montar o site, rodar a fidelidade e checar que não há ROM no git.
- **CI:** o GitHub Actions faz a checagem anti-ROM, o build, os testes unitários do núcleo em nativo e em WebAssembly e a publicação no GitHub Pages, com a mesma imagem Docker. O repositório é público, então os minutos são grátis.
- **Fidelidade:** roda só no computador de Marcelo, porque a ROM nunca está no repositório.

### Proteção contra a ROM no git

Uma ROM que entre num commit continua no histórico público mesmo depois de apagada. A regra é pelo conteúdo: nada com conteúdo da ROM entra no git, seja byte, imagem, som, código, trace ou estado. Metadados entram: nomes, endereços, hashes e listas, como o mapa de rótulos, os inventários e a base de conhecimento. A proteção tem quatro camadas.

1. A ROM, o disassembly, os traces e os estados ficam fora da pasta do repositório.
2. O `.gitignore` bloqueia extensões e pastas de ROM.
3. Um hook local, antes do commit e do push, bloqueia por extensão, tamanho e hash da ROM. Ele roda onde o git roda: no contêiner, quando a IA faz commit, e no computador de Marcelo, nos commits e pushes dele. Esta é a camada que realmente impede o vazamento.
4. O CI reprova se achar um arquivo com o hash da ROM. É só a última rede de segurança.

### Estrutura de pastas

```
sonic1/
├─ crates/
│  ├─ core/        núcleo fiel: lógica, drivers de som e descompressão
│  ├─ rom/         identificação e validação da ROM
│  ├─ video/       renderizador fiel em software
│  ├─ audio/       sintetizador (FM e PSG)
│  ├─ fidelidade/  formato do trace, comparador e relatório
│  └─ web/         frontend WebAssembly
├─ xtask/          comandos do projeto
├─ oracle/         workspace separado: emuladores de referência
├─ corpus/         entradas gravadas, só botões, sem traces
├─ refs/           manifesto de hashes e assinatura
├─ docs/           requisitos.md (cópia), estado.md e conhecimento/
├─ .devcontainer/  Dockerfile, devcontainer.json e permissões do Claude Code
│  └─ auditoria/   configuração do contêiner de auditoria
├─ hooks/          hooks do git, em shell
├─ .claude/        configuração do Claude Code no projeto
├─ CLAUDE.md       regras de trabalho para o Claude Code
└─ .github/        CI e deploy
```

## Regras para uso de IA e fluxo por etapa

A IA escreve o código do jogo, mas nunca altera o que mede se ele está certo. As regras valem por bloqueio técnico, não só por texto: o `CLAUDE.md` orienta o modelo, mas quem bloqueia é o sistema de permissões do Claude Code.

- **Onde a IA roda:** o Claude Code roda dentro do contêiner de trabalho. Ele tem escrita no repositório e na pasta de trabalho, e a ROM e o disassembly ficam montados só para leitura. Testes ocultos, chaves e credenciais do GitHub nunca entram nesse contêiner, e a rede dele só alcança o necessário.
- **Permissões:** a IA não lê a ROM nem os binários do disassembly. Depois que cada item é congelado, ela também não edita referências, oráculos, comparador, Dockerfile, CI, hooks nem as próprias permissões. As permissões ficam na imagem, com prioridade sobre o repositório. O executor de testes continua lendo a ROM, porque essas regras não valem para outros programas.
- **Commits:** a IA só faz commit local. Só Marcelo faz push.
- **Referências:** a IA constrói o oráculo e o comparador nas etapas 1 e 2. Marcelo congela esses itens depois de verificações que ele mesmo roda: o comparador acusa divergências plantadas, duas execuções geram o mesmo trace e os dois oráculos concordam nas demos. Uma sessão nova, que não escreveu o código, também revisa. Depois disso, mudar exige a aprovação de Marcelo. A IA pode criar testes novos, mas nunca altera nem apaga testes, traces ou a lista de exclusões já congelados.
- **Assinatura:** o comando local de fidelidade falha se uma referência mudar sem a assinatura do manifesto ou se algum teste deixar de rodar. A chave de assinatura exige senha ou toque físico a cada uso, então a IA não assina sozinha.
- **Testes ocultos:** Marcelo grava jogando, fora do contêiner de trabalho, organizados por zona, com no mínimo uma partida completa de cada ato. Eles ficam cifrados, e só ele tem a chave. Marcelo roda a auditoria no contêiner de auditoria, sem a IA, que só vê "passou" ou "falhou" por zona. Um caso que falha e é mostrado à IA vira caso normal, e Marcelo grava outro.
- **Exclusões da memória:** áreas ignoradas na comparação seguem a mesma aprovação das exclusões de código.
- **Origem dos dados:** o núcleo não pode conter dados copiados dos traces nem do disassembly, cujos arquivos `.asm` também trazem dados da ROM em texto, como mapeamentos e animações. Toda tabela vem da ROM, e cada função cita o rótulo da rotina do disassembly que reproduz. Só o nome do rótulo: nunca copiar código nem comentários do disassembly.
- **Sem atalhos:** a IA não escreve interpretador nem tradutor automático do código do 68000 ou do Z80, e não usa os registradores desses processadores como estado do núcleo.
- **ROM e hooks:** a IA nunca lê a ROM direto nem cola dados dela no código ou no chat, e nunca usa `--no-verify` nem desliga os hooks.
- Um teste que falha se corrige no jogo, nunca no teste.

### Fluxo por etapa

Este documento é a fonte única. O Claude Code lê uma cópia exportada dele em `docs/requisitos.md`, só para leitura, com a data e a revisão do documento, atualizada no início de cada etapa. As regras de trabalho ficam no `CLAUDE.md`. O chat de planejamento é a conversa no Claude onde este documento é mantido e os prompts são gerados.

1. Marcelo pede ao chat de planejamento o prompt da etapa.
2. O chat de planejamento relê este documento e o resumo da etapa anterior, e entrega o prompt e a cópia exportada deste documento.
3. Marcelo salva a cópia exportada em `docs/requisitos.md`.
4. O Claude Code lê a cópia, o `docs/estado.md` e o índice da base de conhecimento, apresenta um plano, e Marcelo aprova.
5. O Claude Code implementa, atualiza o `docs/estado.md` e registra na base de conhecimento o que aprendeu.
6. Uma sessão nova, que não escreveu o código, revisa o trabalho contra este documento.
7. Marcelo cola no chat de planejamento o resumo da etapa e o que entrou na base de conhecimento, e o status da etapa muda aqui.

### Base de conhecimento

Cada etapa deixa conhecimento para as próximas, em `docs/conhecimento/`, com um arquivo por assunto, como vídeo, som, interrupções, objetos ou uma zona.

- **O que entra:** como uma rotina funciona, explicado com palavras próprias; armadilhas encontradas; diferenças entre os oráculos; decisões técnicas e o motivo de cada uma.
- **Evidência:** cada entrada cita o teste, o trace ou o commit que a sustenta. Entrada sem evidência não entra.
- **O que não entra:** código ou comentários copiados do disassembly, dados da ROM e traces.
- **Índice:** `docs/conhecimento/README.md` tem uma linha por arquivo. Cada sessão lê o índice e só os arquivos dos assuntos da etapa, porque a base inteira não cabe no contexto.
- **Uso:** o `docs/estado.md` diz onde o trabalho parou, e a base diz o que já se sabe. Os prompts das etapas seguintes partem dela.
- **Manutenção:** no fim de cada etapa, a IA revisa a base, corrige o que ficou errado, apaga o que ficou velho e atualiza o índice.

## Etapas

Cada etapa entrega algo verificado e nunca regride as anteriores. Cada etapa grande vira sub-etapas na hora de gerar o prompt.

| # | Etapa | Pronto quando | Depende de | Status |
| --- | --- | --- | --- | --- |
| 0 | Esqueleto do repositório | O site publica no Pages, mostra o hash da ROM escolhida e recusa ROMs que não sejam a REV01. Os hooks bloqueiam a ROM real num repositório descartável, sem remoto. O Claude Code roda no contêiner de trabalho, aberto pelo devcontainer.json, com as permissões e a rede restrita. Um teste mostra que a IA não consegue ler a ROM nem editar um arquivo congelado. Existem o CLAUDE.md, o docs/estado.md e a base de conhecimento com índice | nenhuma | Não iniciada |
| 1 | Oráculo | O emulador lê a memória do jogo, do Z80 e do vídeo, injeta entradas, salva e carrega estados e para em endereços. Registra o ponto de cada interrupção (endereço e ciclo), os desvios executados nos dois processadores, as escritas de vídeo com a linha, as de som e do DAC com o ciclo e as esperas pelo Z80. Chama uma rotina isolada a partir de um estado e de registradores definidos. Duas execuções geram o mesmo trace. O mapa de rótulos sai do disassembly fixado, e a ROM montada é idêntica à de referência | 0 | Não iniciada |
| 2 | Corpus e integridade | Existem o gravador de partidas e o formato de entradas. Demos, partidas gravadas e TAS conferidos contra a REV01 viram traces, com hashes por quadro e estados a intervalos. Os inventários de hardware e de memória compartilhada estão prontos. O manifesto é assinado e verificado. Os primeiros testes ocultos, por zona, estão cifrados. O segundo oráculo concorda nas demos | 1 | Não iniciada |
| 3 | Descompressores | A arte, os blocos e os mapas saem idênticos, byte a byte, ao que o oráculo carrega | 1 | Não iniciada |
| 4 | Renderizador fiel | Com as escritas de vídeo capturadas do oráculo, todo quadro do corpus sai idêntico, pixel a pixel, na cor de 9 bits e na área de 320x224, incluindo as mudanças no meio do quadro | 2 | Não iniciada |
| 5 | Núcleo base | Memória espelhada, operações com o mesmo resultado e as mesmas flags do 68000 (sem interpretar instruções), laço de quadro com o tempo gravado, pontos de checagem de interrupção, testes por rotina, bugs injetados e relatório de cobertura e de bugs conhecidos funcionam | 2 | Não iniciada |
| 6 | Driver de som | O driver de som do 68000 e o programa do Z80, com a bateria e a voz SEGA, estão reproduzidos. O estado deles e as escritas nos chips de som, incluindo as do DAC, são idênticos ao oráculo em conteúdo e ordem, em todas as músicas e efeitos | 3, 5 | Não iniciada |
| 7 | Telas de abertura | Do ligar do console ao início da primeira demo, zero divergência. As telas aparecem no navegador. Um experimento do nível 2, que não trava a etapa, tenta prever o fim da espera da voz SEGA e o tempo do boot e mede o erro do modelo | 3, 4, 6 | Não iniciada |
| 8 | Sintetizador | FM, incluindo o DAC, idêntico ao Nuked-OPN2 no modo YM2612 e PSG idêntico ao Nuked-PSG, amostra a amostra, na taxa de cada chip, com as escritas do oráculo nos mesmos instantes. A parte analógica é medida e marcada como aproximada. A música toca no navegador, reamostrada sem cortes | 6 | Não iniciada |
| 9 | Green Hill | Sub-etapas: física e colisão do Sonic; objetos e inimigos; demo e atos 1 e 2; ato 3 e chefe. Zero divergência, cobertura fechada e testes ocultos da zona passando. Jogável no navegador | 7 | Não iniciada |
| 10 | Demais zonas | Uma sub-etapa por zona: Marble, Spring Yard, Labyrinth, Star Light, Scrap Brain e Final Zone. Mesmo critério da Green Hill | 9 | Não iniciada |
| 11 | Special stages | Rotação, esmeraldas e saídas sem divergência | 9 | Não iniciada |
| 12 | Telas finais e segredos | Game over, continues, finais, créditos e os quatro códigos secretos do console japonês sem divergência | 10, 11 | Não iniciada |
| 13 | Navegador completo | O jogo roda do começo ao fim no Pages, com teclado, gamepad e escala inteira, sem nenhuma falha de áudio registrada numa partida completa | 8, 12 | Não iniciada |
| 14 | Nível 1 de pronto | O relatório completo passa: zero divergência, cobertura fechada, bugs injetados detectados, bugs conhecidos cobertos, som idêntico às referências, testes ocultos e dois oráculos | 8, 12 | Não iniciada |
| 15 | Tempo previsto (nível 2) | O tempo previsto pelo núcleo (quadros de lag, ponto das interrupções, fim dos laços de espera, esperas pelo Z80 e ciclo de cada escrita de som) é igual ao do oráculo em todo o corpus e nos testes ocultos. Fecha o 100% | 14 | Não iniciada |

## Fases futuras

Nada abaixo começa antes do nível 2 de pronto, exceto a escala inteira simples na versão jogável. O relatório de fidelidade roda sempre com todas as opções desligadas.

1. Fps acima de 60, até a taxa do monitor, com a lógica fixa em 60 Hz e a imagem interpolada.
2. Tela mais larga sem mudar a jogabilidade. No original, objetos longe da câmera nem são carregados; como mostrá-los na área extra fica a definir na fase.
3. Ampliação em pixel art: escala nítida como padrão, com simulação de CRT e filtros de pixel art como opções.
4. Áudio realçado, com efeitos por canal. Pode usar outro chip compatível, como o YM3438, mais limpo, ou síntese própria. O modo fiel continua disponível.
5. Toggles de correção de bugs originais e de slowdown. A lista de bugs conhecidos do disassembly é o ponto de partida.
6. Menus, savestates e outras funções de conforto, como o botão reset.

## Viabilidade e riscos

O Sonic 1 é viável. O disassembly da comunidade passou por um grande esforço de documentação em 2026: o último grande bloco, a Final Zone e todos os chefes, entrou em julho, e os refinamentos seguiram até 25/10/2026. O OpenGGF, um projeto parecido escrito em Java, se declara alfa e valida o Sonic, a câmera e os objetos contra traces do Genesis Plus GX, sem comparar a memória inteira. O critério deste projeto é mais rígido. Os riscos estão no tempo do processador e no tempo total do projeto.

| Parte | Avaliação | Por quê |
| --- | --- | --- |
| Tempo do processador em partidas novas | Arriscado | Exige prever lag, ponto das interrupções e laços de espera, com DMA, espera do chip de vídeo, refresh da memória e o uso do barramento pelo Z80 |
| Tempo total | Longo | Projeto de anos com pouco tempo por semana, mas cada etapa entrega algo verificado |
| Bateria e voz SEGA | Realista no nível 2 | O programa do Z80 é pequeno e tem laços de custo fixo. Só a parte analógica fica aproximada |
| Física, objetos, fases, chefes e special stages | Realista | Código original documentado e comparável rotina por rotina |
| Vídeo, incluindo a água do Labyrinth | Realista | Pode ser provado pixel a pixel, linha a linha, com as escritas capturadas do oráculo. Os pontos coloridos da água dependem dos oráculos |
| Comandos de música e efeitos | Realista | Comparados um a um com o oráculo |
| Navegador | Realista | Vídeo acelerado e áudio em thread própria funcionam sem recursos experimentais |

Por isso o tempo previsto é a última etapa, mas um experimento na etapa 7 mede cedo o erro do modelo. Até lá, os testes usam o tempo gravado do oráculo. A versão jogável roda sem slowdown só como limitação temporária, nunca como opção.

### Riscos

| Risco | Mitigação |
| --- | --- |
| O jgenesis não expõe o que a comparação precisa | Critérios na etapa 1; versão modificada, ou troca pelo segundo oráculo |
| O oráculo é impreciso | Dois oráculos concordando, regra de desempate e cada diferença registrada |
| O ponto das interrupções muda o resultado, como nos anéis, nos sons e na Labyrinth | Trace com o ponto exato de cada interrupção e pontos de checagem no núcleo |
| O Z80 muda o tempo do 68000, nas esperas pelo barramento e na voz SEGA | Programa do Z80 reproduzido no núcleo, com custo em ciclos, como o código do 68000 |
| Rotinas usam valores que sobraram em registradores, inclusive bugs | O teste por rotina compara os registradores que o chamador usa depois |
| Os oráculos divergem nas cores ou nos pontos coloridos da paleta | Comparação na cor de 9 bits e regra de desempate entre oráculos |
| A IA mascara falhas | Permissões do Claude Code, assinatura, testes ocultos e bugs injetados |
| Os TAS são incompatíveis com a REV01 | Conferir cada TAS contra a REV01 e descartar os que perdem a sincronia |
| A suíte completa e os bugs injetados tomam tempo demais | Níveis de teste e orçamento de tempo por etapa |
| O sintetizador fiel é pesado demais para o navegador | Medir na etapa 8, com um modo mais leve como plano B |
| A previsão do tempo se mostra inviável | Experimento do nível 2 já na etapa 7, decisões mapeadas ao original desde o início, margem de ciclos no trace e revisão das fases futuras |

## Decisões

Todas as decisões abaixo estão fechadas. Mudanças exigem nova aprovação de Marcelo.

| Decisão | Situação |
| --- | --- |
| Escopo | Decidido: só o Sonic 1. Os outros Sonics são decisão futura |
| Plataforma | Decidido: WebAssembly primeiro. Nativo só se o WebAssembly não for viável. Teclado e gamepad, sem touch |
| Slowdown e quadros de lag do original | Decidido: fiel por padrão. O toggle para desligar entra só depois de 100% de fidelidade |
| Linguagem de programação | Decidido: Rust, com aprendizado como objetivo |
| Tempo disponível por semana | Decidido: pouco tempo, sem prazo |
| Critério de pronto | Decidido: relatório reproduzível em dois níveis, sem metas percentuais. O nível 1 usa o tempo gravado, e o nível 2 prevê o tempo |
| Definição de "sem emulação" | Decidido: nenhuma instrução original, do 68000 ou do Z80, é executada, interpretada ou traduzida automaticamente. Ler o código da ROM para calcular ciclos é permitido. Vídeo e som são próprios, guiados pelos mesmos comandos |
| Estrutura do núcleo | Decidido: espelhar a memória e as rotinas do original, com Rust legível e sem registradores do 68000 nem do Z80 como estado. Toda decisão da lógica do jogo corresponde a uma decisão do original |
| Programa do Z80 | Decidido: reproduzido no núcleo como o código do 68000, com custo em ciclos. A bateria deixa de ser aproximada no nível 2 |
| Regras para uso de IA | Decidido: as da seção de regras, aplicadas pelas permissões do Claude Code |
| Onde a IA roda | Decidido: Claude Code dentro do contêiner de trabalho, sem credenciais do GitHub. Só Marcelo faz push |
| Contêineres | Decidido: contêiner de trabalho num devcontainer.json, aberto pelo VS Code, pelo Cursor ou pelo terminal, com a rede restrita ao necessário. Auditoria num contêiner separado, da mesma imagem, sem iniciar o Claude Code |
| O que entra no git | Decidido: metadados, como nomes, endereços, hashes e listas. Conteúdo da ROM nunca |
| Documentação no repositório | Decidido: cópia exportada deste documento, estado da etapa e base de conhecimento com índice, atualizada a cada etapa |
| Testes ocultos | Decidido: gravados por Marcelo, por zona, com no mínimo uma partida completa de cada ato, cifrados, rodados só no contêiner de auditoria e trocados quando falham |
| Versão de referência | Decidido: REV01, a revisão posterior e mais polida. REV00 e REVXB ficam fora. A região é uma configuração separada |
| ROM de referência | Decidido: SHA-1 69e102855d4389c3fd1a8f3dc7d193f8eee5fe5b e MD5 09dadb5071eb35050067a32462e39c5f, conferidos no TASVideos e no verificador do disassembly |
| Console de referência | Decidido: Mega Drive japonês, modelo 1, NTSC, com controle de 3 botões |
| Botão reset | Decidido: fora do escopo. O código de reset vira exclusão aprovada |
| Modo de depuração e seletor de fases | Decidido: incluídos, junto com os outros códigos secretos do console japonês |
| Ritmo em tempo real | Decidido: 60 Hz exatos, com o áudio reamostrado. Revisão quando a interpolação entrar |
| Chip de som | Decidido: YM2612 modelo 1 como referência de fidelidade. Chips compatíveis e mais limpos ficam como opção do modo realçado |
| Tradução | Decidido: fora do escopo |
| Oráculo | Decidido: jgenesis como principal, se cumprir o critério da etapa 1. Segundo oráculo entre BlastEm e Genesis Plus GX |
| Ambiente de desenvolvimento | Decidido: Docker mínimo, Debian slim com Rust, mais as ferramentas que o build e os testes exigem |
| Linguagens do projeto | Decidido: só Rust no código, com um mínimo de HTML e JavaScript no site e de shell nos hooks. Sem Python |
| Repositório e CI | Decidido: público e sem ROM. O CI faz checagem anti-ROM, build, testes do núcleo em WebAssembly e deploy no Pages |
| Testes de fidelidade | Decidido: rodam só localmente, com a ROM fora do repositório |

## Glossário

| Termo | Significado |
| --- | --- |
| 68000 | O processador principal do Mega Drive, que roda o jogo |
| Auditoria | Rodada dos testes ocultos, feita só por Marcelo, no contêiner de auditoria, sem a IA |
| Base de conhecimento | O que cada etapa aprendeu, com evidência, para as sessões e os prompts seguintes |
| Bug conhecido | Bug do original marcado no disassembly, que o núcleo precisa reproduzir |
| Bug injetado | Defeito colocado de propósito no núcleo para ver se a suíte o detecta |
| Chat de planejamento | A conversa no Claude onde este documento é mantido e os prompts das etapas são gerados |
| Cobertura | Parte das decisões do código original, do 68000 e do Z80, testada nos dois caminhos |
| Contêiner de auditoria | Contêiner da mesma imagem, sem iniciar o Claude Code, onde Marcelo roda os testes ocultos |
| Contêiner de trabalho | Contêiner onde o Claude Code e as ferramentas rodam, com a rede restrita e a ROM só para leitura |
| Corpus | Conjunto de entradas gravadas e traces usados nos testes |
| DAC | Canal do chip FM que toca amostras digitais; no Sonic 1, a bateria e a voz SEGA |
| Divergência | Diferença entre o oráculo e o núcleo em algum quadro |
| DMA | Cópia de dados para o chip de vídeo feita pelo hardware; enquanto ela roda, o 68000 fica parado |
| FM | O chip de som principal, o YM2612, que sintetiza a maior parte da música e dos efeitos |
| Interrupção de linha | Interrupção (HBlank) que o chip de vídeo gera numa linha escolhida da tela; o Sonic 1 só a usa na água da Labyrinth |
| Inventário | Lista, tirada do corpus, dos recursos de hardware e da memória compartilhada que o Sonic 1 usa |
| Manifesto | Lista assinada com o hash de cada referência: testes, traces, oráculos e exclusões |
| Mapa de rótulos | Nomes, endereços e valores das rotinas, variáveis e constantes do original, gerados montando o disassembly fixado |
| Margem de ciclos | Quantos ciclos sobraram, ou faltaram, antes do VBlank em cada quadro |
| Memória compartilhada | Memória que o laço principal e as interrupções usam |
| Núcleo | A parte do jogo que espelha a memória e as rotinas do original |
| Oráculo | A ROM rodando num emulador sem tela, usado só nos testes |
| Ponto de interrupção | Lugar do laço principal em que uma interrupção chegou |
| PSG | Chip de som simples, de ondas quadradas e ruído, que fica dentro do chip de vídeo |
| Quadro de lag | Quadro em que a lógica do jogo não termina antes do VBlank |
| Slowdown | Perda de ritmo do jogo, efeito visível dos quadros de lag |
| TAS | Partida feita quadro a quadro com ferramentas, usada como entrada de teste |
| Tempo gravado | Quadros de lag, ponto de cada interrupção, fim de cada laço de espera e esperas pelo Z80, tirados do oráculo e usados no nível 1 |
| Teste por rotina | Teste que roda uma rotina do núcleo isolada e compara com o oráculo, inclusive os registradores que o chamador usa depois |
| Testes ocultos | Conjunto cifrado, gravado por Marcelo e nunca visto pela IA, rodado só na auditoria |
| Toggle | Opção, desligada por padrão, que muda o jogo, como corrigir um bug ou desligar o slowdown |
| Trace | Registro, quadro a quadro, do que o oráculo produziu para uma entrada, com hashes por quadro e estados a intervalos |
| VBlank | Interrupção que o chip de vídeo gera a cada quadro, quando termina de desenhar a tela |
| Z80 | O processador de som do Mega Drive; no Sonic 1, ele só toca a bateria e a voz SEGA |

## Fontes

- [Disassembly do Sonic 1](https://github.com/sonicretro/s1disasm)
- [TASVideos: Sonic the Hedgehog, com os hashes da REV00 e da REV01](https://tasvideos.org/248G)
- [OpenGGF v0.6.20260917, declarado alfa](https://github.com/OpenGGF/OpenGGF/releases/tag/v0.6.20260917)
- [OpenGGF: framework de traces](https://openggf-webzone.pages.dev/docs/guide/contributing/trace-framework-reference)
- [Drivers de som dos jogos do Sonic](https://clownacy.wordpress.com/2021/04/18/the-sound-drivers-of-sonic-the-hedgehog/)
- [jgenesis, emulador de Mega Drive em Rust](https://github.com/jsgroth/jgenesis)
- [BlastEm, emulador de alta precisão](https://retrodev.com/blastem)
- [Nuked-OPN2, núcleo do YM3438 com modo YM2612](https://github.com/nukeykt/Nuked-OPN2)
- [ym3438.h, com o modo YM2612](https://docs.rs/crate/renuked/latest/source/ym3438.h)
- [Furnace: núcleos de emulação, incluindo o Nuked-PSG](https://tildearrow.org/furnace/doc/latest/9-guides/emulation-cores.html)
- [Nuked-MD, emulação baseada em fotos dos chips](https://www.Github.com/nukeykt/Nuked-MD)
- [Sonic Physics Guide](https://info.sonicretro.org/Sonic_Physics_Guide)
- [Cabeçalhos COOP e COEP no GitHub Pages](https://docs.wasmer.io/sdk/wasmer-js/how-to/coop-coep-headers)
- [Claude Code: permissões](https://code.claude.com/docs/en/permissions)
- [Claude Code: dev containers](https://code.claude.com/docs/en/devcontainer)
- [Suporte a WebGPU nos navegadores](https://en.wikipedia.org/wiki/WebGPU)
