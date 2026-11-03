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

## Permissões (sub-etapa 0.2, parte 2: rodada 1 bloqueada pelo Bash; correção aplicada, rodada 2 pendente)

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
