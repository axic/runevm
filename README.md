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

### Building

Run:
```shell
$ cargo build --release
$ chisel run
```

The resulting binary will be at `target/runevm.wasm`.

It expects [chisel](https://github.com/wasmx/wasm-chisel) is installed:
```shell
$ cargo install chisel
```

## Author(s)

Alex Beregszaszi

## License

Apache 2.0
