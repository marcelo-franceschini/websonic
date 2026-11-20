#!/bin/sh
# Teste dos hooks do git (sub-etapa 0.3) num repositorio descartavel.
# Usa so arquivos sinteticos. Nao toca na ROM, no remoto real nem na config do projeto.
# O "remoto" e uma pasta local vazia, sem rede.
set -u

aqui=$(cd "$(dirname "$0")/../.." && pwd)
d=$(mktemp -d)
trap 'rm -rf "$d"' EXIT

# Ambiente isolado: sem config global ou de sistema do usuario.
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t

total=0
falhas=0
# resultado <nome> <codigo esperado: 0 passa, 1 bloqueia> <codigo real>
resultado() {
    total=$((total + 1))
    if { [ "$2" = 0 ] && [ "$3" = 0 ]; } || { [ "$2" = 1 ] && [ "$3" != 0 ]; }; then
        echo "ok    $1"
    else
        echo "FALHA $1 (esperado $2, codigo $3)"
        falhas=$((falhas + 1))
    fi
}

instala_hooks() {
    cp "$aqui/hooks/pre-commit" "$aqui/hooks/pre-push" "$aqui/hooks/lib.sh" "$d/r/.git/hooks/"
    chmod +x "$d/r/.git/hooks/pre-commit" "$d/r/.git/hooks/pre-push"
}

# Tenta commitar o que esta no indice. Limpa o indice se for bloqueado.
commita() {
    git commit -q -m t >"$d/saida" 2>&1
    rc=$?
    [ "$rc" = 0 ] || { git reset -q; git clean -fdq; }
    return "$rc"
}

# Repositorio descartavel com os refs do projeto, mas com um "hash da ROM" sintetico.
git init -q -b main "$d/r" && cd "$d/r" || exit 1
cp -r "$aqui/refs" refs
printf 'conteudo sintetico que faz o papel da ROM\n' > "$d/falsa"
blob=$(git hash-object "$d/falsa")
sed -i "s/^blob_id:.*/blob_id: $blob/" refs/rom-hashes.txt
git add refs && git commit -q -m base
instala_hooks

echo "== pre-commit =="
echo ola > limpo.txt
git add limpo.txt; commita; resultado "arquivo limpo passa" 0 $?

echo x > a.gen
git add -f a.gen; commita; resultado "extensao .gen bloqueada" 1 $?

echo x > B.GEN
git add -f B.GEN; commita; resultado "extensao em maiuscula bloqueada" 1 $?

echo x > "Sonic [!] (W).gen"
git add -f "Sonic [!] (W).gen"; commita; resultado "nome com espacos e colchetes bloqueado" 1 $?

echo x > "com espaco.txt"
git add "com espaco.txt"; commita; resultado "nome com espacos limpo passa" 0 $?

head -c 262143 /dev/zero > quase.txt
git add quase.txt; commita; resultado "262143 bytes passa" 0 $?

head -c 262144 /dev/zero > limite.txt
git add limite.txt; commita; resultado "262144 bytes bloqueado (>=)" 1 $?

cp "$d/falsa" disfarce.txt
git add disfarce.txt; commita; resultado "hash da ROM com nome inocente bloqueado" 1 $?

printf 'disfarce.txt\n' >> refs/excecoes-extensoes.txt
cp "$d/falsa" disfarce.txt
git add disfarce.txt; commita; resultado "excecao nao libera o hash" 1 $?
git checkout -q -- refs

printf 'permitido.bin\n' >> refs/excecoes-extensoes.txt
echo x > permitido.bin
git add -f permitido.bin; commita; resultado "arquivo na lista de excecoes passa" 0 $?
git checkout -q -- refs

mv refs/limite-tamanho.txt "$d/guardado"
echo ok > outro.txt
git add outro.txt; commita; resultado "refs ausentes bloqueia (falha fechada)" 1 $?
mv "$d/guardado" refs/limite-tamanho.txt

echo "== pre-push =="
git init -q --bare -b main "$d/remoto.git"
git remote add origin "$d/remoto.git"

git push -q origin main >"$d/saida" 2>&1; resultado "push limpo passa" 0 $?
antes=$(git -C "$d/remoto.git" rev-parse main)

# Simula o commit que escapou do pre-commit tirando o hook do repositorio descartavel.
rm .git/hooks/pre-commit
# Conteudo unico: o git guarda por conteudo, e um "x" igual ja foi enviado antes.
echo conteudo-que-vazou > vazou.gen
git add -f vazou.gen && git commit -q -m vazou
git rm -q -f vazou.gen && git commit -q -m apagou
instala_hooks

git push -q origin main >"$d/saida" 2>&1; resultado "push com ROM no historico bloqueado" 1 $?
depois=$(git -C "$d/remoto.git" rev-parse main)
[ "$antes" = "$depois" ]; resultado "remoto nao recebeu nada" 0 $?

echo "$total testes, $falhas falhas"
[ "$falhas" = 0 ]
