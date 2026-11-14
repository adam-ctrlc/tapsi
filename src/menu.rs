//! Menu data, loaded from `menu.json` (falls back to the built-in menu).

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Deserializer};

const BUILTIN_MENU: &str = include_str!("../assets/menu.json");

#[derive(Debug, Clone, Deserialize)]
pub struct Menu {
    pub store_name: String,
    #[serde(default)]
    pub tagline: String,
    pub currency_symbol: String,
    pub categories: Vec<Category>,
    #[serde(default)]
    pub option_groups: HashMap<String, OptionGroup>,
    pub items: Vec<MenuItem>,
    /// Item ids suggested on the review screen ("Complete your meal").
    #[serde(default)]
    pub upsell: Vec<String>,
    /// Optional banner shown above the menu grid.
    #[serde(default)]
    pub promo: Option<Promo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Promo {
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    /// Item opened when the banner is tapped.
    pub item: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub icon: String,
    /// When true, this category lists every item that has a `tag`
    /// (e.g. "Best Seller", "New") instead of items assigned to it.
    #[serde(default)]
    pub show_tagged: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MenuItem {
    pub id: String,
    pub category: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// Price in centavos (written as pesos in JSON, e.g. 129 or 59.50).
    #[serde(deserialize_with = "pesos")]
    pub price: i64,
    /// Emoji used for artwork (loaded as a color image from `assets/icons`).
    pub icon: String,
    /// Optional product photo, relative to `assets/` (e.g. "images/chicken.jpg").
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    /// Ids into `Menu::option_groups`, shown in this order.
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default = "default_true")]
    pub available: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OptionGroup {
    pub title: String,
    #[serde(default)]
    pub required: bool,
    /// Maximum number of choices the customer can pick (1 = single choice).
    #[serde(default = "default_one")]
    pub max: usize,
    pub choices: Vec<Choice>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Choice {
    pub name: String,
    /// Extra cost in centavos (written as pesos in JSON).
    #[serde(default, deserialize_with = "pesos")]
    pub price: i64,
}

fn default_true() -> bool {
    true
}

fn default_one() -> usize {
    1
}

fn pesos<'de, D: Deserializer<'de>>(d: D) -> Result<i64, D::Error> {
    let value = f64::deserialize(d)?;
    Ok((value * 100.0).round() as i64)
}

impl MenuItem {
    /// Art key for this item: its photo if set, otherwise its emoji.
    pub fn art(&self) -> &str {
        self.image.as_deref().unwrap_or(&self.icon)
    }
}

impl Menu {
    /// Loads `menu.json` from the working directory or next to the executable.
    /// Returns the menu and a warning if a custom menu existed but was invalid.
    pub fn load() -> (Menu, Option<String>) {
        let mut candidates = vec![PathBuf::from("menu.json")];
        if let Some(dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())) {
            candidates.push(dir.join("menu.json"));
        }

        let mut warning = None;
        for path in candidates {
            let Ok(text) = std::fs::read_to_string(&path) else { continue };
            match Self::parse(&text) {
                Ok(menu) => return (menu, None),
                Err(e) => {
                    warning = Some(format!("{} is invalid ({e}); using the built-in menu.", path.display()));
                    break;
                }
            }
        }
        let menu = Self::parse(BUILTIN_MENU).expect("built-in menu must be valid");
        (menu, warning)
    }

    pub fn parse(text: &str) -> Result<Menu, String> {
        let menu: Menu = serde_json::from_str(text).map_err(|e| e.to_string())?;
        menu.validate()?;
        Ok(menu)
    }

    fn validate(&self) -> Result<(), String> {
        if self.categories.is_empty() {
            return Err("menu has no categories".into());
        }
        for item in &self.items {
            if !self.categories.iter().any(|c| c.id == item.category) {
                return Err(format!("item '{}' has unknown category '{}'", item.id, item.category));
            }
            for gid in &item.options {
                let group = self
                    .option_groups
                    .get(gid)
                    .ok_or_else(|| format!("item '{}' uses unknown option group '{gid}'", item.id))?;
                if group.choices.is_empty() || group.max == 0 {
                    return Err(format!("option group '{gid}' needs choices and max >= 1"));
                }
            }
        }
        for id in &self.upsell {
            if self.item_index(id).is_none() {
                return Err(format!("upsell refers to unknown item '{id}'"));
            }
        }
        if let Some(promo) = &self.promo {
            if self.item_index(&promo.item).is_none() {
                return Err(format!("promo refers to unknown item '{}'", promo.item));
            }
        }
        Ok(())
    }

    pub fn group(&self, id: &str) -> &OptionGroup {
        &self.option_groups[id]
    }

    pub fn item_index(&self, id: &str) -> Option<usize> {
        self.items.iter().position(|i| i.id == id)
    }

    /// Indexes of items shown under the given category.
    pub fn items_in(&self, category: &Category) -> Vec<usize> {
        self.items
            .iter()
            .enumerate()
            .filter(|(_, item)| if category.show_tagged { item.tag.is_some() } else { item.category == category.id })
            .map(|(i, _)| i)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_menu_is_valid() {
        let menu = Menu::parse(BUILTIN_MENU).unwrap();
        assert!(!menu.items.is_empty());
        for cat in &menu.categories {
            assert!(!menu.items_in(cat).is_empty(), "category '{}' is empty", cat.id);
        }
    }

    #[test]
    fn prices_are_centavos() {
        let menu = Menu::parse(
            r#"{"store_name":"T","currency_symbol":"$",
                "categories":[{"id":"a","name":"A","icon":"x"}],
                "items":[{"id":"i","category":"a","name":"I","price":59.5,"icon":"x"}]}"#,
        )
        .unwrap();
        assert_eq!(menu.items[0].price, 5950);
    }

    #[test]
    fn rejects_unknown_option_group() {
        let err = Menu::parse(
            r#"{"store_name":"T","currency_symbol":"$",
                "categories":[{"id":"a","name":"A","icon":"x"}],
                "items":[{"id":"i","category":"a","name":"I","price":1,"icon":"x","options":["nope"]}]}"#,
        )
        .unwrap_err();
        assert!(err.contains("nope"));
    }
}
