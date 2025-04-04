```
$ cargo build --target wasm32-unknown-unknown --release
$ wasm-tools component new --skip-validation ./target/wasm32-unknown-unknown/release/server.wasm -o server.wasm
$ wasm-tools component new --skip-validation ./target/wasm32-unknown-unknown/release/client.wasm -o client.wasm
$ wasmtime serve server.wasm
$ wasmtime run -S http client.wasm
```
