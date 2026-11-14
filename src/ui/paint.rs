//! Low-level drawing helpers shared by widgets and screens.

use std::f32::consts::TAU;

use eframe::egui::{
    pos2, vec2, Align, Align2, Color32, CornerRadius, FontId, Mesh, Painter, Pos2, Rect, Shape, Stroke, StrokeKind,
    Vec2,
};

use super::theme::{stroke, BORDER, GOLD, INK, RED_DARK};

/// Paints text like [`Painter::text`], but vertically centered alignments center on the
/// visual middle of capital letters instead of the line box (which includes space for
/// descenders and makes labels look too high in badges and buttons).
pub fn text(p: &Painter, anchor: Pos2, align: Align2, text: impl ToString, font: FontId, color: Color32) -> Rect {
    if align.y() != Align::Center {
        return p.text(anchor, align, text, font, color);
    }
    let galley = p.layout_no_wrap(text.to_string(), font.clone(), color);
    let caps = p.layout_no_wrap("H".to_owned(), font, color).mesh_bounds;
    let width = galley.size().x;
    let x = match align.x() {
        Align::Min => anchor.x,
        Align::Center => anchor.x - width / 2.0,
        Align::Max => anchor.x - width,
    };
    let pos = pos2(x, anchor.y - caps.center().y);
    let rect = Rect::from_min_size(pos, galley.size());
    p.galley(pos, galley, color);
    rect
}

/// Rounded label ("badge") with optically centered text; returns its rect.
pub fn badge(p: &Painter, min: Pos2, text: &str, font: FontId, fill: Color32, fg: Color32, padding: Vec2) -> Rect {
    let width = p.layout_no_wrap(text.to_owned(), font.clone(), fg).size().x;
    let height = font.size + padding.y * 2.0;
    let rect = Rect::from_min_size(min, vec2(width + padding.x * 2.0, height));
    p.rect_filled(rect, CornerRadius::same((height / 2.0) as u8), fill);
    self::text(p, rect.center(), Align2::CENTER_CENTER, text, font, fg);
    rect
}

pub fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

/// Rectangle filled with a vertical gradient.
pub fn gradient(p: &Painter, rect: Rect, top: Color32, bottom: Color32) {
    let mut mesh = Mesh::default();
    mesh.colored_vertex(rect.left_top(), top);
    mesh.colored_vertex(rect.right_top(), top);
    mesh.colored_vertex(rect.left_bottom(), bottom);
    mesh.colored_vertex(rect.right_bottom(), bottom);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(1, 3, 2);
    p.add(mesh);
}

/// Alternating light rays radiating from `center` (classic fast-food backdrop).
pub fn sunburst(p: &Painter, center: Pos2, radius: f32, rays: usize, rotation: f32, color: Color32) {
    let step = TAU / (rays * 2) as f32;
    for i in 0..rays {
        let a0 = rotation + (i * 2) as f32 * step;
        let a1 = a0 + step;
        let points =
            vec![center, center + radius * vec2(a0.cos(), a0.sin()), center + radius * vec2(a1.cos(), a1.sin())];
        p.add(Shape::convex_polygon(points, color, Stroke::NONE));
    }
}

/// The store's sun logo: a smiling gold sun with rays.
pub fn logo(p: &Painter, c: Pos2, r: f32) {
    for i in 0..12 {
        let a = i as f32 * TAU / 12.0;
        let dir = vec2(a.cos(), a.sin());
        p.line_segment([c + dir * r * 0.74, c + dir * r * 0.98], stroke(r * 0.12, GOLD));
    }
    p.circle_filled(c, r * 0.62, GOLD);
    p.circle_stroke(c, r * 0.62, stroke(r * 0.05, RED_DARK.gamma_multiply(0.25)));
    let eye = r * 0.07;
    p.circle_filled(c + vec2(-r * 0.2, -r * 0.12), eye, RED_DARK);
    p.circle_filled(c + vec2(r * 0.2, -r * 0.12), eye, RED_DARK);
    let smile: Vec<Pos2> = (0..=12)
        .map(|i| {
            let a = (25.0 + 130.0 * i as f32 / 12.0).to_radians();
            c + vec2(0.0, -r * 0.02) + r * 0.32 * vec2(a.cos(), a.sin())
        })
        .collect();
    p.add(Shape::line(smile, stroke(r * 0.08, RED_DARK)));
}

/// Logo + store name, centered on `center`.
pub fn brand(p: &Painter, center: Pos2, name: &str, logo_r: f32, font: FontId, color: Color32) {
    let g = p.layout_no_wrap(name.to_owned(), font, color);
    let gap = 14.0;
    let total = logo_r * 2.0 + gap + g.size().x;
    let left = center.x - total / 2.0;
    logo(p, pos2(left + logo_r, center.y), logo_r);
    p.galley(pos2(left + logo_r * 2.0 + gap, center.y - g.size().y / 2.0), g, color);
}

pub fn check(p: &Painter, c: Pos2, s: f32, color: Color32) {
    let line = stroke(s * 0.14, color);
    let a = c + vec2(-s * 0.32, 0.0);
    let b = c + vec2(-s * 0.08, s * 0.24);
    let d = c + vec2(s * 0.34, -s * 0.24);
    p.line_segment([a, b], line);
    p.line_segment([b, d], line);
}

/// Decorative QR-like pattern for the demo e-wallet screen (not a real QR code).
pub fn demo_qr(p: &Painter, rect: Rect, seed: u64) {
    const N: usize = 25;
    p.rect_filled(rect, CornerRadius::same(20), Color32::WHITE);
    p.rect_stroke(rect, CornerRadius::same(20), stroke(3.0, BORDER), StrokeKind::Inside);
    let inner = rect.shrink(24.0);
    let cell = inner.width() / N as f32;
    let mut s = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407) | 1;
    for y in 0..N {
        for x in 0..N {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            // The three corner "finder" squares make it read as a QR code.
            let finder = |cx: usize, cy: usize| -> Option<bool> {
                let (dx, dy) = (x as i32 - cx as i32, y as i32 - cy as i32);
                if !(-1..=7).contains(&dx) || !(-1..=7).contains(&dy) {
                    return None;
                }
                if !(0..=6).contains(&dx) || !(0..=6).contains(&dy) {
                    return Some(false);
                }
                let ring = dx == 0 || dy == 0 || dx == 6 || dy == 6;
                let core = (2..=4).contains(&dx) && (2..=4).contains(&dy);
                Some(ring || core)
            };
            let on = finder(0, 0).or_else(|| finder(N - 7, 0)).or_else(|| finder(0, N - 7)).unwrap_or(s & 1 == 1);
            if on {
                let min = inner.min + vec2(x as f32 * cell, y as f32 * cell);
                p.rect_filled(Rect::from_min_size(min, vec2(cell, cell)), CornerRadius::ZERO, INK);
            }
        }
    }
}

// ---------------------------------------------------------------- line icons
// Simple vector glyphs in the style of system symbols, sized by `s`.

pub fn chevron(p: &Painter, c: Pos2, s: f32, left: bool, color: Color32) {
    let dx = if left { -1.0 } else { 1.0 };
    let line = stroke(s * 0.14, color);
    p.line_segment([c + vec2(-dx * s * 0.2, -s * 0.4), c + vec2(dx * s * 0.2, 0.0)], line);
    p.line_segment([c + vec2(dx * s * 0.2, 0.0), c + vec2(-dx * s * 0.2, s * 0.4)], line);
}

pub fn icon_home(p: &Painter, c: Pos2, s: f32, color: Color32) {
    let line = stroke(s * 0.09, color);
    let roof = vec![c + vec2(-s * 0.48, s * 0.0), c + vec2(0.0, -s * 0.44), c + vec2(s * 0.48, s * 0.0)];
    p.add(Shape::line(roof, line));
    let body = Rect::from_min_max(c + vec2(-s * 0.34, -s * 0.12), c + vec2(s * 0.34, s * 0.42));
    p.rect_stroke(body, CornerRadius::same((s * 0.06) as u8), line, StrokeKind::Middle);
    let door = Rect::from_min_max(c + vec2(-s * 0.09, s * 0.14), c + vec2(s * 0.09, s * 0.42));
    p.rect_filled(door, CornerRadius::same(2), color);
}

pub fn icon_grid(p: &Painter, c: Pos2, s: f32, color: Color32) {
    let line = stroke(s * 0.09, color);
    let cell = s * 0.36;
    for (dx, dy) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        let center = c + vec2(dx, dy) * (cell / 2.0 + s * 0.05);
        let r = Rect::from_center_size(center, Vec2::splat(cell));
        p.rect_stroke(r, CornerRadius::same((s * 0.08) as u8), line, StrokeKind::Middle);
    }
}

pub fn icon_search(p: &Painter, c: Pos2, s: f32, color: Color32) {
    let line = stroke(s * 0.09, color);
    let lens = c + vec2(-s * 0.08, -s * 0.08);
    p.circle_stroke(lens, s * 0.28, line);
    p.line_segment([c + vec2(s * 0.14, s * 0.14), c + vec2(s * 0.42, s * 0.42)], stroke(s * 0.11, color));
}

pub fn icon_dine(p: &Painter, c: Pos2, s: f32, color: Color32) {
    let line = stroke(s * 0.09, color);
    // Fork: three tines joined at the neck, then the handle.
    let fx = c.x - s * 0.2;
    for dx in [-0.11, 0.0, 0.11] {
        p.line_segment([pos2(fx + dx * s, c.y - s * 0.44), pos2(fx + dx * s, c.y - s * 0.16)], line);
    }
    p.line_segment([pos2(fx - s * 0.11, c.y - s * 0.16), pos2(fx + s * 0.11, c.y - s * 0.16)], line);
    p.line_segment([pos2(fx, c.y - s * 0.16), pos2(fx, c.y + s * 0.44)], line);
    // Knife: blade on the right, then the handle.
    let kx = c.x + s * 0.18;
    let blade = vec![
        pos2(kx, c.y + s * 0.04),
        pos2(kx, c.y - s * 0.44),
        pos2(kx + s * 0.14, c.y - s * 0.2),
        pos2(kx + s * 0.14, c.y + s * 0.04),
        pos2(kx, c.y + s * 0.04),
    ];
    p.add(Shape::line(blade, line));
    p.line_segment([pos2(kx, c.y + s * 0.04), pos2(kx, c.y + s * 0.44)], line);
}

pub fn icon_bag(p: &Painter, c: Pos2, s: f32, color: Color32) {
    let line = stroke(s * 0.09, color);
    let body = Rect::from_min_max(c + vec2(-s * 0.38, -s * 0.16), c + vec2(s * 0.38, s * 0.44));
    p.rect_stroke(body, CornerRadius::same((s * 0.08) as u8), line, StrokeKind::Middle);
    let handle: Vec<Pos2> = (0..=12)
        .map(|i| {
            let a = (180.0 + 180.0 * i as f32 / 12.0f32).to_radians();
            c + vec2(0.0, -s * 0.16) + s * 0.18 * vec2(a.cos(), a.sin())
        })
        .collect();
    p.add(Shape::line(handle, line));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ease_out_is_monotonic_from_zero_to_one() {
        assert_eq!(ease_out(0.0), 0.0);
        assert_eq!(ease_out(1.0), 1.0);
        assert!(ease_out(0.5) > 0.5);
    }
}
