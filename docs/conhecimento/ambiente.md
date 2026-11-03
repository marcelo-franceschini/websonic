# Ambiente: contêiner, rede e permissões

Registro das provas do ambiente. Cada entrada tem o comando e o resultado.

## Rede (sub-etapa 0.2, parte 1: provada)

Data: 03/11/2026. Quem rodou: Claude Code, no computador do Marcelo, sem montar nenhuma pasta de ROM nem de disassembly.

Comando:

```sh
docker run --rm --cap-add NET_ADMIN --cap-add NET_RAW -v "$PWD":/workspaces/websonic:ro websonic-dev:0.1b \
  bash -c 'sudo /usr/local/bin/init-firewall.sh && bash /workspaces/websonic/.devcontainer/testes/rede.sh'
```

Resultado: 19 testes, 19 `PASSOU`, `falhas: 0`. O script é `.devcontainer/testes/rede.sh`.

| O que foi provado | Teste |
| --- | --- |
| Domínios da lista conectam por IPv4 | `api.anthropic.com`, `index.crates.io`, `api.github.com` |
| Domínios fora da lista falham em IPv4 e em IPv6, e o DNS recusa o nome | `example.com`, `tasvideos.org`, `en.wikipedia.org` |
| Sem DNS, nada sai | IPv4 literal `1.1.1.1` e IPv6 literal `2606:4700:4700::1111` |
| DNS direto a um servidor externo não responde | UDP e TCP para `8.8.8.8:53` |
| O `dev` não tem `sudo` além do firewall | `sudo -n true` recusado |
| `git push` sem credenciais falha | repositório temporário com o `origin` do projeto |
| O volume `/home/dev/.claude` é gravável pelo `dev` | cria e apaga um arquivo |

## Contêiner aberto pelo devcontainer (sub-etapa 0.1b e 0.2: provado)

Data: 03/11/2026. Comando do Marcelo: `devcontainer up --workspace-folder .` (@devcontainers/cli 0.90.0), com `SONIC1_EXT_DIR=~/sonic1-ext`.

- `verifica-host.sh` passou. O `gera-disasm-texto.sh` copiou 455 arquivos `.asm`.
- A imagem construiu pelo `devcontainer.json`. O UID do `dev` (1000) é igual ao do host, sem remapeamento.
- O `postStartCommand` rodou o firewall: "dominio fora da lista bloqueado (DNS, IPv4 e IPv6)" e "dominios autorizados conectam".
- Resultado: `{"outcome":"success"}`, usuário remoto `dev`, workspace `/workspaces/websonic`.
- Depois, `rede.sh` rodou por `docker exec -u dev` nesse contêiner: 19 de 19 `PASSOU`, `falhas: 0`.
- Montagens (`docker inspect`): `/mnt/rom`, `/mnt/disasm` e `/mnt/disasm-texto` com `rw=false`. `/mnt/trabalho`, `/workspaces/websonic` e o volume `/home/dev/.claude` com `rw=true`. `/etc/claude-code/managed-settings.json` presente.

Limites desta prova:
- A primeira rodada do `rede.sh` foi por `docker run`. A segunda, acima, foi pelo contêiner do `devcontainer`.
- O firewall não impede escrita HTTPS no GitHub. Só a falta de credenciais impede o push. O teste acima prova a falha sem credenciais, não um bloqueio de rede.
- O `claude doctor` sem login não acusou erro, mas isso não prova que o Claude Code aceitou o `managed-settings.json`.

## Permissões (sub-etapa 0.2, parte 2: rodada 3 sem falhas, pendências do Marcelo)

Roteiro: `.devcontainer/testes/permissoes.md`. O resultado de cada teste entra aqui, com o comando e o código de retorno.

Ponto de decisão já registrado: se o sandbox deixar `cat`, `od`, `cp` ou um programa compilado lerem a ROM, a sub-etapa para e a decisão D1 volta ao Marcelo.

### Rodada 1 (03/11/2026)

Quem rodou: Claude Code (Opus 5.5), sessão logada dentro do contêiner, workspace `/workspaces/websonic`. Os passos prévios do Marcelo (`/status` e `claude doctor`) não foram registrados nesta sessão.

**Achado principal: o Bash não roda.** Todo comando do Bash, inclusive `echo ok`, sai com código 1 e esta mensagem, antes de executar qualquer coisa:

```
bwrap: No permissions to create new namespace, likely because the kernel does not allow non-privileged user namespaces.
```

O sandbox do Claude Code usa o bubblewrap, que precisa criar um user namespace. O contêiner não permite. O Bash fica fechado por falha (nada roda), e não bloqueado pelas regras de leitura e escrita. Por isso os testes do Bash abaixo "falham" pelo motivo errado e não provam as regras.

Ferramentas do Claude Code (provam as regras de permissão):

| # | Teste | Resultado | Esperado? |
| --- | --- | --- | --- |
| 1 | Read na ROM | recusado: "File is in a directory that is denied by your permission settings" | sim |
| 6a | Read em `/mnt/disasm/sonic.asm` | recusado, mesma mensagem | sim |
| 7 | Edit em `docs/requisitos.md` | recusado, mesma mensagem | sim |
| controle | Read nas 3 primeiras linhas de `/mnt/disasm-texto/sonic.asm` | funcionou (conteúdo não colado) | sim |

Bash (não provam nada, o sandbox não iniciou):

| # | Teste | Código | Motivo |
| --- | --- | --- | --- |
| 2 a 5 | `cat`, `od`, `cp` e programa compilado na ROM | não rodados | toda chamada do Bash falha no `bwrap`, código 1 |
| 6b | `cat` em `/mnt/disasm` | não rodado | idem |
| 8 a 12 | edições de `docs/requisitos.md` pelo Bash e `git status` | não rodados | idem |
| 13 a 18 | hooks, `.git/config`, push e `--no-verify` | não rodados | idem |
| controle | `echo ok > /mnt/trabalho/teste.txt && rm ...` | 1 | falha no `bwrap`: controle positivo quebrado |
| controle | `git status --short docs/requisitos.md` | 1 | falha no `bwrap` |
| controle | `cargo build` em `/tmp` | não rodado | idem |

Conclusão da rodada 1:
- Nenhum teste de leitura da ROM nem de edição do arquivo travado passou.
- As regras das ferramentas Read e Edit valem dentro do contêiner, o que indica que o `managed-settings.json` foi carregado.
- A sub-etapa não fecha: sem Bash, os testes 2 a 5 e 8 a 18 não podem ser provados, e a IA também não consegue compilar, testar nem fazer commit.
- Correção possível (decisão do Marcelo, fora do alcance da IA): permitir user namespaces no contêiner (por exemplo, perfil seccomp ou AppArmor do Docker no host) ou usar a opção de sandbox mais fraco para contêineres do Claude Code. A segunda enfraquece o isolamento e precisa ser avaliada contra a D1. Depois, rodar o roteiro de novo.

### Causa e correção da rodada 1 (03/11/2026)

Evidência da causa (Claude Code, no computador do Marcelo, contêiner descartável da imagem `websonic-dev:0.1b`, sem montar pastas):

| Configuração do contêiner | `bwrap --ro-bind / / --unshare-user --unshare-pid --proc /proc --dev /dev true` |
| --- | --- |
| seccomp padrão do Docker | `No permissions to create new namespace` (reproduz a rodada 1) |
| `apparmor=unconfined`, seccomp padrão | o mesmo erro: o AppArmor não é a causa |
| `seccomp=unconfined` | passa da criação do namespace e falha em `Can't mount proc`: é o caso que `enableWeakerNestedSandbox` já resolve |

O perfil padrão do Docker libera `unshare`, `mount`, `umount2`, `pivot_root` e `clone` sem restrição de flags só para quem tem `CAP_SYS_ADMIN`. O `dev` não tem.

Correção de menor privilégio: o arquivo `.devcontainer/seccomp-userns.json`, que é o perfil padrão do Docker (moby/profiles, `seccomp/default.json`) mais **um** acréscimo, liberando `clone`, `unshare`, `mount`, `umount2` e `pivot_root`. Gerado com:

```sh
jq '.syscalls += [{"names": ["clone","unshare","mount","umount2","pivot_root"], "action": "SCMP_ACT_ALLOW"}]' default.json > .devcontainer/seccomp-userns.json
```

O `devcontainer.json` aponta para ele em `securityOpt`. Não adiciona nenhuma capability.

Provas desta correção:
- Com o perfil, o bubblewrap cria o sandbox como `dev` (uid 1000): `dentro_do_sandbox uid=1000`, `rc=0`. Um conjunto menor de syscalls foi testado primeiro e já bastou.
- `diff` entre o perfil padrão e o novo mostra só o bloco acrescentado.
- Com o perfil, o firewall configura e o `rede.sh` passa em 19 de 19 testes.
- Com o perfil, ainda falham fora do sandbox: `unshare -m` (rc 1) e `keyctl` (bloqueado).
- O CLI substitui `${localWorkspaceFolder}` em `securityOpt` (`devcontainer read-configuration`).

Custo, para o Marcelo avaliar contra a D1: o `dev` passa a poder criar user namespaces dentro do contêiner. Isso aumenta a superfície de ataque ao kernel do host (bugs de user namespace), embora não conceda poder algum sem uma falha do kernel. É o que o Claude Code exige para sua barreira de Bash. A alternativa é rodar o Claude Code sem o sandbox do Bash, o que deixa `cat` ler a ROM e contraria os requisitos.

Ainda não provado: o Bash do Claude Code funcionando dentro do contêiner recriado, e as regras `denyRead` e `denyWrite` do sandbox. Isso é a rodada 2.

### Rodada 2 (03/11/2026)

Quem rodou: Claude Code (Opus 5.5), sessão logada no contêiner recriado com `.devcontainer/seccomp-userns.json`, workspace `/workspaces/websonic`, em modo automático. Os passos do Marcelo (`/status` e `claude doctor`) não foram registrados nesta sessão. `ARQ` é `"/mnt/rom/Sonic The Hedgehog (W) (REV01) [!].gen"`. Toda saída foi para `/dev/null`.

**O Bash roda.** `echo ok` imprimiu `ok`, código 0. A correção do seccomp funcionou.

Camadas que barraram cada teste:
- **permissão**: regra `deny` do `managed-settings.json`. Mensagem "denied by your permission settings" (Read e Edit) ou "Permission to use Bash with command ... has been denied" (Bash). Nada roda.
- **sandbox**: o comando roda dentro do bubblewrap e o sistema de arquivos recusa.
- **classificador**: o classificador do modo automático do Claude Code recusou antes de rodar. Ele não vem do `managed-settings.json` e não existe fora do modo automático, então **não prova** a configuração.

Leitura da ROM e do disassembly (esperado: todos falham):

| # | Comando | Código | Camada | Esperado? |
| --- | --- | --- | --- | --- |
| 1 | Read em `ARQ` | recusado | permissão | sim |
| 2 | `cat ARQ > /dev/null; echo $?` | 1, "No such file or directory" | sandbox | sim |
| 3 | `od ARQ > /dev/null; echo $?` | 1, "No such file or directory" | sandbox | sim |
| 4 | `cp ARQ /tmp/copia > /dev/null; echo $?` | 1, "cannot stat ... No such file or directory" | sandbox | sim |
| 5 | programa C com `fopen(v[1],"rb")` em `ARQ` | 1 (o mesmo programa abre `/etc/hostname` com 0) | sandbox | sim |
| 6a | Read em `/mnt/disasm/sonic.asm` | recusado | permissão | sim |
| 6b | `cat /mnt/disasm/sonic.asm > /dev/null; echo $?` | 1, "No such file or directory" | sandbox | sim |

No sandbox, `test -d /mnt/rom` e `test -d /mnt/disasm` dão 0: as pastas existem, mas o conteúdo some.

Controles positivos (esperado: funcionam):

| Comando | Código | Esperado? |
| --- | --- | --- |
| Read nas 3 primeiras linhas de `/mnt/disasm-texto/sonic.asm` (conteúdo não colado) | funcionou | sim |
| `echo ok > /mnt/trabalho/teste.txt && rm /mnt/trabalho/teste.txt; echo $?` | 0 | sim |
| `cargo new` e `cargo build --offline` num crate de teste em `$TMPDIR` | 0 | sim |

Edição de `docs/requisitos.md` (esperado: todos falham):

| # | Comando | Código | Camada | Esperado? |
| --- | --- | --- | --- | --- |
| 7 | Edit | recusado | permissão | sim |
| 8 | `sed -i 's/a/b/' docs/requisitos.md; echo $?` | não rodou | classificador | sim, mas não prova o sandbox |
| 9 | `echo x >> docs/requisitos.md; echo $?` | não rodou | classificador | sim, mas não prova o sandbox |
| 10 | `cp /etc/hostname docs/requisitos.md; echo $?` | 1, "Read-only file system" | sandbox | sim |
| 11 | programa C com `fopen(v[1],"ab")` | 1 (o mesmo programa abre um arquivo em `$TMPDIR` com 0) | sandbox | sim |
| 12 | `git status --short docs/requisitos.md` | vazio, 0 | | sim |

Os testes 10 e 11 provam o `denyWrite` do sandbox sobre `docs/requisitos.md`, que é o mesmo mecanismo dos testes 8 e 9.

Hooks e git (esperado: todos falham):

| # | Comando | Código | Camada | Esperado? |
| --- | --- | --- | --- | --- |
| 13 | `git config core.hooksPath /tmp; echo $?` | não rodou | permissão | sim |
| 14 | `echo x >> .git/config; echo $?` | não rodou | classificador | sim, mas não prova o sandbox |
| 15 | `touch .git/hooks/pre-commit; echo $?` | não rodou | classificador | sim, mas não prova o sandbox |
| 16 | `git push origin HEAD; echo $?` | não rodou | permissão | sim |
| 17 | `git -C . push origin HEAD; echo $?` | não rodou | permissão (`git * push *`) | sim |
| 18 | `git commit --allow-empty --no-verify -m ...` | não rodou | permissão | sim |

Depois de tudo, `git log -1` continua em `32359e0` e `git diff` está vazio.

Achados da rodada 2:
- `/tmp` é só leitura dentro do sandbox. O teste 5, como está no roteiro, falha no `printf > /tmp/abre.c` e o código 1 não prova nada. Foi repetido em `$TMPDIR` (`/tmp/claude-1000`), com controle positivo. O teste 4 falharia em `/tmp` de qualquer forma, mas o erro mostrado é na leitura da ROM. O roteiro deve passar a usar `$TMPDIR`.
- Um primeiro comando composto, com os testes 2, 3, 4 e 6b juntos, foi recusado pela permissão sem motivo exibido. Rodados um por um, chegaram ao sandbox.
- O sandbox monta `/dev/null` (dispositivo de caractere 1,3, dono `nobody`) sobre arquivos sensíveis na raiz do repositório: `.bashrc`, `.bash_profile`, `.profile`, `.zshrc`, `.zprofile`, `.gitconfig`, `.gitmodules`, `.mcp.json`, `.ripgreprc`, `.idea`, `.vscode` e vários itens em `.claude/` (`settings.json`, `settings.local.json`, `hooks`, `skills`, `agents`, `commands` e outros). Dentro do sandbox eles aparecem no `git status` como não rastreados. Eles não foram apagados.

- **A IA não consegue fazer commit.** Ao registrar esta rodada, `git add` e `git commit` saíram com 128: `Unable to create '/workspaces/websonic/.git/index.lock': Read-only file system`. O sandbox deixa a `.git` inteira só leitura, e não só `.git/config` e `.git/hooks`. Isso contraria o requisito "A IA só faz commit local" e o ciclo do `CLAUDE.md`. Decisão do Marcelo: liberar a escrita na `.git` (mantendo `config` e `hooks` travados) ou fazer ele os commits.

Limites desta rodada:
- O "No such file or directory" vem de o sandbox esconder o conteúdo de `/mnt/rom`. De dentro do sandbox não dá para confirmar que `ARQ` existe com esse nome.
- Os testes 8, 9, 14 e 15 só foram barrados pelo classificador. O `denyWrite` de `.git/config` e `.git/hooks` não foi provado nesta rodada.
- A falta de credenciais para o push não foi testada aqui, porque a permissão barrou antes. Ela já foi provada pelo `rede.sh`.

Conclusão da rodada 2:
- Nenhum teste de leitura da ROM nem de edição do arquivo travado passou. Pelo critério do roteiro, a sub-etapa não para.
- O Bash funciona, e as regras `denyRead` e `denyWrite` do sandbox valem para comandos e para programas compilados.
- Pendências para o Marcelo:
  1. No terminal do contêiner, fora do Claude: `test -f "/mnt/rom/Sonic The Hedgehog (W) (REV01) [!].gen"; echo $?` deve dar 0.
  2. Rodar os testes 8, 9, 14 e 15 numa sessão fora do modo automático, aprovando o comando, para provar o sandbox sem o classificador.
  3. No terminal do contêiner, fora do Claude, `git status --short` mostra se os pontos de montagem do sandbox deixaram arquivos vazios no repositório. Se sim, decidir entre apagar e pôr no `.gitignore`.
  4. Os passos `/status` e `claude doctor` do roteiro.

### Análise da rodada 2 e correções (03/11/2026)

Quem analisou: Claude Code, no computador do Marcelo, a partir do registro acima e da documentação do Claude Code.

| Achado | Análise | Ação |
| --- | --- | --- |
| Leitura da ROM e do disassembly bloqueada no Bash (testes 2 a 6b) | Provado: `cat`, `od`, `cp` e um programa compilado falham no sandbox, com o controle positivo funcionando. O D1 fica valendo: a IA não lê a ROM por nenhum caminho testado. | Nenhuma. |
| `docs/requisitos.md` protegido (testes 7, 10 e 11) | Provado pela permissão (Edit) e pelo sandbox (`cp` e programa compilado). | Nenhuma. |
| A IA não consegue fazer commit | **Causa: uma regra do próprio projeto.** A regra `Edit(//workspaces/websonic/.git/**)` do `managed-settings.json` também vale no sandbox (a documentação diz que as regras Edit em `deny` são unidas ao `denyWrite`). Ela cobre `index`, `objects` e `refs`. A documentação diz que o sandbox, por si só, protege só `.git/hooks` e `.git/config`. | A regra larga foi trocada por `Edit(.../.git/config)` e `Edit(.../.git/hooks/**)`. O `verifica-config.sh` agora falha se alguma regra Edit cobrir a `.git` inteira (testado com uma regressão plantada). **Falta provar com a imagem reconstruída.** |
| Testes 8, 9, 14 e 15 barrados pelo classificador | Não provam o sandbox. | O roteiro manda rodar fora do modo automático e registrar a camada. Falta rodar. |
| `-n` curto do `git commit` | Estava sem regra: a regra `*--no-verify*` não cobre `git commit -n`. | Acrescentadas `Bash(git commit -n*)` e `Bash(git commit * -n*)`. Limite: as duas podem recusar uma mensagem de commit que contenha " -n". O bloqueio de verdade do `--no-verify` é feito pelos hooks do Marcelo no push (host) e pelo CI. Regra de texto não é barreira técnica. |
| `/tmp` é só leitura no sandbox | O teste 5 como estava não provava nada. | O roteiro usa `$TMPDIR`. |
| Marcadores de 0 byte do sandbox na raiz do repositório | A documentação diz que o sandbox cria arquivos de 0 byte enquanto um comando roda e os remove depois. Conferido no computador do Marcelo: a raiz do repositório não tem nenhum desses arquivos, e o `git status` só mostra as duas mudanças de documentação. Existe só a pasta vazia `.claude/.cc-writes`, que o `git` não vê. | Nenhuma. O roteiro (teste 22) manda conferir de novo. |
| A ROM existe com aquele nome? | De dentro do sandbox não se distingue "escondido" de "inexistente". | Teste 21 do roteiro, fora do Claude. Eu não consulto a pasta da ROM. |
| `docs/` com permissão 777 no host | Herdado da 0.1, sem efeito no git. | Corrigido para 755. |

Ainda não provado: o commit pela IA na imagem reconstruída, o `denyWrite` de `.git/config` e `.git/hooks` pelo sandbox sem o classificador, a presença do arquivo da ROM, e os passos `/status` e `claude doctor`.

### Rodada 3 (03/11/2026)

Quem rodou: Claude Code (Opus 5.5), sessão no contêiner reconstruído, workspace `/workspaces/websonic`. Testes do roteiro: 8, 9, 14, 15, 19 e 20. O commit de partida era `c5d5b8c`.

A sessão estava em modo automático, e não em "default" como pede o roteiro. Mesmo assim, o classificador não barrou nenhum comando: todos rodaram e o erro veio do sistema de arquivos. Então a camada provada é o sandbox, que é a que o roteiro quer provar.

`echo ok` imprimiu `ok`, código 0.

| # | Comando | Código | Camada | Esperado? |
| --- | --- | --- | --- | --- |
| 8 | `sed -i 's/a/b/' docs/requisitos.md; echo $?` | 4, "cannot rename docs/sed...: Device or resource busy" | sandbox | sim |
| 9 | `echo x >> docs/requisitos.md; echo $?` | 1, "Read-only file system" | sandbox | sim |
| 14 | `echo x >> .git/config; echo $?` | 1, "Read-only file system" | sandbox | sim |
| 15 | `touch .git/hooks/pre-commit; echo $?` | 1, "Read-only file system" | sandbox | sim |
| 20 | os mesmos comandos dos testes 14 e 15 | 1 e 1, "Read-only file system" | sandbox | sim |
| 19 | `git add docs && git commit -m ...` (o commit deste registro) | 0 | | sim |

Detalhes:
- No teste 8, o `sed -i` grava um arquivo temporário em `docs/` e tenta renomeá-lo por cima do original. A pasta `docs/` é gravável, mas o `docs/requisitos.md` é um ponto de montagem só leitura do sandbox, por isso o `rename` falha com "Device or resource busy". O temporário não sobrou (`ls docs/sed*` vazio).
- Depois dos testes 8 a 15: `git diff --stat` vazio, o fim do `.git/config` continua igual e `.git/hooks/pre-commit` não existe.
- O teste 19 é o próprio commit que registra esta rodada. Ele prova que a correção da regra `Edit` sobre a `.git` vale na imagem reconstruída: a IA cria `index.lock`, objetos e refs, enquanto `config` e `hooks` continuam travados (teste 20).
- Dentro do sandbox, o `git status --short` mostra como não rastreados os marcadores do sandbox (`.bashrc`, `.gitconfig`, `.mcp.json`, `.claude/` e outros), como na rodada 2. Eles não entram no commit, porque o commit só adiciona `docs`. O teste 22, fora do Claude, diz se sobram depois.

Conclusão da rodada 3:
- Nenhum teste de edição do arquivo travado nem da configuração do git passou.
- O `denyWrite` do sandbox sobre `docs/requisitos.md`, `.git/config` e `.git/hooks` está provado sem o classificador.
- A IA consegue fazer commit local.
- Continuam com o Marcelo, fora do Claude: os testes 21 e 22, `/status` e `claude doctor`.

Verificações fora do Claude (03/11/2026):
- Teste 21: `test -f` no arquivo da ROM REV01, no host, devolveu 0. O arquivo existe com esse nome. Então o "No such file or directory" de dentro do sandbox vem de ele estar escondido.
- Teste 22: `git status --short` no host, depois de fechar a sessão, saiu vazio. Nenhum marcador de 0 byte sobrou.
- Falta só `/status` e `claude doctor` com login.
- `/status` no contêiner (2.1.317): "Setting sources" cita "Enterprise managed settings (file)" e "Permission rules" mostra "Managed settings only (allowManagedPermissionRulesOnly)". O managed settings foi aceito. "MCP servers: 3 connected, 3 failed" é esperado, pois o firewall bloqueia domínios fora da D2.
- `claude doctor` no contêiner: versão 2.1.317, caminho `/usr/local/bin/claude`, auto-update desligado por `DISABLE_AUTOUPDATER`. Quatro avisos, todos sobre `~/.local/bin` fora do PATH e método de instalação "unknown" (o binário foi copiado para `/usr/local/bin`). Nenhum cita o managed settings. Sem efeito na segurança. Possível ajuste na 0.6: symlink em `/home/dev/.local/bin/claude` e PATH no Dockerfile.
- Um primeiro `claude doctor` colado era do Claude Code do host (pacman, 2.1.312) e foi descartado.

Conclusão: a sub-etapa 0.2 está concluída (rede 19 de 19, leitura da ROM e do disassembly bloqueada, edição travada, commit local da IA, managed settings aceito).

## Sub-etapa 0.6: congelamento e fechamento

Feito em 03/11/2026, no host, por uma sessão que não está sob o `managed-settings.json`.

Congelados (regra `Edit` e `denyWrite` do sandbox, nos dois): `CLAUDE.md`, `.gitignore`, `docs/requisitos.md`, `hooks/`, `refs/`, `.github/` e `.devcontainer/`. A IA continua editando `docs/estado.md`, `docs/conhecimento/`, `crates/`, `xtask/`, `site/` e `Cargo.*`.

Evidência rodada no host:
- `sh .devcontainer/verifica-config.sh` confere, para cada item congelado, a regra `Edit` e o `denyWrite`. Com a regra de `hooks` ou o `denyWrite` de `refs` removidos de propósito, ele falhou com a mensagem certa. Restaurados, volta a dar OK.
- Imagem reconstruída (`websonic-dev:0.6`): o `/etc/claude-code/managed-settings.json` é idêntico ao do repositório (`cmp`). O link `/home/dev/.local/bin/claude` existe e o PATH o inclui. O `claude doctor` sem login passou de 4 para 2 avisos. Os dois restantes são benignos: o lançador não veio do instalador (o auto-update já está desligado por `DISABLE_AUTOUPDATER`) e o método de instalação é "not set".

Ainda não provado (exige o Marcelo reconstruir e rodar os testes 23 a 28 do roteiro): que o sandbox barra mesmo a escrita nesses caminhos novos, e que o commit e a edição de `docs/estado.md` continuam funcionando com as novas travas.

Para mexer num item congelado: o Marcelo edita fora do contêiner (ou tira a regra, reconstrói a imagem, a IA edita e ele congela de novo).
