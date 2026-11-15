# Recompilação antecipada

`sonic1-recomp` é uma ferramenta inicial para descobrir código 68000 alcançável e
emitir Rust antes da execução. A interface recebe uma região explícita da ROM;
somente caminhos alcançáveis desde a entrada são traduzidos. A ordem é determinística
e instruções, alvos ou sobreposições não reconhecidos produzem erro.

```sh
cargo run --locked -p sonic1-recomp -- ROM INICIO_HEX FIM_EXCLUSIVO_HEX SAIDA.rs
```

O executável valida o hash da ROM REV01. O arquivo de saída deve ser novo. A ROM,
arquivos de dados e traces não são copiados para o repositório. O Rust gerado usa
`sonic1_recomp::runtime`, e sua função `passo` despacha pelo endereço já traduzido,
sem buscar nem decodificar instruções da ROM durante a execução.

Por enquanto, o subconjunto suporta somente MOVEQ, NOP, RTS, BRA e Bcc. A estrutura
de estado cobre D0 a D7, PC e CCR. RTS apenas sinaliza o retorno ao chamador; ainda
não há pilha do 68000, memória do console, chamadas, interrupções ou temporização.
O código gerado ainda não está integrado ao jogo nem cobre a lógica de uma fase.

Os testes incluem compilação e execução de Rust gerado a partir de um programa
sintético, caminhos de desvio, extensão de sinal e flags. A geração também foi
exercitada localmente com a REV01 no rótulo `Edge_TopBottom`, em `0x91c2`.
Esse caso contém duas instruções e não constitui prova de fidelidade do jogo.
