use std::{cell::Cell, error::Error, rc::Rc, time::Duration};

use slint::{
    ComponentHandle, LogicalPosition, LogicalSize, Window,
    platform::{Key, WindowEvent},
};

use super::{check, click, overlays::key, read_rgb};
use crate::{
    FluentColorScheme, FluentTheme, GalleryPage, GalleryWindow, OverlayArea,
    PipsPagerButtonVisibility,
};

type SmokeResult = Result<(), Box<dyn Error>>;
type Screens = ((u32, u32), (u32, u32));

fn center(bounds: &OverlayArea) -> LogicalPosition {
    LogicalPosition::new(bounds.x + bounds.width / 2., bounds.y + bounds.height / 2.)
}

fn hover(window: &Window, position: LogicalPosition) {
    window.dispatch_event(WindowEvent::PointerMoved { position });
}

pub(super) fn verify(gallery: &GalleryWindow) -> SmokeResult {
    verify_selector(gallery)?;
    verify_pager(gallery)?;
    gallery.invoke_select_theme(FluentColorScheme::Light);
    Ok(())
}

fn verify_selector(gallery: &GalleryWindow) -> SmokeResult {
    gallery.set_page(GalleryPage::SelectorBar);
    let window = gallery.window();
    window.take_snapshot()?;
    gallery.invoke_select_selector_item(-1);
    gallery.invoke_focus_selector_clear();
    key(window, Key::Backtab);
    let first = gallery.get_selector_bar_probe();
    check(
        first.selected == 0 && first.focused == 0,
        "SelectorBar enters at the first available item when it has no selection",
    )?;
    key(window, Key::RightArrow);
    check(
        gallery.get_selector_bar_probe().selected == 1,
        "SelectorBar selects with the right arrow",
    )?;
    key(window, Key::RightArrow);
    check(
        gallery.get_selector_bar_probe().selected == 3,
        "SelectorBar skips a disabled item",
    )?;
    gallery.invoke_select_selector_item(2);
    check(
        gallery.get_selector_bar_probe().selected == 3,
        "SelectorBar rejects programmatic selection of a disabled item",
    )?;
    key(window, Key::Tab);
    check(
        gallery.get_selector_bar_probe().clear_focused,
        "SelectorBar occupies one Tab stop",
    )?;
    click(
        window,
        LogicalPosition::new(first.bar_start.x + 14., first.bar_start.y),
    );
    check(
        gallery.get_selector_bar_probe().selected == 0,
        "clicking a SelectorBar item selects it",
    )?;

    for theme in [
        FluentColorScheme::Light,
        FluentColorScheme::Dark,
        FluentColorScheme::HighContrast,
    ] {
        gallery.invoke_select_theme(theme);
        gallery.invoke_select_selector_item(0);
        let selected = selector_pill_pixels(gallery)?;
        gallery.invoke_select_selector_item(-1);
        let cleared = selector_pill_pixels(gallery)?;
        check(
            selected > cleared,
            "SelectorBar draws the selection indicator in every theme",
        )?;
    }
    Ok(())
}

fn selector_pill_pixels(gallery: &GalleryWindow) -> Result<usize, Box<dyn Error>> {
    let window = gallery.window();
    let frame = window.take_snapshot()?;
    let probe = gallery.get_selector_bar_probe();
    let scale = window.scale_factor();
    let theme = gallery.global::<FluentTheme>();
    let color = if theme.get_high_contrast() {
        theme.get_contrast_highlight()
    } else {
        theme.get_accent_fill()
    };
    let x0 = (probe.bar_start.x * scale).ceil() as usize;
    let x1 = (probe.bar_end.x * scale).floor() as usize;
    let y0 = (probe.bar_start.y * scale).ceil() as usize;
    let y1 = (probe.bar_bottom * scale).floor() as usize;
    Ok((y0..y1)
        .flat_map(|y| (x0..x1).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            frame
                .as_slice()
                .get(y * frame.width() as usize + x)
                .is_some_and(|pixel| {
                    pixel.r.abs_diff(color.red()) <= 3
                        && pixel.g.abs_diff(color.green()) <= 3
                        && pixel.b.abs_diff(color.blue()) <= 3
                })
        })
        .count())
}

fn verify_pager(gallery: &GalleryWindow) -> SmokeResult {
    gallery.set_page(GalleryPage::PipsPager);
    let window = gallery.window();
    gallery.set_pips_count(10);
    gallery.set_pips_index(0);
    gallery.set_pips_vertical(false);
    gallery.set_pips_wrap(false);
    gallery.set_pips_enabled(true);
    gallery.set_pips_arrows(PipsPagerButtonVisibility::Visible);
    window.take_snapshot()?;
    let first = gallery.get_pips_pager_probe();
    check(
        first.selected == 0
            && first.first == 0
            && first.visible_count == 5
            && !first.previous_shown
            && first.next_shown,
        "PipsPager reserves the unavailable leading arrow and displays five pips",
    )?;
    click(window, center(&first.next));
    check(
        gallery.get_pips_pager_probe().selected == 1,
        "PipsPager next arrow selects the next page",
    )?;
    click(window, gallery.get_pips_pager_probe().selected_center);
    key(window, Key::RightArrow);
    let preview = gallery.get_pips_pager_probe();
    check(
        preview.selected == 1 && preview.focused == 2,
        "PipsPager arrows move focus before activating a page",
    )?;
    key(window, Key::Space);
    check(
        gallery.get_pips_pager_probe().selected == 2,
        "Space activates the focused page",
    )?;
    key(window, Key::RightArrow);
    key(window, Key::Return);
    check(
        gallery.get_pips_pager_probe().selected == 3,
        "Enter activates a focused pip",
    )?;

    gallery.set_pips_wrap(true);
    gallery.set_pips_index(0);
    window.take_snapshot()?;
    let wrapped = gallery.get_pips_pager_probe();
    check(
        wrapped.previous_shown,
        "wrapping displays the leading arrow",
    )?;
    click(window, center(&wrapped.previous));
    let last = gallery.get_pips_pager_probe();
    check(
        last.selected == 9 && last.first == 5,
        "PipsPager wraps to the last visible window",
    )?;
    click(window, center(&last.next));
    check(
        gallery.get_pips_pager_probe().selected == 0,
        "PipsPager wraps back to the first page",
    )?;

    gallery.set_pips_vertical(true);
    window.take_snapshot()?;
    let vertical = gallery.get_pips_pager_probe();
    check(
        vertical.next.y > vertical.previous.y && (vertical.next.x - vertical.previous.x).abs() < 1.,
        "PipsPager places navigation buttons vertically",
    )?;
    click(window, vertical.selected_center);
    key(window, Key::DownArrow);
    key(window, Key::Return);
    check(
        gallery.get_pips_pager_probe().selected == 1,
        "vertical PipsPager responds to Down and Enter",
    )?;

    gallery.set_pips_vertical(false);
    gallery.set_pips_arrows(PipsPagerButtonVisibility::VisibleOnPointerOver);
    window.take_snapshot()?;
    let before = gallery.get_pips_pager_probe();
    click(window, before.selected_center);
    hover(window, LogicalPosition::new(1., 1.));
    check(
        !gallery.get_pips_pager_probe().next_shown,
        "pointer-only arrows hide outside the pager",
    )?;
    hover(window, before.selected_center);
    check(
        gallery.get_pips_pager_probe().next_shown,
        "hovering the pips reveals navigable arrows",
    )?;
    hover(window, center(&before.next));
    check(
        gallery.get_pips_pager_probe().next_shown,
        "moving onto the revealed arrow keeps it visible",
    )?;
    click(window, center(&before.next));
    check(
        gallery.get_pips_pager_probe().selected == 2,
        "the revealed arrow can be activated",
    )?;
    gallery.set_pips_arrows(PipsPagerButtonVisibility::Collapsed);
    window.take_snapshot()?;
    let collapsed = gallery.get_pips_pager_probe();
    check(
        !collapsed.next_shown && collapsed.bounds.width < before.bounds.width,
        "collapsed PipsPager arrows remove their reserved width",
    )?;

    for theme in [
        FluentColorScheme::Light,
        FluentColorScheme::Dark,
        FluentColorScheme::HighContrast,
    ] {
        gallery.invoke_select_theme(theme);
        hover(window, LogicalPosition::new(1., 1.));
        let pip = gallery.get_pips_pager_probe();
        let above = LogicalPosition::new(pip.selected_center.x, pip.bounds.y + 2.);
        check(
            read_rgb(window, pip.selected_center)? != read_rgb(window, above)?,
            "PipsPager paints a visible selection glyph in every theme",
        )?;
    }
    gallery.set_pips_enabled(false);
    click(window, gallery.get_pips_pager_probe().selected_center);
    key(window, Key::RightArrow);
    key(window, Key::Return);
    check(
        gallery.get_pips_pager_probe().selected == 2,
        "a disabled PipsPager does not change page",
    )?;
    gallery.set_pips_enabled(true);

    gallery.set_pips_count(100_000);
    gallery.set_pips_index(99_999);
    window.take_snapshot()?;
    let large = gallery.get_pips_pager_probe();
    check(
        large.selected == 99_999 && large.first == 99_995 && large.visible_count == 5,
        "PipsPager keeps a bounded set of pips at the end of 100,000 pages",
    )?;
    gallery.set_pips_count(3);
    window.take_snapshot()?;
    check(
        gallery.get_pips_pager_probe().selected == 2
            && gallery.get_pips_pager_probe().visible_count == 3,
        "shrinking the page count clamps PipsPager selection",
    )?;
    gallery.set_pips_index(i32::MAX);
    window.take_snapshot()?;
    check(
        gallery.get_pips_index() == 2,
        "programmatic PipsPager requests normalize even when the effective page stays the same",
    )?;
    gallery.set_pips_count(0);
    window.take_snapshot()?;
    let empty = gallery.get_pips_pager_probe();
    check(
        empty.selected == -1
            && empty.visible_count == 0
            && !empty.next_shown
            && !empty.previous_shown,
        "an empty PipsPager has neither a selection nor navigation buttons",
    )?;
    gallery.set_pips_count(-1);
    gallery.set_pips_index(100_000);
    window.take_snapshot()?;
    let infinite = gallery.get_pips_pager_probe();
    check(
        infinite.selected == 100_000 && infinite.visible_count == 5 && infinite.first == 99_998,
        "infinite PipsPager exposes a fixed window around the selected page",
    )?;
    gallery.set_pips_count(10);
    gallery.set_pips_index(0);
    gallery.set_pips_wrap(false);
    gallery.set_pips_arrows(PipsPagerButtonVisibility::Visible);
    Ok(())
}

pub(super) fn verify_compact(gallery: &GalleryWindow, width: f32) -> SmokeResult {
    let window = gallery.window();
    gallery.set_page(GalleryPage::SelectorBar);
    gallery.invoke_select_selector_item(0);
    window.take_snapshot()?;
    gallery.invoke_select_selector_item(4);
    window.take_snapshot()?;
    let bar = gallery.get_selector_bar_probe();
    check(
        bar.selected == 4 && bar.offset > 0. && bar.bar_end.x <= width,
        "SelectorBar scrolls the final item into view in a narrow window",
    )?;
    gallery.invoke_select_selector_item(0);
    window.take_snapshot()?;
    check(
        gallery.get_selector_bar_probe().offset.abs() < 0.5,
        "selecting the first item scrolls SelectorBar back to its origin",
    )?;
    gallery.set_page(GalleryPage::PipsPager);
    gallery.set_pips_arrows(PipsPagerButtonVisibility::Visible);
    gallery.set_pips_index(0);
    window.take_snapshot()?;
    let pips = gallery.get_pips_pager_probe();
    check(
        pips.visible_count > 0
            && pips.visible_count < 5
            && pips.bounds.x + pips.bounds.width <= width
            && pips.next.width >= 44.,
        "compact PipsPager reduces the number of pips and keeps touch-sized targets",
    )?;
    click(window, center(&pips.next));
    check(
        gallery.get_pips_pager_probe().selected == 1,
        "compact PipsPager next arrow works",
    )
}

pub(super) fn verify_batch(
    gallery: &GalleryWindow,
    frames: Rc<Cell<u64>>,
    done: impl FnOnce(Result<Screens, Box<dyn Error>>) + 'static,
) {
    gallery.global::<FluentTheme>().set_reduced_motion(true);
    gallery.set_shell_pane_visible(false);
    if let Err(error) = verify(gallery) {
        done(Err(error));
        return;
    }
    let size = gallery.window().size();
    let wide = (size.width, size.height);
    gallery.set_page(GalleryPage::SelectorBar);
    gallery.invoke_select_selector_item(-1);
    if let Err(error) = gallery.window().take_snapshot() {
        return done(Err(error.into()));
    }
    gallery.global::<FluentTheme>().set_reduced_motion(false);
    gallery.invoke_select_selector_item(1);
    if let Err(error) = gallery.window().take_snapshot() {
        return done(Err(error.into()));
    }

    let weak = gallery.as_weak();
    slint::Timer::single_shot(Duration::from_millis(350), move || {
        let Some(gallery) = weak.upgrade() else {
            return done(Err("Gallery closed during SelectorBar motion".into()));
        };
        let settled = match selector_pill_pixels(&gallery) {
            Ok(count) => count,
            Err(error) => return done(Err(error)),
        };
        let rendered = frames.get();
        slint::Timer::single_shot(Duration::from_millis(450), move || {
            let Some(gallery) = weak.upgrade() else {
                return done(Err("Gallery closed before the idle check".into()));
            };
            let verified = (|| {
                if rendered == 0 {
                    println!(
                        "This Skia surface does not report frame notifications; checking settled pixels"
                    );
                } else {
                    check(
                        frames.get() - rendered <= 2,
                        "SelectorBar becomes idle after changing selection",
                    )?;
                }
                let later = selector_pill_pixels(&gallery)?;
                if settled == 0 || later != settled {
                    return Err(format!(
                        "SelectorBar indicator was unstable ({settled} then {later} pixels), {:?}",
                        gallery.get_selector_bar_probe()
                    )
                    .into());
                }
                Ok(())
            })();
            if let Err(error) = verified {
                return done(Err(error));
            }
            verify_narrow(&gallery, wide, done);
        });
    });
}

fn verify_narrow(
    gallery: &GalleryWindow,
    wide: (u32, u32),
    done: impl FnOnce(Result<Screens, Box<dyn Error>>) + 'static,
) {
    gallery.global::<FluentTheme>().set_reduced_motion(true);
    gallery.set_touch_targets(true);
    gallery.window().set_size(LogicalSize::new(320., 640.));
    let weak = gallery.as_weak();
    slint::Timer::single_shot(Duration::from_millis(250), move || {
        let Some(gallery) = weak.upgrade() else {
            return done(Err("Gallery closed before compact selector checks".into()));
        };
        let size = gallery.window().size();
        let width = size.to_logical(gallery.window().scale_factor()).width;
        done(verify_compact(&gallery, width).map(|()| (wide, (size.width, size.height))));
    });
}
