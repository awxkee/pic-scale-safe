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

/// Struct that represents image size
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ImageSize {
    pub width: usize,
    pub height: usize,
}

impl ImageSize {
    /// Creates new image size
    pub fn new(width: usize, height: usize) -> ImageSize {
        ImageSize { width, height }
    }
}

/// A 32-bit fixed point extension of an index into a memory region
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct FractIndex {
    pub int: usize,
    pub frac: u32,
}

impl From<usize> for FractIndex {
    fn from(value: usize) -> Self {
        Self {
            int: value,
            frac: 0,
        }
    }
}

/// Struct that represents a rectangular region to crop an image to before resizing.
///
/// Each coordinate is a pair of an integral part and a 0.32 fractional
/// part; the latter is zero for rectangles with integer boundaries.
///
/// This is only valid and nonempty if x1 < x2 and y1 < y2.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct CropRect {
    pub x1: FractIndex,
    pub x2: FractIndex,
    pub y1: FractIndex,
    pub y2: FractIndex,
}

impl CropRect {
    /// Creates new rectangle
    pub fn new_integral(x1: usize, x2: usize, y1: usize, y2: usize) -> CropRect {
        CropRect {
            x1: FractIndex::from(x1),
            x2: FractIndex::from(x2),
            y1: FractIndex::from(y1),
            y2: FractIndex::from(y2),
        }
    }

    /// Creates new rectangle
    ///
    /// This converts the rectangle specified by the given floating point values
    /// to fixed point; values are clamped to the range `[0, usize::MAX + 1 - 2^{-32}]`.
    /// NaN may be handled arbitrarily.
    ///
    /// Note that a nonempty floating point rectangle may round to an empty
    /// [CropRect]; for example, if `x1 = 0` and `x2 = 1e-10`, the latter will
    /// round to 0.
    pub fn new_from_f64(x1: f64, x2: f64, y1: f64, y2: f64) -> CropRect {
        let to_fixed = |z: f64| -> FractIndex {
            let mut w = (z * (1u64 << 32) as f64).round() as u128;
            w = w.min((1 << (u32::BITS + usize::BITS)) - 1);
            FractIndex {
                int: (w >> u32::BITS) as usize,
                frac: (w & u32::MAX as u128) as u32,
            }
        };

        CropRect {
            x1: to_fixed(x1),
            x2: to_fixed(x2),
            y1: to_fixed(y1),
            y2: to_fixed(y2),
        }
    }
}

impl From<ImageSize> for CropRect {
    fn from(size: ImageSize) -> Self {
        CropRect::new_integral(0, size.width, 0, size.height)
    }
}
