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

Limites desta prova:
- Rodou por `docker run`, não pelo `devcontainer up`. O caminho do `postStartCommand` ainda não foi exercitado.
- O firewall não impede escrita HTTPS no GitHub. Só a falta de credenciais impede o push. O teste acima prova a falha sem credenciais, não um bloqueio de rede.
- O `claude doctor` sem login não acusou erro, mas isso não prova que o Claude Code aceitou o `managed-settings.json`.

## Permissões (sub-etapa 0.2, parte 2: pendente)

Não provado ainda. Exige uma sessão do Claude Code logada dentro do contêiner aberto pelo `devcontainer.json`. O roteiro está em `.devcontainer/testes/permissoes.md`. O resultado de cada teste entra aqui, com o comando e o código de retorno.

Ponto de decisão já registrado: se o sandbox deixar `cat`, `od`, `cp` ou um programa compilado lerem a ROM, a sub-etapa para e a decisão D1 volta ao Marcelo.
