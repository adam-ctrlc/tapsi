use eframe::egui::{pos2, vec2, Align2, Color32, CornerRadius, Rect, Response, Sense, StrokeKind, Ui, Vec2, Widget};

use super::press_rect;
use crate::menu::Category;
use crate::ui::icons::Icons;
use crate::ui::paint;
use crate::ui::theme::{hairline, semibold, CARD, INK, RED};

/// Category pill for the horizontal category row; filled red when selected.
#[must_use = "add it with `ui.add(...)`"]
pub struct CategoryChip<'a> {
    icons: &'a Icons,
    category: &'a Category,
    selected: bool,
}

impl<'a> CategoryChip<'a> {
    pub fn new(icons: &'a Icons, category: &'a Category, selected: bool) -> Self {
        Self { icons, category, selected }
    }
}

impl Widget for CategoryChip<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (fill, fg) = if self.selected { (RED, Color32::WHITE) } else { (CARD, INK) };
        let galley = ui.painter().layout_no_wrap(self.category.name.clone(), semibold(20.0), fg);
        let size = vec2(14.0 + 40.0 + 10.0 + galley.size().x + 24.0, 64.0);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        let r = press_rect(rect, &response, 2.0);
        let p = ui.painter();
        p.rect_filled(r, CornerRadius::same(32), fill);
        if !self.selected {
            p.rect_stroke(r, CornerRadius::same(32), hairline(), StrokeKind::Inside);
        }
        let icon = pos2(r.left() + 14.0 + 20.0, r.center().y);
        if self.selected {
            p.circle_filled(icon, 22.0, CARD);
        }
        self.icons.paint(p, &self.category.icon, Rect::from_center_size(icon, Vec2::splat(34.0)));
        let label = pos2(icon.x + 30.0, r.center().y);
        paint::text(p, label, Align2::LEFT_CENTER, &self.category.name, semibold(20.0), fg);
        response
    }
}
