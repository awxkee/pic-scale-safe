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
use crate::definitions::ROUNDING_CONST;
use crate::filter_weights::FilterBounds;
use crate::saturate_narrow::SaturateNarrow;
use num_traits::AsPrimitive;
use std::ops::{AddAssign, Mul, Rem};

#[inline(always)]
/// # Generics
/// `T` - template buffer type
/// `J` - accumulator type
pub(crate) fn convolve_column_handler_fixed_point_4<
    T: Copy + 'static + AsPrimitive<J> + Default,
    J: Copy + 'static + AsPrimitive<T> + Mul<Output = J> + AddAssign + SaturateNarrow<T> + Default,
    const CHANNELS: usize,
>(
    src: &[T],
    src_stride: usize,
    dst: &mut [T],
    filter: &[i16],
    bounds: &FilterBounds,
    bit_depth: u32,
    x: usize,
) where
    i32: AsPrimitive<J>,
    i16: AsPrimitive<J>,
{
    let mut sums0 = ColorGroup::<CHANNELS, J>::dup(ROUNDING_CONST.as_());
    let mut sums1 = ColorGroup::<CHANNELS, J>::dup(ROUNDING_CONST.as_());
    let mut sums2 = ColorGroup::<CHANNELS, J>::dup(ROUNDING_CONST.as_());
    let mut sums3 = ColorGroup::<CHANNELS, J>::dup(ROUNDING_CONST.as_());

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

        sums0 += new_px0 * weight;
        sums1 += new_px1 * weight;
        sums2 += new_px2 * weight;
        sums3 += new_px3 * weight;
    }

    let narrow0 = sums0.saturate_narrow(bit_depth);
    let narrow1 = sums1.saturate_narrow(bit_depth);
    let narrow2 = sums2.saturate_narrow(bit_depth);
    let narrow3 = sums3.saturate_narrow(bit_depth);

    let v_dst = &mut dst[v_start_px..(v_start_px + CHANNELS * 4)];

    narrow0.to_store(v_dst);
    narrow1.to_store(&mut v_dst[CHANNELS..CHANNELS * 2]);
    narrow2.to_store(&mut v_dst[CHANNELS * 2..CHANNELS * 3]);
    narrow3.to_store(&mut v_dst[CHANNELS * 3..CHANNELS * 4]);
}

#[inline(always)]
/// # Generics
/// `T` - template buffer type
/// `J` - accumulator type
pub(crate) fn convolve_column_handler_fixed_point_6<
    T: Copy + 'static + AsPrimitive<J> + Default,
    J: Copy + 'static + AsPrimitive<T> + Mul<Output = J> + AddAssign + SaturateNarrow<T> + Default,
    const CHANNELS: usize,
>(
    src: &[T],
    src_stride: usize,
    dst: &mut [T],
    filter: &[i16],
    bounds: &FilterBounds,
    bit_depth: u32,
    x: usize,
) where
    i32: AsPrimitive<J>,
    i16: AsPrimitive<J>,
{
    let mut sums0 = ColorGroup::<CHANNELS, J>::dup(ROUNDING_CONST.as_());
    let mut sums1 = ColorGroup::<CHANNELS, J>::dup(ROUNDING_CONST.as_());
    let mut sums2 = ColorGroup::<CHANNELS, J>::dup(ROUNDING_CONST.as_());
    let mut sums3 = ColorGroup::<CHANNELS, J>::dup(ROUNDING_CONST.as_());
    let mut sums4 = ColorGroup::<CHANNELS, J>::dup(ROUNDING_CONST.as_());
    let mut sums5 = ColorGroup::<CHANNELS, J>::dup(ROUNDING_CONST.as_());

    let v_start_px = x * CHANNELS;

    for (j, &k_weight) in filter.iter().take(bounds.size).enumerate() {
        let py = bounds.start + j;
        let weight = k_weight.as_();
        let offset = src_stride * py + v_start_px;
        let src_ptr = &src[offset..(offset + CHANNELS * 6)];

        let new_px0 = ColorGroup::<CHANNELS, J>::from_slice(&src_ptr[0..CHANNELS]);
        let new_px1 = ColorGroup::<CHANNELS, J>::from_slice(&src_ptr[CHANNELS..CHANNELS * 2]);
        let new_px2 = ColorGroup::<CHANNELS, J>::from_slice(&src_ptr[CHANNELS * 2..CHANNELS * 3]);
        let new_px3 = ColorGroup::<CHANNELS, J>::from_slice(&src_ptr[CHANNELS * 3..CHANNELS * 4]);
        let new_px4 = ColorGroup::<CHANNELS, J>::from_slice(&src_ptr[CHANNELS * 4..CHANNELS * 5]);
        let new_px5 = ColorGroup::<CHANNELS, J>::from_slice(&src_ptr[CHANNELS * 5..CHANNELS * 6]);

        sums0 += new_px0 * weight;
        sums1 += new_px1 * weight;
        sums2 += new_px2 * weight;
        sums3 += new_px3 * weight;
        sums4 += new_px4 * weight;
        sums5 += new_px5 * weight;
    }

    let narrow0 = sums0.saturate_narrow(bit_depth);
    let narrow1 = sums1.saturate_narrow(bit_depth);
    let narrow2 = sums2.saturate_narrow(bit_depth);
    let narrow3 = sums3.saturate_narrow(bit_depth);
    let narrow4 = sums4.saturate_narrow(bit_depth);
    let narrow5 = sums5.saturate_narrow(bit_depth);

    let v_dst = &mut dst[v_start_px..(v_start_px + CHANNELS * 6)];

    narrow0.to_store(v_dst);
    narrow1.to_store(&mut v_dst[CHANNELS..CHANNELS * 2]);
    narrow2.to_store(&mut v_dst[CHANNELS * 2..CHANNELS * 3]);
    narrow3.to_store(&mut v_dst[CHANNELS * 3..CHANNELS * 4]);
    narrow4.to_store(&mut v_dst[CHANNELS * 4..CHANNELS * 5]);
    narrow5.to_store(&mut v_dst[CHANNELS * 5..CHANNELS * 6]);
}

#[inline(always)]
/// # Generics
/// `T` - template buffer type
/// `J` - accumulator type
pub(crate) fn convolve_column_handler_fixed_point_direct_buffer_4<
    T: Copy + 'static + AsPrimitive<J> + Default,
    J: Copy + 'static + AsPrimitive<T> + Mul<Output = J> + AddAssign + SaturateNarrow<T> + Default,
    const BUFFER_SIZE: usize,
>(
    src: &[T],
    src_stride: usize,
    dst: &mut [T],
    filter: &[i16],
    bounds: &FilterBounds,
    bit_depth: u32,
    x: usize,
) where
    i32: AsPrimitive<J>,
    i16: AsPrimitive<J>,
{
    if filter.is_empty() {
        return;
    }
    let mut direct_store = vec![J::default(); BUFFER_SIZE];

    let v_start_px = x;

    let py = bounds.start;
    let weight = filter[0].as_();
    let offset = src_stride * py + v_start_px;
    let src_ptr = &src[offset..(offset + BUFFER_SIZE)];

    for (dst, src) in direct_store.iter_mut().zip(src_ptr) {
        *dst += src.as_() * weight;
    }

    for (j, &k_weight) in filter.iter().take(bounds.size).skip(1).enumerate() {
        let py = bounds.start + j;
        let weight = k_weight.as_();
        let offset = src_stride * py + v_start_px;
        let src_ptr = &src[offset..(offset + BUFFER_SIZE)];

        for (dst, src) in direct_store.iter_mut().zip(src_ptr) {
            *dst += src.as_() * weight;
        }
    }

    let v_dst = &mut dst[v_start_px..(v_start_px + BUFFER_SIZE)];
    for (dst, src) in v_dst.iter_mut().zip(direct_store.iter()) {
        *dst = src.saturate_narrow(bit_depth);
    }
}

#[inline(always)]
/// # Generics
/// `T` - template buffer type
/// `J` - accumulator type
pub(crate) fn convolve_column_handler_fixed_point<
    T: Copy + 'static + AsPrimitive<J> + Default,
    J: Copy + 'static + AsPrimitive<T> + Mul<Output = J> + AddAssign + SaturateNarrow<T> + Default,
    const CHANNELS: usize,
>(
    src: &[T],
    src_stride: usize,
    dst: &mut [T],
    filter: &[i16],
    bounds: &FilterBounds,
    bit_depth: u32,
    x: usize,
) where
    i32: AsPrimitive<J>,
    i16: AsPrimitive<J>,
{
    let mut sums0 = ColorGroup::<CHANNELS, J>::dup(ROUNDING_CONST.as_());

    let v_start_px = x * CHANNELS;

    for (j, &k_weight) in filter.iter().take(bounds.size).enumerate() {
        let py = bounds.start + j;
        let weight = k_weight.as_();
        let offset = src_stride * py + v_start_px;
        let src_ptr = &src[offset..(offset + CHANNELS)];

        let new_px0 = ColorGroup::<CHANNELS, J>::from_slice(&src_ptr[0..CHANNELS]);

        sums0 += new_px0 * weight;
    }

    let narrow0 = sums0.saturate_narrow(bit_depth);
    narrow0.to_store(&mut dst[v_start_px..(v_start_px + CHANNELS)]);
}

/// # Generics
/// `T` - template buffer type
/// `J` - accumulator type
pub(crate) fn column_handler_fixed_point<
    T: Copy + 'static + AsPrimitive<J> + Default,
    J: Copy + 'static + AsPrimitive<T> + Mul<Output = J> + AddAssign + SaturateNarrow<T> + Default,
    const COMPONENTS: usize,
>(
    dst_width: usize,
    bounds: &FilterBounds,
    src: &[T],
    dst: &mut [T],
    src_stride: usize,
    weight: &[i16],
    bit_depth: u32,
) where
    i32: AsPrimitive<J>,
    i16: AsPrimitive<J>,
{
    let mut cx = 0usize;

    if COMPONENTS < 3 {
        let step64 = 64 / COMPONENTS;
        if 64.rem(COMPONENTS) == 0 {
            while cx + step64 < dst_width {
                convolve_column_handler_fixed_point_direct_buffer_4::<T, J, 64>(
                    src,
                    src_stride,
                    dst,
                    weight,
                    bounds,
                    bit_depth,
                    cx * COMPONENTS,
                );

                cx += step64;
            }
        }
        let step32 = 32 / COMPONENTS;
        if 32.rem(COMPONENTS) == 0 {
            while cx + step32 < dst_width {
                convolve_column_handler_fixed_point_direct_buffer_4::<T, J, 32>(
                    src,
                    src_stride,
                    dst,
                    weight,
                    bounds,
                    bit_depth,
                    cx * COMPONENTS,
                );

                cx += step32;
            }
        }
    }

    if COMPONENTS == 4 || COMPONENTS == 3 {
        while cx + 6 < dst_width {
            convolve_column_handler_fixed_point_6::<T, J, COMPONENTS>(
                src, src_stride, dst, weight, bounds, bit_depth, cx,
            );

            cx += 6;
        }
    }

    while cx + 4 < dst_width {
        convolve_column_handler_fixed_point_4::<T, J, COMPONENTS>(
            src, src_stride, dst, weight, bounds, bit_depth, cx,
        );

        cx += 4;
    }

    while cx < dst_width {
        convolve_column_handler_fixed_point::<T, J, COMPONENTS>(
            src, src_stride, dst, weight, bounds, bit_depth, cx,
        );

        cx += 1;
    }
}
