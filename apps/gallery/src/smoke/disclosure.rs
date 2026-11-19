use std::{error::Error, time::Duration};

use slint::{
    ComponentHandle, LogicalPosition, LogicalSize, Window,
    platform::{Key, PointerEventButton, WindowEvent},
};

use super::{check, click, overlays::key, read_rgb};
use crate::{ButtonState, FluentColorScheme, FluentTheme, GalleryPage, GalleryWindow};

type SmokeResult = Result<(), Box<dyn Error>>;

pub(super) fn verify(gallery: &GalleryWindow) -> SmokeResult {
    verify_expander(gallery)?;
    verify_split_button(gallery)?;
    let window = gallery.window();
    let size = window.size().to_logical(window.scale_factor());
    window.dispatch_event(WindowEvent::Resized {
        size: LogicalSize::new(360., size.height),
    });
    gallery.set_page(GalleryPage::Expander);
    window.take_snapshot()?;
    let expander = gallery.get_expander_probe();
    check(
        expander.direction_button.y > expander.header.y
            && expander.custom_bounds.x + expander.custom_bounds.width <= 360.,
        "Expander stacks its direction options and bounds the fixed-width sample on a narrow window",
    )?;
    gallery.set_page(GalleryPage::SplitButton);
    window.take_snapshot()?;
    let split = gallery.get_split_button_probe();
    click(window, split.secondary);
    slint::platform::update_timers_and_animations();
    let narrow = gallery.get_split_button_probe();
    check(
        narrow.open && narrow.panel.x >= 0. && narrow.panel.x + narrow.panel.width <= 360.,
        "SplitButton keeps the flyout in a narrow viewport",
    )?;
    key(window, Key::Escape);
    window.dispatch_event(WindowEvent::Resized { size });
    window.take_snapshot()?;
    Ok(())
}

fn verify_expander(gallery: &GalleryWindow) -> SmokeResult {
    let window = gallery.window();
    gallery.set_page(GalleryPage::Expander);
    gallery.invoke_select_theme(FluentColorScheme::Light);
    window.take_snapshot()?;
    let initial = gallery.get_expander_probe();
    let outside = LogicalPosition::new(1., 1.);
    let ui = gallery.global::<FluentTheme>().get_ui_scale();
    check(
        !initial.expanded && initial.revealed == 0.,
        "Expander starts with its content collapsed",
    )?;
    check(
        (initial.bounds.height - 48. * ui).abs() < 0.75
            && (initial.custom_bounds.width - 500. * ui).abs() < 0.75
            && initial.bounds.x + initial.bounds.width < initial.direction_button.x
            && initial.custom_label.width > 300. * ui,
        "Expander uses a 48-unit header, separate direction options, and a 500-unit centered-title example",
    )?;
    verify_expander_chevron_alignment(window, &initial, ui)?;
    window.dispatch_event(WindowEvent::PointerMoved {
        position: initial.header,
    });
    check(
        gallery.get_expander_probe().state == ButtonState::PointerOver,
        "Expander reports hover on the whole header",
    )?;
    window.dispatch_event(WindowEvent::PointerPressed {
        position: initial.header,
        button: PointerEventButton::Left,
    });
    check(
        gallery.get_expander_probe().state == ButtonState::Pressed,
        "Expander enters Pressed before changing expansion state",
    )?;
    window.dispatch_event(WindowEvent::PointerReleased {
        position: initial.header,
        button: PointerEventButton::Left,
    });
    let open = gallery.get_expander_probe();
    check(
        open.expanded && open.revealed >= 48. && open.body.y > open.header.y && open.focused,
        "Expander reveals its body below the header and focuses the header",
    )?;
    click(window, open.body);
    check(
        gallery.get_expander_probe().expanded,
        "clicking the content leaves the header expanded",
    )?;
    click(window, open.header);
    key(window, Key::Space);
    check(
        gallery.get_expander_probe().expanded,
        "Space expands the focused Expander",
    )?;
    key(window, Key::Return);
    check(
        gallery.get_expander_probe().revealed == 0.,
        "Enter collapses the Expander immediately with reduced motion",
    )?;
    click(window, initial.direction_button);
    check(
        gallery.get_expander_probe().direction_open,
        "ExpandDirection opens its ComboBox",
    )?;
    key(window, Key::End);
    key(window, Key::Return);
    let up = gallery.get_expander_probe();
    click(window, up.header);
    let open_up = gallery.get_expander_probe();
    check(
        open_up.upward && open_up.expanded && open_up.body.y < open_up.header.y,
        "upward expansion reveals content on the opposite side of the header",
    )?;
    for theme in [FluentColorScheme::Dark, FluentColorScheme::HighContrast] {
        gallery.invoke_select_theme(theme);
        window.dispatch_event(WindowEvent::PointerMoved { position: outside });
        let header = gallery.get_expander_probe().header;
        let pos = LogicalPosition::new(header.x, header.y + 16. * ui);
        let expected = gallery.global::<FluentTheme>().get_card_fill();
        let page = gallery.global::<FluentTheme>().get_page();
        let canvas = [page.red(), page.green(), page.blue()];
        let rendered = read_rgb(window, pos)?;
        check(
            rendered
                .iter()
                .zip(blend_over(
                    [expected.red(), expected.green(), expected.blue()],
                    expected.alpha(),
                    canvas,
                ))
                .all(|(&actual, expected)| actual.abs_diff(expected) <= 2),
            "Expander paints WinUI header fill in dark and high contrast",
        )?;
    }
    click(window, open_up.header);
    click(window, gallery.get_expander_probe().disabled_header);
    check(
        !gallery.get_expander_probe().disabled_expanded
            && gallery.get_expander_probe().revealed == 0.,
        "the Expander remains collapsed when a disabled header is clicked",
    )?;
    gallery.invoke_select_theme(FluentColorScheme::Light);
    Ok(())
}

fn verify_split_button(gallery: &GalleryWindow) -> SmokeResult {
    let window = gallery.window();
    gallery.set_page(GalleryPage::SplitButton);
    gallery.invoke_select_theme(FluentColorScheme::Light);
    window.take_snapshot()?;
    let initial = gallery.get_split_button_probe();
    let ui = gallery.global::<FluentTheme>().get_ui_scale();
    let green = read_rgb(window, initial.primary)?;
    if (initial.anchor.width - 71. * ui).abs() >= 0.75
        || (initial.anchor.height - 32. * ui).abs() >= 0.75
        || (initial.primary.x - initial.anchor.x - 18. * ui).abs() >= 0.75
        || (initial.anchor.x - initial.text_anchor.x).abs() >= 0.75
        || initial.text_anchor.width <= initial.anchor.width
        || initial.text_anchor.width >= 140. * ui
        || !swatch_matches(green, [0, 128, 0])
    {
        return Err(format!(
            "unexpected SplitButton sizing or swatch at {ui} UI scale: swatch {:?}, text {:?}, green pixel {green:?}",
            initial.anchor, initial.text_anchor,
        ).into());
    }
    window.dispatch_event(WindowEvent::PointerMoved {
        position: initial.primary,
    });
    let primary_hover = gallery.get_split_button_probe();
    check(
        primary_hover.primary_state == ButtonState::PointerOver
            && primary_hover.secondary_state == ButtonState::Normal,
        "SplitButton only highlights the hovered mouse segment",
    )?;
    click(window, initial.primary);
    let applied = gallery.get_split_button_probe();
    check(
        !applied.open && applied.invocations == 1 && applied.applied == "Green",
        "the SplitButton main segment invokes without opening the flyout",
    )?;
    key(window, Key::Return);
    click(window, initial.secondary);
    window.take_snapshot()?;
    slint::platform::update_timers_and_animations();
    let open = gallery.get_split_button_probe();
    let expected_gap = 4. * gallery.global::<FluentTheme>().get_ui_scale();
    let expected_left =
        (open.anchor.x + (open.anchor.width - open.panel.width) / 2.).max(expected_gap);
    check(
        open.open
            && open.invocations == 2
            && (open.panel.x - expected_left).abs() < 0.75
            && (open.panel.y - open.anchor.y - open.anchor.height - expected_gap).abs() < 0.75
            && open.primary_state == ButtonState::Pressed
            && open.secondary_state == ButtonState::Pressed,
        "the SplitButton secondary segment opens a centered Bottom flyout and presses both segments",
    )?;
    let grid = open.font_grid;
    if grid.columns != 3 || grid.realized != 8 || grid.focused != 0 || grid.selected != -1 {
        return Err(
            format!("unexpected first SplitButton GridView after opening: {grid:?}").into(),
        );
    }
    let red = read_rgb(window, grid.red)?;
    let orange = read_rgb(window, grid.orange)?;
    let yellow = read_rgb(window, grid.yellow)?;
    let green = read_rgb(window, grid.green)?;
    if (open.panel.width - 178. * ui).abs() >= 0.75
        || (open.panel.height - 188. * ui).abs() >= 0.75
        || (grid.orange.x - grid.red.x - 48. * ui).abs() >= 0.75
        || (grid.yellow.x - grid.orange.x - 48. * ui).abs() >= 0.75
        || (grid.green.y - grid.red.y - 48. * ui).abs() >= 0.75
        || grid.green.x != grid.red.x
        || !swatch_matches(red, [255, 0, 0])
        || !swatch_matches(orange, [255, 165, 0])
        || !swatch_matches(yellow, [255, 255, 0])
        || !swatch_matches(green, [0, 128, 0])
    {
        return Err(format!(
            "unexpected GridView geometry for {grid:?} in {:?}, sample pixels {red:?}, {orange:?}, {yellow:?}, {green:?}",
            open.panel,
        ).into());
    }
    let blue_pixel = read_rgb(window, grid.blue)?;
    click(window, grid.blue);
    window.take_snapshot()?;
    let selected = gallery.get_split_button_probe();
    if selected.open
        || selected.selection != "Blue"
        || selected.applied != "Blue"
        || selected.invocations != 2
        || !selected.focused
        || selected.font_grid.realized != 0
    {
        return Err(format!(
            "SplitButton did not commit the blue swatch and restore focus: pixel {blue_pixel:?}, {selected:?}",
        ).into());
    }
    key(window, Key::Space);
    check(
        gallery.get_split_button_probe().applied == "Blue",
        "Space invokes the new primary action",
    )?;
    key(window, Key::F4);
    slint::platform::update_timers_and_animations();
    check(
        gallery.get_split_button_probe().open,
        "F4 opens SplitButton",
    )?;
    key(window, Key::DownArrow);
    key(window, Key::RightArrow);
    key(window, Key::RightArrow);
    window.take_snapshot()?;
    let browsing = gallery.get_split_button_probe();
    check(
        browsing.open
            && browsing.font_grid.focused == 5
            && browsing.font_grid.selected == 5
            && browsing.applied == "Blue",
        "GridView keyboard navigation updates focus and selection before the color is committed",
    )?;
    window.dispatch_event(WindowEvent::KeyPressed {
        text: Key::Control.into(),
    });
    key(window, Key::LeftArrow);
    window.dispatch_event(WindowEvent::KeyReleased {
        text: Key::Control.into(),
    });
    window.take_snapshot()?;
    let browse_with_control = gallery.get_split_button_probe().font_grid;
    check(
        browse_with_control.focused == 4 && browse_with_control.selected == 5,
        "Control+Left moves GridView focus off the selected swatch",
    )?;
    let accent = gallery.global::<FluentTheme>().get_accent_fill();
    let outline = read_rgb(window, browse_with_control.selection_outline)?;
    let expected = [accent.red(), accent.green(), accent.blue()];
    if outline != expected {
        return Err(format!("GridView selection outline in the SplitButton flyout was {outline:?}, expected {expected:?}").into());
    }
    key(window, Key::RightArrow);
    key(window, Key::Return);
    let keyboard_choice = gallery.get_split_button_probe();
    check(
        keyboard_choice.selection == "Indigo"
            && keyboard_choice.applied == "Indigo"
            && !keyboard_choice.open
            && keyboard_choice.focused,
        "arrow navigation moves across rows and columns, and Enter commits with restored focus",
    )?;
    key(window, Key::F4);
    slint::platform::update_timers_and_animations();
    key(window, Key::Escape);
    check(
        !gallery.get_split_button_probe().open && gallery.get_split_button_probe().focused,
        "Escape dismisses the SplitButton flyout and restores focus",
    )?;
    window.dispatch_event(WindowEvent::KeyPressed {
        text: Key::Alt.into(),
    });
    key(window, Key::DownArrow);
    window.dispatch_event(WindowEvent::KeyReleased {
        text: Key::Alt.into(),
    });
    check(
        gallery.get_split_button_probe().open,
        "Alt+Down opens SplitButton",
    )?;
    click(window, LogicalPosition::new(1., 1.));
    check(
        !gallery.get_split_button_probe().open,
        "outside click dismisses SplitButton",
    )?;
    click(window, initial.text_secondary);
    window.take_snapshot()?;
    slint::platform::update_timers_and_animations();
    let text_palette = gallery.get_split_button_probe();
    let text_panel = text_palette.text_panel;
    let expected_left = (text_palette.text_anchor.x
        + (text_palette.text_anchor.width - text_panel.width) / 2.)
        .max(expected_gap);
    check(
        text_palette.text_open
            && !text_palette.open
            && (text_panel.x - expected_left).abs() < 0.75
            && (text_panel.width - 172. * ui).abs() < 0.75
            && (text_panel.height - 172. * ui).abs() < 0.75,
        "the text SplitButton centers its nine-button palette with the default flyout insets",
    )?;
    for (column, row, color) in [
        (0., 0., [255, 0, 0]),
        (1., 0., [255, 165, 0]),
        (2., 0., [255, 255, 0]),
        (0., 1., [0, 128, 0]),
        (2., 2., [0, 0, 0]),
    ] {
        let point = LogicalPosition::new(
            text_panel.x + (40. + 46. * column) * ui,
            text_panel.y + (39. + 46. * row) * ui,
        );
        let actual = read_rgb(window, point)?;
        if !swatch_matches(actual, color) {
            return Err(format!(
                "Choose color swatch at {point:?} was {actual:?}, expected {color:?}"
            )
            .into());
        }
    }
    key(window, Key::Tab);
    key(window, Key::Return);
    let text_choice = gallery.get_split_button_probe();
    if text_choice.text_choice != "Orange"
        || text_choice.text_open
        || !text_choice.text_focused
        || text_choice.applied != "Indigo"
    {
        return Err(format!(
            "Choose color after Tab and Enter: choice {:?}, open {}, focus {}, applied {:?}",
            text_choice.text_choice,
            text_choice.text_open,
            text_choice.text_focused,
            text_choice.applied,
        )
        .into());
    }
    key(window, Key::F4);
    slint::platform::update_timers_and_animations();
    check(
        gallery.get_split_button_probe().text_open,
        "F4 reopens the text color palette",
    )?;
    key(window, Key::Escape);
    let disabled = gallery.get_split_button_probe().disabled;
    click(window, disabled);
    check(
        !gallery.get_split_button_probe().disabled_open,
        "disabled secondary action does not open a flyout",
    )?;

    for theme in [FluentColorScheme::Dark, FluentColorScheme::HighContrast] {
        gallery.invoke_select_theme(theme);
        let probe = gallery.get_split_button_probe();
        let position = LogicalPosition::new(probe.secondary.x + 12., probe.secondary.y);
        let outside = LogicalPosition::new(1., 1.);
        window.dispatch_event(WindowEvent::PointerMoved { position: outside });
        let idle = read_rgb(window, position)?;
        window.dispatch_event(WindowEvent::PointerMoved {
            position: probe.secondary,
        });
        check(
            gallery.get_split_button_probe().secondary_state == ButtonState::PointerOver
                && (read_rgb(window, position)? != idle
                    || theme == FluentColorScheme::HighContrast),
            "SplitButton renders an independent secondary hover in dark and high contrast",
        )?;
    }
    gallery.invoke_select_theme(FluentColorScheme::Light);
    Ok(())
}

fn verify_expander_chevron_alignment(
    window: &Window,
    probe: &crate::ExpanderProbe,
    ui: f32,
) -> SmokeResult {
    let frame = window.take_snapshot()?;
    let scale = window.scale_factor();
    let first_x = ((probe.chevron.x - 6. * ui) * scale).floor() as usize;
    let last_x = ((probe.chevron.x + 6. * ui) * scale).ceil() as usize;
    let first_y = ((probe.bounds.y + 5. * ui) * scale).ceil() as usize;
    let last_y = ((probe.bounds.y + probe.bounds.height - 5. * ui) * scale).floor() as usize;
    let mut rows = (first_y..last_y).filter(|&y| {
        (first_x..last_x).any(|x| {
            let pixel = frame.as_slice()[y * frame.width() as usize + x];
            pixel.r.max(pixel.g).max(pixel.b) < 160
        })
    });
    let top = rows.next().ok_or("Expander chevron pixels are missing")?;
    let bottom = rows.next_back().unwrap_or(top);
    let center = (top + bottom + 1) as f32 / (2. * scale);
    check(
        (probe.chevron.y - probe.header.y).abs() < 0.25
            && (center - probe.header.y).abs() < 2. * ui,
        "the rendered Expander chevron is vertically centered in the header",
    )?;

    let label = &probe.custom_label;
    let first_x = (label.x * scale).ceil() as usize;
    let last_x = ((label.x + label.width) * scale).floor() as usize;
    let first_y = ((label.y + ui) * scale).ceil() as usize;
    let last_y = ((label.y + label.height - ui) * scale).floor() as usize;
    let mut columns = (first_x..last_x).filter(|&x| {
        (first_y..last_y).any(|y| {
            let pixel = frame.as_slice()[y * frame.width() as usize + x];
            pixel.r.max(pixel.g).max(pixel.b) < 160
        })
    });
    let left = columns
        .next()
        .ok_or("Expander custom title pixels are missing")?;
    let right = columns.next_back().unwrap_or(left);
    let center = (left + right + 1) as f32 / (2. * scale);
    let expected = label.x + label.width / 2.;
    if (center - expected).abs() >= 2. * ui {
        return Err(format!(
            "Expander custom title ink center {center}, expected {expected} in {label:?}"
        )
        .into());
    }
    Ok(())
}

fn blend_over(fill: [u8; 3], alpha: u8, background: [u8; 3]) -> [u8; 3] {
    std::array::from_fn(|i| {
        let alpha = u16::from(alpha);
        ((u16::from(fill[i]) * alpha + u16::from(background[i]) * (255 - alpha) + 127) / 255) as u8
    })
}

fn swatch_matches(actual: [u8; 3], expected: [u8; 3]) -> bool {
    actual
        .into_iter()
        .zip(expected)
        .all(|(actual, expected)| actual.abs_diff(expected) <= 8)
}

pub(super) fn verify_motion(gallery: &GalleryWindow, finish: impl FnOnce(SmokeResult) + 'static) {
    gallery.set_page(GalleryPage::Expander);
    let weak = gallery.as_weak();
    slint::Timer::single_shot(Duration::from_millis(200), move || {
        let Some(gallery) = weak.upgrade() else {
            return finish(Err("Gallery closed before Expander motion".into()));
        };
        click(gallery.window(), gallery.get_expander_probe().header);
        slint::Timer::single_shot(Duration::from_millis(100), move || {
            let Some(gallery) = weak.upgrade() else {
                return finish(Err("Gallery closed during Expander motion".into()));
            };
            let middle = gallery.get_expander_probe().revealed;
            slint::Timer::single_shot(Duration::from_millis(320), move || {
                let Some(gallery) = weak.upgrade() else {
                    return finish(Err("Gallery closed before Expander settled".into()));
                };
                let expanded = gallery.get_expander_probe();
                if let Err(error) = check(
                    expanded.expanded && middle > 0. && middle < expanded.revealed,
                    "Expander reveals an intermediate height before settling",
                ) {
                    return finish(Err(error));
                }
                click(gallery.window(), expanded.header);
                slint::Timer::single_shot(Duration::from_millis(240), move || {
                    let Some(gallery) = weak.upgrade() else {
                        return finish(Err("Gallery closed during Expander collapse".into()));
                    };
                    if let Err(error) = check(
                        gallery.get_expander_probe().revealed == 0.,
                        "Expander animates back to its closed height",
                    ) {
                        return finish(Err(error));
                    }
                    verify_chevron_motion(&gallery, finish);
                });
            });
        });
    });
}

fn verify_chevron_motion(gallery: &GalleryWindow, finish: impl FnOnce(SmokeResult) + 'static) {
    gallery.set_page(GalleryPage::SplitButton);
    let weak = gallery.as_weak();
    slint::Timer::single_shot(Duration::from_millis(160), move || {
        let Some(gallery) = weak.upgrade() else {
            return finish(Err("Gallery closed before SplitButton motion".into()));
        };
        let position = gallery.get_split_button_probe().secondary;
        pointer(gallery.window(), position, true);
        slint::Timer::single_shot(Duration::from_millis(70), move || {
            let Some(gallery) = weak.upgrade() else {
                return finish(Err("Gallery closed during SplitButton motion".into()));
            };
            let pressed = gallery.get_split_button_probe();
            if let Err(error) = check(
                pressed.chevron > 0. && pressed.chevron < 0.312,
                "SplitButton chevron displays the official press track's intermediate frame",
            ) {
                return finish(Err(error));
            }
            pointer(gallery.window(), position, false);
            slint::Timer::single_shot(Duration::from_millis(200), move || {
                let Some(gallery) = weak.upgrade() else {
                    return finish(Err("Gallery closed during SplitButton release".into()));
                };
                let released = gallery.get_split_button_probe();
                if !released.open || !(0.346..0.968).contains(&released.chevron) {
                    return finish(Err(format!("SplitButton release track did not advance with the flyout open: {released:?}").into()));
                }
                slint::Timer::single_shot(Duration::from_millis(300), move || {
                    let Some(gallery) = weak.upgrade() else {
                        return finish(Err("Gallery closed before SplitButton settled".into()));
                    };
                    let settled = gallery.get_split_button_probe();
                    let result = check(
                        settled.open
                            && crate::animated_icon::split_chevron_offset(settled.chevron).abs()
                                < 0.01,
                        "SplitButton settles at the resting chevron position while the flyout stays open",
                    );
                    key(gallery.window(), Key::Escape);
                    finish(result);
                });
            });
        });
    });
}

fn pointer(window: &Window, position: LogicalPosition, down: bool) {
    window.dispatch_event(if down {
        WindowEvent::PointerPressed {
            position,
            button: PointerEventButton::Left,
        }
    } else {
        WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        }
    });
}
