# $env:RUSTFLAGS = "-C target-cpu=native"
cargo bench --bench resize_rgba --manifest-path ./app/Cargo.toml