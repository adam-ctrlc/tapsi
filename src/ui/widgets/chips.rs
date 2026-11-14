use eframe::egui::{pos2, vec2, Align2, Color32, CornerRadius, Painter, Pos2, Rect, Response, Sense, Ui, Widget};

use crate::order::OrderType;
use crate::ui::paint;
use crate::ui::theme::{bold, hairline, semibold, stroke, CARD, FILL, HAIRLINE, INK, MUTED, RED};

/// Small rounded label, e.g. "Required".
#[must_use = "add it with `ui.add(...)`"]
pub struct Pill<'a> {
    text: &'a str,
    fill: Color32,
    fg: Color32,
}

impl<'a> Pill<'a> {
    pub fn new(text: &'a str, fill: Color32, fg: Color32) -> Self {
        Self { text, fill, fg }
    }
}

impl Widget for Pill<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let font = bold(16.0);
        let width = ui.painter().layout_no_wrap(self.text.to_owned(), font.clone(), self.fg).size().x;
        let (rect, response) = ui.allocate_exact_size(vec2(width + 28.0, 34.0), Sense::hover());
        let p = ui.painter();
        p.rect_filled(rect, CornerRadius::same(17), self.fill);
        paint::text(p, rect.center(), Align2::CENTER_CENTER, self.text, font, self.fg);
        response
    }
}

/// A line-icon drawing function, see `paint::icon_*`.
type LineIcon = fn(&Painter, Pos2, f32, Color32);

/// Grey pill with a red line icon on a white disc, e.g. the Dine In / Take Out indicator.
///
/// Painted at a fixed size so it lays out the same in any parent direction.
#[must_use = "add it with `ui.add(...)`"]
pub struct Chip<'a> {
    icon: LineIcon,
    text: &'a str,
    switchable: bool,
}

impl<'a> Chip<'a> {
    pub fn new(icon: LineIcon, text: &'a str) -> Self {
        Self { icon, text, switchable: false }
    }

    pub fn order_type(order_type: OrderType) -> Self {
        let icon: LineIcon = match order_type {
            OrderType::DineIn => paint::icon_dine,
            OrderType::TakeOut => paint::icon_bag,
        };
        Self::new(icon, order_type.label())
    }

    /// Show a swap hint and make the chip clickable.
    pub fn switchable(mut self) -> Self {
        self.switchable = true;
        self
    }
}

impl Widget for Chip<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let font = semibold(19.0);
        let text_width = ui.painter().layout_no_wrap(self.text.to_owned(), font.clone(), INK).size().x;
        let extra = if self.switchable { 26.0 } else { 0.0 };
        let size = vec2(8.0 + 38.0 + 10.0 + text_width + 18.0 + extra, 54.0);
        let sense = if self.switchable { Sense::click() } else { Sense::hover() };
        let (rect, response) = ui.allocate_exact_size(size, sense);
        let p = ui.painter();
        p.rect_filled(rect, CornerRadius::same(27), FILL);
        if response.is_pointer_button_down_on() {
            p.rect_filled(rect, CornerRadius::same(27), Color32::from_black_alpha(14));
        }

        let icon_center = pos2(rect.left() + 8.0 + 19.0, rect.center().y);
        p.circle_filled(icon_center, 19.0, CARD);
        (self.icon)(p, icon_center, 24.0, RED);
        let label = pos2(icon_center.x + 19.0 + 10.0, rect.center().y);
        paint::text(p, label, Align2::LEFT_CENTER, self.text, font, INK);
        if self.switchable {
            // Small down-chevron hinting that it can be changed.
            let c = pos2(rect.right() - 24.0, rect.center().y);
            let line = stroke(2.5, MUTED);
            p.line_segment([c + vec2(-6.0, -3.0), c + vec2(0.0, 3.0)], line);
            p.line_segment([c + vec2(0.0, 3.0), c + vec2(6.0, -3.0)], line);
        }
        response
    }
}

/// Dine In / Take Out tabs with icons and a sliding underline under the selected tab.
/// `response.changed()` is set when switched.
#[must_use = "add it with `ui.add(...)`"]
pub struct OrderTypeSwitch<'a> {
    value: &'a mut OrderType,
}

impl<'a> OrderTypeSwitch<'a> {
    pub fn new(value: &'a mut OrderType) -> Self {
        Self { value }
    }
}

impl Widget for OrderTypeSwitch<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        const KINDS: [OrderType; 2] = [OrderType::DineIn, OrderType::TakeOut];
        let width = ui.available_width();
        let tab = vec2(width / KINDS.len() as f32, 80.0);
        let (rect, mut response) = ui.allocate_exact_size(vec2(width, tab.y), Sense::hover());
        let p = ui.painter();
        p.hline(rect.x_range(), rect.bottom() - HAIRLINE / 2.0, hairline());

        for (k, kind) in KINDS.into_iter().enumerate() {
            let r = Rect::from_min_size(rect.min + vec2(k as f32 * tab.x, 0.0), tab);
            let tab_response = ui.interact(r, response.id.with(k), Sense::click());
            if tab_response.is_pointer_button_down_on() {
                p.rect_filled(r.shrink(6.0), CornerRadius::same(14), FILL);
            }
            let selected = kind == *self.value;
            let (color, font) = if selected { (RED, bold(22.0)) } else { (MUTED, semibold(22.0)) };

            // Icon + label, centered together in the tab.
            let (icon_size, gap) = (30.0, 12.0);
            let label_width = p.layout_no_wrap(kind.label().to_owned(), font.clone(), color).size().x;
            let left = r.center().x - (icon_size + gap + label_width) / 2.0;
            let icon: fn(&Painter, Pos2, f32, Color32) = match kind {
                OrderType::DineIn => paint::icon_dine,
                OrderType::TakeOut => paint::icon_bag,
            };
            icon(p, pos2(left + icon_size / 2.0, r.center().y - 2.0), icon_size, color);
            paint::text(
                p,
                pos2(left + icon_size + gap, r.center().y - 2.0),
                Align2::LEFT_CENTER,
                kind.label(),
                font,
                color,
            );

            if tab_response.clicked() && !selected {
                *self.value = kind;
                response.mark_changed();
            }
        }

        // Underline slides to the selected tab.
        let index = KINDS.iter().position(|k| k == self.value).unwrap_or(0) as f32;
        let x = ui.ctx().animate_value_with_time(response.id.with("underline"), rect.left() + index * tab.x, 0.18);
        let underline = Rect::from_min_max(pos2(x + 28.0, rect.bottom() - 5.0), pos2(x + tab.x - 28.0, rect.bottom()));
        p.rect_filled(underline, CornerRadius::same(3), RED);
        response
    }
}
