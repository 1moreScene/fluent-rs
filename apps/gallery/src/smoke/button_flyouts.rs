use std::{cell::Cell, error::Error, rc::Rc, time::Duration};

use slint::{
    ComponentHandle, LogicalPosition, LogicalSize, Window,
    platform::{Key, PointerEventButton, WindowEvent},
};

use super::{check, click, overlays::key, read_rgb};
use crate::{ButtonState, FluentColorScheme, FluentTheme, GalleryPage, GalleryWindow, OverlayArea};

type SmokeResult = Result<(), Box<dyn Error>>;
type BatchResult = Result<((u32, u32), (u32, u32)), Box<dyn Error>>;

fn center(bounds: &OverlayArea) -> LogicalPosition {
    LogicalPosition::new(bounds.x + bounds.width / 2., bounds.y + bounds.height / 2.)
}

fn hover(window: &Window, position: LogicalPosition) {
    window.dispatch_event(WindowEvent::PointerMoved { position });
}

fn pointer(window: &Window, position: LogicalPosition, down: bool) {
    let button = PointerEventButton::Left;
    window.dispatch_event(if down {
        WindowEvent::PointerPressed { position, button }
    } else {
        WindowEvent::PointerReleased { position, button }
    });
}

pub(super) fn verify(gallery: &GalleryWindow) -> SmokeResult {
    verify_drop_down(gallery)?;
    verify_toggle_split(gallery)?;
    gallery.invoke_select_theme(FluentColorScheme::Light);
    Ok(())
}

fn verify_drop_down(gallery: &GalleryWindow) -> SmokeResult {
    gallery.set_page(GalleryPage::DropDownButton);
    let window = gallery.window();
    let away = LogicalPosition::new(1., 1.);
    for theme in [
        FluentColorScheme::Light,
        FluentColorScheme::Dark,
        FluentColorScheme::HighContrast,
    ] {
        gallery.invoke_select_theme(theme);
        hover(window, away);
        let sample = gallery.get_drop_down_probe();
        let face = LogicalPosition::new(sample.text_button.x + 5., center(&sample.text_button).y);
        let idle = read_rgb(window, face)?;
        let trigger = center(&sample.text_button);
        hover(window, trigger);
        let hover_pixel = read_rgb(window, face)?;
        check(
            gallery.get_drop_down_probe().state == ButtonState::PointerOver
                && (theme == FluentColorScheme::HighContrast || hover_pixel != idle),
            "DropDownButton uses the themed hover face across its label and chevron",
        )?;
        pointer(window, trigger, true);
        let pressed = gallery.get_drop_down_probe();
        check(
            !pressed.open && pressed.state == ButtonState::Pressed,
            "DropDownButton presses without opening until release",
        )?;
        pointer(window, trigger, false);
        let open = gallery.get_drop_down_probe();
        let scale = gallery.global::<FluentTheme>().get_ui_scale();
        check(
            open.open
                && (open.panel.x - open.text_button.x).abs() < 1.
                && (open.panel.y - open.text_button.y - open.text_button.height - 4. * scale).abs()
                    < 1.,
            "DropDownButton opens on release with the panel aligned under the left edge",
        )?;
        let borders = if theme == FluentColorScheme::HighContrast {
            4.
        } else {
            2.
        };
        check(
            (open.panel.width - (96. + borders) * scale).abs() < 0.75
                && (open.panel.height - (100. + borders) * scale).abs() < 0.75,
            "MenuFlyout sizes the plain email sample to the desktop content minimum and compact rows",
        )?;
        key(window, Key::Escape);
        check(
            !gallery.get_drop_down_probe().open && gallery.get_drop_down_probe().focused,
            "Escape restores DropDownButton focus",
        )?;
    }

    key(window, Key::Space);
    window.take_snapshot()?;
    key(window, Key::End);
    check(
        gallery.get_drop_down_probe().command_index == 2,
        "Space opens DropDownButton and End moves to the last menu command",
    )?;
    key(window, Key::Return);
    let committed = gallery.get_drop_down_probe();
    check(
        !committed.open
            && committed.focused
            && committed.focus_visible
            && committed.command == "Reply All",
        "Enter invokes the focused command and returns keyboard focus to the trigger",
    )?;

    key(window, Key::Return);
    check(
        gallery.get_drop_down_probe().open,
        "Enter opens the focused DropDownButton",
    )?;
    gallery.set_email_enabled(false);
    window.take_snapshot()?;
    check(
        !gallery.get_drop_down_probe().open
            && gallery.get_drop_down_probe().state == ButtonState::Disabled,
        "disabling an open DropDownButton closes its flyout",
    )?;
    gallery.set_email_enabled(true);
    let sample = gallery.get_drop_down_probe();
    let arrow_side = LogicalPosition::new(
        sample.text_button.x + sample.text_button.width - 6.,
        center(&sample.text_button).y,
    );
    click(window, arrow_side);
    check(
        gallery.get_drop_down_probe().open,
        "DropDownButton also opens from its chevron side",
    )?;
    gallery.set_page(GalleryPage::ToggleSplitButton);
    window.take_snapshot()?;
    check(
        !gallery.get_drop_down_probe().open,
        "leaving the page closes DropDownButton",
    )?;

    gallery.set_page(GalleryPage::DropDownButton);
    window.take_snapshot()?;
    click(window, center(&sample.icon_button));
    check(
        gallery.get_drop_down_probe().icon_open && !gallery.get_drop_down_probe().open,
        "the icon-only DropDownButton opens its own menu",
    )?;
    let panel = gallery.get_drop_down_probe().panel;
    let ui = gallery.global::<FluentTheme>().get_ui_scale();
    check(
        panel.width > 98. * ui,
        "MenuFlyout reserves the icon column in the icon sample",
    )?;
    click(
        window,
        LogicalPosition::new(panel.x + 24. * ui, panel.y + 12. * ui),
    );
    check(
        !gallery.get_drop_down_probe().icon_open && gallery.get_drop_down_probe().command == "Send",
        "a mouse click invokes the icon menu's first item",
    )?;
    click(window, center(&sample.icon_button));
    click(window, away);
    click(window, center(&sample.disabled_button));
    let closed = gallery.get_drop_down_probe();
    check(
        !closed.icon_open
            && !closed.disabled_open
            && closed.disabled_state == ButtonState::Disabled,
        "outside clicks dismiss the icon menu, and the disabled sample never opens",
    )
}

fn verify_toggle_split(gallery: &GalleryWindow) -> SmokeResult {
    gallery.set_page(GalleryPage::ToggleSplitButton);
    gallery.set_list_format_checked(false);
    let window = gallery.window();
    let away = LogicalPosition::new(1., 1.);
    window.take_snapshot()?;
    let initial = gallery.get_toggle_split_probe();
    click(window, initial.primary);
    let primary = gallery.get_toggle_split_probe();
    check(
        gallery.get_list_format_checked()
            && primary.observed_checked
            && primary.activations == initial.activations + 1
            && !primary.open,
        "ToggleSplitButton toggles before invoking its primary click handler",
    )?;
    key(window, Key::Space);
    check(
        !gallery.get_list_format_checked() && !gallery.get_toggle_split_probe().observed_checked,
        "Space changes the primary toggle",
    )?;
    key(window, Key::Return);
    check(
        gallery.get_list_format_checked() && gallery.get_toggle_split_probe().observed_checked,
        "Enter changes the primary toggle",
    )?;

    let actions = gallery.get_toggle_split_probe().activations;
    for theme in [
        FluentColorScheme::Light,
        FluentColorScheme::Dark,
        FluentColorScheme::HighContrast,
    ] {
        gallery.invoke_select_theme(theme);
        hover(window, away);
        let probe = gallery.get_toggle_split_probe();
        let primary_face = LogicalPosition::new(probe.bounds.x + 5., probe.primary.y);
        let secondary_face = LogicalPosition::new(probe.secondary.x - 10., probe.secondary.y);
        gallery.set_list_format_checked(false);
        let normal = read_rgb(window, primary_face)?;
        gallery.set_list_format_checked(true);
        let checked = read_rgb(window, primary_face)?;
        let secondary_checked = read_rgb(window, secondary_face)?;
        check(
            checked != normal,
            "ToggleSplitButton renders a selected face in each theme",
        )?;
        hover(window, probe.secondary);
        let hovered = gallery.get_toggle_split_probe();
        check(
            hovered.primary_state == ButtonState::Normal
                && hovered.secondary_state == ButtonState::PointerOver
                && read_rgb(window, primary_face)? == checked
                && read_rgb(window, secondary_face)? != secondary_checked,
            "ToggleSplitButton highlights only the hovered secondary face",
        )?;
        pointer(window, probe.secondary, true);
        let pressed = gallery.get_toggle_split_probe();
        check(
            pressed.primary_state == ButtonState::Normal
                && pressed.secondary_state == ButtonState::Pressed
                && !pressed.open
                && read_rgb(window, primary_face)? == checked,
            "ToggleSplitButton presses only the mouse-targeted side before opening",
        )?;
        pointer(window, probe.secondary, false);
        let open = gallery.get_toggle_split_probe();
        check(
            open.open
                && open.primary_state == ButtonState::Pressed
                && open.secondary_state == ButtonState::Pressed
                && read_rgb(window, primary_face)? != checked
                && gallery.get_list_format_checked()
                && open.activations == actions,
            "the checked open state affects both faces without triggering the primary action",
        )?;
        let scale = gallery.global::<FluentTheme>().get_ui_scale();
        let variation = if theme == FluentColorScheme::HighContrast {
            2.
        } else {
            0.
        };
        let left = (open.bounds.x + (open.bounds.width - open.panel.width) / 2.).max(4. * scale);
        check(
            (open.panel.width - (118. + variation) * scale).abs() < 0.75
                && (open.panel.height - (76. + variation) * scale).abs() < 0.75
                && (open.panel.x - left).abs() < 0.75
                && (open.panel.y - open.bounds.y - open.bounds.height - 4. * scale).abs() < 0.75,
            "ToggleSplitButton centers the ordinary Flyout and sizes it from two icon tiles and presenter padding",
        )?;
        key(window, Key::Escape);
        check(
            !gallery.get_toggle_split_probe().open && gallery.get_toggle_split_probe().focused,
            "ToggleSplitButton restores focus after Escape",
        )?;
    }
    gallery.set_list_format_checked(false);
    key(window, Key::F4);
    check(
        gallery.get_toggle_split_probe().open && !gallery.get_list_format_checked(),
        "F4 opens ToggleSplitButton without changing its checked value",
    )?;
    key(window, Key::Tab);
    key(window, Key::Return);
    let roman = gallery.get_toggle_split_probe();
    check(
        roman.roman
            && gallery.get_list_format_checked()
            && !roman.open
            && roman.activations == actions,
        "choosing a list style closes the flyout and selects the format without a primary click",
    )?;
    window.dispatch_event(WindowEvent::KeyPressed {
        text: Key::Alt.into(),
    });
    key(window, Key::DownArrow);
    window.dispatch_event(WindowEvent::KeyReleased {
        text: Key::Alt.into(),
    });
    check(
        gallery.get_toggle_split_probe().open,
        "Alt+Down opens ToggleSplitButton",
    )?;
    click(window, away);
    click(window, roman.disabled_primary);
    click(window, roman.disabled_secondary);
    let closed = gallery.get_toggle_split_probe();
    check(
        !closed.open
            && closed.focused
            && !closed.disabled_open
            && closed.disabled_checked
            && closed.disabled_state == ButtonState::Disabled,
        "outside click dismisses the flyout and the disabled sample retains its checked value",
    )
}

pub(super) fn verify_compact(gallery: &GalleryWindow, width: f32) -> SmokeResult {
    let window = gallery.window();
    gallery.set_page(GalleryPage::DropDownButton);
    window.take_snapshot()?;
    let drop_down = gallery.get_drop_down_probe();
    click(window, center(&drop_down.text_button));
    let drop_down = gallery.get_drop_down_probe();
    check(
        drop_down.open
            && drop_down.text_button.height >= 44.
            && drop_down.panel.x + drop_down.panel.width <= width,
        "DropDownButton has a touch-sized target and an unclipped menu in a narrow window",
    )?;
    key(window, Key::Escape);
    gallery.set_page(GalleryPage::ToggleSplitButton);
    window.take_snapshot()?;
    let split = gallery.get_toggle_split_probe();
    click(window, split.secondary);
    let split = gallery.get_toggle_split_probe();
    check(
        split.open
            && split.bounds.height >= 44.
            && split.secondary.x - split.primary.x >= 44.
            && split.panel.x + split.panel.width <= width,
        "ToggleSplitButton keeps both touch-sized targets and flyout inside a narrow window",
    )?;
    key(window, Key::Escape);
    Ok(())
}

pub(super) fn verify_batch(
    gallery: &GalleryWindow,
    frames: Rc<Cell<u64>>,
    finish: impl FnOnce(BatchResult) + 'static,
) {
    gallery.global::<FluentTheme>().set_reduced_motion(true);
    gallery.set_shell_pane_visible(false);
    if let Err(error) = verify(gallery) {
        finish(Err(error));
        return;
    }
    let size = gallery.window().size();
    let wide = (size.width, size.height);
    gallery.set_touch_targets(true);
    gallery.window().set_size(LogicalSize::new(320., 640.));
    let weak = gallery.as_weak();
    slint::Timer::single_shot(Duration::from_millis(250), move || {
        let Some(gallery) = weak.upgrade() else {
            return finish(Err("Gallery closed before the narrow flyout checks".into()));
        };
        let size = gallery.window().size();
        let compact = (size.width, size.height);
        let width = size.to_logical(gallery.window().scale_factor()).width;
        if let Err(error) = verify_compact(&gallery, width) {
            return finish(Err(error));
        }
        verify_motion(&gallery, frames, move |verified| {
            finish(verified.map(|()| (wide, compact)))
        });
    });
}

fn verify_motion(
    gallery: &GalleryWindow,
    frames: Rc<Cell<u64>>,
    finish: impl FnOnce(SmokeResult) + 'static,
) {
    gallery.set_page(GalleryPage::DropDownButton);
    gallery.global::<FluentTheme>().set_reduced_motion(false);
    let weak = gallery.as_weak();
    slint::Timer::single_shot(Duration::from_millis(100), move || {
        let Some(gallery) = weak.upgrade() else {
            return finish(Err("Gallery closed before DropDownButton motion".into()));
        };
        let position = center(&gallery.get_drop_down_probe().text_button);
        pointer(gallery.window(), position, true);
        slint::Timer::single_shot(Duration::from_millis(70), move || {
            let Some(gallery) = weak.upgrade() else {
                return finish(Err("Gallery closed during DropDownButton press".into()));
            };
            let pressed = gallery.get_drop_down_probe();
            if let Err(error) = check(
                (0. ..0.312).contains(&pressed.chevron_progress) && pressed.chevron_progress > 0.,
                "DropDownButton renders an intermediate chevron press frame",
            ) {
                return finish(Err(error));
            }
            pointer(gallery.window(), position, false);
            let panel = gallery.get_drop_down_probe().panel;
            let far = flyout_sample(&gallery, &panel);
            let early = match read_rgb(gallery.window(), far) {
                Ok(pixel) => pixel,
                Err(error) => return finish(Err(error)),
            };
            slint::Timer::single_shot(Duration::from_millis(420), move || {
                let Some(gallery) = weak.upgrade() else {
                    return finish(Err("Gallery closed during the menu entrance".into()));
                };
                let verified = (|| {
                    let settled = gallery.get_drop_down_probe();
                    let surface = read_rgb(gallery.window(), far)?;
                    check(
                        settled.open && early != surface && surface == acrylic_rgb(&gallery),
                        "MenuFlyout reveals its far edge instead of painting the whole panel immediately",
                    )?;
                    check(
                        crate::animated_icon::split_chevron_offset(settled.chevron_progress).abs()
                            < 0.01,
                        "DropDownButton chevron returns to rest with the menu open",
                    )?;
                    key(gallery.window(), Key::Escape);
                    check(
                        gallery.get_drop_down_probe().open,
                        "MenuFlyout stays mounted during the exit fade",
                    )
                })();
                if let Err(error) = verified {
                    return finish(Err(error));
                }
                slint::Timer::single_shot(Duration::from_millis(140), move || {
                    let Some(gallery) = weak.upgrade() else {
                        return finish(Err("Gallery closed during the menu exit".into()));
                    };
                    let closed = gallery.get_drop_down_probe();
                    if let Err(error) = check(
                        !closed.open && closed.focused,
                        "MenuFlyout restores trigger focus after the exit fade",
                    ) {
                        return finish(Err(error));
                    }
                    verify_split_motion(&gallery, frames, finish);
                });
            });
        });
    });
}

fn flyout_sample(gallery: &GalleryWindow, panel: &OverlayArea) -> LogicalPosition {
    let scale = gallery.global::<FluentTheme>().get_ui_scale();
    LogicalPosition::new(panel.x + 7. * scale, panel.y + panel.height - 8. * scale)
}

fn acrylic_rgb(gallery: &GalleryWindow) -> [u8; 3] {
    let color = gallery.global::<FluentTheme>().get_acrylic_fallback();
    [color.red(), color.green(), color.blue()]
}

fn verify_split_motion(
    gallery: &GalleryWindow,
    frames: Rc<Cell<u64>>,
    finish: impl FnOnce(SmokeResult) + 'static,
) {
    gallery.set_page(GalleryPage::ToggleSplitButton);
    let opening = (|| {
        let window = gallery.window();
        window.take_snapshot()?;
        click(window, gallery.get_toggle_split_probe().secondary);
        let split = gallery.get_toggle_split_probe();
        check(split.open, "ToggleSplitButton opens its regular Flyout")?;
        let far = flyout_sample(gallery, &split.panel);
        Ok::<_, Box<dyn Error>>((far, read_rgb(window, far)?))
    })();
    let (far, early) = match opening {
        Ok(sample) => sample,
        Err(error) => return finish(Err(error)),
    };
    let weak = gallery.as_weak();
    slint::Timer::single_shot(Duration::from_millis(320), move || {
        let Some(gallery) = weak.upgrade() else {
            return finish(Err("Gallery closed during the Flyout entrance".into()));
        };
        let verified = (|| {
            let settled = read_rgb(gallery.window(), far)?;
            check(
                early != settled && settled == acrylic_rgb(&gallery),
                "the regular Flyout translates into its final position",
            )
        })();
        if let Err(error) = verified {
            return finish(Err(error));
        }
        let before = frames.get();
        slint::Timer::single_shot(Duration::from_millis(300), move || {
            let Some(gallery) = weak.upgrade() else {
                return finish(Err("Gallery closed before the Flyout exit".into()));
            };
            let verified = (|| {
                check(
                    frames.get() - before <= 2,
                    "the settled Flyout stops redrawing",
                )?;
                key(gallery.window(), Key::Escape);
                check(
                    gallery.get_toggle_split_probe().open,
                    "the regular Flyout stays mounted during its exit fade",
                )
            })();
            if let Err(error) = verified {
                return finish(Err(error));
            }
            slint::Timer::single_shot(Duration::from_millis(140), move || {
                let Some(gallery) = weak.upgrade() else {
                    return finish(Err("Gallery closed during the Flyout exit".into()));
                };
                let closed = gallery.get_toggle_split_probe();
                finish(
                    check(
                        !closed.open && closed.focused,
                        "the regular Flyout restores trigger focus after its exit fade",
                    )
                    .and_then(|()| verify_live_material(&gallery)),
                );
            });
        });
    });
}

fn verify_live_material(gallery: &GalleryWindow) -> SmokeResult {
    let theme = gallery.global::<FluentTheme>();
    let window = gallery.window();
    theme.set_reduced_motion(true);
    let verified = (|| {
        for page in [GalleryPage::DropDownButton, GalleryPage::ToggleSplitButton] {
            gallery.set_page(page);
            window.take_snapshot()?;
            let sample = |reduce_transparency| -> Result<[u8; 3], Box<dyn Error>> {
                theme.set_reduce_transparency(reduce_transparency);
                let panel = if page == GalleryPage::DropDownButton {
                    click(window, center(&gallery.get_drop_down_probe().text_button));
                    let probe = gallery.get_drop_down_probe();
                    check(probe.open, "DropDownButton opens to sample its material")?;
                    probe.panel
                } else {
                    click(window, gallery.get_toggle_split_probe().secondary);
                    let probe = gallery.get_toggle_split_probe();
                    check(probe.open, "ToggleSplitButton opens to sample its material")?;
                    probe.panel
                };
                let color = read_rgb(window, flyout_sample(gallery, &panel))?;
                key(window, Key::Escape);
                Ok(color)
            };
            let solid = sample(true)?;
            let live = sample(false)?;
            if solid != acrylic_rgb(gallery) || live == solid {
                return Err(
                    format!("{page:?} Acrylic pixels: solid {solid:?}, live {live:?}").into(),
                );
            }
        }
        Ok(())
    })();
    theme.set_reduce_transparency(true);
    verified
}
