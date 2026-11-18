use std::{cell::Cell, mem::size_of, ptr};

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::{
    Wdk::System::SystemServices::RtlGetVersion,
    Win32::{
        Foundation::HWND,
        Graphics::Dwm::{
            DWMSBT_AUTO, DWMSBT_TRANSIENTWINDOW, DWMWA_SYSTEMBACKDROP_TYPE,
            DWMWA_USE_IMMERSIVE_DARK_MODE, DwmExtendFrameIntoClientArea, DwmGetWindowAttribute,
            DwmIsCompositionEnabled, DwmSetWindowAttribute,
        },
        System::SystemInformation::OSVERSIONINFOW,
        UI::Controls::MARGINS,
    },
};

const DESKTOP_ACRYLIC_MIN_BUILD: u32 = 22621;

pub(super) fn supported() -> bool {
    let mut version = OSVERSIONINFOW {
        dwOSVersionInfoSize: size_of::<OSVERSIONINFOW>() as u32,
        ..Default::default()
    };
    let mut composition = 0;
    // SAFETY: Both out parameters are initialized, writable locals with the Win32 ABI.
    unsafe {
        RtlGetVersion(&mut version) >= 0
            && version.dwBuildNumber >= DESKTOP_ACRYLIC_MIN_BUILD
            && DwmIsCompositionEnabled(&mut composition) >= 0
            && composition != 0
    }
}

pub(super) fn apply(
    window: &slint::Window,
    previous_dark: &Cell<Option<i32>>,
    enable: bool,
    dark: bool,
) -> bool {
    let enable = enable && supported();
    if !enable && previous_dark.get().is_none() {
        return false;
    }
    let handle_provider = window.window_handle();
    let Ok(handle) = handle_provider.window_handle() else {
        eprintln!("Desktop Acrylic: no native window handle");
        return false;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return false;
    };
    // Keep the window handle provider alive until all DWM calls have finished.
    let hwnd = handle.hwnd.get() as HWND;
    if enable {
        if previous_dark.get().is_none() {
            let mut previous = 0i32;
            // SAFETY: The live HWND is on the UI thread and the destination is a Win32 BOOL.
            let result = unsafe {
                DwmGetWindowAttribute(
                    hwnd,
                    DWMWA_USE_IMMERSIVE_DARK_MODE as u32,
                    ptr::from_mut(&mut previous).cast(),
                    size_of::<i32>() as u32,
                )
            };
            if result < 0 {
                eprintln!("Desktop Acrylic: reading frame theme failed (HRESULT 0x{result:08x})");
                return false;
            }
            previous_dark.set(Some(previous));
        }
        let mode = i32::from(dark);
        let backdrop = DWMSBT_TRANSIENTWINDOW;
        let margins = MARGINS {
            cxLeftWidth: -1,
            cxRightWidth: -1,
            cyTopHeight: -1,
            cyBottomHeight: -1,
        };
        // SAFETY: This runs on the UI thread. The live HWND belongs to the Gallery,
        // and each pointer is to a local with the ABI and size of its DWM attribute.
        let result = unsafe {
            let theme = set_attribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE, &mode);
            if theme < 0 {
                theme
            } else {
                let frame = DwmExtendFrameIntoClientArea(hwnd, &margins);
                if frame < 0 {
                    frame
                } else {
                    set_attribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE, &backdrop)
                }
            }
        };
        if result >= 0 {
            return true;
        }
        eprintln!("Desktop Acrylic: DWM activation failed (HRESULT 0x{result:08x})");
    }

    if let Some(previous) = previous_dark.take() {
        // SAFETY: The same live HWND is used to restore the frame and backdrop.
        let (frame, backdrop, theme) = unsafe {
            (
                DwmExtendFrameIntoClientArea(hwnd, &MARGINS::default()),
                set_attribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE, &DWMSBT_AUTO),
                set_attribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE, &previous),
            )
        };
        if frame < 0 || backdrop < 0 || theme < 0 {
            eprintln!(
                "Desktop Acrylic: frame restore returned HRESULTs 0x{frame:08x}, 0x{backdrop:08x}, 0x{theme:08x}"
            );
        }
    }
    false
}

unsafe fn set_attribute(hwnd: HWND, attribute: i32, value: &i32) -> i32 {
    // SAFETY: The caller guarantees HWND validity and a 32-bit payload for this attribute.
    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            attribute as u32,
            ptr::from_ref(value).cast(),
            size_of::<i32>() as u32,
        )
    }
}
