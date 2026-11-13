# Registro da revisão das etapas 0 a 8

Plano: `docs/plano-revisao-etapas-0-8.md`. Execução de 12/11/2026, no host (fora do contêiner, com a L1 inativa), só o bloco P (R0.01 a R0.03), autorizado pelo Marcelo na aprovação do plano. Nenhuma leitura da ROM.

Status possíveis: CONCLUÍDO, CONCLUÍDO COM VERIFICAÇÃO PENDENTE, NÃO CONCLUÍDO, BLOQUEADO.

## Base

Verificação de base no host, em 12/11/2026, antes do primeiro item (HEAD `8850cf1`). A saída completa fica em `scratchpad/base/*.log`. O host não tem `rustup` nem `wasmtime`, então os comandos de wasm não rodam aqui.

| Comando | Código | Resumo |
| --- | --- | --- |
| `git status --porcelain` vazio | 0 | árvore limpa |
| `cargo fmt --all --check` | 0 | |
| `sh .devcontainer/testes/hooks.sh` | 0 | 14 testes, 0 falhas |
| `sh .devcontainer/testes/anti-rom.sh` | 0 | 8 testes, 0 falhas |
| `sh .github/checa-core.sh` | 0 | |
| `cargo xtask anti-rom` | 0 | |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | |
| `cargo test --workspace` | 0 | 295 testes passaram, 0 falhas |
| relatório: `cargo fmt --check` | 0 | |
| relatório: `cargo clippy` com `--locked` e `--target-dir target/relatorio` | 0 | |
| relatório: `cargo test` com `--locked` e `--target-dir target/relatorio` | 0 | 45 passaram |
| `sh .github/lints-core.sh` | 1 | falha da base: alvo `wasm32-unknown-unknown` ausente no host |
| `cargo xtask site` | 1 | falha da base: alvo `wasm32-unknown-unknown` ausente no host |

Não rodados no host: `cargo test --target wasm32-wasip1 ...` (sem `wasmtime`). As duas falhas já existiam na base e não vêm de nenhum item. No contêiner elas devem passar, como no CI.

## Itens

| ID | Status | Arquivos | Commit | Veredito | Verificação |
| --- | --- | --- | --- | --- | --- |
| R0.01 | CONCLUÍDO | docs/pedidos/rev-R0.01.patch | `ef0e471` | APROVADO pelo Sonnet, sem ajustes. Conferido por mim. | Worktree limpo com os patches L1 e o rev: `cargo test` 0 (18 passaram), `cargo clippy ... -D warnings` 0, `cargo fmt --check` 0, `cargo tree` só `detector-rom`. O revisor comparou o SHA-1 com `sha1sum` em 731 arquivos e 11 tamanhos, pegou 17 defeitos plantados e achou o detector L1 e o novo iguais em 423 casos sintéticos. |
| R0.02 | CONCLUÍDO | docs/pedidos/rev-R0.02.patch | `5c61aaf` | APROVADO pelo Sonnet, sem ajustes. Conferido por mim. | Worktree limpo com os 4 patches em ordem: T1 (`hooks/` somente leitura, `cargo build --locked --target-dir`) 0 e binário criado; T3 `sh .devcontainer/testes/hooks.sh` 14 testes, 0 falhas; T4 18 testes do detector; `sh -n hooks/pre-commit` ok. Revisor: defeito reproduzido com o hook antigo e `hooks/` somente leitura, hook novo passa; nome sintético com espaços e colchetes ok; prioridade `/mnt/rom` testada em cópia do hook; shell ok em sh, bash e busybox. |
| R0.03a | CONCLUÍDO | docs/pedidos/rev-R0.03a.patch | `02cefbc` | APROVADO COM AJUSTES pelo Sonnet. Conferido por mim. | Worktree limpo com os 5 patches em ordem: `cargo test --locked` 0 (20 passaram), `cargo clippy ... -D warnings` 0, `cargo fmt --check` 0, `cargo tree` só `detector-rom`. O revisor achou 2 regressões reais no patch original (cópia escrita dentro de comentário e valor fora da faixa derrubando o fluxo da lista) e as corrigiu. Também testou 60 entradas hostis sem pânico, comparou o detector antigo e o novo em 11 mil arquivos sintéticos (nada deixou de ser achado, salvo o que a nova semântica de `-` e `$` pede), fez mutação (65 de 68 pegas, 3 equivalentes) e varreu 375 arquivos rastreados com zero achados. |
| R0.03b | CONCLUÍDO | docs/pedidos/rev-R0.03b.patch | `a8d8a0f` | APROVADO COM AJUSTES pelo Sonnet. Conferido por mim. | Worktree limpo com os 6 patches em ordem: `cargo test --locked` 0 (20 passaram), `cargo clippy ... -D warnings` 0, `cargo fmt --check` 0, `cargo tree` só `detector-rom`, build com `hooks/` somente leitura 0, `sh .devcontainer/testes/hooks.sh` 14 testes, 0 falhas, `sh -n hooks/pre-commit` ok. O revisor regerou o patch sobre o `a` novo (o `index` e os hunks do original estavam errados) e reforçou as tabelas de teste: 60 mutantes, 49 mortos, 8 equivalentes, 3 que só alargam o formato. Fuzz de 200 mil casos sem pânico, 15 mil casos de regressão (0 violações) e 376 arquivos rastreados varridos com 8 ROMs sintéticas (0 achados). |

## Observações da execução

- O R0.03 foi dividido em R0.03a (números e listas) e R0.03b (leitura hexadecimal em texto) por tamanho. A implementação inteira do Haiku deu 442 linhas, acima do limite do plano (cerca de 300 por item). É a mesma mudança do item aprovado, em duas partes.
- O `rev-R0.03a.patch` tem 737 linhas (580 adicionadas, 45 removidas), acima do limite, porque o revisor corrigiu duas regressões reais e reforçou os testes. Decisão do coordenador: aceitar, com a evidência do revisor e a leitura e a verificação do coordenador.
- Falha técnica: na primeira tentativa o Haiku do R0.02 parou por limite de uso da API. Foi retomado e concluiu (segunda tentativa).
- Desvio menor do Haiku do R0.02: consultou se existia `.gen` em `/mnt/rom` (a pasta não existe no host). Não leu conteúdo nenhum.
- O único teste existente que muda em todo o bloco P é `numero_colado_a_letra_nao_conta` (R0.03a). Ele fixava a lacuna (`12u8` recusado). A troca estava prevista no item.
- Nenhuma ROM e nenhum binário do disassembly foram lidos.

## Pendências levantadas pelos revisores (fora dos itens)

1. R0.02: o binário do detector fica em `target/detector`, que a IA escreve. Um binário falso no lugar seria aceito pelo hook. Avaliar junto com o R0.04 (pre-push no host do Marcelo).
2. R0.02: `recusar` (hooks/lib.sh, congelado) imprime "Nada com conteudo da ROM" também no erro de compilação. Só estético.
3. R0.02: se um `cargo run` já criou `hooks/detector/Cargo.lock` antes, o `git apply` do rev-R0.02 falha ("já existe"). Aplicar todos os patches antes de rodar o detector, ou apagar o lock.
4. R0.03a: o detector informa a linha de forma aproximada nos fluxos u16be e u32be (início da lista mais o índice no fluxo). Corrigir exige guardar posições.
5. R0.03a: lista de 8 a 15 longs (32 a 60 bytes) fica fora da busca (`MIN_VALORES` 16). Baixar o limite deu falso positivo com uma ROM adversária. Decidir com a ROM real depois da L1.
6. O falso positivo do detector com a ROM real só se mede depois da L1.
7. O hook compila em debug: cerca de 1,5 s por MiB de números.
8. `confirmar_rev01` segue sem teste unitário próprio.
9. `docs/estado.md:11` cita "11 testes unitários do detector". Com o bloco P são 20 (ou mais).
10. R0.03b, importante: formatos de hexadecimal que o detector ainda NÃO vê (medidos pelo revisor): `xxd -p` com linhas de 60 dígitos, `hexdump -C`, grupos de 4 ou 8 dígitos, `0x4E 0x71` só com espaço, escapes `\x4e`, `%4e`, separadores `-` e `;`, linhas com `//` ou `;` na frente e inteiro hexadecimal com número ímpar de dígitos. O `xxd -p` é o mais provável de acontecer. Sugestão: um item novo, R0.03c. Ele não está no plano e precisa da aprovação do Marcelo.
11. R0.03b: a linha informada num achado `hex` é a do início da corrida, não a do byte exato.

## Resultado do bloco P

- Quatro commits, um por patch: `ef0e471` (R0.01), `5c61aaf` (R0.02), `02cefbc` (R0.03a), `a8d8a0f` (R0.03b). O plano está no commit `8850cf1`.
- Os três itens do plano (R0.01, R0.02 e R0.03) estão CONCLUÍDOS. O R0.03 virou dois patches.
- O defeito crítico foi corrigido: com os patches, o hook do commit roda o detector no contêiner, com `hooks/` somente leitura.
- Nada com a ROM foi feito. A verificação com a ROM real (controles do passo 2 da L1 e falso positivo) continua sendo do Marcelo.

## Instruções ao Marcelo para aplicar o bloco P (host, na raiz do repositório, árvore limpa)

1. Aplique os seis patches, um por vez e nesta ordem. O laço para no primeiro que falhar:

```
for p in l1-hook-detector l1-hook-pre-commit rev-R0.01 rev-R0.02 rev-R0.03a rev-R0.03b; do
    git apply --check docs/pedidos/$p.patch && git apply docs/pedidos/$p.patch || { echo "falhou em $p"; break; }
done
```

2. Confira: `sh .devcontainer/testes/hooks.sh` (esperado: 14 testes, 0 falhas) e `cargo test --locked --manifest-path hooks/detector/Cargo.toml --target-dir target/detector` (esperado: 20 passaram).
3. Aplique TODOS os patches antes de rodar o detector com `cargo run`. O `cargo run` cria `hooks/detector/Cargo.lock`, e o `rev-R0.02.patch` falha com "já existe". Se isso acontecer, apague `hooks/detector/Cargo.lock` e aplique de novo.
4. Siga os passos 2 a 4 de "Ainda não vale" em `docs/estado.md` (controles com a ROM real, `l1-permissoes-rom.patch`, reconstruir a imagem, levar a decisão ao chat de planejamento). Nos controles pode usar `--locked --target-dir target/detector` no `cargo run`, para não gravar em `hooks/`.
5. O commit que traz o detector já roda o hook novo. Ele procura a ROM em `/mnt/rom` e depois em `$SONIC1_EXT_DIR/rom` (padrão `~/sonic1-ext/rom`). Sem ROM, o commit é bloqueado com "ROM nao encontrada".
6. Depois disso a fase 2 pode seguir no contêiner, a partir do R0.04, com todos os itens do plano.

## Itens que ficam para a sessão no contêiner

R0.04 em diante (ver o plano). Os itens do MARCELO nunca vão para subagente.
