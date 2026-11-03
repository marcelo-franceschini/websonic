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
for item in CLAUDE.md .gitignore docs/requisitos.md hooks refs .github .devcontainer; do
    base="//workspaces/websonic/$item"
    grep -q "\"Edit($base)\"\|\"Edit($base/\*\*)\"" "$ms" || falha "falta a regra Edit de $item."
    grep -q "\"/workspaces/websonic/$item\"" "$ms" || falha "falta o denyWrite de $item."
done

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
