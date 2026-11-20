#!/bin/sh
# Teste dos hooks com a ROM REAL. Rode no HOST, pelo Marcelo. A IA nunca roda este script.
# Copia a ROM para um repositorio descartavel (fora do projeto, sem remoto) com nomes
# disfarcados e confere que o commit e recusado. Nao mostra nem guarda o conteudo.
set -u

aqui=$(cd "$(dirname "$0")/../.." && pwd)
: "${SONIC1_EXT_DIR:?defina SONIC1_EXT_DIR}"
rom=$(ls "$SONIC1_EXT_DIR"/rom/*.gen 2>/dev/null | head -n 1)
[ -n "$rom" ] || { echo "nenhum .gen em $SONIC1_EXT_DIR/rom"; exit 1; }

d=$(mktemp -d)
trap 'rm -rf "$d"' EXIT
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t

git init -q -b main "$d/r" && cd "$d/r" || exit 1
cp -r "$aqui/refs" refs
git add refs && git commit -q -m base
cp "$aqui/hooks/pre-commit" "$aqui/hooks/pre-push" "$aqui/hooks/lib.sh" .git/hooks/
chmod +x .git/hooks/pre-commit .git/hooks/pre-push

falhas=0
for nome in jogo.gen disfarce.txt; do
    cp "$rom" "$nome"
    git add -f "$nome"
    if git commit -q -m t >"$d/saida" 2>&1; then
        echo "FALHA $nome: a ROM entrou no commit"; falhas=$((falhas + 1))
    else
        # So conta como prova se o hash foi reconhecido (a linha diz "e a ROM").
        grep -q "e a ROM de referencia" "$d/saida" \
            && echo "ok    $nome bloqueado, hash reconhecido" \
            || { echo "FALHA $nome: bloqueado, mas sem reconhecer o hash"; falhas=$((falhas + 1)); }
    fi
    git reset -q; git clean -fdq
done
[ "$falhas" = 0 ]
