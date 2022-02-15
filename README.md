# swc-plugin-console-prefix

SWC Transform to prefix logs. Useful for adding file and line number to logs

```json
{
    "jsc": {
        "experimental": {
            "plugins": [
                ["swc-plugin-console-prefix", { "prefixPattern": "hello", "ignore": ["info"] }]
            ]
        }
    }
}
```

```js
// test.js
console.log('world')
```

```bash
$ npx swc ./test.js
```

```js
// output
console.log('hello', 'world')
```

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
