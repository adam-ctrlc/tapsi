use eframe::egui::{self, vec2, Align, Color32, CornerRadius, Layout, Margin, RichText, Sense, Stroke, Ui};

use super::Stepper;
use crate::order::{format_money, CartLine, MAX_QTY};
use crate::ui::icons::Icons;
use crate::ui::theme::{bold, hairline, medium, regular, semibold, HAIRLINE, INK, MUTED, PEACH, RED};

/// What the customer did with a cart line this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineAction {
    None,
    Remove,
    SetQty(u32),
}

/// One line of the order, drawn as a row inside a [`super::list_group`].
///
/// ```text
/// [art]  name / options / unit price        total
///        [- qty +]  Remove
/// ```
#[must_use = "call `.show(ui)`"]
pub struct CartLineRow<'a> {
    icons: &'a Icons,
    line: &'a CartLine,
    currency: &'a str,
    separator: bool,
}

impl<'a> CartLineRow<'a> {
    pub fn new(icons: &'a Icons, line: &'a CartLine, currency: &'a str) -> Self {
        Self { icons, line, currency, separator: true }
    }

    pub fn separator(mut self, separator: bool) -> Self {
        self.separator = separator;
        self
    }

    pub fn show(self, ui: &mut Ui) -> LineAction {
        let line = self.line;
        let mut action = LineAction::None;
        let response = egui::Frame::new().inner_margin(Margin::symmetric(20, 18)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_top(|ui| {
                let (r, _) = ui.allocate_exact_size(vec2(100.0, 100.0), Sense::hover());
                ui.painter().rect_filled(r, CornerRadius::same(18), PEACH);
                self.icons.paint(ui.painter(), &line.icon, r.shrink(14.0));

                let text_width = (ui.available_width() - 150.0).max(200.0);
                ui.allocate_ui_with_layout(vec2(text_width, 0.0), Layout::top_down(Align::Min), |ui| {
                    ui.set_width(text_width);
                    ui.spacing_mut().item_spacing.y = 4.0;
                    ui.add(egui::Label::new(RichText::new(&line.name).font(semibold(23.0)).color(INK)).wrap());
                    let options = line.options_text();
                    if !options.is_empty() {
                        ui.add(egui::Label::new(RichText::new(options).font(regular(18.0)).color(MUTED)).wrap());
                    }
                    let each = format!("{} each", format_money(self.currency, line.unit_price()));
                    ui.label(RichText::new(each).font(regular(18.0)).color(MUTED));
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        let mut qty = line.qty;
                        if ui.add(Stepper::new(&mut qty, 1..=MAX_QTY)).changed() {
                            action = LineAction::SetQty(qty);
                        }
                        ui.add_space(6.0);
                        let remove = egui::Button::new(RichText::new("Remove").font(medium(19.0)).color(RED))
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::NONE)
                            .min_size(vec2(110.0, 62.0));
                        if ui.add(remove).clicked() {
                            action = LineAction::Remove;
                        }
                    });
                });

                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    let total = format_money(self.currency, line.total());
                    ui.label(RichText::new(total).font(bold(24.0)).color(INK));
                });
            });
        });
        if self.separator {
            let r = response.response.rect;
            ui.painter().hline((r.left() + 140.0)..=r.right(), r.bottom() - HAIRLINE / 2.0, hairline());
        }
        action
    }
}
