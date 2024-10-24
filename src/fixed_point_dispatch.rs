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
use crate::definitions::PRECISION;
use crate::filter_weights::FilterWeights;
use crate::handler_provider::{ColumnHandlerFixedPoint, RowHandlerFixedPoint};
use crate::image_size::ImageSize;
use crate::saturate_narrow::SaturateNarrow;
use num_traits::AsPrimitive;
#[cfg(feature = "rayon")]
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
#[cfg(feature = "rayon")]
use rayon::prelude::{ParallelSlice, ParallelSliceMut};
use std::ops::{AddAssign, Mul};

pub(crate) fn convolve_row_fixed_point<T, J, const CHANNELS: usize>(
    image_store: &[T],
    image_size: ImageSize,
    filter_weights: FilterWeights<f32>,
    destination: &mut [T],
    destination_size: ImageSize,
    bit_depth: u32,
) where
    T: Copy + 'static + AsPrimitive<J> + Default + RowHandlerFixedPoint<T, J> + Send + Sync,
    J: Copy + 'static + AsPrimitive<T> + Mul<Output = J> + AddAssign + SaturateNarrow<T> + Default,
    i32: AsPrimitive<J>,
    i16: AsPrimitive<J>,
{
    assert_eq!(
        image_store.len(),
        image_size.width * image_size.height * CHANNELS,
        "Source image slice must match its dimensions"
    );
    assert_eq!(
        destination.len(),
        destination_size.width * destination_size.height * CHANNELS,
        "Source image slice must match its dimensions"
    );

    let mut overflowed = false;

    let (src_stride, k_overflowed) = image_size.width.overflowing_mul(CHANNELS);
    assert!(!k_overflowed, "Stride must be always less than usize::MAX");
    let (src_stride_4, k_overflowed) = src_stride.overflowing_mul(4);
    if k_overflowed {
        overflowed = true;
    }

    let (dst_stride, k_overflowed) = destination_size.width.overflowing_mul(CHANNELS);
    assert!(!k_overflowed, "Stride must be always less than usize::MAX");
    let (dst_stride_4, k_overflowed) = dst_stride.overflowing_mul(4);
    if k_overflowed {
        overflowed = true;
    }

    let weights = filter_weights.numerical_approximation_i16::<PRECISION>(0);
    if !overflowed {
        #[cfg(not(feature = "rayon"))]
        {
            let image_store_4_iter = image_store.chunks_exact(src_stride_4);
            let dst_store_4_iter = destination.chunks_exact_mut(dst_stride_4);

            for (src, dst) in image_store_4_iter.zip(dst_store_4_iter) {
                T::handle_row_4::<CHANNELS>(src, src_stride, dst, dst_stride, &weights, bit_depth);
            }

            let image_store_iter_rem = image_store.chunks_exact(src_stride_4).remainder();
            let dst_store_iter_rem = destination.chunks_exact_mut(dst_stride_4).into_remainder();

            let image_store_iter = image_store_iter_rem.chunks_exact(src_stride);
            let dst_store_iter = dst_store_iter_rem.chunks_exact_mut(dst_stride);

            for (src, dst) in image_store_iter.zip(dst_store_iter) {
                T::handle_row::<CHANNELS>(src, dst, &weights, bit_depth);
            }
        }
        #[cfg(feature = "rayon")]
        {
            let image_store_4_iter = image_store.par_chunks_exact(src_stride_4);
            let dst_store_4_iter = destination.par_chunks_exact_mut(dst_stride_4);

            image_store_4_iter
                .zip(dst_store_4_iter)
                .for_each(|(src, dst)| {
                    T::handle_row_4::<CHANNELS>(
                        src, src_stride, dst, dst_stride, &weights, bit_depth,
                    );
                });

            let image_store_iter_rem = image_store.par_chunks_exact(src_stride_4).remainder();
            let dst_store_iter_rem = destination
                .par_chunks_exact_mut(dst_stride_4)
                .into_remainder();

            let image_store_iter = image_store_iter_rem.par_chunks_exact(src_stride);
            let dst_store_iter = dst_store_iter_rem.par_chunks_exact_mut(dst_stride);

            image_store_iter.zip(dst_store_iter).for_each(|(src, dst)| {
                T::handle_row::<CHANNELS>(src, dst, &weights, bit_depth);
            });
        }
    } else {
        #[cfg(feature = "rayon")]
        {
            let image_store_iter = image_store.par_chunks_exact(src_stride);
            let dst_store_iter = destination.par_chunks_exact_mut(dst_stride);

            image_store_iter.zip(dst_store_iter).for_each(|(src, dst)| {
                T::handle_row::<CHANNELS>(src, dst, &weights, bit_depth);
            });
        }
        #[cfg(not(feature = "rayon"))]
        {
            let image_store_iter = image_store.chunks_exact(src_stride);
            let dst_store_iter = destination.chunks_exact_mut(dst_stride);

            for (src, dst) in image_store_iter.zip(dst_store_iter) {
                T::handle_row::<CHANNELS>(src, dst, &weights, bit_depth);
            }
        }
    }
}

pub(crate) fn convolve_column_fixed_point<T, J, const CHANNELS: usize>(
    image_store: &[T],
    image_size: ImageSize,
    filter_weights: FilterWeights<f32>,
    destination: &mut [T],
    destination_size: ImageSize,
    bit_depth: u32,
) where
    T: Copy + 'static + AsPrimitive<J> + Default + ColumnHandlerFixedPoint<T, J> + Send + Sync,
    J: Copy + 'static + AsPrimitive<T> + Mul<Output = J> + AddAssign + SaturateNarrow<T> + Default,
    i32: AsPrimitive<J>,
    i16: AsPrimitive<J>,
{
    assert_eq!(
        image_store.len(),
        image_size.width * image_size.height * CHANNELS,
        "Source image slice must match its dimensions"
    );
    assert_eq!(
        destination.len(),
        destination_size.width * destination_size.height * CHANNELS,
        "Source image slice must match its dimensions"
    );

    let (src_stride, k_overflowed) = image_size.width.overflowing_mul(CHANNELS);
    assert!(!k_overflowed, "Stride must be always less than usize::MAX");
    let (dst_stride, k_overflowed) = destination_size.width.overflowing_mul(CHANNELS);
    assert!(!k_overflowed, "Stride must be always less than usize::MAX");

    let weights = filter_weights.numerical_approximation_i16::<PRECISION>(0);

    #[cfg(feature = "rayon")]
    {
        let dst_store_iter = destination.par_chunks_exact_mut(dst_stride);
        dst_store_iter
            .zip(weights.bounds.par_iter())
            .zip(weights.weights.par_chunks_exact(weights.aligned_size))
            .for_each(|((dst, bounds), weights)| {
                T::handle_column::<CHANNELS>(
                    destination_size.width,
                    bounds,
                    image_store,
                    dst,
                    src_stride,
                    weights,
                    bit_depth,
                );
            });
    }
    #[cfg(not(feature = "rayon"))]
    {
        let dst_store_iter = destination.chunks_exact_mut(dst_stride);
        for ((dst, bounds), weights) in dst_store_iter
            .zip(weights.bounds)
            .zip(weights.weights.chunks_exact(weights.aligned_size))
        {
            T::handle_column::<CHANNELS>(
                destination_size.width,
                &bounds,
                image_store,
                dst,
                src_stride,
                weights,
                bit_depth,
            );
        }
    }
}
