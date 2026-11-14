//! The eframe application shell: scaling, idle timeout, screen transitions and dialogs.

use std::mem::{discriminant, Discriminant};
use std::time::{Duration, Instant};

use eframe::egui::{self, vec2, Color32, CornerRadius, Event, Id, Key, LayerId, Order, RichText, ViewportCommand};

use crate::kiosk::Kiosk;
use crate::ui::fit_to_window;
use crate::ui::paint;
use crate::ui::screens::{Screen, View};
use crate::ui::theme::{self, CREAM};
use crate::ui::theme::{regular, MUTED};
use crate::ui::widgets::{toast, KioskButton, Sheet};

/// No touch for this long shows "Are you still there?".
const IDLE_AFTER: Duration = Duration::from_secs(60);
/// ...and this much longer cancels the order.
const IDLE_GRACE: Duration = Duration::from_secs(20);
/// Fade-in duration when switching screens.
const FADE_SECS: f32 = 0.22;

pub struct KioskApp {
    kiosk: Kiosk,
    screen: Screen,
    last_input: Instant,
    screen_since: Instant,
}

impl KioskApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install(&cc.egui_ctx);
        // The kiosk controls its own scale; disable Ctrl +/- zooming.
        cc.egui_ctx.options_mut(|o| o.zoom_with_keyboard = false);
        Self {
            kiosk: Kiosk::new(&cc.egui_ctx),
            screen: Screen::attract(),
            last_input: Instant::now(),
            screen_since: Instant::now(),
        }
    }

    fn handle_keys(&self, ctx: &egui::Context) {
        let (quit, toggle_full, full) = ctx.input(|i| {
            (
                // Hidden staff shortcut to exit the kiosk.
                i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(Key::Q),
                i.key_pressed(Key::F11),
                i.viewport().fullscreen.unwrap_or(false),
            )
        });
        if quit {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
        if toggle_full {
            ctx.send_viewport_cmd(ViewportCommand::Fullscreen(!full));
        }
    }

    /// Cancel confirmation, idle warning and toast. Returns true to reset the session.
    fn overlays(&mut self, ctx: &egui::Context) -> bool {
        let mut reset = false;

        if let Some(opened_at) = self.kiosk.confirm_cancel {
            let mut keep = false;
            let sheet = Sheet::new("confirm_cancel", opened_at).title("Start over?").show(ctx, |ui| {
                ui.label(RichText::new("All items in your tray will be removed.").font(regular(22.0)).color(MUTED));
                ui.add_space(8.0);
                let width = ui.available_width();
                reset = ui.add(KioskButton::primary("Yes, Start Over").size(vec2(width, 84.0))).clicked();
                keep = ui.add(KioskButton::secondary("Keep Ordering").size(vec2(width, 84.0))).clicked();
            });
            if keep || sheet.close {
                self.kiosk.confirm_cancel = None;
            }
        }

        if self.screen.times_out() {
            let idle = self.last_input.elapsed();
            if idle >= IDLE_AFTER + IDLE_GRACE {
                reset = true;
            } else if idle >= IDLE_AFTER {
                let left = (IDLE_AFTER + IDLE_GRACE - idle).as_secs() + 1;
                let opened_at = self.last_input + IDLE_AFTER;
                // Any touch counts as activity, which dismisses this sheet on its own.
                Sheet::new("idle", opened_at).title("Still there?").show(ctx, |ui| {
                    let body = format!("Your order will be cancelled in {left} seconds.");
                    ui.label(RichText::new(body).font(regular(22.0)).color(MUTED));
                    ui.add_space(8.0);
                    let width = ui.available_width();
                    ui.add(KioskButton::primary("I'm Still Here").size(vec2(width, 84.0)));
                });
            }
        }

        if let Some(message) = self.kiosk.toast() {
            toast(ctx, message);
        }
        reset
    }

    /// Fades a freshly shown screen in from the background color. Returns true while fading.
    fn paint_transition(&mut self, ctx: &egui::Context, previous: Discriminant<Screen>) -> bool {
        if discriminant(&self.screen) != previous {
            self.screen_since = Instant::now();
        }
        let t = self.screen_since.elapsed().as_secs_f32() / FADE_SECS;
        if t >= 1.0 {
            return false;
        }
        let alpha = ((1.0 - paint::ease_out(t)) * 255.0) as u8;
        let overlay = Color32::from_rgba_unmultiplied(CREAM.r(), CREAM.g(), CREAM.b(), alpha);
        ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("screen_fade"))).rect_filled(
            ctx.screen_rect(),
            CornerRadius::ZERO,
            overlay,
        );
        true
    }
}

impl eframe::App for KioskApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        fit_to_window(ctx);
        self.handle_keys(ctx);
        if had_user_input(ctx) {
            self.last_input = Instant::now();
        }

        let previous = discriminant(&self.screen);
        if let Some(next) = self.screen.show(ctx, &mut self.kiosk) {
            self.screen = next;
        }
        if self.overlays(ctx) {
            self.screen = Screen::attract();
        }
        if self.screen.is_attract() {
            self.kiosk.reset_session();
        }
        let fading = self.paint_transition(ctx, previous);

        // Smooth frames while something moves; slow ticks for countdowns otherwise.
        let animated = self.screen.animated() || fading || self.kiosk.cart_bump_scale() > 1.0 || ctx.is_using_pointer();
        ctx.request_repaint_after(Duration::from_millis(if animated { 16 } else { 250 }));
    }
}

fn had_user_input(ctx: &egui::Context) -> bool {
    ctx.input(|i| {
        i.events.iter().any(|e| {
            matches!(
                e,
                Event::PointerButton { .. } | Event::Touch { .. } | Event::Key { .. } | Event::MouseWheel { .. }
            )
        })
    })
}
