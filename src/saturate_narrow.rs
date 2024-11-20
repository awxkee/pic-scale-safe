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

pub(crate) trait SaturateNarrow<J> {
    fn saturate_narrow(self, bit_depth: u32) -> J;
}

impl SaturateNarrow<u8> for i32 {
    #[inline(always)]
    #[allow(clippy::manual_clamp)]
    fn saturate_narrow(self, _: u32) -> u8 {
        (self >> PRECISION).max(0).min(255) as u8
    }
}

impl SaturateNarrow<u8> for i64 {
    #[inline(always)]
    #[allow(clippy::manual_clamp)]
    fn saturate_narrow(self, _: u32) -> u8 {
        (self >> PRECISION).max(0).min(255) as u8
    }
}

impl SaturateNarrow<u16> for i32 {
    #[inline(always)]
    #[allow(clippy::manual_clamp)]
    fn saturate_narrow(self, bit_depth: u32) -> u16 {
        (self >> PRECISION).max(0).min((1 << bit_depth) - 1) as u16
    }
}

impl SaturateNarrow<u16> for i64 {
    #[inline(always)]
    #[allow(clippy::manual_clamp)]
    fn saturate_narrow(self, bit_depth: u32) -> u16 {
        (self >> PRECISION).max(0).min((1 << bit_depth) - 1) as u16
    }
}
