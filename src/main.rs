//! r7 Crypto Lab - GUI Application

// Release builds are a desktop program, not a console tool: without this,
// double-clicking the .exe on Windows opens a terminal window behind the GUI.
#![cfg_attr(
    all(not(debug_assertions), not(target_arch = "wasm32")),
    windows_subsystem = "windows"
)]

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    use eframe::egui;
    use r7crypto::viz::{icon, R7CryptoApp};

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_min_inner_size([700.0, 500.0])
            .with_title("r7 Crypto Lab")
            // Wayland takes the taskbar icon from the .desktop file that
            // matches this app id; the window icon alone is ignored there.
            .with_app_id("r7crypto")
            .with_icon(icon::icon_data()),
        ..Default::default()
    };

    eframe::run_native(
        "r7 Crypto Lab",
        options,
        Box::new(|_cc| Ok(Box::new(R7CryptoApp::default()))),
    )
}

// Trunk compiles the library as the web entry point, but Cargo still sees this
// binary target. It has no native API on wasm32, so keep it as a no-op there.
#[cfg(target_arch = "wasm32")]
fn main() {}
