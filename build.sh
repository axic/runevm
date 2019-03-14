#!/usr/bin/env bash

set -e

# build interpreter
cargo build --release --target wasm32-unknown-unknown
echo "Code size after compilation: $(wc -c < target/wasm32-unknown-unknown/release/runevm.wasm)"

# strip code
chisel target/wasm32-unknown-unknown/release/runevm.wasm target/wasm32-unknown-unknown/release/runevm.wasm
echo "Code size after stripping: $(wc -c < target/wasm32-unknown-unknown/release/runevm.wasm)"

# build deployer
wat2wasm -o target/deployer.wasm src/deployer.wast

# calculate size
size=$(wc -c < target/wasm32-unknown-unknown/release/runevm.wasm)
# store the file size as a 32-bit little endian number
printf "0: %.8x" $size | sed -E 's/0: (..)(..)(..)(..)/0: \4\3\2\1/' | xxd -r -g0 >target/le32size.bin
echo "Interpreter code size: $size"

# create deployment code
cat target/deployer.wasm target/wasm32-unknown-unknown/release/runevm.wasm target/le32size.bin >target/runevm.wasm
echo "Built evm2wasm compatible version as target/runevm.wasm"
echo "Total size: $(wc -c < target/runevm.wasm)"
