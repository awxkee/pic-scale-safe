//! Confirm that a rescaled image roughly matches the expected value.
//!
//! The design of the library and linearity of scaling operations makes it
//! unlikely that there are any scaling bugs which depend on the precise content
//! of the image, so this fuzz target does not expose much flexibility in the
//! image content.
//!
//! Instead, it rescales only a simple class of rendered images. Using rendered
//! images instead of arbitrary ones also makes it practical to render a
//! "perfectly" scaled reference image and thus check that the rescaled output
//! looks similar enough to the target.
//!
//! This will not be able to catch fine grained errors or subtle bugs in
//! specific filters.
#![no_main]

use std::{
    fs::File,
    path::{Path, PathBuf},
};

use arbitrary::Arbitrary;
use libfuzzer_sys::{fuzz_target, Corpus};

use pic_scale_safe::{CropRect, FractIndex, ImageSize, ResamplingFunction};
use tiff::encoder::{
    colortype::{self, ColorType},
    compression::DeflateLevel,
    Compression, TiffValue,
};

#[derive(Debug, Arbitrary, Clone, Copy)]
#[repr(transparent)]
struct Color([f32; 4]);

impl Color {
    fn premultiply(&mut self) {
        self.0[0] *= self.0[3];
        self.0[1] *= self.0[3];
        self.0[2] *= self.0[3];
    }
}

/// (x,y): x * cx + y * cy + b >= 0
#[derive(Debug, Arbitrary, Clone, Copy)]
struct Halfspace {
    /// +.127
    cx: i128,
    /// +.127
    cy: i128,
    /// 68.60
    b: i128,
}

/// 64.64 * +.127 -> 68.60
fn strict_i64d64_mul(x: u128, y: i128) -> i128 {
    let (_lo, hi) = x.carrying_mul(y.unsigned_abs(), 0);

    let v = hi >> 3;

    if y < 0 {
        -(v as i128)
    } else {
        v as i128
    }
}

fn render_halfspace<F: FnMut(Color, usize, usize)>(
    space: Halfspace,
    view_x: (u128, u128),
    view_y: (u128, u128),
    size: (usize, usize),
    color_n: Color,
    color_p: Color,
    mut blit: F,
) {
    let (width, height) = size;

    let xbase = strict_i64d64_mul(view_x.0, space.cx);
    // The division by `width` is not perfectly accurate, but should be close enough that any
    // rendering errors are below the threshold
    let xstep = strict_i64d64_mul(view_x.1.saturating_sub(view_x.0) / width as u128, space.cx);

    let ybase = strict_i64d64_mul(view_y.0, space.cy);
    let ystep = strict_i64d64_mul(view_y.1.saturating_sub(view_y.0) / height as u128, space.cy);

    // This is quite inefficient, because only a few pixels fall on the boundary. Quadtree
    // filtering may be faster and may naturally reduce interpolation rounding error
    let mut vy = ybase;
    for y in 0..height {
        let vy0 = vy;
        vy += ystep;
        let vy1 = vy;

        let mut vx = xbase;
        for x in 0..width {
            let vx0 = vx;
            vx += xstep;
            let vx1 = vx;

            let p00 = vx0.strict_add(vy0).strict_add(space.b) as f32;
            let p01 = vx0.strict_add(vy1).strict_add(space.b) as f32;
            let p10 = vx1.strict_add(vy0).strict_add(space.b) as f32;
            let p11 = vx1.strict_add(vy1).strict_add(space.b) as f32;

            let (color_00, anti_00) = if p00 >= 0.0 {
                (color_p, color_n)
            } else {
                (color_n, color_p)
            };

            let blend = |a: Color, b: Color, f: f32| -> Color {
                // This formula is only reasonable accurate for a,b in [0,1]
                Color([
                    f * a.0[0] + (1.0 - f) * b.0[0],
                    f * a.0[1] + (1.0 - f) * b.0[1],
                    f * a.0[2] + (1.0 - f) * b.0[2],
                    f * a.0[3] + (1.0 - f) * b.0[3],
                ])
            };

            // p00 p10
            // p01 p11
            let color = match (p00 >= 0.0, p01 >= 0.0, p10 >= 0.0, p11 >= 0.0) {
                (false, false, false, false) | (true, true, true, true) => {
                    // uniform, most common case by far
                    color_00
                }

                (true, true, false, false) | (false, false, true, true) => {
                    // vertical cut
                    let cut0 = p00.abs() / (p00.abs() + p10.abs());
                    let cut1 = p01.abs() / (p01.abs() + p11.abs());
                    blend(color_00, anti_00, (cut0 + cut1) / 2.0)
                }
                (true, false, true, false) | (false, true, false, true) => {
                    // horizontal cut
                    let cut0 = p00.abs() / (p00.abs() + p01.abs());
                    let cut1 = p10.abs() / (p10.abs() + p11.abs());
                    blend(color_00, anti_00, (cut0 + cut1) / 2.0)
                }

                (true, false, false, false) | (false, true, true, true) => {
                    // top left corner
                    let cuty = p00.abs() / (p00.abs() + p01.abs());
                    let cutx = p00.abs() / (p00.abs() + p10.abs());
                    blend(color_00, anti_00, cutx * cuty * 0.5)
                }
                (false, false, true, false) | (true, true, false, true) => {
                    // top right corner
                    let cuty = p10.abs() / (p10.abs() + p00.abs());
                    let cutx = p10.abs() / (p10.abs() + p11.abs());
                    blend(color_00, anti_00, 1.0 - cutx * cuty * 0.5)
                }
                (false, false, false, true) | (true, true, true, false) => {
                    // bottom right corner
                    let cuty = p11.abs() / (p11.abs() + p10.abs());
                    let cutx = p11.abs() / (p11.abs() + p01.abs());
                    blend(color_00, anti_00, 1.0 - cutx * cuty * 0.5)
                }
                (false, true, false, false) | (true, false, true, true) => {
                    // bottom left corner
                    let cuty = p01.abs() / (p01.abs() + p00.abs());
                    let cutx = p01.abs() / (p01.abs() + p11.abs());
                    blend(color_00, anti_00, 1.0 - cutx * cuty * 0.5)
                }
                (true, false, false, true) | (false, true, true, false) => unreachable!(),
            };
            blit(color, x, y);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render<T: Copy, G: Fn(f32) -> T>(
    dst: &mut [T],
    space: Halfspace,
    view_x: (u128, u128),
    view_y: (u128, u128),
    size: (usize, usize),
    color_n: Color,
    color_p: Color,
    channels: ChannelCount,
    conv: G,
) {
    let (width, _height) = size;
    match channels {
        ChannelCount::P => render_halfspace(
            space,
            view_x,
            view_y,
            size,
            color_n,
            color_p,
            |c: Color, x: usize, y: usize| {
                dst[y * width + x] = conv(c.0[0]);
            },
        ),
        ChannelCount::LA => render_halfspace(
            space,
            view_x,
            view_y,
            size,
            color_n,
            color_p,
            |c: Color, x: usize, y: usize| {
                dst[2 * y * width + 2 * x] = conv(c.0[0]);
                dst[2 * y * width + 2 * x + 1] = conv(c.0[3]);
            },
        ),
        ChannelCount::RGB => render_halfspace(
            space,
            view_x,
            view_y,
            size,
            color_n,
            color_p,
            |c: Color, x: usize, y: usize| {
                dst[3 * y * width + 3 * x] = conv(c.0[0]);
                dst[3 * y * width + 3 * x + 1] = conv(c.0[1]);
                dst[3 * y * width + 3 * x + 2] = conv(c.0[2]);
            },
        ),
        ChannelCount::RGBA => render_halfspace(
            space,
            view_x,
            view_y,
            size,
            color_n,
            color_p,
            |c: Color, x: usize, y: usize| {
                dst[4 * y * width + 4 * x] = conv(c.0[0]);
                dst[4 * y * width + 4 * x + 1] = conv(c.0[1]);
                dst[4 * y * width + 4 * x + 2] = conv(c.0[2]);
                dst[4 * y * width + 4 * x + 3] = conv(c.0[3]);
            },
        ),
    }
}

#[derive(Debug, Arbitrary, Clone, Copy)]
enum ChannelType {
    F32,
    U8,
    U16,
}

impl ChannelType {
    fn bpc(&self) -> u8 {
        match self {
            ChannelType::U8 => 1,
            ChannelType::U16 => 2,
            ChannelType::F32 => 4,
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Arbitrary, Clone, Copy)]
enum ChannelCount {
    P,
    LA,
    RGB,
    RGBA,
}

impl ChannelCount {
    fn channels(&self) -> u8 {
        match self {
            ChannelCount::P => 1,
            ChannelCount::LA => 2,
            ChannelCount::RGB => 3,
            ChannelCount::RGBA => 4,
        }
    }
}

#[derive(Debug, Arbitrary)]
struct FuzzCase {
    channel_type: ChannelType,
    channel_count: ChannelCount,
    actual_bits: u8, /* For u16 */

    src_size: (u64, u64),

    dst_size: (u64, u64),
    view_x: ((u64, u32), (u64, u32)),
    view_y: ((u64, u32), (u64, u32)),

    filter: u32,

    space: Halfspace,
    /** unpremultiplied */
    color_p: Color,
    /** unpremultiplied */
    color_n: Color,
}

impl FuzzCase {
    fn crop_rect(&self) -> CropRect {
        CropRect {
            x1: FractIndex {
                int: (self.view_x.0 .0 % self.src_size.0.saturating_add(1)) as usize,
                frac: self.view_x.0 .1,
            },
            x2: FractIndex {
                int: (self.view_x.1 .0 % self.src_size.0.saturating_add(1)) as usize,
                frac: self.view_x.1 .1,
            },
            y1: FractIndex {
                int: (self.view_y.0 .0 % self.src_size.1.saturating_add(1)) as usize,
                frac: self.view_y.0 .1,
            },
            y2: FractIndex {
                int: (self.view_y.1 .0 % self.src_size.1.saturating_add(1)) as usize,
                frac: self.view_y.1 .1,
            },
        }
    }
}

fn la_to_rgb<T: Copy>(x: &[T]) -> Vec<T> {
    x.chunks(2).flat_map(|c| [c[0], c[0], c[0], c[1]]).collect()
}

fn log_image<T, C>(path: &Path, data: &[T], size: (u64, u64))
where
    C: ColorType<Inner = T>,
    [T]: TiffValue,
{
    let Ok(writer) = File::create(path) else {
        panic!("Failed to write to {:?}", path);
    };
    let (Ok(w), Ok(h)) = (u32::try_from(size.0), u32::try_from(size.1)) else {
        return;
    };

    let mut enc = tiff::encoder::TiffEncoder::new_big(writer)
        .unwrap()
        .with_compression(Compression::Deflate(DeflateLevel::Balanced));

    enc.write_image::<C>(w, h, data)
        .expect("failed to save image");
}

#[allow(clippy::type_complexity)]
fn test<T: Default + Copy + Into<f32>, G: Fn(f32) -> T + Copy, H: Fn(&Path, &[T], (u64, u64))>(
    data: &FuzzCase,
    max: f32,
    scale_fn: &dyn Fn(&[T]) -> Result<Vec<T>, String>,
    conv: G,
    write: H,
) {
    let mut src = vec![
        T::default();
        usize::try_from(
            data.src_size
                .0
                .strict_mul(data.src_size.1)
                .strict_mul(data.channel_count.channels() as _)
        )
        .unwrap()
    ];
    // Technically this could be materialized on demand'
    let mut com = vec![
        T::default();
        usize::try_from(
            data.dst_size
                .0
                .strict_mul(data.dst_size.1)
                .strict_mul(data.channel_count.channels() as _)
        )
        .unwrap()
    ];

    let s64_32_to_s64_64 =
        |x: FractIndex| -> u128 { (u128::from(x.int as u64) << 64) | (u128::from(x.frac) << 32) };
    let u64_to_s64_64 = |x: u64| -> u128 { u128::from(x) << 64 };

    let view = data.crop_rect();

    let vx = (s64_32_to_s64_64(view.x1), s64_32_to_s64_64(view.x2));
    let vy = (s64_32_to_s64_64(view.y1), s64_32_to_s64_64(view.y2));

    let mut color_n = data.color_n;
    let mut color_p = data.color_p;
    color_n.premultiply();
    color_p.premultiply();

    if std::env::var_os("FUZZ_NO_COLOR").is_some() {
        color_n = Color([0.0, 0.0, 0.0, 0.0]);
        color_p = Color([1.0, 1.0, 1.0, 1.0]);
    }

    if !src.is_empty() {
        render(
            &mut src,
            data.space,
            (0, u64_to_s64_64(data.src_size.0)),
            (0, u64_to_s64_64(data.src_size.1)),
            (data.src_size.0 as usize, data.src_size.1 as usize),
            color_n,
            color_p,
            data.channel_count,
            conv,
        );
    }
    if !com.is_empty() {
        render(
            &mut com,
            data.space,
            vx,
            vy,
            (data.dst_size.0 as usize, data.dst_size.1 as usize),
            color_n,
            color_p,
            data.channel_count,
            conv,
        );
    }

    let Ok(dst) = scale_fn(&src) else {
        // Rejected input, safe
        return;
    };
    assert!(
        data.src_size.0 != 0
            && data.src_size.1 != 0
            && data.dst_size.0 != 0
            && data.dst_size.1 != 0
    );
    assert!(vx.1 >= vx.0 && vy.1 >= vy.0);

    if std::env::var_os("FUZZ_SKIP_ACCURACY").is_some() {
        return;
    }

    let color_disc = color_n
        .0
        .iter()
        .zip(color_p.0.iter())
        .map(|(x, y)| (conv(*x).into() / max - conv(*y).into() / max).abs())
        .sum::<f32>()
        / f32::from(data.channel_count.channels());

    if color_disc < 1e-2 {
        // Very small floating point color differences are hard to perceive, and
        // are easily outweighed by general floating point calculation error
        return;
    }

    let mut total_err = 0u128;
    let scale = 2.0f32.powi(64) / max / f32::from(data.channel_count.channels());
    for (c, d) in com.iter().zip(dst.iter()) {
        let c: f32 = (*c).into();
        let d: f32 = (*d).into();
        // Alpha is premultiplied, so should be remain relatively stable
        total_err += ((c - d).abs() * scale) as u128;
    }
    let total_err = (total_err as f32) / 2.0f32.powi(64);

    // NOTE: this is generally not a tight estimate
    let line_len = data.dst_size.0 as f32 + data.dst_size.1 as f32;

    let view_xf = (vx.1 - vx.0) as f32 / 2.0f32.powi(64);
    let view_yf = (vy.1 - vy.0) as f32 / 2.0f32.powi(64);
    let xscale = data.dst_size.0 as f32 / view_xf;
    let yscale = data.dst_size.1 as f32 / view_yf;

    let linear_error = total_err / line_len;
    // Note: some filters may have stronger guarantees, but `nearest` will always
    // have constant error per line length when scaling down
    let threshold = 1.5 * (xscale.max(1.0) + yscale.max(1.0)) * color_disc;

    if linear_error > threshold {
        let failure_path = std::env::var_os("FUZZ_IMAGE_LOG_PATH");

        if let Some(path) = failure_path {
            let path = PathBuf::from(path);
            write(&path.join("fuzz-src.tiff"), &src, data.src_size);
            write(&path.join("fuzz-com.tiff"), &com, data.dst_size);
            write(&path.join("fuzz-dst.tiff"), &dst, data.dst_size);
        }
        eprintln!("{:?} {:?}", data, view);
        panic!(
            "Scaled and ideal images are too different: linear error {}, threshold {}",
            linear_error, threshold
        );
    }
}

fuzz_target!(|data: FuzzCase| -> Corpus {
    let data = &data;

    let max_size = std::env::var_os("FUZZ_MAX_IMAGE_BYTES")
        .and_then(|x| x.to_str().unwrap().parse::<usize>().ok())
        .unwrap_or(1 << 16);

    let resampling_function = ResamplingFunction::from(data.filter);
    if resampling_function == ResamplingFunction::Bilinear
        && data.filter != ResamplingFunction::Bilinear as u32
    {
        // ResamplingFunction::from maps unknown values to Bilinear
        return Corpus::Reject;
    }

    let color_valid = |x: &Color| -> bool {
        // Note: this accepts -0.0
        (0.0 <= x.0[0] && x.0[0] <= 1.0)
            && (0.0 <= x.0[1] && x.0[1] <= 1.0)
            && (0.0 <= x.0[2] && x.0[2] <= 1.0)
            && (0.0 <= x.0[3] && x.0[3] <= 1.0)
    };
    if !color_valid(&data.color_p) || !color_valid(&data.color_n) {
        return Corpus::Reject;
    }

    if data.actual_bits >= 32 {
        return Corpus::Reject;
    }

    if data.space.b.unsigned_abs() >= 1 << 126
        || data.space.cx.unsigned_abs() >= 1 << 127
        || data.space.cy.unsigned_abs() >= 1 << 127
        || data
            .space
            .cx
            .unsigned_abs()
            .max(data.space.cy.unsigned_abs())
            <= 1 << 125
    {
        return Corpus::Reject;
    }

    let bpp = data.channel_type.bpc() * data.channel_count.channels();

    // if data.view_x.0.0 > data.src_size.0  || data.view_x.1.0 > data.src_size.0 ||
    //     data.view_y.0.0 > data.src_size.0  || data.view_y.1.0 > data.src_size.0 {
    //     return Corpus::Reject;
    // }

    // Accept but skip inputs not valid on 32-bit systems to make corpus
    // transfer easier.
    let (Ok(s0), Ok(s1)) = (
        usize::try_from(data.src_size.0),
        usize::try_from(data.src_size.1),
    ) else {
        return Corpus::Keep;
    };
    let (Ok(d0), Ok(d1)) = (
        usize::try_from(data.dst_size.0),
        usize::try_from(data.dst_size.1),
    ) else {
        return Corpus::Keep;
    };
    let Some(src_bytes) = s0
        .checked_mul(s1)
        .and_then(|x| x.checked_mul(usize::from(bpp)))
    else {
        return Corpus::Keep;
    };
    let Some(dst_bytes) = d0
        .checked_mul(d1)
        .and_then(|x| x.checked_mul(usize::from(bpp)))
    else {
        return Corpus::Keep;
    };

    if src_bytes > max_size || dst_bytes > max_size {
        return Corpus::Keep;
    }

    let source_size = ImageSize::new(s0, s1);
    let dst_size = ImageSize::new(d0, d1);

    let source_rect = data.crop_rect();

    match data.channel_type {
        ChannelType::F32 => {
            test::<f32, _, _>(
                data,
                1.0f32,
                &|input| -> Result<Vec<f32>, String> {
                    match data.channel_count {
                        ChannelCount::P => pic_scale_safe::cropped_resize_plane_f32(
                            input,
                            source_size,
                            source_rect,
                            dst_size,
                            resampling_function,
                        ),
                        ChannelCount::LA => pic_scale_safe::cropped_resize_luma_alpha_f32(
                            input,
                            source_size,
                            source_rect,
                            dst_size,
                            resampling_function,
                        ),
                        ChannelCount::RGB => pic_scale_safe::cropped_resize_rgb_f32(
                            input,
                            source_size,
                            source_rect,
                            dst_size,
                            resampling_function,
                        ),
                        ChannelCount::RGBA => pic_scale_safe::cropped_resize_rgba_f32(
                            input,
                            source_size,
                            source_rect,
                            dst_size,
                            resampling_function,
                        ),
                    }
                },
                |x| x,
                |path: &Path, v: &[f32], size: (u64, u64)| match data.channel_count {
                    ChannelCount::P => log_image::<f32, colortype::Gray32Float>(path, v, size),
                    ChannelCount::LA => {
                        log_image::<f32, colortype::RGBA32Float>(path, &la_to_rgb(v), size)
                    }
                    ChannelCount::RGB => log_image::<f32, colortype::RGB32Float>(path, v, size),
                    ChannelCount::RGBA => log_image::<f32, colortype::RGBA32Float>(path, v, size),
                },
            );
        }
        ChannelType::U16 => {
            let max_val = ((1u32 << data.actual_bits) - 1) as f32;
            test::<u16, _, _>(
                data,
                max_val,
                &|input: &[u16]| -> Result<Vec<u16>, String> {
                    match data.channel_count {
                        ChannelCount::P => pic_scale_safe::cropped_resize_plane16(
                            input,
                            source_size,
                            source_rect,
                            dst_size,
                            u32::from(data.actual_bits),
                            resampling_function,
                        ),
                        ChannelCount::LA => pic_scale_safe::cropped_resize_plane16_with_alpha(
                            input,
                            source_size,
                            source_rect,
                            dst_size,
                            u32::from(data.actual_bits),
                            resampling_function,
                        ),
                        ChannelCount::RGB => pic_scale_safe::cropped_resize_rgb16(
                            input,
                            source_size,
                            source_rect,
                            dst_size,
                            u32::from(data.actual_bits),
                            resampling_function,
                        ),
                        ChannelCount::RGBA => pic_scale_safe::cropped_resize_rgba16(
                            input,
                            source_size,
                            source_rect,
                            dst_size,
                            u32::from(data.actual_bits),
                            resampling_function,
                        ),
                    }
                },
                |x| (x * max_val).round_ties_even() as u16,
                |path: &Path, v: &[u16], size: (u64, u64)| match data.channel_count {
                    ChannelCount::P => log_image::<u16, colortype::Gray16>(path, v, size),
                    ChannelCount::LA => {
                        log_image::<u16, colortype::RGBA16>(path, &la_to_rgb(v), size)
                    }
                    ChannelCount::RGB => log_image::<u16, colortype::RGB16>(path, v, size),
                    ChannelCount::RGBA => log_image::<u16, colortype::RGBA16>(path, v, size),
                },
            );
        }
        ChannelType::U8 => {
            test::<u8, _, _>(
                data,
                255.0f32,
                &|input: &[u8]| -> Result<Vec<u8>, String> {
                    match data.channel_count {
                        ChannelCount::P => pic_scale_safe::cropped_resize_plane8(
                            input,
                            source_size,
                            source_rect,
                            dst_size,
                            resampling_function,
                        ),
                        ChannelCount::LA => pic_scale_safe::cropped_resize_plane8_with_alpha(
                            input,
                            source_size,
                            source_rect,
                            dst_size,
                            resampling_function,
                        ),
                        ChannelCount::RGB => pic_scale_safe::cropped_resize_rgb8(
                            input,
                            source_size,
                            source_rect,
                            dst_size,
                            resampling_function,
                        ),
                        ChannelCount::RGBA => pic_scale_safe::cropped_resize_rgba8(
                            input,
                            source_size,
                            source_rect,
                            dst_size,
                            resampling_function,
                        ),
                    }
                },
                |x| (x * 255.0).round_ties_even() as u8,
                |path: &Path, v: &[u8], size: (u64, u64)| match data.channel_count {
                    ChannelCount::P => log_image::<u8, colortype::Gray8>(path, v, size),
                    ChannelCount::LA => {
                        log_image::<u8, colortype::RGBA8>(path, &la_to_rgb(v), size)
                    }
                    ChannelCount::RGB => log_image::<u8, colortype::RGB8>(path, v, size),
                    ChannelCount::RGBA => log_image::<u8, colortype::RGBA8>(path, v, size),
                },
            );
        }
    }

    Corpus::Keep
});
