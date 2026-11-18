#[cfg(not(target_os = "android"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let smoke_test = std::env::args_os().skip(1).any(|arg| arg == "--smoke-test");
    fluent_gallery::run(smoke_test)
}

// Android launches the shared library through NativeActivity.
#[cfg(target_os = "android")]
fn main() {}
