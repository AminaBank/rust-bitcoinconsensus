FROM rust:1.70-bookworm as builder

RUN apt-get update \
 && apt-get -y dist-upgrade \
    && apt-get install --no-install-recommends -y \
       build-essential \
       ca-certificates \
       clang \
       emscripten \
       libc6-dev-i386 \
       libstdc++-11-dev \
       pkg-config \
       sudo \
       wasi-libc \
    && apt -y autoremove \
    && apt clean \
    && rm -rf /var/lib/apt/lists/*

ARG UID
RUN useradd -m -u $UID satoshi
USER satoshi
WORKDIR /bitcoinconsensus

RUN cargo install wasm-pack
RUN rustup target add wasm32-unknown-emscripten \
 && rustup target add wasm32-unknown-unknown \
 && rustup target add wasm32-wasi