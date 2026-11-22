# Plano da Etapa 2: Corpus e integridade

Escrito em 06/11/2026. Aguarda aprovação do Marcelo. Nada foi implementado.
Fonte: `docs/requisitos.md` (sem conflito achado), `docs/estado.md`, `docs/backlog.md`, `docs/conhecimento/oraculo.md` e `hooks.md`.

## 0. Pré-condição e achado novo

- 1.8 e 1.9 estão registradas como feitas pelo Marcelo (commit c188a83). A ROM montada é idêntica e o trace de 600 quadros é determinístico com a ROM real.
- **Achado:** o `refs/rotulos.txt` commitado está com os valores errados. O `montar` rodou com a base 10, e a tabela vem em hexadecimal. Só 2 variáveis entraram e `v_plc_ptrnemcode` não existe. É inferência, ainda sem comando que a prove. Detalhes em `docs/conhecimento/oraculo.md`. Isso vira a sub-etapa 2.0, porque os inventários e a parada por rótulo dependem do mapa.
- Nada nesta sessão rodou sobre a ROM.

## 1. Sub-etapas

Ordem: o mais barato e arriscado primeiro. Legenda de "roda": **IA** roda com dados sintéticos; **Marcelo** roda no host, fora do Claude, com a ROM.

| # | Entrega | Prova (comando e resultado esperado) | Roda |
| --- | --- | --- | --- |
| 2.0 | Mapa de rótulos corrigido. `xtask rotulos` passa a usar base 16 por padrão e falha se não houver símbolo de RAM nem `v_ram_end`. | IA: `cargo test -p xtask` com listagem sintética em hexadecimal (o teste atual repetia a suposição errada; entra um caso com valores `FFFFF600`). Marcelo: `sh oracle/scripts/monta-disasm.sh montar s1built.bin sonic.lst 16`. Esperado: `IDENTICA`, `variaveis` na casa das centenas, `grep -P '^v_plc_ptrnemcode\t' refs/rotulos.txt` acha uma linha. | IA e Marcelo |
| 2.1 | Teste curto de viabilidade do segundo oráculo (seção 8). Registro em `docs/conhecimento/oraculos.md`. | Marcelo: o `hash` de `ram68k` do quadro N do segundo oráculo contra o do jgenesis, no boot, 600 quadros, sem entrada. Esperado: relatório com "igual" ou "difere no quadro X, categoria Y". Qualquer resultado é aceito: o objetivo é saber o custo. | IA escreve o harness; Marcelo roda |
| 2.2 | Gravador de partidas (seção 3), como patch do jgenesis com janela. | IA: teste com entradas sintéticas (gravar uma lista, reler, mesmo SHA-1). Marcelo: joga 600 quadros, depois `oraculo rodar --entradas <gravado>` sem janela. Esperado: o `hash_da_ram68k` do último quadro impresso pela janela ao sair é igual ao do headless. | IA e Marcelo |
| 2.3 | Formato de entradas `.s1in` em `corpus/`, validador e conversores de TAS (seção 2). Crate novo `crates/corpus` (MIT). | IA: `cargo test -p sonic1-corpus` com filmes sintéticos (GMV e BK2 inventados por nós). Esperado: ida e volta idêntica; quadro de lag; entrada malformada recusada. | IA |
| 2.4 | Traces do corpus: demos internas, partidas e TAS (seção 4). `cargo xtask corpus-trace`. | Marcelo: `cargo xtask corpus-trace --tudo`. Esperado: por caso, uma linha `nome hash_do_trace quadros` e `determinismo: PASSOU`. | Marcelo (IA testa com ROM sintética) |
| 2.5 | Segundo oráculo concorda nas demos, e registro das diferenças (seção 8). | Marcelo: `cargo xtask oraculos-comparar --corpus demos`. Esperado: `concordam` ou lista de diferenças com decisão. | Marcelo |
| 2.6 | Inventários de hardware e de memória compartilhada (seção 5). | IA: testes com trace sintético. Marcelo: `cargo xtask inventario`. Esperado: `docs/inventario-hardware.md` e `docs/inventario-memoria.md` gerados só com nomes, endereços e contagens, cada item com evidência. | IA e Marcelo |
| 2.7 | Manifesto e assinatura (seção 6). | IA: chave descartável criada no teste; `cargo xtask manifesto verificar` passa, e falha nos 4 casos plantados. Marcelo: assina com a chave real. | IA e Marcelo |
| 2.8 | Testes ocultos cifrados, ferramenta e primeiros casos (seção 7). | IA: dados sintéticos e chave descartável. Marcelo: grava, cifra e roda a auditoria. | IA e Marcelo |
| 2.9 | Fechamento: `estado.md`, base de conhecimento, backlog, e sessão de revisão só do que a Etapa 2 mudou. | `cargo xtask fidelidade` (esqueleto: manifesto + traces + segundo oráculo) passa no host do Marcelo. | Marcelo |

Se a etapa estourar uma sessão, o corte padrão é: primeiro os conversores de TAS (2.3, parte TAS), depois os testes ocultos além da Green Hill (2.8). O resto é o "Pronto quando".

## 2. Formato de entradas e TAS

- Já existe em `oracle/runner/src/entradas.rs`: texto, uma linha por quadro, máscara de 2 dígitos hexadecimais, `#` comenta. Bits: Cima 01, Baixo 02, Esquerda 04, Direita 08, B 10, C 20, A 40, Start 80. Porta 1, controle de 3 botões. Porta 2 vazia. Depois do último quadro, nenhum botão.
- O arquivo novo é `corpus/<nome>.s1in` (mesma sintaxe) mais `corpus/<nome>.meta` com: fonte, licença, SHA-1 da ROM, número de quadros, SHA-1 das entradas. A extensão `.s1in` não é bloqueada pelos hooks, o que é de propósito.
- **Regra do quadro de lag:** o índice é o quadro do emulador, e não o da lógica do jogo. Toda linha vale para um quadro de vídeo, tenha o jogo lido o controle ou não. Num quadro de lag o jogo não lê o controle, então a linha é ignorada sozinha, como no original. Por isso nenhuma conversão corrige lag. Filmes de TAS também indexam por quadro do emulador. Prova: caso sintético com lag em `crates/corpus` e, com a ROM, o trace mostra `leitura_busack`/lag igual nas duas execuções.
- **Conversão de TAS:** `cargo xtask tas-converter ARQUIVO` lê GMV (cabeçalho e um byte-par por quadro) e BK2 (arquivo zip com o log de entrada em texto). **Os formatos exatos ainda não foram conferidos em documentação nesta sessão.** A primeira tarefa da 2.3 é confirmar na documentação do TASVideos e do BizHawk e citar a fonte em `docs/conhecimento/corpus.md`. Se um formato não for confirmável, ele não entra.
- **Conferência contra a REV01:** o TAS roda no oráculo. Aceito só se, no último quadro, o modo de jogo, a zona e o ato (variáveis do mapa 2.0) coincidem com o que o TAS declara no cadastro, e o segundo oráculo concorda (seção 8). Se a sincronia se perde, o TAS é descartado e a linha vai em `corpus/descartados.txt` (nome, fonte, motivo). O estado inicial do oráculo é zerado (R5), então TAS gravados em outros emuladores podem perder a sincronia desde o primeiro quadro. Esse descarte é esperado, e não falha.
- Onde ficam os arquivos baixados: `$SONIC1_EXT_DIR/tas/`, fora do repositório. Só a entrada convertida (botões) entra em `corpus/`.

## 3. Gravador de partidas

Opções:

| Opção | Prós | Contras |
| --- | --- | --- |
| A. Patch do jgenesis com janela (o frontend gráfico dele) que grava a máscara de botões por quadro | Mesmo núcleo do oráculo: a gravação reproduz sem risco de sincronia. Controle e som já funcionam. Mudança pequena | O README do jgenesis não cita gravação de entradas (lido em 06/11/2026). Preciso confirmar no código onde as entradas são lidas por quadro. Compilar o frontend exige SDL3 e wgpu no host. Mais um patch GPLv3 para manter |
| B. Frontend próprio (novo binário no workspace do oráculo com janela, `gilrs` e `cpal`) | Controle total | Muito trabalho de interface e áudio. Afasta do objetivo |
| C. Gravar em outro emulador que exporte filme (ex.: BizHawk) e converter | Pronto, com gamepad | Sincronia com o oráculo não é garantida (R5). Cada partida passa pela conferência de TAS e pode ser descartada. Ruim para testes ocultos, que não podem ser descartados |

**Recomendação: A.** Fallback: C só para o corpus (nunca para testes ocultos). A 2.2 começa lendo o código do frontend do jgenesis para confirmar que o patch cabe em dezenas de linhas. Se não couber, a 2.2 para e o fallback C entra, com a decisão registrada.

## 4. Traces do corpus e determinismo

- **Demos internas:** as demos rodam sozinhas na tela de título sem entrada alguma. O caso é uma lista só de zeros. A duração em quadros vem de uma execução de reconhecimento (`oraculo perfil`, nova): imprime a cada 60 quadros o modo de jogo, a zona e o ato (números, vindos do mapa 2.0) e nada mais. Marcelo cola essa lista (metadado) e fixo o número de quadros de cada demo.
- **Saídas**, tudo em `$SONIC1_EXT_DIR/privado/traces/`: `<nome>.s1t` (formato da 1.2) e `<nome>.<quadro>.s1e` (estado salvo) a cada K quadros, K=600 por padrão. Nada disso entra no git. A extensão `.s1t` e `.s1e` precisa entrar nas listas de bloqueio (R9, patch na seção 11).
- **No git** só vão: `corpus/*.s1in`, `corpus/*.meta` e, no manifesto, o `hash_do_trace` de cada caso.
- **Determinismo:** `cargo xtask corpus-trace` roda cada caso duas vezes e compara com `cmp`, como em `roda-oraculo.sh`. Além disso: recarrega o estado do quadro K e roda até o fim, e os hashes dos quadros restantes têm de ser iguais aos da execução completa. Isso prova que "estados a intervalos" servem para regenerar um quadro.
- Mudança necessária no oráculo (congelado): `rodar --estado-a-cada N` (pedido P1, patch entregue na 2.4).

## 5. Inventários

- **Hardware** (`docs/inventario-hardware.md`): registradores da VDP escritos e valores distintos; modos de DMA usados (de 68000 para VRAM, preenchimento, cópia); endereços de E/S tocados no 68000 (versão, controle, BUSREQ, RESET do Z80, SRAM, TMSS); registradores do YM2612 escritos e o uso do DAC; comandos do PSG; níveis de interrupção; sinais do barramento do Z80. **Fonte:** os eventos do trace do corpus inteiro (tipos 1 a 11), decodificados por `cargo xtask inventario`. Item não observado no corpus não entra. Cada linha: o item, quantas vezes, o primeiro caso, quadro e `seq` onde apareceu. É a evidência.
- **Memória compartilhada** (`docs/inventario-memoria.md`): variáveis de RAM escritas dentro de uma interrupção e lidas ou escritas no laço principal, ou o contrário. **Fonte:** o trace atual guarda só hashes de RAM, sem endereços de acesso (R7). Precisa de um patch no oráculo (pedido P2): um mapa de bits por contexto (laço principal, VBlank, HBlank), com o endereço de cada acesso à RAM do 68000, ligado a uma opção. A saída é só o conjunto de endereços por contexto. Cruzo com `refs/rotulos.txt` para dar o nome da variável. Como verificação cruzada, a lista estática: o conjunto de nomes que as rotinas de interrupção citam nos `.asm` de texto. Divergência entre as duas listas vai para o registro.
- **Como cada item cita evidência sem copiar o disassembly:** a citação é `trace, quadro, seq` (dinâmico) mais o nome do rótulo da rotina. Nunca código, comentário ou tabela. A geração do documento passa por um filtro que recusa linhas com bytes hexadecimais longos ou mnemônicos.

## 6. Manifesto e assinatura

- **Formato:** `refs/manifesto.txt`, texto, uma linha `tipo<TAB>nome<TAB>sha256`. SHA-256, e não SHA-1, porque aqui o hash é defesa contra adulteração (os hashes do trace seguem em SHA-1, por decisão da Etapa 1). Cobre: o commit e os patches do oráculo; os arquivos de `crates/fidelidade`; a lista de exclusões; `refs/*`; cada `.s1in` e `.meta` do corpus; o `hash_do_trace` de cada caso; a lista completa de testes (nome de cada teste de cada crate, de `cargo test -- --list`); o hash de cada arquivo cifrado dos testes ocultos e a lista de zonas. Só hashes, nomes e listas.
- **Assinatura:** `refs/manifesto.txt.sig` no formato SSHSIG, criada por `ssh-keygen -Y sign -f <chave> -n websonic-manifesto refs/manifesto.txt`. A chave pública fica em `refs/assinante.pub`.
- **Ferramenta recomendada:** `ssh-keygen` do OpenSSH. Já existe no computador do Marcelo, não pede dependência nova, e aceita chave `ed25519-sk` (token FIDO2, exige toque a cada assinatura) ou `ed25519` com senha. Recomendo `ed25519-sk` se houver token, e `ed25519` com senha se não houver. O mesmo comando de verificação serve para os dois. (A exigência de toque para `-sk` vem de relatos e guias consultados em 06/11/2026; não confirmei na página de manual do OpenSSH. Confirmar com `man ssh-keygen` na 2.7.) Alternativas descartadas: minisign e age (ferramentas a mais sem ganho).
- **Verificação:** `cargo xtask manifesto verificar` roda `ssh-keygen -Y verify`. Roda só no host (o comando de fidelidade já é só local), então a imagem não muda.
- **Como o comando de fidelidade falha** (`cargo xtask fidelidade`, quatro casos, todos plantados na prova da 2.7):
  1. Assinatura inválida ou ausente.
  2. Um arquivo referenciado difere do hash do manifesto assinado (referência mudou).
  3. Um teste do manifesto não aparece no relatório da execução, ou aparece como ignorado.
  4. O relatório tem teste que não está no manifesto: **não falha**, mas ele sai marcado "não assinado" e não conta para o critério de pronto, até a próxima assinatura. Assim a IA pode criar testes novos sem poder mascarar nada.
- A IA nunca assina nem vê a chave. A prova usa uma chave descartável gerada pelo próprio teste.

## 7. Testes ocultos cifrados

- **Ferramenta:** o formato `age`, usado pelo crate Rust `age` (a página do projeto `rage` diz que quem quer usar como biblioteca deve usar esse crate; os detalhes da API estão por conferir na 2.8). Comandos novos em xtask: `ocultos cifrar`, `ocultos decifrar`, `ocultos auditar`. A cifra usa só a chave **pública** do Marcelo, então cifrar não exige segredo. A identidade privada fica no computador do Marcelo, protegida por senha (ou por token com o plugin `age-plugin-yubikey`, que o README do rage cita). A IA nunca a vê.
- **Organização:** fora do repositório, em `$SONIC1_EXT_DIR/ocultos/<zona>/<caso>.s1in.age`, com zonas `ghz mz syz lz slz sbz especial final telas`. No manifesto entram só o hash de cada arquivo cifrado e a lista de zonas. O nome do caso não revela o conteúdo.
- **Auditoria:** no contêiner de auditoria (`.devcontainer/auditoria/`, congelado; o `devcontainer.json` dele é o pedido P3 na seção 11, entregue na 2.8). A pasta `ocultos` é montada só nele. A identidade é lida de um arquivo que o Marcelo fornece na hora. A decifração é para memória ou `tmpfs`, nunca para disco.
- **Saída:** `ocultos auditar` imprime exatamente `zona: passou` ou `zona: falhou`, mais a contagem de casos da zona. Nenhum nome de caso, quadro ou hash. Erros viram um código fixo. Se um caso falhar e for mostrado à IA, o procedimento do requisitos vale: ele vira caso normal e o Marcelo grava outro.
- **Prova com dados sintéticos:** (a) cifra e decifra voltam ao mesmo SHA-1; (b) chave errada não decifra; (c) auditoria com um "sistema sob teste" falso que passa e outro que falha uma zona planejada: a saída tem só as linhas esperadas (comparação exata de texto); (d) nenhum arquivo `.s1in` claro sobra no disco depois da auditoria.
- **Primeiros casos reais (Marcelo, com o gravador 2.2):** mínimo para fechar a Etapa 2: Green Hill atos 1 a 3 (uma partida cada) e um caso curto em cada uma das outras zonas. As partidas completas de cada ato das demais zonas valem até a Etapa 14. Isto é minha leitura de "primeiros testes ocultos, por zona". Se o Marcelo ler de outro jeito, a 2.8 se adapta.

## 8. Segundo oráculo

O que a Etapa 1 exigiu e o segundo oráculo precisa dar para "concordar nas demos":

- Nível mínimo da Etapa 2 (recomendado): **hashes por quadro** de `ram68k`, `ram_z80`, `vram`, `cram`, `vsram`, mais `regs_vdp`, com as mesmas entradas. Isso prova o estado de jogo. Eventos e ciclos exatos (nível 2) ficam para o BACKLOG, porque o jgenesis só dá o ciclo no início da instrução (V5) e talvez nenhum candidato dê mais.

| | BlastEm | Genesis Plus GX |
| --- | --- | --- |
| O que a documentação diz (fontes abaixo) | Depurador próprio (`-d`), stub de gdb (`-D`), linguagem de script no depurador, savestates. Pontos de trace e watch "não são suportados". Modo sem tela não aparece | Descrito como focado em precisão. A página do projeto não diz nada sobre modo sem tela, filmes ou depurador |
| Como entraria | Patch em C: gancho por quadro para gravar hashes e ler entradas de arquivo. Precisa de janela ou de um modo nulo de vídeo, a confirmar no código | Compilado como núcleo libretro e carregado por um pequeno programa hospedeiro sem tela. A memória sai pela API do libretro (confirmar nos cabeçalhos do libretro na 2.1) |
| Linguagem e build | C, no host. O repositório é em retrodev.com (hg) | C, no host. O repositório é no GitHub |
| Precisão de tempo | Relatada como alta, não verificada por mim | Menos apertada em tempo, segundo o que sei; **não verifiquei**. O requisitos o cita como referência do OpenGGF |
| Custo estimado | Médio a alto | Baixo a médio |

Fontes: README do BlastEm (retrodev.com/repos/blastem/file/tip/README, lido em 06/11/2026); README do Genesis Plus GX (github.com/ekeeke/Genesis-Plus-GX, lido em 06/11/2026); README do jgenesis (github.com/jsgroth/jgenesis, lido em 06/11/2026). Tudo o que está marcado "a confirmar" é verificado na 2.1 e nada vira "provado" antes disso.

- **Recomendação:** começar pelo Genesis Plus GX via libretro (menor custo, só estado por quadro), e deixar o BlastEm como fallback e como árbitro de tempo se a 2.1 mostrar que ele aceita o gancho por quadro. Os dois constroem no host, então não pedem domínio novo na rede do contêiner.
- **2.1, o teste curto:** compilar o candidato, rodar 600 quadros sem entrada com a ROM real, imprimir os hashes por quadro e comparar com `oraculo rodar`. Primeiro com a ROM sintética (a IA roda), depois com a real (Marcelo). Riscos já conhecidos: alinhamento do quadro (onde o quadro começa em cada emulador) e o estado inicial (zeros no jgenesis; indefinido no real). Se os hashes difiram desde o quadro 0 só por alinhamento, a comparação passa a ocorrer em um ponto fixo (início do VBlank) e isso se registra.
- **Registro das diferenças:** `docs/conhecimento/oraculos.md`, uma tabela com id, caso, quadro, categoria, o que difere, causa conhecida ou hipótese, evidência, decisão. Decisão pelas regras do requisitos: documentação do hardware e testes publicados primeiro; sem evidência, vale o oráculo principal e a diferença fica registrada. Nuked-MD como árbitro só em trechos curtos, e só se a 2.5 deixar uma diferença sem decisão.
- Oráculos ficam em workspace separado (`oracle/`), fora do build do site e do `core`.

## 9. Decisões que dependem do Marcelo (com a minha recomendação aplicada por padrão)

| Decisão | Recomendação adotada no plano |
| --- | --- |
| Ferramenta de assinatura e chave | `ssh-keygen -Y`, chave `ed25519-sk` se houver token, senão `ed25519` com senha. Chave em `~/.ssh/websonic_assinatura`, nunca em `SONIC1_EXT_DIR` nem em pasta montada. Pública em `refs/assinante.pub` |
| Chave dos testes ocultos | Par `age` próprio, identidade em `~/.config/websonic/ocultos.key` protegida por senha. Pública em `refs/ocultos.pub` |
| ROM e demos que o oráculo precisa | ROM em `~/sonic1-ext/rom/` (já é assim). As demos são da própria ROM e não existem como arquivo. TAS baixados em `$SONIC1_EXT_DIR/tas/`. Traces e estados em `$SONIC1_EXT_DIR/privado/traces/` |
| Domínios novos | **Nenhum**, se os oráculos forem construídos no host (como na Etapa 1). `dominios.txt` não muda. Se algum dia o BlastEm for construído no contêiner, o pedido seria `retrodev.com` |
| Onde obter TAS | TASVideos (a referência de hashes do projeto já é de lá). **Licença não verificada nesta sessão.** Regra: baixar só o que a página permitir, guardar fora do git, e commitar só os botões convertidos com o link e o autor no `.meta`. Verificar a licença no site antes do primeiro download |

## 10. Riscos e o que NÃO será feito

Riscos:
- O segundo oráculo pode não expor o necessário (mitigação: 2.1 primeiro; só estado por quadro).
- O patch do gravador pode não caber no frontend (fallback C).
- O inventário de memória depende de patch novo no oráculo (P2).
- TAS perdem a sincronia por estado inicial (esperado; descartados).
- O mapa de rótulos corrigido pode revelar mais problemas de parser (2.0 vem antes de tudo).
- A etapa pode passar de uma sessão (corte padrão na seção 1).

Não será feito:
- Nenhuma leitura da ROM nem de binários do disassembly pela IA.
- Nenhum trace, estado, ROM, imagem ou som no git.
- Assinar, ver a chave ou ver os testes ocultos reais.
- Editar o oráculo, o comparador ou as exclusões congelados. Mudanças vão como patch.
- Comparar eventos e ciclos exatos com o segundo oráculo (nível 2, backlog).
- Gravar as partidas completas de todas as zonas (valem até a Etapa 14).
- Mexer em sandbox, hooks ou CI além do patch da seção 11.

## 11. Itens congelados que precisam do Marcelo

Patches prontos:
- **P0, extensões de trace e estado (R9):** `docs/pedidos/extensoes-trace.patch`. Edita `hooks/lib.sh`, `.gitignore` e `xtask/src/anti_rom.rs` juntos (o teste do xtask confere que as listas coincidem). Aplicar com `patch -p1 < docs/pedidos/extensoes-trace.patch` e rodar `sh .devcontainer/testes/hooks.sh` e `cargo test -p xtask`. Fazer antes da 2.4.

Patches a entregar na sub-etapa indicada (dependem de código que ainda não existe):
- **P1** (2.4): `oracle/`, `rodar --estado-a-cada N`.
- **P2** (2.6): `oracle/`, mapa de bits de acessos à RAM por contexto.
- **P2b** (2.2): `oracle/patches/`, o patch do gravador no frontend do jgenesis, com a atualização de `oracle/patches/SHA1` pelo `gera-patches.sh`.
- **P3** (2.8): `.devcontainer/auditoria/devcontainer.json` e o roteiro de montagem.
- **P4** (2.7 e 2.8): `refs/assinante.pub`, `refs/ocultos.pub` e `refs/manifesto.txt.sig`, que só o Marcelo cria.
- **P5** (a confirmar na 2.7): se o `cargo xtask fidelidade` precisar do `openssh-client` na imagem. Pelo plano não precisa.

Sem mudança prevista em `managed-settings.json`, `Dockerfile`, `dominios.txt` ou `.github/`. Logo a revisão final não reaudita sandbox, hooks nem CI, a menos que algum desses mude.

## 12. Backlog gerado por este plano

Está em `docs/backlog.md`.
