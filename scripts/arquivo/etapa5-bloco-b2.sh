#!/bin/sh
# Etapa 5, bloco B2 (Marcelo, fora do Claude): repete so o que o bloco B reprovou. Aplica o segundo
# patch do oraculo (eventos de rotinas que atravessam quadros), roda o teste por rotina com a ROM,
# gera o mapa de decisoes, a cobertura e os bugs conhecidos. Imprime so nomes, numeros e vereditos.
#   sh scripts/etapa5-bloco-b2.sh 2>&1 | tee /tmp/etapa5-bloco-b2.txt
set -u
repo=$(cd "$(dirname "$0")/.." && pwd -P)
cd "$repo"
ROM="$HOME/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen"
ext=${SONIC1_EXT_DIR:-$HOME/sonic1-ext}
traces=$ext/privado/traces
acc=$ext/privado/etapa5
mkdir -p "$acc"
dif_alvo=$ext/trabalho/oracle/alvo-dif

echo "##### 1. PATCH DO ORACULO (eventos de rotinas longas) #####"
if git apply --check docs/pedidos/etapa5-oraculo-2.patch 2>/dev/null; then
    git apply docs/pedidos/etapa5-oraculo-2.patch && echo "etapa5-oraculo-2: aplicado"
else
    echo "etapa5-oraculo-2: ja aplicado (ou nao aplica; veja git status)"
fi
echo "--- teste do patch (rotina que atravessa quadros) ---"
CARGO_TARGET_DIR=$dif_alvo cargo test --release --manifest-path diferencial/Cargo.toml --test quadros 2>&1 | grep -E "^test |test result|^error"

echo "##### 2. TESTE POR ROTINA COM A ROM (KosDec, EniDec, NemDecToRAM) #####"
export CARGO_TARGET_DIR=$dif_alvo
cargo build --release -q --manifest-path diferencial/Cargo.toml
unset CARGO_TARGET_DIR
dif=$dif_alvo/release/diferencial
: > "$acc/mapa-proposto.txt"
: > "$acc/executados-rotinas.txt"
: > "$acc/nucleo-rotinas.txt"
for r in KosDec EniDec NemDecToRAM; do
    "$dif" rotina "$r" --rom "$ROM" --rotulos refs/rotulos.txt --manifesto descompressao/manifesto.txt \
        --metadados crates/teste-nucleo/rotinas --proposta-de-mapa "$acc/$r.mapa" \
        --executados-saida "$acc/$r.exec" --cobertura-nucleo "$acc/$r.nuc" > "$acc/$r.txt"
    grep -E "^$r: |^decisao |^a1 do nucleo" "$acc/$r.txt"
    grep -E "DIFERE|ERRO|AUSENTE" "$acc/$r.txt" | head -12
    echo "registradores ignorados que o original mexeu (conjunto, quantos casos):"
    grep -o 'ignorados_mexidos=\[[^]]*\]' "$acc/$r.txt" | sort | uniq -c
    cat "$acc/$r.mapa" >> "$acc/mapa-proposto.txt" 2>/dev/null || true
    cat "$acc/$r.exec" >> "$acc/executados-rotinas.txt" 2>/dev/null || true
    cat "$acc/$r.nuc" >> "$acc/nucleo-rotinas.txt" 2>/dev/null || true
done
echo "--- mapa de decisoes proposto (nome, endereco, polaridade) ---"
cat "$acc/mapa-proposto.txt"
# O mapa proposto vira o mapa declarado so se cada decisao tem UM candidato. Depois confere de novo.
{ echo "# Decisao do nucleo para desvio do original. id<TAB>endereco_hex<TAB>direto|inverso"
  echo "# Gerado pelo diferencial rotina (--proposta-de-mapa) e conferido de novo com --mapa."
  sort -u "$acc/mapa-proposto.txt"; } > cobertura/mapa-decisoes.txt
echo "--- conferencia com o mapa declarado ---"
for r in KosDec EniDec NemDecToRAM; do
    "$dif" rotina "$r" --rom "$ROM" --rotulos refs/rotulos.txt --manifesto descompressao/manifesto.txt \
        --metadados crates/teste-nucleo/rotinas --mapa cobertura/mapa-decisoes.txt | grep -E "^$r: |fora do mapa"
done

echo "##### 3. COBERTURA #####"
WEBSONIC_COBERTURA_ARQ="$acc/nucleo-testes.txt" cargo test -q -p sonic1-teste-descompressao --test cobertura 2>&1 | grep -E "test result|FAILED"
cargo fidelidade cobertura extrair --traces "$traces" --saida cobertura/executados-68k.txt
cat "$acc/nucleo-testes.txt" "$acc/nucleo-rotinas.txt" > "$acc/nucleo-todos.txt"
cargo fidelidade cobertura relatorio --nucleo "$acc/nucleo-todos.txt" --executados cobertura/executados-68k.txt \
    --executados "$acc/executados-rotinas.txt" --mapa cobertura/mapa-decisoes.txt --saida "$acc/cobertura.tsv" \
    | grep -v '^id	' | cut -c1-160

echo "##### 4. BUGS CONHECIDOS #####"
cargo fidelidade bugs-conhecidos --saida "$acc/bugs.txt"
echo "##### FIM #####"
