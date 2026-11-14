//! Reusable kiosk components.
//!
//! Most components implement [`egui::Widget`] and are configured with builder
//! methods, e.g. `ui.add(KioskButton::primary("Pay").size(vec2(300.0, 80.0)))`.
//! App chrome (navigation bar, tab bar, bottom bars) lives in `nav`, `tab_bar` and `bars`.

mod banners;
mod bars;
mod button;
mod cards;
mod cart_line;
mod category;
mod chips;
mod dialog;
mod list;
mod nav;
mod search;
mod sheet;
mod stepper;
mod suggestion;
mod tab_bar;

pub use banners::{ItemHero, PromoBanner};
pub use bars::{bottom_bar, checkout_bar};
pub use button::KioskButton;
pub use cards::{item_grid, Tile};
pub use cart_line::{CartLineRow, LineAction};
pub use category::CategoryChip;
pub use chips::{Chip, OrderTypeSwitch, Pill};
pub use dialog::toast;
pub use list::{group_header, list_group, section_header, Accessory, Avatar, ListRow};
pub use nav::{nav_bar, Nav, Step};
pub use search::{Keyboard, KeyboardAction, SearchField};
pub use sheet::Sheet;
pub use stepper::Stepper;
pub use suggestion::{pitch, SuggestionRow};
pub use tab_bar::{tab_bar, Tab};

use eframe::egui::{Rect, Response};

/// Number of columns that fit `min_width` cells, and the resulting cell width.
pub fn grid_columns(available: f32, min_width: f32, gap: f32) -> (usize, f32) {
    let cols = (((available + gap) / (min_width + gap)).floor() as usize).max(1);
    let width = ((available - gap * (cols - 1) as f32) / cols as f32).floor();
    (cols, width)
}

/// Shrinks a rect slightly while it is being pressed, for tactile feedback.
fn press_rect(rect: Rect, response: &Response, amount: f32) -> Rect {
    if response.is_pointer_button_down_on() {
        rect.shrink(amount)
    } else {
        rect
    }
}

#[cfg(test)]
mod tests {
    use super::grid_columns;

    #[test]
    fn grid_columns_fit_available_width() {
        assert_eq!(grid_columns(668.0, 190.0, 16.0), (3, 212.0));
        assert_eq!(grid_columns(100.0, 190.0, 16.0), (1, 100.0));
    }
}
