//! Item customization, shown as a bottom sheet over the menu or the order review.

use std::time::Instant;

use eframe::egui::{self, vec2, Align, Layout, RichText, ScrollArea};

use crate::kiosk::Kiosk;
use crate::menu::Menu;
use crate::order::{format_money, CartLine, SelectedOption, MAX_QTY};
use crate::ui::theme::{bold, medium, MUTED, RED};
use crate::ui::widgets::{list_group, section_header, Accessory, ItemHero, KioskButton, ListRow, Sheet, Stepper};

/// Result of drawing the sheet for one frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SheetOutcome {
    Open,
    /// Closed with ✕ / backdrop, or the item was added to the cart.
    Closed,
}

pub struct Customize {
    item: usize,
    /// Chosen choice indexes, one list per option group of the item.
    picks: Vec<Vec<usize>>,
    qty: u32,
    opened_at: Instant,
}

impl Customize {
    /// Starts with the first choice of every required group preselected.
    pub fn new(menu: &Menu, item: usize) -> Self {
        let picks = menu.items[item]
            .options
            .iter()
            .map(|gid| if menu.group(gid).required { vec![0] } else { Vec::new() })
            .collect();
        Self { item, picks, qty: 1, opened_at: Instant::now() }
    }

    /// Taps a choice. Single-choice groups switch, multi-choice groups toggle up
    /// to `max`, and required groups always keep at least one pick.
    pub fn toggle(&mut self, menu: &Menu, group_index: usize, choice: usize) -> Result<(), String> {
        let group = menu.group(&menu.items[self.item].options[group_index]);
        let picks = &mut self.picks[group_index];
        if let Some(pos) = picks.iter().position(|&c| c == choice) {
            if !(group.required && picks.len() == 1) {
                picks.remove(pos);
            }
        } else if group.max == 1 {
            *picks = vec![choice];
        } else if picks.len() < group.max {
            picks.push(choice);
            picks.sort_unstable();
        } else {
            return Err(format!("You can choose up to {} {}", group.max, group.title.to_lowercase()));
        }
        Ok(())
    }

    pub fn selected(&self, menu: &Menu) -> Vec<SelectedOption> {
        let item = &menu.items[self.item];
        item.options
            .iter()
            .zip(&self.picks)
            .flat_map(|(gid, picks)| {
                let group = menu.group(gid);
                picks.iter().map(move |&c| SelectedOption {
                    group: group.title.clone(),
                    choice: group.choices[c].name.clone(),
                    price: group.choices[c].price,
                })
            })
            .collect()
    }

    pub fn to_line(&self, menu: &Menu) -> CartLine {
        let item = &menu.items[self.item];
        CartLine {
            item_id: item.id.clone(),
            name: item.name.clone(),
            icon: item.art().to_owned(),
            base_price: item.price,
            options: self.selected(menu),
            qty: self.qty,
        }
    }

    /// Draws the sheet; adds the item to the cart when confirmed.
    pub fn show_sheet(&mut self, ctx: &egui::Context, kiosk: &mut Kiosk) -> SheetOutcome {
        let menu = &kiosk.menu;
        let icons = &kiosk.icons;
        let currency = menu.currency_symbol.as_str();
        let item = &menu.items[self.item];
        let (mut add, mut warning) = (false, None);

        let response = Sheet::new(("customize", self.item), self.opened_at).height_fraction(0.88).show(ctx, |ui| {
            let footer = 176.0;
            ScrollArea::vertical().max_height(ui.available_height() - footer).auto_shrink([false, false]).show(
                ui,
                |ui| {
                    ui.add(ItemHero::new(icons, item, currency));
                    for (gi, gid) in item.options.iter().enumerate() {
                        let group = menu.group(gid);
                        let note = match (group.required, group.max) {
                            (true, 1) => ("Required".to_string(), RED),
                            (true, n) => (format!("Required · up to {n}"), RED),
                            (false, 1) => ("Optional".to_string(), MUTED),
                            (false, n) => (format!("Optional · up to {n}"), MUTED),
                        };
                        section_header(ui, &group.title, Some((&note.0, note.1)));
                        list_group(ui, |ui| {
                            let last = group.choices.len() - 1;
                            for (ci, choice) in group.choices.iter().enumerate() {
                                let selected = self.picks[gi].contains(&ci);
                                let price =
                                    (choice.price > 0).then(|| format!("+{}", format_money(currency, choice.price)));
                                let accessory = if group.max > 1 {
                                    Accessory::Checkbox(selected)
                                } else {
                                    Accessory::Radio(selected)
                                };
                                let row = ListRow::new(&choice.name)
                                    .detail(price.as_deref())
                                    .accessory(accessory)
                                    .separator(ci != last);
                                if ui.add(row).clicked() {
                                    if let Err(e) = self.toggle(menu, gi, ci) {
                                        warning = Some(e);
                                    }
                                }
                            }
                        });
                    }
                    ui.add_space(8.0);
                },
            );

            // Fixed footer: quantity, total and the main action.
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add(Stepper::new(&mut self.qty, 1..=MAX_QTY));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let total = format_money(currency, self.to_line(menu).total());
                    ui.label(RichText::new(total).font(bold(34.0)));
                    ui.label(RichText::new("Total").font(medium(20.0)).color(MUTED));
                });
            });
            let width = ui.available_width();
            add = ui.add(KioskButton::primary("Add to Order").size(vec2(width, 86.0))).clicked();
        });

        if let Some(w) = warning {
            kiosk.show_toast(w);
        }
        if add {
            kiosk.add_to_cart(self.to_line(&kiosk.menu));
            return SheetOutcome::Closed;
        }
        if response.close {
            SheetOutcome::Closed
        } else {
            SheetOutcome::Open
        }
    }
}

/// Shows an optional customize sheet and clears it once closed.
pub fn show_customize_sheet(sheet: &mut Option<Customize>, ctx: &egui::Context, kiosk: &mut Kiosk) {
    if let Some(customize) = sheet {
        if customize.show_sheet(ctx, kiosk) == SheetOutcome::Closed {
            *sheet = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn menu() -> Menu {
        Menu::parse(
            r#"{"store_name":"T","currency_symbol":"$",
                "categories":[{"id":"c","name":"C","icon":"x"}],
                "option_groups":{
                    "size":{"title":"Size","required":true,"choices":[{"name":"Regular"},{"name":"Large","price":20}]},
                    "extras":{"title":"Extras","max":2,"choices":[{"name":"A","price":5},{"name":"B","price":5},{"name":"C","price":5}]}
                },
                "items":[{"id":"i","category":"c","name":"Item","price":100,"icon":"x","options":["size","extras"]}]}"#,
        )
        .unwrap()
    }

    #[test]
    fn required_groups_start_with_first_choice() {
        let menu = menu();
        let c = Customize::new(&menu, 0);
        assert_eq!(c.picks, vec![vec![0], vec![]]);
        assert_eq!(c.to_line(&menu).unit_price(), 10_000);
    }

    #[test]
    fn single_choice_switches_and_cannot_be_cleared_when_required() {
        let menu = menu();
        let mut c = Customize::new(&menu, 0);
        c.toggle(&menu, 0, 1).unwrap();
        assert_eq!(c.picks[0], vec![1]);
        c.toggle(&menu, 0, 1).unwrap();
        assert_eq!(c.picks[0], vec![1], "required pick stays selected");
        assert_eq!(c.to_line(&menu).unit_price(), 12_000);
    }

    #[test]
    fn multi_choice_respects_max_and_toggles_off() {
        let menu = menu();
        let mut c = Customize::new(&menu, 0);
        c.toggle(&menu, 1, 0).unwrap();
        c.toggle(&menu, 1, 2).unwrap();
        assert!(c.toggle(&menu, 1, 1).is_err(), "max of 2 reached");
        c.toggle(&menu, 1, 0).unwrap();
        assert_eq!(c.picks[1], vec![2]);
    }
}
