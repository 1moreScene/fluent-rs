//! Gallery URI host integration, independent of the control's presentation.

#[cfg(target_os = "macos")]
pub(super) fn open(uri: &str) -> Result<(), String> {
    use objc2_app_kit::NSWorkspace;
    use objc2_foundation::{NSString, NSURL};

    let url = NSURL::URLWithString(&NSString::from_str(uri))
        .ok_or_else(|| "The link address is invalid.".to_owned())?;
    if NSWorkspace::sharedWorkspace().openURL(&url) {
        Ok(())
    } else {
        Err("The link could not be opened. Check your default application.".to_owned())
    }
}

#[cfg(not(target_os = "macos"))]
pub(super) fn open(_uri: &str) -> Result<(), String> {
    Err("Opening external links is not configured for this Gallery host yet.".to_owned())
}
