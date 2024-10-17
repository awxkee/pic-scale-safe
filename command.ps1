$env:RUSTFLAGS = "-C target-cpu=native"
cargo bench --bench resize_rgba_u16 --manifest-path ./app/Cargo.toml