# Cobertura (Etapa 5)

Metadado da cobertura de decisões. Só nomes, endereços e contagens. Nenhum dado da ROM.

- `mapa-decisoes.txt`: decisão do núcleo para desvio do original. Uma linha por decisão: `id<TAB>endereco_hex<TAB>direto|inverso`. O `diferencial rotina` propõe o mapa (`--proposta-de-mapa`) e verifica o mapa declarado (`--mapa`) em todos os casos. O Marcelo congela o arquivo.
- `executados-68k.txt`: desvios condicionais do 68000 executados pelo oráculo no corpus: `endereco_hex<TAB>tomado<TAB>nao_tomado`. Gerado por `cargo fidelidade cobertura extrair`.
- `equivalentes.txt`: mutantes sobreviventes que não mudam o comportamento do jogo, com a justificativa. Só o Marcelo edita: `id<TAB>justificativa`.
- `exclusoes-propostas.txt`: exclusões que a IA propõe e o Marcelo aprova (ou não). A IA nunca as aplica sozinha.

Universo: o requisito mede os desvios executados pelo oráculo. Um desvio que nenhum caso executou não aparece aqui; achá-lo pede decodificar o código sem executar (backlog).
