use eframe::egui::{self, vec2, Align, Color32, Event, Frame, Key, Layout, Margin, RichText, ScrollArea, Sense};

use super::customize::show_customize_sheet;
use super::{Customize, Screen, View};
use crate::kiosk::Kiosk;
use crate::menu::{Menu, Promo};
use crate::ui::theme::{bold, heavy, medium, regular, CREAM, INK, MUTED, PEACH};
use crate::ui::widgets::{
    checkout_bar, item_grid, nav_bar, tab_bar, CategoryChip, Keyboard, KeyboardAction, Nav, PromoBanner, SearchField,
    Step, Tab,
};

const GRID_MIN_WIDTH: f32 = 250.0;
const KEYBOARD_BG: Color32 = Color32::from_rgb(0xDF, 0xE0, 0xE5);

/// The menu: search, category chips, promo banner and item grid.
/// Choosing an item with options opens the customize sheet on top.
#[derive(Default)]
pub struct MenuBrowser {
    query: String,
    /// On-screen keyboard is open.
    keyboard: bool,
    sheet: Option<Customize>,
}

impl MenuBrowser {
    pub fn searching() -> Self {
        Self { keyboard: true, ..Self::default() }
    }

    fn is_searching(&self) -> bool {
        self.keyboard || !self.query.is_empty()
    }

    /// Lets a physical keyboard type into the search too (handy on a desktop).
    fn read_physical_keys(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            for event in &i.events {
                match event {
                    Event::Text(text) => self.query.push_str(&text.to_lowercase()),
                    Event::Key { key: Key::Backspace, pressed: true, .. } => {
                        self.query.pop();
                    }
                    Event::Key { key: Key::Enter | Key::Escape, pressed: true, .. } => self.keyboard = false,
                    _ => {}
                }
            }
        });
    }
}

impl View for MenuBrowser {
    fn show(&mut self, ctx: &egui::Context, kiosk: &mut Kiosk) -> Option<Screen> {
        let title = if self.is_searching() { "Search" } else { "Menu" };
        let nav = Nav { title, back: Some("Back"), step: Some(Step::Menu) };
        let mut next = nav_bar(ctx, kiosk, nav).then(Screen::order_type);

        if self.keyboard {
            if self.sheet.is_none() {
                self.read_physical_keys(ctx);
            }
            egui::TopBottomPanel::bottom("keyboard")
                .exact_height(4.0 * 76.0 + 3.0 * 8.0 + 40.0)
                .show_separator_line(false)
                .frame(Frame::new().fill(KEYBOARD_BG).inner_margin(Margin::same(20)))
                .show(ctx, |ui| {
                    if Keyboard::new(&mut self.query).show(ui) == KeyboardAction::Done {
                        self.keyboard = false;
                    }
                });
        } else {
            let active = if self.query.is_empty() { Tab::Menu } else { Tab::Search };
            next = tab_bar(ctx, kiosk, active).or(next);
            if checkout_bar(ctx, kiosk) {
                next = Some(Screen::review());
            }
        }

        let mut tapped = None;
        egui::CentralPanel::default().frame(Frame::new().fill(CREAM).inner_margin(Margin::same(24))).show(ctx, |ui| {
            if ui.add(SearchField::new(&self.query, self.keyboard)).clicked() {
                self.keyboard = true;
            }
            ui.add_space(4.0);

            if self.is_searching() {
                tapped = search_results(ui, kiosk, &self.query);
                return;
            }

            // Category chips scroll horizontally, like a mobile app.
            let menu = &kiosk.menu;
            ScrollArea::horizontal().id_salt("category_chips").show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    for (i, category) in menu.categories.iter().enumerate() {
                        if ui.add(CategoryChip::new(&kiosk.icons, category, i == kiosk.category)).clicked() {
                            kiosk.category = i;
                        }
                    }
                });
            });

            let category = &menu.categories[kiosk.category];
            let items = menu.items_in(category);
            ScrollArea::vertical().id_salt(("menu_grid", kiosk.category)).auto_shrink([false; 2]).show(ui, |ui| {
                ui.add_space(6.0);
                if kiosk.category == 0 {
                    if let Some((promo, i)) = promo(menu) {
                        if ui.add(PromoBanner::new(&kiosk.icons, promo, menu.items[i].art())).clicked() {
                            tapped = Some(i);
                        }
                        ui.add_space(6.0);
                    }
                }
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&category.name).font(heavy(36.0)).color(INK));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let n = items.len();
                        let label = format!("{n} item{}", if n == 1 { "" } else { "s" });
                        ui.label(RichText::new(label).font(medium(19.0)).color(MUTED));
                    });
                });
                if let Some(i) = item_grid(ui, &kiosk.icons, menu, &items, GRID_MIN_WIDTH) {
                    tapped = Some(i);
                }
                ui.add_space(16.0);
            });
        });

        if let Some(i) = tapped {
            self.keyboard = false;
            self.sheet = kiosk.add_or_customize(i);
        }
        show_customize_sheet(&mut self.sheet, ctx, kiosk);
        next
    }
}

fn promo(menu: &Menu) -> Option<(&Promo, usize)> {
    let promo = menu.promo.as_ref()?;
    Some((promo, menu.item_index(&promo.item)?))
}

/// Matching items across all categories (popular items when the query is empty).
fn search_results(ui: &mut egui::Ui, kiosk: &Kiosk, query: &str) -> Option<usize> {
    let menu = &kiosk.menu;
    let query = query.trim().to_lowercase();
    let (heading, items): (String, Vec<usize>) = if query.is_empty() {
        let popular = (0..menu.items.len()).filter(|&i| menu.items[i].tag.is_some()).collect();
        ("Popular right now".to_owned(), popular)
    } else {
        let found: Vec<usize> = (0..menu.items.len())
            .filter(|&i| {
                let item = &menu.items[i];
                item.name.to_lowercase().contains(&query) || item.description.to_lowercase().contains(&query)
            })
            .collect();
        let n = found.len();
        (format!("{n} result{} for “{query}”", if n == 1 { "" } else { "s" }), found)
    };

    let mut tapped = None;
    ScrollArea::vertical().id_salt("search_results").auto_shrink([false; 2]).show(ui, |ui| {
        ui.add_space(6.0);
        ui.label(RichText::new(heading).font(bold(26.0)).color(INK));
        if items.is_empty() {
            ui.add_space(60.0);
            ui.vertical_centered(|ui| {
                let (r, _) = ui.allocate_exact_size(vec2(150.0, 150.0), Sense::hover());
                ui.painter().circle_filled(r.center(), 75.0, PEACH);
                kiosk.icons.paint(ui.painter(), "🍽", r.shrink(36.0));
                ui.label(RichText::new("No matches").font(heavy(30.0)).color(INK));
                ui.label(RichText::new("Try “chicken”, “fries” or “sundae”.").font(regular(21.0)).color(MUTED));
            });
        } else {
            tapped = item_grid(ui, &kiosk.icons, menu, &items, GRID_MIN_WIDTH);
        }
        ui.add_space(16.0);
    });
    tapped
}
