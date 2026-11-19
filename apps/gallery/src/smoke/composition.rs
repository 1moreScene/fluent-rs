use slint::{
    ComponentHandle, LogicalPosition, LogicalSize, Model, ModelRc, VecModel, Window,
    platform::{Key, WindowEvent},
};

use crate::{
    FluentColorScheme, GalleryPage, GalleryWindow, ItemData, ListSelectionMode, list_demo,
};

use super::{check, click, read_rgb};

type SmokeResult = Result<(), Box<dyn std::error::Error>>;

pub(super) fn verify(gallery: &GalleryWindow) -> SmokeResult {
    gallery.set_page(GalleryPage::Composition);
    gallery.invoke_select_theme(FluentColorScheme::Light);
    let (model, reads) = list_demo::items();
    gallery.set_composition_items(model);

    let window = gallery.window();
    window.take_snapshot()?;
    check(reads.get() < 1000, "100,000 records use a virtual repeater")?;
    let origin = gallery.get_composition_list_origin();
    let first = LogicalPosition::new(origin.x + 10., origin.y + 10.);
    click(window, first);
    check(
        selected(gallery, &[0]),
        "pointer selects a visible list row",
    )?;
    let disabled =
        LogicalPosition::new(first.x, first.y + 2. * gallery.get_composition_row_height());
    click(window, disabled);
    check(
        selected(gallery, &[0]),
        "disabled rows ignore pointer activation",
    )?;
    verify_selected_fill(gallery, first)?;
    key(window, Key::DownArrow);
    key(window, Key::DownArrow);
    check(
        gallery.get_composition_focused_row() == 3 && selected(gallery, &[3]),
        "single selection follows keyboard focus and skips a disabled row",
    )?;

    window.dispatch_event(WindowEvent::PointerScrolled {
        position: first,
        delta_x: 0.,
        delta_y: -160.,
    });
    check(
        gallery.get_composition_scroll_offset() > 0. && selected(gallery, &[3]),
        "scrolling retains selection",
    )?;
    let rail = gallery.get_composition_list_scroll_rail();
    let offset = gallery.get_composition_scroll_offset();
    let before_reads = reads.get();
    window.dispatch_event(WindowEvent::PointerMoved { position: rail });
    for _ in 0..12 {
        window.dispatch_event(WindowEvent::PointerScrolled {
            position: rail,
            delta_x: 0.,
            delta_y: -40.,
        });
    }
    window.take_snapshot()?;
    check(
        gallery.get_composition_scroll_offset() > offset + 100.
            && selected(gallery, &[3])
            && reads.get() - before_reads < 250,
        "a wheel burst over the scrollbar reaches the virtual list without rebuilding it",
    )?;
    println!(
        "ListBox scrollbar wheel burst read {} rows",
        reads.get() - before_reads
    );
    key(window, Key::End);
    window.take_snapshot()?;
    check(
        gallery.get_composition_focused_row() == 99_999
            && selected(gallery, &[99_999])
            && gallery.get_composition_scroll_offset() > 2_000_000.
            && reads.get() < 1500,
        "End scrolls to the last of 100,000 records without materializing the list",
    )?;
    key(window, Key::Home);
    check(
        gallery.get_composition_scroll_offset() == 0.,
        "Home brings the first record back into view",
    )?;

    gallery.set_composition_selection_mode(ListSelectionMode::Multiple);
    gallery.invoke_set_composition_selected_index(-1);
    key(window, Key::Space);
    key(window, Key::DownArrow);
    check(
        selected(gallery, &[0]),
        "arrow keys leave multiple selection intact",
    )?;
    key(window, Key::Space);
    check(selected(gallery, &[0, 1]), "Space adds a second record")?;
    key(window, Key::Space);
    check(selected(gallery, &[0]), "Space toggles the focused record")?;

    verify_popup(gallery)?;
    click(window, gallery.get_composition_modal_trigger());
    check(
        gallery.get_composition_modal_open(),
        "the modal panel opens",
    )?;
    let panel = gallery.get_composition_modal_bounds();
    check(
        first.x < panel.x && panel.width > 0. && panel.height > 0.,
        "the modal has a panel apart from the tested scrim point",
    )?;
    click(window, first);
    check(
        gallery.get_composition_modal_open() && selected(gallery, &[0]),
        "the modal scrim consumes a background click",
    )?;
    for _ in 0..5 {
        key(window, Key::Tab);
        check(
            gallery.get_composition_modal_open()
                && !gallery.get_composition_modal_trigger_focused()
                && !gallery.get_composition_popup_trigger_focused()
                && gallery.get_composition_focused_row() == -1,
            "Tab stays in the modal popup",
        )?;
    }
    key(window, Key::Escape);
    check(
        !gallery.get_composition_modal_open() && gallery.get_composition_modal_trigger_focused(),
        "Escape closes the modal and restores trigger focus",
    )?;

    click(window, gallery.get_composition_popup_trigger());
    gallery.set_page(GalleryPage::Button);
    window.take_snapshot()?;
    check(
        !gallery.get_composition_popup_open(),
        "leaving the page closes its popup",
    )?;
    gallery.set_page(GalleryPage::Composition);
    check(selected(gallery, &[0]), "navigation preserves selection")?;
    click(window, gallery.get_composition_modal_trigger());
    key(window, Key::Tab);
    key(window, Key::Space);
    check(
        !gallery.get_composition_modal_open()
            && gallery.get_composition_modal_trigger_focused()
            && selected(gallery, &[0]),
        "the modal's first action cancels and returns focus",
    )?;
    click(window, gallery.get_composition_modal_trigger());
    key(window, Key::Tab);
    key(window, Key::Tab);
    key(window, Key::Space);
    check(
        !gallery.get_composition_modal_open()
            && gallery.get_composition_popup_trigger_focused()
            && selected(gallery, &[]),
        "confirming clears selection and focuses the remaining action",
    )?;
    click(window, first);
    gallery.set_composition_empty(true);
    window.take_snapshot()?;
    check(
        selected(gallery, &[]) && gallery.get_composition_scroll_offset() == 0.,
        "replacing the list with an empty model clears selection and scrolling",
    )?;
    gallery.set_composition_empty(false);
    gallery.set_composition_selection_mode(ListSelectionMode::Single);
    println!(
        "Virtual list read {} rows during the wide scenario",
        reads.get()
    );
    verify_listbox_modes(gallery)
}

fn verify_listbox_modes(gallery: &GalleryWindow) -> SmokeResult {
    let window = gallery.window();
    let items = (0..8)
        .map(|row| ItemData {
            text: format!("Option {}", row + 1).into(),
            disabled: row == 2,
        })
        .collect::<Vec<_>>();
    gallery.set_composition_items(ModelRc::new(VecModel::from(items)));
    gallery.set_composition_selection_mode(ListSelectionMode::Extended);
    window.take_snapshot()?;
    let origin = gallery.get_composition_list_origin();
    let row = |index: f32| {
        LogicalPosition::new(
            origin.x + 10.,
            origin.y + gallery.get_composition_row_height() * index + 10.,
        )
    };
    click(window, row(0.));
    held(window, Key::Shift, || click(window, row(3.)));
    check(
        selected(gallery, &[0, 1, 3]),
        "Shift-click extends selection and skips a disabled row",
    )?;
    held(window, Key::Shift, || key(window, Key::DownArrow));
    check(
        selected(gallery, &[0, 1, 3, 4]),
        "Shift-Down extends from the original anchor",
    )?;
    held(window, Key::Shift, || key(window, Key::UpArrow));
    check(
        selected(gallery, &[0, 1, 3]),
        "reversing the range shrinks the selection",
    )?;
    held(window, Key::Control, || click(window, row(0.)));
    check(
        selected(gallery, &[1, 3]),
        "the selection modifier toggles one row",
    )?;

    let toggle_all = || {
        held(window, Key::Control, || {
            window.dispatch_event(WindowEvent::KeyPressed { text: "a".into() });
            window.dispatch_event(WindowEvent::KeyReleased { text: "a".into() });
        })
    };
    toggle_all();
    check(
        selected(gallery, &[0, 1, 3, 4, 5, 6, 7]),
        "select-all includes enabled rows",
    )?;
    toggle_all();
    check(
        selected(gallery, &[]),
        "select-all toggles an already complete selection",
    )?;

    gallery.set_composition_selection_mode(ListSelectionMode::Single);
    gallery.set_composition_follows_focus(false);
    let edits = gallery.get_composition_selection_edits();
    gallery.invoke_set_composition_selected_index(1);
    check(
        selected(gallery, &[1]) && gallery.get_composition_selection_edits() == edits,
        "programmatic selection does not report a user edit",
    )?;
    key(window, Key::PageDown);
    check(
        gallery.get_composition_focused_row() == 7 && selected(gallery, &[1]),
        "PageDown navigates without selecting when follow-focus is off",
    )?;
    key(window, Key::Return);
    check(selected(gallery, &[7]), "Enter commits the focused row")?;
    gallery.set_composition_list_enabled(false);
    click(window, row(0.));
    key(window, Key::Home);
    check(
        selected(gallery, &[7]),
        "a disabled ListBox preserves selection and rejects input",
    )?;
    gallery.set_composition_list_enabled(true);
    gallery.set_composition_follows_focus(true);
    let (model, _) = list_demo::items();
    gallery.set_composition_items(model);
    Ok(())
}

fn held(window: &Window, modifier: Key, action: impl FnOnce()) {
    window.dispatch_event(WindowEvent::KeyPressed {
        text: modifier.into(),
    });
    action();
    window.dispatch_event(WindowEvent::KeyReleased {
        text: modifier.into(),
    });
}

pub(super) fn verify_compact(gallery: &GalleryWindow, width: f32) -> SmokeResult {
    gallery.set_page(GalleryPage::Composition);
    check(
        gallery.get_composition_row_height() >= 44.,
        "compact list has touch-sized rows",
    )?;
    let window = gallery.window();
    for scheme in [FluentColorScheme::Light, FluentColorScheme::Dark] {
        gallery.invoke_select_theme(scheme);
        click(window, gallery.get_composition_popup_trigger());
        let panel = gallery.get_composition_popup_bounds();
        let trigger = gallery.get_composition_popup_trigger();
        check(
            gallery.get_composition_popup_open()
                && panel.x >= 0.
                && panel.x + panel.width <= width
                && panel.y + panel.height <= trigger.y,
            "compact anchored popup fits above its trigger",
        )?;
        let color = gallery
            .global::<crate::FluentTheme>()
            .get_acrylic_fallback();
        check(
            read_rgb(window, LogicalPosition::new(panel.x + 6., panel.y + 12.))?
                == [color.red(), color.green(), color.blue()],
            "Skia renders the panel in the selected theme",
        )?;
        key(window, Key::Escape);
        check(
            !gallery.get_composition_popup_open(),
            "compact popup dismisses",
        )?;
    }
    Ok(())
}

fn verify_selected_fill(gallery: &GalleryWindow, first: LogicalPosition) -> SmokeResult {
    let window = gallery.window();
    let theme = gallery.global::<crate::FluentTheme>();
    window.dispatch_event(WindowEvent::PointerMoved {
        position: LogicalPosition::new(1., 1.),
    });
    for scheme in [
        FluentColorScheme::Light,
        FluentColorScheme::Dark,
        FluentColorScheme::HighContrast,
    ] {
        gallery.invoke_select_theme(scheme);
        let selection = theme.get_list_box_selection();
        let surface = theme.get_list_box_background();
        let blend = |front: u8, behind: u8| -> u8 {
            let alpha = u16::from(selection.alpha());
            ((u16::from(front) * alpha + u16::from(behind) * (255 - alpha) + 127) / 255) as u8
        };
        let expected = [
            blend(selection.red(), surface.red()),
            blend(selection.green(), surface.green()),
            blend(selection.blue(), surface.blue()),
        ];
        let painted = read_rgb(window, first)?;
        check(
            painted
                .into_iter()
                .zip(expected)
                .all(|(actual, expected)| actual.abs_diff(expected) <= 1),
            "Skia paints the sourced ListBox selection over its theme background",
        )?;
    }
    gallery.invoke_select_theme(FluentColorScheme::Light);
    Ok(())
}

fn verify_popup(gallery: &GalleryWindow) -> SmokeResult {
    let window = gallery.window();
    let trigger = gallery.get_composition_popup_trigger();
    click(window, trigger);
    check(gallery.get_composition_popup_open(), "anchored popup opens")?;
    key(window, Key::Escape);
    check(
        !gallery.get_composition_popup_open() && gallery.get_composition_popup_trigger_focused(),
        "Escape closes the popup and restores trigger focus",
    )?;
    click(window, trigger);
    click(window, LogicalPosition::new(2., 2.));
    check(
        !gallery.get_composition_popup_open() && gallery.get_composition_popup_trigger_focused(),
        "a click outside dismisses the popup and restores trigger focus",
    )?;

    click(window, trigger);
    let size = window.size().to_logical(window.scale_factor());
    window.dispatch_event(WindowEvent::Resized {
        size: LogicalSize::new(size.width - 40., size.height),
    });
    window.take_snapshot()?;
    let dismissed =
        !gallery.get_composition_popup_open() && gallery.get_composition_popup_trigger_focused();
    window.dispatch_event(WindowEvent::Resized { size });
    check(dismissed, "resizing dismisses the popup and restores focus")
}

fn selected(gallery: &GalleryWindow, expected: &[i32]) -> bool {
    gallery
        .get_composition_selected()
        .iter()
        .eq(expected.iter().copied())
}

fn key(window: &Window, key: Key) {
    window.dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window.dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}
