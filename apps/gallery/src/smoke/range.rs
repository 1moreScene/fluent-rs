use std::{error::Error, time::Duration};

use slint::{
    ComponentHandle, LogicalPosition, Window,
    platform::{Key, PointerEventButton, WindowEvent},
};

use super::{check, click, read_rgb};
use crate::{
    ButtonState, FluentColorScheme, FluentTheme, GalleryPage, GalleryWindow, ProgressBarStatus,
    RangeBounds,
};

pub(super) fn verify(gallery: &GalleryWindow) -> Result<(), Box<dyn Error>> {
    let window = gallery.window();
    gallery.set_page(GalleryPage::Ranges);
    gallery.invoke_select_theme(FluentColorScheme::Light);
    gallery.global::<FluentTheme>().set_reduced_motion(true);

    for _ in 0..24 {
        key(window, Key::Tab);
        if gallery.get_slider_focus_visible() {
            break;
        }
    }
    check(gallery.get_slider_focus_visible(), "Tab shows Slider focus")?;
    key(window, Key::Home);
    key(window, Key::RightArrow);
    check(
        gallery.get_slider_value() == 5.,
        "Slider arrow uses small change",
    )?;
    window.dispatch_event(WindowEvent::KeyPressRepeated {
        text: Key::RightArrow.into(),
    });
    check(
        gallery.get_slider_value() == 10.,
        "Slider supports key repeat",
    )?;

    gallery.set_slider_maximum(95.);
    gallery.set_slider_snap_to_ticks(true);
    key(window, Key::End);
    check(
        gallery.get_slider_value() == 95.,
        "End reaches the off-grid maximum",
    )?;
    key(window, Key::LeftArrow);
    check(
        gallery.get_slider_value() == 80.,
        "Slider steps down from an off-grid maximum",
    )?;
    gallery.set_slider_reversed(true);
    key(window, Key::RightArrow);
    check(
        gallery.get_slider_value() == 60.,
        "Reversed Slider inverts the arrow",
    )?;
    gallery.set_slider_reversed(false);
    gallery.set_slider_snap_to_ticks(false);
    gallery.set_slider_maximum(100.);

    let slider = gallery.get_slider_bounds();
    let moved = |position| window.dispatch_event(WindowEvent::PointerMoved { position });
    let target = point(&slider, 0.52, 0.5);
    let fill = point(&slider, 0.15, 0.5);
    moved(LogicalPosition::new(1., 1.));
    check(
        gallery.get_slider_state() == ButtonState::Normal,
        "Slider normal state",
    )?;
    let normal = read_rgb(window, fill)?;
    let outline = LogicalPosition::new(
        slider.position.x + slider.width / 2.,
        slider.position.y - 2.,
    );
    let focused = read_rgb(window, outline)?;
    moved(target);
    check(
        gallery.get_slider_state() == ButtonState::PointerOver,
        "Slider hover state",
    )?;
    let hover = read_rgb(window, fill)?;
    check(hover != normal, "Skia paints hovered Slider fill")?;
    let button = PointerEventButton::Left;
    window.dispatch_event(WindowEvent::PointerPressed {
        position: target,
        button,
    });
    check(
        gallery.get_slider_state() == ButtonState::Pressed && !gallery.get_slider_focus_visible(),
        "Slider pressed state and pointer focus",
    )?;
    check(
        read_rgb(window, fill)? != hover,
        "Skia paints pressed Slider fill",
    )?;
    check(
        read_rgb(window, outline)? != focused,
        "Skia paints keyboard-only Slider focus",
    )?;
    check(
        gallery.get_slider_value() == 50.,
        "Slider track click snaps to step",
    )?;
    moved(point(&slider, 1.05, 0.5));
    check(
        gallery.get_slider_value() == 100.,
        "Slider captures drag beyond the track",
    )?;
    gallery.set_slider_enabled(false);
    let disabled_fill = read_rgb(window, fill)?;
    window.dispatch_event(WindowEvent::PointerReleased {
        position: target,
        button,
    });
    click(window, target);
    key(window, Key::Home);
    check(
        gallery.get_slider_state() == ButtonState::Disabled
            && gallery.get_slider_value() == 100.
            && disabled_fill != normal,
        "Disabled Slider cancels capture and ignores input",
    )?;
    gallery.set_slider_enabled(true);
    let edits = gallery.get_slider_edits();
    gallery.set_slider_value(40.);
    check(
        gallery.get_slider_edits() == edits,
        "Host writes do not emit Slider edits",
    )?;

    window.dispatch_event(WindowEvent::PointerScrolled {
        position: target,
        delta_x: 0.,
        delta_y: -220.,
    });
    let vertical = gallery.get_vertical_slider_bounds();
    click(window, point(&vertical, 0.5, 0.02));
    check(
        gallery.get_vertical_slider_value() == 50.,
        "Vertical Slider maximum is at the top",
    )?;
    key(window, Key::DownArrow);
    check(
        gallery.get_vertical_slider_value() == 40.,
        "Vertical Slider steps down by a tick",
    )?;
    window.dispatch_event(WindowEvent::PointerScrolled {
        position: target,
        delta_x: 0.,
        delta_y: 2000.,
    });
    verify_progress(gallery)
}

fn verify_progress(gallery: &GalleryWindow) -> Result<(), Box<dyn Error>> {
    let window = gallery.window();
    for scheme in [
        FluentColorScheme::Light,
        FluentColorScheme::Dark,
        FluentColorScheme::HighContrast,
    ] {
        gallery.invoke_select_theme(scheme);
        gallery.set_progress_ring_active(true);
        gallery.set_slider_value(0.);
        let empty = indicators(gallery)?;
        gallery.set_slider_maximum(0.);
        gallery.set_slider_value(50.);
        check(
            indicators(gallery)? == empty,
            "A zero-span range draws no progress",
        )?;
        gallery.set_slider_maximum(100.);
        let half = indicators(gallery)?;
        let bar = gallery.get_progress_bar_bounds();
        let ring = gallery.get_progress_ring_bounds();
        let theme = gallery.global::<FluentTheme>();
        let accent = if scheme == FluentColorScheme::HighContrast {
            theme.get_contrast_highlight()
        } else {
            theme.get_accent_fill()
        };
        let accent_rgb = [accent.red(), accent.green(), accent.blue()];
        check(
            half.0 != empty.0
                && half.1 != empty.1
                && read_rgb(window, point(&bar, 0.25, 0.5))? == accent_rgb
                && read_rgb(window, point(&ring, 0.94, 0.5))? == accent_rgb
                && read_rgb(window, point(&ring, 0.06, 0.5))? != accent_rgb,
            "Skia paints the partial bar and ring in the current theme",
        )?;
        gallery.set_slider_value(120.);
        let filled = indicators(gallery)?;
        check(
            filled.0 != half.0
                && filled.1 != half.1
                && read_rgb(window, point(&bar, 0.9, 0.5))? == accent_rgb
                && read_rgb(window, point(&ring, 0.06, 0.5))? == accent_rgb,
            "Out-of-range progress saturates at full",
        )?;
        gallery.set_progress_ring_active(false);
        check(
            indicators(gallery)?.1 != filled.1,
            "Inactive ProgressRing hides the arc and track",
        )?;
        gallery.set_progress_ring_active(true);
    }
    gallery.invoke_select_theme(FluentColorScheme::Light);
    gallery.set_slider_value(40.);
    gallery.set_progress_indeterminate(true);
    let bar = gallery.get_progress_bar_bounds();
    check(
        read_rgb(window, point(&bar, 0.1, 0.5))? != read_rgb(window, point(&bar, 0.5, 0.5))?,
        "Reduced Motion leaves a static indeterminate bar segment",
    )?;
    for (status, expected) in [
        (ProgressBarStatus::Paused, [0x9d, 0x5d, 0]),
        (ProgressBarStatus::Error, [0xc4, 0x2b, 0x1c]),
    ] {
        gallery.set_progress_bar_status(status);
        check(
            read_rgb(window, point(&bar, 0.1, 0.5))? == expected
                && read_rgb(window, point(&bar, 0.9, 0.5))? == expected,
            "Paused or error indeterminate ProgressBar shows a solid status color",
        )?;
    }
    gallery.set_progress_bar_status(ProgressBarStatus::Normal);
    gallery.set_progress_indeterminate(false);
    Ok(())
}

pub(super) fn verify_compact(
    gallery: &GalleryWindow,
    window_width: f32,
) -> Result<(), Box<dyn Error>> {
    gallery.set_page(GalleryPage::Ranges);
    let _ = gallery.window().take_snapshot()?;
    let slider = gallery.get_slider_bounds();
    check(
        slider.height >= 44.,
        "Compact Slider keeps its touch target",
    )?;
    for bounds in [
        slider,
        gallery.get_progress_bar_bounds(),
        gallery.get_progress_ring_bounds(),
    ] {
        check(
            bounds.position.x > 0. && bounds.position.x + bounds.width <= window_width,
            "Slider and progress examples fit the compact window",
        )?;
    }
    Ok(())
}

pub(super) fn verify_motion(
    gallery: &GalleryWindow,
    done: impl FnOnce(Result<(), Box<dyn Error>>) + 'static,
) {
    gallery.set_page(GalleryPage::Ranges);
    gallery.invoke_select_theme(FluentColorScheme::Light);
    gallery.set_progress_indeterminate(true);
    gallery.global::<FluentTheme>().set_reduced_motion(false);
    let before = match indicators(gallery) {
        Ok(samples) => samples,
        Err(error) => return done(Err(error)),
    };
    let weak = gallery.as_weak();
    slint::Timer::single_shot(Duration::from_millis(300), move || {
        let Some(gallery) = weak.upgrade() else {
            return;
        };
        let suspended = (|| {
            let animated = indicators(&gallery)?;
            check(
                animated.0 != before.0 && animated.1 != before.1,
                "Skia renders both indeterminate animations over time",
            )?;
            gallery.set_range_animations_enabled(false);
            indicators(&gallery)
        })();
        let suspended = match suspended {
            Ok(samples) => samples,
            Err(error) => return done(Err(error)),
        };
        let weak = gallery.as_weak();
        slint::Timer::single_shot(Duration::from_millis(300), move || {
            let Some(gallery) = weak.upgrade() else {
                return;
            };
            let static_result = indicators(&gallery).and_then(|after| {
                check(
                    after == suspended,
                    "Host pause keeps both indicators static",
                )
            });
            if let Err(error) = static_result {
                done(Err(error));
            } else {
                verify_determinate_motion(&gallery, done);
            }
        });
    });
}

fn verify_determinate_motion(
    gallery: &GalleryWindow,
    done: impl FnOnce(Result<(), Box<dyn Error>>) + 'static,
) {
    gallery.set_progress_indeterminate(false);
    gallery.set_slider_value(50.);
    let half = match indicators(gallery) {
        Ok((_, ring)) => ring,
        Err(error) => return done(Err(error)),
    };
    gallery.set_slider_value(0.);
    let empty = match indicators(gallery) {
        Ok((_, ring)) => ring,
        Err(error) => return done(Err(error)),
    };

    let weak = gallery.as_weak();
    slint::Timer::single_shot(Duration::from_millis(25), move || {
        let Some(gallery) = weak.upgrade() else {
            return;
        };
        gallery.set_range_animations_enabled(true);
        if let Err(error) = indicators(&gallery) {
            return done(Err(error));
        }
        gallery.set_slider_value(50.);
        let weak = gallery.as_weak();
        slint::Timer::single_shot(Duration::from_millis(300), move || {
            let Some(gallery) = weak.upgrade() else {
                return;
            };
            let intermediate = indicators(&gallery).and_then(|(_, ring)| {
                check(
                    ring != empty,
                    "Determinate ProgressRing has started its transition",
                )?;
                check(
                    ring != half,
                    "Determinate ProgressRing paints an intermediate arc",
                )
            });
            if let Err(error) = intermediate {
                return done(Err(error));
            }
            let weak = gallery.as_weak();
            slint::Timer::single_shot(Duration::from_millis(850), move || {
                let Some(gallery) = weak.upgrade() else {
                    return;
                };
                done(indicators(&gallery).and_then(|(_, ring)| {
                    check(
                        ring == half,
                        "Determinate ProgressRing reaches its target arc",
                    )
                }));
            });
        });
    });
}

type IndicatorPixels = (Vec<[u8; 3]>, Vec<[u8; 3]>);

fn indicators(gallery: &GalleryWindow) -> Result<IndicatorPixels, Box<dyn Error>> {
    let window = gallery.window();
    let scale = window.scale_factor();
    let frame = window.take_snapshot()?;
    let sample = |bounds: RangeBounds| -> Result<Vec<_>, Box<dyn Error>> {
        let start = point(&bounds, 0., 0.).to_physical(scale);
        let end = point(&bounds, 1., 1.).to_physical(scale);
        check(
            start.x >= 0
                && start.y >= 0
                && end.x <= frame.width() as i32
                && end.y <= frame.height() as i32,
            "Progress sample lies inside the Skia frame",
        )?;
        Ok((start.y..end.y)
            .flat_map(|y| {
                let row = y as usize * frame.width() as usize;
                frame.as_slice()[row + start.x as usize..row + end.x as usize]
                    .iter()
                    .map(|pixel| [pixel.r, pixel.g, pixel.b])
            })
            .collect())
    };
    Ok((
        sample(gallery.get_progress_bar_bounds())?,
        sample(gallery.get_progress_ring_bounds())?,
    ))
}

fn point(bounds: &RangeBounds, x: f32, y: f32) -> LogicalPosition {
    LogicalPosition::new(
        bounds.position.x + bounds.width * x,
        bounds.position.y + bounds.height * y,
    )
}

fn key(window: &Window, key: Key) {
    window.dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window.dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}
