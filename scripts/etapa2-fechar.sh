#!/bin/sh
# Etapa 2, fechamento, passo final (Marcelo, fora do Claude): refaz os traces do corpus com o
# oraculo novo, gera o manifesto (com os ocultos cifrados), assina e roda o comando de fidelidade.
# Imprime so nomes, hashes, contagens e vereditos.
# Uso: sh scripts/etapa2-fechar.sh "$HOME/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen"
set -eu
rom=${1:?uso: etapa2-fechar.sh ROM}
repo=$(cd "$(dirname "$0")/.." && pwd -P)
ext=${SONIC1_EXT_DIR:-$HOME/sonic1-ext}
traces=$ext/privado/traces
bin=$ext/trabalho/oracle/alvo/release/oraculo
cd "$repo"
[ -x "$bin" ] || { echo "Erro: rode antes scripts/etapa2-final.sh." >&2; exit 1; }

echo "=== CORPUS-TRACE (oraculo novo) ==="
for caso in lz-agua boot-demos; do
    cargo run -q -p xtask -- corpus-trace --rom "$rom" --oraculo "$bin" \
        --rotulos refs/rotulos.txt --saida "$traces" --caso "$caso"
done | tee "$traces/resumo5.tsv"
grep -hv '^determinismo' "$traces/resumo5.tsv" > "$traces/hashes5.tsv"

echo "=== AUDITORIA DOS OCULTOS (decifra em /dev/shm; so zona: passou|falhou) ==="
if [ -d "$ext/ocultos" ]; then
    ROM="$rom" ORACULO="$bin" cargo run -q -p xtask -- ocultos auditar --pasta "$ext/ocultos" \
        --identidade "$HOME/.config/websonic/ocultos.key" --sut "$repo/scripts/sut-reproduz.sh"
    ocultos="--ocultos $ext/ocultos"
else
    echo "(sem pasta de ocultos)"
    ocultos=
fi

echo "=== MANIFESTO ==="
# shellcheck disable=SC2086
cargo run -q -p xtask -- manifesto gerar --traces "$traces/hashes5.tsv" $ocultos
ssh-keygen -Y sign -f "$HOME/.ssh/websonic_assinatura" -n websonic-manifesto refs/manifesto.txt
cargo run -q -p xtask -- manifesto verificar --pular-testes

echo "=== FIDELIDADE (comando unico) ==="
cargo run -q --release -p xtask -- fidelidade --rom "$rom" --oraculo "$bin" --saida "$traces"
echo "=== FIM ==="
