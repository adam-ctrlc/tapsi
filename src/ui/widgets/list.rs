//! iOS-style grouped lists: a white rounded group of rows separated by hairlines.

use eframe::egui::{
    pos2, vec2, Align2, Color32, CornerRadius, Frame, Margin, Painter, Pos2, Rect, Response, RichText, Sense,
    StrokeKind, Ui, Vec2, Widget,
};

use crate::ui::icons::Icons;
use crate::ui::paint;
use crate::ui::theme::{
    hairline, medium, regular, semibold, stroke, BORDER, CARD, FILL, GOLD_LIGHT, HAIRLINE, INK, MUTED, RED, RED_TINT,
};

/// White rounded container with a hairline border; rows inside should not add spacing.
pub fn list_group<R>(ui: &mut Ui, rows: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::new()
        .fill(CARD)
        .stroke(hairline())
        .corner_radius(CornerRadius::same(22))
        .inner_margin(Margin::symmetric(0, 4))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 0.0;
            rows(ui)
        })
        .inner
}

/// Avatar shown at the start of a [`group_header`].
#[derive(Clone, Copy)]
pub enum Avatar {
    /// The smiling sun mascot, for friendly "talking to you" messages.
    Mascot,
    /// A red line icon (see `paint::icon_*`).
    Icon(fn(&Painter, Pos2, f32, Color32)),
}

/// Conversational header row at the top of a [`list_group`]: an avatar, a headline
/// and a sentence, with a hairline separating it from the rows below.
pub fn group_header(ui: &mut Ui, avatar: Avatar, headline: &str, body: &str) {
    let response = Frame::new().inner_margin(Margin::symmetric(20, 18)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 16.0;
            let (rect, _) = ui.allocate_exact_size(vec2(64.0, 64.0), Sense::hover());
            let c = rect.center();
            match avatar {
                Avatar::Mascot => {
                    ui.painter().circle_filled(c, 32.0, GOLD_LIGHT);
                    paint::logo(ui.painter(), c, 26.0);
                }
                Avatar::Icon(draw) => {
                    ui.painter().circle_filled(c, 32.0, RED_TINT);
                    draw(ui.painter(), c, 32.0, RED);
                }
            }
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 4.0;
                ui.add_space(4.0);
                ui.label(RichText::new(headline).font(semibold(23.0)).color(INK));
                ui.add(eframe::egui::Label::new(RichText::new(body).font(regular(19.0)).color(MUTED)).wrap());
            });
        });
    });
    let r = response.response.rect;
    ui.painter().hline(r.x_range(), r.bottom() - HAIRLINE / 2.0, hairline());
}

/// Small uppercase heading above a group, with an optional colored note on the right.
pub fn section_header(ui: &mut Ui, title: &str, note: Option<(&str, Color32)>) {
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        ui.label(RichText::new(title.to_uppercase()).font(semibold(17.0)).color(MUTED));
        if let Some((text, color)) = note {
            ui.with_layout(eframe::egui::Layout::right_to_left(eframe::egui::Align::Center), |ui| {
                ui.add_space(8.0);
                ui.label(RichText::new(text).font(semibold(17.0)).color(color));
            });
        }
    });
}

/// Trailing control of a row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Accessory {
    None,
    /// Round check (single choice).
    Radio(bool),
    /// Square check (multiple choice).
    Checkbox(bool),
    /// Navigates somewhere.
    Chevron,
}

/// One tappable row of a grouped list.
#[must_use = "add it with `ui.add(...)`"]
pub struct ListRow<'a> {
    title: &'a str,
    subtitle: Option<&'a str>,
    detail: Option<&'a str>,
    icon: Option<(&'a Icons, &'a str, Color32)>,
    accessory: Accessory,
    separator: bool,
    height: f32,
}

impl<'a> ListRow<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            subtitle: None,
            detail: None,
            icon: None,
            accessory: Accessory::None,
            separator: true,
            height: 78.0,
        }
    }

    pub fn subtitle(mut self, subtitle: &'a str) -> Self {
        self.subtitle = Some(subtitle);
        self
    }

    /// Right-aligned text before the accessory (e.g. a price).
    pub fn detail(mut self, detail: Option<&'a str>) -> Self {
        self.detail = detail;
        self
    }

    /// Leading artwork on a colored disc.
    pub fn icon(mut self, icons: &'a Icons, icon: &'a str, background: Color32) -> Self {
        self.icon = Some((icons, icon, background));
        self
    }

    pub fn accessory(mut self, accessory: Accessory) -> Self {
        self.accessory = accessory;
        self
    }

    /// Draw a separator under the row (turn off for the last row).
    pub fn separator(mut self, separator: bool) -> Self {
        self.separator = separator;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

impl Widget for ListRow<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), self.height), Sense::click());
        let p = ui.painter();
        let selected = matches!(self.accessory, Accessory::Radio(true) | Accessory::Checkbox(true));
        if response.is_pointer_button_down_on() {
            p.rect_filled(rect.shrink2(vec2(6.0, 2.0)), CornerRadius::same(16), FILL);
        } else if selected {
            p.rect_filled(rect.shrink2(vec2(6.0, 2.0)), CornerRadius::same(16), RED_TINT.gamma_multiply(0.6));
        }

        let mut x = rect.left() + 24.0;
        if let Some((icons, icon, background)) = self.icon {
            let radius = (self.height * 0.32).min(40.0);
            let c = pos2(x + radius, rect.center().y);
            p.circle_filled(c, radius, background);
            icons.paint(p, icon, Rect::from_center_size(c, Vec2::splat(radius * 1.3)));
            x += radius * 2.0 + 22.0;
        }

        let title_color = if selected { RED } else { INK };
        match self.subtitle {
            Some(subtitle) => {
                paint::text(
                    p,
                    pos2(x, rect.center().y - 14.0),
                    Align2::LEFT_CENTER,
                    self.title,
                    semibold(24.0),
                    title_color,
                );
                paint::text(p, pos2(x, rect.center().y + 18.0), Align2::LEFT_CENTER, subtitle, regular(19.0), MUTED);
            }
            None => {
                paint::text(p, pos2(x, rect.center().y), Align2::LEFT_CENTER, self.title, medium(23.0), title_color);
            }
        }

        let mut right = rect.right() - 26.0;
        let c = pos2(right - 16.0, rect.center().y);
        match self.accessory {
            Accessory::None => {}
            Accessory::Radio(on) => {
                if on {
                    p.circle_filled(c, 16.0, RED);
                    paint::check(p, c, 22.0, Color32::WHITE);
                } else {
                    p.circle_stroke(c, 15.0, stroke(2.0, BORDER));
                }
                right -= 48.0;
            }
            Accessory::Checkbox(on) => {
                let b = Rect::from_center_size(c, vec2(32.0, 32.0));
                if on {
                    p.rect_filled(b, CornerRadius::same(9), RED);
                    paint::check(p, c, 22.0, Color32::WHITE);
                } else {
                    p.rect_stroke(b, CornerRadius::same(9), stroke(2.0, BORDER), StrokeKind::Inside);
                }
                right -= 48.0;
            }
            Accessory::Chevron => {
                paint::chevron(p, c + vec2(6.0, 0.0), 26.0, false, MUTED);
                right -= 40.0;
            }
        }
        if let Some(detail) = self.detail {
            let color = if selected { RED } else { MUTED };
            paint::text(p, pos2(right, rect.center().y), Align2::RIGHT_CENTER, detail, medium(20.0), color);
        }

        if self.separator {
            p.hline(x..=rect.right(), rect.bottom() - HAIRLINE / 2.0, hairline());
        }
        response
    }
}
