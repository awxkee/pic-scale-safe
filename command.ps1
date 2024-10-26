$env:RUSTFLAGS = "-C target-cpu=native"
cargo bench --bench resize_rgba_f32 --manifest-path ./app/Cargo.toml