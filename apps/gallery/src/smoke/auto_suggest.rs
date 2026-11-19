use slint::{
    ComponentHandle, LogicalPosition, Model, SharedString, Window,
    platform::{Key, PointerEventButton, WindowEvent},
};

use super::{check, click, read_rgb};
use crate::{
    AutoSuggestionTextChangeReason, FluentColorScheme, FluentTheme, GalleryPage, GalleryWindow,
    ItemData, list_demo,
};

type SmokeResult = Result<(), Box<dyn std::error::Error>>;

pub(super) fn verify(gallery: &GalleryWindow) -> SmokeResult {
    gallery.set_page(GalleryPage::AutoSuggestBox);
    gallery.set_suggestion_enabled(true);
    gallery.set_suggestion_query_icon(true);
    gallery.set_suggestion_update_text(true);
    gallery.invoke_select_theme(FluentColorScheme::Light);
    let window = gallery.window();
    window.take_snapshot()?;
    check(
        !gallery.get_suggestion_probe().open,
        "AutoSuggestBox starts with its suggestions closed",
    )?;

    type_query(gallery, "box")?;
    check(
        gallery.get_suggestion_reason() == AutoSuggestionTextChangeReason::UserInput
            && gallery.get_suggestion_items().row_count() == 5
            && gallery.get_suggestion_probe().open
            && gallery.get_suggestion_probe().selected == -1,
        "user input filters the caller-owned model and opens an unselected list",
    )?;

    for scheme in [
        FluentColorScheme::Light,
        FluentColorScheme::Dark,
        FluentColorScheme::HighContrast,
    ] {
        gallery.invoke_select_theme(scheme);
        window.take_snapshot()?;
        verify_geometry(gallery)?;
        let panel = gallery.get_suggestion_probe().popup;
        let fill = gallery.global::<FluentTheme>().get_acrylic_fallback();
        let actual = read_rgb(
            window,
            LogicalPosition::new(panel.x + panel.width - 25., panel.y + 10.),
        )?;
        check(
            actual == [fill.red(), fill.green(), fill.blue()],
            "the suggestion list uses the current themed surface",
        )?;
        let input = gallery.get_suggestion_probe().input;
        let seam_y = if panel.y < input.y {
            panel.y + panel.height - 3.
        } else {
            panel.y + 3.
        };
        check(
            read_rgb(window, LogicalPosition::new(panel.x + 3., seam_y))?
                == [fill.red(), fill.green(), fill.blue()],
            "the popup corner meeting the editor is square",
        )?;
    }
    gallery.invoke_select_theme(FluentColorScheme::Light);
    let original_size = window.size().to_logical(window.scale_factor());
    let input = gallery.get_suggestion_probe().input;
    window.dispatch_event(WindowEvent::Resized {
        size: slint::LogicalSize::new(original_size.width, input.y + input.height + 12.),
    });
    window.take_snapshot()?;
    gallery.invoke_show_suggestions();
    window.take_snapshot()?;
    let above = gallery.get_suggestion_probe();
    check(
        above.open
            && above.popup.y < above.input.y
            && (above.popup.y + above.popup.height - above.input.y).abs() < 1.,
        "a short viewport opens the suggestions above the editor",
    )?;
    let fill = gallery.global::<FluentTheme>().get_acrylic_fallback();
    check(
        read_rgb(
            window,
            LogicalPosition::new(above.popup.x + 3., above.popup.y + above.popup.height - 3.),
        )? == [fill.red(), fill.green(), fill.blue()],
        "the upward popup has a square bottom seam",
    )?;
    window.dispatch_event(WindowEvent::Resized {
        size: original_size,
    });
    window.take_snapshot()?;
    gallery.invoke_show_suggestions();
    key(window, Key::DownArrow);
    check(
        gallery.get_suggestion_text() == "AutoSuggestBox"
            && gallery.get_suggestion_probe().selected == 0
            && gallery.get_suggestion_reason() == AutoSuggestionTextChangeReason::SuggestionChosen,
        "Down previews the first suggestion with the selection change reason",
    )?;
    key(window, Key::UpArrow);
    check(
        gallery.get_suggestion_text() == "box" && gallery.get_suggestion_probe().selected == -1,
        "Up beyond the first suggestion restores the typed query",
    )?;
    key(window, Key::UpArrow);
    check(
        gallery.get_suggestion_text() == "TextBox",
        "Up from the query previews the final suggestion",
    )?;
    key(window, Key::DownArrow);
    check(
        gallery.get_suggestion_text() == "box",
        "Down beyond the last suggestion restores the query",
    )?;
    key(window, Key::DownArrow);
    key(window, Key::Escape);
    window.take_snapshot()?;
    check(
        !gallery.get_suggestion_probe().open
            && gallery.get_suggestion_probe().focused
            && gallery.get_suggestion_text() == "box",
        "Escape closes suggestions and restores the user's text and editor focus",
    )?;

    type_query(gallery, "pro")?;
    key(window, Key::DownArrow);
    let submissions = gallery.get_suggestion_submissions();
    key(window, Key::Return);
    window.take_snapshot()?;
    check(
        gallery.get_suggestion_submissions() == submissions + 1
            && gallery.get_suggestion_submitted_text() == "ProgressBar"
            && gallery.get_suggestion_submitted_index() == 0
            && !gallery.get_suggestion_probe().open,
        "Enter submits the previewed item once and closes the list",
    )?;

    type_query(gallery, "box")?;
    key(window, Key::DownArrow);
    let probe = gallery.get_suggestion_probe();
    let before_cancel = gallery.get_suggestion_submissions();
    window.dispatch_event(WindowEvent::PointerMoved {
        position: probe.query,
    });
    let swatch = LogicalPosition::new(probe.query.x + 10., probe.query.y + 6.);
    let hover = read_rgb(window, swatch)?;
    window.dispatch_event(WindowEvent::PointerPressed {
        position: probe.query,
        button: PointerEventButton::Left,
    });
    let pressed = read_rgb(window, swatch)?;
    check(
        hover != pressed,
        "the inline query button paints its hover and pressed states",
    )?;
    let outside =
        LogicalPosition::new(probe.input.x + 10., probe.input.y + probe.input.height / 2.);
    window.dispatch_event(WindowEvent::PointerMoved { position: outside });
    window.dispatch_event(WindowEvent::PointerReleased {
        position: outside,
        button: PointerEventButton::Left,
    });
    check(
        gallery.get_suggestion_submissions() == before_cancel,
        "dragging off the inline query button cancels submission",
    )?;
    click(window, probe.query);
    window.take_snapshot()?;
    check(
        gallery.get_suggestion_submitted_text() == "AutoSuggestBox"
            && gallery.get_suggestion_submitted_index() == -1
            && !gallery.get_suggestion_probe().open,
        "the inline search button works with suggestions open and submits only text",
    )?;

    type_query(gallery, "box")?;
    let changes = gallery.get_suggestion_changes();
    click(window, gallery.get_suggestion_probe().clear);
    window.take_snapshot()?;
    check(
        gallery.get_suggestion_text().is_empty()
            && gallery.get_suggestion_reason() == AutoSuggestionTextChangeReason::UserInput
            && gallery.get_suggestion_changes() == changes + 1
            && gallery.get_suggestion_probe().focused,
        "the inline clear action raises one user edit and preserves the editor",
    )?;

    type_query(gallery, "ring")?;
    key(window, Key::DownArrow);
    key(window, Key::UpArrow);
    check(
        gallery.get_suggestion_text() == "ProgressRing"
            && gallery.get_suggestion_probe().selected == 0,
        "a single selected suggestion stays selected for either arrow",
    )?;
    key(window, Key::Tab);
    window.take_snapshot()?;
    check(
        gallery.get_suggestion_text() == "ring"
            && !gallery.get_suggestion_probe().open
            && gallery.get_suggestion_following_focused(),
        "Tab restores typed text and advances past the inline actions",
    )?;

    gallery.set_suggestion_text("中文查询".into());
    window.take_snapshot()?;
    check(
        gallery.get_suggestion_reason() == AutoSuggestionTextChangeReason::ProgrammaticChange
            && !gallery.get_suggestion_probe().open,
        "programmatic Unicode text does not refilter or reopen the list",
    )?;
    type_query(gallery, "no matching control")?;
    check(
        gallery.get_suggestion_items().row_count() == 0 && !gallery.get_suggestion_probe().open,
        "empty results close the suggestion list",
    )?;

    type_query(gallery, "box")?;
    gallery.set_suggestion_update_text(false);
    key(window, Key::DownArrow);
    check(
        gallery.get_suggestion_text() == "box" && gallery.get_suggestion_probe().selected == 0,
        "UpdateTextOnSelect can preserve the typed text",
    )?;
    key(window, Key::Return);
    check(
        gallery.get_suggestion_submitted_text() == "box"
            && gallery.get_suggestion_submitted_index() == 0,
        "submission carries the chosen item even when selection leaves text unchanged",
    )?;
    gallery.set_suggestion_update_text(true);

    gallery.set_suggestion_items(slint::ModelRc::new(slint::VecModel::from(vec![
        ItemData {
            text: "Disabled".into(),
            disabled: true,
        },
        ItemData {
            text: "可选项".into(),
            disabled: false,
        },
    ])));
    gallery.invoke_show_suggestions();
    key(window, Key::DownArrow);
    check(
        gallery.get_suggestion_probe().selected == 1 && gallery.get_suggestion_text() == "可选项",
        "suggestion navigation skips disabled entries and handles Unicode text",
    )?;
    gallery.set_suggestion_items(Default::default());
    window.take_snapshot()?;
    check(
        !gallery.get_suggestion_probe().open && gallery.get_suggestion_probe().selected == -1,
        "replacing the open model with an empty model clears the stale preview",
    )?;
    gallery.set_suggestion_items(slint::ModelRc::new(slint::VecModel::from(vec![ItemData {
        text: "Later result".into(),
        disabled: false,
    }])));
    window.take_snapshot()?;
    check(
        gallery.get_suggestion_probe().open && gallery.get_suggestion_probe().selected == -1,
        "a later result model reopens an empty result list while editing continues",
    )?;
    gallery.invoke_dismiss_suggestions();

    gallery.set_suggestion_query_icon(false);
    type_query(gallery, "box")?;
    let probe = gallery.get_suggestion_probe();
    let row_height = 40. * gallery.global::<FluentTheme>().get_ui_scale();
    click(
        window,
        LogicalPosition::new(probe.popup.x + 40., probe.popup.y + 3. + row_height / 2.),
    );
    window.take_snapshot()?;
    check(
        gallery.get_suggestion_submitted_index() == 0 && !gallery.get_suggestion_probe().open,
        "clicking a suggestion submits it when the query icon is absent",
    )?;
    gallery.set_suggestion_query_icon(true);

    let (items, reads) = list_demo::items();
    gallery.set_suggestion_items(items);
    gallery.invoke_show_suggestions();
    window.take_snapshot()?;
    let probe = gallery.get_suggestion_probe();
    for _ in 0..12 {
        window.dispatch_event(WindowEvent::PointerScrolled {
            position: LogicalPosition::new(
                probe.popup.x + probe.popup.width - 6.,
                probe.popup.y + probe.popup.height / 2.,
            ),
            delta_x: 0.,
            delta_y: -40.,
        });
        window.take_snapshot()?;
    }
    key(window, Key::UpArrow);
    window.take_snapshot()?;
    check(
        gallery.get_suggestion_probe().selected == 99_999 && reads.get() < 500,
        "the shared virtual list seeks the final suggestion without materializing the full model",
    )?;
    println!(
        "AutoSuggestBox verified at {}x scale; virtual model made {} row-data reads",
        window.scale_factor(),
        reads.get()
    );
    gallery.set_suggestion_enabled(false);
    window.take_snapshot()?;
    gallery.invoke_show_suggestions();
    check(
        !gallery.get_suggestion_probe().open && !gallery.get_suggestion_probe().focused,
        "disabling the input closes suggestions and releases editor focus",
    )?;
    gallery.set_suggestion_enabled(true);
    gallery.set_suggestion_items(Default::default());
    gallery.set_suggestion_text("".into());
    Ok(())
}

pub(super) fn verify_compact(gallery: &GalleryWindow) -> SmokeResult {
    gallery.set_page(GalleryPage::AutoSuggestBox);
    type_query(gallery, "box")?;
    verify_geometry(gallery)?;
    check(
        gallery.get_suggestion_probe().input.height >= 44.,
        "compact suggestions retain a touch-sized editor",
    )?;
    gallery.set_page(GalleryPage::Button);
    gallery.window().take_snapshot()?;
    check(
        !gallery.get_suggestion_probe().open,
        "leaving the page dismisses suggestions",
    )
}

fn verify_geometry(gallery: &GalleryWindow) -> SmokeResult {
    let probe = gallery.get_suggestion_probe();
    let size = gallery
        .window()
        .size()
        .to_logical(gallery.window().scale_factor());
    let touching_below = (probe.popup.y - probe.input.y - probe.input.height).abs() < 1.;
    let touching_above = (probe.popup.y + probe.popup.height - probe.input.y).abs() < 1.;
    check(
        probe.open
            && (touching_above || touching_below)
            && (probe.popup.x - probe.input.x).abs() < 1.
            && (probe.popup.width - probe.input.width).abs() < 1.
            && probe.popup.x >= 0.
            && probe.popup.y >= 0.
            && probe.popup.x + probe.popup.width <= size.width + 1.
            && probe.popup.y + probe.popup.height <= size.height + 1.,
        "the suggestions adjoin the editor on the available side and fit the viewport",
    )
}

fn type_query(gallery: &GalleryWindow, query: &str) -> SmokeResult {
    gallery.invoke_dismiss_suggestions();
    gallery.invoke_select_suggestion_input();
    for character in query.chars() {
        key(gallery.window(), character.to_string());
        gallery.window().take_snapshot()?;
    }
    Ok(())
}

fn key(window: &Window, text: impl Into<SharedString>) {
    let text = text.into();
    window.dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    window.dispatch_event(WindowEvent::KeyReleased { text });
}
