use slint::{
    ComponentHandle, LogicalPosition, LogicalSize,
    platform::{Key, WindowEvent},
};

use super::{check, click, overlays::key, read_rgb};
use crate::{
    ContentDialogButton, ContentDialogResult, FluentColorScheme, FluentTheme, GalleryPage,
    GalleryWindow,
};

type SmokeResult = Result<(), Box<dyn std::error::Error>>;

pub(super) fn verify(gallery: &GalleryWindow) -> SmokeResult {
    use ContentDialogButton::{Close, Primary, Secondary};
    let window = gallery.window();
    gallery.set_page(GalleryPage::ContentDialog);
    gallery.invoke_select_theme(FluentColorScheme::Light);
    window.take_snapshot()?;

    let background = LogicalPosition::new(2., 2.);
    let normal_background = read_rgb(window, background)?;
    click(window, gallery.get_dialog_default_trigger());
    window.take_snapshot()?;
    check(
        gallery.get_dialog_open()
            && gallery.get_dialog_focused_command() == Primary
            && !gallery.get_dialog_default_trigger_focused(),
        "the configured default receives initial focus",
    )?;
    check(
        read_rgb(window, background)? != normal_background,
        "ContentDialog dims its background",
    )?;
    check_surface(gallery, [0xff; 3], [0xf3; 3])?;

    click(window, gallery.get_dialog_no_default_trigger());
    click(window, background);
    key(window, Key::F8);
    check(
        gallery.get_dialog_open()
            && gallery.get_dialog_openings() == 1
            && gallery.get_dialog_background_keys() == 0
            && gallery.get_dialog_completions() == 0,
        "outside clicks and background shortcuts are blocked",
    )?;
    for expected in [Secondary, Close, Primary, Secondary, Close, Primary] {
        key(window, Key::Tab);
        window.take_snapshot()?;
        check(
            gallery.get_dialog_focused_command() == expected,
            "Tab wraps inside the ContentDialog",
        )?;
    }
    for expected in [Close, Secondary, Primary] {
        key(window, Key::Backtab);
        window.take_snapshot()?;
        check(
            gallery.get_dialog_focused_command() == expected,
            "Shift+Tab wraps inside the ContentDialog",
        )?;
    }

    key(window, Key::Return);
    window.take_snapshot()?;
    check(
        !gallery.get_dialog_open()
            && gallery.get_dialog_result() == ContentDialogResult::Primary
            && gallery.get_dialog_completions() == 1
            && gallery.get_dialog_default_trigger_focused()
            && !gallery.get_dialog_trigger_focus_visible(),
        "Enter runs the default once and restores pointer-origin focus",
    )?;
    key(window, Key::F8);
    check(
        gallery.get_dialog_background_keys() == 1,
        "the background resumes after closing",
    )?;

    key(window, Key::Return);
    gallery.set_dialog_block_close(true);
    key(window, Key::Escape);
    check(
        gallery.get_dialog_open() && gallery.get_dialog_completions() == 1,
        "closing can be declined",
    )?;
    gallery.set_dialog_block_close(false);
    let size = window.size().to_logical(window.scale_factor());
    window.dispatch_event(WindowEvent::Resized {
        size: LogicalSize::new(390., 370.),
    });
    window.take_snapshot()?;
    check(
        gallery.get_dialog_open() && gallery.get_dialog_focused_command() == Primary,
        "resizing preserves dialog modality and focus",
    )?;
    check_centered(gallery, LogicalSize::new(390., 370.))?;
    window.dispatch_event(WindowEvent::Resized { size });
    key(window, Key::Escape);
    window.take_snapshot()?;
    check(
        !gallery.get_dialog_open()
            && gallery.get_dialog_result() == ContentDialogResult::None
            && gallery.get_dialog_completions() == 2
            && gallery.get_dialog_trigger_focus_visible(),
        "Escape cancels once and restores the keyboard focus ring",
    )?;

    click(window, gallery.get_dialog_default_trigger());
    key(window, Key::Tab);
    key(window, Key::Return);
    window.take_snapshot()?;
    check(
        !gallery.get_dialog_open() && gallery.get_dialog_result() == ContentDialogResult::Secondary,
        "a focused secondary command takes precedence over the default",
    )?;

    gallery.set_dialog_primary_enabled(false);
    click(window, gallery.get_dialog_default_trigger());
    window.take_snapshot()?;
    check(
        gallery.get_dialog_focused_command() == Secondary,
        "initial focus skips a disabled default",
    )?;
    key(window, Key::Return);
    window.take_snapshot()?;
    check(
        gallery.get_dialog_result() == ContentDialogResult::Secondary,
        "a disabled default is never invoked",
    )?;
    gallery.set_dialog_primary_enabled(true);

    click(window, gallery.get_dialog_no_default_trigger());
    window.take_snapshot()?;
    check(
        gallery.get_dialog_focused_command() == Primary,
        "no-default dialog focuses the first command",
    )?;
    key(window, Key::Escape);
    window.take_snapshot()?;
    check(
        gallery.get_dialog_no_default_trigger_focused(),
        "focus restores to the actual opener",
    )?;

    click(window, gallery.get_dialog_editor_trigger());
    window.take_snapshot()?;
    if !gallery.get_dialog_open() || !gallery.get_dialog_editor_focused() {
        return Err(format!(
            "custom slot should receive focus, got open {}, editor focus {}, and focused command {:?}",
            gallery.get_dialog_open(), gallery.get_dialog_editor_focused(), gallery.get_dialog_focused_command()
        ).into());
    }
    key(window, Key::Return);
    check(
        gallery.get_dialog_open() && gallery.get_dialog_editor_submissions() == 1,
        "a TextBox that handles Enter prevents the default command",
    )?;
    key(window, Key::Tab);
    key(window, Key::Tab);
    key(window, Key::Tab);
    window.take_snapshot()?;
    check(
        gallery.get_dialog_editor_focused(),
        "Tab wraps back to custom content",
    )?;
    key(window, Key::Escape);
    window.take_snapshot()?;
    check(
        !gallery.get_dialog_open() && gallery.get_dialog_editor_trigger_focused(),
        "Esc from a custom field restores the opener",
    )
}

pub(super) fn verify_compact(gallery: &GalleryWindow) -> SmokeResult {
    gallery.set_page(GalleryPage::ContentDialog);
    let window = gallery.window();
    let size = window.size().to_logical(window.scale_factor());
    for (theme, content, footer) in [
        (FluentColorScheme::Light, [0xff; 3], [0xf3; 3]),
        (FluentColorScheme::Dark, [0x2b; 3], [0x20; 3]),
        (FluentColorScheme::HighContrast, [0; 3], [0; 3]),
    ] {
        gallery.invoke_select_theme(theme);
        window.take_snapshot()?;
        click(window, gallery.get_dialog_default_trigger());
        check(gallery.get_dialog_open(), "the compact ContentDialog opens")?;
        check_centered(gallery, size)?;
        check_surface(gallery, content, footer)?;
        key(window, Key::Escape);
        window.take_snapshot()?;
        check(
            gallery.get_dialog_default_trigger_focused(),
            "compact dialog restores focus in each theme",
        )?;
    }
    Ok(())
}

pub(super) fn verify_motion(gallery: &GalleryWindow, finish: impl FnOnce(SmokeResult) + 'static) {
    use std::time::Duration;
    gallery.set_page(GalleryPage::ContentDialog);
    gallery.invoke_select_theme(FluentColorScheme::Light);
    gallery.global::<FluentTheme>().set_reduced_motion(false);
    let window = gallery.window();
    let completions = gallery.get_dialog_completions();
    click(window, gallery.get_dialog_default_trigger());
    let weak = gallery.as_weak();
    slint::Timer::single_shot(Duration::from_millis(300), move || {
        let Some(gallery) = weak.upgrade() else {
            finish(Err("Gallery closed while opening ContentDialog".into()));
            return;
        };
        let result = check(
            gallery.get_dialog_open() && gallery.get_motion_enabled(),
            "the animated ContentDialog opens",
        )
        .and_then(|()| check_surface(&gallery, [0xff; 3], [0xf3; 3]));
        if let Err(error) = result {
            finish(Err(error));
            return;
        }
        let window = gallery.window();
        key(window, Key::Escape);
        click(window, LogicalPosition::new(2., 2.));
        if !gallery.get_dialog_open() || gallery.get_dialog_completions() != completions {
            finish(Err(
                "ContentDialog must stay modal until its exit transition completes".into(),
            ));
            return;
        }
        slint::Timer::single_shot(Duration::from_millis(220), move || {
            let result = weak
                .upgrade()
                .ok_or_else(|| "Gallery closed during ContentDialog exit".into())
                .and_then(|gallery| {
                    check(
                        !gallery.get_dialog_open()
                            && gallery.get_dialog_completions() == completions + 1
                            && gallery.get_dialog_default_trigger_focused(),
                        "animated dismissal completes once and restores focus",
                    )
                });
            finish(result);
        });
    });
}

fn check_centered(gallery: &GalleryWindow, size: LogicalSize) -> SmokeResult {
    let panel = gallery.get_dialog_bounds();
    check(
        panel.x >= 0.
            && panel.y >= 0.
            && (panel.x + panel.width / 2. - size.width / 2.).abs() <= 1.
            && (panel.y + panel.height / 2. - size.height / 2.).abs() <= 1.,
        "ContentDialog stays centered within the available viewport",
    )
}

fn check_surface(gallery: &GalleryWindow, content: [u8; 3], footer: [u8; 3]) -> SmokeResult {
    let panel = gallery.get_dialog_bounds();
    let window = gallery.window();
    let actual_content = read_rgb(window, LogicalPosition::new(panel.x + 12., panel.y + 16.))?;
    let actual_footer = read_rgb(
        window,
        LogicalPosition::new(panel.x + 12., panel.y + panel.height - 12.),
    )?;
    if actual_content != content || actual_footer != footer {
        return Err(format!("expected dialog content {content:?} and footer {footer:?}, got {actual_content:?} and {actual_footer:?} for {panel:?}").into());
    }
    Ok(())
}
