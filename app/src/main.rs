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
use image::{GenericImageView, ImageReader};
use pic_scale_safe::{resize_rgb8, ImageSize, ResamplingFunction};
use std::time::Instant;

fn main() {
    let img = ImageReader::open("./assets/nasa-4928x3279.png")
        .unwrap()
        .decode()
        .unwrap();
    let dimensions = img.dimensions();
    let transient = img.to_rgb8();

    let start = Instant::now();

    let src_size = ImageSize::new(dimensions.0 as usize, dimensions.1 as usize);
    let dst_size = ImageSize::new(dimensions.0 as usize / 4, dimensions.1 as usize / 4);

    let resized = resize_rgb8(
        &transient,
        src_size,
        dst_size,
        ResamplingFunction::Ginseng,
    )
    .unwrap();

    println!("Working time {:?}", start.elapsed());

    // let shifted = resized.iter().map(|&x| (x >> 4) as u8).collect::<Vec<_>>();

    image::save_buffer(
        "converted.jpg",
        &resized,
        dst_size.width as u32,
        dst_size.height as u32,
        image::ColorType::Rgb8,
    )
    .unwrap();
}
