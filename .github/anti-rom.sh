#!/bin/sh
# Anti-ROM do CI, em shell. Roda no runner, logo depois do checkout e ANTES de construir a imagem.
#
# Por que em shell e fora da imagem: o Dockerfile e o xtask (Rust) podem ser alterados por um
# commit ruim. Este script roda antes de qualquer coisa do repositorio ser construida ou executada.
# Ele usa a mesma regra dos hooks (hooks/lib.sh): hash do blob da ROM, tamanho e extensao.
#
# Cobre TODO o historico (git rev-list --objects --all), inclusive arquivos apagados depois.
# Em duvida, bloqueia: historico raso, refs ausentes ou lista vazia reprovam.
set -u

aqui=$(cd "$(dirname "$0")" && pwd)
cd "$aqui/.." || exit 1
# shellcheck source=../hooks/lib.sh
. "$aqui/../hooks/lib.sh"

# Com historico raso, o que ficou de fora nao e examinado. Reprova.
if [ "$(git rev-parse --is-shallow-repository)" != false ]; then
    recusar "anti-rom" "  o repositorio e raso (ou nao foi possivel saber). Use fetch-depth: 0 no checkout."
fi

lista=$(mktemp) || exit 1
trap 'rm -f "$lista"' EXIT
git rev-list --objects --all > "$lista" || recusar "anti-rom" "  git rev-list falhou"
[ -s "$lista" ] || recusar "anti-rom" "  nenhum objeto encontrado, o que nao deveria acontecer"

problemas=$(verificar_objetos < "$lista")
[ -z "$problemas" ] || recusar "anti-rom" "$problemas"

echo "anti-rom: sem suspeitos em $(grep -c . "$lista") objetos do historico."
exit 0
