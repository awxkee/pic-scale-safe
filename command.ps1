# $env:RUSTFLAGS = "-C target-cpu=native"
cargo bench --bench resize_rgb_u16 --manifest-path ./app/Cargo.toml