//! Bottom sheet: a modal panel that slides up from the bottom, with a grab handle
//! and a close (✕) button in the top-right. Tapping the dimmed backdrop also closes it.

use std::hash::Hash;
use std::time::Instant;

use eframe::egui::{self, pos2, vec2, Align2, Color32, CornerRadius, Frame, Id, Margin, Modal, Rect, Sense, Ui, Vec2};

use crate::ui::paint;
use crate::ui::theme::{heavy, stroke, BORDER, CARD, FILL, INK};

const SLIDE_SECS: f32 = 0.28;
const MARGIN_X: i8 = 32;

pub struct SheetResponse {
    /// The customer tapped ✕ or the backdrop.
    pub close: bool,
}

#[must_use = "call `.show(ctx, ...)`"]
pub struct Sheet<'a> {
    id: Id,
    opened_at: Instant,
    title: Option<&'a str>,
    height: Option<f32>,
}

impl<'a> Sheet<'a> {
    /// `opened_at` drives the slide-in animation.
    pub fn new(id: impl Hash, opened_at: Instant) -> Self {
        Self { id: Id::new(id), opened_at, title: None, height: None }
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// Fixed height as a fraction of the screen; otherwise the sheet fits its content.
    pub fn height_fraction(mut self, fraction: f32) -> Self {
        self.height = Some(fraction);
        self
    }

    pub fn show(self, ctx: &egui::Context, content: impl FnOnce(&mut Ui)) -> SheetResponse {
        let t = (self.opened_at.elapsed().as_secs_f32() / SLIDE_SECS).min(1.0);
        let eased = paint::ease_out(t);
        if t < 1.0 {
            ctx.request_repaint();
        }
        let screen = ctx.screen_rect();
        let height = self.height.map(|f| screen.height() * f);
        let slide = (1.0 - eased) * height.unwrap_or(screen.height() * 0.4);

        let area = Modal::default_area(self.id).anchor(Align2::CENTER_BOTTOM, vec2(0.0, slide));
        let frame = Frame::new()
            .fill(CARD)
            .corner_radius(CornerRadius { nw: 36, ne: 36, sw: 0, se: 0 })
            .inner_margin(Margin { left: MARGIN_X, right: MARGIN_X, top: 10, bottom: 28 });

        let mut close = false;
        let response = Modal::new(self.id)
            .area(area)
            .frame(frame)
            .backdrop_color(Color32::from_black_alpha((110.0 * eased) as u8))
            .show(ctx, |ui| {
                let width = screen.width() - 2.0 * MARGIN_X as f32;
                ui.set_width(width);
                if let Some(h) = height {
                    ui.set_height(h - 38.0);
                }
                ui.spacing_mut().item_spacing.y = 10.0;

                // Grab handle.
                let (handle, _) = ui.allocate_exact_size(vec2(width, 18.0), Sense::hover());
                ui.painter().rect_filled(Rect::from_center_size(handle.center(), vec2(64.0, 7.0)), 4, BORDER);

                // Title and close button.
                let (row, _) = ui.allocate_exact_size(vec2(width, 60.0), Sense::hover());
                if let Some(title) = self.title {
                    paint::text(ui.painter(), row.left_center(), Align2::LEFT_CENTER, title, heavy(32.0), INK);
                }
                let x = Rect::from_center_size(pos2(row.right() - 28.0, row.center().y), Vec2::splat(56.0));
                let x_response = ui.interact(x, self.id.with("close"), Sense::click());
                let fill = if x_response.is_pointer_button_down_on() { BORDER } else { FILL };
                ui.painter().circle_filled(x.center(), 28.0, fill);
                let (c, s) = (x.center(), 9.0);
                ui.painter().line_segment([c + vec2(-s, -s), c + vec2(s, s)], stroke(3.5, INK));
                ui.painter().line_segment([c + vec2(-s, s), c + vec2(s, -s)], stroke(3.5, INK));
                close = x_response.clicked();

                content(ui)
            });
        close |= response.should_close();
        SheetResponse { close }
    }
}
