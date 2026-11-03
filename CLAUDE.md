# Regras do Projeto: Reconstrução do Sonic 1

Este arquivo orienta a atuação da IA no repositório.

## Fonte da verdade

- O arquivo `docs/requisitos.md` é a fonte única de verdade do projeto.
- Leia `docs/requisitos.md` inteiro antes de qualquer tarefa.
- Se houver qualquer conflito entre instruções e esse documento, pare imediatamente e avise o Marcelo.
- Consulte `docs/estado.md` para saber a etapa e sub-etapa atuais.
- Consulte o índice em `docs/conhecimento/README.md` e leia os tópicos pertinentes à sessão.

## Estilo e comunicação

- Escreva sempre em português.
- Use frases curtas e objetivas.
- Nunca use travessões em nenhum arquivo. Use vírgulas, pontos ou parênteses.
- Rust deve ser legível e idiomático. Comente escolhas de implementação não óbvias.
- O código JavaScript e HTML do site deve ser mínimo e comentado.

## Integridade da ROM e do disassembly

- Nunca leia a ROM diretamente.
- Nunca leia arquivos binários do disassembly.
- A única exceção são os testes de isolamento da sub-etapa 0.2. Nesses testes, desvie a saída para `/dev/null` e avalie apenas o código de retorno.
- Nada com conteúdo da ROM pode entrar no git. Imagens, sons, código extraído, dados binários ou traces não entram.
- Apenas metadados podem entrar no git: hashes, nomes, endereços e listas.

## Git e fluxo de trabalho

- Faça apenas commits locais.
- Nunca faça push. O push é exclusivo do Marcelo.
- Nunca use a opção `--no-verify`.
- Nunca desative nem ignore hooks do git.
- Siga rigorosamente o ciclo por sub-etapa: detalhar o plano, implementar, realizar commit local e parar.
- Apresente o plano completo da etapa antes de iniciar suas sub-etapas e aguarde aprovação.

## Regras de código Rust

- O crate `core` proíbe o uso de aritmética de ponto flutuante (`clippy::float_arithmetic`).
- O crate `core` exige tratamento explícito de estouro de inteiros (`clippy::arithmetic_side_effects`).
- O crate `core` proíbe coleções com ordem não determinística, como `HashMap` e `HashSet`.
- O crate `core` proíbe casts que possam causar truncamento silencioso.
- O jogo não emula nem interpreta instruções do 68000 ou Z80. O núcleo espelha memória e rotinas com lógica nativa em Rust.
- Regra de ouro: teste que falha se corrige no jogo, nunca no teste.
