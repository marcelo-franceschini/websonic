#!/bin/sh
# D1, teste da hipotese em muitos quadros do boot-demos (host, fora do Claude).
# Para cada quadro N compara a RAM do principal no quadro N com a do segundo oraculo no quadro
# N+1 e separa os bytes diferentes em tres grupos:
#   pilha   : variavel mais proxima v_objstate_end (a pilha fica entre v_systemstack_end,
#             FFFCC0, e v_systemstack, FFFE00, conforme refs/rotulos.txt);
#   tempo   : v_vblank_routine, v_jpadhold1, v_jpadpress1 (variaveis que o VBlank muda);
#   outros  : qualquer outra variavel. Estas sao impressas por nome e contagem.
# A hipotese preve "outros" vazio em todos os quadros. Um so quadro com "outros" a derruba.
# Imprime so contagens e nomes de variaveis (nunca valores).
# Uso: sh scripts/etapa2-d1c.sh ROM [QUADROS...]   (padrao: 100 a 7100 de 500 em 500)
set -eu
rom=${1:?uso: etapa2-d1c.sh ROM [QUADROS...]}
shift
[ $# -gt 0 ] || set -- 100 600 1100 1600 2100 2600 3100 3600 4100 4600 5100 5600 6100 6600 7100
repo=$(cd "$(dirname "$0")/.." && pwd -P)
ext=${SONIC1_EXT_DIR:-$HOME/sonic1-ext}
acc=$ext/privado/acessos
bin=$ext/trabalho/oracle/alvo/release/oraculo
o2=$repo/oracle2/target/release/websonic-oracle2
nucleo=$ext/trabalho/oracle2/gpgx/genesis_plus_gx_libretro.so
cd "$repo"
mkdir -p "$acc"
fora=0
for n in "$@"; do
    m=$((n + 1))
    "$bin" rodar --rom "$rom" --quadros $((n + 1)) --saida /dev/null --despejar-ram "$acc/d1c.p.ram" >/dev/null
    "$o2" rodar --nucleo "$nucleo" --rom "$rom" --quadros $((m + 1)) --saida "$acc/d1c.g.tsv" \
        --despejar "$m:$acc/d1c.g.ram" >/dev/null
    "$o2" diferencas --a "$acc/d1c.p.ram" --b "$acc/d1c.g.ram" --rotulos refs/rotulos.txt > "$acc/d1c.saida.txt"
    # Lê o total e a tabela "por variavel" (linhas "  nome: n").
    resumo=$(awk '
        /^ordem usada:/ { sub(/.*bytes diferentes: /, ""); total = $0 }
        /^por variavel/ { tabela = 1; next }
        tabela && /^  [^ ]+: [0-9]+$/ {
            nome = $1; sub(/:$/, "", nome); n = $2
            if (nome == "v_objstate_end") pilha += n
            else if (nome == "v_vblank_routine" || nome == "v_jpadhold1" || nome == "v_jpadpress1") tempo += n
            else outros = outros " " nome "=" n
        }
        END { printf "total=%d pilha=%d tempo=%d outros=%s", total, pilha, tempo, (outros == "" ? "nenhum" : outros) }
    ' "$acc/d1c.saida.txt")
    echo "quadro $n: $resumo"
    case $resumo in *"outros=nenhum") ;; *) fora=$((fora + 1)) ;; esac
done
echo "quadros com variavel fora de pilha e tempo: $fora de $#"
