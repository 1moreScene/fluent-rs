use slint::{ComponentHandle, LogicalPosition, Model, platform::Key};

use super::{check, click, overlays::key, read_rgb};
use crate::{
    FluentColorScheme, FluentTheme, GalleryPage, GalleryWindow, InfoBarCloseReason, InfoBarSample,
};

type SmokeResult = Result<(), Box<dyn std::error::Error>>;

pub(super) fn verify(gallery: &GalleryWindow) -> SmokeResult {
    gallery.set_page(GalleryPage::InfoBar);
    let window = gallery.window();
    let theme = gallery.global::<FluentTheme>();

    for (scheme, fills, accents) in [
        (
            FluentColorScheme::Light,
            [[245; 3], [223, 246, 221], [255, 244, 206], [253, 231, 233]],
            [
                rgb(theme.get_system_accent()),
                [15, 123, 15],
                [157, 93, 0],
                [196, 43, 28],
            ],
        ),
        (
            FluentColorScheme::Dark,
            [[39; 3], [57, 61, 27], [67, 53, 25], [68, 39, 38]],
            [
                rgb(theme.get_dark_accent()),
                [108, 203, 95],
                [252, 225, 0],
                [255, 153, 164],
            ],
        ),
        (
            FluentColorScheme::HighContrast,
            [rgb(theme.get_contrast_surface()); 4],
            [rgb(theme.get_contrast_highlight()); 4],
        ),
    ] {
        gallery.invoke_select_theme(scheme);
        window.take_snapshot()?;
        for (index, (fill, accent)) in fills.into_iter().zip(accents).enumerate() {
            let bar = sample(gallery, index)?;
            check(
                bar.shown && bar.bounds.height >= 48.,
                "all InfoBar levels are visible",
            )?;
            let actual_fill = read_rgb(
                window,
                LogicalPosition::new(bar.bounds.x + 8., bar.bounds.y + 10.),
            )?;
            let actual_accent =
                read_rgb(window, LogicalPosition::new(bar.icon.x + 5., bar.icon.y))?;
            if !near_color(actual_fill, fill) || !near_color(actual_accent, accent) {
                return Err(format!(
                    "InfoBar {scheme:?} row {index} fills should be {fill:?} / {accent:?}, got {actual_fill:?} / {actual_accent:?}"
                ).into());
            }
        }
    }
    gallery.invoke_select_theme(FluentColorScheme::Light);

    let short = sample(gallery, 0)?;
    check(
        !short.stacked,
        "short InfoBar content uses one row when there is room",
    )?;
    gallery.set_info_bar_long_message(true);
    window.take_snapshot()?;
    let long = sample(gallery, 0)?;
    check(
        long.stacked
            && long.message_height > short.message_height
            && long.bounds.height > short.bounds.height,
        "long InfoBar content wraps and expands the layout",
    )?;
    gallery.set_info_bar_long_message(false);
    window.take_snapshot()?;

    gallery.set_info_bar_prevent_dismissal(true);
    click(window, short.close);
    window.take_snapshot()?;
    check(
        sample(gallery, 0)?.shown
            && gallery.get_info_bar_close_clicks() == 1
            && gallery.get_info_bar_close_attempts() == 1
            && gallery.get_info_bar_closed_count() == 0,
        "canceling a close request leaves the banner visible",
    )?;
    gallery.set_info_bar_prevent_dismissal(false);

    for _ in 0..24 {
        if gallery.get_info_bar_close_focused() {
            break;
        }
        key(window, Key::Tab);
    }
    check(
        gallery.get_info_bar_close_focused(),
        "the close button is keyboard reachable",
    )?;
    key(window, Key::Escape);
    check(
        sample(gallery, 0)?.shown,
        "Escape leaves the nonmodal banner open",
    )?;
    let following = sample(gallery, 1)?;
    key(window, Key::Space);
    window.take_snapshot()?;
    let hidden = sample(gallery, 0)?;
    check(
        !hidden.shown
            && hidden.bounds.height <= 1.
            && sample(gallery, 1)?.bounds.y < following.bounds.y - 30.
            && !gallery.get_info_bar_close_focused()
            && gallery.get_info_bar_close_clicks() == 2
            && gallery.get_info_bar_closed_count() == 1
            && gallery.get_info_bar_close_reason() == InfoBarCloseReason::CloseButton,
        "keyboard close reports its reason, releases focus, and collapses the banner",
    )?;

    gallery.invoke_show_info_bars();
    gallery.set_info_bar_closable(false);
    window.take_snapshot()?;
    click(window, short.close);
    check(
        sample(gallery, 0)?.shown && gallery.get_info_bar_close_clicks() == 2,
        "a hidden close button cannot be clicked",
    )?;
    gallery.set_info_bar_prevent_dismissal(true);
    gallery.invoke_set_information_open(false);
    window.take_snapshot()?;
    check(
        sample(gallery, 0)?.shown && gallery.get_info_bar_closed_count() == 1,
        "programmatic close can also be canceled",
    )?;
    gallery.set_info_bar_prevent_dismissal(false);
    gallery.invoke_set_information_open(false);
    window.take_snapshot()?;
    check(
        !sample(gallery, 0)?.shown
            && gallery.get_info_bar_closed_count() == 2
            && gallery.get_info_bar_close_reason() == InfoBarCloseReason::Programmatic,
        "setting is-open closes a banner without a close button and reports programmatic closure",
    )?;
    gallery.invoke_set_information_open(false);
    window.take_snapshot()?;
    check(
        gallery.get_info_bar_closed_count() == 2,
        "an already closed banner emits no duplicate result",
    )?;
    gallery.set_info_bar_closable(true);
    gallery.invoke_show_info_bars();
    Ok(())
}

pub(super) fn verify_compact(gallery: &GalleryWindow, width: f32) -> SmokeResult {
    gallery.set_page(GalleryPage::InfoBar);
    gallery.invoke_select_theme(FluentColorScheme::Light);
    let window = gallery.window();
    window.take_snapshot()?;
    let short = sample(gallery, 0)?;
    gallery.set_info_bar_long_message(true);
    window.take_snapshot()?;
    let long = sample(gallery, 0)?;
    check(
        long.stacked
            && long.bounds.x >= 0.
            && long.bounds.x + long.bounds.width <= width
            && long.message_height >= short.message_height + 40.
            && long.bounds.height >= short.bounds.height + 40.
            && long.close.x < long.bounds.x + long.bounds.width,
        "the compact long message remains inside the surface and grows below its heading",
    )?;
    gallery.set_info_bar_long_message(false);
    Ok(())
}

fn sample(
    gallery: &GalleryWindow,
    index: usize,
) -> Result<InfoBarSample, Box<dyn std::error::Error>> {
    gallery
        .get_info_bar_samples()
        .row_data(index)
        .ok_or_else(|| "missing Gallery InfoBar sample".into())
}

fn rgb(color: slint::Color) -> [u8; 3] {
    [color.red(), color.green(), color.blue()]
}

fn near_color(actual: [u8; 3], expected: [u8; 3]) -> bool {
    actual
        .into_iter()
        .zip(expected)
        .all(|(a, b)| a.abs_diff(b) <= 1)
}
