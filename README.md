# instant_json
Blazingly fast, schema-based JSON parsing


## Contributing

### Commit messages
Please use [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).

### WASM
For building the WASM module, install **wasm-pack** and add the wasm32 target:

```shell
cargo install wasm-pack
rustup target add wasm32-unknown-unknown
```

And compile it to a WASM-module using `wasm-pack build --target web`.
Run tests with `wasm-pack test --node`.



