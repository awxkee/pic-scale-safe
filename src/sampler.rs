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
#![allow(clippy::excessive_precision)]

use crate::math::bartlett::{bartlett, bartlett_hann};
use crate::math::bc_spline::{
    b_spline, catmull_rom, hermite_spline, mitchell_netravalli, robidoux, robidoux_sharp,
};
use crate::math::bilinear::bilinear;
use crate::math::blackman::blackman;
use crate::math::bohman::bohman;
use crate::math::cubic::{bicubic_spline, cubic_spline};
use crate::math::gaussian::gaussian;
use crate::math::hann::{hamming, hann, hanning};
use crate::math::kaiser::kaiser;
use crate::math::lagrange::{lagrange2, lagrange3};
use crate::math::lanczos::{
    lanczos2, lanczos2_jinc, lanczos3, lanczos3_jinc, lanczos4, lanczos4_jinc, lanczos6,
    lanczos6_jinc,
};
use crate::math::quadric::quadric;
use crate::math::sinc::sinc;
use crate::math::sphinx::sphinx;
use crate::math::spline_n::{spline16, spline36, spline64};
use crate::math::welch::welch;
use crate::math::{ConstPI, ConstSqrt2, Jinc};
use num_traits::{AsPrimitive, Float, Signed};
use std::ops::{AddAssign, MulAssign, Neg};

#[inline(always)]
pub(crate) fn box_weight<V: Copy + 'static>(_: V) -> V
where
    f32: AsPrimitive<V>,
{
    1f32.as_()
}

#[derive(Debug, Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
/// Describes resampling function that will be used
pub enum ResamplingFunction {
    Bilinear,
    Nearest,
    Cubic,
    #[default]
    MitchellNetravalli,
    CatmullRom,
    Hermite,
    BSpline,
    Hann,
    Bicubic,
    Hamming,
    Hanning,
    Blackman,
    Welch,
    Quadric,
    Gaussian,
    Sphinx,
    Bartlett,
    Robidoux,
    RobidouxSharp,
    Spline16,
    Spline36,
    Spline64,
    Kaiser,
    BartlettHann,
    Box,
    Bohman,
    Lanczos2,
    Lanczos3,
    Lanczos4,
    Lanczos2Jinc,
    Lanczos3Jinc,
    Lanczos4Jinc,
    Ginseng,
    HaasnSoft,
    Lagrange2,
    Lagrange3,
    Lanczos6,
    Lanczos6Jinc,
    /// This method replicates `INTER_AREA` behaviour from OpenCV
    Area,
}

impl From<u32> for ResamplingFunction {
    fn from(value: u32) -> Self {
        match value {
            0 => ResamplingFunction::Bilinear,
            1 => ResamplingFunction::Nearest,
            2 => ResamplingFunction::Cubic,
            3 => ResamplingFunction::MitchellNetravalli,
            4 => ResamplingFunction::CatmullRom,
            5 => ResamplingFunction::Hermite,
            6 => ResamplingFunction::BSpline,
            7 => ResamplingFunction::Hann,
            8 => ResamplingFunction::Bicubic,
            9 => ResamplingFunction::Hamming,
            10 => ResamplingFunction::Hanning,
            11 => ResamplingFunction::Blackman,
            12 => ResamplingFunction::Welch,
            13 => ResamplingFunction::Quadric,
            14 => ResamplingFunction::Gaussian,
            15 => ResamplingFunction::Sphinx,
            16 => ResamplingFunction::Bartlett,
            17 => ResamplingFunction::Robidoux,
            18 => ResamplingFunction::RobidouxSharp,
            19 => ResamplingFunction::Spline16,
            20 => ResamplingFunction::Spline36,
            21 => ResamplingFunction::Spline64,
            22 => ResamplingFunction::Kaiser,
            23 => ResamplingFunction::BartlettHann,
            24 => ResamplingFunction::Box,
            25 => ResamplingFunction::Bohman,
            26 => ResamplingFunction::Lanczos2,
            27 => ResamplingFunction::Lanczos3,
            28 => ResamplingFunction::Lanczos4,
            29 => ResamplingFunction::Lanczos2Jinc,
            30 => ResamplingFunction::Lanczos3Jinc,
            31 => ResamplingFunction::Lanczos4Jinc,
            32 => ResamplingFunction::Ginseng,
            33 => ResamplingFunction::HaasnSoft,
            34 => ResamplingFunction::Lagrange2,
            35 => ResamplingFunction::Lagrange3,
            36 => ResamplingFunction::Lanczos6,
            37 => ResamplingFunction::Lanczos6Jinc,
            38 => ResamplingFunction::Area,
            _ => ResamplingFunction::Bilinear,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct ResamplingWindow<T> {
    pub(crate) window: fn(T) -> T,
    pub(crate) window_size: f32,
    pub(crate) blur: f32,
    pub(crate) taper: f32,
}

impl<T> ResamplingWindow<T> {
    fn new(window: fn(T) -> T, window_size: f32, blur: f32, taper: f32) -> ResamplingWindow<T> {
        ResamplingWindow {
            window,
            window_size,
            blur,
            taper,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct ResamplingFilter<T> {
    pub kernel: fn(T) -> T,
    pub window: Option<ResamplingWindow<T>>,
    pub min_kernel_size: f32,
    pub is_resizable_kernel: bool,
    pub is_area_filter: bool,
}

impl<T> ResamplingFilter<T> {
    fn new(kernel: fn(T) -> T, min_kernel_size: f32) -> ResamplingFilter<T> {
        ResamplingFilter {
            kernel,
            window: None,
            min_kernel_size,
            is_resizable_kernel: true,
            is_area_filter: false,
        }
    }

    fn new_area(kernel: fn(T) -> T, min_kernel_size: f32) -> ResamplingFilter<T> {
        ResamplingFilter {
            kernel,
            window: None,
            min_kernel_size,
            is_resizable_kernel: true,
            is_area_filter: true,
        }
    }

    fn new_with_window(
        kernel: fn(T) -> T,
        window: ResamplingWindow<T>,
        min_kernel_size: f32,
    ) -> ResamplingFilter<T> {
        ResamplingFilter::<T> {
            kernel,
            window: Some(window),
            min_kernel_size,
            is_resizable_kernel: true,
            is_area_filter: false,
        }
    }

    fn new_with_fixed_kernel(kernel: fn(T) -> T, min_kernel_size: f32) -> ResamplingFilter<T> {
        ResamplingFilter::<T> {
            kernel,
            window: None,
            min_kernel_size,
            is_resizable_kernel: false,
            is_area_filter: false,
        }
    }
}

impl ResamplingFunction {
    pub(crate) fn get_resampling_filter<T>(&self) -> ResamplingFilter<T>
    where
        T: Copy
            + Neg
            + Signed
            + Float
            + 'static
            + ConstPI
            + MulAssign<T>
            + AddAssign<T>
            + AsPrimitive<f64>
            + AsPrimitive<usize>
            + Jinc<T>
            + ConstSqrt2,
        f32: AsPrimitive<T>,
        f64: AsPrimitive<T>,
        usize: AsPrimitive<T>,
    {
        match self {
            ResamplingFunction::Bilinear => ResamplingFilter::new(bilinear, 2f32),
            ResamplingFunction::Area => ResamplingFilter::new_area(box_weight, 2f32),
            ResamplingFunction::Nearest => {
                // Just a stab for nearest
                ResamplingFilter::new(bilinear, 2f32)
            }
            ResamplingFunction::Cubic => ResamplingFilter::new(cubic_spline, 2f32),
            ResamplingFunction::MitchellNetravalli => {
                ResamplingFilter::new(mitchell_netravalli, 2f32)
            }
            ResamplingFunction::Lanczos3 => ResamplingFilter::new(lanczos3, 3f32),
            ResamplingFunction::CatmullRom => ResamplingFilter::new(catmull_rom, 2f32),
            ResamplingFunction::Hermite => ResamplingFilter::new(hermite_spline, 2f32),
            ResamplingFunction::BSpline => ResamplingFilter::new(b_spline, 2f32),
            ResamplingFunction::Hann => ResamplingFilter::new(hann, 3f32),
            ResamplingFunction::Bicubic => ResamplingFilter::new(bicubic_spline, 2f32),
            ResamplingFunction::Lanczos4 => ResamplingFilter::new(lanczos4, 4f32),
            ResamplingFunction::Lanczos2 => ResamplingFilter::new(lanczos2, 2f32),
            ResamplingFunction::Hamming => ResamplingFilter::new(hamming, 2f32),
            ResamplingFunction::Hanning => ResamplingFilter::new(hanning, 2f32),
            ResamplingFunction::Welch => ResamplingFilter::new(welch, 2f32),
            ResamplingFunction::Quadric => ResamplingFilter::new(quadric, 2f32),
            ResamplingFunction::Gaussian => ResamplingFilter::new(gaussian, 2f32),
            ResamplingFunction::Sphinx => ResamplingFilter::new(sphinx, 2f32),
            ResamplingFunction::Bartlett => ResamplingFilter::new(bartlett, 2f32),
            ResamplingFunction::Robidoux => ResamplingFilter::new(robidoux, 2f32),
            ResamplingFunction::RobidouxSharp => ResamplingFilter::new(robidoux_sharp, 2f32),
            ResamplingFunction::Spline16 => ResamplingFilter::new_with_fixed_kernel(spline16, 2f32),
            ResamplingFunction::Spline36 => ResamplingFilter::new_with_fixed_kernel(spline36, 4f32),
            ResamplingFunction::Spline64 => ResamplingFilter::new_with_fixed_kernel(spline64, 6f32),
            ResamplingFunction::Kaiser => ResamplingFilter::new(kaiser, 2f32),
            ResamplingFunction::BartlettHann => ResamplingFilter::new(bartlett_hann, 2f32),
            ResamplingFunction::Box => ResamplingFilter::new(box_weight, 2f32),
            ResamplingFunction::Bohman => ResamplingFilter::new(bohman, 2f32),
            ResamplingFunction::Lanczos2Jinc => ResamplingFilter::new(lanczos2_jinc, 2f32),
            ResamplingFunction::Lanczos3Jinc => ResamplingFilter::new(lanczos3_jinc, 3f32),
            ResamplingFunction::Lanczos4Jinc => ResamplingFilter::new(lanczos4_jinc, 4f32),
            ResamplingFunction::Blackman => ResamplingFilter::new(blackman, 2f32),
            ResamplingFunction::Ginseng => ResamplingFilter::new_with_window(
                sinc,
                ResamplingWindow::new(T::jinc(), 3f32, 1f32, 0f32),
                3f32,
            ),
            ResamplingFunction::HaasnSoft => ResamplingFilter::new_with_window(
                T::jinc(),
                ResamplingWindow::new(hanning, 3f32, 1.11f32, 0f32),
                3f32,
            ),
            ResamplingFunction::Lagrange2 => ResamplingFilter::new(lagrange2, 2f32),
            ResamplingFunction::Lagrange3 => ResamplingFilter::new(lagrange3, 3f32),
            ResamplingFunction::Lanczos6Jinc => ResamplingFilter::new(lanczos6_jinc, 6f32),
            ResamplingFunction::Lanczos6 => ResamplingFilter::new(lanczos6, 6f32),
        }
    }
}
