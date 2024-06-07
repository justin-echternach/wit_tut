#!/bin/bash

cd add
cargo component build --release
cd ../calculator
cargo component build --release
cd ../command
cargo component build --release
cd ..

cp add/target/wasm32-wasi/release/add.wasm add.wasm
cp calculator/target/wasm32-wasi/release/calculator.wasm calculator.wasm
cp command/target/wasm32-wasi/release/command.wasm command.wasm

wasm-tools compose calculator.wasm -d add.wasm -o composed.wasm
wasm-tools compose command.wasm -d composed.wasm -o final.wasm

