use objc2::{MainThreadMarker, MainThreadOnly, rc::Retained};
use objc2_app_kit::{
    NSAppearance, NSAppearanceCustomization, NSAppearanceNameAqua, NSAppearanceNameDarkAqua,
    NSAutoresizingMaskOptions, NSView, NSVisualEffectBlendingMode, NSVisualEffectMaterial,
    NSVisualEffectState, NSVisualEffectView, NSWindowOrderingMode, NSWorkspace,
};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

#[derive(Default)]
pub(super) struct DesktopAcrylic {
    view: Option<Retained<NSVisualEffectView>>,
}

pub(super) fn supported() -> bool {
    !NSWorkspace::sharedWorkspace().accessibilityDisplayShouldReduceTransparency()
}

impl DesktopAcrylic {
    pub(super) fn apply(&mut self, window: &slint::Window, enable: bool, dark: bool) -> bool {
        if enable && supported() {
            match self.install(window, dark) {
                Ok(()) => return true,
                Err(error) => eprintln!("Desktop Acrylic: {error}"),
            }
        }
        self.remove();
        false
    }

    fn install(&mut self, window: &slint::Window, dark: bool) -> Result<(), &'static str> {
        let main_thread = MainThreadMarker::new().ok_or("AppKit requires the main thread")?;
        let content = content_view(window).ok_or("no AppKit content view")?;
        let _window = content.window().ok_or("content view has no window")?;
        // SAFETY: The retained content view is attached to a live main-thread window.
        let parent = unsafe { content.superview() }.ok_or("content view has no backing view")?;
        let appearance = appearance(dark).ok_or("AppKit appearance is unavailable")?;

        let view = self.view.get_or_insert_with(|| {
            let view = NSVisualEffectView::initWithFrame(
                NSVisualEffectView::alloc(main_thread),
                content.frame(),
            );
            view.setAutoresizingMask(
                NSAutoresizingMaskOptions::ViewWidthSizable
                    | NSAutoresizingMaskOptions::ViewHeightSizable,
            );
            view.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
            // Desktop Acrylic is a transient material. Popover also filters the
            // backdrop more smoothly than UnderWindowBackground on macOS.
            view.setMaterial(NSVisualEffectMaterial::Popover);
            view.setState(NSVisualEffectState::FollowsWindowActiveState);
            view
        });
        view.setAppearance(Some(&appearance));
        // SAFETY: The AppKit hierarchy is alive and only accessed on the main thread.
        if unsafe { view.superview() }.as_deref() != Some(&*parent) {
            view.setFrame(content.frame());
            // Keep winit's content view intact, with its Metal surface and input handling above the effect.
            parent.addSubview_positioned_relativeTo(
                view,
                NSWindowOrderingMode::Below,
                Some(&content),
            );
        }
        Ok(())
    }

    fn remove(&mut self) {
        if let Some(view) = self.view.take() {
            view.removeFromSuperview();
        }
    }
}

impl Drop for DesktopAcrylic {
    fn drop(&mut self) {
        self.remove();
    }
}

fn appearance(dark: bool) -> Option<Retained<NSAppearance>> {
    // SAFETY: Both are AppKit-owned appearance names available on the supported macOS versions.
    let name = unsafe {
        if dark {
            NSAppearanceNameDarkAqua
        } else {
            NSAppearanceNameAqua
        }
    };
    NSAppearance::appearanceNamed(name)
}

fn content_view(window: &slint::Window) -> Option<Retained<NSView>> {
    let _main_thread = MainThreadMarker::new()?;
    let provider = window.window_handle();
    let handle = provider.window_handle().ok()?;
    let RawWindowHandle::AppKit(handle) = handle.as_raw() else {
        return None;
    };
    // SAFETY: AppKitWindowHandle provides an NSView valid while the handle is borrowed.
    // Retain it before the window handle and its provider are dropped.
    unsafe { Retained::retain(handle.ns_view.as_ptr().cast::<NSView>()) }
}
