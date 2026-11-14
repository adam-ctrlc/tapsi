//! Kiosk screens. Each screen owns its own state and implements [`View`].
//!
//! Attract -> OrderTypePicker -> MenuBrowser (+ Customize sheet) -> Review -> Payment -> Done

mod attract;
mod customize;
mod done;
mod menu;
mod order_type;
mod payment;
mod review;

pub use attract::Attract;
pub use customize::Customize;
pub use done::Done;
pub use menu::MenuBrowser;
pub use order_type::OrderTypePicker;
pub use payment::Payment;
pub use review::Review;

use eframe::egui;

use crate::kiosk::Kiosk;

/// A full-screen page of the kiosk.
pub trait View {
    /// Draws the screen. Returns the screen to switch to, if the customer navigated.
    fn show(&mut self, ctx: &egui::Context, kiosk: &mut Kiosk) -> Option<Screen>;

    /// Whether an idle customer should be asked "Are you still there?" here.
    fn times_out(&self) -> bool {
        true
    }

    /// Whether the screen animates continuously (needs a high repaint rate).
    fn animated(&self) -> bool {
        false
    }
}

pub enum Screen {
    Attract(Attract),
    OrderType(OrderTypePicker),
    Menu(MenuBrowser),
    Review(Review),
    Payment(Payment),
    Done(Done),
}

impl Screen {
    pub fn attract() -> Self {
        Self::Attract(Attract)
    }

    pub fn order_type() -> Self {
        Self::OrderType(OrderTypePicker)
    }

    pub fn menu() -> Self {
        Self::Menu(MenuBrowser::default())
    }

    /// The menu with the search keyboard open.
    pub fn search() -> Self {
        Self::Menu(MenuBrowser::searching())
    }

    pub fn review() -> Self {
        Self::Review(Review::default())
    }

    pub fn payment() -> Self {
        Self::Payment(Payment::default())
    }

    pub fn is_attract(&self) -> bool {
        matches!(self, Self::Attract(_))
    }

    fn view(&self) -> &dyn View {
        match self {
            Self::Attract(s) => s,
            Self::OrderType(s) => s,
            Self::Menu(s) => s,
            Self::Review(s) => s,
            Self::Payment(s) => s,
            Self::Done(s) => s,
        }
    }

    fn view_mut(&mut self) -> &mut dyn View {
        match self {
            Self::Attract(s) => s,
            Self::OrderType(s) => s,
            Self::Menu(s) => s,
            Self::Review(s) => s,
            Self::Payment(s) => s,
            Self::Done(s) => s,
        }
    }
}

impl View for Screen {
    fn show(&mut self, ctx: &egui::Context, kiosk: &mut Kiosk) -> Option<Screen> {
        self.view_mut().show(ctx, kiosk)
    }

    fn times_out(&self) -> bool {
        self.view().times_out()
    }

    fn animated(&self) -> bool {
        self.view().animated()
    }
}
