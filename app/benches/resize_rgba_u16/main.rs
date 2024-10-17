/*
 * Copyright (c) Radzivon Bartoshyk, 10/2024. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without modification,
 * are permitted provided that the following conditions are met:
 *
 * 1.  Redistributions of source code must retain the above copyright notice, this
 * list of conditions and the following disclaimer.
 *
 * 2.  Redistributions in binary form must reproduce the above copyright notice,
 * this list of conditions and the following disclaimer in the documentation
 * and/or other materials provided with the distribution.
 *
 * 3.  Neither the name of the copyright holder nor the names of its
 * contributors may be used to endorse or promote products derived from
 * this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
 * FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
 * SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
 * CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
 * OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

use criterion::{criterion_group, criterion_main, Criterion};
use fast_image_resize::images::Image;
use fast_image_resize::FilterType::{Bilinear, Lanczos3};
use fast_image_resize::{CpuExtensions, PixelType, ResizeAlg, ResizeOptions, Resizer};
use image::imageops::FilterType;
use image::{DynamicImage, EncodableLayout, GenericImageView, ImageReader};
use pic_scale_safe::{resize_fixed_point, resize_floating_point, ImageSize, ResamplingFunction};

pub fn criterion_benchmark(c: &mut Criterion) {
    let img = ImageReader::open("../assets/nasa-4928x3279.png")
        .unwrap()
        .decode()
        .unwrap();
    let dimensions = img.dimensions();
    let source_bytes = img.to_rgba8();

    let rgba16 = img.to_rgba16();
    let rgba16_dyn_image = DynamicImage::ImageRgba16(rgba16);

    let source_8bit = source_bytes.as_bytes();
    let src_bytes = source_8bit
        .iter()
        .map(|&x| (x as u16) << 10)
        .collect::<Vec<_>>();

    c.bench_function("Pic scale RGBA16: Lanczos 3", |b| {
        b.iter(|| {
            _ = resize_floating_point::<u16, f32, f32, 4>(
                &src_bytes,
                ImageSize::new(dimensions.0 as usize, dimensions.1 as usize),
                ImageSize::new(dimensions.0 as usize / 4, dimensions.1 as usize / 4),
                16,
                ResamplingFunction::Lanczos3,
            )
            .unwrap();
        })
    });

    c.bench_function("Pic scale RGBA10: Lanczos 3", |b| {
        let image_10_bit = source_8bit
            .iter()
            .map(|&x| (x as u16) << 2)
            .collect::<Vec<_>>();
        b.iter(|| {
            _ = resize_floating_point::<u16, f32, f32, 4>(
                &image_10_bit,
                ImageSize::new(dimensions.0 as usize, dimensions.1 as usize),
                ImageSize::new(dimensions.0 as usize / 4, dimensions.1 as usize / 4),
                10,
                ResamplingFunction::Lanczos3,
            )
            .unwrap();
        })
    });

    c.bench_function("Pic scale RGBA10: Bilinear", |b| {
        let image_10_bit = source_8bit
            .iter()
            .map(|&x| (x as u16) << 2)
            .collect::<Vec<_>>();
        b.iter(|| {
            _ = resize_floating_point::<u16, f32, f32, 4>(
                &image_10_bit,
                ImageSize::new(dimensions.0 as usize, dimensions.1 as usize),
                ImageSize::new(dimensions.0 as usize / 4, dimensions.1 as usize / 4),
                10,
                ResamplingFunction::Bilinear,
            )
            .unwrap();
        })
    });

    c.bench_function("Image RGBA: Lanczos 3", |b| {
        b.iter(|| {
            _ = rgba16_dyn_image.clone().resize_exact(
                dimensions.0 / 4,
                dimensions.1 / 4,
                FilterType::Lanczos3,
            );
        })
    });

    let mut transmuted_form = vec![];
    for &pixel in src_bytes.iter() {
        let bytes = pixel.to_le_bytes();
        transmuted_form.push(bytes[0]);
        transmuted_form.push(bytes[1]);
    }

    c.bench_function("Fast image resize RGBA16: Lanczos 3", |b| {
        let mut vc = transmuted_form.to_vec();
        b.iter(|| {
            let pixel_type: PixelType = PixelType::U16x4;
            let src_image =
                Image::from_slice_u8(dimensions.0, dimensions.1, &mut vc, pixel_type).unwrap();
            let mut dst_image = Image::new(dimensions.0 / 4, dimensions.1 / 4, pixel_type);

            let mut resizer = Resizer::new();
            unsafe {
                resizer.set_cpu_extensions(CpuExtensions::None);
            }
            resizer
                .resize(
                    &src_image,
                    &mut dst_image,
                    &ResizeOptions::new()
                        .resize_alg(ResizeAlg::Convolution(Lanczos3))
                        .use_alpha(false),
                )
                .unwrap();
        })
    });

    c.bench_function("Pic scale RGBA16: Bilinear", |b| {
        b.iter(|| {
            _ = resize_floating_point::<u16, f32, f32, 4>(
                &src_bytes,
                ImageSize::new(dimensions.0 as usize, dimensions.1 as usize),
                ImageSize::new(dimensions.0 as usize / 4, dimensions.1 as usize / 4),
                16,
                ResamplingFunction::Bilinear,
            )
            .unwrap();
        })
    });

    c.bench_function("Image RGBA: Bilinear", |b| {
        b.iter(|| {
            _ = rgba16_dyn_image.clone().resize_exact(
                dimensions.0 / 4,
                dimensions.1 / 4,
                FilterType::Triangle,
            );
        })
    });

    c.bench_function("Fast image resize RGBA16: Bilinear", |b| {
        let mut vc = transmuted_form.to_vec();
        b.iter(|| {
            let pixel_type: PixelType = PixelType::U16x4;
            let src_image =
                Image::from_slice_u8(dimensions.0, dimensions.1, &mut vc, pixel_type).unwrap();
            let mut dst_image = Image::new(dimensions.0 / 4, dimensions.1 / 4, pixel_type);

            let mut resizer = Resizer::new();
            unsafe {
                resizer.set_cpu_extensions(CpuExtensions::None);
            }
            resizer
                .resize(
                    &src_image,
                    &mut dst_image,
                    &ResizeOptions::new()
                        .resize_alg(ResizeAlg::Convolution(Bilinear))
                        .use_alpha(false),
                )
                .unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
