#!/bin/sh
# Teste do .github/anti-rom.sh (sub-etapa 0.7) em repositorios descartaveis.
# Usa so arquivos sinteticos. Nao toca na ROM nem no repositorio real.
# Casos negativos: extensao, tamanho no limite (262144), blob exato (blob_id trocado na copia
# dos refs), arquivo que so existe no historico, historico raso e refs ausente.
set -u

aqui=$(cd "$(dirname "$0")/../.." && pwd)
d=$(mktemp -d)
trap 'rm -rf "$d"' EXIT
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t

total=0
falhas=0
# resultado <nome> <esperado: 0 passa, 1 bloqueia> <codigo real>
resultado() {
    total=$((total + 1))
    if { [ "$2" = 0 ] && [ "$3" = 0 ]; } || { [ "$2" = 1 ] && [ "$3" != 0 ]; }; then
        echo "ok    $1"
    else
        echo "FALHA $1 (esperado $2, codigo $3)"
        falhas=$((falhas + 1))
    fi
}

# Cria um repositorio com os scripts e os refs do projeto (como um checkout do CI).
novo() {
    rm -rf "$d/r"
    git init -q -b main "$d/r" && cd "$d/r" || exit 1
    mkdir -p .github hooks
    cp "$aqui/.github/anti-rom.sh" .github/
    cp "$aqui/hooks/lib.sh" hooks/
    cp -r "$aqui/refs" refs
    git add .github hooks refs && git commit -q -m base
}
roda() { sh .github/anti-rom.sh >"$d/saida" 2>&1; }

echo "== repositorio limpo =="
novo
echo ola > limpo.txt && git add limpo.txt && git commit -q -m limpo
roda; resultado "repositorio limpo passa" 0 $?

echo "== extensao =="
novo; echo x > a.gen; git add -f a.gen; git commit -q -m x
roda; resultado "extensao .gen bloqueada" 1 $?
grep -q 'extensao proibida' "$d/saida" || { echo "FALHA mensagem de extensao"; falhas=$((falhas + 1)); }

echo "== tamanho =="
novo; head -c 262143 /dev/zero | tr '\0' a > ok.txt; git add ok.txt; git commit -q -m t1
roda; resultado "262143 bytes passa" 0 $?
novo; head -c 262144 /dev/zero | tr '\0' a > limite.txt; git add limite.txt; git commit -q -m t2
roda; resultado "262144 bytes bloqueado" 1 $?
grep -q 'limite' "$d/saida" || { echo "FALHA mensagem de tamanho"; falhas=$((falhas + 1)); }

echo "== blob exato (hash trocado na copia dos refs) =="
novo
printf 'conteudo sintetico que faz o papel da ROM\n' > nota.txt
blob=$(git hash-object nota.txt)
sed -i "s/^blob_id:.*/blob_id: $blob/" refs/rom-hashes.txt
git add refs nota.txt && git commit -q -m blob
roda; resultado "blob com o id da ROM bloqueado, nome inocente" 1 $?
grep -q 'ROM de referencia' "$d/saida" || { echo "FALHA mensagem de hash"; falhas=$((falhas + 1)); }

echo "== so no historico =="
novo; echo x > velho.gen; git add -f velho.gen; git commit -q -m add
git rm -q -f velho.gen && git commit -q -m remove
[ ! -e velho.gen ] || { echo "FALHA preparo"; falhas=$((falhas + 1)); }
roda; resultado "arquivo apagado, so no historico, bloqueado" 1 $?

echo "== historico raso =="
novo; echo ola > l.txt; git add l.txt; git commit -q -m l2
rm -rf "$d/raso"; git clone -q --depth 1 "file://$d/r" "$d/raso" 2>/dev/null; cd "$d/raso" || exit 1
roda; resultado "clone raso reprovado" 1 $?

echo "== refs ausente (falha fechada) =="
novo; git rm -q -r refs && git commit -q -m semrefs
roda; resultado "sem refs/ bloqueia" 1 $?

echo
echo "total: $total, falhas: $falhas"
[ "$falhas" -eq 0 ]
