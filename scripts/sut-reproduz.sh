#!/bin/sh
# Sistema sob teste minimo da auditoria dos ocultos na Etapa 2: o oraculo sem janela reproduz o
# caso ate o fim sem erro. Recebe o caminho do .s1in. Variaveis: ROM e ORACULO.
# (A partir da Etapa 5 o SUT passa a ser o nucleo comparado com o oraculo.)
set -eu
n=$(wc -l < "$1")
"$ORACULO" rodar --rom "$ROM" --quadros "$n" --saida /dev/null --entradas "$1" >/dev/null
