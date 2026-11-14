//! Shared kiosk state: menu, artwork, the current customer's order and order placement.
//!
//! Screens receive `&mut Kiosk`; screen-specific state lives in the screen itself.

use std::time::{Duration, Instant};

use eframe::egui;

use crate::menu::{Menu, MenuItem};
use crate::order::{format_money, Cart, CartLine, OrderStore, OrderType, PaymentMethod, PlacedOrder};
use crate::ui::icons::Icons;
use crate::ui::screens::{Customize, Done, Screen};

const TOAST_FOR: Duration = Duration::from_millis(1800);
const BUMP_FOR: f32 = 0.45;

/// Artwork used by the UI itself (menu art is preloaded from the menu).
const UI_ART: &[&str] = &["🍽", "🥡", "💵", "💳", "📱", "🛒", "👆", "🍗", "🍔", "🍝", "🍟", "🍦", "🍹"];

struct Toast {
    message: String,
    shown_at: Instant,
}

pub struct Kiosk {
    pub menu: Menu,
    /// Shown on the start screen when a custom `menu.json` could not be used.
    pub menu_warning: Option<String>,
    pub icons: Icons,
    store: OrderStore,
    pub cart: Cart,
    pub order_type: OrderType,
    /// Selected category index on the menu screen.
    pub category: usize,
    /// When the "Start over?" sheet was opened, if it is open.
    pub confirm_cancel: Option<Instant>,
    /// App start time, the clock for continuous animations.
    pub started: Instant,
    toast: Option<Toast>,
    cart_bump: Option<Instant>,
}

impl Kiosk {
    pub fn new(ctx: &egui::Context) -> Self {
        let (menu, menu_warning) = Menu::load();
        if let Some(w) = &menu_warning {
            eprintln!("{w}");
        }

        // Decode all artwork up front so screens never stutter on first show.
        let icons = Icons::new();
        let art = menu
            .items
            .iter()
            .map(MenuItem::art)
            .chain(menu.categories.iter().map(|c| c.icon.as_str()))
            .chain(UI_ART.iter().copied());
        icons.preload(ctx, art);

        Self {
            menu,
            menu_warning,
            icons,
            store: OrderStore::new("orders"),
            cart: Cart::default(),
            order_type: OrderType::DineIn,
            category: 0,
            confirm_cancel: None,
            started: Instant::now(),
            toast: None,
            cart_bump: None,
        }
    }

    pub fn money(&self, centavos: i64) -> String {
        format_money(&self.menu.currency_symbol, centavos)
    }

    pub fn show_toast(&mut self, message: impl Into<String>) {
        self.toast = Some(Toast { message: message.into(), shown_at: Instant::now() });
    }

    /// The current toast message, if it has not expired.
    pub fn toast(&mut self) -> Option<&str> {
        if self.toast.as_ref().is_some_and(|t| t.shown_at.elapsed() > TOAST_FOR) {
            self.toast = None;
        }
        self.toast.as_ref().map(|t| t.message.as_str())
    }

    pub fn add_to_cart(&mut self, line: CartLine) {
        self.show_toast(format!("{} added to your order", line.name));
        self.cart.add(line);
        self.cart_bump = Some(Instant::now());
    }

    /// Items without options go straight into the cart; others return a customize sheet to show.
    pub fn add_or_customize(&mut self, item: usize) -> Option<Customize> {
        let customize = Customize::new(&self.menu, item);
        if self.menu.items[item].options.is_empty() {
            self.add_to_cart(customize.to_line(&self.menu));
            None
        } else {
            Some(customize)
        }
    }

    /// Saves the order, empties the cart and returns the order-number screen.
    pub fn place_order(&mut self, payment: PaymentMethod) -> Screen {
        let order = PlacedOrder {
            number: self.store.next_number(),
            order_type: self.order_type,
            payment,
            paid: payment != PaymentMethod::Counter,
            total: self.cart.total(),
            lines: std::mem::take(&mut self.cart.lines),
            placed_at: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
        };
        if let Err(e) = self.store.save(&order) {
            eprintln!("failed to save order #{}: {e}", order.number);
        }
        Screen::Done(Done::new(order))
    }

    /// "Start Over": leave right away if the tray is empty, otherwise ask first.
    pub fn request_start_over(&mut self) -> Option<Screen> {
        if self.cart.lines.is_empty() {
            Some(Screen::attract())
        } else {
            self.confirm_cancel = Some(Instant::now());
            None
        }
    }

    /// Clears everything about the current customer.
    pub fn reset_session(&mut self) {
        self.cart.clear();
        self.category = 0;
        self.confirm_cancel = None;
        self.toast = None;
        self.cart_bump = None;
    }

    /// Scale for the cart badge: a short bounce after an item is added.
    pub fn cart_bump_scale(&self) -> f32 {
        match self.cart_bump.map(|at| at.elapsed().as_secs_f32()) {
            Some(t) if t < BUMP_FOR => 1.0 + 0.35 * (1.0 - t / BUMP_FOR) * (t * 22.0).sin().abs(),
            _ => 1.0,
        }
    }
}
