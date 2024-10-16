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
use crate::compute_weights::generate_weights;
use crate::fixed_point_dispatch::{convolve_column_fixed_point, convolve_row_fixed_point};
use crate::handler_provider::{ColumnHandlerFixedPoint, RowHandlerFixedPoint};
use crate::image_size::ImageSize;
use crate::resize_nearest::resize_nearest;
use crate::sampler::ResamplingFunction;
use crate::saturate_narrow::SaturateNarrow;
use num_traits::AsPrimitive;
use std::ops::{AddAssign, Mul};
use std::time::Instant;

pub fn resize_fixed_point<T, J, const CHANNELS: usize>(
    src: &[T],
    source_size: ImageSize,
    destination_size: ImageSize,
    bit_depth: u32,
    resampling_function: ResamplingFunction,
) -> Result<Vec<T>, String>
where
    T: Copy
        + 'static
        + AsPrimitive<J>
        + Default
        + ColumnHandlerFixedPoint<T, J>
        + RowHandlerFixedPoint<T, J>
        + Send
        + Sync,
    J: Copy + 'static + AsPrimitive<T> + Mul<Output = J> + AddAssign + SaturateNarrow<T> + Default,
    i32: AsPrimitive<J>,
    i16: AsPrimitive<J>,
{
    assert!(
        CHANNELS <= 4,
        "Images with more than 4 channels is not supported"
    );
    assert_ne!(CHANNELS, 0, "Invalid count of channels");
    if src.len() != source_size.width * CHANNELS * source_size.height {
        return Err(format!(
            "Source slice size must be width * channels * height ({}) but got {}",
            source_size.width * CHANNELS * source_size.height,
            src.len(),
        ));
    }
    let (_, is_stride_overflowed) = source_size.width.overflowing_mul(CHANNELS);
    if is_stride_overflowed {
        return Err("Stride must never exceed usize::MAX".parse().unwrap());
    }
    let (_, is_stride_overflowed) = destination_size.width.overflowing_mul(CHANNELS);
    if is_stride_overflowed {
        return Err("Stride must never exceed usize::MAX".parse().unwrap());
    }

    if source_size.width == destination_size.width && source_size.height == destination_size.height
    {
        return Ok(src.to_vec());
    }

    if resampling_function == ResamplingFunction::Nearest {
        let mut store =
            vec![T::default(); destination_size.width * destination_size.height * CHANNELS];
        resize_nearest::<T, CHANNELS>(
            src,
            source_size.width,
            source_size.height,
            &mut store,
            destination_size.width,
            destination_size.height,
        );

        assert_eq!(
            store.len(),
            destination_size.width * destination_size.height * CHANNELS,
            "The resized image must always have valid target dimensions"
        );

        return Ok(store);
    }

    let start = Instant::now();

    let mut working_slice_size = source_size;
    let mut working_slice_ref = src;

    let mut transient = vec![];

    if working_slice_size.height != destination_size.height {
        let vertical_filters = generate_weights::<f32>(
            resampling_function,
            working_slice_size.height,
            destination_size.height,
        );

        transient =
            vec![T::default(); working_slice_size.width * destination_size.height * CHANNELS];

        let new_vertical_size = ImageSize::new(working_slice_size.width, destination_size.height);

        convolve_column_fixed_point::<T, J, CHANNELS>(
            working_slice_ref,
            working_slice_size,
            vertical_filters,
            &mut transient,
            new_vertical_size,
            bit_depth,
        );

        working_slice_size = new_vertical_size;
        working_slice_ref = &transient;
    }

    println!("Vertical time {:?}", start.elapsed());

    let start = Instant::now();

    if working_slice_size.width != destination_size.width {
        let vertical_filters = generate_weights::<f32>(
            resampling_function,
            working_slice_size.width,
            destination_size.width,
        );

        let mut transient2 =
            vec![T::default(); destination_size.width * working_slice_size.height * CHANNELS];

        let new_vertical_size = ImageSize::new(destination_size.width, working_slice_size.height);

        convolve_row_fixed_point::<T, J, CHANNELS>(
            working_slice_ref,
            working_slice_size,
            vertical_filters,
            &mut transient2,
            new_vertical_size,
            bit_depth,
        );

        transient = transient2;
    }

    println!("Horizontal time {:?}", start.elapsed());

    assert_eq!(
        transient.len(),
        destination_size.width * destination_size.height * CHANNELS,
        "The resized image must always have valid target dimensions"
    );

    Ok(transient)
}
