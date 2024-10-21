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

/// Associate alpha in place
///
/// Note, for scaling alpha must be *associated*
///
/// # Arguments
///
/// * `in_place`: Slice to where premultiply
///
pub fn premultiply_rgba8(in_place: &mut [u8]) {
    for chunk in in_place.chunks_mut(4) {
        let a = chunk[3] as u16;
        let mut r = chunk[0] as u16;
        let mut g = chunk[1] as u16;
        let mut b = chunk[2] as u16;
        r = (r * a) / 255;
        g = (g * a) / 255;
        b = (b * a) / 255;
        chunk[0] = r as u8;
        chunk[1] = g as u8;
        chunk[2] = b as u8;
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
    for chunk in in_place.chunks_mut(4) {
        let a = chunk[3] as u16;
        let mut r = chunk[0] as u16;
        let mut g = chunk[1] as u16;
        let mut b = chunk[2] as u16;
        if a == 0 {
            r = 0;
            g = 0;
            b = 0;
        } else {
            r = (r * 255) / a;
            g = (g * 255) / a;
            b = (b * 255) / a;
        }
        chunk[0] = r as u8;
        chunk[1] = g as u8;
        chunk[2] = b as u8;
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
    for chunk in in_place.chunks_mut(2) {
        let a = chunk[1] as u16;
        let mut r = chunk[0] as u16;
        r = (r * a) / 255;
        chunk[0] = r as u8;
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
    for chunk in in_place.chunks_mut(2) {
        let a = chunk[1] as u16;
        let mut r = chunk[0] as u16;
        if a == 0 {
            r = 0;
        } else {
            r = (r * 255) / a;
        }
        chunk[0] = r as u8;
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
    let max_colors = (1 << bit_depth) - 1;
    for chunk in in_place.chunks_mut(4) {
        let a = chunk[3] as u32;
        let mut r = chunk[0] as u32;
        let mut g = chunk[1] as u32;
        let mut b = chunk[2] as u32;
        r = (r * a) / max_colors;
        g = (g * a) / max_colors;
        b = (b * a) / max_colors;
        chunk[0] = r as u16;
        chunk[1] = g as u16;
        chunk[2] = b as u16;
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
    let max_colors = (1 << bit_depth) - 1;
    for chunk in in_place.chunks_mut(2) {
        let a = chunk[1] as u32;
        let mut r = chunk[0] as u32;
        r = (r * a) / max_colors;
        chunk[0] = r as u16;
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
    let max_colors = (1 << bit_depth) - 1;
    for chunk in in_place.chunks_mut(2) {
        let a = chunk[1] as u32;
        let mut r = chunk[0] as u32;
        if a == 0 {
            r = 0;
        } else {
            r = (r * max_colors) / a;
        }
        chunk[0] = r as u16;
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
    let max_colors = (1 << bit_depth) - 1;
    for chunk in in_place.chunks_mut(4) {
        let a = chunk[3] as u32;
        let mut r = chunk[0] as u32;
        let mut g = chunk[1] as u32;
        let mut b = chunk[2] as u32;
        if a == 0 {
            r = 0;
            g = 0;
            b = 0;
        } else {
            r = (r * max_colors) / a;
            g = (g * max_colors) / a;
            b = (b * max_colors) / a;
        }
        chunk[0] = r as u16;
        chunk[1] = g as u16;
        chunk[2] = b as u16;
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
    for chunk in in_place.chunks_mut(4) {
        let a = chunk[3];
        let mut r = chunk[0];
        let mut g = chunk[1];
        let mut b = chunk[2];
        r *= a;
        g *= a;
        b *= a;
        chunk[0] = r;
        chunk[1] = g;
        chunk[2] = b;
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
    for chunk in in_place.chunks_mut(4) {
        let a = chunk[3];
        let mut r = chunk[0];
        let mut g = chunk[1];
        let mut b = chunk[2];
        if a == 0. {
            r = 0.;
            g = 0.;
            b = 0.;
        } else {
            r /= a;
            g /= a;
            b /= a;
        }
        chunk[0] = r;
        chunk[1] = g;
        chunk[2] = b;
    }
}
