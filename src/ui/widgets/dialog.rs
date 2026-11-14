use eframe::egui::{self, vec2, Align2, Color32, CornerRadius, Frame, Id, Margin, RichText, Sense};

use crate::ui::paint;
use crate::ui::theme::{semibold, GREEN, INK};

/// Floating confirmation message above the bottom bars (and above any sheet).
pub fn toast(ctx: &egui::Context, message: &str) {
    egui::Area::new(Id::new("toast"))
        .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -250.0))
        .order(egui::Order::Tooltip)
        .interactable(false)
        .show(ctx, |ui| {
            Frame::new()
                .fill(INK.gamma_multiply(0.94))
                .corner_radius(CornerRadius::same(36))
                .inner_margin(Margin { left: 14, right: 28, top: 12, bottom: 12 })
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (r, _) = ui.allocate_exact_size(vec2(38.0, 38.0), Sense::hover());
                        ui.painter().circle_filled(r.center(), 19.0, GREEN);
                        paint::check(ui.painter(), r.center(), 22.0, Color32::WHITE);
                        ui.label(RichText::new(message).font(semibold(21.0)).color(Color32::WHITE));
                    });
                });
        });
}
