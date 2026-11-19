use slint::{
    ComponentHandle, LogicalPosition, LogicalSize, Window,
    platform::{Key, WindowEvent},
};

use crate::{FluentColorScheme, FluentTheme, GalleryPage, GalleryWindow, OverlayPlacement};

use super::{check, click, read_rgb};

type SmokeResult = Result<(), Box<dyn std::error::Error>>;

pub(super) fn verify_flyout(gallery: &GalleryWindow) -> SmokeResult {
    gallery.set_page(GalleryPage::Overlays);
    gallery.invoke_select_theme(FluentColorScheme::Light);
    let window = gallery.window();
    let anchor = gallery.get_flyout_anchor();
    let trigger = LogicalPosition::new(anchor.x + anchor.width / 2., anchor.y + anchor.height / 2.);
    let ring_edge = LogicalPosition::new(trigger.x, anchor.y - 2.);
    let unfocused_edge = read_rgb(window, ring_edge)?;
    click(window, trigger);
    window.take_snapshot()?;
    let panel = gallery.get_flyout_bounds();
    let has_room_above =
        anchor.y >= panel.height + 16. * gallery.global::<FluentTheme>().get_ui_scale();
    check(
        gallery.get_flyout_open()
            && gallery.get_flyout_placement() == OverlayPlacement::Top
            && if has_room_above {
                panel.y + panel.height < anchor.y
            } else {
                panel.y > anchor.y + anchor.height
            },
        "Flyout prefers the space above its anchor and flips if the panel cannot fit",
    )?;
    if gallery.get_flyout_trigger_focused()
        || !gallery.get_flyout_action_focused()
        || gallery.get_flyout_action_focus_visible()
    {
        return Err(format!(
            "pointer-opened Flyout should focus the first action without a keyboard ring, got anchor focus {}, action focus {}, and ring {}",
            gallery.get_flyout_trigger_focused(), gallery.get_flyout_action_focused(), gallery.get_flyout_action_focus_visible(),
        ).into());
    }
    key(window, Key::Return);
    check(
        !gallery.get_flyout_open()
            && gallery.get_flyout_confirmations() == 1
            && gallery.get_flyout_trigger_focused()
            && !gallery.get_flyout_trigger_focus_visible(),
        "the initial Flyout action runs and restores anchor focus",
    )?;

    key(window, Key::Space);
    window.take_snapshot()?;
    check(
        gallery.get_flyout_open()
            && gallery.get_flyout_action_focused()
            && gallery.get_flyout_action_focus_visible(),
        "keyboard-opened Flyout displays focus on its first action",
    )?;
    key(window, Key::Escape);
    check(
        !gallery.get_flyout_open() && gallery.get_flyout_trigger_focus_visible(),
        "Escape restores the keyboard focus ring on the anchor",
    )?;
    check(
        read_rgb(window, ring_edge)? != unfocused_edge,
        "Skia paints the restored Flyout anchor's keyboard outline",
    )?;
    key(window, Key::Space);
    click(window, LogicalPosition::new(2., 2.));
    check(
        !gallery.get_flyout_open()
            && gallery.get_flyout_trigger_focused()
            && !gallery.get_flyout_trigger_focus_visible()
            && read_rgb(window, ring_edge)? == unfocused_edge,
        "pointer light-dismiss restores focus without a ring",
    )?;

    gallery.set_flyout_placement(OverlayPlacement::Left);
    click(window, trigger);
    check(
        gallery.get_flyout_open() && gallery.get_flyout_bounds().x >= anchor.x + anchor.width,
        "Flyout flips a left placement at the viewport edge",
    )?;
    click(window, LogicalPosition::new(2., 2.));
    check(
        !gallery.get_flyout_open() && gallery.get_flyout_trigger_focused(),
        "outside click light-dismisses Flyout",
    )?;

    gallery.set_flyout_placement(OverlayPlacement::Top);
    click(window, trigger);
    let size = window.size().to_logical(window.scale_factor());
    window.dispatch_event(WindowEvent::Resized {
        size: LogicalSize::new(size.width - 20., size.height),
    });
    window.take_snapshot()?;
    let closed = !gallery.get_flyout_open() && gallery.get_flyout_trigger_focused();
    window.dispatch_event(WindowEvent::Resized { size });
    check(closed, "window resize closes Flyout and restores focus")?;

    click(window, trigger);
    gallery.set_page(GalleryPage::Button);
    window.take_snapshot()?;
    check(!gallery.get_flyout_open(), "leaving the page closes Flyout")?;
    gallery.set_page(GalleryPage::Overlays);
    gallery.set_flyout_enabled(false);
    click(window, trigger);
    check(
        !gallery.get_flyout_open(),
        "a disabled anchor cannot open Flyout",
    )?;
    gallery.set_flyout_enabled(true);
    Ok(())
}

pub(super) fn verify_flyout_compact(gallery: &GalleryWindow, width: f32) -> SmokeResult {
    gallery.set_page(GalleryPage::Overlays);
    let window = gallery.window();
    let anchor = gallery.get_flyout_anchor();
    check(
        anchor.height >= 44.,
        "the compact Flyout anchor has a touch-sized target",
    )?;
    let trigger = LogicalPosition::new(anchor.x + anchor.width / 2., anchor.y + anchor.height / 2.);
    for theme in [FluentColorScheme::Light, FluentColorScheme::Dark] {
        gallery.invoke_select_theme(theme);
        click(window, trigger);
        let panel = gallery.get_flyout_bounds();
        check(
            gallery.get_flyout_open()
                && panel.x >= 0.
                && panel.x + panel.width <= width
                && panel.y >= 0.
                && panel.height > 0.,
            "the compact Flyout fits the viewport",
        )?;
        let fill = gallery.global::<FluentTheme>().get_acrylic_fallback();
        check(
            read_rgb(window, LogicalPosition::new(panel.x + 7., panel.y + 12.))?
                == [fill.red(), fill.green(), fill.blue()],
            "the Flyout paints its themed surface",
        )?;
        key(window, Key::Escape);
    }
    Ok(())
}

pub(super) fn key(window: &Window, key: Key) {
    window.dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window.dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}

pub(super) fn verify_context_menu(gallery: &GalleryWindow) -> SmokeResult {
    use slint::platform::PointerEventButton;
    gallery.set_page(GalleryPage::Overlays);
    let window = gallery.window();
    let anchor = center(&gallery.get_context_menu_anchor());
    let right = PointerEventButton::Right;
    window.dispatch_event(WindowEvent::PointerPressed {
        position: anchor,
        button: right,
    });
    check(
        !gallery.get_context_menu_open(),
        "a right press waits for release",
    )?;
    window.dispatch_event(WindowEvent::PointerReleased {
        position: anchor,
        button: right,
    });
    window.take_snapshot()?;
    let panel = gallery.get_context_menu_bounds();
    check(
        gallery.get_context_menu_open()
            && gallery.get_context_menu_focus() == 0
            && !gallery.get_context_menu_focus_visible()
            && (panel.x - anchor.x).abs() <= 1.
            && (panel.y - anchor.y).abs() <= 1.,
        "secondary click places ContextMenu at the pointer and focuses the first command",
    )?;
    key(window, Key::DownArrow);
    window.take_snapshot()?;
    check(
        gallery.get_context_menu_focus() == 2,
        "menu navigation skips disabled commands",
    )?;
    key(window, Key::DownArrow);
    window.take_snapshot()?;
    check(
        gallery.get_context_menu_focus() == 0,
        "menu navigation wraps from last to first",
    )?;
    key(window, Key::UpArrow);
    key(window, Key::Return);
    check(
        !gallery.get_context_menu_open()
            && gallery.get_context_menu_last_command() == 2
            && gallery.get_context_menu_trigger_focused(),
        "Enter invokes and dismisses the menu",
    )?;

    window.dispatch_event(WindowEvent::KeyPressed {
        text: Key::Shift.into(),
    });
    key(window, Key::F10);
    window.dispatch_event(WindowEvent::KeyReleased {
        text: Key::Shift.into(),
    });
    window.take_snapshot()?;
    if !gallery.get_context_menu_open()
        || gallery.get_context_menu_focus() != 0
        || !gallery.get_context_menu_focus_visible()
    {
        return Err(format!(
            "Shift-F10 should focus and outline the first command, got open {}, focus {}, and outline {}",
            gallery.get_context_menu_open(), gallery.get_context_menu_focus(), gallery.get_context_menu_focus_visible(),
        ).into());
    }
    key(window, Key::Escape);
    check(
        !gallery.get_context_menu_open() && gallery.get_context_menu_trigger_focused(),
        "Escape restores ContextMenu trigger focus",
    )?;
    key(window, Key::Menu);
    check(
        gallery.get_context_menu_open(),
        "the Menu key opens ContextMenu",
    )?;
    click(window, LogicalPosition::new(2., 2.));
    check(
        !gallery.get_context_menu_open() && gallery.get_context_menu_last_command() == 2,
        "outside click dismisses without invoking a command",
    )?;
    click(window, anchor);
    let panel = gallery.get_context_menu_bounds();
    click(window, LogicalPosition::new(panel.x + 16., panel.y + 60.));
    check(
        gallery.get_context_menu_open() && gallery.get_context_menu_last_command() == 2,
        "a disabled command ignores pointer activation",
    )?;
    click(window, LogicalPosition::new(panel.x + 16., panel.y + 20.));
    check(
        !gallery.get_context_menu_open() && gallery.get_context_menu_last_command() == 0,
        "pointer activation invokes the enabled command and dismisses",
    )?;
    gallery.set_context_menu_enabled(false);
    window.dispatch_event(WindowEvent::PointerPressed {
        position: anchor,
        button: right,
    });
    window.dispatch_event(WindowEvent::PointerReleased {
        position: anchor,
        button: right,
    });
    check(
        !gallery.get_context_menu_open(),
        "a disabled target has no context menu",
    )?;
    gallery.set_context_menu_enabled(true);
    Ok(())
}

pub(super) fn verify_context_compact(gallery: &GalleryWindow, width: f32) -> SmokeResult {
    gallery.set_page(GalleryPage::Overlays);
    let window = gallery.window();
    let anchor = gallery.get_context_menu_anchor();
    check(
        anchor.height >= 44.,
        "the compact menu trigger has a touch-sized target",
    )?;
    for theme in [FluentColorScheme::Light, FluentColorScheme::Dark] {
        gallery.invoke_select_theme(theme);
        click(window, center(&anchor));
        let panel = gallery.get_context_menu_bounds();
        let height = window.size().to_logical(window.scale_factor()).height;
        check(
            gallery.get_context_menu_open()
                && panel.x >= 0.
                && panel.x + panel.width <= width
                && panel.y >= 0.
                && panel.y + panel.height <= height,
            "the compact menu stays in the viewport",
        )?;
        let fill = gallery.global::<FluentTheme>().get_acrylic_fallback();
        check(
            read_rgb(window, LogicalPosition::new(panel.x + 2., panel.y + 22.))?
                == [fill.red(), fill.green(), fill.blue()],
            "the compact menu paints its theme surface",
        )?;
        key(window, Key::Escape);
    }
    Ok(())
}

pub(super) fn verify_tooltip(gallery: &GalleryWindow, finish: impl FnOnce(SmokeResult) + 'static) {
    use std::time::Duration;
    gallery.set_page(GalleryPage::Overlays);
    gallery.invoke_select_theme(FluentColorScheme::Dark);
    gallery.set_tooltip_delay(90);
    gallery.set_tooltip_duration(240);
    let point = center(&gallery.get_tooltip_anchor());
    let away = LogicalPosition::new(2., 2.);
    let window = gallery.window();
    window.dispatch_event(WindowEvent::PointerMoved { position: away });
    window.dispatch_event(WindowEvent::PointerMoved { position: point });
    let pending = window
        .take_snapshot()
        .map(|_| ())
        .map_err(Into::into)
        .and_then(|()| {
            check(
                !gallery.get_tooltip_open() && gallery.get_tooltip_pending(),
                "Tooltip waits for its hover delay",
            )
        });
    if let Err(error) = pending {
        finish(Err(error));
        return;
    }

    let weak = gallery.as_weak();
    slint::Timer::single_shot(Duration::from_millis(150), move || {
        let Some(gallery) = weak.upgrade() else {
            return;
        };
        if let Err(error) = verify_hover_tip(&gallery, point, away) {
            finish(Err(error));
            return;
        }

        let window = gallery.window();
        gallery.invoke_select_theme(FluentColorScheme::Light);
        window.dispatch_event(WindowEvent::PointerMoved { position: away });
        key(window, Key::Tab);
        for _ in 0..24 {
            if gallery.get_tooltip_trigger_focused() {
                break;
            }
            key(window, Key::Tab);
        }
        if let Err(error) = check(
            gallery.get_tooltip_trigger_focused(),
            "Tab reaches the Tooltip owner",
        ) {
            finish(Err(error));
            return;
        }
        let weak = gallery.as_weak();
        slint::Timer::single_shot(Duration::from_millis(145), move || {
            let Some(gallery) = weak.upgrade() else {
                return;
            };
            let window = gallery.window();
            let verify = || {
                check(
                    gallery.get_tooltip_open() && gallery.get_tooltip_trigger_focused(),
                    "keyboard focus opens Tooltip without giving focus to the bubble",
                )?;
                let panel = gallery.get_tooltip_bounds();
                let color = gallery.global::<FluentTheme>().get_acrylic_fallback();
                check(
                    read_rgb(window, LogicalPosition::new(panel.x + 5., panel.y + 8.))?
                        == [color.red(), color.green(), color.blue()],
                    "keyboard Tooltip paints its light surface",
                )?;
                key(window, Key::Escape);
                check(
                    !gallery.get_tooltip_open() && gallery.get_tooltip_trigger_focused(),
                    "Escape hides Tooltip while preserving owner focus",
                )
            };
            if let Err(error) = verify() {
                finish(Err(error));
                return;
            }

            key(window, Key::Tab);
            window.dispatch_event(WindowEvent::PointerMoved { position: point });
            let weak = gallery.as_weak();
            slint::Timer::single_shot(Duration::from_millis(450), move || {
                let Some(gallery) = weak.upgrade() else {
                    return;
                };
                finish(check(
                    !gallery.get_tooltip_open() && !gallery.get_tooltip_pending(),
                    "Tooltip expires without rescheduling under a stationary pointer",
                ));
            });
        });
    });
}

fn verify_hover_tip(
    gallery: &GalleryWindow,
    point: LogicalPosition,
    away: LogicalPosition,
) -> SmokeResult {
    let window = gallery.window();
    check(
        gallery.get_tooltip_open() && !gallery.get_tooltip_trigger_focused(),
        "hover opens Tooltip without stealing focus",
    )?;
    let bounds = gallery.get_tooltip_bounds();
    let viewport = window.size().to_logical(window.scale_factor());
    check(
        bounds.x >= 0.
            && bounds.x + bounds.width <= viewport.width
            && bounds.y >= 0.
            && bounds.y + bounds.height <= viewport.height,
        "Tooltip is contained within the window",
    )?;
    let fill = gallery.global::<FluentTheme>().get_acrylic_fallback();
    check(
        read_rgb(window, LogicalPosition::new(bounds.x + 5., bounds.y + 8.))?
            == [fill.red(), fill.green(), fill.blue()],
        "Tooltip paints the theme surface",
    )?;
    window.dispatch_event(WindowEvent::PointerMoved { position: away });
    window.take_snapshot()?;
    check(
        !gallery.get_tooltip_open(),
        "leaving the owner closes Tooltip",
    )?;
    let count = gallery.get_tooltip_activations();
    click(window, point);
    check(
        gallery.get_tooltip_activations() == count + 1 && !gallery.get_tooltip_open(),
        "Tooltip does not intercept owner activation",
    )
}

fn center(area: &crate::OverlayArea) -> LogicalPosition {
    LogicalPosition::new(area.x + area.width / 2., area.y + area.height / 2.)
}
