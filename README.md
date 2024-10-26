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
| pic-scale(aarch64)        |  11.89   |   8.92   |
| fir(aarch64)              |  25.89   |  11.30   |
| image(x86)                |  192.52  |  88.63   |
| pic-scale(x86)            |  25.50   |  18.37   |
| fir(x86)                  |  42.89   |  24.13   |
| image(x86-cpu-native)     |  205.64  |  89.02   |
| pic-scale(x86-cpu-native) |  14.39   |  11.31   |
| fir(x86-cpu-native)       |  41.21   |  22.77   |

Example comparison time for downscale RGB 4928x3279 `8 bit` image in 4 times.

```bash
cargo bench --bench resize_rgb --manifest-path ./app/Cargo.toml
```

|                           | Lanczos3 | Bilinear |
|---------------------------|:--------:|:--------:|
| image(aarch64)            |  123.85  |  51.30   |
| pic-scale(aarch64)        |  17.23   |  12.32   |
| fir(aarch64)              |  23.61   |  10.53   |
| image(x86)                |  201.52  |  90.82   |
| pic-scale(x86)            |  27.17   |  21.08   |
| fir(x86)                  |  41.97   |  24.39   |
| image(x86-cpu-native)     |  184.57  |  84.69   |
| pic-scale(x86-cpu-native) |  20.96   |  15.16   |
| fir(x86-cpu-native)       |  41.49   |  20.38   |

Example comparison time for downscale RGBA 4928x3279 `16 bit` image in 4 times.

```bash
cargo bench --bench resize_rgba_u16 --manifest-path ./app/Cargo.toml
```

|                           | Lanczos3 | Bilinear |
|---------------------------|:--------:|:--------:|
| image(aarch64)            |  262.32  |  76.91   |
| pic-scale(aarch64)        |  15.49   |  11.38   |
| fir(aarch64)              |  141.78  |  50.08   |
| image(x86)                |  196.28  |  107.78  |
| pic-scale(x86)            |  57.48   |  50.85   |
| fir(x86)                  |  139.56  |  58.48   |
| image(x86-cpu-native)     |  192.85  |  102.05  |
| pic-scale(x86-cpu-native) |  39.60   |  46.44   |
| fir(x86-cpu-native)       |  101.48  |  52.58   |

Example comparison time for downscale RGB 4928x3279 `16 bit` image in 4 times.

```bash
cargo bench --bench resize_rgb_u16 --manifest-path ./app/Cargo.toml
```

|                           | Lanczos3 | Bilinear |
|---------------------------|:--------:|:--------:|
| image(aarch64)            |  130.45  |  57.38   |
| pic-scale(aarch64)        |  16.17   |  12.11   |
| fir(aarch64)              |  110.06  |  42.04   |
| image(x86)                |  204.10  |  148.34  |
| pic-scale(x86)            |  43.21   |  145.73  |
| fir(x86)                  |  210.28  |  51.29   |
| image(x86-cpu-native)     |  190.21  |  98.42   |
| pic-scale(x86-cpu-native) |  33.48   |  28.50   |
| fir(x86-cpu-native)       |  72.88   |  45.17   |

Example comparison time for downscale RGBA 4928x3279 `f32` image in 4 times.

```bash
cargo bench --bench resize_rgba_f32 --manifest-path ./app/Cargo.toml
```

|                           | Lanczos3 | Bilinear |
|---------------------------|:--------:|:--------:|
| image(aarch64)            |  100.16  |  50.09   |
| pic-scale(aarch64)        |  14.07   |  11.18   |
| fir(aarch64)              |  105.30  |  37.75   |
| image(x86)                |  208.25  |  107.84  |
| pic-scale(x86)            |  33.55   |  28.97   |
| fir(x86)                  |  92.38   |  74.12   |
| image(x86-cpu-native)     |  162.83  |  108.54  |
| pic-scale(x86-cpu-native) |  33.13   |  28.54   |
| fir(x86-cpu-native)       |  56.65   |  59.96   |

This project is licensed under either of

- BSD-3-Clause License (see [LICENSE](LICENSE.md))
- Apache License, Version 2.0 (see [LICENSE](LICENSE-APACHE.md))

at your option.
