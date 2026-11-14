use eframe::egui::{self, pos2, vec2, Align2, Frame, Margin, Rect};

use super::{Screen, View};
use crate::kiosk::Kiosk;
use crate::order::OrderType;
use crate::ui::paint;
use crate::ui::theme::{heavy, regular, CREAM, INK, MUTED};
use crate::ui::widgets::{nav_bar, Nav, Step, Tile};

/// "Where will you be eating today?" - Dine In or Take Out.
pub struct OrderTypePicker;

impl View for OrderTypePicker {
    fn show(&mut self, ctx: &egui::Context, kiosk: &mut Kiosk) -> Option<Screen> {
        let nav = Nav { title: "Start your order", back: Some("Home"), step: Some(Step::OrderType) };
        let mut next = nav_bar(ctx, kiosk, nav).then(Screen::attract);

        egui::CentralPanel::default().frame(Frame::new().fill(CREAM).inner_margin(Margin::same(32))).show(ctx, |ui| {
            let rect = ui.max_rect();
            let h = rect.height();
            let at = |f: f32| pos2(rect.center().x, rect.top() + h * f);
            let p = ui.painter();
            paint::text(p, at(0.07), Align2::CENTER_CENTER, "Where will you be", heavy(54.0), INK);
            paint::text(p, at(0.125), Align2::CENTER_CENTER, "eating today?", heavy(54.0), INK);
            paint::text(p, at(0.175), Align2::CENTER_CENTER, "You can change this later", regular(23.0), MUTED);

            let gap = 24.0;
            let size = vec2(((rect.width() - gap) / 2.0).min(410.0), (h * 0.55).min(640.0));
            let offset = vec2((size.x + gap) / 2.0, 0.0);
            let choices = [
                (OrderType::DineIn, "🍽", "Enjoy your meal here", at(0.52) - offset),
                (OrderType::TakeOut, "🥡", "Packed to go", at(0.52) + offset),
            ];
            for (kind, icon, subtitle, center) in choices {
                let tile = Tile::new(&kiosk.icons, icon, kind.label(), subtitle);
                if ui.put(Rect::from_center_size(center, size), tile).clicked() {
                    kiosk.order_type = kind;
                    next = Some(Screen::menu());
                }
            }
        });
        next
    }
}
