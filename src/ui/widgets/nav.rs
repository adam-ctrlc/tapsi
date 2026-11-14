//! Top navigation bar (back button, title, order type) with a checkout progress tracker.

use eframe::egui::{self, pos2, vec2, Align2, Color32, Frame, Id, Margin, Rect, Response, Sense, Ui, Widget};

use super::Chip;
use crate::kiosk::Kiosk;
use crate::order::OrderType;
use crate::ui::paint;
use crate::ui::theme::{
    bold, hairline, medium, semibold, stroke, BORDER, CARD, FILL, HAIRLINE, INK, MUTED, RED, RED_TINT,
};

const BAR_HEIGHT: f32 = 100.0;
const PROGRESS_HEIGHT: f32 = 84.0;

/// Steps of an order, shown as a breadcrumb-style progress tracker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Step {
    OrderType,
    Menu,
    Review,
    Pay,
}

impl Step {
    const ALL: [Step; 4] = [Step::OrderType, Step::Menu, Step::Review, Step::Pay];

    fn label(self) -> &'static str {
        match self {
            Step::OrderType => "Order type",
            Step::Menu => "Choose food",
            Step::Review => "Review",
            Step::Pay => "Pay",
        }
    }
}

/// What the navigation bar shows.
pub struct Nav<'a> {
    pub title: &'a str,
    /// Label of the back button (usually the previous screen's name).
    pub back: Option<&'a str>,
    /// Current step for the progress tracker; `None` hides it.
    pub step: Option<Step>,
}

/// Draws the navigation bar. Returns true when the back button was tapped.
/// Tapping the order-type chip switches between Dine In and Take Out.
pub fn nav_bar(ctx: &egui::Context, kiosk: &mut Kiosk, nav: Nav) -> bool {
    let height = BAR_HEIGHT + if nav.step.is_some() { PROGRESS_HEIGHT } else { 0.0 };
    let mut back = false;
    egui::TopBottomPanel::top("nav_bar")
        .exact_height(height)
        .show_separator_line(false)
        .frame(Frame::new().fill(CARD).inner_margin(Margin::ZERO))
        .show(ctx, |ui| {
            let full = ui.max_rect();
            ui.painter().hline(full.x_range(), full.bottom() - HAIRLINE / 2.0, hairline());
            let bar = Rect::from_min_size(full.min, vec2(full.width(), BAR_HEIGHT));

            if let Some(label) = nav.back {
                let r = Rect::from_min_size(bar.min + vec2(10.0, 18.0), vec2(230.0, 64.0));
                let response = ui.interact(r, Id::new("nav_back"), Sense::click());
                let color = if response.is_pointer_button_down_on() { RED.gamma_multiply(0.5) } else { RED };
                if response.hovered() {
                    ui.painter().rect_filled(r, 16, RED_TINT);
                }
                paint::chevron(ui.painter(), pos2(r.left() + 26.0, r.center().y), 30.0, true, color);
                paint::text(
                    ui.painter(),
                    pos2(r.left() + 46.0, r.center().y),
                    Align2::LEFT_CENTER,
                    label,
                    medium(24.0),
                    color,
                );
                back = response.clicked();
            }

            paint::text(ui.painter(), bar.center(), Align2::CENTER_CENTER, nav.title, bold(28.0), INK);

            let chip_rect =
                Rect::from_min_max(pos2(bar.right() - 240.0, bar.top()), pos2(bar.right() - 20.0, bar.bottom()));
            let chip = ui.scope_builder(
                egui::UiBuilder::new().max_rect(chip_rect).layout(egui::Layout::right_to_left(egui::Align::Center)),
                |ui| ui.add(Chip::order_type(kiosk.order_type).switchable()),
            );
            if chip.inner.clicked() {
                kiosk.order_type = match kiosk.order_type {
                    OrderType::DineIn => OrderType::TakeOut,
                    OrderType::TakeOut => OrderType::DineIn,
                };
            }

            if let Some(step) = nav.step {
                let r = Rect::from_min_max(
                    pos2(full.left() + 30.0, bar.bottom() - 4.0),
                    pos2(full.right() - 30.0, full.bottom() - 10.0),
                );
                ui.put(r, Progress { current: step });
            }
        });
    back
}

/// Four numbered steps joined by lines; done steps show a check.
struct Progress {
    current: Step,
}

impl Widget for Progress {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
        let p = ui.painter();
        let n = Step::ALL.len() as f32;
        let x = |i: usize| rect.left() + rect.width() * (i as f32 + 0.5) / n;
        let y = rect.top() + 24.0;
        let r = 17.0;

        for i in 0..Step::ALL.len() - 1 {
            let color = if Step::ALL[i] < self.current { RED } else { BORDER };
            p.line_segment([pos2(x(i) + r + 8.0, y), pos2(x(i + 1) - r - 8.0, y)], stroke(3.0, color));
        }
        for (i, step) in Step::ALL.into_iter().enumerate() {
            let c = pos2(x(i), y);
            let (label_color, font) = if step < self.current {
                p.circle_filled(c, r, RED);
                paint::check(p, c, 22.0, Color32::WHITE);
                (INK, medium(17.0))
            } else if step == self.current {
                p.circle_filled(c, r + 6.0, RED_TINT);
                p.circle_filled(c, r, RED);
                paint::text(p, c, Align2::CENTER_CENTER, (i + 1).to_string(), bold(17.0), Color32::WHITE);
                (RED, semibold(17.0))
            } else {
                p.circle_filled(c, r, FILL);
                paint::text(p, c, Align2::CENTER_CENTER, (i + 1).to_string(), bold(17.0), MUTED);
                (MUTED, medium(17.0))
            };
            paint::text(p, pos2(c.x, y + 40.0), Align2::CENTER_CENTER, step.label(), font, label_color);
        }
        response
    }
}
