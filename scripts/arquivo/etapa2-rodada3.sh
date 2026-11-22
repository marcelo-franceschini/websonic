#!/bin/sh
# Etapa 2, rodada 3 (Marcelo, fora do Claude). Roda em qualquer shell: sh scripts/etapa2-rodada3.sh
# Passo 1: D1 com a regiao ntsc-j. Passo 2: prova do gravador. Passo 3: um caso oculto por zona.
# Passo 4: manifesto, assinatura e fidelidade. Para na primeira prova que falhar.
set -eu
repo=$(cd "$(dirname "$0")/.." && pwd -P)
cd "$repo"
ROM="$HOME/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen"

echo "##### 1. D1 (regiao ntsc-j) #####"
(cd oracle2 && cargo build --release -q)
sh scripts/etapa2-oraculo2.sh "$ROM" 2>&1 | tee /tmp/etapa2-oraculo2.txt
sh scripts/etapa2-d1.sh "$ROM" 53 120 600 2>&1 | tee /tmp/etapa2-d1b.txt || true

echo "##### 2. PROVA DO GRAVADOR: jogue uns 20 s e feche a janela pelo X #####"
sh scripts/etapa2-gravar.sh "$ROM" prova teste 2>&1 | tee /tmp/etapa2-prova.txt
grep -q 'gravacao: REPRODUZ' /tmp/etapa2-prova.txt || { echo "PAROU: a prova nao reproduziu."; exit 1; }

echo "##### 3. UM CASO POR ZONA: 30 a 60 s cada, feche pelo X #####"
echo "Setas, A, S, D, Enter. NAO use F1 F2 F5 F6 Tab \` P N."
for z in ghz mz syz lz slz sbz; do
    echo "--- zona $z: a janela abre em seguida ---"
    sh scripts/etapa2-gravar.sh "$ROM" "$z" caso1 2>&1 | grep -E "quadros gravados|gravacao|cifrado|Erro"
done

echo "##### 4. MANIFESTO, ASSINATURA E FIDELIDADE (responda y ao sobrescrever) #####"
sh scripts/etapa2-fechar.sh "$ROM" 2>&1 | tee /tmp/etapa2-fechar.txt
echo "##### FIM #####"
