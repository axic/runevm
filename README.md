# RuneVM

EVM interpreter compatible with the evm2wasm interface.

This is a drop in replacement for [evm2wasm](https://github.com/ewasm/evm2wasm).

It uses the EVM interpreter from [parity-ethereum](https://github.com/paritytech/parity-ethereum/).

### Building

Compile it with:
```
cargo build --release --target wasm32-unknown-unknown
```

## Author(s)

Alex Beregszaszi

## License

Apache 2.0
