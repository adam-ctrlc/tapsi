//! User interface: theme, artwork, drawing helpers, reusable widgets and screens.

pub mod icons;
pub mod paint;
pub mod screens;
pub mod theme;
pub mod widgets;

use eframe::egui::{self, vec2, Vec2};

/// The UI is laid out on this portrait canvas (in points) and scaled to fit the
/// window, so a 1080x1920 kiosk display and a small desktop preview look the same.
pub const REFERENCE_SIZE: Vec2 = vec2(900.0, 1600.0);

/// Sets the zoom so the reference canvas fits the window.
pub fn fit_to_window(ctx: &egui::Context) {
    let Some(native) = ctx.native_pixels_per_point() else { return };
    let pixels = ctx.screen_rect().size() * ctx.pixels_per_point();
    let ppp = (pixels.x / REFERENCE_SIZE.x).min(pixels.y / REFERENCE_SIZE.y);
    let zoom = ppp / native;
    if zoom.is_finite() && zoom > 0.0 && (ctx.zoom_factor() - zoom).abs() > 0.001 {
        ctx.set_zoom_factor(zoom);
    }
}
