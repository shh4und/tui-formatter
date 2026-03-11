# TUI Text Processor

Template básico de aplicação TUI usando Ratatui

## Pré-requisitos

É necessário possuir o toolchain do Rust instalado no sistema (`rustc` e `cargo`).
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
O binário finalizado ficará localizado em target/release/tui_app_rust.
