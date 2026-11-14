//! Search field and on-screen keyboard (a touch kiosk has no physical keyboard).

use eframe::egui::{pos2, vec2, Align2, Color32, CornerRadius, Rect, Response, Sense, StrokeKind, Ui, Widget};

use crate::ui::paint;
use crate::ui::theme::{hairline, medium, regular, semibold, stroke, CARD, FILL, INK, MUTED, RED};

const MAX_QUERY: usize = 30;

/// Rounded search field. It is display-only: tapping it opens the on-screen keyboard.
#[must_use = "add it with `ui.add(...)`"]
pub struct SearchField<'a> {
    query: &'a str,
    active: bool,
}

impl<'a> SearchField<'a> {
    pub fn new(query: &'a str, active: bool) -> Self {
        Self { query, active }
    }
}

impl Widget for SearchField<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 68.0), Sense::click());
        let p = ui.painter();
        p.rect_filled(rect, CornerRadius::same(20), FILL);
        if self.active {
            p.rect_stroke(rect, CornerRadius::same(20), stroke(2.5, RED), StrokeKind::Inside);
        }
        paint::icon_search(p, pos2(rect.left() + 38.0, rect.center().y), 30.0, MUTED);
        let x = rect.left() + 66.0;
        let end = if self.query.is_empty() {
            let hint = "Search chicken, burgers, drinks…";
            paint::text(p, pos2(x, rect.center().y), Align2::LEFT_CENTER, hint, regular(22.0), MUTED).left()
        } else {
            paint::text(p, pos2(x, rect.center().y), Align2::LEFT_CENTER, self.query, medium(22.0), INK).right()
        };
        // Blinking caret while typing.
        let blink = (ui.input(|i| i.time) * 2.0) as i64 % 2 == 0;
        if self.active && blink {
            let cx = if self.query.is_empty() { x } else { end + 3.0 };
            p.line_segment([pos2(cx, rect.center().y - 16.0), pos2(cx, rect.center().y + 16.0)], stroke(2.5, RED));
        }
        if self.active {
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(250));
        }
        response
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardAction {
    None,
    /// The customer tapped "Done".
    Done,
}

/// On-screen QWERTY keyboard that edits `text` in place.
#[must_use = "call `.show(ui)`"]
pub struct Keyboard<'a> {
    text: &'a mut String,
}

enum Key {
    Char(char),
    Backspace,
    Space,
    Clear,
    Done,
}

impl<'a> Keyboard<'a> {
    pub fn new(text: &'a mut String) -> Self {
        Self { text }
    }

    pub fn show(self, ui: &mut Ui) -> KeyboardAction {
        let gap = 8.0;
        let key_h = 76.0;
        let width = ui.available_width();
        let key_w = (width - gap * 9.0) / 10.0;
        let top = ui.max_rect().top();
        let left = ui.max_rect().left();

        let mut keys: Vec<(Rect, Key)> = Vec::new();
        for (row, letters) in ["QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"].into_iter().enumerate() {
            let n = letters.len() as f32;
            let extra = if row == 2 { key_w * 1.6 + gap } else { 0.0 };
            let row_w = n * key_w + (n - 1.0) * gap + extra;
            let mut x = left + (width - row_w) / 2.0;
            let y = top + row as f32 * (key_h + gap);
            for ch in letters.chars() {
                keys.push((Rect::from_min_size(pos2(x, y), vec2(key_w, key_h)), Key::Char(ch.to_ascii_lowercase())));
                x += key_w + gap;
            }
            if row == 2 {
                keys.push((Rect::from_min_size(pos2(x, y), vec2(key_w * 1.6, key_h)), Key::Backspace));
            }
        }
        let y = top + 3.0 * (key_h + gap);
        let side = key_w * 2.2;
        keys.push((Rect::from_min_size(pos2(left, y), vec2(side, key_h)), Key::Clear));
        keys.push((
            Rect::from_min_size(pos2(left + side + gap, y), vec2(width - 2.0 * (side + gap), key_h)),
            Key::Space,
        ));
        keys.push((Rect::from_min_size(pos2(left + width - side, y), vec2(side, key_h)), Key::Done));
        ui.allocate_rect(Rect::from_min_size(pos2(left, top), vec2(width, 4.0 * key_h + 3.0 * gap)), Sense::hover());

        let mut action = KeyboardAction::None;
        for (i, (rect, key)) in keys.into_iter().enumerate() {
            let response = ui.interact(rect, ui.id().with(("key", i)), Sense::click());
            let pressed = response.is_pointer_button_down_on();
            let (fill, fg) = match key {
                Key::Done => (if pressed { RED.gamma_multiply(0.8) } else { RED }, Color32::WHITE),
                _ if pressed => (FILL, INK),
                _ => (CARD, INK),
            };
            let p = ui.painter();
            p.rect_filled(rect, CornerRadius::same(12), fill);
            if !matches!(key, Key::Done) {
                p.rect_stroke(rect, CornerRadius::same(12), hairline(), StrokeKind::Inside);
            }
            let label = match &key {
                Key::Char(c) => c.to_ascii_uppercase().to_string(),
                Key::Backspace => "⌫".to_owned(),
                Key::Space => "space".to_owned(),
                Key::Clear => "Clear".to_owned(),
                Key::Done => "Done".to_owned(),
            };
            let font = if matches!(key, Key::Char(_)) { medium(28.0) } else { semibold(22.0) };
            paint::text(p, rect.center(), Align2::CENTER_CENTER, label, font, fg);

            if response.clicked() {
                match key {
                    Key::Char(c) if self.text.len() < MAX_QUERY => self.text.push(c),
                    Key::Space if !self.text.is_empty() && !self.text.ends_with(' ') => self.text.push(' '),
                    Key::Backspace => {
                        self.text.pop();
                    }
                    Key::Clear => self.text.clear(),
                    Key::Done => action = KeyboardAction::Done,
                    _ => {}
                }
            }
        }
        action
    }
}
