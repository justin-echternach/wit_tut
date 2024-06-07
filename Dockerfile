FROM rust:1.78.0-buster

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libssl-dev \
    clang \
    pkg-config \
    curl && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-component && \
    cargo install wasm-tools && \
    cargo install wit-bindgen-cli

RUN curl https://wasmtime.dev/install.sh -sSf | bash
