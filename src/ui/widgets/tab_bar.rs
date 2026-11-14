//! Bottom tab bar, like a mobile app: Home, Menu, Search, My Order.

use eframe::egui::{self, pos2, vec2, Align2, Color32, Frame, Id, Margin, Rect, Sense};

use crate::kiosk::Kiosk;
use crate::ui::paint;
use crate::ui::screens::Screen;
use crate::ui::theme::{bold, hairline, medium, semibold, CARD, FILL, HAIRLINE, MUTED, RED};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Home,
    Menu,
    Search,
    Order,
}

impl Tab {
    const ALL: [Tab; 4] = [Tab::Home, Tab::Menu, Tab::Search, Tab::Order];

    fn label(self) -> &'static str {
        match self {
            Tab::Home => "Home",
            Tab::Menu => "Menu",
            Tab::Search => "Search",
            Tab::Order => "My Order",
        }
    }

    /// Where the tab leads. Home asks for confirmation if the tray isn't empty.
    fn target(self, kiosk: &mut Kiosk) -> Option<Screen> {
        match self {
            Tab::Home => kiosk.request_start_over(),
            Tab::Menu => Some(Screen::menu()),
            Tab::Search => Some(Screen::search()),
            Tab::Order => Some(Screen::review()),
        }
    }
}

/// Draws the tab bar with `active` highlighted. Returns the screen to switch to.
pub fn tab_bar(ctx: &egui::Context, kiosk: &mut Kiosk, active: Tab) -> Option<Screen> {
    let mut picked = None;
    let count = kiosk.cart.count();
    egui::TopBottomPanel::bottom("tab_bar")
        .exact_height(112.0)
        .show_separator_line(false)
        .frame(Frame::new().fill(CARD).inner_margin(Margin::ZERO))
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            ui.painter().hline(rect.x_range(), rect.top() + HAIRLINE / 2.0, hairline());
            let w = rect.width() / Tab::ALL.len() as f32;
            for (k, tab) in Tab::ALL.into_iter().enumerate() {
                let r = Rect::from_min_size(rect.min + vec2(k as f32 * w, 0.0), vec2(w, rect.height()));
                let response = ui.interact(r, Id::new(("tab", k)), Sense::click());
                let p = ui.painter();
                if response.is_pointer_button_down_on() {
                    p.rect_filled(r.shrink2(vec2(14.0, 10.0)), 18, FILL);
                }
                let selected = tab == active;
                let color = if selected { RED } else { MUTED };
                let icon = pos2(r.center().x, r.top() + 42.0);
                match tab {
                    Tab::Home => paint::icon_home(p, icon, 40.0, color),
                    Tab::Menu => paint::icon_grid(p, icon, 40.0, color),
                    Tab::Search => paint::icon_search(p, icon, 40.0, color),
                    Tab::Order => paint::icon_bag(p, icon, 40.0, color),
                }
                if tab == Tab::Order && count > 0 {
                    let badge = icon + vec2(22.0, -16.0);
                    p.circle_filled(badge, 15.0, RED);
                    p.circle_stroke(badge, 15.0, crate::ui::theme::stroke(2.5, CARD));
                    paint::text(p, badge, Align2::CENTER_CENTER, count.to_string(), bold(15.0), Color32::WHITE);
                }
                let font = if selected { semibold(17.0) } else { medium(17.0) };
                paint::text(p, pos2(r.center().x, r.top() + 84.0), Align2::CENTER_CENTER, tab.label(), font, color);
                if response.clicked() && !selected {
                    picked = Some(tab);
                }
            }
        });
    picked.and_then(|tab| tab.target(kiosk))
}
