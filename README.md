# Fast and safe image scaling in Rust

This crate provides zero `unsafe` fast rescaling.

### Example 

```rust
let img = ImageReader::open("./assets/nasa-4928x3279.png")
    .unwrap()
    .decode()
    .unwrap();
let dimensions = img.dimensions();
let transient = img.to_rgb8();

let start = Instant::now();

let src_size = ImageSize::new(dimensions.0 as usize, dimensions.1 as usize);
let dst_size = ImageSize::new(dimensions.0 as usize / 4, dimensions.1 as usize / 4);

let resized = resize_rgb8(&transient, src_size, dst_size, 
                          ResamplingFunction::Lanczos3).unwrap();
```

Example comparison time for downscale RGBA 4928x3279 `8 bit` image in 4 times.

```bash
cargo bench --bench resize_rgba --manifest-path ./app/Cargo.toml
```

|                           | Lanczos3 | Bilinear |
|---------------------------|:--------:|:--------:|
| image(aarch64)            |  121.19  |  48.89   |
| pic-scale(aarch64)        |  26.90   |  15.13   |
| fir(aarch64)              |  25.93   |  11.30   |
| image(x86)                |  192.52  |  88.63   |
| pic-scale(x86)            |  49.79   |  35.98   |
| pic-scale(x86-cpu-native) |  27.21   |  20.48   |
| fir(x86)                  |  42.89   |  24.13   |
| fir(x86-cpu-native)       |  41.17   |  23.62   |

Example comparison time for downscale RGB 4928x3279 `8 bit` image in 4 times.

```bash
cargo bench --bench resize_rgb --manifest-path ./app/Cargo.toml
```

|                    | Lanczos3 | Bilinear |
|--------------------|:--------:|:--------:|
| image(aarch64)     |  123.85  |  51.30   |
| pic-scale(aarch64) |  31.73   |  18.20   |
| fir(aarch64)       |  24.04   |  11.37   |
| image(x86)         |  201.52  |  90.82   |
| pic-scale(x86)     |  34.54   |  25.05   |
| fir(x86)           |  41.97   |  25.21   |

Example comparison time for downscale RGBA 4928x3279 `16 bit` image in 4 times.

```bash
cargo bench --bench resize_rgba_u16 --manifest-path ./app/Cargo.toml
```

|                           | Lanczos3 | Bilinear |
|---------------------------|:--------:|:--------:|
| image(aarch64)            |  123.27  |  52.91   |
| pic-scale(aarch64)        |  28.041  |  18.89   |
| fir(aarch64)              |  149.87  |  50.08   |
| image(x86)                |  196.28  |  194.75  |
| pic-scale(x86)            |  59.89   |  57.99   |
| pic-scale(x86-cpu-native) |  44.07   |  57.99   |
| fir(x86)                  |  52.73   |  28.35   |

Example comparison time for downscale RGB 4928x3279 `16 bit` image in 4 times.

```bash
cargo bench --bench resize_rgb_u16 --manifest-path ./app/Cargo.toml
```

|                    | Lanczos3 | Bilinear |
|--------------------|:--------:|:--------:|
| image(aarch64)     |  130.45  |  61.06   |
| pic-scale(aarch64) |  36.10   |  23.80   |
| fir(aarch64)       |  122.01  |  43.36   |

Example comparison time for downscale RGBA 4928x3279 `f32` image in 4 times.

```bash
cargo bench --bench resize_rgba_f32 --manifest-path ./app/Cargo.toml
```

|                    | Lanczos3 | Bilinear |
|--------------------|:--------:|:--------:|
| image(aarch64)     |  100.16  |  51.21   |
| pic-scale(aarch64) |  43.04   |  19.16   |
| fir(aarch64)       |  114.35  |  37.75   |
| image(x86)         |  164.04  |  98.90   |
| pic-scale(x86)     |  57.39   |  43.84   |
| fir(x86)           |  60.30   |  29.92   |

This project is licensed under either of

- BSD-3-Clause License (see [LICENSE](LICENSE.md))
- Apache License, Version 2.0 (see [LICENSE](LICENSE-APACHE.md))

at your option.
