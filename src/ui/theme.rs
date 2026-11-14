//! Colors, fonts and global style: a clean, iOS-inspired look - flat surfaces,
//! hairline borders instead of shadows, and one brand accent color.

use std::sync::Arc;

use eframe::egui::{self, epaint::Shadow, Color32, FontData, FontDefinitions, FontFamily, FontId, Margin, Stroke};

// Brand.
pub const RED: Color32 = Color32::from_rgb(0xE0, 0x24, 0x2B);
pub const RED_DARK: Color32 = Color32::from_rgb(0xA3, 0x14, 0x1B);
pub const RED_DEEP: Color32 = Color32::from_rgb(0x74, 0x0B, 0x12);
pub const RED_TINT: Color32 = Color32::from_rgb(0xFD, 0xEC, 0xEC);
pub const GOLD: Color32 = Color32::from_rgb(0xFF, 0xC2, 0x0E);
pub const GOLD_LIGHT: Color32 = Color32::from_rgb(0xFF, 0xF1, 0xC7);

// System-style neutrals.
/// Grouped background behind cards.
pub const CREAM: Color32 = Color32::from_rgb(0xF2, 0xF2, 0xF7);
/// Cards, bars and list rows.
pub const CARD: Color32 = Color32::WHITE;
/// Backdrop behind food artwork.
pub const PEACH: Color32 = Color32::from_rgb(0xF7, 0xF5, 0xF2);
/// Search field and stepper track.
pub const FILL: Color32 = Color32::from_rgb(0xEE, 0xEE, 0xF1);
pub const INK: Color32 = Color32::from_rgb(0x1C, 0x1C, 0x1E);
pub const MUTED: Color32 = Color32::from_rgb(0x8E, 0x8E, 0x93);
/// Hairline borders and separators.
pub const BORDER: Color32 = Color32::from_rgb(0xE3, 0xE3, 0xE8);
pub const GREEN: Color32 = Color32::from_rgb(0x34, 0xC7, 0x59);

/// Width of hairline borders (in reference-canvas points).
pub const HAIRLINE: f32 = 1.5;

// Inter (SIL Open Font License, see assets/fonts/OFL.txt), embedded in the binary.
const INTER_REGULAR: &[u8] = include_bytes!("../../assets/fonts/Inter-400.ttf");
const INTER_MEDIUM: &[u8] = include_bytes!("../../assets/fonts/Inter-500.ttf");
const INTER_SEMIBOLD: &[u8] = include_bytes!("../../assets/fonts/Inter-600.ttf");
const INTER_BOLD: &[u8] = include_bytes!("../../assets/fonts/Inter-700.ttf");
const INTER_EXTRABOLD: &[u8] = include_bytes!("../../assets/fonts/Inter-800.ttf");

pub fn regular(size: f32) -> FontId {
    FontId::proportional(size)
}
pub fn medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("medium".into()))
}
pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("semibold".into()))
}
pub fn bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("bold".into()))
}
/// Large titles and prices.
pub fn heavy(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("heavy".into()))
}

pub fn stroke(width: f32, color: Color32) -> Stroke {
    Stroke::new(width, color)
}

/// The standard hairline border.
pub fn hairline() -> Stroke {
    Stroke::new(HAIRLINE, BORDER)
}

/// Soft drop shadow, reserved for the animated start screen.
pub fn soft_shadow() -> Shadow {
    Shadow { offset: [0, 6], blur: 22, spread: 0, color: Color32::from_black_alpha(24) }
}

pub fn install(ctx: &egui::Context) {
    install_fonts(ctx);

    // Always light, regardless of the OS theme.
    ctx.set_theme(egui::Theme::Light);
    let mut visuals = egui::Visuals::light();
    visuals.panel_fill = CREAM;
    visuals.window_fill = CARD;
    visuals.window_stroke = Stroke::NONE;
    visuals.window_corner_radius = egui::CornerRadius::same(28);
    visuals.window_shadow = Shadow::NONE;
    visuals.popup_shadow = Shadow::NONE;
    visuals.override_text_color = Some(INK);
    visuals.selection.bg_fill = RED_TINT;
    visuals.widgets.noninteractive.bg_stroke = hairline();
    ctx.set_visuals(visuals);

    ctx.style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(12.0, 12.0);
        s.spacing.button_padding = egui::vec2(20.0, 10.0);
        s.spacing.window_margin = Margin::same(32);
        s.spacing.scroll.bar_width = 6.0;
        s.spacing.scroll.floating = true;
    });
}

fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    // egui's built-in fonts (incl. emoji) remain as fallbacks for any missing glyphs.
    let builtin = fonts.families[&FontFamily::Proportional].clone();

    let mut add = |name: &str, bytes: Vec<u8>| {
        fonts.font_data.insert(name.to_owned(), Arc::new(FontData::from_owned(bytes)));
        name.to_owned()
    };
    let faces = [
        ("regular", add("inter-regular", INTER_REGULAR.to_vec())),
        ("medium", add("inter-medium", INTER_MEDIUM.to_vec())),
        ("semibold", add("inter-semibold", INTER_SEMIBOLD.to_vec())),
        ("bold", add("inter-bold", INTER_BOLD.to_vec())),
        ("heavy", add("inter-extrabold", INTER_EXTRABOLD.to_vec())),
    ];
    // System fonts cover symbols Inter's Latin subset lacks (e.g. the peso sign).
    let mut extra = Vec::new();
    for (name, path) in [
        ("system-ui", r"C:\Windows\Fonts\segoeuib.ttf"),
        ("system-symbols", r"C:\Windows\Fonts\seguisym.ttf"),
        ("system-dejavu", "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"),
    ] {
        if let Ok(bytes) = std::fs::read(path) {
            extra.push(add(name, bytes));
        }
    }

    for (family, face) in faces {
        let mut list = vec![face];
        list.extend(extra.iter().cloned());
        list.extend(builtin.iter().cloned());
        let key = if family == "regular" { FontFamily::Proportional } else { FontFamily::Name(family.into()) };
        fonts.families.insert(key, list);
    }
    ctx.set_fonts(fonts);
}
