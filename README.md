Requirements:
- wasm-tools 1.228.0
- wasmtime from `wasip3-prototyping` commit: https://github.com/bytecodealliance/wasip3-prototyping/commit/523676ee613fe88717ffdae0befe32a430b4cbf9

```
$ cargo build --target wasm32-unknown-unknown --release
$ wasm-tools component new --skip-validation ./target/wasm32-unknown-unknown/release/server.wasm -o server.wasm
$ wasm-tools component new --skip-validation ./target/wasm32-unknown-unknown/release/client.wasm -o client.wasm
$ wasmtime serve server.wasm
$ wasmtime run -S http client.wasm
```
