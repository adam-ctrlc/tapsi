use std::time::{Duration, Instant};

use eframe::egui::{
    self, vec2, Align, Align2, Color32, CornerRadius, Frame, Layout, Margin, Rect, RichText, ScrollArea, Sense,
    StrokeKind, UiBuilder,
};

use super::{Screen, View};
use crate::kiosk::Kiosk;
use crate::order::{format_money, PlacedOrder};
use crate::ui::paint;
use crate::ui::theme::{
    bold, heavy, medium, regular, semibold, stroke, CARD, CREAM, GOLD, GREEN, INK, MUTED, RED, RED_DARK, RED_DEEP,
};
use crate::ui::widgets::{Chip, KioskButton, Pill};

/// The order-number screen returns to the start after this long.
const RESET_AFTER: Duration = Duration::from_secs(25);

/// "Thank you!" with the order number and a summary.
pub struct Done {
    order: PlacedOrder,
    shown_at: Instant,
}

impl Done {
    pub fn new(order: PlacedOrder) -> Self {
        Self { order, shown_at: Instant::now() }
    }
}

impl View for Done {
    fn show(&mut self, ctx: &egui::Context, kiosk: &mut Kiosk) -> Option<Screen> {
        let remaining = RESET_AFTER.saturating_sub(self.shown_at.elapsed());
        if remaining.is_zero() {
            return Some(Screen::attract());
        }
        let order = &self.order;
        let currency = kiosk.menu.currency_symbol.as_str();
        let t = kiosk.started.elapsed().as_secs_f32();
        let mut next = None;

        egui::CentralPanel::default().frame(Frame::new().fill(RED).inner_margin(Margin::same(36))).show(ctx, |ui| {
            let full = ui.ctx().screen_rect();
            let p = ui.painter();
            paint::gradient(p, full, RED, RED_DEEP);
            paint::sunburst(
                &p.with_clip_rect(full),
                full.center(),
                full.height(),
                20,
                t * 0.04,
                Color32::from_white_alpha(7),
            );

            let rect = ui.max_rect();
            let card = Rect::from_center_size(rect.center(), rect.size().min(vec2(800.0, 1260.0)));
            p.rect_filled(card, CornerRadius::same(40), CARD);

            ui.scope_builder(UiBuilder::new().max_rect(card.shrink(40.0)), |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    let (r, _) = ui.allocate_exact_size(vec2(120.0, 120.0), Sense::hover());
                    ui.painter().circle_filled(r.center(), 60.0, GREEN.gamma_multiply(0.15));
                    ui.painter().circle_filled(r.center(), 46.0, GREEN);
                    paint::check(ui.painter(), r.center(), 52.0, Color32::WHITE);
                    ui.label(RichText::new("Thank you!").font(heavy(64.0)).color(INK));
                    ui.label(RichText::new("Your order number is").font(medium(26.0)).color(MUTED));
                    ui.add_space(4.0);

                    let (r, _) = ui.allocate_exact_size(vec2(420.0, 220.0), Sense::hover());
                    let p = ui.painter();
                    p.rect_filled(r, CornerRadius::same(36), GOLD);
                    p.rect_stroke(
                        r.shrink(10.0),
                        CornerRadius::same(28),
                        stroke(3.0, Color32::from_white_alpha(140)),
                        StrokeKind::Inside,
                    );
                    paint::text(
                        p,
                        r.center() + vec2(0.0, 6.0),
                        Align2::CENTER_CENTER,
                        order.number.to_string(),
                        heavy(150.0),
                        RED_DARK,
                    );
                    ui.add_space(8.0);

                    ui.add(Chip::order_type(order.order_type));
                    let message = if order.paid {
                        "Payment received! Please wait for your number to be called."
                    } else {
                        "Please pay at the counter and show your order number."
                    };
                    ui.add(egui::Label::new(RichText::new(message).font(semibold(24.0)).color(INK)).wrap());
                });
                ui.add_space(16.0);

                Frame::new().fill(CREAM).corner_radius(CornerRadius::same(24)).inner_margin(Margin::same(24)).show(
                    ui,
                    |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Order Summary").font(bold(26.0)).color(INK));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(RichText::new(&order.placed_at).font(regular(17.0)).color(MUTED));
                            });
                        });
                        ScrollArea::vertical().max_height(270.0).show(ui, |ui| {
                            for line in &order.lines {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(format!("{} × {}", line.qty, line.name)).font(semibold(20.0)),
                                    );
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.label(
                                            RichText::new(format_money(currency, line.total())).font(semibold(20.0)),
                                        );
                                    });
                                });
                                let options = line.options_text();
                                if !options.is_empty() {
                                    ui.add(
                                        egui::Label::new(RichText::new(options).font(regular(16.0)).color(MUTED))
                                            .wrap(),
                                    );
                                }
                            }
                        });
                        ui.separator();
                        ui.horizontal(|ui| {
                            let (status, fill) = if order.paid {
                                (format!("PAID · {}", order.payment.label()), GREEN)
                            } else {
                                ("UNPAID · PAY AT COUNTER".to_string(), RED)
                            };
                            ui.add(Pill::new(&status, fill, Color32::WHITE));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(
                                    RichText::new(format_money(currency, order.total)).font(heavy(34.0)).color(RED),
                                );
                                ui.label(RichText::new("Total").font(bold(22.0)));
                            });
                        });
                    },
                );

                ui.add_space(20.0);
                let width = ui.available_width();
                if ui.add(KioskButton::primary("Start New Order").size(vec2(width, 90.0))).clicked() {
                    next = Some(Screen::attract());
                }
                ui.vertical_centered(|ui| {
                    let countdown = format!("Returning to the start screen in {}s", remaining.as_secs() + 1);
                    ui.label(RichText::new(countdown).font(regular(18.0)).color(MUTED));
                });
            });
        });
        next
    }

    fn times_out(&self) -> bool {
        false
    }

    fn animated(&self) -> bool {
        true
    }
}
