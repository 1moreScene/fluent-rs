use std::error::Error;

use slint::{
    ComponentHandle, LogicalPosition, Window,
    platform::{Key, PointerEventButton, WindowEvent},
};

use super::{check, click, read_rgb};
use crate::{ButtonState, FluentColorScheme, FluentTheme, GalleryPage, GalleryWindow};

pub(super) fn verify(gallery: &GalleryWindow) -> Result<(), Box<dyn Error>> {
    let window = gallery.window();
    gallery.set_page(GalleryPage::Radios);
    gallery.set_radio_max_columns(1);
    gallery.set_profile_selection(3);
    gallery.set_quality_selection(1);

    for _ in 0..18 {
        key(window, Key::Tab);
        if gallery.get_profile_focus() >= 0 {
            break;
        }
    }
    check(
        gallery.get_profile_focus() == 3,
        "Tab enters the selected radio",
    )?;

    gallery.set_profile_selection(-1);
    let origin = gallery.get_profile_origin();
    let first = LogicalPosition::new(origin.x + 10., origin.y + 16.);
    click(window, LogicalPosition::new(first.x, first.y + 40.));
    check(
        gallery.get_profile_selection() == -1,
        "disabled radio rejects clicks",
    )?;
    click(window, first);
    let changes = gallery.get_profile_changes();
    click(window, first);
    check(
        gallery.get_profile_selection() == 0 && gallery.get_profile_changes() == changes,
        "click selects once and the selected radio cannot be toggled off",
    )?;

    check_arrow(gallery, Key::UpArrow, 0, 0)?;
    check_arrow(gallery, Key::RightArrow, 0, 0)?;
    check_arrow(gallery, Key::DownArrow, 2, 2)?;
    window.dispatch_event(WindowEvent::KeyPressed {
        text: Key::Control.into(),
    });
    check_arrow(gallery, Key::DownArrow, 3, 2)?;
    window.dispatch_event(WindowEvent::KeyReleased {
        text: Key::Control.into(),
    });
    key(window, Key::Return);
    check(
        gallery.get_profile_selection() == 2,
        "Enter preserves radio selection",
    )?;
    key(window, Key::Space);
    check(
        gallery.get_profile_selection() == 3,
        "Space selects the focused radio",
    )?;
    check_arrow(gallery, Key::DownArrow, 4, 4)?;
    check_arrow(gallery, Key::DownArrow, 4, 4)?;

    window.dispatch_event(WindowEvent::KeyPressed {
        text: Key::Control.into(),
    });
    check_arrow(gallery, Key::UpArrow, 3, 4)?;
    window.dispatch_event(WindowEvent::KeyReleased {
        text: Key::Control.into(),
    });
    key(window, Key::Tab);
    check(
        gallery.get_profile_focus() == -1 && gallery.get_quality_focus() == 1,
        "Tab leaves the group even when focus and selection differ",
    )?;
    window.dispatch_event(WindowEvent::KeyPressed {
        text: Key::Shift.into(),
    });
    key(window, Key::Tab);
    window.dispatch_event(WindowEvent::KeyReleased {
        text: Key::Shift.into(),
    });
    check(
        gallery.get_profile_focus() == 4,
        "Shift+Tab re-enters at the selected radio",
    )?;
    key(window, Key::Tab);
    gallery.set_profile_selection(-1);
    window.dispatch_event(WindowEvent::KeyPressed {
        text: Key::Shift.into(),
    });
    key(window, Key::Tab);
    window.dispatch_event(WindowEvent::KeyReleased {
        text: Key::Shift.into(),
    });
    check(
        gallery.get_profile_focus() == 0 && gallery.get_profile_selection() == -1,
        "Tab enters an unselected group at its first enabled radio without selecting",
    )?;

    gallery.set_radio_max_columns(3);
    let _ = window.take_snapshot()?;
    check(
        gallery.get_profile_columns() == 3,
        "wide radio group uses three columns",
    )?;
    let origin = gallery.get_profile_origin();
    click(window, LogicalPosition::new(origin.x + 10., origin.y + 16.));
    check_arrow(gallery, Key::RightArrow, 2, 2)?;
    check_arrow(gallery, Key::RightArrow, 4, 4)?;
    check_arrow(gallery, Key::RightArrow, 4, 4)?;
    check_arrow(gallery, Key::LeftArrow, 2, 2)?;

    let quality = gallery.get_quality_origin();
    click(
        window,
        LogicalPosition::new(quality.x + 10., quality.y + 16.),
    );
    check(
        gallery.get_quality_selection() == 0 && gallery.get_profile_selection() == 2,
        "selection is independent between groups",
    )?;

    let standalone = gallery.get_standalone_radio_center();
    let position = LogicalPosition::new(standalone.x, standalone.y);
    let outside = LogicalPosition::new(1., 1.);
    window.dispatch_event(WindowEvent::PointerMoved { position });
    check(
        gallery.get_standalone_radio_state() == ButtonState::PointerOver,
        "radio hover",
    )?;
    let button = PointerEventButton::Left;
    window.dispatch_event(WindowEvent::PointerPressed { position, button });
    check(
        gallery.get_standalone_radio_state() == ButtonState::Pressed
            && !gallery.get_standalone_radio_checked(),
        "radio waits for release before selecting",
    )?;
    window.dispatch_event(WindowEvent::PointerMoved { position: outside });
    window.dispatch_event(WindowEvent::PointerReleased {
        position: outside,
        button,
    });
    check(
        !gallery.get_standalone_radio_checked(),
        "drag outside cancels radio activation",
    )?;
    click(window, position);
    check(
        gallery.get_standalone_radio_checked(),
        "standalone radio selects on click",
    )
}

fn key(window: &Window, key: Key) {
    window.dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window.dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}

fn check_arrow(
    gallery: &GalleryWindow,
    direction: Key,
    focus: i32,
    selected: i32,
) -> Result<(), Box<dyn Error>> {
    let window = gallery.window();
    key(window, direction);
    slint::platform::update_timers_and_animations();
    let actual = (gallery.get_profile_focus(), gallery.get_profile_selection());
    if actual == (focus, selected) {
        Ok(())
    } else {
        Err(format!("Radio {direction:?}: expected ({focus}, {selected}), got {actual:?}").into())
    }
}

pub(super) fn verify_compact(gallery: &GalleryWindow) -> Result<(), Box<dyn Error>> {
    gallery.set_page(GalleryPage::Radios);
    gallery.set_radio_max_columns(3);
    check(
        gallery.get_profile_columns() == 2,
        "radio group drops a column in the narrow window",
    )?;
    check(
        gallery.get_standalone_radio_height() >= 44.,
        "radio touch target height",
    )?;
    let window = gallery.window();
    let theme = gallery.global::<FluentTheme>();
    theme.set_reduced_motion(true);
    window.dispatch_event(WindowEvent::PointerMoved {
        position: LogicalPosition::new(1., 1.),
    });
    let origin = gallery.get_profile_origin();
    let mark = LogicalPosition::new(origin.x + 10., origin.y + 22.);
    for scheme in [
        FluentColorScheme::Light,
        FluentColorScheme::Dark,
        FluentColorScheme::HighContrast,
    ] {
        gallery.invoke_select_theme(scheme);
        gallery.set_profile_selection(-1);
        let unchecked = read_rgb(window, mark)?;
        gallery.set_profile_selection(0);
        let checked = read_rgb(window, mark)?;
        let expected = if scheme == FluentColorScheme::HighContrast {
            theme.get_contrast_highlight()
        } else {
            theme.get_accent_text()
        };
        check(
            checked == [expected.red(), expected.green(), expected.blue()] && checked != unchecked,
            "radio indicator uses the selected inner color",
        )?;
        gallery.set_profile_selection(2);
        check(
            read_rgb(window, mark)? == unchecked,
            "the old radio returns to unchecked painting when another is selected",
        )?;
    }
    Ok(())
}
