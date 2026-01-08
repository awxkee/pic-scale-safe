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

const fn make_unpremultiplication_table() -> [u8; 65536] {
    let mut alpha = 0usize;
    let mut buf = [0u8; 65536];
    while alpha < 256 {
        let mut pixel = 0usize;
        while pixel < 256 {
            if alpha == 0 {
                buf[alpha * 255 + pixel] = 0;
            } else {
                let value = (pixel * 255 + alpha / 2) / alpha;
                buf[alpha * 255 + pixel] = if value > 255 { 255 } else { value as u8 };
            }
            pixel += 1;
        }
        alpha += 1;
    }
    buf
}

pub(crate) static UNPREMULTIPLICATION_TABLE: [u8; 65536] = make_unpremultiplication_table();

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
        chunk[0] = UNPREMULTIPLICATION_TABLE[(a + chunk[0] as u16) as usize];
        chunk[1] = UNPREMULTIPLICATION_TABLE[(a + chunk[1] as u16) as usize];
        chunk[2] = UNPREMULTIPLICATION_TABLE[(a + chunk[2] as u16) as usize];
    }
}

/// Associate alpha to new slice
///
/// Faster if you need to do a copy first.
/// Note, for scaling alpha must be *associated*
///
/// # Arguments
///
/// * `source`: Source slice with RGBA data
///
pub fn premultiplied_rgba8(source: &[u8]) -> Vec<u8> {
    let mut target = vec![0u8; source.len()];
    // Almost all loops are not auto-vectorised without doing anything dirty.
    // So everywhere is just added something beautiful.w
    for (dst, src) in target.chunks_exact_mut(4).zip(source.chunks_exact(4)) {
        let a = src[3] as u16;
        dst[0] = UNPREMULTIPLICATION_TABLE[(a + src[0] as u16) as usize];
        dst[1] = UNPREMULTIPLICATION_TABLE[(a + src[1] as u16) as usize];
        dst[2] = UNPREMULTIPLICATION_TABLE[(a + src[2] as u16) as usize];
        dst[3] = src[3];
    }
    target
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
        chunk[0] = UNPREMULTIPLICATION_TABLE[(a + chunk[0] as u16) as usize];
    }
}

/// Associate alpha to a new destination
///
/// Faster if you need to do a copy first.
/// Note, for scaling alpha must be *associated*
///
/// # Arguments
///
/// * `source`: Source slice with LA data
///
pub fn premultiplied_la8(source: &[u8]) -> Vec<u8> {
    let mut target = vec![0u8; source.len()];
    // Almost all loops are not auto-vectorised without doing anything dirty.
    // So everywhere is just added something beautiful.
    for (dst, src) in target.chunks_exact_mut(2).zip(source.chunks_exact(2)) {
        let a = src[1] as u16;
        dst[0] = UNPREMULTIPLICATION_TABLE[(a + src[0] as u16) as usize];
        dst[1] = a as u8;
    }
    target
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

#[inline]
pub fn div_by_1023(v: u32) -> u16 {
    let round = 1 << 9;
    let v = v + round;
    (((v >> 10) + v) >> 10) as u16
}

#[inline]
pub fn div_by_4095(v: u32) -> u16 {
    let round = 1 << 11;
    let v = v + round;
    (((v >> 12) + v) >> 12) as u16
}

#[inline]
pub fn div_by_65535(v: u32) -> u16 {
    let round = 1 << 15;
    let v = v + round;
    (((v >> 16) + v) >> 16) as u16
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
    if bit_depth == 10 {
        for chunk in in_place.chunks_exact_mut(4) {
            let a = chunk[3] as u32;
            chunk[0] = div_by_1023(chunk[0] as u32 * a);
            chunk[1] = div_by_1023(chunk[1] as u32 * a);
            chunk[2] = div_by_1023(chunk[2] as u32 * a);
            chunk[3] = div_by_1023(a * a);
        }
    } else if bit_depth == 12 {
        for chunk in in_place.chunks_exact_mut(4) {
            let a = chunk[3] as u32;
            chunk[0] = div_by_4095(chunk[0] as u32 * a);
            chunk[1] = div_by_4095(chunk[1] as u32 * a);
            chunk[2] = div_by_4095(chunk[2] as u32 * a);
            chunk[3] = div_by_4095(a * a);
        }
    } else if bit_depth == 16 {
        for chunk in in_place.chunks_exact_mut(4) {
            let a = chunk[3] as u32;
            chunk[0] = div_by_65535(chunk[0] as u32 * a);
            chunk[1] = div_by_65535(chunk[1] as u32 * a);
            chunk[2] = div_by_65535(chunk[2] as u32 * a);
            chunk[3] = div_by_65535(a * a);
        }
    } else {
        for chunk in in_place.chunks_exact_mut(4) {
            let a = chunk[3] as u32;
            chunk[0] = (((chunk[0] as u32 * a) as f32 * recip_max_colors).round() as u32)
                .min(max_colors as u32) as u16;
            chunk[1] = (((chunk[1] as u32 * a) as f32 * recip_max_colors).round() as u32)
                .min(max_colors as u32) as u16;
            chunk[2] = (((chunk[2] as u32 * a) as f32 * recip_max_colors).round() as u32)
                .min(max_colors as u32) as u16;
            chunk[3] =
                (((a * a) as f32 * recip_max_colors).round() as u32).min(max_colors as u32) as u16;
        }
    }
}

/// Associate alpha to a new destination
///
/// Faster, if you need to copy data first.
/// Note, for scaling alpha must be *associated*
///
/// # Arguments
///
/// * `source`: Source slice with RGBA16 data
/// * `bit_depth`: Bit-depth of the image
///
pub fn premultiplied_rgba16(source: &[u16], bit_depth: u32) -> Vec<u16> {
    let mut target = vec![0u16; source.len()];
    // Almost all loops are not auto-vectorised without doing anything dirty.
    // So everywhere is just added something beautiful.
    assert!(bit_depth > 0 && bit_depth <= 16);
    let max_colors = (1 << bit_depth) - 1;
    let recip_max_colors = 1. / max_colors as f32;
    if bit_depth == 10 {
        for (dst, src) in target.chunks_exact_mut(4).zip(source.chunks_exact(4)) {
            let a = src[3] as u32;
            dst[0] = div_by_1023(src[0] as u32 * a);
            dst[1] = div_by_1023(src[1] as u32 * a);
            dst[2] = div_by_1023(src[2] as u32 * a);
            dst[3] = div_by_1023(a * a);
        }
    } else if bit_depth == 12 {
        for (dst, src) in target.chunks_exact_mut(4).zip(source.chunks_exact(4)) {
            let a = src[3] as u32;
            dst[0] = div_by_4095(src[0] as u32 * a);
            dst[1] = div_by_4095(src[1] as u32 * a);
            dst[2] = div_by_4095(src[2] as u32 * a);
            dst[3] = div_by_4095(a * a);
        }
    } else if bit_depth == 16 {
        for (dst, src) in target.chunks_exact_mut(4).zip(source.chunks_exact(4)) {
            let a = src[3] as u32;
            dst[0] = div_by_65535(src[0] as u32 * a);
            dst[1] = div_by_65535(src[1] as u32 * a);
            dst[2] = div_by_65535(src[2] as u32 * a);
            dst[3] = div_by_65535(a * a);
        }
    } else {
        for (dst, src) in target.chunks_exact_mut(4).zip(source.chunks_exact(4)) {
            let a = src[3] as u32;
            dst[0] = (((src[0] as u32 * a) as f32 * recip_max_colors).round() as u32)
                .min(max_colors as u32) as u16;
            dst[1] = (((src[1] as u32 * a) as f32 * recip_max_colors).round() as u32)
                .min(max_colors as u32) as u16;
            dst[2] = (((src[2] as u32 * a) as f32 * recip_max_colors).round() as u32)
                .min(max_colors as u32) as u16;
            dst[3] =
                (((a * a) as f32 * recip_max_colors).round() as u32).min(max_colors as u32) as u16;
        }
    }
    target
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
    if bit_depth == 10 {
        for chunk in in_place.chunks_exact_mut(2) {
            let a = chunk[1] as u32;
            chunk[0] = div_by_1023(chunk[0] as u32 * a);
            chunk[1] = div_by_1023(a * a);
        }
    } else if bit_depth == 12 {
        for chunk in in_place.chunks_exact_mut(2) {
            let a = chunk[1] as u32;
            chunk[0] = div_by_4095(chunk[0] as u32 * a);
            chunk[1] = div_by_4095(a * a);
        }
    } else if bit_depth == 16 {
        for chunk in in_place.chunks_exact_mut(2) {
            let a = chunk[1] as u32;
            chunk[0] = div_by_65535(chunk[0] as u32 * a);
            chunk[1] = div_by_65535(a * a);
        }
    } else {
        let recip_max_colors = 1. / max_colors as f32;
        for chunk in in_place.chunks_exact_mut(2) {
            let a = chunk[1] as u32;
            chunk[0] = (((chunk[0] as u32 * a) as f32 * recip_max_colors).round() as u32)
                .min(max_colors as u32) as u16;
            chunk[1] =
                (((a * a) as f32 * recip_max_colors).round() as u32).min(max_colors as u32) as u16;
        }
    }
}

/// Associate alpha for up to 16 bit-depth image to a new destination
///
/// Faster, if you need to copy data first.
/// Note, for scaling alpha must be *associated*
///
/// # Arguments
///
/// * `source`: Slice with source LA16 data
/// * `bit_depth`: Bit-depth of the image
///
pub fn premultiplied_la16(source: &[u16], bit_depth: u32) -> Vec<u16> {
    let mut target = vec![0u16; source.len()];
    // Almost all loops are not auto-vectorised without doing anything dirty.
    // So everywhere is just added something beautiful.
    assert!(bit_depth > 0 && bit_depth <= 16);
    let max_colors = (1 << bit_depth) - 1;
    if bit_depth == 10 {
        for (dst, src) in target.chunks_exact_mut(2).zip(source.chunks_exact(2)) {
            let a = src[1] as u32;
            dst[0] = div_by_1023(src[0] as u32 * a);
            dst[1] = div_by_1023(a * a);
        }
    } else if bit_depth == 12 {
        for (dst, src) in target.chunks_exact_mut(2).zip(source.chunks_exact(2)) {
            let a = src[1] as u32;
            dst[0] = div_by_4095(src[0] as u32 * a);
            dst[1] = div_by_4095(a * a);
        }
    } else if bit_depth == 16 {
        for (dst, src) in target.chunks_exact_mut(2).zip(source.chunks_exact(2)) {
            let a = src[1] as u32;
            dst[0] = div_by_65535(src[0] as u32 * a);
            dst[1] = div_by_65535(a * a);
        }
    } else {
        let recip_max_colors = 1. / max_colors as f32;
        for (dst, src) in target.chunks_exact_mut(2).zip(source.chunks_exact(2)) {
            let a = src[1] as u32;
            dst[0] = (((src[0] as u32 * a) as f32 * recip_max_colors).round() as u32)
                .min(max_colors as u32) as u16;
            dst[1] =
                (((a * a) as f32 * recip_max_colors).round() as u32).min(max_colors as u32) as u16;
        }
    }
    target
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

/// Associate alpha in place
///
/// Note, for scaling alpha must be *associated*
///
/// # Arguments
///
/// * `in_place`: Slice to where premultiply
///
pub fn premultiply_luma_alpha_f32(in_place: &mut [f32]) {
    // Almost all loops are not auto-vectorised without doing anything dirty.
    // So everywhere is just added something beautiful.
    for chunk in in_place.chunks_exact_mut(2) {
        let a = chunk[1];
        chunk[0] *= a;
        chunk[2] = a;
    }
}

/// Associate alpha to a new destination
///
/// Faster, if you need to do a copy first
/// Note, for scaling alpha must be *associated*
///
/// # Arguments
///
/// * `source`: Source slice with luma alpha
///
pub fn premultiplied_luma_alpha_f32(source: &[f32]) -> Vec<f32> {
    let mut target = vec![0.; source.len()];
    // Almost all loops are not auto-vectorised without doing anything dirty.
    // So everywhere is just added something beautiful.
    for (dst, src) in target.chunks_exact_mut(2).zip(source.chunks_exact(2)) {
        let a = src[2];
        dst[0] = src[0] * a;
        dst[1] = a;
    }
    target
}

/// Associate alpha to a new destination
///
/// Faster, if you need to do a copy first
/// Note, for scaling alpha must be *associated*
///
/// # Arguments
///
/// * `source`: Source rgba slice
///
pub fn premultiplied_rgba_f32(source: &[f32]) -> Vec<f32> {
    let mut target = vec![0.; source.len()];
    // Almost all loops are not auto-vectorised without doing anything dirty.
    // So everywhere is just added something beautiful.
    for (dst, src) in target.chunks_exact_mut(4).zip(source.chunks_exact(4)) {
        let a = src[3];
        dst[0] = src[0] * a;
        dst[1] = src[1] * a;
        dst[2] = src[2] * a;
        dst[3] = a;
    }
    target
}

/// Un-premultiply alpha in place
///
/// Note, for scaling alpha must be *associated*
///
/// # Arguments
///
/// * `in_place`: Slice to work on
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

/// Un-premultiply alpha in place
///
/// Note, for scaling alpha must be *associated*
///
/// # Arguments
///
/// * `in_place`: Slice to work on
///
pub fn unpremultiply_luma_alpha_f32(in_place: &mut [f32]) {
    for chunk in in_place.chunks_exact_mut(2) {
        let a = chunk[1];
        if a != 0. {
            let a_recip = 1. / a;
            chunk[0] *= a_recip;
            chunk[1] = a;
        }
    }
}
