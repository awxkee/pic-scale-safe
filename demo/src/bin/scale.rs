/*
 * Copyright (c) pic-scale-safe authors, 2026. All rights reserved.
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
use image::{DynamicImage, ImageBuffer};
use pic_scale_safe::{ImageSize, ResamplingFunction, TransferFunction};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

enum ScaleMode {
    Exact((u32, u32)),
    Ratio((f64, f64)),
}

struct ScaleOptions {
    scale: ScaleMode,
    resample_fn: ResamplingFunction,
    transfer_fn: Option<TransferFunction>,
}

fn parse_size(s: &str) -> Result<(u32, u32), String> {
    let Some((w, h)) = s.split_once("x") else {
        return Err("Format does not match WxH".to_string());
    };
    let w = w.parse::<u32>().map_err(|e| format!("Bad width: {}", e))?;
    let h = h.parse::<u32>().map_err(|e| format!("Bad height: {}", e))?;
    if w == 0 || h == 0 {
        return Err("Size must be nonempty".to_string());
    }
    Ok((w, h))
}

fn parse_scale(s: &str) -> Result<(f64, f64), String> {
    if let Some((f1, f2)) = s.split_once("x") {
        let f1 = f1
            .parse::<f64>()
            .map_err(|e| format!("Bad x scale: {}", e))?;
        let f2 = f2
            .parse::<f64>()
            .map_err(|e| format!("Bad y scale: {}", e))?;
        if !(f1 > 0.0 && f2 > 0.0) {
            return Err("Scales must be > 0".to_string());
        }
        Ok((f1, f2))
    } else {
        let f = s
            .parse::<f64>()
            .map_err(|e| format!("Bad x scale: {}", e))?;
        if !(f > 0.0) {
            return Err("Scale must be > 0".to_string());
        }
        Ok((f, f))
    }
}

fn run(src: &Path, dst: &Path, opts: ScaleOptions) -> Result<(), String> {
    let img = image::open(src).map_err(|e| format!("Failed to open {src:?}: {e:?}"))?;
    let src_size = ImageSize::new(img.width() as usize, img.height() as usize);

    let dst_size = match opts.scale {
        ScaleMode::Exact((w, h)) => ImageSize::new(w as usize, h as usize),
        ScaleMode::Ratio((fx, fy)) => ImageSize::new(
            (((src_size.width as f64) * fx).round_ties_even() as u32).max(1) as usize,
            (((src_size.height as f64) * fy).round_ties_even() as u32).max(1) as usize,
        ),
    };

    // A (mediocre) heuristic: in the absence of color metadata, u8 and u16 images are likely
    // sRGB, while any f32 images are likely linear because they don't need a transfer function.
    let tf_fixed = opts.transfer_fn.unwrap_or(TransferFunction::Srgb);
    let tf_float = opts.transfer_fn.unwrap_or(TransferFunction::Linear);

    let out: DynamicImage = match img {
        DynamicImage::ImageLuma8(img) => {
            let mut data = img.into_raw();
            pic_scale_safe::image_to_linear::<1>(&mut data, tf_fixed);
            let mut out =
                pic_scale_safe::resize_plane8(&data, src_size, dst_size, opts.resample_fn)
                    .map_err(|e| format!("Failed to resize: {e:?}"))?;
            pic_scale_safe::linear_to_gamma_image::<1>(&mut out, tf_fixed);
            DynamicImage::ImageLuma8(
                ImageBuffer::<_, _>::from_raw(dst_size.width as u32, dst_size.height as u32, out)
                    .unwrap(),
            )
        }
        DynamicImage::ImageLumaA8(img) => {
            let mut data = img.into_raw();
            pic_scale_safe::image_to_linear::<2>(&mut data, tf_fixed);
            pic_scale_safe::premultiply_la8(&mut data);
            let mut out = pic_scale_safe::resize_plane8_with_alpha(
                &data,
                src_size,
                dst_size,
                opts.resample_fn,
            )
            .map_err(|e| format!("Failed to resize: {e:?}"))?;
            pic_scale_safe::unpremultiply_la8(&mut out);
            pic_scale_safe::linear_to_gamma_image::<2>(&mut out, tf_fixed);

            DynamicImage::ImageLumaA8(
                ImageBuffer::<_, _>::from_raw(dst_size.width as u32, dst_size.height as u32, out)
                    .unwrap(),
            )
        }
        DynamicImage::ImageRgb8(img) => {
            let mut data = img.into_raw();
            pic_scale_safe::image_to_linear::<3>(&mut data, tf_fixed);
            let mut out = pic_scale_safe::resize_rgb8(&data, src_size, dst_size, opts.resample_fn)
                .map_err(|e| format!("Failed to resize: {e:?}"))?;
            pic_scale_safe::linear_to_gamma_image::<3>(&mut out, tf_fixed);
            DynamicImage::ImageRgb8(
                ImageBuffer::<_, _>::from_raw(dst_size.width as u32, dst_size.height as u32, out)
                    .unwrap(),
            )
        }
        DynamicImage::ImageRgba8(img) => {
            let mut data = img.into_raw();
            pic_scale_safe::image_to_linear::<4>(&mut data, tf_fixed);
            pic_scale_safe::premultiply_rgba8(&mut data);
            let mut out = pic_scale_safe::resize_rgba8(&data, src_size, dst_size, opts.resample_fn)
                .map_err(|e| format!("Failed to resize: {e:?}"))?;
            pic_scale_safe::unpremultiply_rgba8(&mut out);
            pic_scale_safe::linear_to_gamma_image::<4>(&mut out, tf_fixed);
            DynamicImage::ImageRgba8(
                ImageBuffer::<_, _>::from_raw(dst_size.width as u32, dst_size.height as u32, out)
                    .unwrap(),
            )
        }
        DynamicImage::ImageLuma16(img) => {
            let mut data = img.into_raw();
            pic_scale_safe::image16_to_linear16::<1>(&mut data, 16, tf_fixed);
            let mut out =
                pic_scale_safe::resize_plane16(&data, src_size, dst_size, 16, opts.resample_fn)
                    .map_err(|e| format!("Failed to resize: {e:?}"))?;
            pic_scale_safe::linear16_to_gamma_image16::<1>(&mut out, 16, tf_fixed);
            DynamicImage::ImageLuma16(
                ImageBuffer::<_, _>::from_raw(dst_size.width as u32, dst_size.height as u32, out)
                    .unwrap(),
            )
        }
        DynamicImage::ImageLumaA16(img) => {
            let mut data = img.into_raw();
            pic_scale_safe::image16_to_linear16::<2>(&mut data, 16, tf_fixed);
            pic_scale_safe::premultiply_la16(&mut data, 16);
            let mut out = pic_scale_safe::resize_plane16_with_alpha(
                &data,
                src_size,
                dst_size,
                16,
                opts.resample_fn,
            )
            .map_err(|e| format!("Failed to resize: {e:?}"))?;
            pic_scale_safe::unpremultiply_la16(&mut out, 16);
            pic_scale_safe::linear16_to_gamma_image16::<2>(&mut out, 16, tf_fixed);
            DynamicImage::ImageLumaA16(
                ImageBuffer::<_, _>::from_raw(dst_size.width as u32, dst_size.height as u32, out)
                    .unwrap(),
            )
        }
        DynamicImage::ImageRgb16(img) => {
            let mut data = img.into_raw();
            pic_scale_safe::image16_to_linear16::<3>(&mut data, 16, tf_fixed);
            let mut out =
                pic_scale_safe::resize_rgb16(&data, src_size, dst_size, 16, opts.resample_fn)
                    .map_err(|e| format!("Failed to resize: {e:?}"))?;
            pic_scale_safe::linear16_to_gamma_image16::<3>(&mut out, 16, tf_fixed);
            DynamicImage::ImageRgb16(
                ImageBuffer::<_, _>::from_raw(dst_size.width as u32, dst_size.height as u32, out)
                    .unwrap(),
            )
        }
        DynamicImage::ImageRgba16(img) => {
            let mut data = img.into_raw();
            pic_scale_safe::image16_to_linear16::<4>(&mut data, 16, tf_fixed);
            pic_scale_safe::premultiply_rgba16(&mut data, 16);
            let mut out =
                pic_scale_safe::resize_rgba16(&data, src_size, dst_size, 16, opts.resample_fn)
                    .map_err(|e| format!("Failed to resize: {e:?}"))?;
            pic_scale_safe::unpremultiply_rgba16(&mut out, 16);
            pic_scale_safe::linear16_to_gamma_image16::<4>(&mut out, 16, tf_fixed);
            DynamicImage::ImageRgba16(
                ImageBuffer::<_, _>::from_raw(dst_size.width as u32, dst_size.height as u32, out)
                    .unwrap(),
            )
        }
        // TODO: image 1.0 will add ImageLuma32F and ImagLumaA32F
        DynamicImage::ImageRgb32F(img) => {
            let mut data = img.into_raw();
            pic_scale_safe::image_f32_to_linear_f32::<3>(&mut data, tf_float);
            let mut out =
                pic_scale_safe::resize_rgb_f32(&data, src_size, dst_size, opts.resample_fn)
                    .map_err(|e| format!("Failed to resize: {e:?}"))?;
            pic_scale_safe::linear_f32_to_gamma_image_f32::<3>(&mut out, tf_float);
            DynamicImage::ImageRgb32F(
                ImageBuffer::<_, _>::from_raw(dst_size.width as u32, dst_size.height as u32, out)
                    .unwrap(),
            )
        }
        DynamicImage::ImageRgba32F(img) => {
            let mut data = img.into_raw();
            pic_scale_safe::image_f32_to_linear_f32::<4>(&mut data, tf_float);
            pic_scale_safe::premultiply_rgba_f32(&mut data);
            let mut out =
                pic_scale_safe::resize_rgba_f32(&data, src_size, dst_size, opts.resample_fn)
                    .map_err(|e| format!("Failed to resize: {e:?}"))?;
            pic_scale_safe::unpremultiply_rgba_f32(&mut out);
            pic_scale_safe::linear_f32_to_gamma_image_f32::<4>(&mut out, tf_float);
            DynamicImage::ImageRgba32F(
                ImageBuffer::<_, _>::from_raw(dst_size.width as u32, dst_size.height as u32, out)
                    .unwrap(),
            )
        }
        _ => {
            return Err(format!("Unsupported image type: {:?}", img.color()));
        }
    };

    out.save(dst)
        .map_err(|e| format!("Failed to save {dst:?}: {e:?}"))?;
    Ok(())
}

fn main() -> ExitCode {
    let cmd = clap::Command::new(env!("CARGO_BIN_NAME"))
        .arg(
            clap::Arg::new("size")
                .long("size")
                .help("Scale destination size, WxH pixels"),
        )
        .arg(
            clap::Arg::new("scale")
                .long("scale")
                .help("Scale factor, F or GxH"),
        )
        .arg(
            clap::Arg::new("rf")
                .long("rf")
                .help("Resampling function to use"),
        )
        .arg(
            clap::Arg::new("tf")
                .long("tf")
                .help("Transfer function to use"),
        )
        .arg(
            clap::Arg::new("src_file")
                .required(true)
                .value_parser(clap::builder::PathBufValueParser::new()),
        )
        .arg(
            clap::Arg::new("dst_file")
                .required(true)
                .value_parser(clap::builder::PathBufValueParser::new()),
        );
    let matches = cmd.get_matches();

    let src_path = matches.get_one::<PathBuf>("src_file").unwrap();
    let dst_path = matches.get_one::<PathBuf>("dst_file").unwrap();

    let mode = if let Some(size) = matches.get_one::<String>("size") {
        match parse_size(size) {
            Ok(x) => ScaleMode::Exact(x),
            Err(e) => {
                eprintln!("Failed to parse --size value '{:?}': {}", size, e);
                return ExitCode::FAILURE;
            }
        }
    } else if let Some(scale) = matches.get_one::<String>("scale") {
        match parse_scale(scale) {
            Ok(x) => ScaleMode::Ratio(x),
            Err(e) => {
                eprintln!("Failed to parse --scale value '{:?}': {}", scale, e);
                return ExitCode::FAILURE;
            }
        }
    } else {
        ScaleMode::Ratio((1.0, 1.0))
    };

    let resample_fn = if let Some(rf) = matches.get_one::<String>("rf") {
        let name_map = [
            ("Bilinear", ResamplingFunction::Bilinear),
            ("Nearest", ResamplingFunction::Nearest),
            ("Cubic", ResamplingFunction::Cubic),
            ("MitchellNetravalli", ResamplingFunction::MitchellNetravalli),
            ("CatmullRom", ResamplingFunction::CatmullRom),
            ("Hermite", ResamplingFunction::Hermite),
            ("BSpline", ResamplingFunction::BSpline),
            ("Hann", ResamplingFunction::Hann),
            ("Bicubic", ResamplingFunction::Bicubic),
            ("Hamming", ResamplingFunction::Hamming),
            ("Hanning", ResamplingFunction::Hanning),
            ("Blackman", ResamplingFunction::Blackman),
            ("Welch", ResamplingFunction::Welch),
            ("Quadric", ResamplingFunction::Quadric),
            ("Gaussian", ResamplingFunction::Gaussian),
            ("Sphinx", ResamplingFunction::Sphinx),
            ("Bartlett", ResamplingFunction::Bartlett),
            ("Robidoux", ResamplingFunction::Robidoux),
            ("RobidouxSharp", ResamplingFunction::RobidouxSharp),
            ("Spline16", ResamplingFunction::Spline16),
            ("Spline36", ResamplingFunction::Spline36),
            ("Spline64", ResamplingFunction::Spline64),
            ("Kaiser", ResamplingFunction::Kaiser),
            ("BartlettHann", ResamplingFunction::BartlettHann),
            ("Box", ResamplingFunction::Box),
            ("Bohman", ResamplingFunction::Bohman),
            ("Lanczos2", ResamplingFunction::Lanczos2),
            ("Lanczos3", ResamplingFunction::Lanczos3),
            ("Lanczos4", ResamplingFunction::Lanczos4),
            ("Lanczos2Jinc", ResamplingFunction::Lanczos2Jinc),
            ("Lanczos3Jinc", ResamplingFunction::Lanczos3Jinc),
            ("Lanczos4Jinc", ResamplingFunction::Lanczos4Jinc),
            ("Ginseng", ResamplingFunction::Ginseng),
            ("HaasnSoft", ResamplingFunction::HaasnSoft),
            ("Lagrange2", ResamplingFunction::Lagrange2),
            ("Lagrange3", ResamplingFunction::Lagrange3),
            ("Lanczos6", ResamplingFunction::Lanczos6),
            ("Lanczos6Jinc", ResamplingFunction::Lanczos6Jinc),
            ("Area", ResamplingFunction::Area),
        ];

        let Some(idx) = name_map.iter().position(|kv| kv.0.eq_ignore_ascii_case(rf)) else {
            eprintln!("Unrecognized resampling function '{:?}'. Options:", rf);
            for (k, _) in name_map {
                eprintln!("{}", k);
            }
            return ExitCode::FAILURE;
        };

        name_map[idx].1
    } else {
        ResamplingFunction::default()
    };

    let transfer_fn = if let Some(tf) = matches.get_one::<String>("tf") {
        let name_map = [
            ("Srgb", TransferFunction::Srgb),
            ("Rec709", TransferFunction::Rec709),
            ("Gamma2p2", TransferFunction::Gamma2p2),
            ("Gamma2p8", TransferFunction::Gamma2p8),
            ("Smpte428", TransferFunction::Smpte428),
            ("Log100", TransferFunction::Log100),
            ("Log100Sqrt10", TransferFunction::Log100Sqrt10),
            ("Bt1361", TransferFunction::Bt1361),
            ("Smpte240", TransferFunction::Smpte240),
            ("Iec61966", TransferFunction::Iec61966),
            ("Linear", TransferFunction::Linear),
        ];

        let Some(idx) = name_map.iter().position(|kv| kv.0.eq_ignore_ascii_case(tf)) else {
            eprintln!("Unrecognized transfer_fn function '{:?}'. Options:", tf);
            for (k, _) in name_map {
                eprintln!("{}", k);
            }
            return ExitCode::FAILURE;
        };

        Some(name_map[idx].1)
    } else {
        None
    };

    let scale_options = ScaleOptions {
        scale: mode,
        resample_fn,
        transfer_fn,
    };

    if let Err(e) = run(src_path, dst_path, scale_options) {
        eprintln!("{} failed: {}", env!("CARGO_BIN_NAME"), e);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
