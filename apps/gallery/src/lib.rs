#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "ios",
    target_os = "android"
)))]
compile_error!("Fluent RS Gallery supports Windows, macOS, iOS, and Android.");

use slint::ComponentHandle;
#[cfg(target_os = "windows")]
use std::{cell::Cell, rc::Rc};

slint::include_modules!();

mod radio_navigation;
#[cfg(not(target_os = "android"))]
mod smoke;
#[cfg(target_os = "windows")]
mod windows_backdrop;

fn create_gallery() -> Result<GalleryWindow, slint::PlatformError> {
    let gallery = GalleryWindow::new()?;
    gallery.set_touch_targets(cfg!(any(target_os = "android", target_os = "ios")));
    if cfg!(target_os = "windows") {
        gallery.set_font_family("Segoe UI".into());
    }
    connect_appearance(&gallery);
    gallery
        .global::<RadioNavigation>()
        .on_target(radio_navigation::target);
    Ok(gallery)
}

fn connect_appearance(gallery: &GalleryWindow) {
    #[cfg(target_os = "windows")]
    let previous_dark = Rc::new(Cell::new(None));
    #[cfg(target_os = "windows")]
    let theme_frame = Rc::clone(&previous_dark);
    let weak = gallery.as_weak();
    gallery.on_select_theme(move |scheme| {
        if let Some(gallery) = weak.upgrade() {
            gallery.global::<FluentTheme>().set_scheme(scheme);
            #[cfg(target_os = "windows")]
            sync_desktop_acrylic(&gallery, &theme_frame);
        }
    });

    let weak = gallery.as_weak();
    gallery.on_select_backdrop(move |material| {
        if let Some(gallery) = weak.upgrade() {
            gallery.set_backdrop(material);
            #[cfg(target_os = "windows")]
            sync_desktop_acrylic(&gallery, &previous_dark);
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

    let gallery = create_gallery()?;
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
