use slint::{
    ComponentHandle, LogicalPosition, SharedString, Window,
    platform::{Key, PointerEventButton, WindowEvent},
};

use crate::{
    ButtonState, FluentColorScheme, FluentTheme, GalleryPage, GalleryWindow, OverlayArea, list_demo,
};

use super::{check, click, read_rgb};

type SmokeResult = Result<(), Box<dyn std::error::Error>>;

pub(super) fn verify_editable(gallery: &GalleryWindow) -> SmokeResult {
    gallery.set_page(GalleryPage::ComboBox);
    gallery.set_combo_enabled(true);
    gallery.invoke_select_theme(FluentColorScheme::Light);
    let window = gallery.window();
    window.take_snapshot()?;
    let bounds = gallery.get_editable_combo_bounds();
    let entry = LogicalPosition::new(bounds.x + 20., bounds.y + bounds.height / 2.);
    let arrow = LogicalPosition::new(bounds.x + bounds.width - 19., entry.y);
    let type_text = |value: &str| {
        with_modifier(window, Key::Control, || type_key(window, "a"));
        for character in value.chars() {
            type_key(window, character.to_string());
        }
    };

    check(
        gallery.get_editable_combo_value() == "10"
            && gallery.get_editable_combo_selected_index() == 2,
        "editable ComboBox shows the initially selected value",
    )?;
    click(window, entry);
    check(
        gallery.get_editable_combo_focused(),
        "click focuses the native editor",
    )?;
    type_text("2");
    check(
        gallery.get_editable_combo_text() == "20" && gallery.get_editable_combo_value() == "10",
        "typing completes an option without committing it",
    )?;
    key(window, Key::Backspace);
    check(
        gallery.get_editable_combo_text() == "2",
        "Backspace removes the suggested suffix without completing it again",
    )?;
    type_key(window, "4");
    check(
        gallery.get_editable_combo_text() == "24",
        "typing replaces the suggested suffix",
    )?;
    key(window, Key::Return);
    check(
        gallery.get_editable_combo_value() == "24"
            && gallery.get_editable_combo_selected_index() == 9,
        "Enter commits the completed option",
    )?;
    type_text("27");
    key(window, Key::Escape);
    check(
        gallery.get_editable_combo_text() == "24",
        "Escape restores the committed text",
    )?;
    type_text("27");
    key(window, Key::Tab);
    check(
        gallery.get_editable_combo_value() == "27"
            && gallery.get_editable_combo_selected_index() == -1
            && !gallery.get_editable_combo_focused(),
        "Tab commits valid custom text and moves focus",
    )?;
    click(window, entry);
    type_text("105");
    key(window, Key::Return);
    check(
        gallery.get_editable_combo_value() == "27" && gallery.get_editable_combo_text() == "27",
        "the demo rejects input outside its valid range",
    )?;
    let editor_corner = LogicalPosition::new(bounds.x + 0.75, bounds.y + bounds.height - 0.75);
    let rounded = read_rgb(window, editor_corner)?;
    click(window, arrow);
    let panel = gallery.get_editable_combo_popup_bounds();
    check(
        gallery.get_editable_combo_open()
            && gallery.get_editable_combo_focused()
            && (panel.y - (bounds.y + bounds.height)).abs() < 0.1,
        "the editable dropdown opens adjacent to the faceplate and preserves editor focus",
    )?;
    verify_editable_seam(gallery, &bounds, &panel)?;
    let flat = read_rgb(window, editor_corner)?;
    let has_accent = |[red, _, blue]: [u8; 3]| blue.saturating_sub(red) > 50;
    if has_accent(rounded) || !has_accent(flat) {
        return Err(format!("the joined editor needs a square focus underline, got {rounded:?} when closed and {flat:?} when open").into());
    }
    type_text("16");
    check(
        gallery.get_editable_combo_open() && gallery.get_editable_combo_highlighted_index() == 6,
        "text entry still previews matching rows while the dropdown is open",
    )?;
    key(window, Key::DownArrow);
    check(
        gallery.get_editable_combo_text() == "18" && gallery.get_editable_combo_value() == "27",
        "Down previews the next row in the editor",
    )?;
    key(window, Key::Escape);
    check(
        !gallery.get_editable_combo_open()
            && gallery.get_editable_combo_text() == "27"
            && gallery.get_editable_combo_focused(),
        "Escape closes the editable dropdown and restores the custom value and focus",
    )?;
    check(
        read_rgb(window, editor_corner)? == rounded,
        "closing restores the editor's rounded corners",
    )?;
    key(window, Key::F4);
    type_text("31");
    click(window, LogicalPosition::new(2., 2.));
    check(
        !gallery.get_editable_combo_open() && gallery.get_editable_combo_value() == "31",
        "a light dismiss submits the edited text",
    )?;
    key(window, Key::F4);
    let panel = gallery.get_editable_combo_popup_bounds();
    click(window, LogicalPosition::new(panel.x + 22., panel.y + 25.));
    check(
        !gallery.get_editable_combo_open()
            && gallery.get_editable_combo_value() == "8"
            && gallery.get_editable_combo_selected_index() == 0,
        "a pointer click commits an editable dropdown row",
    )?;
    gallery.set_combo_enabled(false);
    click(window, arrow);
    check(
        !gallery.get_editable_combo_open() && !gallery.get_editable_combo_focused(),
        "the disabled editable ComboBox refuses pointer entry",
    )?;
    gallery.set_combo_enabled(true);
    Ok(())
}

pub(super) fn verify(gallery: &GalleryWindow) -> SmokeResult {
    gallery.set_page(GalleryPage::ComboBox);
    gallery.invoke_select_theme(FluentColorScheme::Light);
    let window = gallery.window();
    let trigger = trigger_center(gallery);
    let bounds = gallery.get_combo_trigger_bounds();
    let sample = LogicalPosition::new(bounds.x + 5., bounds.y + bounds.height / 2.);
    window.dispatch_event(WindowEvent::PointerMoved {
        position: LogicalPosition::new(1., 1.),
    });
    let normal = read_rgb(window, sample)?;
    let focus_edge = LogicalPosition::new(bounds.x - 2., trigger.y);
    let unfocused = read_rgb(window, focus_edge)?;
    check(
        gallery.get_combo_selected_index() == -1
            && gallery.get_combo_selected_text().is_empty()
            && gallery.get_combo_state() == ButtonState::Normal
            && !gallery.get_combo_keyboard_focus(),
        "ComboBox starts at its placeholder",
    )?;
    window.dispatch_event(WindowEvent::PointerMoved { position: trigger });
    let hovered = read_rgb(window, sample)?;
    check(
        gallery.get_combo_state() == ButtonState::PointerOver && normal != hovered,
        "the ComboBox hover state is painted",
    )?;
    window.dispatch_event(WindowEvent::PointerPressed {
        position: trigger,
        button: PointerEventButton::Left,
    });
    check(
        !gallery.get_combo_open()
            && gallery.get_combo_state() == ButtonState::Pressed
            && read_rgb(window, sample)? != hovered,
        "the ComboBox paints its press before opening",
    )?;
    window.dispatch_event(WindowEvent::PointerReleased {
        position: trigger,
        button: PointerEventButton::Left,
    });
    assert_highlight(
        gallery,
        0,
        "opening without a selection focuses the first option",
    )?;
    let short_panel = gallery.get_combo_popup_bounds();
    let fill = gallery.global::<FluentTheme>().get_acrylic_fallback();
    check(
        read_rgb(
            window,
            LogicalPosition::new(short_panel.x + short_panel.width - 4., short_panel.y + 23.),
        )? == [fill.red(), fill.green(), fill.blue()],
        "a dropdown with room for every option has no scrollbar thumb",
    )?;
    verify_faceplate_alignment(gallery, 2)?;
    check(
        !gallery.get_combo_candidate_focus_visible(),
        "a pointer-opened ComboBox does not paint a keyboard focus ring on the candidate",
    )?;
    key(window, Key::DownArrow);
    key(window, Key::DownArrow);
    assert_highlight(gallery, 3, "popup navigation skips a disabled option")?;
    check(
        gallery.get_combo_selected_index() == -1 && gallery.get_combo_selection_edits() == 0,
        "the highlight leaves the committed selection intact",
    )?;
    key(window, Key::Escape);
    check(
        !gallery.get_combo_open()
            && gallery.get_combo_selected_index() == -1
            && gallery.get_combo_trigger_focused()
            && !gallery.get_combo_keyboard_focus(),
        "Escape restores pointer focus without a keyboard focus ring",
    )?;
    check(
        read_rgb(window, focus_edge)? == unfocused,
        "pointer focus restoration leaves the faceplate edge visually unchanged",
    )?;

    key(window, Key::Return);
    assert_highlight(gallery, 0, "Enter opens the collapsed ComboBox")?;
    check(
        gallery.get_combo_candidate_focus_visible(),
        "keyboard activation shows a keyboard indicator in the dropdown",
    )?;
    let popup = gallery.get_combo_popup_bounds();
    let row = |index: f32| LogicalPosition::new(popup.x + 16., popup.y + (index + 0.5) * 40.);
    click(window, row(2.));
    check(
        gallery.get_combo_open(),
        "clicking a disabled option keeps the popup open",
    )?;
    click(window, row(3.));
    check(
        !gallery.get_combo_open()
            && gallery.get_combo_selected_index() == 3
            && gallery.get_combo_selected_text() == "Field notes"
            && gallery.get_combo_trigger_focused()
            && !gallery.get_combo_keyboard_focus()
            && gallery.get_combo_selection_edits() == 1,
        "a pointer commit closes and updates the display and selected index once",
    )?;
    key(window, Key::Space);
    assert_highlight(gallery, 3, "reopening focuses the committed option")?;
    verify_faceplate_alignment(gallery, 3)?;
    let position = gallery.get_combo_popup_bounds().y;
    key(window, Key::UpArrow);
    check(
        gallery.get_combo_popup_bounds().y == position,
        "candidate movement keeps the popup anchored",
    )?;
    key(window, Key::Return);
    check(
        !gallery.get_combo_open() && gallery.get_combo_selected_text() == "Studio",
        "Enter commits the highlighted option",
    )?;
    key(window, Key::DownArrow);
    check(
        !gallery.get_combo_open() && gallery.get_combo_selected_index() == 3,
        "a collapsed arrow commits and skips disabled rows",
    )?;
    window.dispatch_event(WindowEvent::PointerScrolled {
        position: trigger,
        delta_x: 0.,
        delta_y: 24.,
    });
    check(
        gallery.get_combo_selected_index() == 1,
        "a wheel step changes a focused collapsed selector",
    )?;

    key(window, Key::F4);
    assert_highlight(gallery, 1, "F4 opens at the selection")?;
    key(window, Key::Tab);
    check(
        !gallery.get_combo_open() && gallery.get_combo_clear_focused(),
        "Tab dismisses the ComboBox and continues to the next control",
    )?;
    click(window, trigger);
    key(window, Key::DownArrow);
    click(window, LogicalPosition::new(2., 2.));
    check(
        !gallery.get_combo_open()
            && gallery.get_combo_selected_index() == 1
            && gallery.get_combo_trigger_focused(),
        "click away dismisses without committing the candidate",
    )?;
    with_modifier(window, Key::Alt, || key(window, Key::DownArrow));
    assert_highlight(gallery, 1, "Option-Down opens at the selection")?;
    with_modifier(window, Key::Alt, || key(window, Key::UpArrow));
    check(!gallery.get_combo_open(), "Option-Up closes the popup")?;
    key(window, Key::F4);
    key(window, Key::F4);
    check(!gallery.get_combo_open(), "F4 closes the open popup")?;

    let edits = gallery.get_combo_selection_edits();
    gallery.invoke_set_combo_selected_index(4);
    check(
        gallery.get_combo_selected_text() == "Print room"
            && gallery.get_combo_selection_edits() == edits,
        "host writes update the displayed option without a user edit",
    )?;
    click(window, trigger);
    gallery.set_combo_enabled(false);
    window.take_snapshot()?;
    check(
        !gallery.get_combo_open() && gallery.get_combo_state() == ButtonState::Disabled,
        "disabling the ComboBox closes the popup and changes its state",
    )?;
    let disabled = read_rgb(window, sample)?;
    click(window, trigger);
    check(
        !gallery.get_combo_open() && disabled != normal,
        "the disabled trigger stays closed",
    )?;
    gallery.set_combo_enabled(true);

    gallery.set_combo_empty(true);
    window.take_snapshot()?;
    click(window, trigger);
    check(
        !gallery.get_combo_open() && gallery.get_combo_selected_index() == -1,
        "an empty model clears selection and has no dropdown",
    )?;
    gallery.set_combo_empty(false);
    verify_surface(gallery, trigger)?;
    verify_long_model(gallery, trigger)
}

fn verify_surface(gallery: &GalleryWindow, trigger: LogicalPosition) -> SmokeResult {
    let window = gallery.window();
    let theme = gallery.global::<FluentTheme>();
    gallery.set_combo_material_probe(true);
    window.take_snapshot()?;
    click(window, trigger);
    assert_highlight(gallery, 0, "ComboBox material probe opens at the first row")?;
    let panel = gallery.get_combo_popup_bounds();
    let accent = theme.get_accent_fill();
    check(
        read_rgb(window, LogicalPosition::new(panel.x + 8.5, panel.y + 21.))?
            == [accent.red(), accent.green(), accent.blue()],
        "the ComboBox selection paints its accent pill",
    )?;
    let seam = panel.x + panel.width / 2.;
    let y = panel.y + panel.height - 12.;
    let contrast = |distance: f32| -> Result<u16, Box<dyn std::error::Error>> {
        let left = read_rgb(window, LogicalPosition::new(seam - distance, y))?;
        let right = read_rgb(window, LogicalPosition::new(seam + distance, y))?;
        Ok(left
            .into_iter()
            .zip(right)
            .map(|(a, b)| u16::from(a.abs_diff(b)))
            .sum())
    };
    check(
        contrast(35.)? == 0,
        "reduced transparency uses a uniform opaque fallback",
    )?;
    theme.set_reduce_transparency(false);
    let near = contrast(2.)?;
    let far = contrast(35.)?;
    theme.set_reduce_transparency(true);
    check(
        near < far && far > 10,
        "Skia blurs the existing window content through the ComboBox popup",
    )?;
    key(window, Key::Escape);
    gallery.set_combo_material_probe(false);
    Ok(())
}

fn verify_long_model(gallery: &GalleryWindow, trigger: LogicalPosition) -> SmokeResult {
    let window = gallery.window();
    let (items, reads) = list_demo::items();
    gallery.set_composition_items(items);
    gallery.set_combo_use_long_list(true);
    window.take_snapshot()?;
    click(window, trigger);
    let first = gallery.get_combo_first_row();
    check(
        first > 0,
        "placement of a long unselected model is based on the middle row",
    )?;
    let enabled = if first % 17 == 2 { first + 1 } else { first };
    assert_highlight(
        gallery,
        enabled,
        "a long unselected dropdown focuses its first visible enabled item",
    )?;
    verify_faceplate_alignment(gallery, 50_000)?;
    key(window, Key::End);
    assert_highlight(
        gallery,
        99_999,
        "the virtual dropdown navigates to its last row",
    )?;
    check(
        gallery.get_combo_selected_index() == -1 && reads.get() < 500,
        "the dropdown previews the last record without loading the whole model",
    )?;
    let panel = gallery.get_combo_popup_bounds();
    click(
        window,
        LogicalPosition::new(panel.x + 16., panel.y + panel.height - 20.),
    );
    check(
        !gallery.get_combo_open() && gallery.get_combo_selected_text() == "Record 100000",
        "End scrolls the last virtual option into pointer reach",
    )?;
    click(window, trigger);
    assert_highlight(
        gallery,
        99_999,
        "reopening scrolls to the selected virtual option",
    )?;
    verify_faceplate_alignment(gallery, 99_999)?;
    click(window, trigger);
    check(
        !gallery.get_combo_open() && gallery.get_combo_selected_index() == 99_999,
        "the row under the faceplate is the aligned selection",
    )?;
    click(window, trigger);
    gallery.set_page(GalleryPage::Button);
    window.take_snapshot()?;
    check(
        !gallery.get_combo_open(),
        "changing pages closes the dropdown",
    )?;
    gallery.set_page(GalleryPage::ComboBox);
    check(
        gallery.get_combo_selected_index() == 99_999,
        "page navigation preserves the committed value",
    )?;
    click(window, trigger);
    let panel = gallery.get_combo_popup_bounds();
    let bar_x = panel.x + panel.width - 5.;
    let thumb = LogicalPosition::new(bar_x, panel.y + panel.height - 32.);
    let panel_fill = gallery.global::<FluentTheme>().get_acrylic_fallback();
    let fill = [panel_fill.red(), panel_fill.green(), panel_fill.blue()];
    check(
        read_rgb(window, thumb)? != fill
            && read_rgb(
                window,
                LogicalPosition::new(bar_x, panel.y + panel.height - 4.),
            )? == fill,
        "the idle mouse scrollbar leaves the end button outside the thumb track",
    )?;
    let expanded_edge = LogicalPosition::new(bar_x - 3., thumb.y);
    let idle_edge = read_rgb(window, expanded_edge)?;
    window.dispatch_event(WindowEvent::PointerMoved { position: thumb });
    check(
        read_rgb(window, expanded_edge)? != idle_edge && read_rgb(window, thumb)? != fill,
        "hover widens the thumb on its existing track",
    )?;
    let button = PointerEventButton::Left;
    window.dispatch_event(WindowEvent::PointerPressed {
        position: thumb,
        button,
    });
    let top = LogicalPosition::new(bar_x, panel.y + 7.);
    window.dispatch_event(WindowEvent::PointerMoved { position: top });
    window.take_snapshot()?;
    let bottom = LogicalPosition::new(bar_x, panel.y + panel.height - 7.);
    window.dispatch_event(WindowEvent::PointerMoved { position: bottom });
    window.take_snapshot()?;
    window.dispatch_event(WindowEvent::PointerMoved { position: top });
    window.dispatch_event(WindowEvent::PointerReleased {
        position: top,
        button,
    });
    window.take_snapshot()?;
    check(
        gallery.get_combo_open() && reads.get() < 500,
        "scrollbar dragging retains virtualization and keeps the dropdown open",
    )?;
    window.dispatch_event(WindowEvent::PointerMoved {
        position: LogicalPosition::new(panel.x + 16., panel.y + 25.),
    });
    check(
        read_rgb(window, LogicalPosition::new(bar_x, panel.y + 4.))? == fill
            && read_rgb(window, LogicalPosition::new(bar_x, panel.y + 20.))? != fill,
        "the idle scrollbar thumb returns to the start of the mouse track",
    )?;
    click(window, LogicalPosition::new(panel.x + 16., panel.y + 25.));
    check(
        gallery.get_combo_selected_text() == "Record 1",
        "dragging the thumb to the top makes the first virtual row clickable",
    )?;
    println!("ComboBox virtual model made {} row-data reads", reads.get());
    gallery.set_combo_use_long_list(false);
    Ok(())
}

pub(super) fn verify_compact(gallery: &GalleryWindow, width: f32) -> SmokeResult {
    gallery.set_page(GalleryPage::ComboBox);
    let window = gallery.window();
    let trigger = gallery.get_combo_trigger_bounds();
    check(
        trigger.x + trigger.width <= width && trigger.height >= 44.,
        "the compact ComboBox fits and has a touch-sized trigger",
    )?;

    for scheme in [FluentColorScheme::Light, FluentColorScheme::Dark] {
        gallery.invoke_select_theme(scheme);
        click(window, trigger_center(gallery));
        let panel = gallery.get_combo_popup_bounds();
        let height = window.size().to_logical(window.scale_factor()).height;
        check(
            gallery.get_combo_open()
                && panel.x >= 0.
                && panel.x + panel.width <= width
                && panel.y >= 0.
                && panel.y + panel.height <= height,
            "the compact dropdown fits within the viewport",
        )?;
        let fill = gallery.global::<FluentTheme>().get_acrylic_fallback();
        check(
            read_rgb(window, LogicalPosition::new(panel.x + 2., panel.y + 20.))?
                == [fill.red(), fill.green(), fill.blue()],
            "the popup draws its themed surface",
        )?;
        key(window, Key::Escape);
    }
    key(window, Key::Tab);
    for _ in 0..20 {
        if gallery.get_combo_keyboard_focus() {
            break;
        }
        key(window, Key::Tab);
    }
    check(
        gallery.get_combo_keyboard_focus(),
        "Tab shows ComboBox keyboard focus",
    )?;
    let edge = LogicalPosition::new(trigger.x - 2., trigger.y + trigger.height / 2.);
    let focused = read_rgb(window, edge)?;
    key(window, Key::Tab);
    check(
        !gallery.get_combo_keyboard_focus() && read_rgb(window, edge)? != focused,
        "Skia paints the ComboBox focus ring without clipping it",
    )?;

    let theme = gallery.global::<FluentTheme>();
    if cfg!(target_os = "macos") {
        theme.set_ui_scale(1.25);
        window.take_snapshot()?;
    }
    let result = verify_compact_editable(gallery, width);
    theme.set_ui_scale(1.);
    result
}

fn verify_editable_seam(
    gallery: &GalleryWindow,
    editor: &OverlayArea,
    panel: &OverlayArea,
) -> SmokeResult {
    let above = panel.y < editor.y;
    let seam = if above {
        panel.y + panel.height
    } else {
        panel.y
    };
    let editor_edge = if above {
        editor.y
    } else {
        editor.y + editor.height
    };
    check(
        (seam - editor_edge).abs() <= 0.1,
        "popup and editor share an edge",
    )?;
    let y = seam + if above { -2.75 } else { 2.75 };
    let fill = gallery.global::<FluentTheme>().get_acrylic_fallback();
    let expected = [fill.red(), fill.green(), fill.blue()];
    for x in [panel.x + 2.75, panel.x + panel.width - 2.75] {
        let actual = read_rgb(gallery.window(), LogicalPosition::new(x, y))?;
        if actual
            .into_iter()
            .zip(expected)
            .any(|(a, b)| a.abs_diff(b) > 3)
        {
            return Err(format!(
                "Skia should paint the square popup corner at ({x}, {y}) in {expected:?}, got {actual:?}"
            ).into());
        }
    }
    Ok(())
}

fn verify_compact_editable(gallery: &GalleryWindow, width: f32) -> SmokeResult {
    let window = gallery.window();
    let height = window.size().to_logical(window.scale_factor()).height;
    let initial = gallery.get_editable_combo_bounds();
    let mut entry = initial.clone();
    if entry.y + entry.height > height - 24. {
        window.dispatch_event(WindowEvent::PointerScrolled {
            position: LogicalPosition::new(width - 30., height - 35.),
            delta_x: 0.,
            delta_y: -height,
        });
        window.take_snapshot()?;
        entry = gallery.get_editable_combo_bounds();
    }
    if entry.x + entry.width > width || entry.y + entry.height > height || entry.height < 44. {
        return Err(format!(
            "the compact editable ComboBox requires a touch-sized input in {width}×{height}, got {entry:?} after scrolling from {initial:?}",
        ).into());
    }
    for scheme in [FluentColorScheme::Light, FluentColorScheme::Dark] {
        gallery.invoke_select_theme(scheme);
        click(
            window,
            LogicalPosition::new(entry.x + entry.width - 19., entry.y + entry.height / 2.),
        );
        let panel = gallery.get_editable_combo_popup_bounds();
        let adjoining = (panel.y + panel.height - entry.y).abs() < 1.
            || (panel.y - (entry.y + entry.height)).abs() < 1.;
        if !gallery.get_editable_combo_open()
            || !adjoining
            || panel.x < 0.
            || panel.x + panel.width > width
            || panel.y < 0.
            || panel.y + panel.height > height
        {
            return Err(format!(
                "compact editable dropdown should adjoin {entry:?} within {width}×{height}, got {panel:?} with open={}",
                gallery.get_editable_combo_open(),
            ).into());
        }
        let fill = gallery.global::<FluentTheme>().get_acrylic_fallback();
        check(
            read_rgb(window, LogicalPosition::new(panel.x + 2., panel.y + 20.))?
                == [fill.red(), fill.green(), fill.blue()],
            "the compact editable dropdown draws the themed surface",
        )?;
        verify_editable_seam(gallery, &entry, &panel)?;
        key(window, Key::Escape);
    }
    Ok(())
}

fn verify_faceplate_alignment(gallery: &GalleryWindow, aligned_row: i32) -> SmokeResult {
    let panel = gallery.get_combo_popup_bounds();
    let face = gallery.get_combo_trigger_bounds();
    let row_height = if gallery.get_touch_targets() {
        44.
    } else {
        40.
    };
    let row_center = panel.y
        + 5.
        + (aligned_row - gallery.get_combo_first_row()) as f32 * row_height
        + row_height / 2.;
    check(
        (panel.x - face.x).abs() < 1. && (row_center - face.y - face.height / 2.).abs() < 1.,
        "the placed row overlays the ComboBox faceplate without an anchor gap",
    )
}

fn assert_highlight(gallery: &GalleryWindow, index: i32, description: &'static str) -> SmokeResult {
    gallery.window().take_snapshot()?;
    let current = gallery.get_combo_highlighted_index();
    if !gallery.get_combo_open() || current != index {
        return Err(format!(
            "{description} (open: {}, highlight: {current}, expected: {index})",
            gallery.get_combo_open()
        )
        .into());
    }
    Ok(())
}

fn trigger_center(gallery: &GalleryWindow) -> LogicalPosition {
    let rect = gallery.get_combo_trigger_bounds();
    LogicalPosition::new(rect.x + rect.width / 2., rect.y + rect.height / 2.)
}

fn key(window: &Window, key: Key) {
    type_key(window, key);
}

fn type_key(window: &Window, text: impl Into<SharedString>) {
    let text = text.into();
    window.dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    window.dispatch_event(WindowEvent::KeyReleased { text });
}

fn with_modifier(window: &Window, modifier: Key, action: impl FnOnce()) {
    window.dispatch_event(WindowEvent::KeyPressed {
        text: modifier.into(),
    });
    action();
    window.dispatch_event(WindowEvent::KeyReleased {
        text: modifier.into(),
    });
}

pub(super) fn verify_motion(gallery: &GalleryWindow, finish: impl FnOnce(SmokeResult) + 'static) {
    use std::time::Duration;
    gallery.set_page(GalleryPage::ComboBox);
    gallery.invoke_select_theme(FluentColorScheme::Light);
    gallery.global::<FluentTheme>().set_reduced_motion(false);
    let trigger = trigger_center(gallery);
    let button = PointerEventButton::Left;
    gallery
        .window()
        .dispatch_event(WindowEvent::PointerPressed {
            position: trigger,
            button,
        });
    let weak = gallery.as_weak();
    slint::Timer::single_shot(Duration::from_millis(100), move || {
        let Some(gallery) = weak.upgrade() else {
            return;
        };
        let window = gallery.window();
        let begin = || -> Result<_, Box<dyn std::error::Error>> {
            window.take_snapshot()?;
            check(
                gallery.get_combo_glyph_offset() > 0.,
                "ComboBox chevron moves down when pressed",
            )?;
            window.dispatch_event(WindowEvent::PointerReleased {
                position: trigger,
                button,
            });
            let panel = gallery.get_combo_popup_bounds();
            let x = panel.x + 2.;
            let fill = gallery.global::<FluentTheme>().get_acrylic_fallback();
            let expected = [fill.red(), fill.green(), fill.blue()];
            let center = LogicalPosition::new(x, trigger.y);
            let top = LogicalPosition::new(x, panel.y + 10.);
            let bottom = LogicalPosition::new(x, panel.y + panel.height - 10.);
            check(
                read_rgb(window, center)? == expected
                    && read_rgb(window, top)? != expected
                    && read_rgb(window, bottom)? != expected,
                "opening shows a solid band at the faceplate and reveals towards both ends",
            )?;
            Ok((expected, top, bottom))
        };
        let (expected, top, bottom) = match begin() {
            Ok(result) => result,
            Err(error) => {
                finish(Err(error));
                return;
            }
        };
        let weak = gallery.as_weak();
        slint::Timer::single_shot(Duration::from_millis(325), move || {
            let Some(gallery) = weak.upgrade() else {
                return;
            };
            let window = gallery.window();
            let verify_open = || -> SmokeResult {
                check(
                    read_rgb(window, top)? == expected
                        && read_rgb(window, bottom)? == expected
                        && gallery.get_combo_glyph_offset().abs() < 0.05,
                    "both split edges reach their final positions",
                )?;
                click(window, LogicalPosition::new(2., 2.));
                check(
                    gallery.get_combo_open() && gallery.get_combo_closing(),
                    "outside dismissal starts the collapse while keeping its visual alive",
                )
            };
            if let Err(error) = verify_open() {
                finish(Err(error));
                return;
            }

            let weak = gallery.as_weak();
            slint::Timer::single_shot(Duration::from_millis(200), move || {
                let Some(gallery) = weak.upgrade() else {
                    return;
                };
                let window = gallery.window();
                let result = (|| {
                    check(
                        !gallery.get_combo_open()
                            && !gallery.get_combo_closing()
                            && !gallery.get_combo_keyboard_focus(),
                        "the close finishes and restores pointer focus without a black border",
                    )?;
                    gallery.global::<FluentTheme>().set_reduced_motion(true);
                    click(window, trigger);
                    check(
                        read_rgb(window, top)? == expected
                            && read_rgb(window, bottom)? == expected
                            && gallery.get_combo_glyph_offset().abs() < 0.05,
                        "reduced motion displays settled bounds immediately",
                    )?;
                    key(window, Key::Escape);
                    check(
                        !gallery.get_combo_open(),
                        "reduced motion dismisses immediately",
                    )
                })();
                finish(result);
            });
        });
    });
}
