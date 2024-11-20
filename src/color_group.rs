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
use crate::mlaf::mlaf;
use crate::saturate_narrow::SaturateNarrow;
use num_traits::{FromPrimitive, MulAdd};
use std::ops::{Add, AddAssign, Mul, Shr, ShrAssign, Sub, SubAssign};

#[derive(Debug, Clone, Copy)]
pub(crate) struct ColorGroup<const COMPS: usize, J: Copy> {
    pub r: J,
    pub g: J,
    pub b: J,
    pub a: J,
}

impl<const COMPS: usize, J> ColorGroup<COMPS, J>
where
    J: Copy + Default,
{
    #[inline(always)]
    pub(crate) fn new() -> ColorGroup<COMPS, J> {
        ColorGroup {
            r: J::default(),
            g: J::default(),
            b: J::default(),
            a: J::default(),
        }
    }

    #[inline(always)]
    pub(crate) fn from_components(r: J, g: J, b: J, a: J) -> ColorGroup<COMPS, J> {
        ColorGroup { r, g, b, a }
    }

    #[inline(always)]
    pub(crate) fn dup(v: J) -> ColorGroup<COMPS, J> {
        ColorGroup {
            r: v,
            g: v,
            b: v,
            a: v,
        }
    }
}

macro_rules! load_color_group {
    ($store: expr, $channels: expr, $vtype: ty) => {{
        if $channels == 1 {
            ColorGroup::<$channels, $vtype> {
                r: $store[0].as_(),
                g: 0.as_(),
                b: 0.as_(),
                a: 0.as_(),
            }
        } else if $channels == 2 {
            ColorGroup::<$channels, $vtype> {
                r: $store[0].as_(),
                g: $store[1].as_(),
                b: 0.as_(),
                a: 0.as_(),
            }
        } else if $channels == 3 {
            ColorGroup::<$channels, $vtype> {
                r: $store[0].as_(),
                g: $store[1].as_(),
                b: $store[2].as_(),
                a: 0.as_(),
            }
        } else if $channels == 4 {
            ColorGroup::<$channels, $vtype> {
                r: $store[0].as_(),
                g: $store[1].as_(),
                b: $store[2].as_(),
                a: $store[3].as_(),
            }
        } else {
            unimplemented!("Not implemented.")
        }
    }};
}

pub(crate) use load_color_group;

macro_rules! load_color_group_with_offset {
    ($store: expr, $channels: expr, $offset: expr, $vtype: ty) => {{
        if $channels == 1 {
            ColorGroup::<$channels, $vtype> {
                r: $store[$offset].as_(),
                g: 0.as_(),
                b: 0.as_(),
                a: 0.as_(),
            }
        } else if $channels == 2 {
            ColorGroup::<$channels, $vtype> {
                r: $store[$offset].as_(),
                g: $store[$offset + 1].as_(),
                b: 0.as_(),
                a: 0.as_(),
            }
        } else if $channels == 3 {
            ColorGroup::<$channels, $vtype> {
                r: $store[$offset].as_(),
                g: $store[$offset + 1].as_(),
                b: $store[$offset + 2].as_(),
                a: 0.as_(),
            }
        } else if $channels == 4 {
            ColorGroup::<$channels, $vtype> {
                r: $store[$offset].as_(),
                g: $store[$offset + 1].as_(),
                b: $store[$offset + 2].as_(),
                a: $store[$offset + 3].as_(),
            }
        } else {
            unimplemented!("Not implemented.")
        }
    }};
}

pub(crate) use load_color_group_with_offset;

macro_rules! store_color_group {
    ($color_group: expr, $store: expr, $components: expr) => {{
        $store[0] = $color_group.r;
        if $components > 1 {
            $store[1] = $color_group.g;
        }
        if $components > 2 {
            $store[2] = $color_group.b;
        }
        if $components == 4 {
            $store[3] = $color_group.a;
        }
    }};
}

pub(crate) use store_color_group;

macro_rules! fast_mixed_store_color_group {
    ($color_group: expr, $store: expr, $components: expr, $bit_depth: expr) => {{
        $store[0] = $color_group.r.to_mixed($bit_depth);
        if $components > 1 {
            $store[1] = $color_group.g.to_mixed($bit_depth);
        }
        if $components > 2 {
            $store[2] = $color_group.b.to_mixed($bit_depth);
        }
        if $components == 4 {
            $store[3] = $color_group.a.to_mixed($bit_depth);
        }
    }};
}

pub(crate) use fast_mixed_store_color_group;

impl<const COMPS: usize, J> Mul<J> for ColorGroup<COMPS, J>
where
    J: Copy + Mul<Output = J> + Default + 'static,
{
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: J) -> Self::Output {
        if COMPS == 1 {
            ColorGroup::from_components(self.r * rhs, self.g, self.b, self.a)
        } else if COMPS == 2 {
            ColorGroup::from_components(self.r * rhs, self.g * rhs, self.b, self.a)
        } else if COMPS == 3 {
            ColorGroup::from_components(self.r * rhs, self.g * rhs, self.b * rhs, self.a)
        } else if COMPS == 4 {
            ColorGroup::from_components(self.r * rhs, self.g * rhs, self.b * rhs, self.a * rhs)
        } else {
            unimplemented!("Not implemented.");
        }
    }
}

impl<const COMPS: usize, J> ColorGroup<COMPS, J>
where
    J: Copy + Default + 'static,
{
    #[inline(always)]
    pub(crate) fn saturate_narrow<V>(&self, bit_depth: u32) -> ColorGroup<COMPS, V>
    where
        V: Copy + Default,
        J: SaturateNarrow<V>,
    {
        if COMPS == 1 {
            ColorGroup::<COMPS, V>::from_components(
                self.r.saturate_narrow(bit_depth),
                V::default(),
                V::default(),
                V::default(),
            )
        } else if COMPS == 2 {
            ColorGroup::<COMPS, V>::from_components(
                self.r.saturate_narrow(bit_depth),
                self.g.saturate_narrow(bit_depth),
                V::default(),
                V::default(),
            )
        } else if COMPS == 3 {
            ColorGroup::<COMPS, V>::from_components(
                self.r.saturate_narrow(bit_depth),
                self.g.saturate_narrow(bit_depth),
                self.b.saturate_narrow(bit_depth),
                V::default(),
            )
        } else {
            ColorGroup::<COMPS, V>::from_components(
                self.r.saturate_narrow(bit_depth),
                self.g.saturate_narrow(bit_depth),
                self.b.saturate_narrow(bit_depth),
                self.a.saturate_narrow(bit_depth),
            )
        }
    }
}

impl<const COMPS: usize, J> Mul<ColorGroup<COMPS, J>> for ColorGroup<COMPS, J>
where
    J: Copy + Mul<Output = J> + Default + 'static,
{
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: ColorGroup<COMPS, J>) -> Self::Output {
        if COMPS == 1 {
            ColorGroup::from_components(self.r * rhs.r, self.g, self.b, self.a)
        } else if COMPS == 2 {
            ColorGroup::from_components(self.r * rhs.r, self.g * rhs.g, self.b, self.a)
        } else if COMPS == 3 {
            ColorGroup::from_components(self.r * rhs.r, self.g * rhs.g, self.b * rhs.b, self.a)
        } else if COMPS == 4 {
            ColorGroup::from_components(
                self.r * rhs.r,
                self.g * rhs.g,
                self.b * rhs.b,
                self.a * rhs.b,
            )
        } else {
            unimplemented!("Not implemented.");
        }
    }
}

impl<const COMPS: usize, J> Sub<J> for ColorGroup<COMPS, J>
where
    J: Copy + Sub<Output = J> + Default + 'static,
{
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: J) -> Self::Output {
        if COMPS == 1 {
            ColorGroup::from_components(self.r - rhs, self.g, self.b, self.a)
        } else if COMPS == 2 {
            ColorGroup::from_components(self.r - rhs, self.g - rhs, self.b, self.a)
        } else if COMPS == 3 {
            ColorGroup::from_components(self.r - rhs, self.g - rhs, self.b - rhs, self.a)
        } else if COMPS == 4 {
            ColorGroup::from_components(self.r - rhs, self.g - rhs, self.b - rhs, self.a - rhs)
        } else {
            unimplemented!("Not implemented.");
        }
    }
}

impl<const COMPS: usize, J> Sub<ColorGroup<COMPS, J>> for ColorGroup<COMPS, J>
where
    J: Copy + Sub<Output = J> + Default + 'static,
{
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: ColorGroup<COMPS, J>) -> Self::Output {
        if COMPS == 1 {
            ColorGroup::from_components(self.r - rhs.r, self.g, self.b, self.a)
        } else if COMPS == 2 {
            ColorGroup::from_components(self.r - rhs.r, self.g - rhs.g, self.b, self.a)
        } else if COMPS == 3 {
            ColorGroup::from_components(self.r - rhs.r, self.g - rhs.g, self.b - rhs.b, self.a)
        } else if COMPS == 4 {
            ColorGroup::from_components(
                self.r - rhs.r,
                self.g - rhs.g,
                self.b - rhs.b,
                self.a - rhs.a,
            )
        } else {
            unimplemented!("Not implemented.");
        }
    }
}

impl<const COMPS: usize, J> Add<ColorGroup<COMPS, J>> for ColorGroup<COMPS, J>
where
    J: Copy + Add<Output = J> + Default + 'static,
{
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: ColorGroup<COMPS, J>) -> Self::Output {
        if COMPS == 1 {
            ColorGroup::from_components(self.r + rhs.r, self.g, self.b, self.a)
        } else if COMPS == 2 {
            ColorGroup::from_components(self.r + rhs.r, self.g + rhs.g, self.b, self.a)
        } else if COMPS == 3 {
            ColorGroup::from_components(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b, self.a)
        } else if COMPS == 4 {
            ColorGroup::from_components(
                self.r + rhs.r,
                self.g + rhs.g,
                self.b + rhs.b,
                self.a + rhs.a,
            )
        } else {
            unimplemented!("Not implemented.");
        }
    }
}

impl<const COMPS: usize, J> Add<J> for ColorGroup<COMPS, J>
where
    J: Copy + Add<Output = J> + Default + 'static,
{
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: J) -> Self::Output {
        if COMPS == 1 {
            ColorGroup::from_components(self.r + rhs, self.g, self.b, self.a)
        } else if COMPS == 2 {
            ColorGroup::from_components(self.r + rhs, self.g + rhs, self.b, self.a)
        } else if COMPS == 3 {
            ColorGroup::from_components(self.r + rhs, self.g + rhs, self.b + rhs, self.a)
        } else if COMPS == 4 {
            ColorGroup::from_components(self.r + rhs, self.g + rhs, self.b + rhs, self.a + rhs)
        } else {
            unimplemented!("Not implemented.");
        }
    }
}

impl<const COMPS: usize, J> Shr<J> for ColorGroup<COMPS, J>
where
    J: Copy + Shr<J, Output = J> + Default + 'static,
{
    type Output = Self;

    #[inline(always)]
    fn shr(self, rhs: J) -> Self::Output {
        if COMPS == 1 {
            ColorGroup::from_components(self.r >> rhs, self.g, self.b, self.a)
        } else if COMPS == 2 {
            ColorGroup::from_components(self.r >> rhs, self.g >> rhs, self.b, self.a)
        } else if COMPS == 3 {
            ColorGroup::from_components(self.r >> rhs, self.g >> rhs, self.b >> rhs, self.a)
        } else if COMPS == 4 {
            ColorGroup::from_components(self.r >> rhs, self.g >> rhs, self.b >> rhs, self.a >> rhs)
        } else {
            unimplemented!("Not implemented.");
        }
    }
}

impl<const COMPS: usize, J> ShrAssign<J> for ColorGroup<COMPS, J>
where
    J: Copy + ShrAssign<J> + Default + 'static,
{
    #[inline(always)]
    fn shr_assign(&mut self, rhs: J) {
        if COMPS == 1 {
            self.r >>= rhs;
        } else if COMPS == 2 {
            self.r >>= rhs;
            self.g >>= rhs;
        } else if COMPS == 3 {
            self.r >>= rhs;
            self.g >>= rhs;
            self.b >>= rhs;
        } else if COMPS == 4 {
            self.r >>= rhs;
            self.g >>= rhs;
            self.b >>= rhs;
            self.a >>= rhs;
        }
    }
}

impl<const COMPS: usize, J> MulAdd<ColorGroup<COMPS, J>, J> for ColorGroup<COMPS, J>
where
    J: Copy + MulAdd<J, Output = J> + Mul<J, Output = J> + Add<J, Output = J> + Default + 'static,
{
    type Output = Self;

    #[inline(always)]
    fn mul_add(self, a: ColorGroup<COMPS, J>, b: J) -> Self::Output {
        if COMPS == 1 {
            ColorGroup::from_components(mlaf(self.r, a.r, b), self.g, self.b, self.a)
        } else if COMPS == 2 {
            ColorGroup::from_components(mlaf(self.r, a.r, b), mlaf(self.g, a.g, b), self.b, self.a)
        } else if COMPS == 3 {
            ColorGroup::from_components(
                mlaf(self.r, a.r, b),
                mlaf(self.g, a.g, b),
                mlaf(self.b, a.b, b),
                self.a,
            )
        } else if COMPS == 4 {
            ColorGroup::from_components(
                mlaf(self.r, a.r, b),
                mlaf(self.g, a.g, b),
                mlaf(self.b, a.b, b),
                mlaf(self.a, a.a, b),
            )
        } else {
            unimplemented!("Not implemented.");
        }
    }
}

impl<const COMPS: usize, J> AddAssign<ColorGroup<COMPS, J>> for ColorGroup<COMPS, J>
where
    J: Copy + AddAssign,
{
    #[inline(always)]
    fn add_assign(&mut self, rhs: ColorGroup<COMPS, J>) {
        if COMPS == 1 {
            self.r += rhs.r;
        } else if COMPS == 2 {
            self.r += rhs.r;
            self.g += rhs.g;
        } else if COMPS == 3 {
            self.r += rhs.r;
            self.g += rhs.g;
            self.b += rhs.b;
        } else if COMPS == 4 {
            self.r += rhs.r;
            self.g += rhs.g;
            self.b += rhs.b;
            self.a += rhs.a;
        }
    }
}

impl<const COMPS: usize, J> SubAssign<ColorGroup<COMPS, J>> for ColorGroup<COMPS, J>
where
    J: Copy + SubAssign,
{
    #[inline(always)]
    fn sub_assign(&mut self, rhs: ColorGroup<COMPS, J>) {
        if COMPS == 1 {
            self.r -= rhs.r;
        } else if COMPS == 2 {
            self.r -= rhs.r;
            self.g -= rhs.g;
        } else if COMPS == 3 {
            self.r -= rhs.r;
            self.g -= rhs.g;
            self.b -= rhs.b;
        } else if COMPS == 4 {
            self.r -= rhs.r;
            self.g -= rhs.g;
            self.b -= rhs.b;
            self.a -= rhs.a;
        }
    }
}

impl<const COMPS: usize, J> Default for ColorGroup<COMPS, J>
where
    J: Copy + FromPrimitive + Default,
{
    #[inline(always)]
    fn default() -> Self {
        ColorGroup::new()
    }
}
