# swc-plugin-console-prefix
SWC Transform to prefix logs. Useful for adding file and line number to logs

## Run example
```sh
# macos
cargo run --package swc-plugin-console-source --example usage --target=x86_64-apple-darwin
# linux
cargo run --package swc-plugin-console-source --example usage --target=x86_64-unknown-linux-gnu
```

## Tests
```sh
# macos
cargo test_apple  --  --exact --nocapture
# linux
cargo test_linux  --  --exact --nocapture  
```
