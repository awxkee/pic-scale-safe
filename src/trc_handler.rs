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
use crate::TransferFunction;
#[cfg(feature = "rayon")]
use rayon::iter::ParallelIterator;
#[cfg(feature = "rayon")]
use rayon::prelude::ParallelSliceMut;

/// Converts 8-bit image to linear
///
/// On `CHANNELS` == 2 or `CHANNELS` == 4 alpha will be considered as last item
///
/// # Arguments
///
/// * `in_place`: Where convert to
/// * `trc` - Transfer function, see [TransferFunction] for more info
///
pub fn image_to_linear<const CHANNELS: usize>(in_place: &mut [u8], trc: TransferFunction) {
    assert!(CHANNELS != 0 && CHANNELS <= 4, "Channels must be 1..=4");
    let mut lut_table = [0u8; 256];
    for (i, item) in lut_table.iter_mut().enumerate() {
        *item = (trc.linearize(i as f32 * (1. / 255.0)) * 255.).min(255.) as u8;
    }
    let iter;
    #[cfg(feature = "rayon")]
    {
        iter = in_place.par_chunks_exact_mut(CHANNELS);
    }
    #[cfg(not(feature = "rayon"))]
    {
        iter = in_place.chunks_exact_mut(CHANNELS);
    }
    iter.for_each(|dst| {
        if CHANNELS == 1 || CHANNELS == 2 {
            dst[0] = lut_table[dst[0] as usize];
        } else if CHANNELS == 3 || CHANNELS == 4 {
            dst[0] = lut_table[dst[0] as usize];
            dst[1] = lut_table[dst[1] as usize];
            dst[2] = lut_table[dst[2] as usize];
        }
    });
}

/// Converts 8-bit linear image to gamma
///
/// On `CHANNELS` == 2 or `CHANNELS` == 4 alpha will be considered as last item
///
/// # Arguments
///
/// * `in_place`: Where convert to
/// * `trc` - Transfer function, see [TransferFunction] for more info
///
pub fn linear_to_gamma_image<const CHANNELS: usize>(in_place: &mut [u8], trc: TransferFunction) {
    assert!(CHANNELS != 0 && CHANNELS <= 4, "Channels must be 1..=4");
    let mut lut_table = [0u8; 256];
    for (i, item) in lut_table.iter_mut().enumerate() {
        *item = (trc.gamma(i as f32 * (1. / 255.0)) * 255.).min(255.) as u8;
    }
    let iter;
    #[cfg(feature = "rayon")]
    {
        iter = in_place.par_chunks_exact_mut(CHANNELS);
    }
    #[cfg(not(feature = "rayon"))]
    {
        iter = in_place.chunks_exact_mut(CHANNELS);
    }
    iter.for_each(|dst| {
        if CHANNELS == 1 || CHANNELS == 2 {
            dst[0] = lut_table[dst[0] as usize];
        } else if CHANNELS == 3 || CHANNELS == 4 {
            dst[0] = lut_table[dst[0] as usize];
            dst[1] = lut_table[dst[1] as usize];
            dst[2] = lut_table[dst[2] as usize];
        }
    });
}

/// Converts 8-16-bit image to linear
///
/// On `CHANNELS` == 2 or `CHANNELS` == 4 alpha will be considered as last item
///
/// # Arguments
///
/// * `in_place`: Where convert to
/// * `trc` - Transfer function, see [TransferFunction] for more info
///
pub fn image16_to_linear16<const CHANNELS: usize>(
    in_place: &mut [u16],
    bit_depth: u32,
    trc: TransferFunction,
) {
    assert!(CHANNELS != 0 && CHANNELS <= 4, "Channels must be 1..=4");
    assert!((1..=16).contains(&bit_depth), "Bit depth must be 1..=16");
    let max_colors = (1 << bit_depth) - 1;
    let mut lut_table = vec![0u16; max_colors + 1];
    for (i, item) in lut_table.iter_mut().enumerate() {
        *item = (trc.linearize(i as f32 * (1. / max_colors as f32)) * max_colors as f32)
            .min(max_colors as f32) as u16;
    }
    let iter;
    #[cfg(feature = "rayon")]
    {
        iter = in_place.par_chunks_exact_mut(CHANNELS);
    }
    #[cfg(not(feature = "rayon"))]
    {
        iter = in_place.chunks_exact_mut(CHANNELS);
    }
    iter.for_each(|dst| {
        if CHANNELS == 1 || CHANNELS == 2 {
            dst[0] = lut_table[dst[0] as usize];
        } else if CHANNELS == 3 || CHANNELS == 4 {
            dst[0] = lut_table[dst[0] as usize];
            dst[1] = lut_table[dst[1] as usize];
            dst[2] = lut_table[dst[2] as usize];
        }
    });
}

/// Converts 8-16-bit linear image to gamma
///
/// On `CHANNELS` == 2 or `CHANNELS` == 4 alpha will be considered as last item
///
/// # Arguments
///
/// * `in_place`: Where convert to
/// * `trc` - Transfer function, see [TransferFunction] for more info
///
pub fn linear16_to_gamma_image16<const CHANNELS: usize>(
    in_place: &mut [u16],
    bit_depth: u32,
    trc: TransferFunction,
) {
    assert!(CHANNELS != 0 && CHANNELS <= 4, "Channels must be 1..=4");
    assert!((1..=16).contains(&bit_depth), "Bit depth must be 1..=16");
    let max_colors = (1 << bit_depth) - 1;
    let mut lut_table = vec![0u16; max_colors + 1];
    for (i, item) in lut_table.iter_mut().enumerate() {
        *item = (trc.gamma(i as f32 * (1. / max_colors as f32)) * max_colors as f32)
            .min(max_colors as f32) as u16;
    }
    let iter;
    #[cfg(feature = "rayon")]
    {
        iter = in_place.par_chunks_exact_mut(CHANNELS);
    }
    #[cfg(not(feature = "rayon"))]
    {
        iter = in_place.chunks_exact_mut(CHANNELS);
    }
    iter.for_each(|dst| {
        if CHANNELS == 1 || CHANNELS == 2 {
            dst[0] = lut_table[dst[0] as usize];
        } else if CHANNELS == 3 || CHANNELS == 4 {
            dst[0] = lut_table[dst[0] as usize];
            dst[1] = lut_table[dst[1] as usize];
            dst[2] = lut_table[dst[2] as usize];
        }
    });
}

/// Converts `f32` image to linear
///
/// On `CHANNELS` == 2 or `CHANNELS` == 4 alpha will be considered as last item
///
/// # Arguments
///
/// * `in_place`: Where convert to
/// * `trc` - Transfer function, see [TransferFunction] for more info
///
pub fn image_f32_to_linear_f32<const CHANNELS: usize>(in_place: &mut [f32], trc: TransferFunction) {
    assert!(CHANNELS != 0 && CHANNELS <= 4, "Channels must be 1..=4");
    let iter;
    #[cfg(feature = "rayon")]
    {
        iter = in_place.par_chunks_exact_mut(CHANNELS);
    }
    #[cfg(not(feature = "rayon"))]
    {
        iter = in_place.chunks_exact_mut(CHANNELS);
    }
    iter.for_each(|dst| {
        if CHANNELS == 1 || CHANNELS == 2 {
            dst[0] = trc.linearize(dst[0]);
        } else if CHANNELS == 3 || CHANNELS == 4 {
            dst[0] = trc.linearize(dst[0]);
            dst[1] = trc.linearize(dst[1]);
            dst[2] = trc.linearize(dst[2]);
        }
    });
}

/// Converts `f32` linear image to gamma
///
/// On `CHANNELS` == 2 or `CHANNELS` == 4 alpha will be considered as last item
///
/// # Arguments
///
/// * `in_place`: Where convert to
/// * `trc` - Transfer function, see [TransferFunction] for more info
///
pub fn linear_f32_to_gamma_image_f32<const CHANNELS: usize>(
    in_place: &mut [f32],
    trc: TransferFunction,
) {
    assert!(CHANNELS != 0 && CHANNELS <= 4, "Channels must be 1..=4");
    let iter;
    #[cfg(feature = "rayon")]
    {
        iter = in_place.par_chunks_exact_mut(CHANNELS);
    }
    #[cfg(not(feature = "rayon"))]
    {
        iter = in_place.chunks_exact_mut(CHANNELS);
    }
    iter.for_each(|dst| {
        if CHANNELS == 1 || CHANNELS == 2 {
            dst[0] = trc.gamma(dst[0]);
        } else if CHANNELS == 3 || CHANNELS == 4 {
            dst[0] = trc.gamma(dst[0]);
            dst[1] = trc.gamma(dst[1]);
            dst[2] = trc.gamma(dst[2]);
        }
    });
}
