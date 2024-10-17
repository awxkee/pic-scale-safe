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
use crate::{resize_fixed_point, resize_floating_point, ImageSize, ResamplingFunction};

/// Performs resizing on RGBA 8 bit-depth image
///
/// To perform scaling on the image alpha must be unassociated first
/// use [unpremultiply_rgba8] before do scaling, and [premultiply_rgba8]
/// after.
///
/// Any content preferred to be in linear colorspace or perceptual before resizing,
/// consider using [linear_to_gamma_image] and [image_to_linear] if required,
/// otherwise results will degrade.
///
/// # Arguments
///
/// * `source`: Source image
/// * `source_size`: Source image size
/// * `destination_size`: Destination image size
/// * `resampling_function`: Resampling filter, see [ResamplingFunction] for more info
///
/// # Returns
///
/// Resized image, this bounds always match destination size
///
/// # Limitations
///
/// The contract `width * channels < usize::MAX` must be always satisfied and cannot be broken
///
/// This is using integral approximations, if more precise results are required use direct call
/// to [resize_floating_point::<u8, f32, f32, 4>]
///
pub fn resize_rgba8(
    source: &[u8],
    source_size: ImageSize,
    destination_size: ImageSize,
    resampling_function: ResamplingFunction,
) -> Result<Vec<u8>, String> {
    resize_fixed_point::<u8, i32, 4>(
        source,
        source_size,
        destination_size,
        8,
        resampling_function,
    )
}

/// Performs resizing on RGB 8 bit-depth image
///
/// Any content preferred to be in linear colorspace or perceptual before resizing,
/// consider using [linear_to_gamma_image] and [image_to_linear] if required,
/// otherwise results will degrade.
///
/// # Arguments
///
/// * `source`: Source image
/// * `source_size`: Source image size
/// * `destination_size`: Destination image size
/// * `resampling_function`: Resampling filter, see [ResamplingFunction] for more info
///
/// # Returns
///
/// Resized image, this bounds always match destination size
///
/// # Limitations
///
/// The contract `width * channels < usize::MAX` must be always satisfied and cannot be broken
///
/// This is using integral approximations, if more precise results are required use direct call
/// to [resize_floating_point::<u8, f32, f32, 3>]
///
pub fn resize_rgb8(
    source: &[u8],
    source_size: ImageSize,
    destination_size: ImageSize,
    resampling_function: ResamplingFunction,
) -> Result<Vec<u8>, String> {
    resize_fixed_point::<u8, i32, 3>(
        source,
        source_size,
        destination_size,
        8,
        resampling_function,
    )
}

/// Performs resizing on planar 8 bit-depth image
///
/// Any content preferred to be in linear colorspace or perceptual before resizing,
/// consider using [linear_to_gamma_image] and [image_to_linear] if required,
/// otherwise results will degrade.
///
/// # Arguments
///
/// * `source`: Source image
/// * `source_size`: Source image size
/// * `destination_size`: Destination image size
/// * `resampling_function`: Resampling filter, see [ResamplingFunction] for more info
///
/// # Returns
///
/// Resized image, this bounds always match destination size
///
/// # Limitations
///
/// The contract `width * channels < usize::MAX` must be always satisfied and cannot be broken
///
/// This is using integral approximations, if more precise results are required use direct call
/// to [resize_floating_point::<u8, f32, f32, 1>]
///
pub fn resize_plane8(
    source: &[u8],
    source_size: ImageSize,
    destination_size: ImageSize,
    resampling_function: ResamplingFunction,
) -> Result<Vec<u8>, String> {
    resize_fixed_point::<u8, i32, 1>(
        source,
        source_size,
        destination_size,
        8,
        resampling_function,
    )
}

/// Performs resizing on planar image with alpha 8 bit-depth image
///
/// Any content preferred to be in linear colorspace or perceptual before resizing,
/// consider using [linear_to_gamma_image] and [image_to_linear] if required,
/// otherwise results will degrade.
///
/// # Arguments
///
/// * `source`: Source image
/// * `source_size`: Source image size
/// * `destination_size`: Destination image size
/// * `resampling_function`: Resampling filter, see [ResamplingFunction] for more info
///
/// # Returns
///
/// Resized image, this bounds always match destination size
///
/// # Limitations
///
/// The contract `width * channels < usize::MAX` must be always satisfied and cannot be broken
///
/// This is using integral approximations, if more precise results are required use direct call
/// to [resize_floating_point::<u8, f32, f32, 2>]
///
pub fn resize_plane8_with_alpha(
    source: &[u8],
    source_size: ImageSize,
    destination_size: ImageSize,
    resampling_function: ResamplingFunction,
) -> Result<Vec<u8>, String> {
    resize_fixed_point::<u8, i32, 2>(
        source,
        source_size,
        destination_size,
        8,
        resampling_function,
    )
}

/// Performs resizing on RGBA 8-16 bit-depth image
///
/// To perform scaling on the image alpha must be unassociated first
/// use [unpremultiply_rgba16] before do scaling, and [premultiply_rgba16]
/// after.
///
/// Any content preferred to be in linear colorspace or perceptual before resizing,
/// consider using [linear16_to_gamma_image16] and [image16_to_linear16] if required,
/// otherwise results will degrade.
///
/// # Arguments
///
/// * `source`: Source image
/// * `source_size`: Source image size
/// * `destination_size`: Destination image size
/// * `bit_depth`: Bit-depth of the image
/// * `resampling_function`: Resampling filter, see [ResamplingFunction] for more info
///
/// # Returns
///
/// Resized image, this bounds always match destination size
///
/// # Limitations
///
/// The contract `width * channels < usize::MAX` must be always satisfied and cannot be broken
///
pub fn resize_rgba16(
    source: &[u16],
    source_size: ImageSize,
    destination_size: ImageSize,
    bit_depth: u32,
    resampling_function: ResamplingFunction,
) -> Result<Vec<u16>, String> {
    if bit_depth > 16 {
        return Err("Bit depth cannot be greater than 16".parse().unwrap());
    }
    resize_floating_point::<u16, f32, f32, 4>(
        source,
        source_size,
        destination_size,
        bit_depth,
        resampling_function,
    )
}

/// Performs resizing on RGB 8-16 bit-depth image
///
/// Any content preferred to be in linear colorspace or perceptual before resizing,
/// consider using [linear16_to_gamma_image16] and [image16_to_linear16] if required,
/// otherwise results will degrade.
///
/// # Arguments
///
/// * `source`: Source image
/// * `source_size`: Source image size
/// * `destination_size`: Destination image size
/// * `bit_depth`: Bit-depth of the image
/// * `resampling_function`: Resampling filter, see [ResamplingFunction] for more info
///
/// # Returns
///
/// Resized image, this bounds always match destination size
///
/// # Limitations
///
/// The contract `width * channels < usize::MAX` must be always satisfied and cannot be broken
///
pub fn resize_rgb16(
    source: &[u16],
    source_size: ImageSize,
    destination_size: ImageSize,
    bit_depth: u32,
    resampling_function: ResamplingFunction,
) -> Result<Vec<u16>, String> {
    if bit_depth > 16 {
        return Err("Bit depth cannot be greater than 16".parse().unwrap());
    }
    resize_floating_point::<u16, f32, f32, 3>(
        source,
        source_size,
        destination_size,
        bit_depth,
        resampling_function,
    )
}

/// Performs resizing on planar 8-16 bit-depth image
///
/// Any content preferred to be in linear colorspace or perceptual before resizing,
/// consider using [linear16_to_gamma_image16] and [image16_to_linear16] if required,
/// otherwise results will degrade.
///
/// # Arguments
///
/// * `source`: Source image
/// * `source_size`: Source image size
/// * `destination_size`: Destination image size
/// * `bit_depth`: Bit-depth of the image
/// * `resampling_function`: Resampling filter, see [ResamplingFunction] for more info
///
/// # Returns
///
/// Resized image, this bounds always match destination size
///
/// # Limitations
///
/// The contract `width * channels < usize::MAX` must be always satisfied and cannot be broken
///
pub fn resize_plane16(
    source: &[u16],
    source_size: ImageSize,
    destination_size: ImageSize,
    bit_depth: u32,
    resampling_function: ResamplingFunction,
) -> Result<Vec<u16>, String> {
    if bit_depth > 16 {
        return Err("Bit depth cannot be greater than 16".parse().unwrap());
    }
    resize_floating_point::<u16, f32, f32, 1>(
        source,
        source_size,
        destination_size,
        bit_depth,
        resampling_function,
    )
}

/// Performs resizing on RGBA f32 image
///
/// To perform scaling on the image alpha must be unassociated first
/// use [unpremultiply_rgba_f32] before do scaling, and [premultiply_rgba_f32]
/// after.
///
/// Any content preferred to be in linear colorspace or perceptual before resizing,
/// consider using [linear_f32_to_gamma_image_f32] and [image_f32_to_linear_f32] if required,
/// otherwise results will degrade.
///
/// # Arguments
///
/// * `source`: Source image
/// * `source_size`: Source image size
/// * `destination_size`: Destination image size
/// * `resampling_function`: Resampling filter, see [ResamplingFunction] for more info
///
/// # Returns
///
/// Resized image, this bounds always match destination size
///
/// # Limitations
///
/// The contract `width * channels < usize::MAX` must be always satisfied and cannot be broken
///
pub fn resize_rgba_f32(
    source: &[f32],
    source_size: ImageSize,
    destination_size: ImageSize,
    resampling_function: ResamplingFunction,
) -> Result<Vec<f32>, String> {
    resize_floating_point::<f32, f32, f32, 4>(
        source,
        source_size,
        destination_size,
        8,
        resampling_function,
    )
}

/// Performs resizing on RGB f32 image
///
/// Any content preferred to be in linear colorspace or perceptual before resizing,
/// consider using [linear_f32_to_gamma_image_f32] and [image_f32_to_linear_f32] if required,
/// otherwise results will degrade.
///
/// # Arguments
///
/// * `source`: Source image
/// * `source_size`: Source image size
/// * `destination_size`: Destination image size
/// * `resampling_function`: Resampling filter, see [ResamplingFunction] for more info
///
/// # Returns
///
/// Resized image, this bounds always match destination size
///
/// # Limitations
///
/// The contract `width * channels < usize::MAX` must be always satisfied and cannot be broken
///
pub fn resize_rgb_f32(
    source: &[f32],
    source_size: ImageSize,
    destination_size: ImageSize,
    resampling_function: ResamplingFunction,
) -> Result<Vec<f32>, String> {
    resize_floating_point::<f32, f32, f32, 3>(
        source,
        source_size,
        destination_size,
        8,
        resampling_function,
    )
}

/// Performs resizing on RGB f32 image
///
/// Any content preferred to be in linear colorspace or perceptual before resizing,
/// consider using [linear_f32_to_gamma_image_f32] and [image_f32_to_linear_f32] if required,
/// otherwise results will degrade.
///
/// # Arguments
///
/// * `source`: Source image
/// * `source_size`: Source image size
/// * `destination_size`: Destination image size
/// * `resampling_function`: Resampling filter, see [ResamplingFunction] for more info
///
/// # Returns
///
/// Resized image, this bounds always match destination size
///
/// # Limitations
///
/// The contract `width * channels < usize::MAX` must be always satisfied and cannot be broken
///
pub fn resize_plane_f32(
    source: &[f32],
    source_size: ImageSize,
    destination_size: ImageSize,
    resampling_function: ResamplingFunction,
) -> Result<Vec<f32>, String> {
    resize_floating_point::<f32, f32, f32, 1>(
        source,
        source_size,
        destination_size,
        8,
        resampling_function,
    )
}
