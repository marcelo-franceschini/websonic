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

## Permissões (sub-etapa 0.2, parte 2: pendente)

Não provado ainda. Exige uma sessão do Claude Code logada dentro do contêiner aberto pelo `devcontainer.json`. O roteiro está em `.devcontainer/testes/permissoes.md`. O resultado de cada teste entra aqui, com o comando e o código de retorno.

Ponto de decisão já registrado: se o sandbox deixar `cat`, `od`, `cp` ou um programa compilado lerem a ROM, a sub-etapa para e a decisão D1 volta ao Marcelo.
