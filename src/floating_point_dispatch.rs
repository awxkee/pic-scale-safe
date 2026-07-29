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

use crate::filter_weights::FilterWeights;
use crate::handler_provider::{ColumnHandlerFloatingPoint, RowHandlerFloatingPoint};
use crate::image_size::ImageSize;
use crate::mixed_storage::MixedStorage;
#[cfg(feature = "rayon")]
use crate::scratch_pool::for_each_chunk_with_scratch;
use num_traits::{AsPrimitive, MulAdd};
#[cfg(feature = "rayon")]
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
#[cfg(feature = "rayon")]
use rayon::prelude::{ParallelSlice, ParallelSliceMut};

/// Runs the vertical kernel of `weights` for `scratch.len() / src_stride` rows, starting at
/// `start_row`, straight into `scratch`.
#[inline]
fn vertical_pass_into_scratch<T, J, F>(
    image_store: &[T],
    src_stride: usize,
    scratch: &mut [T],
    weights: &FilterWeights<F>,
    start_row: usize,
    bit_depth: u32,
) where
    T: Copy + 'static + AsPrimitive<J> + Default + ColumnHandlerFloatingPoint<T, J, F>,
    J: Copy + 'static + AsPrimitive<T> + MulAdd<J, Output = J> + Default + MixedStorage<T>,
    F: Copy + 'static + AsPrimitive<J>,
    i32: AsPrimitive<J>,
    f32: AsPrimitive<J>,
{
    for (i, scratch_row) in scratch.chunks_exact_mut(src_stride).enumerate() {
        let row = start_row + i;
        let offset = row * weights.aligned_size;
        T::handle_column(
            &weights.bounds[row],
            image_store,
            scratch_row,
            src_stride,
            &weights.weights[offset..(offset + weights.aligned_size)],
            bit_depth,
        );
    }
}

/// Convolves both axes in a single sweep over the destination.
///
/// Rather than materializing a full `source width * destination height` intermediate image,
/// the vertical pass fills a local scratch of at most 4 rows and the horizontal pass consumes
/// it immediately, so the working set stays in cache and the allocation stops scaling with
/// image height.
pub(crate) fn convolve_trampoline_floating_point<T, J, F, const CHANNELS: usize>(
    image_store: &[T],
    image_size: ImageSize,
    vertical_weights: FilterWeights<F>,
    horizontal_weights: FilterWeights<F>,
    destination: &mut [T],
    destination_size: ImageSize,
    bit_depth: u32,
) where
    T: Copy
        + 'static
        + AsPrimitive<J>
        + Default
        + ColumnHandlerFloatingPoint<T, J, F>
        + RowHandlerFloatingPoint<T, J, F>
        + Send
        + Sync,
    J: Copy + 'static + AsPrimitive<T> + MulAdd<J, Output = J> + Default + MixedStorage<T>,
    F: Copy + 'static + AsPrimitive<J> + Send + Sync,
    i32: AsPrimitive<J>,
    f32: AsPrimitive<J>,
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

    let scratch_len = src_stride * 4.min(destination_size.height);

    let quads = destination_size.height / 4;
    let (dst_quads, dst_rem) = destination.split_at_mut(quads * 4 * dst_stride);

    #[cfg(not(feature = "rayon"))]
    {
        let mut scratch = vec![T::default(); scratch_len];

        for (quad, dst) in dst_quads.chunks_exact_mut(dst_stride * 4).enumerate() {
            vertical_pass_into_scratch::<T, J, F>(
                image_store,
                src_stride,
                &mut scratch,
                &vertical_weights,
                quad * 4,
                bit_depth,
            );
            T::handle_row_4::<CHANNELS>(
                &scratch,
                src_stride,
                dst,
                dst_stride,
                &horizontal_weights,
                bit_depth,
            );
        }

        let scratch_row = &mut scratch[..src_stride];

        for (y, dst) in dst_rem.chunks_exact_mut(dst_stride).enumerate() {
            vertical_pass_into_scratch::<T, J, F>(
                image_store,
                src_stride,
                scratch_row,
                &vertical_weights,
                quads * 4 + y,
                bit_depth,
            );
            T::handle_row::<CHANNELS>(scratch_row, dst, &horizontal_weights, bit_depth);
        }
    }
    #[cfg(feature = "rayon")]
    {
        for_each_chunk_with_scratch(
            dst_quads,
            dst_stride * 4,
            scratch_len,
            |quad, dst, scratch| {
                vertical_pass_into_scratch::<T, J, F>(
                    image_store,
                    src_stride,
                    scratch,
                    &vertical_weights,
                    quad * 4,
                    bit_depth,
                );
                T::handle_row_4::<CHANNELS>(
                    scratch,
                    src_stride,
                    dst,
                    dst_stride,
                    &horizontal_weights,
                    bit_depth,
                );
            },
        );

        if !dst_rem.is_empty() {
            let mut scratch = vec![T::default(); src_stride];

            for (y, dst) in dst_rem.chunks_exact_mut(dst_stride).enumerate() {
                vertical_pass_into_scratch::<T, J, F>(
                    image_store,
                    src_stride,
                    &mut scratch,
                    &vertical_weights,
                    quads * 4 + y,
                    bit_depth,
                );
                T::handle_row::<CHANNELS>(&scratch, dst, &horizontal_weights, bit_depth);
            }
        }
    }
}

pub(crate) fn convolve_row_floating_point<T, J, F, const CHANNELS: usize>(
    image_store: &[T],
    image_size: ImageSize,
    weights: FilterWeights<F>,
    destination: &mut [T],
    destination_size: ImageSize,
    bit_depth: u32,
) where
    T: Copy + 'static + AsPrimitive<J> + Default + RowHandlerFloatingPoint<T, J, F> + Sync + Send,
    J: Copy + 'static + AsPrimitive<T> + MulAdd<J, Output = J> + Default + MixedStorage<T>,
    F: Copy + 'static + AsPrimitive<J> + Send + Sync,
    i32: AsPrimitive<J>,
    f32: AsPrimitive<J>,
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
    let src_stride_4 = src_stride * 4;

    let (dst_stride, k_overflowed) = destination_size.width.overflowing_mul(CHANNELS);
    assert!(!k_overflowed, "Stride must be always less than usize::MAX");
    let dst_stride_4 = dst_stride * 4;

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
                T::handle_row_4::<CHANNELS>(src, src_stride, dst, dst_stride, &weights, bit_depth);
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
}

pub(crate) fn convolve_column_floating_point<T, J, F, const CHANNELS: usize>(
    image_store: &[T],
    image_size: ImageSize,
    weights: FilterWeights<F>,
    destination: &mut [T],
    destination_size: ImageSize,
    bit_depth: u32,
) where
    T: Copy
        + 'static
        + AsPrimitive<J>
        + Default
        + ColumnHandlerFloatingPoint<T, J, F>
        + Send
        + Sync,
    J: Copy + 'static + AsPrimitive<T> + MulAdd<J, Output = J> + Default + MixedStorage<T>,
    F: Copy + 'static + AsPrimitive<J> + Send + Sync,
    i32: AsPrimitive<J>,
    f32: AsPrimitive<J>,
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

    #[cfg(feature = "rayon")]
    {
        let dst_store_iter = destination.par_chunks_exact_mut(dst_stride);
        dst_store_iter
            .zip(weights.bounds.par_iter())
            .zip(weights.weights.par_chunks_exact(weights.aligned_size))
            .for_each(|((dst, bounds), weights)| {
                T::handle_column(bounds, image_store, dst, src_stride, weights, bit_depth);
            });
    }
    #[cfg(not(feature = "rayon"))]
    {
        let dst_store_iter = destination.chunks_exact_mut(dst_stride);
        for ((dst, bounds), weights) in dst_store_iter
            .zip(weights.bounds)
            .zip(weights.weights.chunks_exact(weights.aligned_size))
        {
            T::handle_column(&bounds, image_store, dst, src_stride, weights, bit_depth);
        }
    }
}
