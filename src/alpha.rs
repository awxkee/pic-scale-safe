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

#[inline]
fn div_by_255(v: u16) -> u8 {
    ((((v + 0x80) >> 8) + v + 0x80) >> 8).min(255) as u8
}

/// Associate alpha in place
///
/// Note, for scaling alpha must be *associated*
///
/// # Arguments
///
/// * `in_place`: Slice to where premultiply
///
pub fn premultiply_rgba8(in_place: &mut [u8]) {
    // Almost all loops are not auto-vectorised without doing anything dirty.
    // So everywhere is just added something beautiful.
    for chunk in in_place.chunks_exact_mut(4) {
        let a = chunk[3] as u16;
        chunk[0] = div_by_255(chunk[0] as u16 * a);
        chunk[1] = div_by_255(chunk[1] as u16 * a);
        chunk[2] = div_by_255(chunk[2] as u16 * a);
        chunk[3] = div_by_255(a * a);
    }
}

/// Un premultiply alpha in place
///
/// Note, for scaling alpha must be *associated*
///
/// # Arguments
///
/// * `in_place`: Slice to work on
///
///
pub fn unpremultiply_rgba8(in_place: &mut [u8]) {
    // Almost all loops are not auto-vectorised without doing anything dirty.
    // So everywhere is just added something beautiful.
    for chunk in in_place.chunks_exact_mut(4) {
        let a = chunk[3];
        if a != 0 {
            let a_recip = 1. / a as f32;
            chunk[0] = ((chunk[0] as f32 * 255.) * a_recip) as u8;
            chunk[1] = ((chunk[1] as f32 * 255.) * a_recip) as u8;
            chunk[2] = ((chunk[2] as f32 * 255.) * a_recip) as u8;
            chunk[3] = ((a as f32 * 255.) * a_recip) as u8;
        }
    }
}

/// Associate alpha in place
///
/// Note, for scaling alpha must be *associated*
///
/// # Arguments
///
/// * `in_place`: Slice to where premultiply
///
pub fn premultiply_la8(in_place: &mut [u8]) {
    // Almost all loops are not auto-vectorised without doing anything dirty.
    // So everywhere is just added something beautiful.
    for chunk in in_place.chunks_exact_mut(2) {
        let a = chunk[1] as u16;
        chunk[0] = div_by_255(chunk[0] as u16 * a);
        chunk[1] = div_by_255(chunk[1] as u16 * a);
    }
}

/// Un premultiply alpha in place
///
/// Note, for scaling alpha must be *associated*
///
/// # Arguments
///
/// * `in_place`: Slice to work on
///
///
pub fn unpremultiply_la8(in_place: &mut [u8]) {
    // Almost all loops are not auto-vectorised without doing anything dirty.
    // So everywhere is just added something beautiful.
    for chunk in in_place.chunks_exact_mut(2) {
        let a = chunk[1];
        if a != 0 {
            let a_recip = 1. / a as f32;
            chunk[0] = ((chunk[0] as f32 * 255.) * a_recip) as u8;
            chunk[1] = ((a as f32 * 255.) * a_recip) as u8;
        }
    }
}

/// Associate alpha in place
///
/// Note, for scaling alpha must be *associated*
///
/// # Arguments
///
/// * `in_place`: Slice to where premultiply
/// * `bit_depth`: Bit-depth of the image
///
pub fn premultiply_rgba16(in_place: &mut [u16], bit_depth: u32) {
    // Almost all loops are not auto-vectorised without doing anything dirty.
    // So everywhere is just added something beautiful.
    assert!(bit_depth > 0 && bit_depth <= 16);
    let max_colors = (1 << bit_depth) - 1;
    let recip_max_colors = 1. / max_colors as f32;
    for chunk in in_place.chunks_exact_mut(4) {
        let a = chunk[3] as u32;
        chunk[0] = (((chunk[0] as u32 * a) as f32 * recip_max_colors) as u32).min(max_colors as u32)
            as u16;
        chunk[1] = (((chunk[1] as u32 * a) as f32 * recip_max_colors) as u32).min(max_colors as u32)
            as u16;
        chunk[2] = (((chunk[2] as u32 * a) as f32 * recip_max_colors) as u32).min(max_colors as u32)
            as u16;
        chunk[3] = (((a * a) as f32 * recip_max_colors) as u32).min(max_colors as u32) as u16;
    }
}

/// Associate alpha in place for up to 16 bit-depth image
///
/// Note, for scaling alpha must be *associated*
///
/// # Arguments
///
/// * `in_place`: Slice to where premultiply
/// * `bit_depth`: Bit-depth of the image
///
pub fn premultiply_la16(in_place: &mut [u16], bit_depth: u32) {
    // Almost all loops are not auto-vectorised without doing anything dirty.
    // So everywhere is just added something beautiful.
    assert!(bit_depth > 0 && bit_depth <= 16);
    let max_colors = (1 << bit_depth) - 1;
    let recip_max_colors = 1. / max_colors as f32;
    for chunk in in_place.chunks_exact_mut(2) {
        let a = chunk[1] as u32;
        chunk[0] = (((chunk[0] as u32 * a) as f32 * recip_max_colors) as u32).min(max_colors as u32)
            as u16;
        chunk[1] = (((a * a) as f32 * recip_max_colors) as u32).min(max_colors as u32) as u16;
    }
}

/// Un premultiply alpha in place for up to 16 bit-depth image
///
/// Note, for scaling alpha must be *associated*
///
/// # Arguments
///
/// * `in_place`: Slice to work on
/// * `bit_depth`: Bit-depth of the image
///
///
pub fn unpremultiply_la16(in_place: &mut [u16], bit_depth: u32) {
    // Almost all loops are not auto-vectorised without doing anything dirty.
    // So everywhere is just added something beautiful.
    assert!(bit_depth > 0 && bit_depth <= 16);
    let max_colors = (1 << bit_depth) - 1;
    for chunk in in_place.chunks_exact_mut(2) {
        let a = chunk[1] as u32;
        if a != 0 {
            let a_recip = 1. / a as f32;
            chunk[0] = ((chunk[0] as u32 * max_colors) as f32 * a_recip) as u16;
            chunk[1] = ((a * max_colors) as f32 * a_recip) as u16;
        }
    }
}

/// Un premultiply alpha in place
///
/// Note, for scaling alpha must be *associated*
///
/// # Arguments
///
/// * `in_place`: Slice to work on
/// * `bit_depth`: Bit-depth of the image
///
///
pub fn unpremultiply_rgba16(in_place: &mut [u16], bit_depth: u32) {
    // Almost all loops are not auto-vectorised without doing anything dirty.
    // So everywhere is just added something beautiful.
    assert!(bit_depth > 0 && bit_depth <= 16);
    let max_colors = (1 << bit_depth) - 1;
    for chunk in in_place.chunks_exact_mut(4) {
        let a = chunk[3] as u32;
        if a != 0 {
            let a_recip = 1. / a as f32;
            chunk[0] = ((chunk[0] as u32 * max_colors) as f32 * a_recip) as u16;
            chunk[1] = ((chunk[1] as u32 * max_colors) as f32 * a_recip) as u16;
            chunk[2] = ((chunk[2] as u32 * max_colors) as f32 * a_recip) as u16;
            chunk[3] = ((a * max_colors) as f32 * a_recip) as u16;
        }
    }
}

/// Associate alpha in place
///
/// Note, for scaling alpha must be *associated*
///
/// # Arguments
///
/// * `in_place`: Slice to where premultiply
///
pub fn premultiply_rgba_f32(in_place: &mut [f32]) {
    // Almost all loops are not auto-vectorised without doing anything dirty.
    // So everywhere is just added something beautiful.
    for chunk in in_place.chunks_exact_mut(4) {
        let a = chunk[3];
        chunk[0] *= a;
        chunk[1] *= a;
        chunk[2] *= a;
        chunk[3] = a;
    }
}

/// Un premultiply alpha in place
///
/// Note, for scaling alpha must be *associated*
///
/// # Arguments
///
/// * `in_place`: Slice to work on
///
///
pub fn unpremultiply_rgba_f32(in_place: &mut [f32]) {
    for chunk in in_place.chunks_exact_mut(4) {
        let a = chunk[3];
        if a != 0. {
            let a_recip = 1. / a;
            chunk[0] *= a_recip;
            chunk[1] *= a_recip;
            chunk[2] *= a_recip;
            chunk[3] = a;
        }
    }
}
