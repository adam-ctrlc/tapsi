//! Food artwork: color images loaded from `assets/` and cached as textures.
//!
//! An art key is either an emoji (e.g. "🍗", loaded from `assets/icons/1f357.png`)
//! or an image path relative to `assets/` (e.g. "images/chicken.jpg") for real
//! product photos. Emoji artwork is also embedded in the binary (see `build.rs`), so it
//! shows even without the `assets/` folder. If no image is found the glyph is drawn.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

use eframe::egui::{
    self, pos2, Align2, Color32, ColorImage, FontId, Painter, Rect, TextureHandle, TextureOptions, Vec2,
};

include!(concat!(env!("OUT_DIR"), "/embedded_icons.rs"));

/// Longest side textures are downscaled to (keeps memory and load time low).
const MAX_TEXTURE_SIDE: u32 = 384;

struct Art {
    texture: TextureHandle,
    aspect: f32,
}

pub struct Icons {
    assets: Option<PathBuf>,
    cache: RefCell<HashMap<String, Option<Art>>>,
}

impl Icons {
    pub fn new() -> Self {
        Self { assets: find_assets_dir(), cache: RefCell::default() }
    }

    /// Loads (once) and returns the texture for an art key.
    fn get(&self, ctx: &egui::Context, key: &str) -> Option<(egui::TextureId, f32)> {
        let mut cache = self.cache.borrow_mut();
        let art = cache.entry(key.to_owned()).or_insert_with(|| self.load(ctx, key));
        art.as_ref().map(|a| (a.texture.id(), a.aspect))
    }

    pub fn preload<'a>(&self, ctx: &egui::Context, keys: impl IntoIterator<Item = &'a str>) {
        for key in keys {
            self.get(ctx, key);
        }
    }

    /// Finds the image for an art key: a file in `assets/` first (so artwork can be
    /// swapped without rebuilding), then the artwork embedded at build time.
    fn read_image(&self, key: &str) -> Option<image::DynamicImage> {
        let lower = key.to_ascii_lowercase();
        let is_photo = [".png", ".jpg", ".jpeg"].iter().any(|ext| lower.ends_with(ext));
        let file_name = if is_photo { key.to_owned() } else { format!("icons/{}.png", codepoints(key)) };

        if let Some(path) = self.assets.as_ref().map(|dir| dir.join(&file_name)).filter(|p| p.exists()) {
            match image::open(&path) {
                Ok(image) => return Some(image),
                Err(e) => eprintln!("could not load {}: {e}", path.display()),
            }
        }
        if is_photo {
            return None;
        }
        let name = codepoints(key);
        let (_, bytes) = EMBEDDED_ICONS.iter().find(|(embedded, _)| *embedded == name)?;
        image::load_from_memory(bytes).ok()
    }

    fn load(&self, ctx: &egui::Context, key: &str) -> Option<Art> {
        let image = self.read_image(key)?;
        let image = if image.width().max(image.height()) > MAX_TEXTURE_SIDE {
            image.resize(MAX_TEXTURE_SIDE, MAX_TEXTURE_SIDE, image::imageops::FilterType::Triangle)
        } else {
            image
        };
        let rgba = image.into_rgba8();
        let (w, h) = (rgba.width() as usize, rgba.height() as usize);
        let color = ColorImage::from_rgba_unmultiplied([w, h], rgba.as_raw());
        let texture = ctx.load_texture(key, color, TextureOptions::LINEAR);
        Some(Art { texture, aspect: w as f32 / h as f32 })
    }

    /// Draws the art fitted (aspect-preserving) and centered in `rect`.
    pub fn paint(&self, p: &Painter, key: &str, rect: Rect) {
        self.paint_tinted(p, key, rect, Color32::WHITE);
    }

    /// Like [`Icons::paint`] with a tint; `Color32::from_white_alpha(a)` fades it.
    pub fn paint_tinted(&self, p: &Painter, key: &str, rect: Rect, tint: Color32) {
        match self.get(p.ctx(), key) {
            Some((id, aspect)) => {
                let size = if aspect >= rect.aspect_ratio() {
                    Vec2::new(rect.width(), rect.width() / aspect)
                } else {
                    Vec2::new(rect.height() * aspect, rect.height())
                };
                let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
                p.image(id, Rect::from_center_size(rect.center(), size), uv, tint);
            }
            None => {
                let color = Color32::from_rgba_unmultiplied(0xDA, 0x1F, 0x26, tint.a());
                p.text(rect.center(), Align2::CENTER_CENTER, key, FontId::proportional(rect.height() * 0.75), color);
            }
        }
    }
}

/// "🍗" -> "1f357" (variation selectors dropped, multiple code points joined by '_').
fn codepoints(emoji: &str) -> String {
    emoji.chars().filter(|&c| c != '\u{fe0f}').map(|c| format!("{:x}", c as u32)).collect::<Vec<_>>().join("_")
}

/// `assets/` in the working directory, next to the executable, or a few levels above
/// it (so `target/release/kiosk.exe` finds the project's `assets/` when run directly).
fn find_assets_dir() -> Option<PathBuf> {
    let mut candidates = vec![PathBuf::from("assets")];
    if let Ok(exe) = std::env::current_exe() {
        candidates.extend(exe.ancestors().skip(1).take(4).map(|dir| dir.join("assets")));
    }
    candidates.into_iter().find(|p| p.is_dir())
}

#[cfg(test)]
mod tests {
    use super::codepoints;

    #[test]
    fn emoji_to_file_name() {
        assert_eq!(codepoints("🍗"), "1f357");
        assert_eq!(codepoints("☕"), "2615");
        assert_eq!(codepoints("☕\u{fe0f}"), "2615");
    }
}
