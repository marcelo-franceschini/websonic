#!/bin/sh
# Confere a configuracao do contêiner sem precisar do Docker. Pode rodar no host ou no contêiner.
# 1. A lista de dominios do managed-settings.json coincide com a de dominios.txt.
# 2. As chaves que so valem em certo nivel estao no nivel certo (chave no lugar errado e ignorada).
# 3. O caminho do workspace e unico e nao ha travessoes nos arquivos de configuracao.
set -eu

cd "$(dirname "$0")"
ms=managed-settings.json
falhou=0
falha() {
    echo "FALHA: $*" >&2
    falhou=1
}

# 1. Dominios: os do JSON (entre "allowedDomains": [ e ]) contra os de dominios.txt.
json=$(sed -n '/"allowedDomains": \[/,/\]/p' "$ms" | grep -o '"[a-z0-9.-]*\.[a-z]*"' | tr -d '"' | sort)
txt=$(sed -e 's/#.*//' -e 's/[[:space:]]//g' dominios.txt | sed '/^$/d' | sort)
[ -n "$json" ] || falha "nenhum dominio lido do $ms."
[ "$json" = "$txt" ] || falha "a lista de dominios do $ms difere de dominios.txt."

# 2. Niveis das chaves. O arquivo usa 2 espacos por nivel.
grep -q '^  "allowManagedPermissionRulesOnly": true' "$ms" \
    || falha "allowManagedPermissionRulesOnly deve ficar na raiz."
grep -q '^    "disableBypassPermissionsMode": "disable"' "$ms" \
    || falha "disableBypassPermissionsMode deve ficar em permissions."
grep -q '^      "allowManagedDomainsOnly": true' "$ms" \
    || falha "allowManagedDomainsOnly deve ficar em sandbox.network."
grep -q '^    "network": {' "$ms" || falha "falta sandbox.network."
grep -q '^    "filesystem": {' "$ms" || falha "falta sandbox.filesystem."
grep -q '^    "failIfUnavailable": true' "$ms" || falha "falta sandbox.failIfUnavailable."
if grep -q '^    "allowedDomains"\|^    "allowManagedDomainsOnly"' "$ms"; then
    falha "allowedDomains/allowManagedDomainsOnly fora de sandbox.network."
fi

# 2a. Nenhuma regra Edit pode cobrir a .git inteira: a regra tambem vale no sandbox e impede o commit.
#     So config e hooks ficam travados.
if grep -q 'Edit(//workspaces/websonic/.git/\*\*)' "$ms"; then
    falha "regra Edit cobre a .git inteira. Use so .git/config e .git/hooks/**."
fi

# 2c. Itens congelados (0.6): cada um precisa da regra Edit E do denyWrite do sandbox.
#     So o denyWrite vale para qualquer programa; so a regra Edit vale para as ferramentas.
# 0.7: entram tambem a configuracao do Claude Code no projeto (.claude, .mcp.json, .vscode)
#      e a configuracao do cargo (.cargo, rust-toolchain.toml).
for item in CLAUDE.md .gitignore docs/requisitos.md hooks refs .github .devcontainer \
    .claude .mcp.json .vscode .cargo rust-toolchain.toml; do
    base="//workspaces/websonic/$item"
    grep -q "\"Edit($base)\"\|\"Edit($base/\*\*)\"" "$ms" || falha "falta a regra Edit de $item."
    grep -q "\"/workspaces/websonic/$item\"" "$ms" || falha "falta o denyWrite de $item."
done

# 2d. Hooks e MCP (0.7). Sem estas chaves, um hook escrito em .claude/settings.json roda
#     fora do sandbox, como dev (sonda S1). As tres ficam na raiz, e a lista de MCP fica vazia
#     (lista vazia = nenhum servidor do usuario ou do projeto).
grep -q '^  "allowManagedHooksOnly": true' "$ms" \
    || falha "allowManagedHooksOnly deve ficar na raiz, ligado."
grep -q '^  "allowManagedMcpServersOnly": true' "$ms" \
    || falha "allowManagedMcpServersOnly deve ficar na raiz, ligado."
grep -q '^  "allowedMcpServers": \[\],\?$' "$ms" \
    || falha "allowedMcpServers deve ficar na raiz e vazio ([])."
# Regras Edit que alcancam arquivos aninhados do cargo (config e toolchain em qualquer pasta).
for regra in '**/.cargo/**' '**/rust-toolchain' '**/rust-toolchain.toml'; do
    grep -qF "\"Edit(//workspaces/websonic/$regra)\"" "$ms" || falha "falta a regra Edit de $regra."
done
# O .claude.json do usuario fica em CLAUDE_CONFIG_DIR. Se a variavel mudar de lugar, a regra
# Edit(//home/dev/.claude/**) deixa de cobri-lo.
grep -q '"CLAUDE_CONFIG_DIR": "/home/dev/.claude"' devcontainer.json \
    || falha "CLAUDE_CONFIG_DIR deve ser /home/dev/.claude (o .claude.json do usuario fica la)."
grep -qF '"Edit(//home/dev/.claude/**)"' "$ms" || falha "falta a regra Edit de /home/dev/.claude/**."

# 2e. C8 (0.7): o sandbox so grava em registry e git do CARGO_HOME, e nao no CARGO_HOME inteiro.
#     Assim a IA nao escreve /home/dev/.cargo/config.toml (rustflags, [env], rustc-wrapper).
#     O cargo continua baixando crates: .global-cache e .package-cache da raiz sao tolerados.
grep -q '^        "/home/dev/.cargo/registry",' "$ms" || falha "falta allowWrite de /home/dev/.cargo/registry."
grep -q '^        "/home/dev/.cargo/git"' "$ms" || falha "falta allowWrite de /home/dev/.cargo/git."
if grep -q '^        "/home/dev/.cargo"' "$ms"; then
    falha "allowWrite de /home/dev/.cargo inteiro. Use so registry e git."
fi

# 2b. Perfil seccomp que permite o sandbox do Claude Code (user namespaces) no contêiner.
perfil=seccomp-userns.json
[ -f "$perfil" ] || falha "falta $perfil."
grep -q '"pivot_root"' "$perfil" || falha "$perfil nao libera os syscalls do sandbox."
if command -v jq >/dev/null 2>&1; then
    jq -e . "$perfil" >/dev/null 2>&1 || falha "$perfil nao e um JSON valido."
fi
grep -q 'seccomp=${localWorkspaceFolder}/.devcontainer/seccomp-userns.json' devcontainer.json \
    || falha "devcontainer.json nao aponta para $perfil em securityOpt."

# 3. Caminho unico do workspace e ausencia de travessoes.
if grep -n --exclude=verifica-config.sh 'workspaces/sonic1' ./* ../README.md 2>/dev/null; then
    falha "caminho /workspaces/sonic1 encontrado. O unico caminho e /workspaces/websonic."
fi
if grep -n -P --exclude=verifica-config.sh '[\x{2013}\x{2014}]' ./* 2>/dev/null; then
    falha "travessao encontrado."
fi

if [ "$falhou" -eq 0 ]; then
    echo "OK: configuracao consistente."
fi
exit "$falhou"
