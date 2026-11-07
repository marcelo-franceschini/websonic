#!/bin/sh
# Etapa 2, gravador de partidas (2.2) e testes ocultos (2.8). Roda no host, fora do Claude.
# Abre o jgenesis com janela, grava os botoes por quadro, PROVA que o oraculo sem janela
# reproduz a partida (a RAM do ultimo quadro tem de ser identica) e, se a ZONA nao for "prova",
# cifra o caso. O texto claro fica so em /dev/shm e e apagado no fim.
#
# ANTES, nas configuracoes do jgenesis (menu): regiao forcada Japao, NTSC, controle 1 de 3
# botoes, controle 2 nenhum. Sem isso a prova acusa DIFERE.
# A partida comeca no ligar do console (nao use savestate nem reset). Feche a janela no fim.
#
# Uso: sh scripts/etapa2-gravar.sh ROM ZONA CASO
#   ZONA: prova | ghz | mz | syz | lz | slz | sbz | especial | final | telas
set -eu
rom=${1:?uso: etapa2-gravar.sh ROM ZONA CASO}
zona=${2:?falta ZONA}
caso=${3:?falta CASO}
repo=$(cd "$(dirname "$0")/.." && pwd -P)
ext=${SONIC1_EXT_DIR:-$HOME/sonic1-ext}
gui=$ext/trabalho/oracle/alvo-gui/release/jgenesis
bin=$ext/trabalho/oracle/alvo/release/oraculo
[ -x "$gui" ] && [ -x "$bin" ] || { echo "Erro: rode antes scripts/etapa2-final.sh." >&2; exit 1; }
cd "$repo"

tmp=$(mktemp -d /dev/shm/websonic-gravar.XXXXXX)
trap 'rm -rf "$tmp"' EXIT INT TERM

echo "Abrindo o jgenesis. Abra a ROM pelo menu, jogue, e feche a janela no fim."
WEBSONIC_GRAVAR="$tmp/g" "$gui" || true
[ -s "$tmp/g.s1in" ] || { echo "Erro: nada foi gravado." >&2; exit 1; }
n=$(wc -l < "$tmp/g.s1in")
echo "quadros gravados: $n"

"$bin" rodar --rom "$rom" --quadros "$n" --saida /dev/null --entradas "$tmp/g.s1in" \
    --despejar-ram "$tmp/r.ram" >/dev/null
if cmp -s "$tmp/g.ram" "$tmp/r.ram"; then
    echo "gravacao: REPRODUZ (RAM do ultimo quadro identica)"
else
    echo "gravacao: DIFERE (a janela e o oraculo nao concordam; nada foi cifrado)"
    exit 1
fi

if [ "$zona" = prova ]; then
    echo "zona prova: nada cifrado."
    exit 0
fi
cargo run -q -p xtask -- ocultos cifrar --publica refs/ocultos.pub --entrada "$tmp/g.s1in" \
    --pasta "$ext/ocultos" --zona "$zona" --caso "$caso"
echo "cifrado: zona=$zona"
