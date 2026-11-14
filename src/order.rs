//! Cart, placed orders, money formatting and order persistence.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum OrderType {
    DineIn,
    TakeOut,
}

impl OrderType {
    pub fn label(self) -> &'static str {
        match self {
            OrderType::DineIn => "Dine In",
            OrderType::TakeOut => "Take Out",
        }
    }

    /// Short confirmation of the customer's choice, e.g. on the review screen.
    pub fn headline(self) -> &'static str {
        match self {
            OrderType::DineIn => "You're eating here",
            OrderType::TakeOut => "You're taking it out",
        }
    }

    /// What happens to the food for this order type.
    pub fn details(self) -> &'static str {
        match self {
            OrderType::DineIn => "We'll serve your order on a tray for dining in.",
            OrderType::TakeOut => "We'll pack your order in a take-out bag.",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PaymentMethod {
    Counter,
    Card,
    EWallet,
}

impl PaymentMethod {
    pub fn label(self) -> &'static str {
        match self {
            PaymentMethod::Counter => "Pay at Counter",
            PaymentMethod::Card => "Card",
            PaymentMethod::EWallet => "E-Wallet",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SelectedOption {
    pub group: String,
    pub choice: String,
    pub price: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CartLine {
    pub item_id: String,
    pub name: String,
    pub icon: String,
    pub base_price: i64,
    pub options: Vec<SelectedOption>,
    pub qty: u32,
}

impl CartLine {
    pub fn unit_price(&self) -> i64 {
        self.base_price + self.options.iter().map(|o| o.price).sum::<i64>()
    }

    pub fn total(&self) -> i64 {
        self.unit_price() * self.qty as i64
    }

    pub fn options_text(&self) -> String {
        self.options.iter().map(|o| o.choice.as_str()).collect::<Vec<_>>().join(" · ")
    }

    fn same_as(&self, other: &CartLine) -> bool {
        self.item_id == other.item_id && self.options == other.options
    }
}

#[derive(Debug, Default)]
pub struct Cart {
    pub lines: Vec<CartLine>,
}

pub const MAX_QTY: u32 = 20;

impl Cart {
    /// Adds a line, merging it with an identical existing line.
    pub fn add(&mut self, line: CartLine) {
        if let Some(existing) = self.lines.iter_mut().find(|l| l.same_as(&line)) {
            existing.qty = (existing.qty + line.qty).min(MAX_QTY);
        } else {
            self.lines.push(line);
        }
    }

    pub fn count(&self) -> u32 {
        self.lines.iter().map(|l| l.qty).sum()
    }

    pub fn total(&self) -> i64 {
        self.lines.iter().map(CartLine::total).sum()
    }

    pub fn contains_item(&self, item_id: &str) -> bool {
        self.lines.iter().any(|l| l.item_id == item_id)
    }

    pub fn clear(&mut self) {
        self.lines.clear();
    }
}

/// 12% VAT already included in a VAT-inclusive total.
pub fn vat_included(total: i64) -> i64 {
    (total * 12 + 56) / 112
}

/// Formats centavos as e.g. `₱1,234.50`.
pub fn format_money(symbol: &str, centavos: i64) -> String {
    let sign = if centavos < 0 { "-" } else { "" };
    let c = centavos.abs();
    let digits = (c / 100).to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    format!("{sign}{symbol}{grouped}.{:02}", c % 100)
}

#[derive(Debug, Clone, Serialize)]
pub struct PlacedOrder {
    pub number: u32,
    pub order_type: OrderType,
    pub payment: PaymentMethod,
    pub paid: bool,
    pub lines: Vec<CartLine>,
    pub total: i64,
    pub placed_at: String,
}

/// Saves orders as JSON lines (for the kitchen / POS) and hands out order numbers.
pub struct OrderStore {
    dir: PathBuf,
}

const FIRST_NUMBER: u32 = 101;
const LAST_NUMBER: u32 = 999;

impl OrderStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    pub fn next_number(&self) -> u32 {
        let path = self.dir.join("counter.txt");
        let last = fs::read_to_string(&path).ok().and_then(|s| s.trim().parse::<u32>().ok());
        let next = match last {
            Some(n) if (FIRST_NUMBER..LAST_NUMBER).contains(&n) => n + 1,
            _ => FIRST_NUMBER,
        };
        let _ = fs::create_dir_all(&self.dir);
        let _ = fs::write(&path, next.to_string());
        next
    }

    pub fn save(&self, order: &PlacedOrder) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        let mut file = OpenOptions::new().create(true).append(true).open(self.dir.join("orders.jsonl"))?;
        writeln!(file, "{}", serde_json::to_string(order)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(id: &str, price: i64, opts: &[(&str, i64)], qty: u32) -> CartLine {
        CartLine {
            item_id: id.into(),
            name: id.into(),
            icon: String::new(),
            base_price: price,
            options: opts
                .iter()
                .map(|(c, p)| SelectedOption { group: "g".into(), choice: (*c).into(), price: *p })
                .collect(),
            qty,
        }
    }

    #[test]
    fn money_format() {
        assert_eq!(format_money("₱", 0), "₱0.00");
        assert_eq!(format_money("₱", 12900), "₱129.00");
        assert_eq!(format_money("₱", 123456789), "₱1,234,567.89");
        assert_eq!(format_money("$", -550), "-$5.50");
    }

    #[test]
    fn cart_merges_identical_lines() {
        let mut cart = Cart::default();
        cart.add(line("a", 10000, &[("Large", 2000)], 1));
        cart.add(line("a", 10000, &[("Large", 2000)], 2));
        cart.add(line("a", 10000, &[("Regular", 0)], 1));
        assert_eq!(cart.lines.len(), 2);
        assert_eq!(cart.count(), 4);
        assert_eq!(cart.total(), 3 * 12000 + 10000);
    }

    #[test]
    fn vat_is_twelve_percent_inclusive() {
        assert_eq!(vat_included(11200), 1200);
    }
}
