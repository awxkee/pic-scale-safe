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

use num_traits::{AsPrimitive, Float};
use std::ops::{AddAssign, Div, Mul, MulAssign, Sub};

#[inline(always)]
pub(crate) fn bessel_i0<
    V: Copy + Mul<Output = V> + Div<Output = V> + MulAssign + AddAssign + 'static + PartialOrd,
>(
    x: V,
) -> V
where
    f64: AsPrimitive<V>,
{
    let mut s = 1.0.as_();
    let y = x * x / 4.0.as_();
    let mut t = y;
    let mut i: V = 2.0f64.as_();
    while t > 1e-12.as_() {
        s += t;
        t *= y / (i * i);
        i += 1f64.as_();
    }
    s
}

#[inline(always)]
pub(crate) fn kaiser<
    V: Copy
        + Mul<Output = V>
        + Div<Output = V>
        + MulAssign
        + AddAssign
        + 'static
        + PartialOrd
        + Sub<Output = V>
        + Float,
>(
    x: V,
) -> V
where
    f64: AsPrimitive<V>,
    f32: AsPrimitive<V>,
{
    if x > 1f32.as_() {
        return 0f32.as_();
    }
    let i0a = 1.0f64.as_() / bessel_i0(6.33f64.as_());
    bessel_i0(6.33f64.as_() * (1.0f64.as_() - x * x).sqrt()) * i0a
}
