//! Quiet Premium: общие токены, геометрические иконки и доступные элементы.
use crate::db::{FieldDef, FieldKind};
use chrono::Datelike;
use chrono::NaiveDate;
use eframe::egui::{self, Color32, RichText};

pub const BG: Color32 = Color32::from_rgb(246, 248, 252);
pub const INK: Color32 = Color32::from_rgb(21, 32, 51);
pub const MUTED: Color32 = Color32::from_rgb(98, 112, 137);
pub const BLUE: Color32 = Color32::from_rgb(46, 107, 255);
pub const SOFT_BLUE: Color32 = Color32::from_rgb(234, 241, 255);
pub const BORDER: Color32 = Color32::from_rgb(220, 228, 240);
pub const DANGER: Color32 = Color32::from_rgb(207, 59, 65);
pub const GREEN: Color32 = Color32::from_rgb(30, 143, 110);
pub const RED_SOFT: Color32 = Color32::from_rgb(255, 239, 240);

#[derive(Clone, Copy)]
pub enum Glyph {
    Brand,
    Home,
    Person,
    Table,
    Search,
    Tools,
    Form,
    Backup,
    Settings,
    Help,
    Folder,
    Plus,
    Up,
    Down,
    Phone,
    Database,
    Shield,
    Calendar,
    Info,
    Appearance,
}

pub fn draw_glyph(p: &egui::Painter, r: egui::Rect, glyph: Glyph, color: Color32) {
    let pixels = p.ctx().pixels_per_point();
    let scale = r.width().min(r.height()) / 24.0;
    let center = egui::pos2(
        (r.center().x * pixels).round() / pixels,
        (r.center().y * pixels).round() / pixels,
    );
    let stroke_px = (2.0 * pixels).round().max(1.0);
    let stroke = egui::Stroke::new(stroke_px / pixels, color);
    let offset = if stroke_px as i32 % 2 == 0 { 0.0 } else { 0.5 };
    let snap = |v: f32| ((v * pixels - offset).round() + offset) / pixels;
    let pt = |x: f32, y: f32| egui::pos2(snap(center.x + x * scale), snap(center.y + y * scale));
    let line = |a, b, c, d| {
        p.line_segment([pt(a, b), pt(c, d)], stroke);
    };
    let path = |points: &[(f32, f32)], closed: bool| {
        let points = points.iter().map(|&(x, y)| pt(x, y)).collect();
        p.add(if closed {
            egui::Shape::closed_line(points, stroke)
        } else {
            egui::Shape::line(points, stroke)
        });
    };
    let rect = |a, b, c, d| {
        p.rect_stroke(
            egui::Rect::from_two_pos(pt(a, b), pt(c, d)),
            2.0,
            stroke,
            egui::StrokeKind::Middle,
        );
    };
    let circle = |x, y, radius: f32| {
        p.circle_stroke(
            pt(x, y),
            (radius * scale * pixels).round().max(1.0) / pixels,
            stroke,
        );
    };
    match glyph {
        Glyph::Brand => {
            let mark_stroke =
                egui::Stroke::new((1.7 * scale * pixels).round().max(1.0) / pixels, color);
            for points in crate::brand::mark_paths() {
                p.add(egui::Shape::line(
                    points
                        .into_iter()
                        .map(|v| center + egui::vec2(v[0] * scale, v[1] * scale))
                        .collect(),
                    mark_stroke,
                ));
            }
        }
        Glyph::Home => {
            path(&[(-9.0, -1.0), (0.0, -9.0), (9.0, -1.0)], false);
            path(&[(-7.0, 0.0), (-7.0, 9.0), (7.0, 9.0), (7.0, 0.0)], false);
            path(&[(-2.0, 9.0), (-2.0, 3.0), (2.0, 3.0), (2.0, 9.0)], false);
        }
        Glyph::Person => {
            circle(0.0, -5.0, 4.0);
            p.add(egui::epaint::CubicBezierShape::from_points_stroke(
                [pt(-8.0, 9.0), pt(-8.0, -3.0), pt(8.0, -3.0), pt(8.0, 9.0)],
                false,
                Color32::TRANSPARENT,
                stroke,
            ));
        }
        Glyph::Table => {
            rect(-9.0, -9.0, 9.0, 9.0);
            line(-9.0, -3.0, 9.0, -3.0);
            line(-9.0, 3.0, 9.0, 3.0);
            line(-3.0, -9.0, -3.0, 9.0);
        }
        Glyph::Calendar => {
            rect(-9.0, -7.0, 9.0, 9.0);
            line(-9.0, -1.0, 9.0, -1.0);
            line(-4.0, -10.0, -4.0, -5.0);
            line(4.0, -10.0, 4.0, -5.0);
            line(-4.0, 4.0, -1.0, 4.0);
            line(3.0, 4.0, 5.0, 4.0);
        }
        Glyph::Search => {
            // Preserve a true circular lens; snapping its centre independently
            // of its radius used to distort the join at fractional DPI.
            let lens = center + egui::vec2(-2.0, -2.0) * scale;
            p.circle_stroke(lens, 6.5 * scale, stroke);
            let join = lens + egui::vec2(1.0, 1.0).normalized() * (6.5 * scale);
            p.line_segment([join, center + egui::vec2(9.0, 9.0) * scale], stroke);
        }
        Glyph::Tools => {
            for (y, x) in [(-6.0, 3.0), (0.0, -4.0), (6.0, 1.0)] {
                line(-9.0, y, x - 2.0, y);
                line(x + 2.0, y, 9.0, y);
                rect(x - 2.0, y - 2.0, x + 2.0, y + 2.0);
            }
        }
        Glyph::Form => {
            rect(-8.0, -10.0, 8.0, 10.0);
            line(-4.0, -5.0, 4.0, -5.0);
            rect(-4.0, -1.0, 4.0, 3.0);
            line(-4.0, 7.0, 2.0, 7.0);
        }
        Glyph::Backup => {
            // Two documents: a readily recognisable copy symbol.
            rect(-5.0, -5.0, 9.0, 9.0);
            path(&[(-9.0, 5.0), (-9.0, -9.0), (5.0, -9.0)], false);
        }
        Glyph::Info => {
            circle(0.0, 0.0, 9.0);
            line(0.0, -1.0, 0.0, 5.0);
            p.circle_filled(pt(0.0, -5.0), stroke.width * 0.65, color);
        }
        Glyph::Appearance => {
            rect(-9.0, -8.0, 9.0, 5.0);
            line(0.0, 5.0, 0.0, 9.0);
            line(-5.0, 9.0, 5.0, 9.0);
        }
        Glyph::Settings => {
            circle(0.0, 0.0, 5.0);
            for (a, b, c, d) in [
                (0.0, -10.0, 0.0, -7.0),
                (0.0, 7.0, 0.0, 10.0),
                (-10.0, 0.0, -7.0, 0.0),
                (7.0, 0.0, 10.0, 0.0),
                (-7.0, -7.0, -5.0, -5.0),
                (5.0, 5.0, 7.0, 7.0),
                (-7.0, 7.0, -5.0, 5.0),
                (5.0, -5.0, 7.0, -7.0),
            ] {
                line(a, b, c, d);
            }
            p.circle_filled(center, 1.3 * scale, color);
        }
        Glyph::Help => {
            circle(0.0, 0.0, 10.0);
            p.add(egui::epaint::CubicBezierShape::from_points_stroke(
                [pt(-3.5, -4.0), pt(-3.5, -9.0), pt(7.5, -7.0), pt(2.0, -1.0)],
                false,
                Color32::TRANSPARENT,
                stroke,
            ));
            path(&[(2.0, -1.0), (0.0, 1.0), (0.0, 2.0)], false);
            p.circle_filled(pt(0.0, 6.0), stroke.width * 0.65, color);
        }
        Glyph::Folder => {
            path(
                &[
                    (-10.0, -6.0),
                    (-3.0, -6.0),
                    (0.0, -3.0),
                    (10.0, -3.0),
                    (10.0, 8.0),
                    (-10.0, 8.0),
                ],
                true,
            );
        }
        Glyph::Plus => {
            line(-7.0, 0.0, 7.0, 0.0);
            line(0.0, -7.0, 0.0, 7.0);
        }
        Glyph::Up | Glyph::Down => {
            let d = if matches!(glyph, Glyph::Up) {
                -1.0
            } else {
                1.0
            };
            line(0.0, -8.0 * d, 0.0, 8.0 * d);
            path(&[(-5.0, 3.0 * d), (0.0, 8.0 * d), (5.0, 3.0 * d)], false);
        }
        Glyph::Phone => {
            p.add(egui::epaint::CubicBezierShape::from_points_stroke(
                [pt(-8.0, -9.0), pt(-12.0, 1.0), pt(2.0, 13.0), pt(9.0, 8.0)],
                false,
                Color32::TRANSPARENT,
                stroke,
            ));
            p.add(egui::epaint::CubicBezierShape::from_points_stroke(
                [pt(-4.0, -3.0), pt(-4.0, 0.0), pt(1.0, 5.0), pt(4.0, 4.0)],
                false,
                Color32::TRANSPARENT,
                stroke,
            ));
            path(
                &[(-8.0, -9.0), (-5.0, -10.0), (-1.0, -5.0), (-4.0, -3.0)],
                false,
            );
            path(&[(4.0, 4.0), (7.0, 1.0), (11.0, 5.0), (9.0, 8.0)], false);
        }
        Glyph::Database => {
            rect(-8.0, -9.0, 8.0, 9.0);
            line(-8.0, -3.0, 8.0, -3.0);
            line(-8.0, 3.0, 8.0, 3.0);
            p.circle_filled(pt(-4.0, -6.0), scale, color);
            p.circle_filled(pt(-4.0, 0.0), scale, color);
            p.circle_filled(pt(-4.0, 6.0), scale, color);
        }
        Glyph::Shield => {
            path(
                &[
                    (0.0, -10.0),
                    (8.0, -6.0),
                    (8.0, 2.0),
                    (5.0, 7.0),
                    (0.0, 10.0),
                    (-5.0, 7.0),
                    (-8.0, 2.0),
                    (-8.0, -6.0),
                ],
                true,
            );
            path(&[(-4.0, 0.0), (-1.0, 3.0), (4.0, -3.0)], false);
        }
    }
}
// Align visible glyphs, not the font's ascent/descent line box.
fn ink_position(p: &egui::Painter, galley: &egui::Galley, center: egui::Pos2) -> egui::Pos2 {
    let bounds = galley.mesh_bounds;
    let pos = egui::pos2(center.x - bounds.center().x, center.y - bounds.center().y);
    let pixels = p.ctx().pixels_per_point();
    egui::pos2(
        (pos.x * pixels).round() / pixels,
        (pos.y * pixels).round() / pixels,
    )
}
pub fn paint_centered_text(
    p: &egui::Painter,
    center: egui::Pos2,
    text: &str,
    font: egui::FontId,
    color: Color32,
) {
    let galley = p.layout_no_wrap(text.to_owned(), font, color);
    let pos = ink_position(p, &galley, center);
    p.galley(pos, galley, color);
}
pub fn centered_caption(ui: &mut egui::Ui, text: &str, width: f32, height: f32) {
    let mut job = egui::text::LayoutJob::simple(
        text.to_owned(),
        egui::FontId::proportional(14.0),
        MUTED,
        width,
    );
    job.halign = egui::Align::Center;
    job.justify = false;
    let galley = ui.painter().layout_job(job);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let pos = ink_position(ui.painter(), &galley, rect.center());
    ui.painter().galley(pos, galley, MUTED);
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, ui.is_enabled(), text));
}
pub fn icon_badge(ui: &mut egui::Ui, glyph: Glyph, size: f32, bg: Color32, fg: Color32) {
    let (r, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    ui.painter().rect_filled(r, 12.0, bg);
    draw_glyph(ui.painter(), r.shrink(size * 0.18), glyph, fg);
}

pub fn navigation_icon_button(ui: &mut egui::Ui, glyph: Glyph, tip: &str) -> egui::Response {
    let response = ui.add(
        egui::Button::new("")
            .min_size(egui::vec2(36.0, 36.0))
            .fill(BG)
            .stroke(egui::Stroke::NONE)
            .corner_radius(14),
    );
    draw_glyph(ui.painter(), response.rect.shrink(9.0), glyph, INK);
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), tip));
    response.on_hover_text(tip)
}
pub fn nav_item(
    ui: &mut egui::Ui,
    glyph: Glyph,
    label: &str,
    selected: bool,
    collapsed: bool,
) -> egui::Response {
    let response = ui.add_sized(
        [ui.available_width(), 46.0],
        egui::Button::new("")
            .fill(if selected { BLUE } else { Color32::TRANSPARENT })
            .stroke(egui::Stroke::NONE)
            .corner_radius(14),
    );
    let color = if selected { Color32::WHITE } else { MUTED };
    let r = response.rect;
    if !selected && response.hovered() {
        ui.painter().rect_filled(r, 14.0, SOFT_BLUE);
    }
    let x = if collapsed {
        r.center().x - 12.0
    } else {
        r.left() + 12.0
    };
    draw_glyph(
        ui.painter(),
        egui::Rect::from_min_size(egui::pos2(x, r.center().y - 12.0), egui::vec2(24.0, 24.0)),
        glyph,
        color,
    );
    if !collapsed {
        let mut job = egui::text::LayoutJob::simple(
            label.to_owned(),
            egui::FontId::proportional(15.0),
            if selected { Color32::WHITE } else { INK },
            (r.width() - 52.0).max(20.0),
        );
        job.wrap.max_rows = 1;
        job.wrap.break_anywhere = false;
        let galley = ui.painter().layout_job(job);
        ui.painter().galley(
            egui::pos2(
                r.left() + 46.0,
                ink_position(ui.painter(), &galley, r.center()).y,
            ),
            galley,
            color,
        );
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::SelectableLabel,
            ui.is_enabled(),
            selected,
            label,
        )
    });
    response.on_hover_text(label)
}
#[derive(Clone, Copy)]
enum ButtonKind {
    Primary,
    Secondary,
    Danger,
}
pub struct CenteredButton<'a> {
    label: &'a str,
    glyph: Option<Glyph>,
    kind: ButtonKind,
}
impl egui::Widget for CenteredButton<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let primary = matches!(self.kind, ButtonKind::Primary);
        let color = if primary {
            Color32::WHITE
        } else if matches!(self.kind, ButtonKind::Danger) {
            DANGER
        } else {
            BLUE
        };
        let (label, glyph) = if let Some(label) = self.label.strip_prefix("+ ") {
            (label, Some(Glyph::Plus))
        } else {
            (self.label, self.glyph)
        };
        let galley =
            ui.painter()
                .layout_no_wrap(label.to_owned(), egui::FontId::proportional(16.0), color);
        let icon_size = 22.0;
        let gap = 10.0;
        let group_width = galley.mesh_bounds.width()
            + if glyph.is_some() {
                icon_size + gap
            } else {
                0.0
            };
        let width = group_width + 36.0;
        let height = if primary { 44.0 } else { 40.0 };
        let response = ui.add(
            egui::Button::new("")
                .min_size(egui::vec2(width, height))
                .fill(if primary { BLUE } else { Color32::WHITE })
                .stroke(if primary {
                    egui::Stroke::NONE
                } else {
                    egui::Stroke::new(1.0_f32, BORDER)
                })
                .corner_radius(14),
        );
        if response.hovered() || response.has_focus() {
            ui.painter().rect(
                response.rect,
                14.0,
                if primary {
                    Color32::from_rgb(34, 88, 225)
                } else {
                    SOFT_BLUE
                },
                egui::Stroke::new(1.0_f32, BLUE),
                egui::StrokeKind::Inside,
            );
        }
        // Centre the whole visible composition. Layout direction only changes
        // the order, never its centre or padding, including add_sized toolbars.
        let rtl = false; // Russian content stays LTR even in a right-aligned toolbar.
        let extra = if glyph.is_some() {
            icon_size + gap
        } else {
            0.0
        };
        let label_center =
            response.rect.center() + egui::vec2(if rtl { -extra / 2.0 } else { extra / 2.0 }, 0.0);
        let label_min = ink_position(ui.painter(), &galley, label_center);
        if let Some(glyph) = glyph {
            let icon_x = response.rect.center().x
                + if rtl {
                    group_width / 2.0 - icon_size / 2.0
                } else {
                    -group_width / 2.0 + icon_size / 2.0
                };
            draw_glyph(
                ui.painter(),
                egui::Rect::from_center_size(
                    egui::pos2(icon_x, response.rect.center().y),
                    egui::vec2(icon_size, icon_size),
                ),
                glyph,
                color,
            );
        }
        ui.painter().galley(label_min, galley, color);
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label)
        });
        response
    }
}
pub fn action_button(
    ui: &mut egui::Ui,
    glyph: Glyph,
    label: &str,
    primary: bool,
) -> egui::Response {
    ui.add(CenteredButton {
        label,
        glyph: Some(glyph),
        kind: if primary {
            ButtonKind::Primary
        } else {
            ButtonKind::Secondary
        },
    })
}
pub fn primary_button(label: &str) -> CenteredButton<'_> {
    CenteredButton {
        label,
        glyph: None,
        kind: ButtonKind::Primary,
    }
}
pub fn secondary_button(label: &str) -> CenteredButton<'_> {
    CenteredButton {
        label,
        glyph: None,
        kind: ButtonKind::Secondary,
    }
}
pub fn danger_button(label: &str) -> CenteredButton<'_> {
    CenteredButton {
        label,
        glyph: None,
        kind: ButtonKind::Danger,
    }
}
pub fn search_input(
    ui: &mut egui::Ui,
    id: &str,
    value: &mut String,
    width: f32,
    hint: &str,
) -> egui::Response {
    egui::Frame::new()
        .fill(BG)
        .stroke(egui::Stroke::new(1.0_f32, BORDER))
        .corner_radius(14)
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.set_width((width - 24.0).max(80.0));
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    let (icon_rect, _) =
                        ui.allocate_exact_size(egui::vec2(22.0, 28.0), egui::Sense::hover());
                    let output = egui::TextEdit::singleline(value)
                        .id_salt(id)
                        .frame(false)
                        .font(egui::FontId::proportional(16.0))
                        .margin(egui::Vec2::ZERO)
                        .vertical_align(egui::Align::Center)
                        .desired_width((width - 70.0).max(50.0))
                        .min_size(egui::vec2((width - 70.0).max(50.0), 28.0))
                        .hint_text(hint)
                        .show(ui);
                    let galley = if value.is_empty() {
                        ui.painter().layout_no_wrap(
                            hint.to_owned(),
                            egui::FontId::proportional(16.0),
                            MUTED,
                        )
                    } else {
                        output.galley.clone()
                    };
                    let y = output.galley_pos.y + galley.mesh_bounds.center().y;
                    draw_glyph(
                        ui.painter(),
                        egui::Rect::from_center_size(
                            egui::pos2(icon_rect.center().x, y),
                            egui::vec2(22.0, 22.0),
                        ),
                        Glyph::Search,
                        MUTED,
                    );
                    output.response
                })
                .inner
            })
            .inner
        })
        .inner
}
pub fn title(ui: &mut egui::Ui, heading: &str, description: &str) {
    ui.label(
        RichText::new(heading)
            .size(28.0)
            .family(egui::FontFamily::Name("heading".into())),
    );
    ui.label(RichText::new(description).size(14.0).color(MUTED));
}
pub fn section_title(ui: &mut egui::Ui, glyph: Glyph, label: &str) {
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        egui::FontId::new(17.0, egui::FontFamily::Name("heading".into())),
        INK,
    );
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 28.0), egui::Sense::hover());
    draw_glyph(
        ui.painter(),
        egui::Rect::from_center_size(
            egui::pos2(rect.left() + 13.0, rect.center().y),
            egui::vec2(26.0, 26.0),
        ),
        glyph,
        BLUE,
    );
    let y = ink_position(ui.painter(), &galley, rect.center()).y;
    ui.painter()
        .galley(egui::pos2(rect.left() + 38.0, y), galley, INK);
    ui.add_space(6.0);
}
pub fn card<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::new()
        .fill(Color32::WHITE)
        .corner_radius(16)
        .stroke(egui::Stroke::new(1.0_f32, BORDER))
        .inner_margin(18)
        .show(ui, |ui| {
            // `Ui::columns` inherits horizontal justification. For wrapped
            // labels that can spread individual letters across a whole row.
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                ui.set_width(ui.available_width());
                add(ui)
            })
            .inner
        })
        .inner
}
pub fn empty_state(ui: &mut egui::Ui, glyph: Glyph, heading: &str, description: &str) {
    egui::Frame::new()
        .fill(SOFT_BLUE)
        .corner_radius(16)
        .inner_margin(20)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                icon_badge(ui, glyph, 44.0, Color32::WHITE, BLUE);
                ui.vertical(|ui| {
                    ui.label(RichText::new(heading).size(16.0).strong().color(INK));
                    ui.label(RichText::new(description).size(14.0).color(MUTED));
                });
            });
        });
}
pub fn configure_style(ctx: &egui::Context, zoom: f32) {
    let mut fonts = egui::FontDefinitions::default();
    #[cfg(target_os = "windows")]
    if let Some(windir) = std::env::var_os("WINDIR") {
        let dir = std::path::PathBuf::from(windir).join("Fonts");
        for (name, file) in [
            ("segoe-ui", "segoeui.ttf"),
            ("segoe-semibold", "seguisb.ttf"),
        ] {
            if let Ok(bytes) = std::fs::read(dir.join(file)) {
                let mut data = egui::FontData::from_owned(bytes);
                data.tweak.y_offset_factor = -0.08;
                fonts.font_data.insert(name.into(), data.into());
            }
        }
        if fonts.font_data.contains_key("segoe-ui") {
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "segoe-ui".into());
        }
    }
    let mut headings = fonts.families[&egui::FontFamily::Proportional].clone();
    if fonts.font_data.contains_key("segoe-semibold") {
        headings.insert(0, "segoe-semibold".into());
    }
    fonts
        .families
        .insert(egui::FontFamily::Name("heading".into()), headings);
    ctx.set_fonts(fonts);
    ctx.set_zoom_factor(zoom.clamp(0.8, 1.5));
    ctx.tessellation_options_mut(|options| {
        options.round_text_to_pixels = true;
        options.round_line_segments_to_pixels = true;
        options.feathering = true;
        options.feathering_size_in_pixels = 1.0;
    });
    let mut style = egui::Style::default();
    style.visuals = egui::Visuals::light();
    style.visuals.override_text_color = Some(INK);
    style.visuals.panel_fill = BG;
    style.visuals.window_fill = Color32::WHITE;
    style.visuals.extreme_bg_color = BG;
    style.visuals.window_stroke = egui::Stroke::new(1.0_f32, BORDER);
    style.visuals.window_corner_radius = egui::CornerRadius::same(16);
    for widget in [
        &mut style.visuals.widgets.noninteractive,
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
        &mut style.visuals.widgets.open,
    ] {
        widget.bg_fill = Color32::WHITE;
        widget.weak_bg_fill = Color32::WHITE;
        widget.bg_stroke = egui::Stroke::new(1.0_f32, BORDER);
        widget.fg_stroke = egui::Stroke::new(1.0_f32, BLUE);
        widget.corner_radius = egui::CornerRadius::same(14);
        widget.expansion = 0.0;
    }
    style.visuals.widgets.hovered.weak_bg_fill = SOFT_BLUE;
    style.visuals.widgets.noninteractive.bg_fill = BORDER;
    style.visuals.widgets.inactive.bg_fill = SOFT_BLUE;
    style.visuals.widgets.active.bg_fill = BLUE;
    style.visuals.faint_bg_color = Color32::from_rgb(247, 250, 255);
    style.visuals.widgets.hovered.bg_fill = SOFT_BLUE;
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, BLUE);
    style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.5_f32, BLUE);
    style.visuals.selection.bg_fill = SOFT_BLUE;
    style.visuals.selection.stroke = egui::Stroke::new(1.0_f32, BLUE);
    style.spacing.item_spacing = egui::vec2(12.0, 8.0);
    style.spacing.button_padding = egui::vec2(16.0, 9.0);
    style.spacing.interact_size = egui::vec2(40.0, 36.0);
    style.spacing.combo_height = 240.0;
    style.spacing.scroll = egui::style::ScrollStyle::solid();
    style.spacing.scroll.foreground_color = true;
    style.spacing.scroll.bar_width = 8.0;
    style.spacing.scroll.bar_inner_margin = 12.0;
    style.spacing.scroll.bar_outer_margin = 12.0;
    style.spacing.scroll.handle_min_length = 28.0;
    style.spacing.window_margin = egui::Margin::same(16);
    style.visuals.slider_trailing_fill = true;
    style.visuals.handle_shape = egui::style::HandleShape::Circle;
    style
        .text_styles
        .insert(egui::TextStyle::Body, egui::FontId::proportional(16.0));
    style
        .text_styles
        .insert(egui::TextStyle::Button, egui::FontId::proportional(16.0));
    style
        .text_styles
        .insert(egui::TextStyle::Heading, egui::FontId::proportional(20.0));
    style
        .text_styles
        .insert(egui::TextStyle::Small, egui::FontId::proportional(14.0));
    // Native startup can change the active theme after CreationContext.
    // Install both slots so no system-theme event restores egui defaults.
    ctx.set_theme(egui::Theme::Light);
    ctx.all_styles_mut(|target| *target = style.clone());
}
pub fn table_heading(ui: &mut egui::Ui, label: &str) {
    let r = ui.max_rect();
    ui.painter().rect_filled(r, 4.0, SOFT_BLUE);
    ui.painter()
        .hline(r.x_range(), r.bottom(), egui::Stroke::new(1.0_f32, BORDER));
    cell_label(ui, label, false, true);
}
pub fn cell_label(ui: &mut egui::Ui, text: &str, centered: bool, heading: bool) -> egui::Response {
    let rect = ui.max_rect();
    let response = ui.allocate_rect(
        rect,
        if heading {
            egui::Sense::hover()
        } else {
            egui::Sense::click()
        },
    );
    let mut job = egui::text::LayoutJob::simple(
        text.to_owned(),
        egui::FontId::proportional(16.0),
        if heading { MUTED } else { INK },
        (rect.width() - 24.0).max(8.0),
    );
    job.wrap.max_rows = 1;
    let galley = ui.painter().layout_job(job);
    let center = egui::pos2(
        if centered {
            rect.center().x
        } else {
            rect.left() + 12.0 + galley.mesh_bounds.width() / 2.0
        },
        rect.center().y,
    );
    let pos = ink_position(ui.painter(), &galley, center);
    ui.painter()
        .galley(pos, galley, if heading { MUTED } else { INK });
    response.on_hover_text(text)
}
pub fn quiet_checkbox(ui: &mut egui::Ui, value: &mut bool, label: &str) -> egui::Response {
    let galley =
        ui.painter()
            .layout_no_wrap(label.to_owned(), egui::FontId::proportional(16.0), INK);
    let (r, mut response) = ui.allocate_exact_size(
        egui::vec2(
            24.0 + if label.is_empty() {
                0.0
            } else {
                10.0 + galley.size().x
            },
            32.0,
        ),
        egui::Sense::click(),
    );
    if response.clicked() {
        *value = !*value;
        response.mark_changed();
    }
    let box_rect = egui::Rect::from_center_size(
        egui::pos2(r.left() + 12.0, r.center().y),
        egui::vec2(20.0, 20.0),
    );
    ui.painter().rect(
        box_rect,
        5.0,
        if *value { BLUE } else { Color32::WHITE },
        egui::Stroke::new(
            1.0_f32,
            if *value || response.hovered() {
                BLUE
            } else {
                BORDER
            },
        ),
        egui::StrokeKind::Inside,
    );
    if *value {
        ui.painter().add(egui::Shape::line(
            vec![
                box_rect.center() + egui::vec2(-5.0, 0.0),
                box_rect.center() + egui::vec2(-1.0, 4.0),
                box_rect.center() + egui::vec2(6.0, -4.0),
            ],
            egui::Stroke::new(2.0_f32, Color32::WHITE),
        ));
    }
    let pos = ink_position(
        ui.painter(),
        &galley,
        egui::pos2(
            r.left() + 34.0 + galley.mesh_bounds.width() / 2.0,
            r.center().y,
        ),
    );
    ui.painter().galley(pos, galley, INK);
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *value, label)
    });
    response
}
/// Read-only preview uses the normal contrast; it never owns a TextEdit or data reference.
pub fn preview_field(ui: &mut egui::Ui, field: &FieldDef) {
    ui.label(
        RichText::new(format!(
            "{}{}",
            field_label(&field.name),
            if field.required { " *" } else { "" }
        ))
        .size(14.0)
        .color(MUTED),
    );
    let height = if field.kind == FieldKind::LongText {
        76.0
    } else {
        40.0
    };
    let (r, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        egui::Sense::hover(),
    );
    ui.painter().rect(
        r,
        14.0,
        BG,
        egui::Stroke::new(1.0_f32, BORDER),
        egui::StrokeKind::Inside,
    );
    let value = if !field.default_value.is_empty() {
        field.default_value.as_str()
    } else {
        match field.kind {
            FieldKind::Date => crate::i18n::tr("ГГГГ-ММ-ДД"),
            FieldKind::Boolean => crate::i18n::tr("Да / Нет"),
            FieldKind::File => crate::i18n::tr("Документ"),
            FieldKind::Url => "https://example.com",
            _ => crate::i18n::tr("Введите значение"),
        }
    };
    let galley = ui.painter().layout(
        value.to_owned(),
        egui::FontId::proportional(16.0),
        INK,
        (r.width() - 24.0).max(20.0),
    );
    let pos = ink_position(
        ui.painter(),
        &galley,
        egui::pos2(
            r.left() + 12.0 + galley.mesh_bounds.width() / 2.0,
            r.center().y,
        ),
    );
    ui.painter().galley(pos, galley, INK);
}
pub fn edit_field(ui: &mut egui::Ui, field: &FieldDef, value: &mut String) -> bool {
    ui.push_id(field.id, |ui| {
        let label = RichText::new(format!(
            "{}{}",
            field_label(&field.name),
            if field.required { " *" } else { "" }
        ))
        .color(MUTED)
        .size(14.0);
        if ui.available_width() >= 380.0 {
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(140.0, 36.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.set_min_width(140.0);
                        ui.label(label);
                    },
                );
                ui.vertical(|ui| field_input(ui, field, value)).inner
            })
            .inner
        } else {
            ui.label(label);
            field_input(ui, field, value)
        }
    })
    .inner
}
fn field_input(ui: &mut egui::Ui, field: &FieldDef, value: &mut String) -> bool {
    let width = ui.available_width();
    match field.kind {
        FieldKind::LongText => ui
            .add_sized(
                [width, 76.0],
                egui::TextEdit::multiline(value)
                    .margin(egui::vec2(10.0, 8.0))
                    .desired_rows(3),
            )
            .changed(),
        FieldKind::Boolean => {
            let mut yes = value == "1";
            if quiet_checkbox(ui, &mut yes, crate::i18n::tr("Да")).changed() {
                *value = if yes { "1" } else { "0" }.into();
                true
            } else {
                false
            }
        }
        FieldKind::Date => {
            let mut changed = false;
            ui.horizontal(|ui| {
                changed |= ui
                    .add_sized(
                        [(width - 52.0).max(60.0), 38.0],
                        egui::TextEdit::singleline(value)
                            .margin(egui::vec2(12.0, 8.0))
                            .vertical_align(egui::Align::Center)
                            .hint_text(crate::i18n::tr("ГГГГ-ММ-ДД")),
                    )
                    .changed();
                let mut date = NaiveDate::parse_from_str(value, "%Y-%m-%d")
                    .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2010, 1, 1).unwrap());
                let response = localized_date_picker(ui, &mut date, &field.name);
                draw_glyph(
                    ui.painter(),
                    egui::Rect::from_center_size(response.rect.center(), egui::vec2(20.0, 20.0)),
                    Glyph::Calendar,
                    MUTED,
                );
                if response.changed() {
                    *value = date.format("%Y-%m-%d").to_string();
                    changed = true;
                }
                response.on_hover_text(crate::i18n::tr("Выбрать дату"));
            });
            changed
        }
        _ => ui
            .add_sized(
                [width, 36.0],
                egui::TextEdit::singleline(value)
                    .margin(egui::vec2(12.0, 8.0))
                    .vertical_align(egui::Align::Center)
                    .hint_text(match field.kind {
                        FieldKind::Integer => crate::i18n::tr("Целое число"),
                        FieldKind::Decimal => crate::i18n::tr("Число"),
                        FieldKind::Time => crate::i18n::tr("ЧЧ:ММ"),
                        FieldKind::DateTime => crate::i18n::tr("ГГГГ-ММ-ДД ЧЧ:ММ"),
                        FieldKind::Url => "https://example.com",
                        _ => crate::i18n::tr("Введите значение"),
                    }),
            )
            .changed(),
    }
}

pub fn field_label(name: &str) -> String {
    let label = match name {
        "Адрес_Проживания" | "Адрес Проживания" => "Адрес проживания",
        "Увлечения_Хобби" | "Увлечения Хобби" => "Увлечения, хобби",
        "Ссылка_На_Сайт" | "Ссылка На Сайт" => "Ссылка на сайт",
        "Дата_Рождения" | "Дата Рождения" => "Дата рождения",
        _ => name,
    };
    crate::i18n::tr(label).replace('_', " ")
}
pub fn plural<'a>(n: usize, one: &'a str, few: &'a str, many: &'a str) -> &'a str {
    if (11..=14).contains(&(n % 100)) {
        many
    } else {
        match n % 10 {
            1 => one,
            2..=4 => few,
            _ => many,
        }
    }
}
pub fn record_word(n: usize) -> &'static str {
    match crate::i18n::current() {
        crate::i18n::Language::Ru => plural(n, "запись", "записи", "записей"),
        _ => crate::i18n::tr(if n == 1 {
            "запись"
        } else {
            "записи"
        }),
    }
}
pub fn field_word(n: usize) -> &'static str {
    match crate::i18n::current() {
        crate::i18n::Language::Ru => plural(n, "поле", "поля", "полей"),
        _ => crate::i18n::tr(if n == 1 { "поле" } else { "поля" }),
    }
}
pub fn copy_word(n: usize) -> &'static str {
    match crate::i18n::current() {
        crate::i18n::Language::Ru => {
            plural(n, "резервная копия", "резервные копии", "резервных копий")
        }
        _ => crate::i18n::tr(if n == 1 {
            "резервная копия"
        } else {
            "резервные копии"
        }),
    }
}

pub fn cache_column_headers(ctx: &egui::Context, key: &str, rects: Vec<egui::Rect>) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(("column-headers", key)), rects));
}
/// Supplement TableBuilder for native integrations that batch an entire short
/// gesture in one frame. Normal multi-frame resizing remains TableBuilder's.
pub fn batched_column_resize(
    ctx: &egui::Context,
    key: &str,
    widths: &mut Vec<f32>,
    minimums: &[f32],
) -> bool {
    let time = ctx.input(|i| i.time);
    let done = egui::Id::new(("column-batch-time", key));
    if ctx.data(|d| d.get_temp::<f64>(done)) == Some(time) {
        return false;
    }
    let rects = ctx
        .data(|d| d.get_temp::<Vec<egui::Rect>>(egui::Id::new(("column-headers", key))))
        .unwrap_or_default();
    let drag_id = egui::Id::new(("column-pointer-start", key));
    let mut start = ctx.data(|d| d.get_temp::<(usize, f32, f32)>(drag_id));
    for event in ctx.input(|i| i.events.clone()) {
        match event {
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                ..
            } => {
                start = rects
                    .iter()
                    .enumerate()
                    .find(|(_, r)| {
                        pos.y >= r.top()
                            && pos.y <= r.bottom()
                            && (pos.x - r.right() - 6.0).abs() <= 8.0
                    })
                    .map(|(i, r)| (i, pos.x, r.width()));
            }
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                ..
            } => {
                if let Some((index, x, width)) = start.take() {
                    if (pos.x - x).abs() > 4.0 {
                        if widths.len() != rects.len() {
                            *widths = rects.iter().map(|r| r.width()).collect();
                        }
                        widths[index] = (width + pos.x - x)
                            .clamp(minimums.get(index).copied().unwrap_or(80.0), 1200.0);
                        ctx.data_mut(|d| {
                            d.insert_temp(done, time);
                            d.remove::<(usize, f32, f32)>(drag_id);
                        });
                        return true;
                    }
                }
            }
            egui::Event::PointerGone => start = None,
            _ => {}
        }
    }
    ctx.data_mut(|d| {
        if let Some(start) = start {
            d.insert_temp(drag_id, start);
        } else {
            d.remove::<(usize, f32, f32)>(drag_id);
        }
    });
    false
}

pub fn result_details(
    ui: &mut egui::Ui,
    name: &str,
    details: &str,
    width: f32,
    height: f32,
) -> egui::Rect {
    let mut name_job = egui::text::LayoutJob::simple(
        name.to_owned(),
        egui::FontId::new(15.0, egui::FontFamily::Name("heading".into())),
        INK,
        width,
    );
    name_job.wrap.max_rows = 2;
    let mut details_job = egui::text::LayoutJob::simple(
        details.to_owned(),
        egui::FontId::proportional(14.0),
        MUTED,
        width,
    );
    details_job.wrap.max_rows = 2;
    let lines = [
        ui.painter().layout_job(name_job),
        ui.painter().layout_job(details_job),
    ];
    let total = lines.iter().map(|g| g.mesh_bounds.height()).sum::<f32>() + 8.0;
    let (r, response) =
        ui.allocate_exact_size(egui::vec2(width, height.max(total)), egui::Sense::hover());
    let mut y = r.center().y - total / 2.0;
    for (index, galley) in lines.into_iter().enumerate() {
        let h = galley.mesh_bounds.height();
        let pos = ink_position(
            ui.painter(),
            &galley,
            egui::pos2(r.left() + galley.mesh_bounds.width() / 2.0, y + h / 2.0),
        );
        ui.painter()
            .galley(pos, galley, if index == 0 { INK } else { MUTED });
        y += h + 8.0;
    }
    response.on_hover_text(format!("{name}\n{details}"));
    r
}

fn localized_date_picker(
    ui: &mut egui::Ui,
    selection: &mut NaiveDate,
    key: &str,
) -> egui::Response {
    let (_, mut response) = ui.allocate_exact_size(egui::vec2(40.0, 38.0), egui::Sense::click());
    ui.painter()
        .rect_filled(response.rect, 12.0, Color32::WHITE);
    ui.painter().rect_stroke(
        response.rect,
        12.0,
        egui::Stroke::new(1.0_f32, BORDER),
        egui::StrokeKind::Inside,
    );
    let id = ui.make_persistent_id(("calendar", key));
    let state_id = id.with("month");
    if response.clicked() {
        ui.memory_mut(|m| m.toggle_popup(id));
        ui.data_mut(|d| d.insert_temp(state_id, selection.with_day(1).unwrap()));
    }
    let mut chosen = None;
    egui::popup_below_widget(
        ui,
        id,
        &response,
        egui::PopupCloseBehavior::CloseOnClickOutside,
        |ui| {
            let mut month = ui
                .data(|d| d.get_temp::<NaiveDate>(state_id))
                .unwrap_or(selection.with_day(1).unwrap());
            ui.horizontal(|ui| {
                if ui.button("‹").clicked() {
                    month = month
                        .checked_sub_months(chrono::Months::new(1))
                        .unwrap_or(month);
                }
                ui.label(month.format("%Y-%m").to_string());
                if ui.button("›").clicked() {
                    month = month
                        .checked_add_months(chrono::Months::new(1))
                        .unwrap_or(month);
                }
            });
            ui.data_mut(|d| d.insert_temp(state_id, month));
            let weekdays = match crate::i18n::current() {
                crate::i18n::Language::Ru => ["Пн", "Вт", "Ср", "Чт", "Пт", "Сб", "Вс"],
                crate::i18n::Language::De => ["Mo", "Di", "Mi", "Do", "Fr", "Sa", "So"],
                crate::i18n::Language::En => ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"],
            };
            egui::Grid::new(id.with("days"))
                .spacing(egui::vec2(6.0, 6.0))
                .show(ui, |ui| {
                    for day in weekdays {
                        ui.label(day);
                    }
                    ui.end_row();
                    let offset = month.weekday().num_days_from_monday();
                    let mut col = 0;
                    for _ in 0..offset {
                        ui.label("");
                        col += 1;
                    }
                    for day in 1..=31 {
                        let Some(date) = month.with_day(day) else {
                            break;
                        };
                        if ui
                            .selectable_label(date == *selection, day.to_string())
                            .clicked()
                        {
                            chosen = Some(date);
                        }
                        col += 1;
                        if col == 7 {
                            ui.end_row();
                            col = 0;
                        }
                    }
                });
        },
    );
    if let Some(date) = chosen {
        *selection = date;
        response.mark_changed();
        ui.memory_mut(|m| m.close_popup());
    }
    response
}
