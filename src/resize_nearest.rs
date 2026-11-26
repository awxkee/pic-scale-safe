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

use crate::image_size::FractIndex;

pub(crate) fn resize_nearest<T: Copy + Send + Sync, const CHANNELS: usize>(
    src: &[T],
    src_x_start: FractIndex,
    src_x_end: FractIndex,
    src_width: usize,
    src_y_start: FractIndex,
    src_y_end: FractIndex,
    src_height: usize,
    dst: &mut [T],
    dst_width: usize,
    dst_height: usize,
) {
    const SHIFT: i32 = 32;

    assert!(src_x_start <= src_x_end && src_x_end <= FractIndex::from(src_width));
    assert!(src_y_start <= src_y_end && src_y_end <= FractIndex::from(src_height));

    let b_x = ((src_x_start.int as u64) << SHIFT) + src_x_start.frac as u64;
    let b_y = ((src_y_start.int as u64) << SHIFT) + src_y_start.frac as u64;

    let l_x = (((src_x_end.int - src_x_start.int) as u64) << SHIFT) + src_x_end.frac as u64
        - src_x_start.frac as u64;
    let l_y = (((src_y_end.int - src_y_start.int) as u64) << SHIFT) + src_y_end.frac as u64
        - src_y_start.frac as u64;

    let k_x: u64 = l_x / dst_width as u64;
    let k_y: u64 = l_y / dst_height as u64;
    let k_x_half: u64 = k_x >> 1;
    let k_y_half: u64 = k_y >> 1;

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
        let src_y = ((b_y + y as u64 * k_y + k_y_half) >> SHIFT) as usize;
        let src_offset_y = src_y * src_stride;

        let mut src_x_fixed = b_x + k_x_half;
        for dst_chunk in dst_row.chunks_exact_mut(CHANNELS) {
            let src_x = (src_x_fixed >> SHIFT) as usize;

            let src_px = src_x * CHANNELS;
            let offset = src_offset_y + src_px;

            let src_slice = &src[offset..(offset + CHANNELS)];

            for (src, dst) in src_slice.iter().zip(dst_chunk.iter_mut()) {
                *dst = *src;
            }

            src_x_fixed += k_x;
        }
    });
}
