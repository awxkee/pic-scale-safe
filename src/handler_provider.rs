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
use crate::fixed_point_horizontal::{
    convolve_row_handler_fixed_point, convolve_row_handler_fixed_point_4,
};
use crate::fixed_point_vertical::column_handler_fixed_point;
use crate::floating_point_horizontal::{
    convolve_row_handler_floating_point, convolve_row_handler_floating_point_4,
};
use crate::floating_point_vertical::column_handler_floating_point;
use crate::mixed_storage::MixedStorage;
use crate::saturate_narrow::SaturateNarrow;
use num_traits::{AsPrimitive, Float, MulAdd};
use std::ops::{Add, AddAssign, Mul};

pub trait ColumnHandlerFixedPoint<T, J>
where
    T: Copy + 'static + AsPrimitive<J> + Default,
    J: Copy + 'static + AsPrimitive<T> + Mul<Output = J> + AddAssign + SaturateNarrow<T> + Default,
    i32: AsPrimitive<J>,
    i16: AsPrimitive<J>,
{
    fn handle_column<const COMPONENTS: usize>(
        dst_width: usize,
        bounds: &FilterBounds,
        src: &[T],
        dst: &mut [T],
        src_stride: usize,
        weight: &[i16],
        bit_depth: u32,
    );
}

pub trait RowHandlerFixedPoint<T, J>
where
    T: Copy + 'static + AsPrimitive<J> + Default,
    J: Copy + 'static + AsPrimitive<T> + Mul<Output = J> + AddAssign + SaturateNarrow<T> + Default,
    i32: AsPrimitive<J>,
    i16: AsPrimitive<J>,
{
    fn handle_row_4<const COMPONENTS: usize>(
        src: &[T],
        src_stride: usize,
        dst: &mut [T],
        dst_stride: usize,
        filter_weights: &FilterWeights<i16>,
        bit_depth: u32,
    );

    fn handle_row<const COMPONENTS: usize>(
        src: &[T],
        dst: &mut [T],
        filter_weights: &FilterWeights<i16>,
        bit_depth: u32,
    );
}

impl<J> RowHandlerFixedPoint<u8, J> for u8
where
    J: Copy
        + 'static
        + AsPrimitive<u8>
        + Mul<Output = J>
        + AddAssign
        + SaturateNarrow<u8>
        + Add<J, Output = J>
        + Default,
    i32: AsPrimitive<J>,
    u8: AsPrimitive<J>,
    i16: AsPrimitive<J>,
{
    fn handle_row_4<const COMPONENTS: usize>(
        src: &[u8],
        src_stride: usize,
        dst: &mut [u8],
        dst_stride: usize,
        filter_weights: &FilterWeights<i16>,
        bit_depth: u32,
    ) {
        convolve_row_handler_fixed_point_4::<u8, J, COMPONENTS>(
            src,
            src_stride,
            dst,
            dst_stride,
            filter_weights,
            bit_depth,
        )
    }

    fn handle_row<const COMPONENTS: usize>(
        src: &[u8],
        dst: &mut [u8],
        filter_weights: &FilterWeights<i16>,
        bit_depth: u32,
    ) {
        convolve_row_handler_fixed_point::<u8, J, COMPONENTS>(src, dst, filter_weights, bit_depth)
    }
}

impl<J> RowHandlerFixedPoint<u16, J> for u16
where
    J: Copy
        + 'static
        + AsPrimitive<u16>
        + Mul<Output = J>
        + AddAssign
        + SaturateNarrow<u16>
        + Add<J, Output = J>
        + Default,
    i32: AsPrimitive<J>,
    u16: AsPrimitive<J>,
    i16: AsPrimitive<J>,
{
    fn handle_row_4<const COMPONENTS: usize>(
        src: &[u16],
        src_stride: usize,
        dst: &mut [u16],
        dst_stride: usize,
        filter_weights: &FilterWeights<i16>,
        bit_depth: u32,
    ) {
        convolve_row_handler_fixed_point_4::<u16, J, COMPONENTS>(
            src,
            src_stride,
            dst,
            dst_stride,
            filter_weights,
            bit_depth,
        )
    }

    fn handle_row<const COMPONENTS: usize>(
        src: &[u16],
        dst: &mut [u16],
        filter_weights: &FilterWeights<i16>,
        bit_depth: u32,
    ) {
        convolve_row_handler_fixed_point::<u16, J, COMPONENTS>(src, dst, filter_weights, bit_depth)
    }
}

impl<J> ColumnHandlerFixedPoint<u8, J> for u8
where
    J: Copy
        + 'static
        + AsPrimitive<u8>
        + Mul<Output = J>
        + AddAssign
        + SaturateNarrow<u8>
        + Default,
    i32: AsPrimitive<J>,
    i16: AsPrimitive<J>,
    u8: AsPrimitive<J>,
{
    fn handle_column<const COMPONENTS: usize>(
        dst_width: usize,
        bounds: &FilterBounds,
        src: &[u8],
        dst: &mut [u8],
        src_stride: usize,
        weight: &[i16],
        bit_depth: u32,
    ) {
        column_handler_fixed_point::<u8, J, COMPONENTS>(
            dst_width, bounds, src, dst, src_stride, weight, bit_depth,
        );
    }
}

impl<J> ColumnHandlerFixedPoint<u16, J> for u16
where
    J: Copy
        + 'static
        + AsPrimitive<u16>
        + Mul<Output = J>
        + AddAssign
        + SaturateNarrow<u16>
        + Default,
    i32: AsPrimitive<J>,
    i16: AsPrimitive<J>,
    u16: AsPrimitive<J>,
{
    fn handle_column<const COMPONENTS: usize>(
        dst_width: usize,
        bounds: &FilterBounds,
        src: &[u16],
        dst: &mut [u16],
        src_stride: usize,
        weight: &[i16],
        bit_depth: u32,
    ) {
        column_handler_fixed_point::<u16, J, COMPONENTS>(
            dst_width, bounds, src, dst, src_stride, weight, bit_depth,
        );
    }
}

pub trait ColumnHandlerFloatingPoint<T, J, F>
where
    T: Copy + 'static + AsPrimitive<J> + Default,
    J: Copy + 'static + AsPrimitive<T> + MulAdd<J, Output = J> + Default + MixedStorage<T>,
    F: Copy + 'static + AsPrimitive<J>,
    i32: AsPrimitive<J>,
    f32: AsPrimitive<J>,
{
    fn handle_column<const COMPONENTS: usize>(
        dst_width: usize,
        bounds: &FilterBounds,
        src: &[T],
        dst: &mut [T],
        src_stride: usize,
        weight: &[F],
        bit_depth: u32,
    );
}

macro_rules! default_floating_column_handler {
    ($column_type:ty) => {
        impl<J, F> ColumnHandlerFloatingPoint<$column_type, J, F> for $column_type
        where
            J: Copy
                + 'static
                + AsPrimitive<$column_type>
                + MulAdd<J, Output = J>
                + MixedStorage<$column_type>
                + Default
                + Mul<J, Output = J>
                + Add<J, Output = J>,
            F: Copy + 'static + Float + AsPrimitive<J>,
            i32: AsPrimitive<J>,
            $column_type: AsPrimitive<J>,
            f32: AsPrimitive<J>,
        {
            fn handle_column<const COMPONENTS: usize>(
                dst_width: usize,
                bounds: &FilterBounds,
                src: &[$column_type],
                dst: &mut [$column_type],
                src_stride: usize,
                weight: &[F],
                bit_depth: u32,
            ) {
                column_handler_floating_point::<$column_type, J, F, COMPONENTS>(
                    dst_width, bounds, src, dst, src_stride, weight, bit_depth,
                )
            }
        }
    };
}

default_floating_column_handler!(u8);
default_floating_column_handler!(u16);
default_floating_column_handler!(u32);
default_floating_column_handler!(f32);
default_floating_column_handler!(f64);

pub trait RowHandlerFloatingPoint<T, J, F>
where
    T: Copy + 'static + AsPrimitive<J> + Default,
    J: Copy + 'static + AsPrimitive<T> + MulAdd<J, Output = J> + Default + MixedStorage<T>,
    F: Copy + 'static + AsPrimitive<J>,
    i32: AsPrimitive<J>,
    f32: AsPrimitive<J>,
{
    fn handle_row_4<const COMPONENTS: usize>(
        src: &[T],
        src_stride: usize,
        dst: &mut [T],
        dst_stride: usize,
        filter_weights: &FilterWeights<F>,
        bit_depth: u32,
    );

    fn handle_row<const COMPONENTS: usize>(
        src: &[T],
        dst: &mut [T],
        filter_weights: &FilterWeights<F>,
        bit_depth: u32,
    );
}

macro_rules! default_floating_column_handler {
    ($row_type:ty) => {
        impl<J, F> RowHandlerFloatingPoint<$row_type, J, F> for $row_type
        where
            J: Copy
                + 'static
                + AsPrimitive<$row_type>
                + MulAdd<J, Output = J>
                + Default
                + MixedStorage<$row_type>
                + Mul<J, Output = J>
                + Add<J, Output = J>,
            F: Copy + 'static + AsPrimitive<J> + Float,
            i32: AsPrimitive<J>,
            f32: AsPrimitive<J>,
            $row_type: AsPrimitive<J>,
        {
            fn handle_row_4<const COMPONENTS: usize>(
                src: &[$row_type],
                src_stride: usize,
                dst: &mut [$row_type],
                dst_stride: usize,
                filter_weights: &FilterWeights<F>,
                bit_depth: u32,
            ) {
                convolve_row_handler_floating_point_4::<$row_type, J, F, COMPONENTS>(
                    src,
                    src_stride,
                    dst,
                    dst_stride,
                    filter_weights,
                    bit_depth,
                )
            }

            fn handle_row<const COMPONENTS: usize>(
                src: &[$row_type],
                dst: &mut [$row_type],
                filter_weights: &FilterWeights<F>,
                bit_depth: u32,
            ) {
                convolve_row_handler_floating_point::<$row_type, J, F, COMPONENTS>(
                    src,
                    dst,
                    filter_weights,
                    bit_depth,
                )
            }
        }
    };
}

default_floating_column_handler!(f32);
default_floating_column_handler!(f64);
default_floating_column_handler!(u8);
default_floating_column_handler!(u16);
