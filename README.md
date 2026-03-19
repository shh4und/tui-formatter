# TUI Text Processor

Processador de Texto SQL (por enquanto?) usando [ratatui](https://ratatui.rs/) como interface no terminal e a crate [sqlparser](https://docs.rs/sqlparser/latest/sqlparser/) como analisador léxico e sintático 
## Pré-requisitos

Para compilar e executar é necessário possuir o toolchain do Rust instalado no sistema (`rustc` e `cargo`).
Para instalar, utilize o script oficial (disponível em [rust-lang.org](https://rust-lang.org/tools/install/)):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

- Para compilar e rodar a aplicação imediatamente em ambiente de desenvolvimento:
```bash
cargo run
```

- Para construir o binário otimizado para produção (release):
```bash
cargo build --release
```
O binário executável finalizado ficará localizado em target/release/
