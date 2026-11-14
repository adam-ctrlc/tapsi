use eframe::egui::{self, vec2, Frame, Margin, Ui};

use super::KioskButton;
use crate::kiosk::Kiosk;
use crate::ui::theme::{hairline, CARD, CREAM, HAIRLINE};

/// White bottom bar with a hairline top border.
pub fn bottom_bar<R>(ctx: &egui::Context, id: &str, height: f32, contents: impl FnOnce(&mut Ui) -> R) -> R {
    egui::TopBottomPanel::bottom(id.to_owned())
        .exact_height(height)
        .show_separator_line(false)
        .frame(Frame::new().fill(CARD).inner_margin(Margin::symmetric(24, 16)))
        .show(ctx, |ui| {
            let rect = ui.clip_rect();
            ui.painter().hline(rect.x_range(), rect.top() + HAIRLINE / 2.0, hairline());
            contents(ui)
        })
        .inner
}

/// "View Order" button floating above the tab bar while the tray has items.
/// Returns true when tapped. Call it after the tab bar so it sits above it.
pub fn checkout_bar(ctx: &egui::Context, kiosk: &Kiosk) -> bool {
    let count = kiosk.cart.count();
    if count == 0 {
        return false;
    }
    let label = format!(
        "View Order  ·  {count} item{}  ·  {}",
        if count == 1 { "" } else { "s" },
        kiosk.money(kiosk.cart.total())
    );
    egui::TopBottomPanel::bottom("checkout_bar")
        .exact_height(108.0)
        .show_separator_line(false)
        .frame(Frame::new().fill(CREAM).inner_margin(Margin::symmetric(24, 12)))
        .show(ctx, |ui| {
            let width = ui.available_width();
            let scale = kiosk.cart_bump_scale();
            ui.add(KioskButton::primary(&label).font_size(24.0 * scale.min(1.08)).size(vec2(width, 84.0))).clicked()
        })
        .inner
}
