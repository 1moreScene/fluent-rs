use std::{error::Error, time::Duration};

use slint::{
    ComponentHandle, LogicalPosition, LogicalSize, SharedString, Window,
    platform::{Key, PointerEventButton, WindowEvent},
};

use super::{check, click, read_rgb};
use crate::{
    ButtonState, FluentColorScheme, FluentTheme, GalleryPage, GalleryWindow, PasswordRevealMode,
};

type SmokeResult = Result<(), Box<dyn Error>>;
type BatchResult = Result<((u32, u32), (u32, u32)), Box<dyn Error>>;

pub(super) fn verify_batch(gallery: &GalleryWindow, done: impl FnOnce(BatchResult) + 'static) {
    let theme = gallery.global::<FluentTheme>();
    theme.set_reduced_motion(true);
    gallery.set_shell_pane_visible(false);
    let initial = (|| {
        verify_password(gallery)?;
        verify_switch(gallery)?;
        theme.set_ui_scale(1.);
        super::text_box::verify(gallery)?;
        verify_password(gallery)?;
        verify_switch(gallery)?;
        Ok::<_, Box<dyn Error>>(gallery.window().size())
    })();
    let wide = match initial {
        Ok(size) => (size.width, size.height),
        Err(error) => {
            done(Err(error));
            return;
        }
    };
    gallery.set_touch_targets(true);
    gallery.window().set_size(LogicalSize::new(320., 640.));
    let weak = gallery.as_weak();
    slint::Timer::single_shot(Duration::from_millis(250), move || {
        let Some(gallery) = weak.upgrade() else {
            done(Err(
                "Gallery closed before compact password verification".into()
            ));
            return;
        };
        if let Err(error) = verify_compact(&gallery, 320.) {
            done(Err(error));
            return;
        }
        let size = gallery.window().size();
        let compact = (size.width, size.height);
        verify_motion(&gallery, move |motion| {
            done(motion.map(|()| (wide, compact)))
        });
    });
}

pub(super) fn verify_password(gallery: &GalleryWindow) -> SmokeResult {
    gallery.set_page(GalleryPage::PasswordBox);
    gallery.set_password_enabled(true);
    gallery.set_password_reveal_mode(PasswordRevealMode::Peek);
    gallery.set_password("".into());
    gallery.invoke_select_theme(FluentColorScheme::Light);
    let window = gallery.window();
    window.take_snapshot()?;
    gallery.invoke_focus_password_input();
    window.take_snapshot()?;
    let edits = gallery.get_password_edits();
    key(window, "a");
    window.take_snapshot()?;
    check(
        gallery.get_password() == "a" && gallery.get_password_edits() == edits + 1,
        "PasswordBox uses the native editor and emits one change",
    )?;
    let probe = gallery.get_password_probe();
    check(
        probe.focused && probe.button_visible && !probe.revealed,
        "typing into an empty focused PasswordBox enables a masked peek button",
    )?;

    let masked = window.take_snapshot()?;
    verify_password_geometry(gallery)?;
    window.dispatch_event(WindowEvent::PointerMoved {
        position: probe.reveal,
    });
    window.dispatch_event(WindowEvent::PointerPressed {
        position: probe.reveal,
        button: PointerEventButton::Left,
    });
    let visible = window.take_snapshot()?;
    let scale = window.scale_factor();
    let mut text_pixels_changed = false;
    for y in ((probe.bounds.y + 4.) * scale) as usize
        ..((probe.bounds.y + probe.bounds.height - 4.) * scale) as usize
    {
        for x in ((probe.bounds.x + 8.) * scale) as usize..((probe.bounds.x + 32.) * scale) as usize
        {
            let offset = y * masked.width() as usize + x;
            text_pixels_changed |= masked.as_slice()[offset] != visible.as_slice()[offset];
        }
    }
    check(
        gallery.get_password_probe().revealed
            && gallery.get_password_probe().focused
            && text_pixels_changed,
        "holding the reveal button changes the rendered text without losing focus",
    )?;
    let outside = LogicalPosition::new(
        probe.bounds.x + 8.,
        probe.bounds.y + probe.bounds.height / 2.,
    );
    window.dispatch_event(WindowEvent::PointerMoved { position: outside });
    window.take_snapshot()?;
    check(
        !gallery.get_password_probe().revealed,
        "dragging out of the reveal button masks the password",
    )?;
    window.dispatch_event(WindowEvent::PointerReleased {
        position: outside,
        button: PointerEventButton::Left,
    });
    window.dispatch_event(WindowEvent::KeyPressed {
        text: Key::Alt.into(),
    });
    window.dispatch_event(WindowEvent::KeyPressed {
        text: Key::F8.into(),
    });
    check(
        gallery.get_password_probe().revealed,
        "Alt+F8 reveals an eligible password",
    )?;
    window.dispatch_event(WindowEvent::KeyReleased {
        text: Key::F8.into(),
    });
    window.dispatch_event(WindowEvent::KeyReleased {
        text: Key::Alt.into(),
    });
    check(
        !gallery.get_password_probe().revealed,
        "releasing F8 masks the password",
    )?;

    key(window, Key::Tab);
    window.take_snapshot()?;
    check(
        gallery.get_password_following_focused() && !gallery.get_password_probe().button_visible,
        "Tab skips the reveal button and loss of focus hides it",
    )?;
    gallery.invoke_focus_password_input();
    window.take_snapshot()?;
    check(
        !gallery.get_password_probe().button_visible,
        "refocusing an existing password does not expose peek",
    )?;
    key(window, "b");
    window.take_snapshot()?;
    check(
        !gallery.get_password_probe().button_visible,
        "appending to a pre-filled password keeps peek hidden",
    )?;
    gallery.invoke_select_password_input();
    key(window, Key::Backspace);
    key(window, "密");
    window.take_snapshot()?;
    check(
        gallery.get_password_probe().button_visible && gallery.get_password() == "密",
        "clearing and typing again enables peek with Unicode input",
    )?;

    for scheme in [
        FluentColorScheme::Light,
        FluentColorScheme::Dark,
        FluentColorScheme::HighContrast,
    ] {
        gallery.invoke_select_theme(scheme);
        window.take_snapshot()?;
        let bounds = gallery.get_password_probe().bounds;
        let color = read_rgb(
            window,
            LogicalPosition::new(bounds.x + bounds.width / 2., bounds.y + bounds.height - 1.),
        )?;
        let theme = gallery.global::<FluentTheme>();
        let expected = if scheme == FluentColorScheme::HighContrast {
            theme.get_contrast_highlight()
        } else {
            theme.get_accent_fill()
        };
        check(
            color == [expected.red(), expected.green(), expected.blue()],
            "PasswordBox shares the themed focus underline",
        )?;
    }
    gallery.invoke_select_theme(FluentColorScheme::Light);
    gallery.set_password_reveal_mode(PasswordRevealMode::Hidden);
    window.take_snapshot()?;
    check(
        !gallery.get_password_probe().button_visible && !gallery.get_password_probe().revealed,
        "Hidden suppresses both the reveal button and disclosure",
    )?;
    gallery.set_password_reveal_mode(PasswordRevealMode::Visible);
    window.take_snapshot()?;
    check(
        gallery.get_password_probe().revealed && !gallery.get_password_probe().button_visible,
        "Visible shows the password without a peek button",
    )?;
    let submitted = gallery.get_password_submissions();
    key(window, Key::Return);
    check(
        gallery.get_password_submissions() == submitted + 1,
        "Enter submits PasswordBox once",
    )?;

    #[cfg(target_os = "macos")]
    verify_password_clipboard(gallery)?;

    gallery.set_password_reveal_mode(PasswordRevealMode::Hidden);
    gallery.set_password("Long masked input ".repeat(40).into());
    gallery.invoke_select_password_input();
    window.take_snapshot()?;
    line_edge(window, false);
    window.take_snapshot()?;
    check(
        gallery.get_password_probe().scroll > 0.,
        "long passwords scroll the existing editor to the caret",
    )?;
    line_edge(window, true);
    window.take_snapshot()?;
    check(
        gallery.get_password_probe().scroll < 1.,
        "moving to the first password character restores the viewport",
    )?;

    gallery.set_password_enabled(false);
    window.take_snapshot()?;
    let before = gallery.get_password();
    click(
        window,
        LogicalPosition::new(
            probe.bounds.x + 20.,
            probe.bounds.y + probe.bounds.height / 2.,
        ),
    );
    key(window, "z");
    check(
        gallery.get_password() == before && !gallery.get_password_probe().button_visible,
        "a disabled PasswordBox rejects input and peek",
    )?;
    gallery.set_password_enabled(true);
    gallery.set_password_reveal_mode(PasswordRevealMode::Peek);
    gallery.set_password("preset".into());
    gallery.invoke_focus_password_input();
    window.take_snapshot()?;
    check(
        !gallery.get_password_probe().button_visible,
        "programmatically supplied passwords keep peek hidden",
    )?;
    gallery.set_page(GalleryPage::Button);
    window.take_snapshot()?;
    check(
        !gallery.get_password_probe().revealed && !gallery.get_password_probe().focused,
        "leaving the password page clears transient disclosure and focus",
    )
}

#[cfg(target_os = "macos")]
fn verify_password_clipboard(gallery: &GalleryWindow) -> SmokeResult {
    let mut clipboard = super::clipboard::ClipboardGuard::new()?;
    let result = (|| {
        for mode in [PasswordRevealMode::Hidden, PasswordRevealMode::Visible] {
            gallery.set_password_reveal_mode(mode);
            gallery.set_password("clipboard verification".into());
            gallery.invoke_select_password_input();
            gallery.window().take_snapshot()?;
            clipboard.set_text("clipboard sentinel")?;
            for letter in ["c", "x"] {
                shortcut(gallery.window(), letter);
                gallery.window().take_snapshot()?;
                clipboard.recognize_test_write("clipboard verification");
                check(
                    clipboard.text().as_deref() == Some("clipboard sentinel")
                        && gallery.get_password() == "clipboard verification",
                    "password copy and cut remain blocked in masked and visible modes",
                )?;
            }
            clipboard.set_text("pasted value")?;
            shortcut(gallery.window(), "v");
            gallery.window().take_snapshot()?;
            check(
                gallery.get_password() == "pasted value",
                "PasswordBox accepts a system clipboard paste",
            )?;
        }
        Ok(())
    })();
    let restored = clipboard.restore();
    result.and(restored)
}

pub(super) fn verify_switch(gallery: &GalleryWindow) -> SmokeResult {
    gallery.set_page(GalleryPage::ToggleSwitch);
    gallery.set_switch_enabled(true);
    gallery.set_switch_on(false);
    let window = gallery.window();
    window.take_snapshot()?;
    let probe = gallery.get_switch_probe();
    let ui_scale = gallery.global::<FluentTheme>().get_ui_scale();
    let center = LogicalPosition::new(
        probe.track.x + 20. * ui_scale,
        probe.track.y + 10. * ui_scale,
    );
    let toggles = gallery.get_switch_toggles();
    click(window, center);
    window.take_snapshot()?;
    check(
        gallery.get_switch_on()
            && gallery.get_switch_toggles() == toggles + 1
            && !gallery.get_switch_probe().focus_visible,
        "clicking ToggleSwitch toggles once and uses pointer focus",
    )?;
    key(window, Key::Return);
    check(
        gallery.get_switch_on(),
        "Enter does not activate ToggleSwitch",
    )?;
    window.dispatch_event(WindowEvent::KeyPressed {
        text: Key::Space.into(),
    });
    check(gallery.get_switch_on(), "Space waits for release")?;
    window.dispatch_event(WindowEvent::KeyReleased {
        text: Key::Space.into(),
    });
    window.take_snapshot()?;
    check(
        !gallery.get_switch_on() && gallery.get_switch_probe().focus_visible,
        "Space release toggles with keyboard focus",
    )?;
    let toggles = gallery.get_switch_toggles();
    gallery.set_switch_on(true);
    window.take_snapshot()?;
    check(
        gallery.get_switch_toggles() == toggles + 1,
        "programmatic IsOn changes emit the same toggle notification",
    )?;
    gallery.set_switch_on(false);
    window.take_snapshot()?;
    let header = LogicalPosition::new(probe.bounds.x + 5., probe.bounds.y - 10. * ui_scale);
    click(window, header);
    check(
        !gallery.get_switch_on(),
        "the header is outside ToggleSwitch's thumb hit region",
    )?;
    click(
        window,
        LogicalPosition::new(probe.bounds.x + probe.bounds.width - 5., center.y),
    );
    check(gallery.get_switch_on(), "clicking the On/Off label toggles")?;

    gallery.set_switch_on(false);
    window.take_snapshot()?;
    window.dispatch_event(WindowEvent::PointerPressed {
        position: center,
        button: PointerEventButton::Left,
    });
    window.dispatch_event(WindowEvent::PointerMoved {
        position: LogicalPosition::new(center.x + 15. * ui_scale, center.y),
    });
    check(
        gallery.get_switch_probe().dragging,
        "horizontal pointer movement drags the shared thumb",
    )?;
    window.dispatch_event(WindowEvent::PointerReleased {
        position: LogicalPosition::new(center.x + 15. * ui_scale, center.y),
        button: PointerEventButton::Left,
    });
    window.take_snapshot()?;
    check(
        gallery.get_switch_on(),
        "a drag beyond the midpoint commits On without an extra click toggle",
    )?;
    window.dispatch_event(WindowEvent::PointerPressed {
        position: center,
        button: PointerEventButton::Left,
    });
    window.dispatch_event(WindowEvent::PointerMoved {
        position: LogicalPosition::new(center.x - 15. * ui_scale, center.y),
    });
    gallery.set_switch_enabled(false);
    window.take_snapshot()?;
    window.dispatch_event(WindowEvent::PointerReleased {
        position: LogicalPosition::new(center.x - 15. * ui_scale, center.y),
        button: PointerEventButton::Left,
    });
    check(
        gallery.get_switch_on() && !gallery.get_switch_probe().dragging,
        "disabling mid-drag cancels the pending change",
    )?;
    gallery.set_switch_enabled(true);

    for scheme in [
        FluentColorScheme::Light,
        FluentColorScheme::Dark,
        FluentColorScheme::HighContrast,
    ] {
        gallery.invoke_select_theme(scheme);
        window.dispatch_event(WindowEvent::PointerMoved {
            position: LogicalPosition::new(1., 1.),
        });
        window.take_snapshot()?;
        let probe = gallery.get_switch_probe();
        let expected = if scheme == FluentColorScheme::HighContrast {
            gallery.global::<FluentTheme>().get_contrast_highlight()
        } else {
            gallery.global::<FluentTheme>().get_accent_fill()
        };
        check(
            read_rgb(
                window,
                LogicalPosition::new(
                    probe.track.x + 10. * ui_scale,
                    probe.track.y + 10. * ui_scale,
                ),
            )? == [expected.red(), expected.green(), expected.blue()],
            "the On track uses the source theme's accent fill",
        )?;
        let normal = knob_span(gallery)?;
        window.dispatch_event(WindowEvent::PointerMoved {
            position: probe.knob,
        });
        let hover = knob_span(gallery)?;
        window.dispatch_event(WindowEvent::PointerPressed {
            position: probe.knob,
            button: PointerEventButton::Left,
        });
        let pressed = knob_span(gallery)?;
        check(
            hover > normal && pressed > hover,
            "the knob expands from 12 to 14 to 17 logical pixels in Skia",
        )?;
        window.dispatch_event(WindowEvent::PointerMoved {
            position: LogicalPosition::new(1., 1.),
        });
        gallery.set_switch_enabled(false);
        window.dispatch_event(WindowEvent::PointerReleased {
            position: LogicalPosition::new(1., 1.),
            button: PointerEventButton::Left,
        });
        window.take_snapshot()?;
        check(
            gallery.get_switch_probe().state == ButtonState::Disabled
                && !gallery.get_switch_probe().focus_visible,
            "disabled ToggleSwitch suppresses its focus ring",
        )?;
        gallery.set_switch_enabled(true);
    }
    gallery.invoke_select_theme(FluentColorScheme::Light);
    gallery.set_switch_on(false);
    Ok(())
}

fn knob_span(gallery: &GalleryWindow) -> Result<usize, Box<dyn Error>> {
    let probe = gallery.get_switch_probe();
    let scale = gallery.window().scale_factor();
    let ui_scale = gallery.global::<FluentTheme>().get_ui_scale();
    let image = gallery.window().take_snapshot()?;
    let color = if gallery.global::<FluentTheme>().get_high_contrast() {
        if matches!(probe.state, ButtonState::PointerOver | ButtonState::Pressed) {
            gallery.global::<FluentTheme>().get_contrast_button_text()
        } else {
            gallery
                .global::<FluentTheme>()
                .get_contrast_highlight_text()
        }
    } else {
        gallery.global::<FluentTheme>().get_accent_text()
    };
    let row = ((probe.track.y + 10. * ui_scale) * scale) as usize;
    let left = (probe.track.x * scale).ceil() as usize;
    let right = ((probe.track.x + 40. * ui_scale) * scale).floor() as usize;
    Ok((left..right)
        .filter(|&column| {
            let pixel = image.as_slice()[row * image.width() as usize + column];
            [pixel.r, pixel.g, pixel.b]
                .into_iter()
                .zip([color.red(), color.green(), color.blue()])
                .all(|(actual, target)| actual.abs_diff(target) <= 3)
        })
        .count())
}

fn verify_password_geometry(gallery: &GalleryWindow) -> SmokeResult {
    let window = gallery.window();
    let frame = window.take_snapshot()?;
    let probe = gallery.get_password_probe();
    let backing_scale = window.scale_factor();
    let ui_scale = gallery.global::<FluentTheme>().get_ui_scale();
    let dark_bounds = |left: f32, top: f32, right: f32, bottom: f32| {
        let mut points = Vec::new();
        for y in (top * backing_scale).ceil() as usize..(bottom * backing_scale).floor() as usize {
            for x in
                (left * backing_scale).ceil() as usize..(right * backing_scale).floor() as usize
            {
                let pixel = frame.as_slice()[y * frame.width() as usize + x];
                if pixel.r < 170 && pixel.g < 170 && pixel.b < 170 {
                    points.push((x, y));
                }
            }
        }
        let min_x = points.iter().map(|p| p.0).min()?;
        let max_x = points.iter().map(|p| p.0).max()?;
        let min_y = points.iter().map(|p| p.1).min()?;
        let max_y = points.iter().map(|p| p.1).max()?;
        Some((min_x, min_y, max_x, max_y))
    };
    let dot = dark_bounds(
        probe.bounds.x + 11. * ui_scale,
        probe.bounds.y + 4. * ui_scale,
        probe.bounds.x + 19. * ui_scale,
        probe.bounds.y + probe.bounds.height - 4. * ui_scale,
    )
    .ok_or("the masked password should render a black circle")?;
    let diameter = (dot.3 - dot.1 + 1) as f32 / (backing_scale * ui_scale);
    check(
        (4. ..=8.).contains(&diameter),
        "the password circle uses the compact mask-font metrics",
    )?;
    let center = LogicalPosition::new(probe.reveal.x - 2. * ui_scale, probe.reveal.y);
    let eye = dark_bounds(
        center.x - 8. * ui_scale,
        center.y - 8. * ui_scale,
        center.x + 8. * ui_scale,
        center.y + 8. * ui_scale,
    )
    .ok_or("the reveal button should render its eye glyph")?;
    let actual = LogicalPosition::new(
        (eye.0 + eye.2 + 1) as f32 / (2. * backing_scale),
        (eye.1 + eye.3 + 1) as f32 / (2. * backing_scale),
    );
    if (actual.x - center.x).abs() > 0.8 || (actual.y - center.y).abs() > 0.8 {
        return Err(format!("reveal glyph should center at {center:?}, got {actual:?}").into());
    }
    println!("PasswordBox circle diameter {diameter:.2} logical pixels; reveal glyph aligned");
    Ok(())
}

pub(super) fn verify_compact(gallery: &GalleryWindow, width: f32) -> SmokeResult {
    gallery.set_page(GalleryPage::PasswordBox);
    gallery.window().take_snapshot()?;
    let password = gallery.get_password_probe();
    check(
        password.bounds.x >= 0.
            && password.bounds.x + password.bounds.width <= width
            && password.bounds.height >= 44.,
        "compact PasswordBox fits and keeps a touch-sized input",
    )?;
    gallery.set_page(GalleryPage::ToggleSwitch);
    gallery.set_switch_long_label(true);
    gallery.window().take_snapshot()?;
    let switch = gallery.get_switch_probe();
    check(
        switch.bounds.x >= 0.
            && switch.bounds.x + switch.bounds.width <= width
            && switch.bounds.height >= 44.,
        "compact ToggleSwitch wraps its state label within the viewport",
    )?;
    gallery.set_switch_long_label(false);
    Ok(())
}

pub(super) fn verify_motion(gallery: &GalleryWindow, done: impl FnOnce(SmokeResult) + 'static) {
    gallery.set_page(GalleryPage::ToggleSwitch);
    gallery.set_switch_long_label(false);
    gallery.set_switch_enabled(true);
    gallery.invoke_select_theme(FluentColorScheme::Light);
    let theme = gallery.global::<FluentTheme>();
    theme.set_reduced_motion(true);
    gallery.set_switch_on(false);
    if let Err(error) = gallery.window().take_snapshot() {
        done(Err(error.into()));
        return;
    }
    let off = gallery.get_switch_probe().knob.x;
    theme.set_reduced_motion(false);
    gallery.set_switch_on(true);
    if let Err(error) = gallery.window().take_snapshot() {
        done(Err(error.into()));
        return;
    }
    let weak = gallery.as_weak();
    slint::Timer::single_shot(Duration::from_millis(300), move || {
        let result = (|| {
            let gallery = weak
                .upgrade()
                .ok_or("Gallery closed before switch motion completed")?;
            gallery.window().take_snapshot()?;
            let travel = 20. * gallery.global::<FluentTheme>().get_ui_scale();
            check(
                (gallery.get_switch_probe().knob.x - off - travel).abs() < 1.,
                "ToggleSwitch's finite reposition reaches the opposite slot",
            )?;
            gallery.global::<FluentTheme>().set_reduced_motion(true);
            gallery.set_switch_on(false);
            gallery.window().take_snapshot()?;
            check(
                (gallery.get_switch_probe().knob.x - off).abs() < 1.,
                "reduced motion places the knob immediately",
            )
        })();
        done(result);
    });
}

fn key(window: &Window, text: impl Into<SharedString>) {
    let text = text.into();
    window.dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    window.dispatch_event(WindowEvent::KeyReleased { text });
}

fn line_edge(window: &Window, start: bool) {
    #[cfg(target_os = "macos")]
    shortcut(
        window,
        &SharedString::from(if start {
            Key::LeftArrow
        } else {
            Key::RightArrow
        }),
    );
    #[cfg(not(target_os = "macos"))]
    key(window, if start { Key::Home } else { Key::End });
}

#[cfg(target_os = "macos")]
fn shortcut(window: &Window, text: &str) {
    window.dispatch_event(WindowEvent::KeyPressed {
        text: Key::Control.into(),
    });
    key(window, text);
    window.dispatch_event(WindowEvent::KeyReleased {
        text: Key::Control.into(),
    });
}
