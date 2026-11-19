use std::error::Error;

use slint::{
    ComponentHandle, LogicalPosition, SharedString, Window,
    platform::{Key, PointerEventButton, WindowEvent},
};

use super::{check, click, read_rgb};
use crate::{FluentColorScheme, FluentTheme, GalleryPage, GalleryWindow, TextBoxState};

pub(super) fn verify_page_scrollbar(gallery: &GalleryWindow) -> Result<(), Box<dyn Error>> {
    gallery.set_page(GalleryPage::TextBox);
    let theme = gallery.global::<FluentTheme>();
    theme.set_scheme(FluentColorScheme::Light);
    let window = gallery.window();
    window.take_snapshot()?;
    let rail = gallery.get_text_box_page_scroll_rail();
    window.dispatch_event(WindowEvent::PointerMoved {
        position: LogicalPosition::new(1., 1.),
    });
    let idle = read_rgb(window, rail)?;
    window.dispatch_event(WindowEvent::PointerMoved { position: rail });
    let expanded = read_rgb(window, rail)?;
    let right = gallery.get_text_box_right();
    if right + 6. > rail.x || expanded == idle {
        return Err(format!(
            "the page scrollbar should expand in its own gutter, got TextBox edge {right}, rail {rail:?} and pixels {idle:?} -> {expanded:?}"
        ).into());
    }
    let track = LogicalPosition::new(rail.x - 6., rail.y);
    let initial_track = read_rgb(window, track)?;
    let reduced = theme.get_reduce_transparency();
    theme.set_reduce_transparency(!reduced);
    let alternative = read_rgb(window, track);
    theme.set_reduce_transparency(reduced);
    check(
        alternative? != initial_track,
        "the hovered rail switches between acrylic tint and its solid fallback",
    )
}

pub(super) fn verify(gallery: &GalleryWindow) -> Result<(), Box<dyn Error>> {
    gallery.set_page(GalleryPage::TextBox);
    gallery
        .global::<FluentTheme>()
        .set_scheme(FluentColorScheme::Light);
    let window = gallery.window();
    let editable = LogicalPosition::new(gallery.get_text_box_x(), gallery.get_text_box_y());
    window.dispatch_event(WindowEvent::PointerMoved {
        position: LogicalPosition::new(1., 1.),
    });
    check(
        gallery.get_text_box_state() == TextBoxState::Normal,
        "TextBox normal",
    )?;
    window.dispatch_event(WindowEvent::PointerMoved { position: editable });
    check(
        gallery.get_text_box_state() == TextBoxState::PointerOver,
        "TextBox hover",
    )?;

    for _ in 0..16 {
        key(window, Key::Tab);
        if gallery.get_text_box_state() == TextBoxState::Focused {
            break;
        }
    }
    check(
        gallery.get_text_box_state() == TextBoxState::Focused,
        "Tab focuses TextBox",
    )?;
    check(
        !gallery.get_text_box_clear_visible(),
        "empty TextBox hides the clear button",
    )?;
    verify_focus_colors(gallery)?;

    for character in ["a", "b", "c"] {
        key(window, character);
    }
    check(gallery.get_text_box_text() == "abc", "typing into TextBox")?;
    key(window, Key::LeftArrow);
    key(window, Key::Backspace);
    check(
        gallery.get_text_box_text() == "ac",
        "Left and Backspace edit at the caret",
    )?;
    shortcut(window, "z", false);
    check(
        gallery.get_text_box_text() == "abc",
        "undo restores deletion",
    )?;
    if cfg!(target_os = "windows") {
        shortcut(window, "y", false);
    } else {
        shortcut(window, "z", true);
    }
    check(gallery.get_text_box_text() == "ac", "redo repeats deletion")?;
    let (line_start, line_end) = if cfg!(target_os = "macos") {
        (Key::LeftArrow, Key::RightArrow)
    } else {
        (Key::Home, Key::End)
    };
    let move_to_line_edge = |edge| {
        if cfg!(target_os = "macos") {
            shortcut(window, edge, false);
        } else {
            key(window, edge);
        }
    };
    move_to_line_edge(line_end);
    key(window, "中");
    move_to_line_edge(line_start);
    key(window, Key::Delete);
    check(
        gallery.get_text_box_text() == "c中",
        "line navigation, Delete and Unicode",
    )?;
    let long_text = "A long single-line value. ".repeat(20);
    gallery.set_text_box_text(long_text.clone().into());
    move_to_line_edge(line_end);
    key(window, "!");
    move_to_line_edge(line_start);
    key(window, "^");
    check(
        gallery.get_text_box_text() == format!("^{long_text}!"),
        "line navigation allows edits at both ends of a clipped line",
    )?;
    shortcut(window, "a", false);
    check(
        gallery.get_text_box_selection(),
        "select all in editable TextBox",
    )?;
    key(window, "Z");
    check(
        gallery.get_text_box_text() == "Z",
        "typing replaces the selection",
    )?;
    check(
        gallery.get_text_box_edits() > 0,
        "TextBox notified about edits",
    )?;
    key(window, Key::Return);
    check(
        gallery.get_text_box_submissions() == 1 && gallery.get_text_box_text() == "Z",
        "Enter submits single-line text without inserting a newline",
    )?;
    verify_inline_clear(gallery, editable)?;

    gallery.set_text_box_enabled(false);
    click(window, editable);
    key(window, "X");
    check(
        gallery.get_text_box_state() == TextBoxState::Disabled
            && gallery.get_text_box_text() == "Z"
            && !gallery.get_text_box_clear_visible(),
        "disabled TextBox blocks focus and editing",
    )?;
    gallery.set_text_box_enabled(true);

    let read_only = LogicalPosition::new(gallery.get_read_only_x(), gallery.get_read_only_y());
    let original = gallery.get_read_only_text();
    click(window, read_only);
    check(
        gallery.get_read_only_state() == TextBoxState::Focused,
        "read-only TextBox accepts focus",
    )?;
    shortcut(window, "a", false);
    check(
        gallery.get_read_only_selection(),
        "read-only TextBox allows selection",
    )?;
    check(
        !gallery.get_other_clear_visible(),
        "read-only TextBox hides the clear button",
    )?;
    key(window, Key::Backspace);
    key(window, Key::Delete);
    key(window, "X");
    check(
        gallery.get_read_only_text() == original,
        "read-only TextBox blocks edits",
    )?;

    let multi_line = LogicalPosition::new(gallery.get_multi_line_x(), gallery.get_multi_line_y());
    click(window, multi_line);
    key(window, "a");
    key(window, Key::Return);
    key(window, "b");
    check(
        gallery.get_multi_line_text() == "a\nb",
        "multiline Enter inserts a newline",
    )?;
    check(
        !gallery.get_other_clear_visible(),
        "multiline TextBox hides the clear button",
    )?;
    for _ in 0..12 {
        key(window, Key::Return);
        key(window, "x");
    }
    let offset = gallery.get_multi_line_scroll_offset();
    check(offset > 0., "the multiline editor scrolls to its caret")?;
    window.dispatch_event(WindowEvent::PointerScrolled {
        position: LogicalPosition::new(gallery.get_multi_line_right() - 6., multi_line.y),
        delta_x: 0.,
        delta_y: 50.,
    });
    check(
        gallery.get_multi_line_scroll_offset() < offset,
        "the multiline scrollbar scrolls the editor viewport",
    )?;

    gallery.set_page(GalleryPage::Button);
    gallery.set_page(GalleryPage::TextBox);
    check(
        gallery.get_text_box_text() == "Z",
        "TextBox retains its text when switching pages",
    )
}

fn verify_inline_clear(
    gallery: &GalleryWindow,
    editable: LogicalPosition,
) -> Result<(), Box<dyn Error>> {
    let window = gallery.window();
    check(
        gallery.get_text_box_clear_visible(),
        "focused TextBox with text shows the clear button",
    )?;
    key(window, Key::Tab);
    check(
        gallery.get_text_box_state() != TextBoxState::Focused
            && !gallery.get_text_box_clear_visible(),
        "Tab skips the clear button and leaving TextBox hides it",
    )?;
    click(window, editable);
    shortcut(window, "a", false);
    check(
        gallery.get_text_box_selection(),
        "selection before clearing",
    )?;

    let right = gallery.get_text_box_right();
    let half_height = gallery.get_text_box_bottom() - editable.y;
    let clear = LogicalPosition::new(right - half_height - 2., editable.y);
    window.dispatch_event(WindowEvent::PointerMoved { position: editable });
    verify_clear_glyph_center(window, clear)?;
    let outside = LogicalPosition::new(clear.x - 40., clear.y);
    let button = PointerEventButton::Left;
    let edits_before = gallery.get_text_box_edits();
    window.dispatch_event(WindowEvent::PointerPressed {
        position: clear,
        button,
    });
    window.dispatch_event(WindowEvent::PointerMoved { position: outside });
    window.dispatch_event(WindowEvent::PointerReleased {
        position: outside,
        button,
    });
    check(
        gallery.get_text_box_text() == "Z" && gallery.get_text_box_edits() == edits_before,
        "dragging away from the clear button cancels the click",
    )?;
    check(
        gallery.get_text_box_clear_visible(),
        "canceling clear keeps the TextBox focused",
    )?;

    click(window, LogicalPosition::new(right - 2., clear.y));
    check(
        gallery.get_text_box_text() == "",
        "the trailing margin remains part of the clear button hit target",
    )?;
    check(
        gallery.get_text_box_edits() == edits_before + 1,
        "clear reports one edit",
    )?;
    check(
        gallery.get_text_box_state() == TextBoxState::Focused,
        "clear retains focus",
    )?;
    check(
        !gallery.get_text_box_selection(),
        "clear resets the selection",
    )?;
    check(
        !gallery.get_text_box_clear_visible(),
        "clear hides the button",
    )?;
    shortcut(window, "z", false);
    check(
        gallery.get_text_box_text() == "",
        "undo does not restore cleared content",
    )?;
    key(window, "Z");
    check(
        gallery.get_text_box_text() == "Z" && gallery.get_text_box_clear_visible(),
        "typing can resume immediately after clearing",
    )
}

fn verify_clear_glyph_center(
    window: &Window,
    expected: LogicalPosition,
) -> Result<(), Box<dyn Error>> {
    let scale = window.scale_factor();
    let frame = window.take_snapshot()?;
    let start = LogicalPosition::new(expected.x - 7., expected.y - 7.).to_physical(scale);
    let end = LogicalPosition::new(expected.x + 7., expected.y + 7.).to_physical(scale);
    let (mut left, mut top, mut right, mut bottom) = (end.x, end.y, start.x, start.y);
    for y in start.y..end.y {
        for x in start.x..end.x {
            let pixel = frame.as_slice()[y as usize * frame.width() as usize + x as usize];
            if pixel.r < 144 && pixel.g < 144 && pixel.b < 144 {
                left = left.min(x);
                top = top.min(y);
                right = right.max(x);
                bottom = bottom.max(y);
            }
        }
    }
    let center_x = (left + right + 1) as f32 / (2. * scale);
    let center_y = (top + bottom + 1) as f32 / (2. * scale);
    check(
        left <= right
            && top <= bottom
            && (center_x - expected.x).abs() <= 0.8
            && (center_y - expected.y).abs() <= 0.8,
        "clear glyph is centered in the inset visual surface",
    )
}

fn verify_focus_colors(gallery: &GalleryWindow) -> Result<(), Box<dyn Error>> {
    let theme = gallery.global::<FluentTheme>();
    let scale = gallery.window().scale_factor();
    let x = (gallery.get_text_box_x() * scale) as usize;
    let y = ((gallery.get_text_box_bottom() - 1.) * scale) as usize;
    for scheme in [FluentColorScheme::Light, FluentColorScheme::Dark] {
        theme.set_scheme(scheme);
        let snapshot = gallery.window().take_snapshot()?;
        let pixel = snapshot.as_slice().get(y * snapshot.width() as usize + x);
        let accent = theme.get_accent_fill();
        check(
            pixel.is_some_and(|pixel| {
                [pixel.r, pixel.g, pixel.b] == [accent.red(), accent.green(), accent.blue()]
            }),
            "TextBox focus underline uses the theme accent",
        )?;
    }
    theme.set_scheme(FluentColorScheme::Light);
    Ok(())
}

fn key(window: &Window, text: impl Into<SharedString>) {
    let text = text.into();
    window.dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    window.dispatch_event(WindowEvent::KeyReleased { text });
}

fn shortcut(window: &Window, text: impl Into<SharedString>, shift: bool) {
    // Slint's platform event uses Control for the macOS Command key.
    window.dispatch_event(WindowEvent::KeyPressed {
        text: Key::Control.into(),
    });
    if shift {
        window.dispatch_event(WindowEvent::KeyPressed {
            text: Key::Shift.into(),
        });
    }
    key(window, text);
    if shift {
        window.dispatch_event(WindowEvent::KeyReleased {
            text: Key::Shift.into(),
        });
    }
    window.dispatch_event(WindowEvent::KeyReleased {
        text: Key::Control.into(),
    });
}
