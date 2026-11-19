use std::{error::Error, time::Duration};

use slint::{
    ComponentHandle, LogicalPosition, LogicalSize, Model,
    platform::{Key, PointerEventButton, WindowEvent},
};

use crate::{
    FluentColorScheme, FluentPopupMotion, FluentTheme, GalleryPage, GalleryWindow,
    NavigationDisplayMode, NavigationPaneDisplayMode, NavigationProbe, NavigationTransition,
    PopupMotionProfile, PopupMotionTrack,
};

use super::{check, click, overlays::key, read_rgb};

type SmokeResult = Result<(), Box<dyn Error>>;

pub(super) fn verify_gallery_shell(gallery: &GalleryWindow) -> SmokeResult {
    let window = gallery.window();
    let original = window.size().to_logical(window.scale_factor());
    window.dispatch_event(WindowEvent::Resized {
        size: LogicalSize::new(980., 700.),
    });
    window.take_snapshot()?;
    let shell = gallery.get_shell_probe();
    let row = |index: usize| {
        LogicalPosition::new(
            shell.items.x + 24.,
            shell.items.y + shell.item_step * (index as f32 + 0.5),
        )
    };
    if shell.mode != NavigationDisplayMode::Expanded
        || !shell.pane_open
        || shell.selected != 0
        || gallery.get_page() != GalleryPage::Button
    {
        return Err(format!(
            "expected an expanded Gallery NavigationView with Button selected, got {shell:?}"
        )
        .into());
    }
    click(window, row(1));
    check(
        gallery.get_page() == GalleryPage::TextBox && gallery.get_shell_probe().selected == 1,
        "clicking a Gallery navigation item updates the main page and selection",
    )?;
    key(window, Key::DownArrow);
    key(window, Key::Return);
    check(
        gallery.get_page() == GalleryPage::Toggles,
        "keyboard navigation opens a Gallery destination",
    )?;
    click(window, row(7));
    check(
        gallery
            .get_shell_probe()
            .expanded_items
            .iter()
            .any(|index| index == 7)
            && gallery.get_page() == GalleryPage::Toggles,
        "the Overlays group expands without replacing the current page",
    )?;
    click(window, row(8));
    check(
        gallery.get_page() == GalleryPage::Overlays && gallery.get_shell_probe().selected == 8,
        "a child item opens its Gallery page",
    )?;
    click(window, row(7));

    window.dispatch_event(WindowEvent::Resized {
        size: LogicalSize::new(380., 700.),
    });
    window.take_snapshot()?;
    let narrow = gallery.get_shell_probe();
    check(
        narrow.mode == NavigationDisplayMode::Minimal
            && !narrow.pane_open
            && narrow.pane_width == 0.,
        "the Gallery NavigationView uses a drawer at narrow widths",
    )?;
    click(window, narrow.toggle);
    window.take_snapshot()?;
    check(
        gallery.get_shell_probe().pane_open,
        "the narrow Gallery drawer opens",
    )?;
    key(window, Key::Escape);
    check(
        !gallery.get_shell_probe().pane_open && gallery.get_page() == GalleryPage::Overlays,
        "Escape dismisses the Gallery drawer and preserves its current page",
    )?;

    window.dispatch_event(WindowEvent::Resized { size: original });
    gallery.set_page(GalleryPage::Button);
    window.take_snapshot()?;
    check(
        gallery.get_shell_probe().selected == 0,
        "programmatic Gallery page changes synchronize the selection",
    )
}

pub(super) fn verify_startup_scale(gallery: &GalleryWindow) -> SmokeResult {
    let window = gallery.window();
    gallery.set_page(GalleryPage::NavigationView);
    window.take_snapshot()?;
    let nav = gallery.get_navigation_probe();
    check(
        nav.mode == NavigationDisplayMode::Compact
            && is_width(nav.pane_width, 60.)
            && is_width(nav.item_step, 50.),
        "macOS scales the NavigationView rail and menu rows to 125%",
    )?;
    click(window, nav.toggle);
    window.take_snapshot()?;
    let nav = gallery.get_navigation_probe();
    check(
        nav.drawer_open && is_width(nav.drawer.width, 400.),
        "the macOS NavigationView drawer scales with the controls",
    )?;
    key(window, Key::Escape);
    check(
        !gallery.get_navigation_probe().drawer_open,
        "the scaled drawer dismisses",
    )
}

pub(super) fn verify(gallery: &GalleryWindow) -> SmokeResult {
    let window = gallery.window();
    let original = window.size().to_logical(window.scale_factor());
    gallery.set_page(GalleryPage::NavigationView);
    gallery.invoke_select_theme(FluentColorScheme::Light);
    window.take_snapshot()?;

    let nav = gallery.get_navigation_probe();
    check(
        nav.mode == NavigationDisplayMode::Compact
            && is_width(nav.pane_width, 48.)
            && !nav.pane_open
            && nav.selected == 0
            && nav.route == 0,
        "NavigationView starts with a compact rail and the Home page",
    )?;
    let margin = original.width - nav.content.width - nav.pane_width;
    let resize = |width: f32| {
        window.dispatch_event(WindowEvent::Resized {
            size: LogicalSize::new(width + margin, original.height),
        });
        gallery.get_navigation_probe()
    };
    check(
        resize(640.).mode == NavigationDisplayMode::Minimal
            && resize(641.).mode == NavigationDisplayMode::Compact
            && resize(1007.).mode == NavigationDisplayMode::Compact,
        "NavigationView switches between Minimal and Compact at 641 logical px",
    )?;
    let nav = resize(1008.);
    check(
        nav.mode == NavigationDisplayMode::Expanded
            && nav.pane_open
            && is_width(nav.pane_width, 320.),
        "NavigationView expands inline at 1008 logical px",
    )?;
    let colors = gallery.global::<FluentTheme>();
    let pill = LogicalPosition::new(nav.items.x + 5.5, nav.items.y + nav.item_step / 2.);
    let glyph = LogicalPosition::new(nav.toggle.x, nav.toggle.y - 0.5);
    let glyph_background = LogicalPosition::new(nav.toggle.x + 9., nav.toggle.y - 0.5);
    let light = read_rgb(window, glyph)?;
    check(
        light != read_rgb(window, glyph_background)?,
        "the licensed Fluent navigation SVG is painted in the toggle",
    )?;
    let accent = colors.get_accent_fill();
    check(
        read_rgb(window, pill)? == [accent.red(), accent.green(), accent.blue()],
        "the light NavigationView renders its accent selection indicator",
    )?;
    gallery.invoke_select_theme(FluentColorScheme::Dark);
    window.take_snapshot()?;
    let accent = colors.get_accent_fill();
    check(
        read_rgb(window, pill)? == [accent.red(), accent.green(), accent.blue()]
            && read_rgb(window, glyph)? != light,
        "the dark NavigationView renders its theme accent and glyph tint",
    )?;
    gallery.invoke_select_theme(FluentColorScheme::Light);

    let home = content_pixels(gallery, &nav)?;
    click(window, row(&nav, 1));
    check_route(gallery, 1)?;
    check(
        home != content_pixels(gallery, &nav)?,
        "Frame paints the Library page in place of Home",
    )?;
    click(window, row(&nav, 2));
    check_route(gallery, 1)?;
    key(window, Key::DownArrow);
    window.take_snapshot()?;
    check(
        gallery.get_navigation_probe().focused == 3 && gallery.get_navigation_probe().route == 1,
        "Down skips the disabled item and moves focus before selecting",
    )?;
    key(window, Key::Return);
    check_route(gallery, 3)?;
    click(window, gallery.get_navigation_probe().back);
    check_route(gallery, 1)?;
    check(
        gallery.get_navigation_probe().can_go_forward,
        "Frame exposes Forward after Back",
    )?;
    click(window, gallery.get_navigation_probe().forward);
    check_route(gallery, 3)?;
    click(window, gallery.get_navigation_probe().back);
    click(
        window,
        LogicalPosition::new(nav.footer.x + 24., nav.footer.y + nav.item_step / 2.),
    );
    check_route(gallery, 9)?;
    check(
        !gallery.get_navigation_probe().can_go_forward,
        "a new Frame navigation discards its forward branch",
    )?;

    click(window, nav.toggle);
    let closed = gallery.get_navigation_probe();
    check(
        !closed.pane_open && is_width(closed.pane_width, 48.) && closed.content.x < nav.content.x,
        "collapsing the inline pane expands the page and leaves an icon rail",
    )?;
    resize(640.);
    let restored = resize(1008.);
    check(
        !restored.pane_open && is_width(restored.pane_width, 48.),
        "an explicitly closed pane stays closed after resizing back to Expanded",
    )?;
    click(window, restored.toggle);
    check(
        gallery.get_navigation_probe().pane_open,
        "the inline pane reopens",
    )?;

    window.dispatch_event(WindowEvent::Resized { size: original });
    let nav = gallery.get_navigation_probe();
    click(window, nav.toggle);
    window.take_snapshot()?;
    let open = gallery.get_navigation_probe();
    check(
        open.drawer_open
            && open.pane_open
            && is_width(open.drawer.width, 320.)
            && open.content == nav.content,
        "Compact opens a shared overlay while keeping the page in place",
    )?;
    key(window, Key::Escape);
    window.take_snapshot()?;
    check(
        !gallery.get_navigation_probe().drawer_open
            && gallery.get_navigation_probe().toggle_focused,
        "Escape closes the navigation overlay and restores trigger focus",
    )?;
    click(window, nav.toggle);
    window.take_snapshot()?;
    click(window, row(&nav, 0));
    window.take_snapshot()?;
    check_route(gallery, 0)?;
    check(
        !gallery.get_navigation_probe().drawer_open,
        "invoking a leaf closes the Compact pane",
    )
}

pub(super) fn verify_styles(gallery: &GalleryWindow) -> SmokeResult {
    let window = gallery.window();
    let original = window.size().to_logical(window.scale_factor());
    gallery.set_navigation_pane_display_mode(NavigationPaneDisplayMode::Top);
    window.take_snapshot()?;
    let top = gallery.get_navigation_probe();
    check(
        top.mode == NavigationDisplayMode::Top
            && is_width(top.pane_width, 0.)
            && !top.pane_open
            && top.overflow_count == 0
            && top.top_items.row_count() == 7,
        "Top lays out the measured main items in one row with a separate footer",
    )?;
    let mail = top
        .top_items
        .iter()
        .find(|slot| slot.index == 3)
        .ok_or("Mail should be on the primary bar")?;
    click(
        window,
        LogicalPosition::new(top.top_origin.x + mail.x + mail.width / 2., top.overflow.y),
    );
    check_route(gallery, 3)?;
    check(
        gallery.get_navigation_probe().transition == NavigationTransition::FromRight,
        "Top recommends a horizontal transition from the right",
    )?;
    click(window, gallery.get_navigation_probe().back);
    check_route(gallery, 0)?;
    check(
        gallery.get_navigation_probe().transition == NavigationTransition::FromRight,
        "Frame retains the originating horizontal transition when going back",
    )?;

    gallery.set_navigation_pane_visible(false);
    window.take_snapshot()?;
    check(
        !gallery.get_navigation_probe().pane_visible,
        "Top supports hiding its pane without changing the route",
    )?;
    gallery.set_navigation_pane_visible(true);
    window.dispatch_event(WindowEvent::Resized {
        size: LogicalSize::new(320., original.height),
    });
    window.take_snapshot()?;
    let narrow = gallery.get_navigation_probe();
    check(
        narrow.overflow_count > 0 && narrow.top_items.iter().any(|slot| slot.index == 0),
        "Top reserves More and the selected route in a narrow window",
    )?;
    let (destination_row, destination) = narrow
        .overflow_items
        .iter()
        .enumerate()
        .find(|(_, index)| *index == 1 || *index == 3)
        .ok_or("narrow Top should overflow Library or Mail")?;
    let offset: f32 = narrow
        .overflow_items
        .iter()
        .take(destination_row)
        .map(|index| if index == 5 { 8. } else { narrow.item_step })
        .sum();
    click(window, narrow.overflow);
    window.take_snapshot()?;
    let popup = gallery.get_navigation_probe();
    check(
        popup.overflow_open && popup.overflow_bounds.y >= narrow.overflow.y,
        "More opens a flyout below the Top bar",
    )?;
    let sample = || -> Result<Vec<[u8; 3]>, Box<dyn Error>> {
        [8., 12., 17., 23.]
            .into_iter()
            .map(|inset| {
                read_rgb(
                    window,
                    LogicalPosition::new(
                        popup.overflow_bounds.x + popup.overflow_bounds.width - 14.,
                        popup.overflow_bounds.y + inset,
                    ),
                )
            })
            .collect()
    };
    let fallback = sample()?;
    let theme = gallery.global::<FluentTheme>();
    key(window, Key::Escape);
    theme.set_reduce_transparency(false);
    window.take_snapshot()?;
    click(window, narrow.overflow);
    window.take_snapshot()?;
    check(
        gallery.get_navigation_probe().overflow_acrylic,
        "Top overflow successfully captures its backdrop before opening",
    )?;
    check(
        sample()? != fallback,
        "Skia renders the Top overflow Acrylic and its solid fallback differently",
    )?;
    click(
        window,
        LogicalPosition::new(
            popup.overflow_bounds.x + 23.,
            popup.overflow_bounds.y + 3. + offset + narrow.item_step / 2.,
        ),
    );
    check_route(gallery, destination)?;
    let promoted = gallery.get_navigation_probe();
    theme.set_reduce_transparency(true);
    check(
        !promoted.overflow_open
            && !promoted.overflow_acrylic
            && promoted
                .top_items
                .iter()
                .any(|slot| slot.index == destination),
        "invoking an overflow item promotes it and releases the Acrylic capture",
    )?;
    check(
        promoted.transition == NavigationTransition::FromRight,
        "overflow navigation enters from the right",
    )?;

    window.dispatch_event(WindowEvent::Resized { size: original });
    gallery.set_navigation_pane_display_mode(NavigationPaneDisplayMode::Left);
    window.take_snapshot()?;
    let left = gallery.get_navigation_probe();
    let branch_y = left.items.y + 5. * left.item_step + 8. + left.item_step / 2.;
    click(window, LogicalPosition::new(left.items.x + 23., branch_y));
    window.take_snapshot()?;
    let expanded = gallery.get_navigation_probe();
    check(
        expanded.expanded_items.iter().any(|index| index == 6) && expanded.route == destination,
        "an expanded Left branch reveals children without navigating to the group heading",
    )?;
    click(
        window,
        LogicalPosition::new(left.items.x + 55., branch_y + left.item_step),
    );
    check_route(gallery, 7)?;

    click(window, left.toggle);
    window.take_snapshot()?;
    let rail = gallery.get_navigation_probe();
    let branch_y = rail.items.y + 4. * rail.item_step + 8. + rail.item_step / 2.;
    click(window, LogicalPosition::new(rail.items.x + 23., branch_y));
    window.take_snapshot()?;
    let children = gallery.get_navigation_probe();
    check(
        children.child_flyout_open && children.child_flyout_bounds.x > rail.items.x,
        "a closed icon rail opens the child destinations in a flyout",
    )?;
    let bounds = children.child_flyout_bounds;
    click(
        window,
        LogicalPosition::new(bounds.x + 23., bounds.y + 2. + rail.item_step * 1.5),
    );
    check_route(gallery, 8)?;
    check(
        !gallery.get_navigation_probe().child_flyout_open,
        "invoking a child closes its flyout",
    )?;
    click(window, rail.toggle);
    gallery.set_navigation_pane_display_mode(NavigationPaneDisplayMode::LeftMinimal);
    check(
        gallery.get_navigation_probe().mode == NavigationDisplayMode::Minimal,
        "LeftMinimal overrides a wide viewport",
    )?;
    gallery.set_navigation_pane_display_mode(NavigationPaneDisplayMode::Auto);
    gallery.invoke_navigate_example(0);
    Ok(())
}

pub(super) fn verify_custom_pane(gallery: &GalleryWindow) -> SmokeResult {
    let window = gallery.window();
    gallery.set_navigation_pane_display_mode(NavigationPaneDisplayMode::Left);
    window.take_snapshot()?;
    let before = gallery.get_navigation_probe();
    gallery.set_navigation_custom_pane(true);
    window.take_snapshot()?;
    let custom = gallery.get_navigation_probe();
    check(
        custom.pane_custom_height > 0. && custom.items.y > before.items.y,
        "PaneCustomContent reserves its measured height above the Left menu",
    )?;
    click(
        window,
        LogicalPosition::new(
            custom.items.x + 20.,
            custom.items.y - custom.pane_custom_height / 2.,
        ),
    );
    check(
        gallery.get_navigation_probe().slot_activations == 1,
        "custom Pane content remains interactive",
    )?;
    gallery.set_navigation_pane_display_mode(NavigationPaneDisplayMode::Top);
    window.take_snapshot()?;
    let top = gallery.get_navigation_probe();
    check(
        top.top_custom_width > 0.
            && top.slot_activations == 1
            && top.top_origin.x > before.content.x - before.pane_width,
        "Top reserves custom slots and retains the host-owned state",
    )?;
    gallery.set_navigation_custom_pane(false);
    gallery.set_navigation_pane_display_mode(NavigationPaneDisplayMode::Auto);
    window.take_snapshot()?;
    check(
        is_width(gallery.get_navigation_probe().pane_custom_height, 0.),
        "empty custom slots return their space to the menu",
    )
}

pub(super) fn verify_compact(gallery: &GalleryWindow) -> SmokeResult {
    gallery.set_page(GalleryPage::NavigationView);
    let window = gallery.window();
    gallery.invoke_select_theme(FluentColorScheme::HighContrast);
    window.take_snapshot()?;
    let nav = gallery.get_navigation_probe();
    check(
        nav.mode == NavigationDisplayMode::Minimal
            && is_width(nav.pane_width, 0.)
            && nav.content.width <= 320.
            && nav.toggle.y < nav.content.y,
        "a 320px window places the toggle over a full-width content page",
    )?;
    click(window, nav.toggle);
    window.take_snapshot()?;
    let open = gallery.get_navigation_probe();
    check(
        open.drawer_open
            && is_width(open.drawer.width, nav.content.width)
            && open.drawer.height > nav.item_step,
        "the Minimal overlay fits the narrow NavigationView",
    )?;
    click(window, LogicalPosition::new(2., 2.));
    window.take_snapshot()?;
    check(
        !gallery.get_navigation_probe().drawer_open
            && gallery.get_navigation_probe().toggle_focused,
        "light dismissal closes the Minimal overlay and restores focus",
    )?;
    gallery.set_navigation_pane_display_mode(NavigationPaneDisplayMode::LeftCompact);
    check(
        gallery.get_navigation_probe().mode == NavigationDisplayMode::Compact,
        "LeftCompact overrides Auto in a narrow window",
    )?;
    gallery.set_navigation_pane_display_mode(NavigationPaneDisplayMode::Auto);
    Ok(())
}

pub(super) fn verify_motion(gallery: &GalleryWindow, finish: impl FnOnce(SmokeResult) + 'static) {
    gallery.set_page(GalleryPage::NavigationView);
    gallery.invoke_select_theme(FluentColorScheme::Light);
    gallery.global::<FluentTheme>().set_reduced_motion(false);
    let window = gallery.window();
    let nav = gallery.get_navigation_probe();
    click(window, nav.toggle);
    let weak = gallery.as_weak();
    slint::Timer::single_shot(Duration::from_millis(85), move || {
        let Some(gallery) = weak.upgrade() else {
            finish(Err("Gallery closed during NavigationView motion".into()));
            return;
        };
        let open = gallery.get_navigation_probe();
        if let Err(error) = check(
            open.drawer_open && open.drawer_progress > 0. && open.drawer_progress < 1.,
            "the navigation overlay renders an intermediate reveal frame",
        ) {
            finish(Err(error));
            return;
        }
        let window = gallery.window();
        key(window, Key::Escape);
        if let Err(error) = check(
            gallery.get_navigation_probe().drawer_open && !gallery.get_navigation_probe().pane_open,
            "the navigation overlay remains present until its exit completes",
        ) {
            finish(Err(error));
            return;
        }
        slint::Timer::single_shot(Duration::from_millis(185), move || {
            let Some(gallery) = weak.upgrade() else {
                finish(Err("Gallery closed before Frame motion verification".into()));
                return;
            };
            let window = gallery.window();
            let nav = gallery.get_navigation_probe();
            if let Err(error) = check(
                !nav.drawer_open && nav.toggle_focused,
                "the animated overlay restores focus after its exit",
            ) {
                finish(Err(error));
                return;
            }
            let original = window.size().to_logical(window.scale_factor());
            window.dispatch_event(WindowEvent::Resized {
                size: LogicalSize::new(1100., original.height),
            });
            let wide = gallery.get_navigation_probe();
            click(window, row(&wide, 3));
            click(window, wide.toggle);
            slint::Timer::single_shot(Duration::from_millis(85), move || {
                let partial = weak
                    .upgrade()
                    .ok_or_else(|| "Gallery closed during inline animation".into())
                    .and_then(|gallery| {
                        let nav = gallery.get_navigation_probe();
                        if nav.pane_width > 48.
                            && nav.pane_width < 320.
                            && nav.transitioning
                            && nav.page_progress > 0
                            && nav.selected == 3
                            && nav.route == 3
                            && nav.indicator_height > 16.
                        {
                            Ok(())
                        } else {
                            Err(format!(
                                "expected intermediate inline pane, Frame and indicator: {nav:?}"
                            )
                            .into())
                        }
                    });
                if let Err(error) = partial {
                    finish(Err(error));
                    return;
                }
                slint::Timer::single_shot(Duration::from_millis(620), move || {
                    let Some(gallery) = weak.upgrade() else {
                        finish(Err("Gallery closed before navigation settled".into()));
                        return;
                    };
                    let nav = gallery.get_navigation_probe();
                    if let Err(error) = check(
                        is_width(nav.pane_width, 48.)
                            && !nav.transitioning
                            && is_width(nav.indicator_height, 16.),
                        "the pane, Frame and stretched indicator settle",
                    ) {
                        finish(Err(error));
                        return;
                    }
                    verify_top_motion(&gallery, original, finish);
                });
            });
        });
    });
}

fn verify_top_motion(
    gallery: &GalleryWindow,
    original: LogicalSize,
    finish: impl FnOnce(SmokeResult) + 'static,
) {
    gallery.set_navigation_pane_display_mode(NavigationPaneDisplayMode::Top);
    let nav = gallery.get_navigation_probe();
    let result = content_pixels(gallery, &nav).and_then(|pixels| {
        let nav = gallery.get_navigation_probe();
        let slot = nav
            .top_items
            .iter()
            .find(|slot| slot.index == 1)
            .ok_or("moving the top indicator needs Library in the primary bar")?;
        click(
            gallery.window(),
            LogicalPosition::new(nav.top_origin.x + slot.x + slot.width / 2., nav.overflow.y),
        );
        Ok(pixels)
    });
    let initial = match result {
        Ok(initial) => initial,
        Err(error) => return finish(Err(error)),
    };
    let weak = gallery.as_weak();
    slint::Timer::single_shot(Duration::from_millis(92), move || {
        let middle = weak
            .upgrade()
            .ok_or_else(|| "Gallery closed during horizontal navigation".into())
            .and_then(|gallery| {
                let nav = gallery.get_navigation_probe();
                check(
                    nav.route == 1
                        && nav.transition == NavigationTransition::FromLeft
                        && nav.transitioning
                        && nav.indicator_height > 16.,
                    "Top renders intermediate horizontal movement and indicator stretch",
                )?;
                check(
                    initial != content_pixels(&gallery, &nav)?,
                    "Skia shows the outgoing Top page in motion",
                )
            });
        if let Err(error) = middle {
            return finish(Err(error));
        }
        slint::Timer::single_shot(Duration::from_millis(620), move || {
            let Some(gallery) = weak.upgrade() else {
                return finish(Err(
                    "Gallery closed before horizontal navigation settled".into()
                ));
            };
            let nav = gallery.get_navigation_probe();
            if let Err(error) = check(
                nav.route == 1 && !nav.transitioning && is_width(nav.indicator_height, 16.),
                "the horizontal transition and stretched indicator settle",
            ) {
                return finish(Err(error));
            }
            verify_popup_profile(&gallery, original, finish);
        });
    });
}

fn verify_popup_profile(
    gallery: &GalleryWindow,
    original: LogicalSize,
    finish: impl FnOnce(SmokeResult) + 'static,
) {
    let window = gallery.window();
    window.dispatch_event(WindowEvent::Resized { size: original });
    let nav = gallery.get_navigation_probe();
    if let Err(error) = check(
        nav.overflow_count > 0,
        "the narrow Top fixture has overflow",
    ) {
        return finish(Err(error));
    }
    // Test payload only. The Gallery does not assert that these are Windows theme timings.
    let motion = gallery.global::<FluentPopupMotion>();
    motion.set_profile(PopupMotionProfile {
        enter_translation: PopupMotionTrack {
            duration: 110,
            ..Default::default()
        },
        exit_opacity: PopupMotionTrack {
            delay: 20,
            duration: 110,
            ..Default::default()
        },
        exit_opacity_from: 1.,
        exit_opacity_to: 0.,
        ..Default::default()
    });
    click(window, nav.overflow);
    key(window, Key::Escape);
    if let Err(error) = check(
        gallery.get_navigation_probe().overflow_open,
        "a supplied exit track keeps the flyout open until it finishes",
    ) {
        return finish(Err(error));
    }
    let weak = gallery.as_weak();
    slint::Timer::single_shot(Duration::from_millis(190), move || {
        let result = weak
            .upgrade()
            .ok_or_else(|| "Gallery closed during popup profile verification".into())
            .and_then(|gallery| {
                gallery
                    .global::<FluentPopupMotion>()
                    .set_profile(PopupMotionProfile::default());
                check(
                    !gallery.get_navigation_probe().overflow_open,
                    "the supplied exit track completes and dismisses the flyout",
                )?;
                gallery.set_navigation_pane_display_mode(NavigationPaneDisplayMode::Auto);
                Ok(())
            });
        match (result, weak.upgrade()) {
            (Ok(()), Some(gallery)) => verify_shell_items(&gallery, finish),
            (Err(error), _) => finish(Err(error)),
            _ => finish(Err("Gallery closed before shell item verification".into())),
        }
    });
}

fn verify_shell_items(gallery: &GalleryWindow, finish: impl FnOnce(SmokeResult) + 'static) {
    let window = gallery.window();
    let original = window.size().to_logical(window.scale_factor());
    // Make the physical render target wide enough for both the shell and its nested example.
    window.set_size(LogicalSize::new(980., 700.));
    gallery.set_touch_targets(false);
    gallery.set_shell_pane_visible(true);
    gallery.set_page(GalleryPage::NavigationView);
    gallery.set_navigation_pane_display_mode(NavigationPaneDisplayMode::Left);
    gallery.set_navigation_custom_pane(true);
    let nav = gallery.get_navigation_probe();
    if !nav.pane_open {
        click(window, nav.toggle);
    }
    let weak = gallery.as_weak();
    slint::Timer::single_shot(Duration::from_millis(250), move || {
        let Some(gallery) = weak.upgrade() else {
            return finish(Err(
                "Gallery closed before shell badges were rendered".into()
            ));
        };
        let window = gallery.window();
        let shell = gallery.get_shell_probe();
        let badge = LogicalPosition::new(
            shell.items.x + shell.pane_width - 19.,
            shell.items.y + shell.item_step * 9.5,
        );
        let nav = gallery.get_navigation_probe();
        let mail_badge = LogicalPosition::new(
            nav.items.x + nav.pane_width - 19.,
            nav.items.y + nav.item_step * 3.5,
        );
        let tested = (|| {
            let shell_before = region(window, badge, 16., 8.)?;
            let mail_before = region(window, mail_badge, 16., 8.)?;
            let accent = gallery.global::<FluentTheme>().get_accent_fill();
            let has_badge = |pixels: &[slint::Rgba8Pixel]| {
                pixels
                    .iter()
                    .filter(|p| (p.r, p.g, p.b) == (accent.red(), accent.green(), accent.blue()))
                    .count()
                    > 5
            };
            check(
                has_badge(&shell_before) && has_badge(&mail_before),
                "InfoBadge renders on NavigationView in the shell and Mail in the example",
            )?;
            click(
                window,
                LogicalPosition::new(nav.items.x + 20., nav.items.y - nav.pane_custom_height / 2.),
            );
            window.dispatch_event(WindowEvent::PointerMoved {
                position: LogicalPosition::new(1., 1.),
            });
            check(
                shell_before != region(window, badge, 16., 8.)?
                    && mail_before != region(window, mail_badge, 16., 8.)?,
                "adding a notification updates the rendered badge number on both NavigationViews",
            )?;
            Ok::<_, Box<dyn Error>>(())
        })();
        if let Err(error) = tested {
            return finish(Err(error));
        }
        let chevron = LogicalPosition::new(
            shell.items.x + shell.pane_width - 24.,
            shell.items.y + shell.item_step * 7.5,
        );
        let label = LogicalPosition::new(shell.items.x + 28., chevron.y);
        window.dispatch_event(WindowEvent::PointerMoved { position: label });
        slint::Timer::single_shot(Duration::from_millis(220), move || {
            let Some(gallery) = weak.upgrade() else {
                return finish(Err("Gallery closed before its chevron animation".into()));
            };
            let window = gallery.window();
            let start = match icon_ink(window, chevron) {
                Ok(start) => start,
                Err(error) => return finish(Err(error)),
            };
            click(window, label);
            slint::Timer::single_shot(Duration::from_millis(75), move || {
                let Some(gallery) = weak.upgrade() else {
                    return finish(Err("Gallery closed during the chevron animation".into()));
                };
                let middle = match icon_ink(gallery.window(), chevron) {
                    Ok(middle) => middle,
                    Err(error) => return finish(Err(error)),
                };
                slint::Timer::single_shot(Duration::from_millis(575), move || {
                    let result = weak.upgrade().ok_or_else(|| "Gallery closed before the chevron settled".into()).and_then(|gallery| {
                        let window = gallery.window();
                        let end = icon_ink(window, chevron)?;
                        check(
                            start.iter().any(|ink| *ink) && end.iter().any(|ink| *ink)
                                && start != end && start != middle && middle != end
                                && gallery.get_shell_probe().expanded_items.iter().any(|index| index == 7),
                            "the same Gallery navigation chevron renders closed, intermediate and expanded poses",
                        )?;
                        Ok(())
                    });
                    match (result, weak.upgrade()) {
                        (Ok(()), Some(gallery)) => {
                            verify_settings_animation(&gallery, original, finish)
                        }
                        (Err(error), _) => finish(Err(error)),
                        _ => finish(Err("Gallery closed before the Settings animation".into())),
                    }
                });
            });
        });
    });
}

fn verify_settings_animation(
    gallery: &GalleryWindow,
    original: LogicalSize,
    finish: impl FnOnce(SmokeResult) + 'static,
) {
    let window = gallery.window();
    let shell = gallery.get_shell_probe();
    let position =
        LogicalPosition::new(shell.footer.x + 24., shell.footer.y + shell.item_step / 2.);
    let idle = match icon_ink(window, position) {
        Ok(ink) => ink,
        Err(error) => return finish(Err(error)),
    };
    window.dispatch_event(WindowEvent::PointerMoved { position });
    let button = PointerEventButton::Left;
    window.dispatch_event(WindowEvent::PointerPressed { position, button });
    let weak = gallery.as_weak();
    slint::Timer::single_shot(Duration::from_millis(360), move || {
        let Some(gallery) = weak.upgrade() else {
            return finish(Err("Gallery closed while Settings was pressed".into()));
        };
        let window = gallery.window();
        let pressed = match icon_ink(window, position) {
            Ok(ink) => ink,
            Err(error) => return finish(Err(error)),
        };
        window.dispatch_event(WindowEvent::PointerReleased { position, button });
        slint::Timer::single_shot(Duration::from_millis(110), move || {
            let Some(gallery) = weak.upgrade() else {
                return finish(Err("Gallery closed during the Settings spin".into()));
            };
            let spinning = match icon_ink(gallery.window(), position) {
                Ok(ink) => ink,
                Err(error) => return finish(Err(error)),
            };
            slint::Timer::single_shot(Duration::from_millis(600), move || {
                let result = weak.upgrade().ok_or_else(|| "Gallery closed before Settings settled".into()).and_then(|gallery| {
                    let settled = icon_ink(gallery.window(), position)?;
                    check(
                        idle.iter().any(|ink| *ink) && idle != pressed && spinning != idle
                            && spinning != pressed && settled != spinning
                            && gallery.get_page() == GalleryPage::Foundations,
                        "the Settings gear renders press and release poses while its destination changes",
                    )?;
                    gallery.window().set_size(original);
                    Ok(())
                });
                finish(result);
            });
        });
    });
}

fn icon_ink(window: &slint::Window, center: LogicalPosition) -> Result<Vec<bool>, Box<dyn Error>> {
    Ok(region(window, center, 12., 11.)?
        .iter()
        .map(|p| p.r < 100 && p.g < 100 && p.b < 100)
        .collect())
}

fn region(
    window: &slint::Window,
    center: LogicalPosition,
    half_width: f32,
    half_height: f32,
) -> Result<Vec<slint::Rgba8Pixel>, Box<dyn Error>> {
    let frame = window.take_snapshot()?;
    let scale = window.scale_factor();
    let first =
        LogicalPosition::new(center.x - half_width, center.y - half_height).to_physical(scale);
    let end =
        LogicalPosition::new(center.x + half_width, center.y + half_height).to_physical(scale);
    let width = frame.width() as usize;
    let left = (first.x as usize).min(width);
    let right = (end.x as usize).min(width);
    let mut pixels = Vec::new();
    for y in first.y as usize..(end.y as usize).min(frame.height() as usize) {
        pixels.extend_from_slice(&frame.as_slice()[y * width + left..y * width + right]);
    }
    Ok(pixels)
}

fn row(nav: &NavigationProbe, index: i32) -> LogicalPosition {
    LogicalPosition::new(
        nav.items.x + 24.,
        nav.items.y + nav.item_step * (index as f32 + 0.5),
    )
}

fn content_pixels(
    gallery: &GalleryWindow,
    nav: &NavigationProbe,
) -> Result<Vec<slint::Rgba8Pixel>, Box<dyn Error>> {
    let window = gallery.window();
    let frame = window.take_snapshot()?;
    let scale = window.scale_factor();
    let first = LogicalPosition::new(nav.content.x + 20., nav.content.y + 65.).to_physical(scale);
    let end = LogicalPosition::new(nav.content.x + 260., nav.content.y + 118.).to_physical(scale);
    let width = frame.width() as usize;
    let left = first.x as usize;
    let right = (end.x as usize).min(width);
    let mut pixels = Vec::new();
    for y in first.y as usize..(end.y as usize).min(frame.height() as usize) {
        pixels.extend_from_slice(&frame.as_slice()[y * width + left..y * width + right]);
    }
    Ok(pixels)
}

fn check_route(gallery: &GalleryWindow, expected: i32) -> SmokeResult {
    gallery.window().take_snapshot()?;
    let nav = gallery.get_navigation_probe();
    if nav.selected != expected || nav.route != expected {
        return Err(format!("expected route and selection {expected}, got {nav:?}").into());
    }
    Ok(())
}

fn is_width(actual: f32, expected: f32) -> bool {
    (actual - expected).abs() <= 0.5
}
