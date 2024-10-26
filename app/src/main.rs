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
mod image_wrapper;

use fast_image_resize::images::Image;
use fast_image_resize::{CpuExtensions, FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};
use image::{
    imageops, DynamicImage, EncodableLayout, GenericImageView, ImageBuffer, ImageFormat,
    ImageReader, Rgb, RgbImage,
};
use pic_scale_safe::{premultiply_rgba8, resize_fixed_point, resize_floating_point, resize_rgb16, resize_rgb8, resize_rgb_f32, resize_rgba16, resize_rgba8, unpremultiply_rgba8, ImageSize, ResamplingFunction};
use std::ops::{BitXor, Shr};
use std::time::Instant;

fn main() {
    let img = ImageReader::open("./assets/nasa-4928x3279.png")
        .unwrap()
        .decode()
        .unwrap();
    let dimensions = img.dimensions();
    let transient = img.to_rgba8();

    let mut working_store = transient.to_vec();

    let start = Instant::now();

    let src_size = ImageSize::new(dimensions.0 as usize, dimensions.1 as usize);
    let dst_size = ImageSize::new(
        dimensions.0 as usize / 2,
        dimensions.1 as usize / 2,
    );

    premultiply_rgba8(&mut working_store);

    let mut resized = resize_rgba8(
        &working_store,
        src_size,
        dst_size,
        ResamplingFunction::Lanczos3,
    )
    .unwrap();

    // unpremultiply_rgba8(&mut resized);

    println!("Working time {:?}", start.elapsed());

    // let rgba_image = DynamicImage::ImageRgb16(ImageBuffer::<Rgb<u16>, Vec<u16>>::from_vec(dimensions.0 * 4, dimensions.1 / 4, resized).unwrap());
    // rgba_image.save_with_format("converted.png", ImageFormat::Png).unwrap();

    // let shifted = resized
    //     .iter()
    //     .map(|&x| (x * 255.) as u8)
    //     .collect::<Vec<_>>();

    image::save_buffer(
        "converted.png",
        &resized,
        dst_size.width as u32,
        dst_size.height as u32,
        image::ColorType::Rgba8,
    )
    .unwrap();

    // let mut src_bytes = transient.as_bytes().to_vec();
    // let pixel_type: PixelType = PixelType::U8x3;
    // let src_image =
    //     Image::from_slice_u8(dimensions.0, dimensions.1, &mut src_bytes, pixel_type).unwrap();
    // let mut dst_image = Image::new(dimensions.0 / 8, dimensions.1 / 8, pixel_type);
    //
    // let mut resizer = Resizer::new();
    // unsafe {
    //     resizer.set_cpu_extensions(CpuExtensions::None);
    // }
    //
    // let start = Instant::now();
    //
    // resizer
    //     .resize(
    //         &src_image,
    //         &mut dst_image,
    //         &ResizeOptions::new()
    //             .resize_alg(ResizeAlg::Convolution(FilterType::Bilinear))
    //             .use_alpha(false),
    //     )
    //     .unwrap();
    //
    // println!("Working time {:?}", start.elapsed());
    //
    // // let img = u8_to_u16(dst_image.buffer());
    // //
    // // let rgba_image = DynamicImage::ImageRgb8(RgbImage::from_raw(dst_image.width() as u32, dst_image.height() as u32, dst_image.buffer().to_vec()).unwrap());
    // // rgba_image.save_with_format("fast_image.png", ImageFormat::Png).unwrap();
    // image::save_buffer(
    //     "fast_image.png",
    //     dst_image.buffer(),
    //     dst_image.width(),
    //     dst_image.height(),
    //     image::ColorType::Rgb8,
    // )
    // .unwrap();
}

fn u8_to_u16(u8_buffer: &[u8]) -> &[u16] {
    let len = u8_buffer.len() / 2;
    unsafe { std::slice::from_raw_parts(u8_buffer.as_ptr() as *const u16, len) }
}
