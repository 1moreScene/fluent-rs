use std::{error::Error, time::Duration};

use slint::{
    ComponentHandle, LogicalPosition, SharedString, Timer,
    platform::{Key, PointerEventButton, WindowEvent},
};

use super::{check, click, read_rgb};
use crate::{
    ButtonState, FluentColorScheme, FluentTheme, GalleryPage, GalleryWindow, ToggleProbe,
    ToggleValue,
};

struct Example {
    probe: fn(&GalleryWindow) -> ToggleProbe,
    value: fn(&GalleryWindow) -> ToggleValue,
    set_value: fn(&GalleryWindow, ToggleValue),
    activations: fn(&GalleryWindow) -> i32,
}

const TOGGLE: Example = Example {
    probe: GalleryWindow::get_toggle_button_probe,
    value: GalleryWindow::get_toggle_button_value,
    set_value: GalleryWindow::set_toggle_button_value,
    activations: GalleryWindow::get_toggle_button_activations,
};
const CHECK: Example = Example {
    probe: GalleryWindow::get_check_box_probe,
    value: GalleryWindow::get_check_box_value,
    set_value: GalleryWindow::set_check_box_value,
    activations: GalleryWindow::get_check_box_activations,
};

pub(super) fn verify(gallery: &GalleryWindow) -> Result<(), Box<dyn Error>> {
    gallery.set_page(GalleryPage::Toggles);
    gallery.invoke_select_theme(FluentColorScheme::Light);
    verify_input(gallery, TOGGLE)?;
    verify_input(gallery, CHECK)?;
    verify_marks(gallery)
}

fn verify_input(gallery: &GalleryWindow, example: Example) -> Result<(), Box<dyn Error>> {
    let window = gallery.window();
    let probe = || (example.probe)(gallery);
    let value = || (example.value)(gallery);
    let count = || (example.activations)(gallery);
    let center = probe().center;
    let position = LogicalPosition::new(center.x, center.y);
    let fill_sample = LogicalPosition::new(center.x - probe().width / 2. + 4., center.y - 5.);
    let ring_sample = LogicalPosition::new(center.x, center.y - probe().height / 2. - 2.);
    let outside = LogicalPosition::new(1., 1.);
    let button = PointerEventButton::Left;
    let press = |text: SharedString| window.dispatch_event(WindowEvent::KeyPressed { text });
    let release = |text: SharedString| window.dispatch_event(WindowEvent::KeyReleased { text });

    gallery.set_toggles_three_state(true);
    (example.set_value)(gallery, ToggleValue::Unchecked);
    let initially = count();
    window.dispatch_event(WindowEvent::PointerMoved { position: outside });
    let normal_fill = read_rgb(window, fill_sample)?;
    let unfocused_ring = read_rgb(window, ring_sample)?;
    window.dispatch_event(WindowEvent::PointerMoved { position });
    check(probe().state == ButtonState::PointerOver, "toggle hover")?;
    let hover_fill = read_rgb(window, fill_sample)?;
    check(normal_fill != hover_fill, "toggle paints hover fill")?;
    window.dispatch_event(WindowEvent::PointerPressed { position, button });
    check(
        probe().state == ButtonState::Pressed && value() == ToggleValue::Unchecked,
        "toggle press waits for release",
    )?;
    check(
        read_rgb(window, fill_sample)? != hover_fill,
        "toggle paints press fill",
    )?;
    window.dispatch_event(WindowEvent::PointerMoved { position: outside });
    window.dispatch_event(WindowEvent::PointerReleased {
        position: outside,
        button,
    });
    check(
        count() == initially && value() == ToggleValue::Unchecked,
        "drag out cancels toggle",
    )?;
    for (step, expected) in [
        ToggleValue::Checked,
        ToggleValue::Indeterminate,
        ToggleValue::Unchecked,
    ]
    .into_iter()
    .enumerate()
    {
        click(window, position);
        check(
            value() == expected && count() == initially + step as i32 + 1,
            "toggle three-state cycle",
        )?;
        check(
            !probe().focus_visible,
            "pointer activation hides the keyboard ring",
        )?;
    }

    for _ in 0..20 {
        press(Key::Tab.into());
        release(Key::Tab.into());
        if probe().focus_visible {
            break;
        }
    }
    check(probe().focus_visible, "tab reaches the toggle")?;
    check(
        read_rgb(window, ring_sample)? != unfocused_ring,
        "toggle paints its keyboard focus ring",
    )?;
    gallery.set_toggles_three_state(false);
    let before_keys = count();
    for (step, (key, expected)) in [
        (SharedString::from(" "), ToggleValue::Checked),
        (Key::Return.into(), ToggleValue::Unchecked),
    ]
    .into_iter()
    .enumerate()
    {
        press(key.clone());
        window.dispatch_event(WindowEvent::KeyPressRepeated { text: key.clone() });
        check(
            probe().state == ButtonState::Pressed && count() == before_keys + step as i32,
            "held toggle key waits for release",
        )?;
        release(key);
        check(
            value() == expected && count() == before_keys + step as i32 + 1,
            "toggle two-state keyboard cycle",
        )?;
    }

    press(" ".into());
    gallery.set_toggles_enabled(false);
    release(" ".into());
    click(window, position);
    check(
        probe().state == ButtonState::Disabled
            && !probe().focus_visible
            && value() == ToggleValue::Unchecked
            && count() == before_keys + 2,
        "disable cancels the pending activation and blocks clicks",
    )?;
    check(
        read_rgb(window, ring_sample)? == unfocused_ring
            && read_rgb(window, fill_sample)? != normal_fill,
        "disabled toggle clears its ring and paints disabled fill",
    )?;
    (example.set_value)(gallery, ToggleValue::Indeterminate);
    check(
        value() == ToggleValue::Indeterminate && count() == before_keys + 2,
        "bound value writes without user event",
    )?;
    gallery.set_toggles_enabled(true);
    click(window, position);
    check(
        value() == ToggleValue::Unchecked && count() == before_keys + 3,
        "mixed value returns to unchecked in two-state mode",
    )
}

fn verify_marks(gallery: &GalleryWindow) -> Result<(), Box<dyn Error>> {
    let window = gallery.window();
    window.dispatch_event(WindowEvent::PointerMoved {
        position: LogicalPosition::new(1., 1.),
    });
    let check_box = gallery.get_check_box_probe();
    let box_left = check_box.center.x - check_box.width / 2.;
    let box_fill = LogicalPosition::new(box_left + 4., check_box.center.y - 5.);
    let mark = LogicalPosition::new(box_left + 13., check_box.center.y + 1.);
    let toggle = gallery.get_toggle_button_probe();
    let toggle_fill =
        LogicalPosition::new(toggle.center.x, toggle.center.y - toggle.height / 2. + 3.);

    for scheme in [
        FluentColorScheme::Light,
        FluentColorScheme::Dark,
        FluentColorScheme::HighContrast,
    ] {
        gallery.invoke_select_theme(scheme);
        gallery.set_toggle_button_value(ToggleValue::Unchecked);
        gallery.set_check_box_value(ToggleValue::Unchecked);
        let unchecked = read_rgb(window, box_fill)?;
        let toggle_unchecked = read_rgb(window, toggle_fill)?;
        gallery.set_toggle_button_value(ToggleValue::Checked);
        gallery.set_check_box_value(ToggleValue::Checked);
        let checked = read_rgb(window, box_fill)?;
        let check_mark = read_rgb(window, mark)?;
        check(
            checked != unchecked && read_rgb(window, toggle_fill)? != toggle_unchecked,
            "checked controls paint accent fills",
        )?;
        gallery.set_toggle_button_value(ToggleValue::Indeterminate);
        gallery.set_check_box_value(ToggleValue::Indeterminate);
        check(
            read_rgb(window, box_fill)? == checked && read_rgb(window, mark)? != check_mark,
            "indeterminate CheckBox uses the checked fill and a distinct mark",
        )?;
        if scheme != FluentColorScheme::HighContrast {
            check(
                read_rgb(window, toggle_fill)? == toggle_unchecked,
                "mixed ToggleButton uses standard fill",
            )?;
        }
    }
    Ok(())
}

pub(super) fn verify_compact(
    gallery: &GalleryWindow,
    window_width: f32,
) -> Result<(), Box<dyn Error>> {
    gallery.set_page(GalleryPage::Toggles);
    for probe in [
        gallery.get_toggle_button_probe(),
        gallery.get_check_box_probe(),
    ] {
        check(
            probe.height >= 44. && probe.width >= 44.,
            "compact toggle hit target",
        )?;
        check(
            probe.center.x - probe.width / 2. >= 0.
                && probe.center.x + probe.width / 2. <= window_width,
            "compact toggle fits horizontally",
        )?;
    }
    let base_height = gallery.get_check_box_probe().height;
    gallery.set_check_box_label(
        "A longer CheckBox label that wraps onto several lines inside a compact window. \
         The label must be long enough for the text to grow beyond the minimum touch target."
            .into(),
    );
    let wrapped = gallery.get_check_box_probe();
    check(
        wrapped.height > base_height,
        "long CheckBox text wraps in a compact window",
    )?;
    check(
        wrapped.center.x + wrapped.width / 2. <= window_width,
        "wrapped CheckBox fits horizontally",
    )?;
    gallery.set_check_box_label("CheckBox".into());
    verify_marks(gallery)
}

pub(super) fn verify_mark_motion(
    gallery: &GalleryWindow,
    complete: impl FnOnce(Result<(), Box<dyn Error>>) + 'static,
) {
    gallery.set_page(GalleryPage::Toggles);
    gallery.invoke_select_theme(FluentColorScheme::Light);
    let theme = gallery.global::<FluentTheme>();
    theme.set_reduced_motion(true);
    gallery.set_check_box_value(ToggleValue::Unchecked);
    let probe = gallery.get_check_box_probe();
    let left = probe.center.x - probe.width / 2.;
    let fill = LogicalPosition::new(left + 4., probe.center.y - 5.);
    let mark = LogicalPosition::new(left + 12., probe.center.y + 0.5);
    let sample = move |gallery: &GalleryWindow| -> Result<bool, Box<dyn Error>> {
        let window = gallery.window();
        Ok(read_rgb(window, fill)? != read_rgb(window, mark)?)
    };

    let initial =
        sample(gallery).and_then(|present| check(!present, "unchecked mark starts clear"));
    if initial.is_err() {
        complete(initial);
        return;
    }
    theme.set_reduced_motion(false);
    gallery.set_check_box_value(ToggleValue::Checked);
    // Resolve the animated binding on this event-loop tick.
    if let Err(error) = sample(gallery) {
        complete(Err(error));
        return;
    }
    let weak = gallery.as_weak();
    Timer::single_shot(Duration::from_millis(170), move || {
        let Some(gallery) = weak.upgrade() else {
            complete(Err("Gallery closed during the checkbox animation".into()));
            return;
        };
        let progress = sample(&gallery).and_then(|present| {
            check(
                present,
                "checkbox easing reaches the final stroke segment by mid-animation",
            )
        });
        if progress.is_err() {
            complete(progress);
            return;
        }
        Timer::single_shot(Duration::from_millis(175), move || {
            gallery.set_check_box_value(ToggleValue::Unchecked);
            if let Err(error) = sample(&gallery) {
                complete(Err(error));
                return;
            }
            Timer::single_shot(Duration::from_millis(120), move || {
                complete(sample(&gallery).and_then(|present| {
                    check(
                        !present,
                        "checkbox erasure completes after the short exit segment",
                    )
                }));
            });
        });
    });
}
