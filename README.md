# Tutorial

This is a tutorial on how to use the Wasmtime Rust bindings to create a simple calculator.

```bash
cargo component new add --lib
cargo component new calculator --lib
cargo component new command --command
cargo component build

wasm-tools compose calculator.wasm -d add.wasm -o composed.wasm
wasm-tools compose command.wasm -d composed.wasm -o final.wasm

wasmtime run final.wasm 1 2 add
wasmtime run final.wasm 1 2 mult

WASMTIME_LOG=wasmtime_wasi=trace wasmtime run final.wasm 1 2 add

wasmtime compile final.wasm
wasmtime --allow-precompiled final.cwasm 1 5 mult

cd app_host
cargo run -- "../web/target/wasm32-wasi/debug/web.wasm" "make-get-request" "https://dog.ceo/api/breeds/image/random"

cargo run -- "../web/target/wasm32-wasi/debug/web.wasm" "make-post-request" "https://httpbin.org/post" "{\"test_key\":\"test_value\", \"test_key2\":\"test_value2\"}"

cargo run -- "../composed.wasm" "eval-expression" mult 1 2
cargo run -- "../composed.wasm" "eval-expression" add 1 2
```

