#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

#[cfg(any(target_os = "macos", target_os = "windows"))]
#[path = "native.rs"]
mod native;

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn main() {
    native::run();
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn main() {
    eprintln!("LightCraft Preview native host is supported on macOS & Windows only");
}
