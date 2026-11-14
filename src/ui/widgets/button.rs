use eframe::egui::{vec2, Align2, Color32, CornerRadius, Response, Sense, Stroke, StrokeKind, Ui, Vec2, Widget};

use crate::ui::paint;

use crate::ui::theme::{bold, hairline, stroke, CARD, GOLD, INK, RED, RED_TINT};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonStyle {
    /// Filled red: the main action on a screen.
    Primary,
    /// Light red with red text: secondary actions.
    Tinted,
    /// White with a hairline border.
    Secondary,
    /// Filled gold, for use on red backgrounds.
    Gold,
    /// Transparent with a white outline, for use on red backgrounds.
    OnRed,
}

impl ButtonStyle {
    /// (fill, text, border)
    fn colors(self) -> (Color32, Color32, Stroke) {
        match self {
            Self::Primary => (RED, Color32::WHITE, Stroke::NONE),
            Self::Tinted => (RED_TINT, RED, Stroke::NONE),
            Self::Secondary => (CARD, INK, hairline()),
            Self::Gold => (GOLD, INK, Stroke::NONE),
            Self::OnRed => (Color32::TRANSPARENT, Color32::WHITE, stroke(2.0, Color32::from_white_alpha(200))),
        }
    }
}

/// Large rounded touch button.
#[must_use = "add it with `ui.add(...)`"]
pub struct KioskButton<'a> {
    text: &'a str,
    style: ButtonStyle,
    size: Option<Vec2>,
    font_size: f32,
    enabled: bool,
}

impl<'a> KioskButton<'a> {
    pub fn new(text: &'a str, style: ButtonStyle) -> Self {
        Self { text, style, size: None, font_size: 24.0, enabled: true }
    }

    pub fn primary(text: &'a str) -> Self {
        Self::new(text, ButtonStyle::Primary)
    }

    pub fn tinted(text: &'a str) -> Self {
        Self::new(text, ButtonStyle::Tinted)
    }

    pub fn secondary(text: &'a str) -> Self {
        Self::new(text, ButtonStyle::Secondary)
    }

    #[allow(dead_code)]
    pub fn gold(text: &'a str) -> Self {
        Self::new(text, ButtonStyle::Gold)
    }

    #[allow(dead_code)]
    pub fn on_red(text: &'a str) -> Self {
        Self::new(text, ButtonStyle::OnRed)
    }

    /// Exact size; otherwise the button fits its text (or the rect given to `ui.put`).
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = Some(size);
        self
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl Widget for KioskButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (fill, fg, border) = self.style.colors();
        let font = bold(self.font_size);
        let text_width = ui.painter().layout_no_wrap(self.text.to_owned(), font.clone(), fg).size().x;
        let size = self.size.unwrap_or(vec2(text_width + 48.0, self.font_size + 36.0));
        let sense = if self.enabled { Sense::click() } else { Sense::hover() };
        let (rect, response) = ui.allocate_exact_size(size, sense);
        if !ui.is_rect_visible(rect) {
            return response;
        }

        let pressed = response.is_pointer_button_down_on();
        let r = if pressed { rect.shrink(2.0) } else { rect };
        let fade = if self.enabled { 1.0 } else { 0.4 };
        let cr = CornerRadius::same(18);
        let p = ui.painter();
        p.rect_filled(r, cr, fill.gamma_multiply(fade));
        if border != Stroke::NONE {
            p.rect_stroke(r, cr, border, StrokeKind::Inside);
        }
        if pressed {
            p.rect_filled(r, cr, Color32::from_black_alpha(22));
        }
        paint::text(p, r.center(), Align2::CENTER_CENTER, self.text, font, fg.gamma_multiply(fade));
        response
    }
}
