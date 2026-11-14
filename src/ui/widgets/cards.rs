use eframe::egui::{pos2, vec2, Align2, Color32, CornerRadius, Rect, Response, Sense, StrokeKind, Ui, Vec2, Widget};

use super::{grid_columns, press_rect};
use crate::menu::{Menu, MenuItem};
use crate::order::format_money;
use crate::ui::icons::Icons;
use crate::ui::paint;
use crate::ui::theme::{bold, hairline, heavy, regular, semibold, stroke, CARD, INK, MUTED, PEACH, RED};

/// Menu item card: artwork, name, price and an add button. Flat, with a hairline border.
#[must_use = "add it with `ui.add(...)`"]
pub struct ItemCard<'a> {
    icons: &'a Icons,
    item: &'a MenuItem,
    currency: &'a str,
    size: Vec2,
}

impl<'a> ItemCard<'a> {
    pub fn new(icons: &'a Icons, item: &'a MenuItem, currency: &'a str, size: Vec2) -> Self {
        Self { icons, item, currency, size }
    }
}

impl Widget for ItemCard<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let item = self.item;
        let sense = if item.available { Sense::click() } else { Sense::hover() };
        let (rect, response) = ui.allocate_exact_size(self.size, sense);
        if !ui.is_rect_visible(rect) {
            return response;
        }
        let r = press_rect(rect, &response, 3.0);
        let p = ui.painter();
        let cr = CornerRadius::same(22);
        p.rect_filled(r, cr, CARD);
        let border = if response.hovered() && item.available { stroke(2.0, RED) } else { hairline() };
        p.rect_stroke(r, cr, border, StrokeKind::Inside);

        let art = Rect::from_min_size(r.min, vec2(r.width(), r.height() * 0.56)).shrink(8.0);
        p.rect_filled(art, CornerRadius::same(16), PEACH);
        self.icons.paint(p, item.art(), Rect::from_center_size(art.center(), Vec2::splat(art.height() * 0.78)));

        if let Some(tag) = &item.tag {
            let min = art.min + vec2(10.0, 10.0);
            paint::badge(p, min, &tag.to_uppercase(), bold(12.0), RED, Color32::WHITE, vec2(9.0, 6.0));
        }

        let name = p.layout(item.name.clone(), semibold(20.0), INK, r.width() - 32.0);
        p.galley(pos2(r.left() + 16.0, art.bottom() + 10.0), name, INK);

        let price = format_money(self.currency, item.price);
        paint::text(p, pos2(r.left() + 16.0, r.bottom() - 18.0), Align2::LEFT_BOTTOM, price, bold(22.0), INK);
        let plus = pos2(r.right() - 34.0, r.bottom() - 34.0);
        p.circle_filled(plus, 21.0, RED);
        let s = stroke(3.0, Color32::WHITE);
        p.line_segment([plus - vec2(8.0, 0.0), plus + vec2(8.0, 0.0)], s);
        p.line_segment([plus - vec2(0.0, 8.0), plus + vec2(0.0, 8.0)], s);

        if !item.available {
            p.rect_filled(r, cr, Color32::from_white_alpha(200));
            paint::text(p, r.center(), Align2::CENTER_CENTER, "Sold out", heavy(24.0), MUTED);
        }
        response
    }
}

/// Grid of item cards that fills the available width; returns the tapped item index.
pub fn item_grid(ui: &mut Ui, icons: &Icons, menu: &Menu, items: &[usize], min_width: f32) -> Option<usize> {
    let gap = 16.0;
    let (cols, w) = grid_columns(ui.available_width(), min_width, gap);
    let size = vec2(w, (w * 1.15).clamp(260.0, 320.0));
    let mut tapped = None;
    ui.spacing_mut().item_spacing = vec2(gap, gap);
    for row in items.chunks(cols) {
        ui.horizontal(|ui| {
            for &i in row {
                if ui.add(ItemCard::new(icons, &menu.items[i], &menu.currency_symbol, size)).clicked() {
                    tapped = Some(i);
                }
            }
        });
    }
    tapped
}

/// Large touch tile (used for Dine In / Take Out). Fills the space it is given,
/// so place it with `ui.put(rect, tile)`.
#[must_use = "add it with `ui.put(...)`"]
pub struct Tile<'a> {
    icons: &'a Icons,
    icon: &'a str,
    title: &'a str,
    subtitle: &'a str,
}

impl<'a> Tile<'a> {
    pub fn new(icons: &'a Icons, icon: &'a str, title: &'a str, subtitle: &'a str) -> Self {
        Self { icons, icon, title, subtitle }
    }
}

impl Widget for Tile<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click());
        let r = press_rect(rect, &response, 5.0);
        let p = ui.painter();
        let cr = CornerRadius::same(32);
        p.rect_filled(r, cr, CARD);
        let border = if response.hovered() { stroke(2.5, RED) } else { hairline() };
        p.rect_stroke(r, cr, border, StrokeKind::Inside);

        let icon_center = pos2(r.center().x, r.top() + r.height() * 0.40);
        let radius = r.width() * 0.32;
        p.circle_filled(icon_center, radius, PEACH);
        self.icons.paint(p, self.icon, Rect::from_center_size(icon_center, Vec2::splat(radius * 1.25)));
        let at = |f: f32| pos2(r.center().x, r.top() + r.height() * f);
        paint::text(p, at(0.76), Align2::CENTER_CENTER, self.title, bold(38.0), INK);
        paint::text(p, at(0.85), Align2::CENTER_CENTER, self.subtitle, regular(22.0), MUTED);
        response
    }
}
