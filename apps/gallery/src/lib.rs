#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "ios",
    target_os = "android"
)))]
compile_error!("Fluent RS Gallery supports Windows, macOS, iOS, and Android.");

use slint::{ComponentHandle, Model, ModelRc, VecModel};
#[cfg(target_os = "windows")]
use std::cell::Cell;
#[cfg(target_os = "macos")]
use std::cell::RefCell;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use std::rc::Rc;

slint::include_modules!();

mod acrylic;
mod animated_icon;
mod auto_suggest;
mod chevron_tracks;
mod combo_popup;
mod combo_text;
mod command_bar;
mod frame_journal;
mod grid_navigation;
mod list_demo;
#[cfg(target_os = "macos")]
mod macos_backdrop;
mod navigation_tree;
mod number_person;
#[cfg(target_os = "macos")]
mod page_memory;
mod radio_navigation;
mod selection;
#[cfg(not(target_os = "android"))]
mod smoke;
mod top_navigation;
mod tree_view;
mod uri_launcher;
#[cfg(target_os = "windows")]
mod windows_backdrop;

fn create_gallery() -> Result<GalleryWindow, slint::PlatformError> {
    let gallery = GalleryWindow::new()?;
    number_person::connect(&gallery);
    gallery
        .global::<UriLauncher>()
        .on_open(|uri| match uri_launcher::open(&uri) {
            Ok(()) => slint::SharedString::default(),
            Err(error) => error.into(),
        });
    gallery.set_touch_targets(cfg!(any(target_os = "android", target_os = "ios")));
    if cfg!(target_os = "macos") {
        gallery
            .global::<FluentTheme>()
            .set_password_mask_font_family("Helvetica Neue".into());
        gallery.global::<FluentTheme>().set_ui_scale(1.25);
        gallery
            .window()
            .set_size(slint::LogicalSize::new(980., 700.));
    }
    gallery.set_tint_native_desktop_acrylic(cfg!(target_os = "macos"));
    if cfg!(target_os = "windows") {
        gallery.set_font_family("Segoe UI".into());
    }
    #[cfg(target_os = "macos")]
    gallery
        .global::<FluentTheme>()
        .set_reduce_transparency(!macos_backdrop::supported());
    connect_appearance(&gallery);
    let routes = gallery.global::<GalleryRoutes>();
    routes.on_index(|destinations, page| {
        destinations
            .iter()
            .position(|entry| !entry.title.is_empty() && entry.page == page)
            .map_or(-1, |index| index as i32)
    });
    gallery
        .global::<RadioNavigation>()
        .on_target(radio_navigation::target);
    gallery
        .global::<FrameJournal>()
        .on_append(frame_journal::append);
    let tree = gallery.global::<NavigationTree>();
    tree.on_details(navigation_tree::details);
    tree.on_has_children(navigation_tree::has_children);
    tree.on_visible(navigation_tree::visible);
    tree.on_ancestor(navigation_tree::ancestor);
    tree.on_rows(navigation_tree::rows);
    let top = gallery.global::<TopNavigationLayout>();
    top.on_measure(top_navigation::measure);
    top.on_arrange(top_navigation::arrange);
    top.on_step(top_navigation::step);
    top.on_indices(top_navigation::indices);
    top.on_position(top_navigation::position);
    top.on_slot(top_navigation::slot);
    top.on_visible_items(top_navigation::visible_items);
    let nodes = gallery.global::<TreeLogic>();
    nodes.on_items(tree_view::items);
    nodes.on_project(tree_view::project);
    nodes.on_visible_selection(tree_view::visible_selection);
    nodes.on_visible_ancestor(tree_view::visible_ancestor);
    nodes.on_toggle_check(tree_view::toggle_check);
    gallery.on_load_large_tree(tree_view::large_demo);
    let commands = gallery.global::<CommandBarLogic>();
    commands.on_arrange(command_bar::arrange);
    commands.on_menu(command_bar::menu);
    let weak = gallery.as_weak();
    gallery
        .global::<AcrylicBackdrop>()
        .on_capture(move |request| {
            weak.upgrade()
                .and_then(|gallery| acrylic::capture(gallery.window(), request))
                .unwrap_or_default()
        });
    let icon = gallery.global::<AnimatedIconLogic>();
    icon.on_segment(animated_icon::segment);
    icon.on_queue(animated_icon::queue);
    icon.on_skip_first(animated_icon::skip_first);
    let chevron = gallery.global::<ChevronAnimation>();
    chevron.on_pose(animated_icon::chevron_pose);
    chevron.set_source_duration(animated_icon::DURATION_MS as i64);
    chevron.set_markers(animated_icon::markers());
    gallery
        .global::<SplitChevronMotion>()
        .on_offset(animated_icon::split_chevron_offset);
    gallery
        .global::<GridNavigation>()
        .on_step(grid_navigation::step);
    let selection = gallery.global::<SelectionLogic>();
    gallery
        .global::<AutoSuggestText>()
        .on_byte_length(auto_suggest::byte_length);
    gallery.on_filter_suggestions(auto_suggest::filter);
    selection.on_target(selection::target);
    selection.on_contains(selection::contains);
    selection.on_update(selection::update);
    selection.on_range(selection::range);
    selection.on_equal(selection::equal);
    let popup_layout = gallery.global::<ComboPopupLayout>();
    popup_layout.on_arrange(combo_popup::arrange);
    popup_layout.on_arrange_editable(combo_popup::arrange_editable);
    let text_search = gallery.global::<ComboTextSearch>();
    text_search.on_find(combo_text::find);
    text_search.on_contains_text(combo_text::contains_text);
    let (items, _) = list_demo::items();
    gallery.set_composition_items(items);
    gallery.set_shell_items(ModelRc::new(VecModel::from(
        gallery
            .get_destinations()
            .iter()
            .map(|entry| ItemData {
                text: if entry.label.is_empty() {
                    entry.title
                } else {
                    entry.label
                },
                disabled: false,
            })
            .collect::<Vec<_>>(),
    )));
    Ok(gallery)
}

fn connect_appearance(gallery: &GalleryWindow) {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    let backdrop_state = Rc::new(Default::default());
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    let theme_frame = Rc::clone(&backdrop_state);
    let weak = gallery.as_weak();
    gallery.on_select_theme(move |scheme| {
        if let Some(gallery) = weak.upgrade() {
            gallery.global::<FluentTheme>().set_scheme(scheme);
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            sync_desktop_acrylic(&gallery, &theme_frame);
        }
    });

    let weak = gallery.as_weak();
    gallery.on_select_backdrop(move |material| {
        if let Some(gallery) = weak.upgrade() {
            gallery.set_backdrop(material);
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            sync_desktop_acrylic(&gallery, &backdrop_state);
        }
    });

    let theme = gallery.global::<FluentTheme>();
    let default_accents = (theme.get_light_accent(), theme.get_dark_accent());
    let weak = gallery.as_weak();
    gallery.on_change_appearance(move |action| {
        let Some(gallery) = weak.upgrade() else {
            return;
        };
        let theme = gallery.global::<FluentTheme>();
        match action {
            AppearanceAction::ToggleMotion => theme.set_reduced_motion(!theme.get_reduced_motion()),
            AppearanceAction::ToggleTextScale => {
                theme.set_text_scale(if theme.get_text_scale() > 1. {
                    1.
                } else {
                    1.25
                });
            }
            AppearanceAction::ToggleAccent => {
                let alternate = !gallery.get_alternate_accent();
                let (light, dark) = if alternate {
                    (
                        slint::Color::from_rgb_u8(0x57, 0x2b, 0xa1),
                        slint::Color::from_rgb_u8(0xd0, 0xb0, 0xff),
                    )
                } else {
                    default_accents
                };
                theme.set_light_accent(light);
                theme.set_dark_accent(dark);
                gallery.set_alternate_accent(alternate);
            }
        }
    });
}

#[cfg(target_os = "macos")]
fn sync_desktop_acrylic(gallery: &GalleryWindow, state: &RefCell<macos_backdrop::DesktopAcrylic>) {
    let scheme = gallery.global::<FluentTheme>().get_scheme();
    let enable = gallery.get_backdrop() == FluentMaterial::DesktopAcrylic
        && scheme != FluentColorScheme::HighContrast;
    let active =
        state
            .borrow_mut()
            .apply(gallery.window(), enable, scheme == FluentColorScheme::Dark);
    gallery.set_native_desktop_acrylic(active);
}

#[cfg(target_os = "windows")]
fn sync_desktop_acrylic(gallery: &GalleryWindow, previous_dark: &Cell<Option<i32>>) {
    let scheme = gallery.global::<FluentTheme>().get_scheme();
    if gallery.get_backdrop() == FluentMaterial::DesktopAcrylic
        && scheme != FluentColorScheme::HighContrast
    {
        gallery.set_native_desktop_acrylic(windows_backdrop::apply(
            gallery.window(),
            previous_dark,
            true,
            scheme == FluentColorScheme::Dark,
        ));
    } else if gallery.get_native_desktop_acrylic() {
        gallery.set_native_desktop_acrylic(false);
        windows_backdrop::apply(gallery.window(), previous_dark, false, false);
    }
}

/// Starts the Gallery on the main thread, optionally checking input and Skia rendering.
#[cfg(not(target_os = "android"))]
pub fn run(smoke_test: bool) -> Result<(), Box<dyn std::error::Error>> {
    slint::BackendSelector::new()
        .backend_name("winit".into())
        .renderer_name("skia".into())
        .select()?;

    #[cfg(target_os = "macos")]
    if std::env::args_os().any(|arg| arg == "--renderer-memory-check") {
        return page_memory::verify_renderer();
    }
    let gallery = create_gallery()?;
    #[cfg(target_os = "macos")]
    if std::env::args_os().any(|arg| arg == "--command-bar-profile") {
        return page_memory::profile_command_bar(&gallery);
    }
    #[cfg(target_os = "macos")]
    if std::env::args_os().any(|arg| arg == "--acrylic-memory-check") {
        return page_memory::verify_acrylic(&gallery);
    }
    #[cfg(target_os = "macos")]
    if std::env::args_os()
        .skip(1)
        .any(|arg| arg == "--navigation-memory-check")
    {
        return page_memory::verify(&gallery);
    }
    if !smoke_test {
        return Ok(gallery.run()?);
    }
    smoke::verify(&gallery)
}

#[cfg(target_os = "android")]
// SAFETY: android-activity calls this exported Rust entry point with the initialized app handle.
#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) {
    let result = slint::android::init(app)
        .map_err(slint::PlatformError::SetPlatformError)
        .and_then(|()| create_gallery()?.run());
    if let Err(error) = result {
        eprintln!("Could not start Fluent RS Gallery: {error}");
    }
}
