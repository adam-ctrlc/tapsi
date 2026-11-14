use std::time::{Duration, Instant};

use eframe::egui::{self, vec2, Color32, CornerRadius, Frame, Margin, Rect, RichText, Sense, StrokeKind, UiBuilder};

use super::{Screen, View};
use crate::kiosk::Kiosk;
use crate::order::PaymentMethod;
use crate::ui::paint;
use crate::ui::theme::{hairline, heavy, medium, regular, CARD, CREAM, INK, MUTED, PEACH, RED};
use crate::ui::widgets::{list_group, nav_bar, section_header, Accessory, KioskButton, ListRow, Nav, Step};

/// Simulated card / e-wallet processing time (demo mode).
const PAYMENT_SIM: Duration = Duration::from_secs(4);

const METHODS: [(PaymentMethod, &str, &str, Color32); 3] = [
    (PaymentMethod::Counter, "💵", "Cash or card at the cashier", Color32::from_rgb(0xE3, 0xF5, 0xE9)),
    (PaymentMethod::Card, "💳", "Tap, insert or swipe your card", Color32::from_rgb(0xE6, 0xEE, 0xFD)),
    (PaymentMethod::EWallet, "📱", "Scan a QR code with your app", Color32::from_rgb(0xF1, 0xE7, 0xFD)),
];

/// Payment method selection, then (for card / e-wallet) a simulated terminal.
#[derive(Default)]
pub struct Payment {
    /// Set while a card / e-wallet payment is being processed.
    processing: Option<(PaymentMethod, Instant)>,
}

impl Payment {
    fn choose_method(&mut self, ctx: &egui::Context, kiosk: &mut Kiosk) -> (Option<Screen>, Option<PaymentMethod>) {
        let nav = Nav { title: "Payment", back: Some("My Order"), step: Some(Step::Pay) };
        let next = nav_bar(ctx, kiosk, nav).then(Screen::review);
        let mut chosen = None;
        let amount = kiosk.money(kiosk.cart.total());
        let count = kiosk.cart.count();
        egui::CentralPanel::default().frame(Frame::new().fill(CREAM).inner_margin(Margin::same(24))).show(ctx, |ui| {
            // Amount summary card.
            Frame::new()
                .fill(CARD)
                .stroke(hairline())
                .corner_radius(CornerRadius::same(24))
                .inner_margin(Margin::same(28))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("Amount due").font(medium(22.0)).color(MUTED));
                        ui.label(RichText::new(&amount).font(heavy(64.0)).color(INK));
                        let summary =
                            format!("{} · {count} item{}", kiosk.order_type.label(), if count == 1 { "" } else { "s" });
                        ui.label(RichText::new(summary).font(regular(20.0)).color(MUTED));
                    });
                });

            section_header(ui, "Choose how to pay", None);
            list_group(ui, |ui| {
                for (k, (method, icon, subtitle, accent)) in METHODS.into_iter().enumerate() {
                    let row = ListRow::new(method.label())
                        .subtitle(subtitle)
                        .icon(&kiosk.icons, icon, accent)
                        .accessory(Accessory::Chevron)
                        .height(124.0)
                        .separator(k != METHODS.len() - 1);
                    if ui.add(row).clicked() {
                        chosen = Some(method);
                    }
                }
            });
            ui.add_space(10.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("Prices include 12% VAT").font(regular(18.0)).color(MUTED));
            });
        });
        (next, chosen)
    }

    /// Returns true if the customer cancelled.
    fn processing(&self, ctx: &egui::Context, kiosk: &mut Kiosk, method: PaymentMethod) -> bool {
        nav_bar(ctx, kiosk, Nav { title: "Payment", back: None, step: Some(Step::Pay) });
        let mut cancel = false;
        let total = kiosk.cart.total();
        let amount = kiosk.money(total);
        egui::CentralPanel::default().frame(Frame::new().fill(CREAM).inner_margin(Margin::same(24))).show(ctx, |ui| {
            let rect = ui.max_rect();
            let card = Rect::from_center_size(rect.center(), vec2(rect.width(), rect.height().min(1150.0)));
            ui.painter().rect_filled(card, CornerRadius::same(28), CARD);
            ui.painter().rect_stroke(card, CornerRadius::same(28), hairline(), StrokeKind::Inside);
            ui.scope_builder(UiBuilder::new().max_rect(card.shrink(40.0)), |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.label(RichText::new("Amount due").font(medium(24.0)).color(MUTED));
                    ui.label(RichText::new(&amount).font(heavy(62.0)).color(INK));
                    ui.add_space(24.0);
                    let (r, _) = ui.allocate_exact_size(vec2(360.0, 360.0), Sense::hover());
                    let (title, subtitle) = if method == PaymentMethod::EWallet {
                        paint::demo_qr(ui.painter(), r, total as u64);
                        ("Scan this QR code", "with your e-wallet app")
                    } else {
                        ui.painter().circle_filled(r.center(), 170.0, PEACH);
                        kiosk.icons.paint(ui.painter(), "💳", r.shrink(80.0));
                        ("Tap, insert or swipe", "your card on the terminal below")
                    };
                    ui.add_space(16.0);
                    ui.label(RichText::new(title).font(heavy(34.0)));
                    ui.label(RichText::new(subtitle).font(regular(23.0)).color(MUTED));
                    ui.add_space(30.0);
                    ui.add(egui::Spinner::new().size(54.0).color(RED));
                    ui.label(RichText::new("Waiting for payment…").font(medium(22.0)).color(MUTED));
                    ui.label(
                        RichText::new("Demo mode: payment is approved automatically").font(regular(17.0)).color(MUTED),
                    );
                    ui.add_space(24.0);
                    cancel = ui.add(KioskButton::secondary("Cancel").size(vec2(300.0, 80.0))).clicked();
                });
            });
        });
        cancel
    }
}

impl View for Payment {
    fn show(&mut self, ctx: &egui::Context, kiosk: &mut Kiosk) -> Option<Screen> {
        match self.processing {
            None => {
                let (next, chosen) = self.choose_method(ctx, kiosk);
                match chosen {
                    Some(PaymentMethod::Counter) => Some(kiosk.place_order(PaymentMethod::Counter)),
                    Some(method) => {
                        self.processing = Some((method, Instant::now()));
                        next
                    }
                    None => next,
                }
            }
            Some((method, since)) => {
                if self.processing(ctx, kiosk, method) {
                    self.processing = None;
                    None
                } else if since.elapsed() >= PAYMENT_SIM {
                    Some(kiosk.place_order(method))
                } else {
                    None
                }
            }
        }
    }

    fn times_out(&self) -> bool {
        self.processing.is_none()
    }
}
