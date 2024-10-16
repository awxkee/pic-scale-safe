# Fast and safe image scaling in Rust

Example comparison time for downscale RGBA 4928x3279 `8 bit` image in 4 times.

```bash
cargo bench --bench resize_rgba --manifest-path ./app/Cargo.toml
```

|                    | Lanczos3 | Bilinear |
|--------------------|:--------:|:--------:|
| image(aarch64)     |  121.19  |  48.89   |
| pic-scale(aarch64) |  26.90   |  15.13   |
| fir(aarch64)       |  25.93   |  11.30   |
| image(x86)         |  192.52  |  89.87   |
| pic-scale(x86)     |  31.92   |  22.76   |
| fir(x86)           |  15.53   |   9.04   |

Example comparison time for downscale RGB 4928x3279 `8 bit` image in 4 times.

```bash
cargo bench --bench resize_rgb --manifest-path ./app/Cargo.toml
```

|                    | Lanczos3 | Bilinear |
|--------------------|:--------:|:--------:|
| pic-scale(aarch64) |  44.60   |  32.51   |
| fir(aarch64)       |  43.23   |  27.09   |
| image(x86)         |  242.04  |  131.91  |
| pic-scale(x86)     |  82.51   |  69.13   |
| fir(x86)           |  24.37   |  17.93   |

Example comparison time for downscale RGBA 4928x3279 `16 bit` image in two times.

```bash
cargo bench --bench resize_rgba_u16 --manifest-path ./app/Cargo.toml
```

|                    | Lanczos3 | Bilinear |
|--------------------|:--------:|:--------:|
| pic-scale(aarch64) |  176.43  |  124.46  |
| fir(aarch64)       |  180.15  |  77.10   |
| image(x86)         |  196.28  |  194.75  |
| pic-scale(x86)     |  96.84   |  50.99   |
| fir(aarch64)       |  52.73   |  28.35   |

Example comparison time for downscale RGB 4928x3279 `16 bit` image in two times.

```bash
cargo bench --bench resize_rgb_u16 --manifest-path ./app/Cargo.toml
```

|                | Lanczos3 | Bilinear |
|----------------|:--------:|:--------:|
| pic-scale      |  134.08  |  93.14   |
| fir            |  146.44  |  60.14   |
| image(x86)     |  184.06  |  93.90   |
| pic-scale(x86) |  62.29   |  43.48   |
| fir(x86)       |  53.39   |  32.34   |

Example comparison time for downscale RGBA 4928x3279 `f32` image in two times.

```angular2html
cargo bench --bench resize_rgba_f32 --manifest-path ./app/Cargo.toml
```

|                    | Lanczos3 | Bilinear |
|--------------------|:--------:|:--------:|
| image(aarch64)     |  99.00   |  47.67   |
| pic-scale(aarch64) |  60.34   |  19.16   |
| fir(aarch64)       |  105.60  |  37.75   |
| image(x86)         |  164.04  |  98.90   |
| pic-scale(x86)     |  57.39   |  43.84   |
| fir(x86)           |  51.08   |  29.92   |

This project is licensed under either of

- BSD-3-Clause License (see [LICENSE](LICENSE.md))
- Apache License, Version 2.0 (see [LICENSE](LICENSE-APACHE.md))

at your option.
