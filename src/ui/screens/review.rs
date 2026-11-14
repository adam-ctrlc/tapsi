use eframe::egui::{self, vec2, Align, Frame, Layout, Margin, RichText, ScrollArea, Sense};

use super::customize::show_customize_sheet;
use super::{Customize, Screen, View};
use crate::kiosk::Kiosk;
use crate::order::{format_money, vat_included};
use crate::ui::paint;
use crate::ui::theme::{heavy, medium, regular, semibold, CREAM, INK, MUTED, PEACH};
use crate::ui::widgets::{
    bottom_bar, group_header, list_group, nav_bar, pitch, section_header, tab_bar, Avatar, CartLineRow, KioskButton,
    LineAction, Nav, OrderTypeSwitch, Step, SuggestionRow, Tab,
};

const MAX_SUGGESTIONS: usize = 3;

/// "My Order": order type, cart lines, upsell suggestions and the total.
#[derive(Default)]
pub struct Review {
    sheet: Option<Customize>,
    /// Suggestions stay hidden until asked for, so they aren't mistaken for the order.
    show_suggestions: bool,
}

impl View for Review {
    fn show(&mut self, ctx: &egui::Context, kiosk: &mut Kiosk) -> Option<Screen> {
        let nav = Nav { title: "My Order", back: Some("Menu"), step: Some(Step::Review) };
        let mut next = nav_bar(ctx, kiosk, nav).then(Screen::menu);
        next = tab_bar(ctx, kiosk, Tab::Order).or(next);

        let currency = kiosk.menu.currency_symbol.clone();
        let total = kiosk.cart.total();
        let empty = kiosk.cart.lines.is_empty();
        bottom_bar(ctx, "review_bar", 188.0, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    let n = kiosk.cart.count();
                    let label = format!("Total · {n} item{}", if n == 1 { "" } else { "s" });
                    ui.label(RichText::new(label).font(medium(21.0)).color(INK));
                    let vat = format!("VAT included: {}", format_money(&currency, vat_included(total)));
                    ui.label(RichText::new(vat).font(regular(17.0)).color(MUTED));
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new(format_money(&currency, total)).font(heavy(40.0)).color(INK));
                });
            });
            let width = ui.available_width();
            let pay = KioskButton::primary("Proceed to Payment").size(vec2(width, 84.0)).enabled(!empty);
            if ui.add(pay).clicked() {
                next = Some(Screen::payment());
            }
        });

        let (mut action, mut tapped) = (None, None);
        egui::CentralPanel::default().frame(Frame::new().fill(CREAM).inner_margin(Margin::same(24))).show(ctx, |ui| {
            ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                section_header(ui, "How will you enjoy your order?", None);
                list_group(ui, |ui| {
                    ui.add(OrderTypeSwitch::new(&mut kiosk.order_type));
                    let kind = kiosk.order_type;
                    Frame::new().inner_margin(Margin::symmetric(24, 16)).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.spacing_mut().item_spacing.y = 4.0;
                        ui.label(RichText::new(kind.headline()).font(semibold(23.0)).color(INK));
                        ui.label(RichText::new(kind.details()).font(regular(19.0)).color(MUTED));
                    });
                });

                if empty {
                    ui.add_space(110.0);
                    ui.vertical_centered(|ui| {
                        let (r, _) = ui.allocate_exact_size(vec2(170.0, 170.0), Sense::hover());
                        ui.painter().circle_filled(r.center(), 85.0, PEACH);
                        kiosk.icons.paint(ui.painter(), "🛒", r.shrink(40.0));
                        ui.label(RichText::new("Your tray is empty").font(heavy(32.0)).color(INK));
                        ui.label(
                            RichText::new("Browse the menu and tap an item to add it.")
                                .font(regular(21.0))
                                .color(MUTED),
                        );
                        ui.add_space(12.0);
                        if ui.add(KioskButton::tinted("Browse Menu").size(vec2(280.0, 76.0))).clicked() {
                            next = Some(Screen::menu());
                        }
                    });
                    return;
                }

                ui.add_space(14.0);
                list_group(ui, |ui| {
                    let n = kiosk.cart.count();
                    let body = format!(
                        "{n} item{} so far. Change quantities or remove anything below.",
                        if n == 1 { "" } else { "s" }
                    );
                    group_header(ui, Avatar::Icon(paint::icon_bag), "Here's your order", &body);
                    let last = kiosk.cart.lines.len() - 1;
                    for (i, line) in kiosk.cart.lines.iter().enumerate() {
                        match CartLineRow::new(&kiosk.icons, line, &currency).separator(i != last).show(ui) {
                            LineAction::None => {}
                            other => action = Some((i, other)),
                        }
                    }
                });

                let suggestions = suggestions(kiosk);
                if let (false, Some(first)) = (suggestions.is_empty(), kiosk.cart.lines.first()) {
                    ui.add_space(14.0);
                    let body = if self.show_suggestions {
                        format!("Here are a few favorites that go great with your {}.", first.name)
                    } else {
                        "Want a drink, side or dessert with that? We can suggest a few favorites.".to_owned()
                    };
                    list_group(ui, |ui| {
                        group_header(ui, Avatar::Mascot, "Make it a complete meal?", &body);
                        if self.show_suggestions {
                            for &i in &suggestions {
                                let item = &kiosk.menu.items[i];
                                let (headline, body) = pitch(item, &kiosk.money(item.price));
                                if ui.add(SuggestionRow::new(&kiosk.icons, item, &headline, &body)).clicked() {
                                    tapped = Some(i);
                                }
                            }
                        }
                        Frame::new().inner_margin(Margin::symmetric(20, 14)).show(ui, |ui| {
                            let label = if self.show_suggestions { "Hide suggestions" } else { "Show suggestions" };
                            let width = ui.available_width();
                            if ui.add(KioskButton::tinted(label).size(vec2(width, 68.0))).clicked() {
                                self.show_suggestions = !self.show_suggestions;
                            }
                        });
                    });
                }
                ui.add_space(16.0);
            });
        });

        match action {
            Some((i, LineAction::SetQty(q))) => kiosk.cart.lines[i].qty = q,
            Some((i, LineAction::Remove)) => {
                kiosk.cart.lines.remove(i);
            }
            _ => {}
        }
        if let Some(i) = tapped {
            self.sheet = kiosk.add_or_customize(i);
        }
        show_customize_sheet(&mut self.sheet, ctx, kiosk);
        next
    }
}

/// Upsell items that are available and not already in the cart.
fn suggestions(kiosk: &Kiosk) -> Vec<usize> {
    kiosk
        .menu
        .upsell
        .iter()
        .filter_map(|id| kiosk.menu.item_index(id))
        .filter(|&i| {
            let item = &kiosk.menu.items[i];
            item.available && !kiosk.cart.contains_item(&item.id)
        })
        .take(MAX_SUGGESTIONS)
        .collect()
}
