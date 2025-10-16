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

#![forbid(unsafe_code)]
#![allow(clippy::manual_clamp)]
#![deny(dead_code, unreachable_pub)]

mod alpha;
mod alpha_check;
mod color_group;
mod compute_weights;
mod definitions;
mod filter_weights;
mod fixed_point_dispatch;
mod fixed_point_horizontal;
mod fixed_point_vertical;
mod floating_point_dispatch;
mod floating_point_horizontal;
mod floating_point_vertical;
mod handler_provider;
mod image_size;
mod math;
mod mixed_storage;
mod mlaf;
mod resize_fixed_point;
mod resize_floating_point;
mod resize_nearest;
mod resizer;
mod sampler;
mod saturate_narrow;
mod trc;
mod trc_handler;

pub use alpha::*;
pub use alpha_check::{
    has_non_constant_alpha_la16, has_non_constant_alpha_la8, has_non_constant_alpha_luma_alpha_f32,
    has_non_constant_alpha_rgba16, has_non_constant_alpha_rgba8, has_non_constant_alpha_rgba_f32,
};
pub use image_size::ImageSize;
pub use resizer::*;
pub use sampler::ResamplingFunction;
pub use trc::*;
pub use trc_handler::*;
