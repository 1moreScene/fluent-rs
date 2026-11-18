use std::{
    cell::{Cell, RefCell},
    error::Error,
    rc::Rc,
    time::Duration,
};

use slint::{
    ComponentHandle, LogicalPosition, LogicalSize, SharedString, Window,
    platform::{Key, PointerEventButton, WindowEvent},
};

use crate::{
    AppearanceAction, ButtonState, FluentColorScheme, FluentMaterial, FluentTheme, GalleryPage,
    GalleryWindow,
};

mod radio;
mod range;
mod text_box;
mod toggles;

pub(super) fn verify(gallery: &GalleryWindow) -> Result<(), Box<dyn Error>> {
    let open_gl_frame = Rc::new(Cell::new(false));
    let frame_notified = Rc::clone(&open_gl_frame);
    gallery.window().set_rendering_notifier(move |state, api| {
        if matches!(state, slint::RenderingState::AfterRendering)
            && matches!(api, slint::GraphicsAPI::NativeOpenGL { .. })
        {
            frame_notified.set(true);
        }
    })?;

    let outcome = Rc::new(RefCell::new(None));
    let result = Rc::clone(&outcome);
    let scale_factor = Rc::new(Cell::new(1.));
    let scale = Rc::clone(&scale_factor);
    let weak = gallery.as_weak();
    let timer = slint::Timer::default();
    timer.start(
        slint::TimerMode::SingleShot,
        Duration::from_millis(250),
        move || {
            let Some(gallery) = weak.upgrade() else {
                return;
            };
            scale.set(gallery.window().scale_factor());
            let wide = match verify_input(&gallery)
                .and_then(|()| verify_themes(&gallery))
                .and_then(|_| text_box::verify(&gallery))
                .and_then(|()| verify_themes(&gallery))
                .and_then(|size| toggles::verify(&gallery).map(|()| size))
                .and_then(|size| radio::verify(&gallery).map(|()| size))
                .and_then(|size| range::verify(&gallery).map(|()| size))
            {
                Ok(size) => size,
                Err(error) => {
                    result.replace(Some(Err(error)));
                    stop_event_loop();
                    return;
                }
            };

            gallery.set_page(GalleryPage::Button);
            gallery.set_touch_targets(true);
            gallery.window().set_size(LogicalSize::new(320., 640.));
            let weak = gallery.as_weak();
            let result = Rc::clone(&result);
            slint::Timer::single_shot(Duration::from_millis(250), move || {
                if let Some(gallery) = weak.upgrade() {
                    match verify_compact(&gallery) {
                        Ok(compact) => {
                            let weak = gallery.as_weak();
                            toggles::verify_mark_motion(&gallery, move |motion| {
                                if let Err(error) = motion {
                                    result.replace(Some(Err(error)));
                                    stop_event_loop();
                                } else if let Some(gallery) = weak.upgrade() {
                                    range::verify_motion(&gallery, move |motion| {
                                        result.replace(Some(motion.map(|()| (wide, compact))));
                                        stop_event_loop();
                                    });
                                }
                            });
                            return;
                        }
                        Err(error) => {
                            result.replace(Some(Err(error)));
                        }
                    }
                }
                stop_event_loop();
            });
        },
    );
    gallery.run()?;

    let ((width, height), (compact_width, compact_height)) = outcome
        .take()
        .ok_or("Gallery closed before its control checks completed")??;
    let backend = if open_gl_frame.get() { " (OpenGL)" } else { "" };
    println!(
        "Fluent RS Gallery: Skia rendered {width}x{height} and {compact_width}x{compact_height}{backend} at {}x scale; all control, theme, and material checks passed",
        scale_factor.get()
    );
    Ok(())
}

fn verify_input(gallery: &GalleryWindow) -> Result<(), Box<dyn Error>> {
    let window = gallery.window();
    gallery.global::<FluentTheme>().set_reduced_motion(true);
    let position = LogicalPosition::new(gallery.get_example_x(), gallery.get_example_y());
    let top = position.y - gallery.get_example_height() / 2.;
    let fill_sample = LogicalPosition::new(position.x, top + 3.);
    let ring_sample = LogicalPosition::new(position.x, top - 2.);
    let outside = LogicalPosition::new(1., 1.);
    let button = PointerEventButton::Left;
    let press = |text: SharedString| window.dispatch_event(WindowEvent::KeyPressed { text });
    let release = |text: SharedString| window.dispatch_event(WindowEvent::KeyReleased { text });

    window.dispatch_event(WindowEvent::PointerMoved { position: outside });
    check(
        gallery.get_example_state() == ButtonState::Normal,
        "normal state",
    )?;
    let normal_fill = read_rgb(window, fill_sample)?;
    let unfocused_ring = read_rgb(window, ring_sample)?;
    window.dispatch_event(WindowEvent::PointerMoved { position });
    check(
        gallery.get_example_state() == ButtonState::PointerOver,
        "mouse hover",
    )?;
    let hover_fill = read_rgb(window, fill_sample)?;
    check(
        hover_fill != normal_fill,
        "Skia paints the hovered Button fill",
    )?;
    window.dispatch_event(WindowEvent::PointerPressed { position, button });
    check(
        gallery.get_example_state() == ButtonState::Pressed,
        "mouse press",
    )?;
    check(
        read_rgb(window, fill_sample)? != hover_fill,
        "Skia paints the pressed Button fill",
    )?;
    check(
        gallery.get_activation_count() == 0,
        "mouse down must not activate",
    )?;
    window.dispatch_event(WindowEvent::PointerMoved { position: outside });
    check(
        gallery.get_example_state() == ButtonState::Normal,
        "drag outside clears pressed visuals",
    )?;
    window.dispatch_event(WindowEvent::PointerReleased {
        position: outside,
        button,
    });
    check(
        gallery.get_activation_count() == 0,
        "drag outside must cancel activation",
    )?;
    window.dispatch_event(WindowEvent::PointerPressed { position, button });
    check(
        gallery.get_example_state() == ButtonState::Pressed,
        "press without pointer movement",
    )?;
    window.dispatch_event(WindowEvent::PointerMoved { position: outside });
    window.dispatch_event(WindowEvent::PointerMoved { position });
    check(
        gallery.get_example_state() == ButtonState::Pressed,
        "drag back inside restores press",
    )?;
    window.dispatch_event(WindowEvent::PointerReleased { position, button });
    check(gallery.get_activation_count() == 1, "mouse release inside")?;
    check(
        !gallery.get_example_focus_visible(),
        "pointer click must hide keyboard focus",
    )?;
    window.dispatch_event(WindowEvent::PointerMoved { position: outside });

    for _ in 0..12 {
        press(Key::Tab.into());
        release(Key::Tab.into());
        if gallery.get_example_focus_visible() {
            break;
        }
    }
    check(
        gallery.get_example_focus_visible(),
        "Tab must show keyboard focus",
    )?;
    check(
        read_rgb(window, ring_sample)? != unfocused_ring,
        "Skia paints the outside keyboard focus ring",
    )?;
    for key in [SharedString::from(" "), Key::Return.into()] {
        let count = gallery.get_activation_count();
        press(key.clone());
        check(
            gallery.get_example_state() == ButtonState::Pressed,
            "keyboard press",
        )?;
        window.dispatch_event(WindowEvent::KeyPressRepeated { text: key.clone() });
        check(
            gallery.get_activation_count() == count,
            "held key must not activate",
        )?;
        release(key);
        check(
            gallery.get_activation_count() == count + 1,
            "keyboard release once",
        )?;
    }

    press(" ".into());
    gallery.set_examples_enabled(false);
    release(" ".into());
    window.dispatch_event(WindowEvent::PointerPressed { position, button });
    window.dispatch_event(WindowEvent::PointerReleased { position, button });
    check(
        gallery.get_example_state() == ButtonState::Disabled,
        "disabled state",
    )?;
    check(
        !gallery.get_example_focus_visible(),
        "disabled button hides focus",
    )?;
    check(
        read_rgb(window, fill_sample)? != normal_fill,
        "Skia paints the disabled Button fill",
    )?;
    check(
        read_rgb(window, ring_sample)? == unfocused_ring,
        "disabling the Button removes its focus ring",
    )?;
    check(
        gallery.get_activation_count() == 3,
        "disabled button must not activate",
    )?;
    gallery.set_examples_enabled(true);
    check(
        gallery.get_example_state() != ButtonState::Pressed,
        "re-enabled button must clear its press",
    )
}

fn read_rgb(window: &Window, position: LogicalPosition) -> Result<[u8; 3], Box<dyn Error>> {
    let point = position.to_physical(window.scale_factor());
    let frame = window.take_snapshot()?;
    if point.x < 0
        || point.y < 0
        || point.x as u32 >= frame.width()
        || point.y as u32 >= frame.height()
    {
        return Err("Sample point is outside the Skia frame".into());
    }
    let offset = point.y as usize * frame.width() as usize + point.x as usize;
    let pixel = frame.as_slice()[offset];
    Ok([pixel.r, pixel.g, pixel.b])
}

fn verify_themes(gallery: &GalleryWindow) -> Result<(u32, u32), Box<dyn Error>> {
    let theme = gallery.global::<FluentTheme>();
    theme.set_reduced_motion(true);
    let mut size = (0, 0);
    for (scheme, expected) in [
        (FluentColorScheme::Light, 0xf3),
        (FluentColorScheme::Dark, 0x20),
        (FluentColorScheme::HighContrast, 0x00),
    ] {
        gallery.invoke_select_theme(scheme);
        check(
            theme.get_scheme() == scheme,
            "Rust applies the selected theme",
        )?;
        let frame = gallery.window().take_snapshot()?;
        let (first, rest) = frame
            .as_slice()
            .split_first()
            .ok_or("Skia returned an empty frame")?;
        check(
            [first.r, first.g, first.b] == [expected; 3],
            "Gallery theme background",
        )?;
        check(
            rest.iter().any(|pixel| pixel != first),
            "Skia rendered Gallery content",
        )?;
        size = (frame.width(), frame.height());
    }
    Ok(size)
}

fn verify_compact(gallery: &GalleryWindow) -> Result<(u32, u32), Box<dyn Error>> {
    let window_width = gallery
        .window()
        .size()
        .to_logical(gallery.window().scale_factor())
        .width;
    check((window_width - 320.).abs() <= 1., "compact window resized")?;
    check(
        gallery.get_examples_right() <= window_width,
        "compact examples fit",
    )?;
    check(gallery.get_example_height() >= 44., "touch target height")?;
    verify_themes(gallery)?;
    gallery.set_page(GalleryPage::TextBox);
    check(
        gallery.get_text_box_x() > 0. && gallery.get_text_box_right() <= window_width,
        "compact TextBox fits",
    )?;
    let size = verify_themes(gallery)?;
    toggles::verify_compact(gallery, window_width)?;
    range::verify_compact(gallery, window_width)?;
    radio::verify_compact(gallery)?;
    verify_foundations(gallery)?;
    Ok(size)
}

fn verify_foundations(gallery: &GalleryWindow) -> Result<(), Box<dyn Error>> {
    let window = gallery.window();
    let theme = gallery.global::<FluentTheme>();
    gallery.set_page(GalleryPage::Foundations);
    gallery.invoke_select_theme(FluentColorScheme::Light);
    let center = gallery.get_motion_button_center();
    click(window, LogicalPosition::new(center.x, center.y));
    check(
        !theme.get_reduced_motion() && gallery.get_motion_enabled(),
        "the motion button updates the Rust preference and enables transitions",
    )?;
    gallery.invoke_select_theme(FluentColorScheme::HighContrast);
    check(
        !gallery.get_motion_enabled(),
        "contrast disables transitions",
    )?;
    gallery.invoke_select_theme(FluentColorScheme::Light);
    gallery.invoke_change_appearance(AppearanceAction::ToggleMotion);
    check(
        !gallery.get_motion_enabled(),
        "reduced motion disables transitions",
    )?;

    let default_accent = theme.get_accent_fill();
    gallery.invoke_change_appearance(AppearanceAction::ToggleAccent);
    let accent = theme.get_accent_fill();
    check(accent != default_accent, "theme uses the alternate accent")?;
    let snapshot = window.take_snapshot()?;
    check(
        snapshot.as_slice().iter().any(|pixel| {
            [pixel.r, pixel.g, pixel.b] == [accent.red(), accent.green(), accent.blue()]
        }),
        "Skia paints the alternate accent",
    )?;
    gallery.invoke_change_appearance(AppearanceAction::ToggleAccent);
    check(
        theme.get_accent_fill() == default_accent,
        "restore the default accent",
    )?;

    gallery.set_page(GalleryPage::Button);
    gallery.set_touch_targets(false);
    let normal_height = gallery.get_example_height();
    gallery.invoke_change_appearance(AppearanceAction::ToggleTextScale);
    check(
        gallery.get_example_height() > normal_height,
        "text scaling changes the actual Button layout",
    )?;
    check(
        gallery.get_examples_right() <= 320.,
        "scaled buttons fit the compact window",
    )?;
    gallery.set_page(GalleryPage::Foundations);
    gallery.invoke_change_appearance(AppearanceAction::ToggleTextScale);

    for (scheme, material, expected) in [
        (FluentColorScheme::Light, FluentMaterial::Mica, 0xf3),
        (FluentColorScheme::Light, FluentMaterial::MicaAlt, 0xda),
        (FluentColorScheme::Dark, FluentMaterial::Mica, 0x20),
        (FluentColorScheme::Dark, FluentMaterial::MicaAlt, 0x0a),
        (
            FluentColorScheme::HighContrast,
            FluentMaterial::MicaAlt,
            0x00,
        ),
    ] {
        gallery.invoke_select_theme(scheme);
        gallery.invoke_select_backdrop(material);
        let frame = window.take_snapshot()?;
        check(
            frame.as_slice().first().is_some_and(|pixel| {
                [pixel.r, pixel.g, pixel.b, pixel.a] == [expected, expected, expected, 0xff]
            }),
            "material fallback renders the expected opaque color",
        )?;
    }
    verify_desktop_acrylic(gallery)?;
    gallery.invoke_select_backdrop(FluentMaterial::SolidFallback);
    let sample = gallery.get_acrylic_sample();
    let point = LogicalPosition::new(sample.x, sample.y);
    gallery.invoke_select_theme(FluentColorScheme::Light);
    check(
        read_rgb(window, point)? == [0xf9; 3],
        "light Acrylic fallback",
    )?;
    gallery.set_preview_acrylic(true);
    check(
        read_rgb(window, point)? != [0xf9; 3],
        "Skia draws the in-app Acrylic preview",
    )?;
    let seam = gallery.get_acrylic_seam();
    let color_delta = |offset| -> Result<u32, Box<dyn Error>> {
        let left = read_rgb(window, LogicalPosition::new(seam.x - offset, seam.y))?;
        let right = read_rgb(window, LogicalPosition::new(seam.x + offset, seam.y))?;
        Ok(left
            .into_iter()
            .zip(right)
            .map(|(a, b)| u32::from(a.abs_diff(b)))
            .sum())
    };
    check(
        color_delta(2.)? < color_delta(40.)?,
        "Skia smooths the hard edge behind the Acrylic preview",
    )?;
    gallery.invoke_select_theme(FluentColorScheme::HighContrast);
    check(
        read_rgb(window, point)? == [0x00; 3],
        "high contrast disables Acrylic blur",
    )?;
    gallery.set_preview_acrylic(false);
    gallery.invoke_select_theme(FluentColorScheme::Dark);
    check(
        read_rgb(window, point)? == [0x2c; 3],
        "dark Acrylic fallback",
    )
}

fn verify_desktop_acrylic(gallery: &GalleryWindow) -> Result<(), Box<dyn Error>> {
    gallery.invoke_select_theme(FluentColorScheme::HighContrast);
    gallery.invoke_select_backdrop(FluentMaterial::DesktopAcrylic);
    check(
        !gallery.get_native_desktop_acrylic(),
        "high contrast uses the Desktop Acrylic fallback",
    )?;
    let window = gallery.window();
    let read_background = || -> Result<[u8; 4], Box<dyn Error>> {
        let snapshot = window.take_snapshot()?;
        let pixel = snapshot
            .as_slice()
            .first()
            .ok_or("Skia returned an empty frame")?;
        Ok([pixel.r, pixel.g, pixel.b, pixel.a])
    };
    check(
        read_background()? == [0, 0, 0, 255],
        "Desktop Acrylic contrast fallback is opaque",
    )?;

    gallery.invoke_select_theme(FluentColorScheme::Light);
    #[cfg(target_os = "windows")]
    if crate::windows_backdrop::supported() {
        check(
            gallery.get_native_desktop_acrylic(),
            "DWM accepted Desktop Acrylic",
        )?;
    }
    if gallery.get_native_desktop_acrylic() {
        check(
            read_background()? == [0; 4],
            "Skia leaves the Acrylic background transparent",
        )?;
        gallery.invoke_select_theme(FluentColorScheme::Dark);
        check(
            gallery.get_native_desktop_acrylic(),
            "Desktop Acrylic remains active in the dark theme",
        )?;
        gallery.invoke_select_theme(FluentColorScheme::HighContrast);
        check(
            !gallery.get_native_desktop_acrylic() && read_background()? == [0, 0, 0, 255],
            "Desktop Acrylic resets to the contrast fallback",
        )?;
        gallery.invoke_select_theme(FluentColorScheme::Light);
        check(
            gallery.get_native_desktop_acrylic(),
            "Desktop Acrylic resumes after high contrast",
        )?;
    } else {
        check(
            read_background()? == [0xf9, 0xf9, 0xf9, 255],
            "unsupported Desktop Acrylic uses its light fallback",
        )?;
    }
    gallery.invoke_select_backdrop(FluentMaterial::SolidFallback);
    check(
        !gallery.get_native_desktop_acrylic() && read_background()? == [0xf3, 0xf3, 0xf3, 255],
        "switching from Desktop Acrylic restores the solid window",
    )
}

fn click(window: &Window, position: LogicalPosition) {
    let button = PointerEventButton::Left;
    window.dispatch_event(WindowEvent::PointerPressed { position, button });
    window.dispatch_event(WindowEvent::PointerReleased { position, button });
}

fn stop_event_loop() {
    if let Err(error) = slint::quit_event_loop() {
        eprintln!("Could not exit the Gallery event loop: {error}");
    }
}

fn check(condition: bool, description: &'static str) -> Result<(), Box<dyn Error>> {
    if !condition {
        return Err(description.into());
    }
    Ok(())
}
