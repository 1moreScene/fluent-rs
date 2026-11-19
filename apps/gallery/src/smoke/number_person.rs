use std::{error::Error, time::Duration};

use super::{check, click, read_rgb};
use crate::{
    FluentColorScheme, FluentTheme, GalleryPage, GalleryWindow,
    NumberBoxSpinButtonPlacement as Placement, NumberBoxValidationMode as Validation,
    PersonPictureState, TextBoxState,
};
use slint::{
    ComponentHandle, LogicalPosition, LogicalSize, SharedString, Window,
    platform::{Key, PointerEventButton, WindowEvent},
};

type SmokeResult = Result<(), Box<dyn Error>>;
type BatchResult = Result<((u32, u32), (u32, u32)), Box<dyn Error>>;

fn key(window: &Window, text: impl Into<SharedString>) {
    let text = text.into();
    window.dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    window.dispatch_event(WindowEvent::KeyReleased { text });
}

pub(super) fn verify(gallery: &GalleryWindow) -> SmokeResult {
    verify_number(gallery)?;
    verify_person(gallery)
}

fn typed(gallery: &GalleryWindow, value: &str) -> SmokeResult {
    gallery.invoke_select_number_input();
    gallery.window().take_snapshot()?;
    for ch in value.chars() {
        key(gallery.window(), ch.to_string());
    }
    if value.is_empty() {
        key(gallery.window(), Key::Backspace);
    }
    gallery.window().take_snapshot()?;
    Ok(())
}

fn verify_number(gallery: &GalleryWindow) -> SmokeResult {
    gallery.set_page(GalleryPage::NumberBox);
    gallery.set_number_enabled(true);
    gallery.set_number_placement(Placement::Hidden);
    gallery.set_number_validation(Validation::InvalidInputOverwritten);
    gallery.set_number_expression(true);
    gallery.set_number_wrap(false);
    gallery.set_number_minimum("0".into());
    gallery.set_number_maximum("100".into());
    gallery.set_number_small_change("1".into());
    gallery.set_number_large_change("10".into());
    gallery.set_number_decimal_separator(".".into());
    gallery.set_number_fraction_digits(0);
    gallery.set_number_value("10".into());
    gallery.invoke_select_theme(FluentColorScheme::Light);
    let window = gallery.window();
    window.take_snapshot()?;
    if !gallery.get_number_probe().error.is_empty() {
        return Err(format!(
            "NumberBox model configuration: {}",
            gallery.get_number_probe().error
        )
        .into());
    }
    check(
        gallery.get_number_text() == "10",
        "NumberBox formats a programmatic value through its Rust model",
    )?;
    gallery.invoke_focus_number_input();
    window.take_snapshot()?;
    check(
        gallery.get_number_probe().selection,
        "NumberBox selects all text on initial focus",
    )?;
    typed(gallery, "1 + 2^2")?;
    check(
        gallery.get_number_value() == "10" && gallery.get_number_text() == "1 + 2^2",
        "editing defers number validation until commit",
    )?;
    let changes = gallery.get_number_changes();
    window.dispatch_event(WindowEvent::KeyPressed {
        text: Key::Return.into(),
    });
    check(
        gallery.get_number_value() == "10",
        "Enter key-down does not commit before release",
    )?;
    window.dispatch_event(WindowEvent::KeyReleased {
        text: Key::Return.into(),
    });
    window.take_snapshot()?;
    check(
        gallery.get_number_value() == "5"
            && gallery.get_number_text() == "5"
            && gallery.get_number_changes() == changes + 1,
        "Enter release evaluates the expression and notifies exactly once",
    )?;
    gallery.set_number_text("2+3".into());
    window.take_snapshot()?;
    check(
        gallery.get_number_value() == "5"
            && gallery.get_number_text() == "5"
            && gallery.get_number_changes() == changes + 1,
        "a programmatic equivalent expression normalizes text without an extra value event",
    )?;
    typed(gallery, "invalid")?;
    key(window, Key::Escape);
    window.take_snapshot()?;
    check(
        gallery.get_number_text() == "5" && gallery.get_number_value() == "5",
        "Escape restores the current numeric value",
    )?;
    typed(gallery, "invalid")?;
    key(window, Key::Return);
    window.take_snapshot()?;
    check(
        gallery.get_number_text() == "5",
        "default validation overwrites invalid text",
    )?;
    gallery.set_number_validation(Validation::Disabled);
    typed(gallery, "invalid")?;
    key(window, Key::Return);
    window.take_snapshot()?;
    check(
        gallery.get_number_text() == "invalid" && gallery.get_number_value() == "5",
        "disabled validation retains invalid text and the previous value",
    )?;
    key(window, Key::UpArrow);
    window.take_snapshot()?;
    check(
        gallery.get_number_value() == "6" && gallery.get_number_text() == "6",
        "stepping validates first and then changes the previous value",
    )?;
    gallery.set_number_validation(Validation::InvalidInputOverwritten);
    key(window, Key::PageUp);
    window.take_snapshot()?;
    check(
        gallery.get_number_value() == "16",
        "PageUp uses LargeChange",
    )?;
    gallery.set_number_maximum("10".into());
    window.take_snapshot()?;
    check(
        gallery.get_number_value() == "10" && !gallery.get_number_probe().up_enabled,
        "Maximum clamps the value and disables the up action",
    )?;
    let changed = gallery.get_number_changes();
    gallery.set_number_value("999".into());
    window.take_snapshot()?;
    check(
        gallery.get_number_value() == "10" && gallery.get_number_changes() == changed,
        "assigning an out-of-range value equal after coercion does not duplicate notification",
    )?;
    gallery.set_number_minimum("12".into());
    window.take_snapshot()?;
    check(
        gallery.get_number_maximum() == "12" && gallery.get_number_value() == "12",
        "changing Minimum raises Maximum and coerces once",
    )?;
    gallery.set_number_maximum("8".into());
    window.take_snapshot()?;
    check(
        gallery.get_number_minimum() == "8" && gallery.get_number_value() == "8",
        "changing Maximum lowers Minimum",
    )?;
    gallery.set_number_minimum("0".into());
    gallery.set_number_maximum("10".into());
    gallery.set_number_value("10".into());
    gallery.set_number_wrap(true);
    window.take_snapshot()?;
    key(window, Key::UpArrow);
    window.take_snapshot()?;
    check(
        gallery.get_number_value() == "0",
        "wrap jumps an upper overshoot to Minimum",
    )?;
    key(window, Key::DownArrow);
    window.take_snapshot()?;
    check(
        gallery.get_number_value() == "10",
        "wrap jumps a lower overshoot to Maximum",
    )?;
    typed(gallery, "")?;
    key(window, Key::UpArrow);
    window.take_snapshot()?;
    check(
        gallery.get_number_value().is_empty()
            && !gallery.get_number_probe().up_enabled
            && !gallery.get_number_probe().down_enabled,
        "empty input commits NaN and cannot be stepped into zero",
    )?;
    let changed = gallery.get_number_changes();
    gallery.invoke_commit_number_input();
    window.take_snapshot()?;
    check(
        gallery.get_number_changes() == changed,
        "empty-to-empty does not notify",
    )?;

    gallery.set_number_minimum("".into());
    gallery.set_number_maximum("".into());
    gallery.set_number_wrap(false);
    gallery.set_number_value("9007199254740991".into());
    window.take_snapshot()?;
    check(
        gallery.get_number_value() == "9007199254740991"
            && gallery.get_number_text() == "9007199255000000",
        "full f64 value survives both Slint transport and ten-significant-digit display rounding",
    )?;
    gallery.set_number_value("12.5".into());
    gallery.set_number_decimal_separator(",".into());
    gallery.set_number_fraction_digits(2);
    window.take_snapshot()?;
    check(
        gallery.get_number_text() == "12,50",
        "decimal formatting supports a caller-selected separator and minimum fraction digits",
    )?;
    typed(gallery, "1,25")?;
    key(window, Key::Return);
    window.take_snapshot()?;
    check(
        gallery.get_number_value() == "1.25" && gallery.get_number_text() == "1,25",
        "regional display text remains separate from canonical numeric state",
    )?;
    gallery.set_number_decimal_separator(".".into());
    gallery.set_number_fraction_digits(0);
    gallery.set_number_minimum("0".into());
    gallery.set_number_maximum("100".into());
    gallery.set_number_value("5".into());
    gallery.invoke_leave_number_input();
    gallery.set_number_placement(Placement::Inline);
    window.take_snapshot()?;
    let probe = gallery.get_number_probe();
    let ui = gallery.global::<FluentTheme>().get_ui_scale();
    check(
        (probe.down.x - probe.up.x - 36. * ui).abs() < 0.75
            && (probe.up.y - probe.input.y - probe.input.height / 2.).abs() < 0.75,
        "inline NumberBox spin buttons use the official columns and vertical alignment",
    )?;
    click(window, probe.up);
    window.take_snapshot()?;
    check(
        gallery.get_number_value() == "6" && gallery.get_number_probe().focused,
        "inline spin uses the shared RepeatButton and keeps editing available",
    )?;
    window.dispatch_event(WindowEvent::PointerScrolled {
        position: probe.up,
        delta_x: 0.,
        delta_y: 120.,
    });
    window.take_snapshot()?;
    // Wheel on the actual editing region is handled only while the editor owns focus.
    let text_point = LogicalPosition::new(
        probe.input.x + 20. * ui,
        probe.input.y + probe.input.height / 2.,
    );
    let before = gallery.get_number_value().parse::<f64>()?;
    window.dispatch_event(WindowEvent::PointerScrolled {
        position: text_point,
        delta_x: 0.,
        delta_y: 120.,
    });
    window.take_snapshot()?;
    check(
        gallery.get_number_value().parse::<f64>()? == before + 1.,
        "focused NumberBox consumes vertical wheel steps",
    )?;
    gallery.invoke_leave_number_input();
    let value = gallery.get_number_value();
    window.dispatch_event(WindowEvent::PointerScrolled {
        position: text_point,
        delta_x: 0.,
        delta_y: 120.,
    });
    window.take_snapshot()?;
    check(
        gallery.get_number_value() == value,
        "unfocused NumberBox leaves the wheel to its scroll container",
    )?;
    gallery.invoke_focus_number_input();
    window.take_snapshot()?;
    let probe = gallery.get_number_probe();
    let clear = LogicalPosition::new(
        probe.input.x + probe.input.width - 72. * ui - 20. * ui,
        probe.input.y + probe.input.height / 2.,
    );
    click(window, clear);
    window.take_snapshot()?;
    check(
        gallery.get_number_text().is_empty(),
        "NumberBox clear button is positioned before the inline spin column",
    )?;
    gallery.set_number_value("10".into());
    gallery.invoke_leave_number_input();
    gallery.set_number_placement(Placement::Compact);
    gallery.invoke_focus_number_input();
    window.take_snapshot()?;
    let popup = gallery.get_number_probe();
    check(
        popup.open && popup.focused,
        "Compact opens on focus without replacing the native editor",
    )?;
    check(
        (popup.panel.x - popup.input.x - popup.input.width + 21. * ui).abs() < 1.
            && (popup.panel.y - popup.input.y + 27. * ui).abs() < 1.,
        "Compact popup uses the official -21/-27 position offsets",
    )?;
    click(window, popup.up);
    window.take_snapshot()?;
    check(
        gallery.get_number_value() == "11" && gallery.get_number_probe().focused,
        "Compact spin increments without losing input focus",
    )?;
    typed(gallery, "7")?;
    key(window, Key::Return);
    window.take_snapshot()?;
    check(
        gallery.get_number_value() == "7",
        "the native editor continues receiving text while Compact is open",
    )?;
    typed(gallery, "bad")?;
    key(window, Key::Escape);
    window.take_snapshot()?;
    check(
        gallery.get_number_text() == "7"
            && gallery.get_number_probe().open
            && gallery.get_number_probe().focused,
        "Escape restores text and retains the focused Compact popup",
    )?;
    key(window, Key::Tab);
    window.take_snapshot()?;
    check(
        gallery.get_number_following_focused() && !gallery.get_number_probe().open,
        "Tab skips every spin button and closes Compact on focus loss",
    )?;

    gallery.invoke_focus_number_input();
    window.take_snapshot()?;
    typed(gallery, "8+1")?;
    click(window, LogicalPosition::new(10., 10.));
    window.take_snapshot()?;
    check(
        gallery.get_number_value() == "9"
            && !gallery.get_number_probe().open
            && !gallery.get_number_probe().focused,
        "outside press commits Compact and clears focus after popup restoration",
    )?;

    gallery.set_number_placement(Placement::Inline);
    for scheme in [
        FluentColorScheme::Light,
        FluentColorScheme::Dark,
        FluentColorScheme::HighContrast,
    ] {
        gallery.invoke_select_theme(scheme);
        gallery.invoke_focus_number_input();
        window.take_snapshot()?;
        let p = gallery.get_number_probe();
        let sample = LogicalPosition::new(
            p.input.x + p.input.width / 2.,
            p.input.y + p.input.height - 1.5 * ui,
        );
        let color = if scheme == FluentColorScheme::HighContrast {
            gallery.global::<FluentTheme>().get_contrast_highlight()
        } else {
            gallery.global::<FluentTheme>().get_accent_fill()
        };
        check(
            read_rgb(window, sample)? == [color.red(), color.green(), color.blue()],
            "NumberBox reuses the real themed TextBox focus chrome",
        )?;
        gallery.set_number_enabled(false);
        window.take_snapshot()?;
        check(
            gallery.get_number_probe().state == TextBoxState::Disabled
                && !gallery.get_number_probe().open,
            "disabled NumberBox closes Compact and disables the editor",
        )?;
        gallery.set_number_enabled(true);
    }
    gallery.invoke_select_theme(FluentColorScheme::Light);
    Ok(())
}

fn verify_person(gallery: &GalleryWindow) -> SmokeResult {
    gallery.set_page(GalleryPage::PersonPicture);
    gallery.set_person_size(96.);
    gallery.set_person_name("Jane Doe".into());
    gallery.set_person_initials("".into());
    gallery.set_person_photo(false);
    gallery.set_person_group(false);
    gallery.set_person_badge_image(false);
    gallery.set_person_badge_number(0);
    gallery.set_person_badge_glyph("".into());
    let window = gallery.window();
    window.take_snapshot()?;
    check(
        gallery.get_person_probe().state == PersonPictureState::Initials
            && gallery.get_person_probe().initials == "JD",
        "PersonPicture generates initials from the display name",
    )?;
    gallery.set_person_initials("SB".into());
    window.take_snapshot()?;
    check(
        gallery.get_person_probe().initials == "SB",
        "explicit initials override the display name",
    )?;
    gallery.set_person_photo(true);
    window.take_snapshot()?;
    check(
        gallery.get_person_probe().state == PersonPictureState::Photo,
        "profile image has priority over initials",
    )?;
    verify_photo_crop(gallery)?;
    gallery.set_person_group(true);
    window.take_snapshot()?;
    check(
        gallery.get_person_probe().state == PersonPictureState::Group,
        "Group overrides photo and initials without deleting them",
    )?;
    gallery.set_person_group(false);
    window.take_snapshot()?;
    check(
        gallery.get_person_probe().state == PersonPictureState::Photo,
        "leaving Group restores the original photo",
    )?;
    gallery.set_person_photo(false);
    gallery.set_person_initials("".into());
    for (name, expected) in [
        ("Jane Doe", "JD"),
        ("John Smith (OSG)", "JS"),
        ("张三", ""),
        ("محمد", ""),
        ("e\u{301}mile Zola", "E\u{301}Z"),
    ] {
        gallery.set_person_name(name.into());
        window.take_snapshot()?;
        check(
            gallery.get_person_probe().initials == expected,
            "PersonPicture follows official name grouping and safe Unicode fallback",
        )?;
    }
    gallery.set_person_badge_glyph("!".into());
    gallery.set_person_badge_number(3);
    window.take_snapshot()?;
    check(
        gallery.get_person_probe().badge_visible && gallery.get_person_probe().badge_label == "3",
        "number badge overrides glyph",
    )?;
    gallery.set_person_badge_number(120);
    window.take_snapshot()?;
    check(
        gallery.get_person_probe().badge_label == "99+"
            && gallery.get_person_probe().accessible_name.contains("120"),
        "badge caps its visual label while keeping the actual count accessible",
    )?;
    gallery.set_person_badge_image(true);
    window.take_snapshot()?;
    let p = gallery.get_person_probe();
    check(
        read_rgb(
            window,
            LogicalPosition::new(
                p.badge.x + p.badge.width * 0.25,
                p.badge.y + p.badge.height * 0.25,
            ),
        )? == [231, 111, 81],
        "badge image overrides numeric and glyph rendering",
    )?;
    gallery.set_person_badge_image(false);
    gallery.set_person_badge_number(-1);
    window.take_snapshot()?;
    check(
        !gallery.get_person_probe().badge_visible,
        "negative badge numbers hide instead of falling back to the glyph",
    )?;
    gallery.set_person_badge_number(0);
    window.take_snapshot()?;
    check(
        gallery.get_person_probe().badge_visible && gallery.get_person_probe().badge_label == "!",
        "zero number allows the glyph badge",
    )?;
    gallery.set_person_badge_number(3);
    gallery.set_person_name("Jane Doe".into());
    let ui = gallery.global::<FluentTheme>().get_ui_scale();
    for size in [32., 64., 96.] {
        gallery.set_person_size(size);
        window.take_snapshot()?;
        let p = gallery.get_person_probe();
        check(
            (p.bounds.width - size * ui).abs() < 0.75
                && (p.initials_size - p.bounds.width * 0.42).abs() < 0.1
                && (p.badge.width - p.bounds.width * 0.5).abs() < 0.1
                && (p.badge.x + p.badge.width - p.bounds.x - p.bounds.width - 4. * ui).abs() < 0.1
                && (p.badge.y - p.bounds.y + 4. * ui).abs() < 0.1,
            "PersonPicture scales its circle, initials and top-right badge from official proportions",
        )?;
    }
    gallery.set_person_size(96.);
    gallery.set_person_badge_glyph("".into());
    gallery.set_person_badge_number(0);
    gallery.set_person_name("".into());
    window.take_snapshot()?;
    check(
        gallery.get_person_probe().state == PersonPictureState::Person,
        "empty image and name use the generic person vector",
    )?;
    for theme in [
        FluentColorScheme::Light,
        FluentColorScheme::Dark,
        FluentColorScheme::HighContrast,
    ] {
        gallery.invoke_select_theme(theme);
        gallery.set_person_photo(true);
        window.take_snapshot()?;
        verify_photo_crop(gallery)?;
        gallery.set_person_photo(false);
        window.take_snapshot()?;
        let before = gallery.get_person_probe();
        click(
            window,
            LogicalPosition::new(
                before.bounds.x + before.bounds.width / 2.,
                before.bounds.y + before.bounds.height / 2.,
            ),
        );
        check(
            gallery.get_person_probe().state == before.state,
            "PersonPicture remains informational under pointer input",
        )?;
    }
    gallery.invoke_focus_person_previous();
    key(window, Key::Tab);
    window.take_snapshot()?;
    check(
        gallery.get_person_following_focused(),
        "Tab skips the informational PersonPicture",
    )?;
    gallery.invoke_select_theme(FluentColorScheme::Light);
    Ok(())
}

fn verify_photo_crop(gallery: &GalleryWindow) -> SmokeResult {
    let p = gallery.get_person_probe();
    let b = p.bounds;
    let window = gallery.window();
    let outside = read_rgb(window, LogicalPosition::new(b.x - 2., b.y - 2.))?;
    check(
        read_rgb(window, LogicalPosition::new(b.x + 1., b.y + 1.))? == outside,
        "profile image clips its rectangular corners to a circle",
    )?;
    check(
        read_rgb(
            window,
            LogicalPosition::new(b.x + b.width * 0.25, b.y + b.height * 0.25),
        )? == [231, 111, 81]
            && read_rgb(
                window,
                LogicalPosition::new(b.x + b.width * 0.75, b.y + b.height * 0.75),
            )? == [233, 196, 106],
        "non-square profile image uses a centered cover crop",
    )
}

pub(super) fn verify_compact(gallery: &GalleryWindow, width: f32) -> SmokeResult {
    gallery.set_page(GalleryPage::NumberBox);
    gallery.set_number_placement(Placement::Inline);
    gallery.window().take_snapshot()?;
    let p = gallery.get_number_probe();
    check(
        p.input.x >= 0. && p.input.x + p.input.width <= width && p.input.height >= 44.,
        "compact NumberBox fits with touch-sized inline controls",
    )?;
    gallery.set_page(GalleryPage::PersonPicture);
    gallery.set_person_size(96.);
    gallery.window().take_snapshot()?;
    let p = gallery.get_person_probe();
    check(
        p.bounds.x >= 0. && p.bounds.x + p.bounds.width <= width,
        "compact PersonPicture wraps its adjacent actions without clipping",
    )?;
    Ok(())
}

pub(super) fn verify_batch(gallery: &GalleryWindow, done: impl FnOnce(BatchResult) + 'static) {
    gallery.global::<FluentTheme>().set_reduced_motion(true);
    gallery.set_shell_pane_visible(false);
    if let Err(error) = verify(gallery).and_then(|()| {
        gallery.global::<FluentTheme>().set_ui_scale(1.);
        verify(gallery)
    }) {
        done(Err(error));
        return;
    }
    let size = gallery.window().size();
    let wide = (size.width, size.height);
    gallery.set_touch_targets(true);
    gallery.window().set_size(LogicalSize::new(320., 640.));
    let weak = gallery.as_weak();
    slint::Timer::single_shot(Duration::from_millis(250), move || {
        let Some(gallery) = weak.upgrade() else {
            done(Err(
                "Gallery closed before compact number/person verification".into(),
            ));
            return;
        };
        if let Err(error) = verify_compact(&gallery, 320.) {
            done(Err(error));
            return;
        }
        let size = gallery.window().size();
        let compact = (size.width, size.height);
        verify_hold(&gallery, move |result| {
            done(result.map(|()| (wide, compact)))
        });
    });
}

pub(super) fn verify_hold(gallery: &GalleryWindow, done: impl FnOnce(SmokeResult) + 'static) {
    let result = (|| {
        gallery.set_page(GalleryPage::NumberBox);
        gallery.set_number_placement(Placement::Inline);
        gallery.set_number_enabled(true);
        gallery.set_number_minimum("0".into());
        gallery.set_number_maximum("5".into());
        gallery.set_number_value("1".into());
        gallery.set_number_small_change("1".into());
        gallery.window().take_snapshot()?;
        let up = gallery.get_number_probe().up;
        gallery
            .window()
            .dispatch_event(WindowEvent::PointerMoved { position: up });
        gallery
            .window()
            .dispatch_event(WindowEvent::PointerPressed {
                position: up,
                button: PointerEventButton::Left,
            });
        check(
            gallery.get_number_value() == "2",
            "NumberBox spin acts immediately on press",
        )?;
        Ok::<_, Box<dyn Error>>(up)
    })();
    let up = match result {
        Ok(up) => up,
        Err(error) => {
            done(Err(error));
            return;
        }
    };
    let weak = gallery.as_weak();
    slint::Timer::single_shot(Duration::from_millis(800), move || {
        let Some(gallery) = weak.upgrade() else {
            done(Err("Gallery closed during NumberBox hold".into()));
            return;
        };
        let result = (|| {
            gallery.window().take_snapshot()?;
            check(
                gallery.get_number_value() == "5"
                    && !gallery.get_number_probe().up_enabled
                    && !gallery.get_number_probe().repeating,
                "held spin stops itself at the numeric bound",
            )?;
            gallery
                .window()
                .dispatch_event(WindowEvent::PointerReleased {
                    position: up,
                    button: PointerEventButton::Left,
                });
            gallery.set_number_value("2".into());
            gallery.window().take_snapshot()?;
            gallery
                .window()
                .dispatch_event(WindowEvent::PointerPressed {
                    position: up,
                    button: PointerEventButton::Left,
                });
            gallery.set_page(GalleryPage::Button);
            gallery.window().take_snapshot()?;
            Ok::<_, Box<dyn Error>>(gallery.get_number_value())
        })();
        let value = match result {
            Ok(value) => value,
            Err(error) => {
                done(Err(error));
                return;
            }
        };
        let weak = gallery.as_weak();
        slint::Timer::single_shot(Duration::from_millis(600), move || {
            let result = (|| {
                let gallery = weak
                    .upgrade()
                    .ok_or("Gallery closed before NumberBox cancellation check")?;
                gallery
                    .window()
                    .dispatch_event(WindowEvent::PointerReleased {
                        position: up,
                        button: PointerEventButton::Left,
                    });
                check(
                    gallery.get_number_value() == value && !gallery.get_number_probe().repeating,
                    "leaving NumberBox cancels a captured spin hold",
                )
            })();
            done(result);
        });
    });
}
