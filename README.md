# Experiments

These are my experiments with wasmtime, the wasm component model and wasi preview 2.
Specifically, I'm interested in creating wasm components that can make http requests.
I created an App Host to do my experiments with the wasi http proxy.

You could also use wasmtime serve and have your component handle an http request incoming and the use the outbound http request, but I wanted to be able to test the http request/response from the component itself.  This is what App Host is for.  I learned a lot about embedding wasmtime for a wasm component and http use case.

These are just my experiments for learning how to use wasmtime and the wasm component model.

## Build the components
```bash
chmod +x ./build.sh
./build.sh
```

## Testing components with App Host
```bash
cd app_host
cargo run -- "../web.wasm" "make-get-request" "https://dog.ceo/api/breeds/image/random"

cargo run -- "../web.wasm" "make-post-request" "https://httpbin.org/post" "{\"test_key\":\"test_value\", \"test_key2\":\"test_value2\"}"

cargo run -- "../composed.wasm" "eval-expression" mult 1 2
cargo run -- "../composed.wasm" "eval-expression" add 1 2
```

## Testing component composition
Check out the build.sh script to see how the components are built and linked together.
```bash
wasmtime run final.wasm 1 2 add
wasmtime run final.wasm 1 2 mult

WASMTIME_LOG=wasmtime_wasi=trace wasmtime run final.wasm 1 2 add
```

## Compile and run the component for better performance
```bash
wasmtime compile final.wasm
wasmtime --allow-precompiled final.cwasm 1 5 mult
```



