# swc-plugin-console-prefix

SWC Transform to prefix logs. Useful for adding file and line number to logs

## Run example

```sh
cargo run --package swc-plugin-console-source --example usage
```

## Test

```sh
cargo test  --  --exact --nocapture
```

## Build

```sh
# dev
cargo build --target=wasm32-wasi

# release
cargo build --release --target=wasm32-wasi
```
