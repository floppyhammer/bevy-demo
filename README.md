## Run on web

1. Install wasm toolchain
```bash
rustup target install wasm32-unknown-unknown
```

2. Install a server runner
```bash
cargo install wasm-server-runner
```

3. Run
```bash
cargo run --target wasm32-unknown-unknown
```
