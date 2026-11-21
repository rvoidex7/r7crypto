//! r7 Crypto Lab - GUI Application

// Release builds are a desktop program, not a console tool: without this,
// double-clicking the .exe on Windows opens a terminal window behind the GUI.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use r7crypto::viz::{icon, R7CryptoApp};

fn main() -> Result<(), eframe::Error> {
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
