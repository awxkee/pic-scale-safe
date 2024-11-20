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
use num_traits::AsPrimitive;
use std::ops::{AddAssign, BitXor};

/// Performs scan on the RGBA 8 bit image if it has variable alpha channel
///
/// # Arguments
///
/// * `store`: Slice with rgba data
/// * `width`: image width
///
/// returns: bool
///
pub fn has_non_constant_alpha_rgba8(store: &[u8], width: usize) -> bool {
    has_non_constant_alpha::<u8, u32, 3, 4>(store, width)
}

/// Performs scan on the Luma+Alpha 8 bit image if it has variable alpha channel
///
/// # Arguments
///
/// * `store`: Slice with luma with alpha data
/// * `width`: image width
///
/// returns: bool
///
pub fn has_non_constant_alpha_la8(store: &[u8], width: usize) -> bool {
    has_non_constant_alpha::<u8, u32, 1, 2>(store, width)
}

/// Performs scan on the RGBA 16 bit image if it has variable alpha channel
///
/// # Arguments
///
/// * `store`: Slice with rgba16 data
/// * `width`: image width
///
/// returns: bool
///
pub fn has_non_constant_alpha_rgba16(store: &[u16], width: usize) -> bool {
    has_non_constant_alpha::<u16, u64, 3, 4>(store, width)
}

/// Performs scan on the Luma+Alpha 16 bit image if it has variable alpha channel
///
/// # Arguments
///
/// * `store`: Slice with luma with alpha 16-bit data
/// * `width`: image width
///
/// returns: bool
///
pub fn has_non_constant_alpha_la16(store: &[u16], width: usize) -> bool {
    has_non_constant_alpha::<u16, u64, 1, 2>(store, width)
}

/// Performs scan on the RGBA f32 bit image if it has variable alpha channel
///
/// # Arguments
///
/// * `store`: Slice with rgba f32 data
/// * `width`: image width
///
/// returns: bool
///
pub fn has_non_constant_alpha_rgba_f32(store: &[f32], width: usize) -> bool {
    has_non_constant_alpha_f32_impl::<3, 4>(store, width)
}

/// Performs scan on the Luma-alpha f32 bit image if it has variable alpha channel
///
/// # Arguments
///
/// * `store`: Slice with Luma alpha f32 data
/// * `width`: image width
///
/// returns: bool
///
pub fn has_non_constant_alpha_luma_alpha_f32(store: &[f32], width: usize) -> bool {
    has_non_constant_alpha_f32_impl::<1, 2>(store, width)
}

#[inline]
fn has_non_constant_alpha<
    V: Copy + PartialEq + BitXor<V, Output = V> + 'static + AsPrimitive<J> + 'static,
    J: Copy + AddAssign + Default + 'static + Eq + Ord,
    const ALPHA_CHANNEL_INDEX: usize,
    const CHANNELS: usize,
>(
    store: &[V],
    width: usize,
) -> bool
where
    i32: AsPrimitive<V>,
    u32: AsPrimitive<V> + AsPrimitive<J>,
{
    assert!(ALPHA_CHANNEL_INDEX < CHANNELS);
    assert!(CHANNELS > 0 && CHANNELS <= 4);
    if store.is_empty() {
        return false;
    }
    let first = store[0];
    let mut row_sums: J = 0u32.as_();
    for row in store.chunks_exact(width * CHANNELS) {
        for color in row.chunks_exact(CHANNELS) {
            row_sums += color[ALPHA_CHANNEL_INDEX].bitxor(first).as_();
        }
        if row_sums != 0.as_() {
            return true;
        }
    }

    let zeros = 0.as_();

    row_sums.ne(&zeros)
}

#[inline]
fn has_non_constant_alpha_f32_impl<const ALPHA_CHANNEL_INDEX: usize, const CHANNELS: usize>(
    store: &[f32],
    width: usize,
) -> bool {
    assert!(ALPHA_CHANNEL_INDEX < CHANNELS);
    assert!(CHANNELS > 0 && CHANNELS <= 4);
    if store.is_empty() {
        return false;
    }
    let first = store[0].to_bits();
    let mut row_sums: u64 = 0u64;
    for row in store.chunks_exact(width * CHANNELS) {
        for color in row.chunks_exact(CHANNELS) {
            row_sums += color[ALPHA_CHANNEL_INDEX].to_bits().bitxor(first) as u64;
        }
        if row_sums != 0 {
            return true;
        }
    }

    let zeros = 0;

    row_sums.ne(&zeros)
}
