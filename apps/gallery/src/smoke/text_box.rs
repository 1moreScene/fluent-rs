use std::error::Error;

use slint::{
    ComponentHandle, LogicalPosition, SharedString, Window,
    platform::{Key, PointerEventButton, WindowEvent},
};

use super::{check, click};
use crate::{FluentColorScheme, FluentTheme, GalleryPage, GalleryWindow, TextBoxState};

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

    for _ in 0..10 {
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
    key(window, Key::End);
    key(window, "中");
    key(window, Key::Home);
    key(window, Key::Delete);
    check(
        gallery.get_text_box_text() == "c中",
        "Home, End, Delete and Unicode",
    )?;
    let long_text = "A long single-line value. ".repeat(20);
    gallery.set_text_box_text(long_text.clone().into());
    key(window, Key::End);
    key(window, "!");
    key(window, Key::Home);
    key(window, "^");
    check(
        gallery.get_text_box_text() == format!("^{long_text}!"),
        "Home and End edit both sides of a clipped line",
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

    let clear = LogicalPosition::new(gallery.get_text_box_right() - 16., editable.y);
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

    click(window, clear);
    check(gallery.get_text_box_text() == "", "click clears text")?;
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

fn shortcut(window: &Window, text: &str, shift: bool) {
    let modifier = if cfg!(target_os = "macos") {
        Key::Meta
    } else {
        Key::Control
    };
    window.dispatch_event(WindowEvent::KeyPressed {
        text: modifier.into(),
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
        text: modifier.into(),
    });
}
