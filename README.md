# Fast and safe image scaling in Rust

Example comparison time for downscale RGBA 4928x3279 `8 bit` image in 4 times.

|                    | Lanczos3 | Bilinear |
|--------------------|:--------:|:--------:|
| pic-scale(aarch64) |  34.82   |  25.89   |
| fir(aarch64)       |  53.02   |  34.47   |
| image(x86)         |  192.52  |  89.87   |
| pic-scale(x86)     |  39.76   |  27.63   |
| fir(x86)           |  15.75   |   9.04   |

Example comparison time for downscale RGB 4928x3279 `8 bit` image in two times.

|           | Lanczos3 | Bilinear |
|-----------|:--------:|:--------:|
| pic-scale |  44.60   |  32.51   |
| fir       |  43.23   |  27.09   |

Example comparison time for downscale RGBA 4928x3279 `16 bit` image in two times.

|           | Lanczos3 | Bilinear |
|-----------|:--------:|:--------:|
| pic-scale |  176.43  |  124.46  |
| fir       |  180.15  |  77.10   |

Example comparison time for downscale RGB 4928x3279 `16 bit` image in two times.

|           | Lanczos3 | Bilinear |
|-----------|:--------:|:--------:|
| pic-scale |  134.08  |  93.14   |
| fir       |  146.44  |  60.14   |

Example comparison time for downscale RGBA 4928x3279 `f32` image in two times.

|           | Lanczos3 | Bilinear |
|-----------|:--------:|:--------:|
| pic-scale |  46.46   |  36.29   |
| fir       |  105.60  |  38.37   |

This project is licensed under either of

- BSD-3-Clause License (see [LICENSE](LICENSE.md))
- Apache License, Version 2.0 (see [LICENSE](LICENSE-APACHE.md))

at your option.
