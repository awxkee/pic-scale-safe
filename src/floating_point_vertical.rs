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
use crate::color_group::ColorGroup;
use crate::filter_weights::FilterBounds;
use crate::mixed_storage::MixedStorage;
use num_traits::{AsPrimitive, Float, MulAdd};
use std::ops::{Add, Mul};

#[inline(always)]
/// # Generics
/// `T` - template buffer type
/// `J` - accumulator type
/// `F` - filter floating type
pub(crate) fn convolve_column_handler_floating_point_4<
    T: Copy + 'static + AsPrimitive<J> + Default,
    J: Copy
        + 'static
        + AsPrimitive<T>
        + MulAdd<J, Output = J>
        + Mul<J, Output = J>
        + Add<J, Output = J>
        + Default
        + MixedStorage<T>,
    F: Copy + 'static + AsPrimitive<J>,
    const CHANNELS: usize,
>(
    src: &[T],
    src_stride: usize,
    dst: &mut [T],
    filter: &[F],
    bounds: &FilterBounds,
    bit_depth: u32,
    x: usize,
) where
    i32: AsPrimitive<J>,
{
    let mut sums0 = ColorGroup::<CHANNELS, J>::dup(0.as_());
    let mut sums1 = ColorGroup::<CHANNELS, J>::dup(0.as_());
    let mut sums2 = ColorGroup::<CHANNELS, J>::dup(0.as_());
    let mut sums3 = ColorGroup::<CHANNELS, J>::dup(0.as_());

    let v_start_px = x * CHANNELS;

    for (j, &k_weight) in filter.iter().take(bounds.size).enumerate() {
        let py = bounds.start + j;
        let weight = k_weight.as_();
        let offset = src_stride * py + v_start_px;
        let src_ptr = &src[offset..(offset + CHANNELS * 4)];

        let new_px0 = ColorGroup::<CHANNELS, J>::from_slice(&src_ptr[0..CHANNELS]);
        let new_px1 = ColorGroup::<CHANNELS, J>::from_slice(&src_ptr[CHANNELS..CHANNELS * 2]);
        let new_px2 = ColorGroup::<CHANNELS, J>::from_slice(&src_ptr[CHANNELS * 2..CHANNELS * 3]);
        let new_px3 = ColorGroup::<CHANNELS, J>::from_slice(&src_ptr[CHANNELS * 3..CHANNELS * 4]);

        sums0 = sums0.mul_add(new_px0, weight);
        sums1 = sums1.mul_add(new_px1, weight);
        sums2 = sums2.mul_add(new_px2, weight);
        sums3 = sums3.mul_add(new_px3, weight);
    }

    let v_dst = &mut dst[v_start_px..(v_start_px + CHANNELS * 4)];

    sums0.to_mixed_store(v_dst, bit_depth);
    sums1.to_mixed_store(&mut v_dst[CHANNELS..CHANNELS * 2], bit_depth);
    sums2.to_mixed_store(&mut v_dst[CHANNELS * 2..CHANNELS * 3], bit_depth);
    sums3.to_mixed_store(&mut v_dst[CHANNELS * 3..CHANNELS * 4], bit_depth);
}

#[inline(always)]
/// # Generics
/// `T` - template buffer type
/// `J` - accumulator type
/// `F` - kernel floating type
pub(crate) fn convolve_column_handler_floating_point<
    T: Copy + 'static + AsPrimitive<J> + Default,
    J: Copy
        + 'static
        + AsPrimitive<T>
        + MulAdd<J, Output = J>
        + Mul<J, Output = J>
        + Add<J, Output = J>
        + MixedStorage<T>
        + Default,
    F: Copy + 'static + Float + AsPrimitive<J>,
    const CHANNELS: usize,
>(
    src: &[T],
    src_stride: usize,
    dst: &mut [T],
    filter: &[F],
    bounds: &FilterBounds,
    bit_depth: u32,
    x: usize,
) where
    i32: AsPrimitive<J>,
{
    let mut sums0 = ColorGroup::<CHANNELS, J>::dup(0.as_());

    let v_start_px = x * CHANNELS;

    for (j, &k_weight) in filter.iter().take(bounds.size).enumerate() {
        let py = bounds.start + j;
        let weight = k_weight.as_();
        let offset = src_stride * py + v_start_px;
        let src_ptr = &src[offset..(offset + CHANNELS)];

        let new_px0 = ColorGroup::<CHANNELS, J>::from_slice(&src_ptr[0..CHANNELS]);

        sums0 = sums0.mul_add(new_px0, weight);
    }

    sums0.to_mixed_store(&mut dst[v_start_px..(v_start_px + CHANNELS)], bit_depth);
}

#[inline(always)]
/// # Generics
/// `T` - template buffer type
/// `J` - accumulator type
pub(crate) fn column_handler_floating_point<
    T: Copy + 'static + AsPrimitive<J> + Default,
    J: Copy
        + 'static
        + AsPrimitive<T>
        + MulAdd<J, Output = J>
        + Mul<J, Output = J>
        + Add<J, Output = J>
        + MixedStorage<T>
        + Default,
    F: Copy + 'static + Float + AsPrimitive<J>,
    const COMPONENTS: usize,
>(
    dst_width: usize,
    bounds: &FilterBounds,
    src: &[T],
    dst: &mut [T],
    src_stride: usize,
    weight: &[F],
    bit_depth: u32,
) where
    i32: AsPrimitive<J>,
{
    let mut cx = 0usize;

    while cx + 4 < dst_width {
        convolve_column_handler_floating_point_4::<T, J, F, COMPONENTS>(
            src, src_stride, dst, weight, bounds, bit_depth, cx,
        );

        cx += 4;
    }

    while cx < dst_width {
        convolve_column_handler_floating_point::<T, J, F, COMPONENTS>(
            src, src_stride, dst, weight, bounds, bit_depth, cx,
        );

        cx += 1;
    }
}
