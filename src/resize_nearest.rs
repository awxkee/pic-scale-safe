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
#[cfg(feature = "rayon")]
use rayon::iter::{IndexedParallelIterator, ParallelIterator};
#[cfg(feature = "rayon")]
use rayon::prelude::ParallelSliceMut;

pub fn resize_nearest<T: Copy + Send + Sync, const CHANNELS: usize>(
    src: &[T],
    src_width: usize,
    src_height: usize,
    dst: &mut [T],
    dst_width: usize,
    dst_height: usize,
) {
    let x_scale = src_width as f32 / dst_width as f32;
    let y_scale = src_height as f32 / dst_height as f32;

    let clip_width = src_width as f32 - 1f32;
    let clip_height = src_height as f32 - 1f32;

    let dst_stride = dst_width * CHANNELS;
    let src_stride = src_width * CHANNELS;

    let iter;
    #[cfg(not(feature = "rayon"))]
    {
        iter = dst.chunks_exact_mut(dst_stride);
    }
    #[cfg(feature = "rayon")]
    {
        iter = dst.par_chunks_exact_mut(dst_stride);
    }

    iter.enumerate().for_each(|(y, dst_row)| {
        for (x, dst_chunk) in dst_row.chunks_exact_mut(CHANNELS).enumerate() {
            let src_x = ((x as f32 + 0.5f32) * x_scale - 0.5f32)
                .min(clip_width)
                .max(0f32) as usize;
            let src_y = ((y as f32 + 0.5f32) * y_scale - 0.5f32)
                .min(clip_height)
                .max(0f32) as usize;
            let src_offset_y = src_y * src_stride;
            let src_px = src_x * CHANNELS;
            let offset = src_offset_y + src_px;

            let src_slice = &src[offset..(offset + CHANNELS)];

            for (src, dst) in src_slice.iter().zip(dst_chunk.iter_mut()) {
                *dst = *src;
            }
        }
    });
}
