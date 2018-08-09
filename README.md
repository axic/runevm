# RuneVM

EVM interpreter compatible with the evm2wasm interface.

This is a drop in replacement for [evm2wasm](https://github.com/ewasm/evm2wasm).

It uses the EVM interpreter from [parity-ethereum](https://github.com/paritytech/parity-ethereum/).

### Technical detail

At runtime the client would call the evm2wasm contract and pass EVM bytecode as an input.
To that the contract would respond with an ewasm bytecode, which can be executed on its own.

Instead of that, *runevm* just returns itself, because during execution time of the translated
contract, the `codecopy` operation still accesses the original EVM bytecode and as a result it
is possible to acquire the code to be run.

*Runevm* runs as as interpreter off the code acquired via *codecopy* and behaves just like a
regular contract.

To achieve this, it consists of two parts:
- deployer
- interpreter

The deployer is a tiny wasm wrapper, which only returns the interpreter.

To make compilation easier, the deployer is hand written in WAT (WebAssembly Text) and
expects a 32-bit little endian number at the end of the code containing the bytecode size
of the interpreter. It will *codecopy* itself to memory and return the memory section
corresponding to the interpreter.

### Building

Compile it with:
```
cargo build --release --target wasm32-unknown-unknown
```

## Author(s)

Alex Beregszaszi

## License

Apache 2.0
