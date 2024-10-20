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
use image::{
    DynamicImage, GrayAlphaImage, GrayImage, ImageBuffer, Luma, LumaA, Rgb, Rgb32FImage, RgbImage,
    Rgba, Rgba32FImage, RgbaImage,
};
use pic_scale_safe::{
    image16_to_linear16, image_f32_to_linear_f32, image_to_linear, linear16_to_gamma_image16,
    linear_f32_to_gamma_image_f32, linear_to_gamma_image, premultiply_la8, premultiply_rgba16,
    premultiply_rgba8, premultiply_rgba_f32, resize_plane16, resize_plane16_with_alpha,
    resize_plane8, resize_plane8_with_alpha, resize_rgb16, resize_rgb8, resize_rgb_f32,
    resize_rgba16, resize_rgba8, resize_rgba_f32, unpremultiply_la16, unpremultiply_la8,
    unpremultiply_rgba16, unpremultiply_rgba8, unpremultiply_rgba_f32, ImageSize,
    ResamplingFunction, TransferFunction,
};

#[derive(Copy, Clone, Debug, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub enum ColorSpace {
    Gamma,
    Linear(TransferFunction),
}

/// Resizes `image` crate image
///
/// # Arguments
///
/// * `image`: Source image
/// * `destination_size`: Destination image size
/// * `resampling_function`: see [ResamplingFunction]
/// * `color_space`: Working color space
/// * `is_alpha_associated`: Flag if alpha is premultiplied
///
/// returns: Result<DynamicImage, String>
///
/// # Examples
///
/// ```
///
/// ```
pub fn resize_image(
    image: DynamicImage,
    destination_size: ImageSize,
    resampling_function: ResamplingFunction,
    color_space: ColorSpace,
    is_alpha_associated: bool,
) -> Result<DynamicImage, String> {
    let source_size = ImageSize::new(image.width() as usize, image.height() as usize);
    match image {
        DynamicImage::ImageLuma8(planar_image) => {
            let mut source_data: &[u8] = planar_image.as_raw();
            let mut working_vec = vec![];
            match color_space {
                ColorSpace::Gamma => {
                    // pass through
                }
                ColorSpace::Linear(trc) => {
                    working_vec = source_data.to_vec();
                    image_to_linear::<1>(&mut working_vec, trc);
                    source_data = working_vec.as_slice();
                }
            }

            let mut result = resize_plane8(
                source_data,
                source_size,
                destination_size,
                resampling_function,
            )?;
            working_vec.resize(0, 0);

            match color_space {
                ColorSpace::Gamma => {
                    // pass through
                }
                ColorSpace::Linear(trc) => {
                    linear_to_gamma_image::<1>(&mut result, trc);
                }
            }

            let gray_image = match GrayImage::from_raw(
                destination_size.width as u32,
                destination_size.height as u32,
                result,
            ) {
                None => {
                    return Err("Image resizing failed".to_string());
                }
                Some(val) => val,
            };

            Ok(DynamicImage::ImageLuma8(gray_image))
        }
        DynamicImage::ImageLumaA8(planar_image_with_alpha) => {
            let mut source_data: &[u8] = planar_image_with_alpha.as_raw();
            let mut working_vec = vec![];

            if is_alpha_associated {
                if working_vec.is_empty() {
                    working_vec = source_data.to_vec();
                }
                unpremultiply_la8(&mut working_vec);
            }

            match color_space {
                ColorSpace::Gamma => {
                    // pass through
                }
                ColorSpace::Linear(trc) => {
                    working_vec = source_data.to_vec();
                    image_to_linear::<2>(&mut working_vec, trc);
                    source_data = working_vec.as_slice();
                }
            }

            let mut result = resize_plane8_with_alpha(
                source_data,
                source_size,
                destination_size,
                resampling_function,
            )?;
            working_vec.resize(0, 0);

            match color_space {
                ColorSpace::Gamma => {
                    // pass through
                }
                ColorSpace::Linear(trc) => {
                    linear_to_gamma_image::<2>(&mut result, trc);
                }
            }

            if is_alpha_associated {
                premultiply_la8(&mut result);
            }

            let gray_image = match GrayAlphaImage::from_raw(
                destination_size.width as u32,
                destination_size.height as u32,
                result,
            ) {
                None => {
                    return Err("Image resizing failed".to_string());
                }
                Some(val) => val,
            };

            Ok(DynamicImage::ImageLumaA8(gray_image))
        }
        DynamicImage::ImageRgb8(rgb_image) => {
            let mut source_data: &[u8] = rgb_image.as_raw();
            let mut working_vec = vec![];
            match color_space {
                ColorSpace::Gamma => {
                    // pass through
                }
                ColorSpace::Linear(trc) => {
                    if working_vec.is_empty() {
                        working_vec = source_data.to_vec();
                    }
                    image_to_linear::<3>(&mut working_vec, trc);
                    source_data = working_vec.as_slice();
                }
            }

            let mut result = resize_rgb8(
                source_data,
                source_size,
                destination_size,
                resampling_function,
            )?;
            working_vec.resize(0, 0);

            match color_space {
                ColorSpace::Gamma => {
                    // pass through
                }
                ColorSpace::Linear(trc) => {
                    linear_to_gamma_image::<3>(&mut result, trc);
                }
            }

            let gray_image = match RgbImage::from_raw(
                destination_size.width as u32,
                destination_size.height as u32,
                result,
            ) {
                None => {
                    return Err("Image resizing failed".to_string());
                }
                Some(val) => val,
            };

            Ok(DynamicImage::ImageRgb8(gray_image))
        }
        DynamicImage::ImageRgba8(rgba_image) => {
            let mut source_data: &[u8] = rgba_image.as_raw();
            let mut working_vec = vec![];

            if is_alpha_associated {
                if working_vec.is_empty() {
                    working_vec = source_data.to_vec();
                }
                unpremultiply_rgba8(&mut working_vec);
            }

            match color_space {
                ColorSpace::Gamma => {
                    // pass through
                }
                ColorSpace::Linear(trc) => {
                    if working_vec.is_empty() {
                        working_vec = source_data.to_vec();
                    }
                    image_to_linear::<4>(&mut working_vec, trc);
                    source_data = working_vec.as_slice();
                }
            }

            let mut result = resize_rgba8(
                source_data,
                source_size,
                destination_size,
                resampling_function,
            )?;
            working_vec.resize(0, 0);

            match color_space {
                ColorSpace::Gamma => {
                    // pass through
                }
                ColorSpace::Linear(trc) => {
                    linear_to_gamma_image::<4>(&mut result, trc);
                }
            }

            if is_alpha_associated {
                premultiply_rgba8(&mut working_vec);
            }

            let gray_image = match RgbaImage::from_raw(
                destination_size.width as u32,
                destination_size.height as u32,
                result,
            ) {
                None => {
                    return Err("Image resizing failed".to_string());
                }
                Some(val) => val,
            };

            Ok(DynamicImage::ImageRgba8(gray_image))
        }
        DynamicImage::ImageLuma16(planar16_image) => {
            let mut source_data: &[u16] = planar16_image.as_raw();
            let mut working_vec = vec![];
            match color_space {
                ColorSpace::Gamma => {
                    // pass through
                }
                ColorSpace::Linear(trc) => {
                    working_vec = source_data.to_vec();
                    image16_to_linear16::<1>(&mut working_vec, 16, trc);
                    source_data = working_vec.as_slice();
                }
            }

            let mut result = resize_plane16(
                source_data,
                source_size,
                destination_size,
                16,
                resampling_function,
            )?;
            working_vec.resize(0, 0);

            match color_space {
                ColorSpace::Gamma => {
                    // pass through
                }
                ColorSpace::Linear(trc) => {
                    linear16_to_gamma_image16::<1>(&mut result, 16, trc);
                }
            }

            let gray_image = match ImageBuffer::<Luma<u16>, Vec<u16>>::from_raw(
                destination_size.width as u32,
                destination_size.height as u32,
                result,
            ) {
                None => {
                    return Err("Image resizing failed".to_string());
                }
                Some(val) => val,
            };

            Ok(DynamicImage::ImageLuma16(gray_image))
        }
        DynamicImage::ImageLumaA16(planar16_with_alpha) => {
            let mut source_data: &[u16] = planar16_with_alpha.as_raw();
            let mut working_vec = vec![];

            if is_alpha_associated {
                if working_vec.is_empty() {
                    working_vec = source_data.to_vec();
                }
                unpremultiply_la16(&mut working_vec, 16);
            }

            match color_space {
                ColorSpace::Gamma => {
                    // pass through
                }
                ColorSpace::Linear(trc) => {
                    if working_vec.is_empty() {
                        working_vec = source_data.to_vec();
                    }
                    image16_to_linear16::<2>(&mut working_vec, 16, trc);
                    source_data = working_vec.as_slice();
                }
            }

            let mut result = resize_plane16_with_alpha(
                source_data,
                source_size,
                destination_size,
                16,
                resampling_function,
            )?;
            working_vec.resize(0, 0);

            match color_space {
                ColorSpace::Gamma => {
                    // pass through
                }
                ColorSpace::Linear(trc) => {
                    linear16_to_gamma_image16::<2>(&mut result, 16, trc);
                }
            }

            if is_alpha_associated {
                premultiply_rgba16(&mut result, 16);
            }

            let gray_image = match ImageBuffer::<LumaA<u16>, Vec<u16>>::from_raw(
                destination_size.width as u32,
                destination_size.height as u32,
                result,
            ) {
                None => {
                    return Err("Image resizing failed".to_string());
                }
                Some(val) => val,
            };

            Ok(DynamicImage::ImageLumaA16(gray_image))
        }
        DynamicImage::ImageRgb16(rgb16_image) => {
            let mut source_data: &[u16] = rgb16_image.as_raw();
            let mut working_vec = vec![];
            match color_space {
                ColorSpace::Gamma => {
                    // pass through
                }
                ColorSpace::Linear(trc) => {
                    working_vec = source_data.to_vec();
                    image16_to_linear16::<3>(&mut working_vec, 16, trc);
                    source_data = working_vec.as_slice();
                }
            }

            let mut result = resize_rgb16(
                source_data,
                source_size,
                destination_size,
                16,
                resampling_function,
            )?;
            working_vec.resize(0, 0);

            match color_space {
                ColorSpace::Gamma => {
                    // pass through
                }
                ColorSpace::Linear(trc) => {
                    linear16_to_gamma_image16::<3>(&mut result, 16, trc);
                }
            }

            let gray_image = match ImageBuffer::<Rgb<u16>, Vec<u16>>::from_raw(
                destination_size.width as u32,
                destination_size.height as u32,
                result,
            ) {
                None => {
                    return Err("Image resizing failed".to_string());
                }
                Some(val) => val,
            };

            Ok(DynamicImage::ImageRgb16(gray_image))
        }
        DynamicImage::ImageRgba16(rgba16_image) => {
            let mut source_data: &[u16] = rgba16_image.as_raw();
            let mut working_vec = vec![];

            if is_alpha_associated {
                if working_vec.is_empty() {
                    working_vec = source_data.to_vec();
                }
                unpremultiply_rgba16(&mut working_vec, 16);
            }

            match color_space {
                ColorSpace::Gamma => {
                    // pass through
                }
                ColorSpace::Linear(trc) => {
                    if working_vec.is_empty() {
                        working_vec = source_data.to_vec();
                    }
                    image16_to_linear16::<4>(&mut working_vec, 16, trc);
                    source_data = working_vec.as_slice();
                }
            }

            let mut result = resize_rgba16(
                source_data,
                source_size,
                destination_size,
                16,
                resampling_function,
            )?;

            match color_space {
                ColorSpace::Gamma => {
                    // pass through
                }
                ColorSpace::Linear(trc) => {
                    linear16_to_gamma_image16::<4>(&mut result, 16, trc);
                }
            }

            if is_alpha_associated {
                premultiply_rgba16(&mut result, 16);
            }

            let gray_image = match ImageBuffer::<Rgba<u16>, Vec<u16>>::from_raw(
                destination_size.width as u32,
                destination_size.height as u32,
                result,
            ) {
                None => {
                    return Err("Image resizing failed".to_string());
                }
                Some(val) => val,
            };

            Ok(DynamicImage::ImageRgba16(gray_image))
        }
        DynamicImage::ImageRgb32F(rgb_f32_image) => {
            let mut source_data: &[f32] = rgb_f32_image.as_raw();
            let mut working_vec = vec![];
            match color_space {
                ColorSpace::Gamma => {
                    // pass through
                }
                ColorSpace::Linear(trc) => {
                    working_vec = source_data.to_vec();
                    image_f32_to_linear_f32::<3>(&mut working_vec, trc);
                    source_data = working_vec.as_slice();
                }
            }

            let mut result = resize_rgb_f32(
                source_data,
                source_size,
                destination_size,
                resampling_function,
            )?;
            working_vec.resize(0, 0.);

            match color_space {
                ColorSpace::Gamma => {
                    // pass through
                }
                ColorSpace::Linear(trc) => {
                    linear_f32_to_gamma_image_f32::<3>(&mut result, trc);
                }
            }

            let gray_image = match Rgb32FImage::from_raw(
                destination_size.width as u32,
                destination_size.height as u32,
                result,
            ) {
                None => {
                    return Err("Image resizing failed".to_string());
                }
                Some(val) => val,
            };

            Ok(DynamicImage::ImageRgb32F(gray_image))
        }
        DynamicImage::ImageRgba32F(rgba_f32_image) => {
            let mut source_data: &[f32] = rgba_f32_image.as_raw();
            let mut working_vec = vec![];

            if is_alpha_associated {
                if working_vec.is_empty() {
                    working_vec = source_data.to_vec();
                }
                unpremultiply_rgba_f32(&mut working_vec);
            }

            match color_space {
                ColorSpace::Gamma => {
                    // pass through
                }
                ColorSpace::Linear(trc) => {
                    if working_vec.is_empty() {
                        working_vec = source_data.to_vec();
                    }
                    image_f32_to_linear_f32::<4>(&mut working_vec, trc);
                    source_data = working_vec.as_slice();
                }
            }

            let mut result = resize_rgba_f32(
                source_data,
                source_size,
                destination_size,
                resampling_function,
            )?;
            working_vec.resize(0, 0.);

            match color_space {
                ColorSpace::Gamma => {
                    // pass through
                }
                ColorSpace::Linear(trc) => {
                    linear_f32_to_gamma_image_f32::<4>(&mut result, trc);
                }
            }

            if is_alpha_associated {
                premultiply_rgba_f32(&mut result);
            }

            let gray_image = match Rgba32FImage::from_raw(
                destination_size.width as u32,
                destination_size.height as u32,
                result,
            ) {
                None => {
                    return Err("Image resizing failed".to_string());
                }
                Some(val) => val,
            };

            Ok(DynamicImage::ImageRgba32F(gray_image))
        }
        _ => Err("This path is not implemented".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageFormat, ImageReader};

    #[test]
    fn test_rescaling() {
        let img = ImageReader::open(".././assets/nasa-4928x3279.png")
            .unwrap()
            .decode()
            .unwrap();

        let destination_size = ImageSize::new(img.width() as usize / 4, img.height() as usize / 4);

        // Luma 8
        let luma8 = img.to_luma8();
        let dyn_luma8 = DynamicImage::ImageLuma8(luma8);
        let l0 = resize_image(
            dyn_luma8.clone(),
            destination_size,
            ResamplingFunction::Bilinear,
            ColorSpace::Gamma,
            false,
        )
        .unwrap();
        l0.save_with_format("./plane_gamma.png", ImageFormat::Png)
            .unwrap();
        let l0 = resize_image(
            dyn_luma8.clone(),
            destination_size,
            ResamplingFunction::Bilinear,
            ColorSpace::Linear(TransferFunction::Srgb),
            false,
        )
        .unwrap();
        l0.save_with_format("./plane_linear.png", ImageFormat::Png)
            .unwrap();

        // Luma with alpha
        let luma_alpha_8 = img.to_luma_alpha8();
        let dyn_luma8 = DynamicImage::ImageLumaA8(luma_alpha_8);
        let l0 = resize_image(
            dyn_luma8.clone(),
            destination_size,
            ResamplingFunction::Bilinear,
            ColorSpace::Gamma,
            false,
        )
        .unwrap();
        l0.save_with_format("./plane_with_alpha.png", ImageFormat::Png)
            .unwrap();
        let l0 = resize_image(
            dyn_luma8.clone(),
            destination_size,
            ResamplingFunction::Bilinear,
            ColorSpace::Linear(TransferFunction::Srgb),
            false,
        )
        .unwrap();
        l0.save_with_format("./plane_with_alpha_linear.png", ImageFormat::Png)
            .unwrap();
        let l0 = resize_image(
            dyn_luma8.clone(),
            destination_size,
            ResamplingFunction::Bilinear,
            ColorSpace::Linear(TransferFunction::Srgb),
            true,
        )
        .unwrap();
        l0.save_with_format("./plane_with_alpha_linear_mul.png", ImageFormat::Png)
            .unwrap();

        // RGB
        let rgb8 = img.to_rgb8();
        let dyn_rgb8 = DynamicImage::ImageRgb8(rgb8);
        let l0 = resize_image(
            dyn_rgb8.clone(),
            destination_size,
            ResamplingFunction::Bilinear,
            ColorSpace::Gamma,
            false,
        )
        .unwrap();
        l0.save_with_format("./rgb_gamma.png", ImageFormat::Png)
            .unwrap();
        let l0 = resize_image(
            dyn_rgb8.clone(),
            destination_size,
            ResamplingFunction::Bilinear,
            ColorSpace::Linear(TransferFunction::Srgb),
            false,
        )
        .unwrap();
        l0.save_with_format("./rgb_linear.png", ImageFormat::Png)
            .unwrap();

        // RGBA
        let rgba_8 = img.to_rgba8();
        let dyn_rgba8 = DynamicImage::ImageRgba8(rgba_8);
        let l0 = resize_image(
            dyn_rgba8.clone(),
            destination_size,
            ResamplingFunction::Bilinear,
            ColorSpace::Gamma,
            false,
        )
        .unwrap();
        l0.save_with_format("./rgba_gamma.png", ImageFormat::Png)
            .unwrap();
        let l0 = resize_image(
            dyn_rgba8.clone(),
            destination_size,
            ResamplingFunction::Bilinear,
            ColorSpace::Linear(TransferFunction::Srgb),
            false,
        )
        .unwrap();
        l0.save_with_format("./rgba_linear.png", ImageFormat::Png)
            .unwrap();
        let l0 = resize_image(
            dyn_rgba8.clone(),
            destination_size,
            ResamplingFunction::Bilinear,
            ColorSpace::Linear(TransferFunction::Srgb),
            true,
        )
        .unwrap();
        l0.save_with_format("./rgba_linear_with_mul.png", ImageFormat::Png)
            .unwrap();

        // Luma 16
        let luma16 = img.to_luma16();
        let dyn_luma16 = DynamicImage::ImageLuma16(luma16);
        let l0 = resize_image(
            dyn_luma16.clone(),
            destination_size,
            ResamplingFunction::Bilinear,
            ColorSpace::Gamma,
            false,
        )
        .unwrap();
        l0.save_with_format("./plane16_gamma.png", ImageFormat::Png)
            .unwrap();
        let l0 = resize_image(
            dyn_luma16.clone(),
            destination_size,
            ResamplingFunction::Bilinear,
            ColorSpace::Linear(TransferFunction::Srgb),
            false,
        )
        .unwrap();
        l0.save_with_format("./plane16_linear.png", ImageFormat::Png)
            .unwrap();

        // Luma16 with alpha
        let luma_alpha_16 = img.to_luma_alpha16();
        let dyn_luma16 = DynamicImage::ImageLumaA16(luma_alpha_16);
        let l0 = resize_image(
            dyn_luma16.clone(),
            destination_size,
            ResamplingFunction::Bilinear,
            ColorSpace::Gamma,
            false,
        )
        .unwrap();
        l0.save_with_format("./plane16_with_alpha.png", ImageFormat::Png)
            .unwrap();
        let l0 = resize_image(
            dyn_luma16.clone(),
            destination_size,
            ResamplingFunction::Bilinear,
            ColorSpace::Linear(TransferFunction::Srgb),
            false,
        )
        .unwrap();
        l0.save_with_format("./plane16_with_alpha_linear.png", ImageFormat::Png)
            .unwrap();
        let l0 = resize_image(
            dyn_luma16.clone(),
            destination_size,
            ResamplingFunction::Bilinear,
            ColorSpace::Linear(TransferFunction::Srgb),
            true,
        )
        .unwrap();
        l0.save_with_format("./plane16_with_alpha_linear_mul.png", ImageFormat::Png)
            .unwrap();

        // RGB 16
        let rgb16 = img.to_rgb16();
        let dyn_rgb16 = DynamicImage::ImageRgb16(rgb16);
        let l0 = resize_image(
            dyn_rgb16.clone(),
            destination_size,
            ResamplingFunction::Bilinear,
            ColorSpace::Gamma,
            false,
        )
        .unwrap();
        l0.save_with_format("./rgb16_gamma.png", ImageFormat::Png)
            .unwrap();
        let l0 = resize_image(
            dyn_rgb16.clone(),
            destination_size,
            ResamplingFunction::Bilinear,
            ColorSpace::Linear(TransferFunction::Srgb),
            false,
        )
        .unwrap();
        l0.save_with_format("./rgb16_linear.png", ImageFormat::Png)
            .unwrap();

        // RGBA16
        let rgba_16 = img.to_rgba16();
        let dyn_rgba16 = DynamicImage::ImageRgba16(rgba_16);
        let l0 = resize_image(
            dyn_rgba16.clone(),
            destination_size,
            ResamplingFunction::Bilinear,
            ColorSpace::Gamma,
            false,
        )
        .unwrap();
        l0.save_with_format("./rgba16_gamma.png", ImageFormat::Png)
            .unwrap();
        let l0 = resize_image(
            dyn_rgba16.clone(),
            destination_size,
            ResamplingFunction::Bilinear,
            ColorSpace::Linear(TransferFunction::Srgb),
            false,
        )
        .unwrap();
        l0.save_with_format("./rgba16_linear.png", ImageFormat::Png)
            .unwrap();
        let l0 = resize_image(
            dyn_rgba16.clone(),
            destination_size,
            ResamplingFunction::Bilinear,
            ColorSpace::Linear(TransferFunction::Srgb),
            true,
        )
        .unwrap();
        l0.save_with_format("./rgba16_linear_with_mul.png", ImageFormat::Png)
            .unwrap();

        // RGB f32
        let rgbf32 = img.to_rgb32f();
        let dyn_rgb16 = DynamicImage::ImageRgb32F(rgbf32);
        let l0 = resize_image(
            dyn_rgb16.clone(),
            destination_size,
            ResamplingFunction::Bilinear,
            ColorSpace::Gamma,
            false,
        )
        .unwrap();
        let fallback_l0 = l0.to_rgb8();
        fallback_l0
            .save_with_format("./rgbf32_gamma.png", ImageFormat::Png)
            .unwrap();
        let l0 = resize_image(
            dyn_rgb16.clone(),
            destination_size,
            ResamplingFunction::Bilinear,
            ColorSpace::Linear(TransferFunction::Srgb),
            false,
        )
        .unwrap();
        let fallback_l0 = l0.to_rgb8();
        fallback_l0
            .save_with_format("./rgbf32_linear.png", ImageFormat::Png)
            .unwrap();

        // RGBA f32
        let rgba_f32 = img.to_rgba32f();
        let dyn_rgba_f32 = DynamicImage::ImageRgba32F(rgba_f32);
        let l0 = resize_image(
            dyn_rgba_f32.clone(),
            destination_size,
            ResamplingFunction::Bilinear,
            ColorSpace::Gamma,
            false,
        )
        .unwrap();
        let fallback_l0 = l0.to_rgba8();
        fallback_l0
            .save_with_format("./rgbaf32_gamma.png", ImageFormat::Png)
            .unwrap();
        let l0 = resize_image(
            dyn_rgba_f32.clone(),
            destination_size,
            ResamplingFunction::Bilinear,
            ColorSpace::Linear(TransferFunction::Srgb),
            false,
        )
        .unwrap();
        let fallback_l0 = l0.to_rgba8();
        fallback_l0
            .save_with_format("./rgbaf32_linear.png", ImageFormat::Png)
            .unwrap();
        let l0 = resize_image(
            dyn_rgba_f32.clone(),
            destination_size,
            ResamplingFunction::Bilinear,
            ColorSpace::Linear(TransferFunction::Srgb),
            true,
        )
        .unwrap();
        let fallback_l0 = l0.to_rgba8();
        fallback_l0
            .save_with_format("./rgbf32_linear_with_mul.png", ImageFormat::Png)
            .unwrap();
    }
}
