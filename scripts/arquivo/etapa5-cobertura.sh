#!/bin/sh
# Etapa 5: desvios condicionais do original executados no corpus (cobertura/executados-68k.txt).
# alguns GB) e apaga tudo no fim. Depois rode de novo scripts/etapa5-assinar.sh: o arquivo gerado
# entra no manifesto.
#   sh scripts/etapa5-cobertura.sh 2>&1 | tee /tmp/etapa5-cobertura.txt
set -eu
repo=$(cd "$(dirname "$0")/.." && pwd -P)
cd "$repo"
ROM="$HOME/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen"
ext=${SONIC1_EXT_DIR:-$HOME/sonic1-ext}
bin=$ext/trabalho/oracle/alvo/release/oraculo
tmp=$ext/privado/trace-cobertura
rm -rf "$tmp"; mkdir -p "$tmp"
trap 'rm -rf "$tmp"' EXIT
rodar() { # nome quadros [entradas]
    nome=$1; q=$2; ent=${3:-}
    if [ -n "$ent" ]; then set -- --entradas "$ent"; else set --; fi
    "$bin" rodar --rom "$ROM" --quadros "$q" --saida "$tmp/$nome.a.s1t" "$@" | grep -E '^quadros='
    echo "caso $nome: $(du -m "$tmp/$nome.a.s1t" | cut -f1) MB de trace (apagado no fim)"
}
echo "##### 1. CASOS (trace completo temporario) #####"
rodar boot-demos 7200
rodar lz-agua 2000 corpus/lz-agua.s1in
rodar partida-ghz1 "$(wc -l < corpus/partida-ghz1.s1in)" corpus/partida-ghz1.s1in
rodar demos-quatro 11700
echo "##### 2. EXTRACAO #####"
cargo fidelidade cobertura extrair --traces "$tmp" --saida cobertura/executados-68k.txt
echo "desvios distintos: $(wc -l < cobertura/executados-68k.txt)"
echo "##### 3. RELATORIO #####"
acc=$ext/privado/etapa5
cat "$acc/nucleo-testes.txt" "$acc/nucleo-rotinas.txt" > "$acc/nucleo-todos.txt"
cargo fidelidade cobertura relatorio --nucleo "$acc/nucleo-todos.txt" --executados cobertura/executados-68k.txt \
    --executados "$acc/executados-rotinas.txt" --mapa cobertura/mapa-decisoes.txt --saida "$acc/cobertura.tsv" \
    | grep -v '^id	' | cut -c1-160
echo "##### FIM #####"
