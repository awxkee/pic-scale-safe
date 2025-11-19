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

let src_size = ImageSize::new(dimensions.0 as usize, dimensions.1 as usize);
let dst_size = ImageSize::new(dimensions.0 as usize / 4, dimensions.1 as usize / 4);

let resized = resize_rgb8(&transient, src_size, dst_size, 
                          ResamplingFunction::Lanczos3).unwrap();
```

This project is licensed under either of

- BSD-3-Clause License (see [LICENSE](LICENSE.md))
- Apache License, Version 2.0 (see [LICENSE](LICENSE-APACHE.md))

at your option.
