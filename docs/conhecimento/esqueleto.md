# Esqueleto Rust e site

Sub-etapa 0.4. Tudo roda na imagem de trabalho (`websonic-dev`), com o Rust 1.99.1 fixado em `rust-toolchain.toml`.

## Estrutura

| Pasta | Pacote | Função |
| --- | --- | --- |
| `crates/core` | `sonic1-core` | Núcleo fiel. Hoje só `aritmetica`, com estouro explícito. |
| `crates/rom` | `sonic1-rom` | SHA-1 em Rust puro e veredito da REV01. Só compara identificadores. |
| `crates/web` | `sonic1-web` | Camada fina `wasm-bindgen`. Expõe `identificar(bytes)`. |
| `xtask` | `xtask` | `cargo xtask site` e `cargo xtask anti-rom`. |
| `site` | | `index.html` e `app.js`, mínimos e comentados. |

O pacote do núcleo não se chama `core`, porque colide com a biblioteca padrão do Rust. A pasta continua `crates/core`.

## Lints do núcleo

Em `crates/core/Cargo.toml` e `crates/core/clippy.toml`: `float_arithmetic`, `arithmetic_side_effects`, `cast_possible_truncation`, `cast_possible_wrap`, `cast_sign_loss` e `disallowed_types` (`HashMap`, `HashSet`, `RandomState`), todos em `deny`, mais `unsafe_code = "forbid"`. Não dá para herdar da workspace e acrescentar, então a tabela do núcleo é completa.

## Decisões

- SHA-1 próprio (cerca de 60 linhas), sem dependência no `core` nem no `rom`. Serve para identificar a ROM, não para segurança.
- Testes de `sonic1-web` só em nativo. Em `wasm32-wasip1` as funções `wasm_bindgen` esperam um host JavaScript, e o `wasmtime` não o tem. Os crates `core` e `rom` rodam nos dois.
- O `anti-rom` usa a mesma regra dos hooks. Um teste confere que a lista de extensões é igual à de `hooks/lib.sh`, para as duas não divergirem.
- O site tem CSP restritiva, lê o arquivo só no navegador, recusa arquivos acima de 16 MiB sem lê-los e escreve na página só com `textContent`.

## Como rodar (na imagem)

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test --target wasm32-wasip1 -p sonic1-core -p sonic1-rom
cargo xtask site        # monta dist/
cargo xtask anti-rom
```

Para ver o site: `cargo xtask site` e servir `dist/` por HTTP (módulos e WebAssembly não carregam por `file://`).

## Evidência (03/11/2026, na imagem `websonic-dev:0.1b`)

- `cargo fmt --check` e `cargo clippy --workspace --all-targets -- -D warnings`: sem erro. O clippy 1.99 pediu `as_chunks` e `if` colapsado, corrigidos no código.
- `cargo test --workspace`: núcleo 4, rom 13, web 1, xtask 8, todos passam.
- `cargo test --target wasm32-wasip1 -p sonic1-core -p sonic1-rom` com o runner `wasmtime`: 4 e 13 passam.
- SHA-1 contra vetores do FIPS, 1 milhão de "a" e fronteiras de preenchimento (55, 56, 63, 64, 65, 119, 120 e 128 bytes), conferidas com `sha1sum`.
- Lints do núcleo: com uma função plantada, o clippy falhou nas quatro regras (soma sem estouro explícito, cast que trunca, ponto flutuante, `HashMap` e `HashSet`). A função foi removida.
- `cargo xtask site` monta `dist/` com `index.html`, `app.js`, `.nojekyll` e `pkg/` (JavaScript de cola e 33 KB de WebAssembly).
- A cola JavaScript e o WebAssembly foram chamados em Node: `abc` recusado com hash `a9993e36...`, arquivo vazio recusado, 512 KiB de zeros recusado por hash. O caminho de aceite da REV01 só é provado com dados sintéticos em Rust (`identificar_com`). A ROM real não foi usada.
- `cargo xtask anti-rom` no repositório: sem suspeitos. Numa cópia descartável com um `.gen` e um arquivo de 300000 bytes commitados e depois apagados, o comando falhou e listou os dois (pega o histórico).

## Não provado

- A página num navegador de verdade. Não há navegador na imagem (decisão aprovada). A conferência fica para o Marcelo: servir `dist/`, escolher um arquivo qualquer (recusa) e a ROM REV01 (aceite).
