// Hide the console window in release builds on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod kiosk;
mod menu;
mod order;
mod ui;

use eframe::egui;

fn main() -> eframe::Result<()> {
    // `--kiosk` / `--fullscreen`: borderless fullscreen for the real (portrait) touch screen.
    let fullscreen = std::env::args().any(|a| a == "--kiosk" || a == "--fullscreen");

    // On a desktop the window is a scaled-down 9:16 preview of the portrait kiosk;
    // the UI scales to fit whatever size the window is (see `ui::REFERENCE_SIZE`).
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Self-Ordering Kiosk")
        .with_inner_size([405.0, 720.0])
        .with_min_inner_size([270.0, 480.0]);
    if fullscreen {
        viewport = viewport.with_fullscreen(true).with_decorations(false);
    }

    let options = eframe::NativeOptions { viewport, ..Default::default() };

    eframe::run_native("Self-Ordering Kiosk", options, Box::new(|cc| Ok(Box::new(app::KioskApp::new(cc)))))
}
