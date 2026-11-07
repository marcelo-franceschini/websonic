#!/bin/sh
# Etapa 2, gravador de partidas (2.2) e testes ocultos (2.8). Roda no host, fora do Claude.
# Abre o jgenesis com janela, grava os botoes por quadro, PROVA que o oraculo sem janela
# reproduz a partida (a RAM do ultimo quadro tem de ser identica) e, se a ZONA nao for "prova",
# cifra o caso. O texto claro fica so em /dev/shm e e apagado no fim.
#
# O script abre o jogo direto, com regiao Japao, NTSC e controle 1 de 3 botoes forcados so
# para esta execucao (o arquivo de configuracao do jgenesis nao muda).
# Teclas: setas, A = botao A, S = botao B, D = botao C, Enter = Start.
# NAO use: F1 e F2 (reset), F5 e F6 (estado salvo), Tab (avanco rapido), ` (voltar), P e N
# (pausa e passo). Eles quebram a reproducao. A partida comeca no ligar do console.
# Para terminar, feche a janela pelo X, jogando antes uns 30 a 60 segundos.
#
# Uso: sh scripts/etapa2-gravar.sh ROM ZONA CASO
#   ZONA: prova | corpus | ghz | mz | syz | lz | slz | sbz | especial | final | telas
#   ZONA corpus: a partida vira um caso PUBLICO do corpus (corpus/CASO.s1in e o .meta, sem cifrar).
#     Jogue de 1 a 2 minutos da Green Hill. Autor Marcelo, licenca MIT. Use CASO=partida-ghz1.
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

echo "Abrindo o jogo. Jogue e feche a janela pelo X no fim."
WEBSONIC_GRAVAR="$tmp/g" "$gui" -f "$rom" \
    --config-override genesis.forced_region=Japan \
    --config-override genesis.forced_timing_mode=Ntsc \
    --config-override input.genesis.p1_type=ThreeButton \
    --config-override input.genesis.p2_type=None 2>&1 | grep -E "WEBSONIC|forced_region|forced_timing|p1_type|Cartridge region|rror" || true
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
if [ "$zona" = corpus ]; then
    # Caso publico: so botoes por quadro (nenhum dado da ROM), com o .meta calculado.
    case $caso in *[!A-Za-z0-9_-]*|"") echo "Erro: CASO invalido." >&2; exit 1 ;; esac
    [ ! -e "corpus/$caso.s1in" ] || { echo "Erro: corpus/$caso.s1in ja existe." >&2; exit 1; }
    cp "$tmp/g.s1in" "corpus/$caso.s1in"
    cargo xtask corpus-meta --entradas "corpus/$caso.s1in" \
        --fonte "partida gravada pelo Marcelo (scripts/etapa2-gravar.sh)" --licenca MIT --autor Marcelo
    echo "corpus: corpus/$caso.s1in ($n quadros)"
    exit 0
fi
cargo fidelidade ocultos cifrar --publica refs/ocultos.pub --entrada "$tmp/g.s1in" \
    --pasta "$ext/ocultos" --zona "$zona" --caso "$caso"
echo "cifrado: zona=$zona"
