use fluent_materials::{AcrylicBounds, AcrylicRecipe};
use slint::{Image, Window};

use crate::{AcrylicCapture, OverlayArea};

pub(super) fn capture(window: &Window, capture: AcrylicCapture) -> Option<Image> {
    fluent_materials::capture(
        window,
        AcrylicRecipe {
            bounds: bounds(capture.bounds),
            scrim_bounds: bounds(capture.scrim_bounds),
            scrim: capture.scrim,
            fallback: capture.fallback,
            blur_radius: capture.blur_radius,
            luminosity: capture.luminosity,
            tint: capture.tint,
            noise_opacity: capture.noise_opacity,
        },
    )
}

fn bounds(area: OverlayArea) -> AcrylicBounds {
    AcrylicBounds {
        x: area.x,
        y: area.y,
        width: area.width,
        height: area.height,
    }
}
