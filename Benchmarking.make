cargo run --manifest-path node/Cargo.toml --features runtime-benchmarks --
benchmark --help
cargo run --manifest-path node/Cargo.toml --features runtime-benchmarks --
benchmark --extrinsic '*' --pallet '*' --wasm-execution=compiled
cargo test --manifest-path pallets/kitties/Cargo.toml --features
runtime-benchmarks -- --nocapture
cargo run --manifest-path node/Cargo.toml --release --features
runtime-benchmarks -- benchmark --extrinsic '*' --pallet pallet_kitties
--output pallets/kitties/src/weights.rs --template=frame-weight-template.hbs
--execution=wasm --wasm-execution=compiled
cargo run --manifest-path node/Cargo.toml --release --features
runtime-benchmarks -- benchmark --extrinsic '*' --pallet pallet_kitties
--output runtime/src/weights/pallet_kitties.rs --execution=wasm
--wasm-execution=compiled