use std::ops::RangeInclusive;

use eframe::egui::{pos2, vec2, Align2, CornerRadius, Rect, Response, Sense, Ui, Widget};

use crate::ui::paint;
use crate::ui::theme::{bold, stroke, BORDER, CARD, FILL, INK, MUTED};

/// iOS-style `−  qty  +` control. Edits the value in place; `response.changed()` is set on change.
#[must_use = "add it with `ui.add(...)`"]
pub struct Stepper<'a> {
    value: &'a mut u32,
    range: RangeInclusive<u32>,
}

impl<'a> Stepper<'a> {
    pub fn new(value: &'a mut u32, range: RangeInclusive<u32>) -> Self {
        Self { value, range }
    }
}

impl Widget for Stepper<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, mut response) = ui.allocate_exact_size(vec2(196.0, 62.0), Sense::hover());
        let (min, max) = (*self.range.start(), *self.range.end());
        let half = vec2(66.0, rect.height());
        let minus = Rect::from_min_size(rect.min, half);
        let plus = Rect::from_min_size(pos2(rect.right() - half.x, rect.top()), half);
        let minus_response = ui.interact(minus, response.id.with("minus"), Sense::click());
        let plus_response = ui.interact(plus, response.id.with("plus"), Sense::click());

        let value = *self.value;
        let p = ui.painter();
        p.rect_filled(rect, CornerRadius::same(20), FILL);
        for (r, button, enabled, is_plus) in
            [(minus, &minus_response, value > min, false), (plus, &plus_response, value < max, true)]
        {
            if button.is_pointer_button_down_on() && enabled {
                p.rect_filled(r.shrink(5.0), CornerRadius::same(16), CARD);
            }
            let color = if enabled { INK } else { MUTED.gamma_multiply(0.5) };
            let s = stroke(3.0, color);
            p.line_segment([r.center() - vec2(10.0, 0.0), r.center() + vec2(10.0, 0.0)], s);
            if is_plus {
                p.line_segment([r.center() - vec2(0.0, 10.0), r.center() + vec2(0.0, 10.0)], s);
            }
        }
        // Dividers between the buttons and the value.
        for x in [minus.right(), plus.left()] {
            p.vline(x, rect.y_range().shrink(16.0), stroke(1.5, BORDER));
        }
        paint::text(p, rect.center(), Align2::CENTER_CENTER, value.to_string(), bold(26.0), INK);

        if minus_response.clicked() && value > min {
            *self.value -= 1;
            response.mark_changed();
        } else if plus_response.clicked() && value < max {
            *self.value += 1;
            response.mark_changed();
        }
        response
    }
}
