#!/bin/sh
# Etapa 2, rodada 4 (Marcelo, fora do Claude): aplica o patch que congela os botoes no quadro
# do gravador, reconstroi oraculo e gravador, regrava a zona mz e refaz manifesto e fidelidade.
# Roda em qualquer shell: sh scripts/etapa2-rodada4.sh
set -eu
repo=$(cd "$(dirname "$0")/.." && pwd -P)
cd "$repo"
ext=${SONIC1_EXT_DIR:-$HOME/sonic1-ext}
ROM="$HOME/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen"

echo "##### 1. PATCH E TESTES #####"
patch -p1 < docs/pedidos/etapa2-oraculo-2.patch
sh oracle/scripts/testa.sh 2>&1 | grep -E "test result|Testes do oraculo|error|FAILED"

echo "##### 2. BUILD DO ORACULO E DO GRAVADOR #####"
sh oracle/scripts/prepara.sh | tail -1
CARGO_TARGET_DIR="$ext/trabalho/oracle/alvo" cargo build --release -q --manifest-path oracle/runner/Cargo.toml
CARGO_TARGET_DIR="$ext/trabalho/oracle/alvo-gui" cargo build --release -q \
    --manifest-path "$repo/target/oracle-fonte/Cargo.toml" -p jgenesis-gui
echo "construido"

echo "##### 3. REGRAVAR A ZONA mz (30 a 60 s, feche pelo X) #####"
echo "Setas, A, S, D, Enter. NAO use F1 F2 F5 F6 Tab \` P N."
sh scripts/etapa2-gravar.sh "$ROM" mz caso1 2>&1 | grep -E "quadros gravados|gravacao|cifrado|Erro"

echo "##### 4. MANIFESTO, ASSINATURA E FIDELIDADE (responda y ao sobrescrever) #####"
sh scripts/etapa2-fechar.sh "$ROM" 2>&1 | tee /tmp/etapa2-fechar2.txt
echo "##### FIM #####"
