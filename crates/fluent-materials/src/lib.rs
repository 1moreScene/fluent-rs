//! Shared Skia compositor for the WinUI in-app Acrylic material.

use skia_safe::{
    AlphaType, BlendMode, ColorType, Data, ImageInfo, Paint, Rect, TileMode, image_filters, images,
    surfaces,
};
use slint::{Color, Image, Rgba8Pixel, SharedPixelBuffer, Window};

/// Acrylic bounds in logical window coordinates.
#[derive(Clone, Copy, Debug, Default)]
pub struct AcrylicBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// The material recipe shared by popup and inline Acrylic hosts.
#[derive(Clone, Copy, Debug, Default)]
pub struct AcrylicRecipe {
    pub bounds: AcrylicBounds,
    pub scrim_bounds: AcrylicBounds,
    pub scrim: Color,
    pub fallback: Color,
    pub blur_radius: f32,
    pub luminosity: Color,
    pub tint: Color,
    pub noise_opacity: f32,
}

thread_local! {
    static NOISE: Option<skia_safe::Shader> = images::deferred_from_encoded_data(
        Data::new_copy(include_bytes!("../../../ui/materials/acrylic-noise.png")),
        None,
    ).and_then(|image| image.to_shader(
        (TileMode::Repeat, TileMode::Repeat),
        skia_safe::SamplingOptions::default(),
        None,
    ));
}

/// Captures a window and renders the requested Acrylic region at its backing scale.
pub fn capture(window: &Window, recipe: AcrylicRecipe) -> Option<Image> {
    let frame = window.take_snapshot().ok()?;
    compose(frame, window.scale_factor(), recipe).map(Image::from_rgba8_premultiplied)
}

#[derive(Clone, Copy)]
struct Pixels {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

fn compose(
    frame: SharedPixelBuffer<Rgba8Pixel>,
    scale: f32,
    recipe: AcrylicRecipe,
) -> Option<SharedPixelBuffer<Rgba8Pixel>> {
    if !scale.is_finite() || scale <= 0. || !recipe.blur_radius.is_finite() {
        return None;
    }
    let bounds = recipe.bounds;
    let left = (bounds.x * scale).floor() as i32;
    let top = (bounds.y * scale).floor() as i32;
    let right = ((bounds.x + bounds.width) * scale).ceil() as i32;
    let bottom = ((bounds.y + bounds.height) * scale).ceil() as i32;
    let frame_width = frame.width() as i32;
    let frame_height = frame.height() as i32;
    let output = Pixels {
        x: left.clamp(0, frame_width),
        y: top.clamp(0, frame_height),
        width: right.clamp(0, frame_width) - left.clamp(0, frame_width),
        height: bottom.clamp(0, frame_height) - top.clamp(0, frame_height),
    };
    if output.width <= 0 || output.height <= 0 {
        return None;
    }

    let sigma = recipe.blur_radius.max(0.) * scale;
    let pad = (sigma * 3.).ceil() as i32;
    let source_x = (output.x - pad).max(0);
    let source_y = (output.y - pad).max(0);
    let source = Pixels {
        x: source_x,
        y: source_y,
        width: (output.x + output.width)
            .saturating_add(pad)
            .min(frame_width)
            - source_x,
        height: (output.y + output.height)
            .saturating_add(pad)
            .min(frame_height)
            - source_y,
    };
    // Keep an owned reference to the Slint frame for as long as Skia may read it.
    // The cropped image uses the original stride and never allocates a second pixel array.
    let stride = frame_width as usize * 4;
    let offset = source.y as usize * stride + source.x as usize * 4;
    let length = (source.height as usize - 1) * stride + source.width as usize * 4;
    let image = images::raster_from_data(
        &rgba_info(source, AlphaType::Unpremul),
        Data::new_with_owner(FrameRegion {
            frame,
            offset,
            length,
        }),
        stride,
    )?;

    let mut backdrop = surfaces::raster_n32_premul((source.width, source.height))?;
    let canvas = backdrop.canvas();
    canvas.clear(skia_color(recipe.fallback));
    canvas.draw_image(&image, (0., 0.), None);
    drop(image);
    if recipe.scrim.alpha() > 0 {
        let area = recipe.scrim_bounds;
        canvas.draw_rect(
            Rect::from_xywh(
                area.x * scale - source.x as f32,
                area.y * scale - source.y as f32,
                area.width * scale,
                area.height * scale,
            ),
            &Paint::new(skia_safe::Color4f::from(skia_color(recipe.scrim)), None),
        );
    }

    // Render directly into the shared upload buffer. No second surface or readback is needed.
    let mut pixels = SharedPixelBuffer::new(output.width as u32, output.height as u32);
    let mut material = surfaces::wrap_pixels(
        &rgba_info(output, AlphaType::Premul),
        pixels.make_mut_bytes(),
        output.width as usize * 4,
        None,
    )?;
    let canvas = material.canvas();
    let mut blur = Paint::default();
    if sigma > 0. {
        blur.set_image_filter(image_filters::blur(
            (sigma, sigma),
            TileMode::Clamp,
            None,
            None,
        ));
    }
    canvas.draw_image(
        backdrop.image_snapshot(),
        ((source.x - output.x) as f32, (source.y - output.y) as f32),
        Some(&blur),
    );
    canvas.draw_color(skia_color(recipe.luminosity), BlendMode::Luminosity);
    canvas.draw_color(skia_color(recipe.tint), BlendMode::Color);
    if recipe.noise_opacity > 0. {
        NOISE.with(|shader| {
            if let Some(shader) = shader {
                let mut paint = Paint::default();
                paint.set_shader(shader.clone());
                paint.set_alpha_f(recipe.noise_opacity);
                canvas.draw_paint(&paint);
            }
        });
    }

    drop(material);
    Some(pixels)
}

struct FrameRegion {
    frame: SharedPixelBuffer<Rgba8Pixel>,
    offset: usize,
    length: usize,
}

impl AsRef<[u8]> for FrameRegion {
    fn as_ref(&self) -> &[u8] {
        &self.frame.as_bytes()[self.offset..self.offset + self.length]
    }
}

fn rgba_info(rect: Pixels, alpha_type: AlphaType) -> ImageInfo {
    ImageInfo::new(
        (rect.width, rect.height),
        ColorType::RGBA8888,
        alpha_type,
        None,
    )
}

fn skia_color(color: Color) -> skia_safe::Color {
    skia_safe::Color::from_argb(color.alpha(), color.red(), color.green(), color.blue())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn official_noise_tile_decodes_in_skia() {
        assert!(NOISE.with(Option::is_some));
    }

    #[test]
    fn backdrop_region_preserves_the_original_stride_and_pixel_values() {
        let source: Vec<_> = (0..20)
            .map(|n| Rgba8Pixel {
                r: n * 10,
                g: 100,
                b: n * 5,
                a: 255,
            })
            .collect();
        let mut frame = SharedPixelBuffer::<Rgba8Pixel>::new(5, 4);
        frame.make_mut_slice().copy_from_slice(&source);
        let result = compose(
            frame,
            1.,
            AcrylicRecipe {
                bounds: AcrylicBounds {
                    x: 2.,
                    y: 1.,
                    width: 2.,
                    height: 3.,
                },
                fallback: Color::from_rgb_u8(0, 0, 0),
                ..Default::default()
            },
        )
        .expect("valid in-bounds Acrylic region");
        assert_eq!(
            result.as_slice(),
            [
                source[7], source[8], source[12], source[13], source[17], source[18]
            ]
        );
    }

    #[test]
    fn luminosity_retains_backdrop_hue_and_color_tint_retains_luminance() {
        let frame = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(&[130u8, 55, 55, 255], 1, 1);
        let recipe = AcrylicRecipe {
            bounds: AcrylicBounds {
                width: 1.,
                height: 1.,
                ..Default::default()
            },
            fallback: Color::from_rgb_u8(0, 0, 0),
            luminosity: Color::from_rgb_u8(110, 110, 110),
            ..Default::default()
        };
        let alpha = compose(frame.clone(), 1., recipe);
        let pixel = alpha
            .as_ref()
            .and_then(|image| image.as_slice().first())
            .copied()
            .unwrap_or_default();
        assert!(pixel.r > pixel.g + 50 && pixel.g == pixel.b && pixel.a == 255);
        let result = compose(
            frame,
            1.,
            AcrylicRecipe {
                tint: Color::from_rgb_u8(0, 0, 215),
                ..recipe
            },
        );
        let tinted = result
            .as_ref()
            .and_then(|image| image.as_slice().first())
            .copied()
            .unwrap_or_default();
        assert!(tinted.b > tinted.r + 70 && tinted.r == tinted.g && tinted.a == 255);
    }
}
