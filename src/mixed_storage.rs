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

pub(crate) trait RoundingBackend {
    fn cpu_round(self) -> Self;
}

impl RoundingBackend for f32 {
    #[inline(always)]
    fn cpu_round(self) -> Self {
        #[cfg(any(
            all(
                any(target_arch = "x86", target_arch = "x86_64"),
                target_feature = "sse4.1"
            ),
            target_arch = "aarch64"
        ))]
        {
            self.round()
        }
        #[cfg(not(any(
            all(
                any(target_arch = "x86", target_arch = "x86_64"),
                target_feature = "sse4.1"
            ),
            target_arch = "aarch64"
        )))]
        {
            // This is always wrong for exactly N.5, so
            // we add just one eps to break this behavior.
            // This method is not valid for NaN, |x| = Inf, |x| >= 2^23
            const SHIFTER: f32 = ((1u32 << 23) + (1u32 << 22)) as f32;
            ((self + f32::EPSILON) + SHIFTER) - SHIFTER
        }
    }
}

impl RoundingBackend for f64 {
    #[inline(always)]
    fn cpu_round(self) -> Self {
        #[cfg(any(
            all(
                any(target_arch = "x86", target_arch = "x86_64"),
                target_feature = "sse4.1"
            ),
            target_arch = "aarch64"
        ))]
        {
            self.round()
        }
        #[cfg(not(any(
            all(
                any(target_arch = "x86", target_arch = "x86_64"),
                target_feature = "sse4.1"
            ),
            target_arch = "aarch64"
        )))]
        {
            // This is always wrong for exactly N.5, so
            // we add just one eps to break this behavior.
            // This method is not valid for NaN, |x| = Inf, |x| >= 2^52.
            const SHIFTER: f64 = ((1u64 << 52) + (1u64 << 51)) as f64;
            ((self + f64::EPSILON) + SHIFTER) - SHIFTER
        }
    }
}

pub(crate) trait MixedStorage<T> {
    fn to_mixed(self, bit_depth: u32) -> T;
}

impl MixedStorage<u8> for f32 {
    #[inline(always)]
    #[allow(clippy::manual_clamp)]
    fn to_mixed(self, _: u32) -> u8 {
        self.cpu_round().max(0.).min(255.) as u8
    }
}

impl MixedStorage<u8> for f64 {
    #[inline(always)]
    #[allow(clippy::manual_clamp)]
    fn to_mixed(self, _: u32) -> u8 {
        self.cpu_round().max(0.).min(255.) as u8
    }
}

impl MixedStorage<u16> for f32 {
    #[inline(always)]
    #[allow(clippy::manual_clamp)]
    fn to_mixed(self, bit_depth: u32) -> u16 {
        self.cpu_round().max(0.).min(((1 << bit_depth) - 1) as f32) as u16
    }
}

impl MixedStorage<u16> for f64 {
    #[inline(always)]
    #[allow(clippy::manual_clamp)]
    fn to_mixed(self, bit_depth: u32) -> u16 {
        self.cpu_round().max(0.).min(((1 << bit_depth) - 1) as f64) as u16
    }
}

impl MixedStorage<f32> for f32 {
    #[inline(always)]
    #[allow(clippy::manual_clamp)]
    fn to_mixed(self, _: u32) -> f32 {
        self
    }
}

impl MixedStorage<f64> for f64 {
    #[inline(always)]
    #[allow(clippy::manual_clamp)]
    fn to_mixed(self, _: u32) -> f64 {
        self
    }
}
