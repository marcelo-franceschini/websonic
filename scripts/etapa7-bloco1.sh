#!/bin/sh
# Etapa 7, bloco 1 (Marcelo, fora do Claude): perfil do trecho do boot ate a primeira demo e
# marcas de rotulo do oraculo (quadro e mclk da primeira chegada do PC). Imprime so nomes e numeros.
# Uso: sh scripts/etapa7-bloco1.sh "$HOME/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen"
set -u
rom=${1:?uso: etapa7-bloco1.sh ROM}
repo=$(cd "$(dirname "$0")/.." && pwd -P)
ext=${SONIC1_EXT_DIR:-$HOME/sonic1-ext}
cd "$repo"

esperado=$(sed -n 's/^sha1: *//p' refs/rom-hashes.txt)
obtido=$(sha1sum "$rom" | cut -d' ' -f1)
[ "$esperado" = "$obtido" ] || { echo "Erro: a ROM nao e a REV01." >&2; exit 1; }

echo "=== AMBIENTE ==="
echo "git: $(git rev-parse --short HEAD) sujo=$(git status --short | wc -l)"
echo "alvo_wasm_no_host: $(rustup target list --installed 2>/dev/null | grep -c wasm32-wasip1)"
if docker image inspect websonic-dev:ci >/dev/null 2>&1; then echo "imagem_websonic-dev:ci: ok"; else echo "imagem_websonic-dev:ci: ausente"; fi

echo "=== OFICINA DO ORACULO ==="
sh oracle/scripts/prepara.sh | tail -1
export CARGO_TARGET_DIR="$ext/trabalho/oracle/alvo"
(cd oracle/runner && cargo build --release -q) || { echo "falha ao construir o oraculo"; exit 1; }
bin=$CARGO_TARGET_DIR/release/oraculo
unset CARGO_TARGET_DIR

echo "=== PERFIL (so as linhas em que modo, zona, ato ou demo mudam) ==="
"$bin" rodar --rom "$rom" --quadros 2600 --saida /dev/null --perfil-a-cada 1 \
    --rotulos refs/rotulos.txt | grep '^perfil ' \
    | awk '{k=$3" "$4" "$5" "$6; if (k!=u) print; u=k}'

echo "=== MARCAS (primeira chegada do PC; formato: marca NOME quadros=N mclk=M parou=ENDERECO) ==="
for r in EntryPoint SkipSetup GameProgram CheckSumOk GameInit GM_Sega PlaySegaSound \
         Sega_WaitPal Sega_WaitEnd Sega_GotoTitle GM_Title Tit_MainLoop GotoDemo \
         GM_Level Level_TtlCardLoop Level_LoadObj Level_Delay Level_StartGame Level_MainLoop; do
    saida=$("$bin" rodar --rom "$rom" --quadros 2600 --saida /dev/null \
        --rotulos refs/rotulos.txt --parar-em "$r" 2>&1) || { echo "marca $r FALHOU"; continue; }
    q=$(echo "$saida" | sed -n 's/^quadros=//p')
    m=$(echo "$saida" | sed -n 's/^mclk_final=//p')
    p=$(echo "$saida" | sed -n 's/^parou_no_endereco=//p')
    echo "marca $r quadros=$q mclk=$m parou=$p"
done
echo "=== FIM ==="
