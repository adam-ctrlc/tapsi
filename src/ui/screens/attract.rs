use eframe::egui::{self, pos2, vec2, Align2, Color32, CornerRadius, Frame, Id, Rect, Sense, Vec2};

use super::{Screen, View};
use crate::kiosk::Kiosk;
use crate::ui::paint;
use crate::ui::theme::{heavy, medium, regular, soft_shadow, stroke, GOLD, GOLD_LIGHT, INK, RED, RED_DEEP};

const HERO_FOODS: &[&str] = &["🍗", "🍔", "🍝", "🍟", "🍦"];
const HERO_PERIOD: f32 = 2.8;

/// Idle "Touch to start" screen with an animated food carousel.
pub struct Attract;

impl View for Attract {
    fn show(&mut self, ctx: &egui::Context, kiosk: &mut Kiosk) -> Option<Screen> {
        let mut next = None;
        egui::CentralPanel::default().frame(Frame::new().fill(RED)).show(ctx, |ui| {
            let rect = ui.max_rect();
            if ui.interact(rect, Id::new("attract"), Sense::click()).clicked() {
                next = Some(Screen::order_type());
            }
            let p = ui.painter();
            let t = kiosk.started.elapsed().as_secs_f32();
            let (w, h) = (rect.width(), rect.height());
            let at = |f: f32| pos2(rect.center().x, rect.top() + h * f);

            paint::gradient(p, rect, RED, RED_DEEP);
            let hero = at(0.37);
            paint::sunburst(&p.with_clip_rect(rect), hero, h, 20, t * 0.04, Color32::from_white_alpha(7));
            paint::brand(p, at(0.075), &kiosk.menu.store_name, 34.0, heavy(46.0), Color32::WHITE);

            // Hero: rotating carousel of food art on a glowing gold disc.
            let disc = w * 0.29;
            p.circle_filled(hero, disc + 34.0, Color32::from_white_alpha(14));
            p.circle_filled(hero, disc + 16.0, Color32::from_white_alpha(24));
            p.add(
                soft_shadow().as_shape(Rect::from_center_size(hero, Vec2::splat(disc * 2.0)), CornerRadius::same(255)),
            );
            p.circle_filled(hero, disc, GOLD);
            let k = (t / HERO_PERIOD) as usize;
            let fade = ((t % HERO_PERIOD) / 0.45).min(1.0);
            let bob = (t * 2.2).sin() * 8.0;
            let size = disc * 1.45;
            let draw = |icon: &str, alpha: f32, scale: f32| {
                let r = Rect::from_center_size(hero + vec2(0.0, bob), Vec2::splat(size * scale));
                kiosk.icons.paint_tinted(p, icon, r, Color32::from_white_alpha((alpha * 255.0) as u8));
            };
            let n = HERO_FOODS.len();
            if fade < 1.0 {
                draw(HERO_FOODS[(k + n - 1) % n], 1.0 - fade, 1.0 + 0.15 * fade);
            }
            draw(HERO_FOODS[k % n], fade, 0.8 + 0.2 * paint::ease_out(fade));

            paint::text(p, at(0.615), Align2::CENTER_CENTER, "Order here,", heavy(76.0), Color32::WHITE);
            paint::text(p, at(0.675), Align2::CENTER_CENTER, "skip the line!", heavy(76.0), GOLD);
            paint::text(p, at(0.73), Align2::CENTER_CENTER, &kiosk.menu.tagline, medium(28.0), GOLD_LIGHT);

            let pulse = 0.5 + 0.5 * (t * 3.0).sin();
            let pill = Rect::from_center_size(at(0.83), vec2(600.0, 124.0) * (1.0 + 0.035 * pulse));
            p.add(soft_shadow().as_shape(pill, CornerRadius::same(62)));
            p.rect_filled(pill, CornerRadius::same(62), GOLD);
            // Center the icon + label together inside the pill.
            let (label, font, icon_size, gap) = ("Touch to start", heavy(44.0), 76.0, 18.0);
            let label_width = p.layout_no_wrap(label.to_owned(), font.clone(), INK).size().x;
            let left = pill.center().x - (icon_size + gap + label_width) / 2.0;
            let hand = Rect::from_center_size(pos2(left + icon_size / 2.0, pill.center().y), Vec2::splat(icon_size));
            // The hand is yellow like the pill, so it sits on a white disc with a "tap" ripple.
            let ripple = (t * 1.2).fract();
            let ring_alpha = ((1.0 - ripple) * 120.0) as u8;
            p.circle_stroke(
                hand.center(),
                icon_size / 2.0 + 4.0 + ripple * 14.0,
                stroke(3.0, Color32::from_white_alpha(ring_alpha)),
            );
            p.circle_filled(hand.center(), icon_size / 2.0, Color32::WHITE);
            kiosk.icons.paint(p, "👆", hand.shrink(14.0));
            paint::text(p, pos2(hand.right() + gap, pill.center().y), Align2::LEFT_CENTER, label, font, INK);

            let footer = "Dine In  •  Take Out  •  Cash  •  Card  •  E-Wallet";
            paint::text(p, at(0.935), Align2::CENTER_CENTER, footer, medium(21.0), GOLD_LIGHT);

            if let Some(warning) = &kiosk.menu_warning {
                paint::text(
                    p,
                    rect.center_bottom() - vec2(0.0, 16.0),
                    Align2::CENTER_BOTTOM,
                    warning,
                    regular(16.0),
                    GOLD_LIGHT,
                );
            }
        });
        next
    }

    fn times_out(&self) -> bool {
        false
    }

    fn animated(&self) -> bool {
        true
    }
}
