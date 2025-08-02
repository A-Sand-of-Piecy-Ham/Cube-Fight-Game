cargo build --bin client --release --target wasm32-unknown-unknown && \
wasm-bindgen --out-dir webbuild/pkg/ --target web ./target/wasm32-unknown-unknown/release/client.wasm