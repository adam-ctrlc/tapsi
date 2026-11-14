use eframe::egui::{
    self, pos2, vec2, Align2, Color32, CornerRadius, Rect, Response, RichText, Sense, Ui, Vec2, Widget,
};

use super::{press_rect, Pill};
use crate::menu::{MenuItem, Promo};
use crate::order::format_money;
use crate::ui::icons::Icons;
use crate::ui::paint;
use crate::ui::theme::{bold, heavy, medium, regular, GOLD, INK, MUTED, PEACH, RED, RED_TINT};

/// Flat red "Today's deal" banner above the menu grid. Clickable.
#[must_use = "add it with `ui.add(...)`"]
pub struct PromoBanner<'a> {
    icons: &'a Icons,
    promo: &'a Promo,
    art: &'a str,
}

impl<'a> PromoBanner<'a> {
    pub fn new(icons: &'a Icons, promo: &'a Promo, art: &'a str) -> Self {
        Self { icons, promo, art }
    }
}

impl Widget for PromoBanner<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 190.0), Sense::click());
        let r = press_rect(rect, &response, 3.0);
        let p = ui.painter();
        p.rect_filled(r, CornerRadius::same(26), RED);

        let disc = r.right_center() - vec2(110.0, 0.0);
        p.circle_filled(disc, 74.0, Color32::from_white_alpha(36));
        self.icons.paint(p, self.art, Rect::from_center_size(disc, Vec2::splat(124.0)));

        let x = r.left() + 30.0;
        let badge = paint::badge(p, pos2(x, r.top() + 26.0), "TODAY'S DEAL", bold(14.0), GOLD, INK, vec2(11.0, 7.0));

        paint::text(p, pos2(x, badge.bottom() + 6.0), Align2::LEFT_TOP, &self.promo.title, heavy(42.0), Color32::WHITE);
        let sub =
            p.layout(self.promo.subtitle.clone(), medium(18.0), Color32::from_white_alpha(220), r.width() - 300.0);
        p.galley(pos2(x, r.bottom() - 24.0 - sub.size().y), sub, Color32::WHITE);
        response
    }
}

/// Item artwork, name, description and price at the top of the customize sheet.
#[must_use = "add it with `ui.add(...)`"]
pub struct ItemHero<'a> {
    icons: &'a Icons,
    item: &'a MenuItem,
    currency: &'a str,
}

impl<'a> ItemHero<'a> {
    pub fn new(icons: &'a Icons, item: &'a MenuItem, currency: &'a str) -> Self {
        Self { icons, item, currency }
    }
}

impl Widget for ItemHero<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let item = self.item;
        ui.horizontal_top(|ui| {
            let (art, _) = ui.allocate_exact_size(vec2(200.0, 200.0), Sense::hover());
            ui.painter().rect_filled(art, CornerRadius::same(26), PEACH);
            self.icons.paint(ui.painter(), item.art(), art.shrink(24.0));
            ui.add_space(8.0);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 8.0;
                ui.add_space(6.0);
                if let Some(tag) = &item.tag {
                    ui.add(Pill::new(&tag.to_uppercase(), RED_TINT, RED));
                }
                ui.add(egui::Label::new(RichText::new(&item.name).font(heavy(34.0)).color(INK)).wrap());
                if !item.description.is_empty() {
                    ui.add(egui::Label::new(RichText::new(&item.description).font(regular(20.0)).color(MUTED)).wrap());
                }
                ui.label(RichText::new(format_money(self.currency, item.price)).font(bold(30.0)).color(RED));
            });
        })
        .response
    }
}
