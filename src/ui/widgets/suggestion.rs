//! Conversational upsell: friendly suggestion rows.

use eframe::egui::{pos2, vec2, Align2, CornerRadius, Rect, Response, Sense, Ui, Vec2, Widget};

use super::press_rect;
use crate::menu::MenuItem;
use crate::ui::icons::Icons;
use crate::ui::paint;
use crate::ui::theme::{bold, hairline, regular, semibold, HAIRLINE, INK, MUTED, PEACH, RED, RED_TINT};

/// A friendly pitch for an upsell item, based on its category.
pub fn pitch(item: &MenuItem, price: &str) -> (String, String) {
    let name = &item.name;
    match item.category.as_str() {
        "sides" => ("Craving something crispy?".into(), format!("Add {name} for just {price}.")),
        "drinks" => ("Thirsty?".into(), format!("An ice-cold {name} goes great with your meal.")),
        "desserts" => ("Save room for dessert!".into(), format!("A {name} is the perfect sweet finish.")),
        _ => ("You might also like".into(), format!("{name}, only {price}.")),
    }
}

/// One suggestion: artwork, a friendly line, the price and an "Add" button.
/// The whole row is tappable. Place rows inside a [`super::list_group`].
#[must_use = "add it with `ui.add(...)`"]
pub struct SuggestionRow<'a> {
    icons: &'a Icons,
    item: &'a MenuItem,
    headline: &'a str,
    body: &'a str,
}

impl<'a> SuggestionRow<'a> {
    pub fn new(icons: &'a Icons, item: &'a MenuItem, headline: &'a str, body: &'a str) -> Self {
        Self { icons, item, headline, body }
    }
}

impl Widget for SuggestionRow<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 128.0), Sense::click());
        let p = ui.painter();
        if response.is_pointer_button_down_on() {
            p.rect_filled(rect.shrink2(vec2(6.0, 3.0)), CornerRadius::same(16), PEACH);
        }

        let art = Rect::from_min_size(pos2(rect.left() + 20.0, rect.center().y - 44.0), Vec2::splat(88.0));
        p.rect_filled(art, CornerRadius::same(18), PEACH);
        self.icons.paint(p, self.item.art(), art.shrink(12.0));

        let button = Rect::from_center_size(pos2(rect.right() - 74.0, rect.center().y), vec2(112.0, 54.0));
        let button = press_rect(button, &response, 2.0);
        p.rect_filled(button, CornerRadius::same(27), RED_TINT);
        paint::text(p, button.center(), Align2::CENTER_CENTER, "+  Add", bold(20.0), RED);

        let x = art.right() + 20.0;
        let text_width = button.left() - 16.0 - x;
        paint::text(p, pos2(x, rect.center().y - 22.0), Align2::LEFT_CENTER, self.headline, semibold(22.0), INK);
        let body = p.layout(self.body.to_owned(), regular(18.0), MUTED, text_width);
        p.galley(pos2(x, rect.center().y - 4.0), body, MUTED);

        p.hline(x..=rect.right(), rect.bottom() - HAIRLINE / 2.0, hairline());
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(category: &str) -> MenuItem {
        serde_json::from_str(&format!(r#"{{"id":"x","category":"{category}","name":"Fries","price":49,"icon":"🍟"}}"#))
            .unwrap()
    }

    #[test]
    fn pitch_depends_on_category() {
        assert_eq!(pitch(&item("sides"), "₱49.00").1, "Add Fries for just ₱49.00.");
        assert_eq!(pitch(&item("drinks"), "₱49.00").0, "Thirsty?");
        assert_eq!(pitch(&item("pasta"), "₱49.00").0, "You might also like");
    }
}
