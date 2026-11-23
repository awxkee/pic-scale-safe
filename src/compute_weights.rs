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
use crate::filter_weights::{FilterBounds, FilterWeights};
use crate::image_size::FractIndex;
use crate::math::{ConstPI, ConstSqrt2, Jinc};
use crate::sampler::ResamplingFunction;
use num_traits::{AsPrimitive, Float, Signed};
use std::fmt::Debug;
use std::ops::{AddAssign, Div, MulAssign, Neg};

/// Fixed point extension of [FractIndex], used to more accurately and
/// efficiently interpolate between [FractIndex] values.
#[derive(Clone, Copy, Debug)]
struct ExtFractIndex {
    main: FractIndex,
    acc: u64,
}

impl ExtFractIndex {
    fn round(&self) -> FractIndex {
        if self.acc >= (1 << (u64::BITS - 1)) {
            let (v1, c) = self.main.frac.carrying_add(1, false);
            FractIndex {
                int: self.main.int.saturating_add(c as usize),
                frac: v1,
            }
        } else {
            self.main
        }
    }

    fn add(&self, y: ExtFractIndex) -> ExtFractIndex {
        let (z2, z2c) = self.acc.carrying_add(y.acc, false);
        let (z1, z1c) = self.main.frac.carrying_add(y.main.frac, z2c);
        let (z0, z0c) = self.main.int.carrying_add(y.main.int, z1c);
        assert!(!z0c); // overflow not expected in this file
        ExtFractIndex {
            main: FractIndex { int: z0, frac: z1 },
            acc: z2,
        }
    }

    fn div(&self, steps: usize) -> ExtFractIndex {
        let steps_u128 = steps as u128;
        let x1 = ((self.main.int as u128) << u32::BITS) | (self.main.frac as u128);
        let v1 = x1 / steps_u128;
        let r1 = x1 % steps_u128;
        let x2 = r1 << u64::BITS | (self.acc as u128);
        let v2 = x2 / steps_u128;

        let o1 = ExtFractIndex {
            main: FractIndex {
                int: ((v1 >> u32::BITS) as usize),
                frac: (v1 & u32::MAX as u128) as u32,
            },
            acc: 0,
        };
        let o2 = ExtFractIndex {
            main: FractIndex {
                int: ((v2 >> (u64::BITS + u32::BITS)) as usize),
                frac: ((v2 >> u64::BITS) & u32::MAX as u128) as u32,
            },
            acc: (v2 & u64::MAX as u128) as u64,
        };
        o1.add(o2)
    }
    fn div2(&self) -> ExtFractIndex {
        ExtFractIndex {
            main: FractIndex {
                int: self.main.int >> 1,
                frac: (self.main.frac >> 1) | (((self.main.int & 0x1) as u32) << (u32::BITS - 1)),
            },
            acc: (self.acc >> 1) | (((self.main.frac & 0x1) as u64) << (u64::BITS - 1)),
        }
    }
}

impl From<FractIndex> for ExtFractIndex {
    fn from(value: FractIndex) -> Self {
        ExtFractIndex {
            main: value,
            acc: 0,
        }
    }
}

pub(crate) fn generate_weights<T>(
    function: ResamplingFunction,
    in_start: FractIndex,
    in_end: FractIndex,
    in_size: usize,
    out_size: usize,
) -> FilterWeights<T>
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
        + AsPrimitive<i64>
        + AsPrimitive<usize>
        + Jinc<T>
        + ConstSqrt2
        + Default
        + AsPrimitive<i32>
        + Div<T, Output = T>
        + Debug,
    f32: AsPrimitive<T>,
    f64: AsPrimitive<T>,
    i64: AsPrimitive<T>,
    i32: AsPrimitive<T>,
    u32: AsPrimitive<T>,
    usize: AsPrimitive<T>,
{
    assert!(in_start < in_end);
    assert!(in_end <= FractIndex::from(in_size));

    // Do index calculations in fixed point, so that they behave essentially the same
    // at the left and the right sides of the image.
    let fixed_sub = |from: FractIndex, to: FractIndex| -> FractIndex {
        let (x, overflow) = to.frac.overflowing_sub(from.frac);
        FractIndex {
            int: to.int - from.int - overflow as usize,
            frac: x,
        }
    };

    let resampling_filter = function.get_resampling_filter();

    let e32: T = (1u32 << 16).as_() * (1u32 << 16).as_();
    let de32: T = T::one() / e32;
    let fixed_to_float = |x: FractIndex| -> T { x.int.as_() + de32 * x.frac.as_() };

    let in_width = fixed_sub(in_start, in_end);

    let scale = fixed_to_float(in_width) / out_size.as_();
    // Note that this a) rounds down, so multiplying by out_size won't overflow
    // b) has 1/(1<<(u32::BITS + usize::BITS)) error, so adding together `out_size`
    // steps will have < 1/(1<<(u32::BITS)) error at the end
    let in_step = ExtFractIndex::from(in_width).div(out_size);
    let half_step = in_step.div2();

    let is_resizable_kernel = resampling_filter.is_resizable_kernel;
    let filter_scale_cutoff = match is_resizable_kernel {
        true => scale.max(1f32.as_()),
        false => 1f32.as_(),
    };
    let filter_base_size = resampling_filter.min_kernel_size * 2.;
    let resampling_function = resampling_filter.kernel;
    let window_func = resampling_filter.window;

    let mut bounds: Vec<FilterBounds> = vec![FilterBounds::new(0, 0); out_size];

    let is_area = resampling_filter.is_area_filter && scale < 1.as_();

    if !is_area {
        let base_size: usize = (filter_base_size.as_() * filter_scale_cutoff).round().as_();
        let kernel_size = base_size;
        let filter_radius = (base_size.as_() / 2.as_()).max(0.5.as_());
        let (filter_radius_int, filter_radius_fract): (usize, u32) = (
            filter_radius.floor().as_(),
            AsPrimitive::<usize>::as_(filter_radius.fract() * e32).min(u32::MAX as usize) as u32,
        );
        let filter_scale = 1f32.as_() / filter_scale_cutoff;
        let mut weights: Vec<T> = vec![T::default(); kernel_size * out_size];
        let mut local_filters = vec![T::default(); kernel_size];
        let mut filter_position = 0usize;
        let blur_scale = match window_func {
            None => 1f32.as_(),
            Some(window) => {
                if window.blur.as_() > 0f32.as_() {
                    1f32.as_() / window.blur.as_()
                } else {
                    0f32.as_()
                }
            }
        };

        let mut center_acc = ExtFractIndex::from(in_start).add(half_step);
        for bound in bounds.iter_mut() {
            let center = center_acc.round();
            center_acc = center_acc.add(in_step);

            let mut weights_sum: T = 0f32.as_();

            let mut start: usize = center.int.saturating_sub(filter_radius_int);
            if filter_radius_fract > center.frac {
                start = start.saturating_sub(1);
            }
            let (ec, carry) = filter_radius_fract.overflowing_add(center.frac);
            let mut end: usize = center.int.saturating_add(filter_radius_int);
            end = end.saturating_add((ec > 0) as usize + carry as usize);
            end = end.min(in_size).min(start.saturating_add(kernel_size));

            for (local_filter_iteration, k) in (start..end).enumerate() {
                let mut dx_int = k.abs_diff(center.int).as_();
                if k < center.int {
                    dx_int = -dx_int;
                }
                let mut dx_frac = center.frac.abs_diff(0x80000000).as_() * de32;
                if center.frac > 0x80000000 {
                    dx_frac = -dx_frac;
                };

                let dx = dx_int + dx_frac;

                let weight;
                if let Some(resampling_window) = window_func {
                    let mut x = dx.abs();
                    x = if resampling_window.blur.as_() > 0f32.as_() {
                        x * blur_scale
                    } else {
                        x
                    };
                    x = if x <= resampling_window.taper.as_() {
                        0f32.as_()
                    } else {
                        (x - resampling_window.taper.as_())
                            / (1f32.as_() - resampling_window.taper.as_())
                    };
                    let window_producer = resampling_window.window;
                    let x_kernel_scaled = x * filter_scale;
                    let window = if x < resampling_window.window_size.as_() {
                        window_producer(x_kernel_scaled * resampling_window.window_size.as_())
                    } else {
                        0f32.as_()
                    };
                    weight = window * resampling_function(x_kernel_scaled);
                } else {
                    let dx = dx.abs();
                    weight = resampling_function(dx * filter_scale);
                }
                weights_sum += weight;
                local_filters[local_filter_iteration] = weight;
            }

            let size = end - start;

            *bound = FilterBounds::new(start, size);

            if weights_sum != 0f32.as_() {
                let recpeq = 1f32.as_() / weights_sum;

                for (dst, src) in weights
                    .iter_mut()
                    .skip(filter_position)
                    .take(size)
                    .zip(local_filters.iter().take(size))
                {
                    *dst = *src * recpeq;
                }
            } else if size > 0 {
                for dst in weights.iter_mut().skip(filter_position).take(size) {
                    *dst = 0f32.as_();
                }
                let center_tap = center.int - start;
                weights[filter_position + center_tap.min(size - 1)] = 1f32.as_();
            }

            filter_position += kernel_size;
        }

        FilterWeights::<T>::new(
            weights,
            kernel_size,
            kernel_size,
            out_size,
            filter_radius.as_(),
            bounds,
        )
    } else {
        // Simulating INTER_AREA from OpenCV, for up scaling here,
        // this is necessary because weight computation is different
        // from any other func
        let inv_scale: T = 1.as_() / scale;
        let kernel_size = 2;
        let filter_radius: T = 1.as_();
        let mut weights: Vec<T> = vec![T::default(); kernel_size * out_size];
        let mut local_filters = vec![T::default(); kernel_size];
        let mut filter_position = 0usize;

        // Example:
        //
        //       s_k           s_{k+1}          s_{k+2}
        //        |               |               |
        // In *------------------------*------------------------*
        //              j=h                      j=h+1
        // Out    +---------------+---------------+
        //               i=k             i=k+1
        //
        //        w_{k,h} = 1       w_{k+1,h} =
        //                            ratio(ceil(s_{k+1}) - s_{k+1},
        //                              s_{k+2} - floor(s_{k+2}))
        //                            = (ceil(s_{k+1}) - s_{k+1})
        //                                 / (s_{k+2} - s_{k+1})
        //                          w_{k+1,h+1} = 1 - w_{k+1,h}
        //
        //  where ratio(x,y) = x/(x+y)
        //

        let mut pos_acc = ExtFractIndex::from(in_start);

        for bound in bounds.iter_mut() {
            let s_0 = pos_acc.round();
            pos_acc = pos_acc.add(in_step);
            let s_1 = pos_acc.round();

            let mut weights_sum: T = 0f32.as_();

            let (start, mut end, weight0, weight1) = if s_0.int == s_1.int || s_1.frac == 0 {
                (s_0.int, s_0.int + 1, T::one(), T::zero())
            } else {
                let r = if s_1.int == s_0.int + 1 {
                    s_1.frac.as_() * de32
                } else {
                    1.as_()
                };
                let w1 = (r * inv_scale).min(1.as_());
                let w0 = 1.as_() - w1;
                (s_0.int, s_0.int + 2, w0, w1)
            };

            local_filters[0] = weight0;
            local_filters[1] = weight1;

            end = end.min(in_size);
            let size = end - start;

            if size == 0 {
                assert!(start == in_size);
                *bound = FilterBounds::new(in_size - 1, 1);
                weights[filter_position] = 1.as_();
            } else {
                weights_sum += weight0;
                if size > 1 {
                    weights_sum += weight1;
                }
                *bound = FilterBounds::new(start, size);

                if weights_sum != 0f32.as_() {
                    let recpeq = 1f32.as_() / weights_sum;

                    for (dst, src) in weights
                        .iter_mut()
                        .skip(filter_position)
                        .take(size)
                        .zip(local_filters.iter().take(size))
                    {
                        *dst = *src * recpeq;
                    }
                } else {
                    weights[filter_position] = 1.as_();
                }
            }

            filter_position += kernel_size;
        }

        FilterWeights::new(
            weights,
            kernel_size,
            kernel_size,
            out_size,
            filter_radius.as_(),
            bounds,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_window_has_weights_summing_to_one() {
        let functions = [
            ResamplingFunction::Bilinear,
            ResamplingFunction::Lanczos3,
            ResamplingFunction::Gaussian,
            ResamplingFunction::Bartlett,
            ResamplingFunction::BartlettHann,
            ResamplingFunction::Lanczos3Jinc,
        ];
        for function in functions {
            for (in_size, out_size) in [(100usize, 37usize), (37, 100), (256, 256), (5, 1)] {
                let filter = generate_weights::<f64>(
                    function,
                    FractIndex::from(0),
                    FractIndex::from(in_size),
                    in_size,
                    out_size,
                );
                for (i, chunk) in filter
                    .weights
                    .chunks_exact(filter.kernel_size)
                    .enumerate()
                    .take(out_size)
                {
                    let sum: f64 = chunk.iter().sum();
                    assert!(
                        (sum - 1.0).abs() < 1e-4,
                        "{function:?} {in_size}->{out_size}: window {i} sums to {sum}"
                    );
                }
            }
        }
    }
}
