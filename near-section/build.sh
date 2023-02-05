#!/bin/bash
cargo fmt --all

#Without any ABI. Plain compilation
cargo build --all --target wasm32-unknown-unknown --release

# Have ABI
if [ ! -d "res" ]; then
    mkdir -p "res"
fi
# if [ ! -d "abi" ]; then
#     mkdir -p "abi"
# fi

# Example
# cargo near build --manifest-path ./contracts/ft/Cargo.toml --out-dir ./res -r
# cargo near build --manifest-path ./contracts/exploit/Cargo.toml --out-dir ./res -r
# cargo near build --manifest-path ./contracts/staking/Cargo.toml --out-dir ./res -r

# mv ./res/*.json ./abi

cp ./target/wasm32-unknown-unknown/release/*.wasm ./res
