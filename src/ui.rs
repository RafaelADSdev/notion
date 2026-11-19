use eframe::egui::{
    self, pos2, vec2, Align, Align2, Button, Color32, ComboBox, Context, FontFamily, FontId,
    Frame, Layout, Painter, Rect, Response, RichText, Sense, Stroke, StrokeKind, TextEdit, Ui,
};

use crate::model::{PageKind, Status};

fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

#[derive(Clone, Copy)]
pub struct Palette {
    pub paper: Color32,
    pub side: Color32,
    pub white: Color32,
    pub ink: Color32,
    pub muted: Color32,
    pub line: Color32,
    pub control_outline: Color32,
    pub accent: Color32,
    pub tint: Color32,
    pub on_accent: Color32,
}

impl Palette {
    pub fn new(dark: bool, high_contrast: bool) -> Self {
        let mut palette = if dark {
            Self {
                paper: rgb(29, 35, 33), side: rgb(23, 29, 27), white: rgb(35, 42, 39),
                ink: rgb(237, 240, 232), muted: rgb(173, 184, 176), line: rgb(56, 67, 61),
                control_outline: rgb(111, 131, 119),
                accent: rgb(158, 208, 189), tint: rgb(43, 62, 54), on_accent: rgb(23, 47, 37),
            }
        } else {
            Self {
                paper: rgb(253, 252, 248), side: rgb(242, 244, 237), white: rgb(255, 255, 252),
                ink: rgb(37, 44, 40), muted: rgb(88, 101, 92), line: rgb(221, 226, 216),
                control_outline: rgb(126, 142, 130),
                accent: rgb(41, 101, 83), tint: rgb(228, 239, 230), on_accent: Color32::WHITE,
            }
        };
        if high_contrast {
            palette.muted = palette.ink;
            palette.control_outline = palette.ink;
            palette.line = if dark { rgb(143, 159, 151) } else { rgb(80, 95, 85) };
        }
        palette
    }

    pub fn apply(self, ctx: &Context, dark: bool, compact: bool) {
        ctx.set_theme(if dark { egui::Theme::Dark } else { egui::Theme::Light });
        let mut style = (*ctx.style()).clone();
        style.visuals = if dark { egui::Visuals::dark() } else { egui::Visuals::light() };
        style.visuals.override_text_color = Some(self.ink);
        style.visuals.panel_fill = self.paper;
        style.visuals.window_fill = self.white;
        style.visuals.window_stroke = Stroke::new(1.0_f32, self.line);
        style.visuals.window_corner_radius = egui::CornerRadius::same(10);
        style.visuals.extreme_bg_color = self.paper;
        style.visuals.hyperlink_color = self.accent;
        style.spacing.window_margin = egui::Margin::same(16);
        for visuals in [
            &mut style.visuals.widgets.inactive,
            &mut style.visuals.widgets.hovered,
            &mut style.visuals.widgets.active,
            &mut style.visuals.widgets.open,
        ] {
            visuals.bg_fill = self.white;
            visuals.weak_bg_fill = self.white;
            visuals.bg_stroke.color = self.line;
            visuals.corner_radius = egui::CornerRadius::same(6);
            visuals.fg_stroke.color = self.ink;
        }
        style.visuals.widgets.noninteractive.bg_stroke.color = self.line;
        style.visuals.widgets.inactive.bg_fill = self.side;
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, self.control_outline);
        style.visuals.widgets.hovered.weak_bg_fill = self.tint;
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, self.accent);
        style.visuals.widgets.hovered.expansion = 0.0;
        style.visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, self.accent);
        style.visuals.widgets.active.weak_bg_fill = self.tint;
        style.visuals.selection.bg_fill = self.tint;
        style.visuals.selection.stroke.color = self.accent;
        style.spacing.button_padding = vec2(10.0, 5.0);
        style.spacing.item_spacing = if compact { vec2(6.0, 4.0) } else { vec2(8.0, 7.0) };
        ctx.set_style(style);
    }

    pub fn label(self, text: impl ToString) -> RichText {
        RichText::new(text.to_string()).size(13.0).color(self.muted)
    }

    pub fn heading(self, text: impl ToString, size: f32) -> RichText {
        RichText::new(text.to_string()).size(size).strong().color(self.ink)
    }

    pub fn display(self, text: impl ToString, size: f32) -> RichText {
        RichText::new(text.to_string()).font(display_font(size)).color(self.ink)
    }
}

pub fn display_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("margem-display".into()))
}

pub fn install_fonts(ctx: &Context) {
    let mut fonts = egui::FontDefinitions::default();
    if let Ok(data) = std::fs::read("C:/Windows/Fonts/segoeui.ttf") {
        fonts.font_data.insert("segoe".into(), egui::FontData::from_owned(data).into());
        fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "segoe".into());
    }
    let mut display = fonts.families[&FontFamily::Proportional].clone();
    if let Ok(data) = std::fs::read("C:/Windows/Fonts/georgia.ttf") {
        fonts.font_data.insert("georgia".into(), egui::FontData::from_owned(data).into());
        display.insert(0, "georgia".into());
    }
    fonts.families.insert(FontFamily::Name("margem-display".into()), display);
    ctx.set_fonts(fonts);
}

pub fn action(ui: &mut Ui, p: Palette, label: &str, primary: bool) -> Response {
    let text = if primary { p.on_accent } else { p.ink };
    ui.scope(|ui| {
        if primary {
            let widgets = &mut ui.visuals_mut().widgets;
            widgets.inactive.weak_bg_fill = p.accent;
            widgets.hovered.weak_bg_fill = p.accent.gamma_multiply(0.93);
            widgets.active.weak_bg_fill = p.accent.gamma_multiply(0.85);
            widgets.inactive.bg_stroke = Stroke::new(1.0_f32, p.accent);
            widgets.hovered.bg_stroke = Stroke::new(1.0_f32, p.ink);
            widgets.active.bg_stroke = Stroke::new(1.0_f32, p.ink);
        } else {
            ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::new(1.0_f32, p.line);
        }
        ui.add(Button::new(RichText::new(label).size(13.0).color(text))
            .corner_radius(6).min_size(vec2(0.0, 29.0)))
    }).inner.on_hover_cursor(egui::CursorIcon::PointingHand)
}

#[derive(Clone, Copy)]
pub enum Icon {
    Home, Search, Models, Trash, Settings, Page, Collection, Star,
}

fn paint_icon(painter: &Painter, center: egui::Pos2, size: f32, icon: Icon, color: Color32) {
    let at = |x, y| center + vec2(x, y) * (size / 16.0);
    let stroke = Stroke::new((size * 0.085).max(1.15), color);
    let line = |a, b| { painter.line_segment([a, b], stroke); };
    let path = |points| { painter.add(egui::Shape::line(points, stroke)); };
    let outline = |left, top, right, bottom| {
        painter.rect_stroke(Rect::from_min_max(at(left, top), at(right, bottom)), 2, stroke, StrokeKind::Inside);
    };
    match icon {
        Icon::Home => {
            path(vec![at(-6.0, -0.5), at(0.0, -6.0), at(6.0, -0.5)]);
            path(vec![at(-4.5, -1.5), at(-4.5, 5.5), at(4.5, 5.5), at(4.5, -1.5)]);
        }
        Icon::Search => {
            painter.circle_stroke(at(-1.0, -1.0), size * 0.26, stroke);
            line(at(2.1, 2.1), at(5.8, 5.8));
        }
        Icon::Models => {
            outline(-5.0, -6.0, 5.0, 6.0);
            line(at(-2.5, -2.5), at(2.5, -2.5));
            line(at(-2.5, 0.3), at(2.5, 0.3));
        }
        Icon::Trash => {
            line(at(-6.0, -4.0), at(6.0, -4.0));
            line(at(-2.0, -6.0), at(2.0, -6.0));
            path(vec![at(-4.5, -4.0), at(-3.5, 5.5), at(3.5, 5.5), at(4.5, -4.0)]);
        }
        Icon::Settings => {
            for (y, x) in [(-4.0, -2.2), (0.0, 2.2), (4.0, -0.5)] {
                line(at(-6.0, y), at(6.0, y));
                line(at(x, y - 1.6), at(x, y + 1.6));
            }
        }
        Icon::Page => {
            path(vec![at(0.5, -6.0), at(-4.5, -6.0), at(-4.5, 6.0), at(4.5, 6.0), at(4.5, -2.0), at(0.5, -6.0), at(0.5, -2.0), at(4.5, -2.0)]);
        }
        Icon::Collection => {
            outline(-6.0, -5.0, 6.0, 5.0);
            line(at(-6.0, -1.7), at(6.0, -1.7));
            line(at(-1.2, -1.7), at(-1.2, 5.0));
        }
        Icon::Star => {
            let points = (0..=10).map(|i| {
                let angle = (i as f32) * std::f32::consts::PI / 5.0 - std::f32::consts::FRAC_PI_2;
                let radius = if i % 2 == 0 { 6.3 } else { 2.9 };
                at(radius * angle.cos(), radius * angle.sin())
            }).collect::<Vec<_>>();
            path(points);
        }
    }
}

pub fn page_icon(kind: PageKind) -> Icon {
    if kind == PageKind::Collection { Icon::Collection } else { Icon::Page }
}

pub fn icon_tile(ui: &mut Ui, p: Palette, icon: Icon, size: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    ui.painter().rect_filled(rect, 9, p.tint);
    paint_icon(ui.painter(), rect.center(), size * 0.58, icon, p.accent);
}

pub fn nav(ui: &mut Ui, p: Palette, label: &str, selected: bool, height: f32) -> Response {
    navigation(ui, p, label, None, selected, height)
}

pub fn nav_icon(ui: &mut Ui, p: Palette, label: &str, icon: Icon, selected: bool, height: f32) -> Response {
    navigation(ui, p, label, Some(icon), selected, height)
}

fn navigation(ui: &mut Ui, p: Palette, label: &str, icon: Option<Icon>, selected: bool, height: f32) -> Response {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label));
    let painter = ui.painter().with_clip_rect(rect);
    if selected || response.hovered() || response.has_focus() {
        painter.rect_filled(rect, 5, p.tint);
    }
    if selected {
        let rail = Rect::from_center_size(rect.left_center() + vec2(2.0, 0.0), vec2(2.0, 13.0));
        painter.rect_filled(rail, 1, p.accent);
    }
    if response.has_focus() {
        painter.rect_stroke(rect, 5, Stroke::new(1.0_f32, p.accent), StrokeKind::Inside);
    }
    let foreground = if selected { p.accent } else { p.ink };
    let offset = if let Some(icon) = icon {
        paint_icon(&painter, rect.left_center() + vec2(15.0, 0.0), 16.0, icon, if selected { p.accent } else { p.muted });
        30.0
    } else { 10.0 };
    row_text(&painter, rect, offset, 8.0, label, foreground);
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn search_button(ui: &mut Ui, p: Palette) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 33.0), Sense::click());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), "Buscar"));
    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(rect, 6, if response.hovered() { p.tint } else { p.paper });
    let stroke = if response.has_focus() { p.accent } else { p.line };
    painter.rect_stroke(rect, 6, Stroke::new(1.0_f32, stroke), StrokeKind::Inside);
    paint_icon(&painter, rect.left_center() + vec2(15.0, 0.0), 16.0, Icon::Search, p.muted);
    row_text(&painter, rect, 30.0, 63.0, "Buscar", p.ink);
    let shortcut = Rect::from_center_size(rect.right_center() - vec2(29.0, 0.0), vec2(44.0, 19.0));
    painter.rect_filled(shortcut, 3, p.side);
    painter.text(shortcut.center(), Align2::CENTER_CENTER, "Ctrl+K", FontId::proportional(10.5), p.muted);
    response.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text("Buscar (Ctrl+K)")
}

pub fn section(ui: &mut Ui, p: Palette, name: &str) {
    ui.label(RichText::new(name).size(11.0).strong().color(p.muted));
}

pub fn logo(ui: &mut Ui, p: Palette) {
    let (rect, _) = ui.allocate_exact_size(vec2(30.0, 30.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 7, p.accent);
    let x = rect.left();
    let y = rect.top();
    let stroke = Stroke::new(1.35_f32, p.on_accent);
    painter.line_segment([pos2(x + 10.0, y + 6.0), pos2(x + 10.0, y + 24.0)], stroke);
    for (offset, length) in [(9.0, 13.0), (15.0, 10.0), (21.0, 13.0)] {
        painter.line_segment([pos2(x + 6.0, y + offset), pos2(x + 6.0 + length, y + offset)], stroke);
    }
}

pub fn app_icon() -> egui::IconData {
    let p = Palette::new(false, false);
    let mut rgba = Vec::with_capacity(32 * 32 * 4);
    for y in 0_i32..32 {
        for x in 0_i32..32 {
            let dx = (x - 15).abs().min((x - 16).abs());
            let dy = (y - 15).abs().min((y - 16).abs());
            let corner = (dx - 10).max(0).pow(2) + (dy - 10).max(0).pow(2);
            let rules = (10..=11).contains(&x) && (7..=24).contains(&y)
                || (6..=22).contains(&x) && matches!(y, 9 | 10 | 21 | 22)
                || (6..=18).contains(&x) && matches!(y, 15 | 16);
            let color = if corner > 25 {
                Color32::TRANSPARENT
            } else if rules {
                p.on_accent
            } else {
                p.accent
            };
            rgba.extend_from_slice(&color.to_array());
        }
    }
    egui::IconData { rgba, width: 32, height: 32 }
}

pub fn page_row(ui: &mut Ui, p: Palette, label: &str, kind: PageKind, trail: &str) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 41.0), Sense::click());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label));
    let painter = ui.painter().with_clip_rect(rect);
    if response.hovered() || response.has_focus() {
        painter.rect_filled(rect, 7, p.tint);
    }
    if response.has_focus() {
        painter.rect_stroke(rect, 7, Stroke::new(1.0_f32, p.accent), StrokeKind::Inside);
    }
    let badge = Rect::from_center_size(rect.left_center() + vec2(17.0, 0.0), vec2(25.0, 25.0));
    painter.rect_filled(badge, 5, p.tint);
    paint_icon(&painter, badge.center(), 16.0, page_icon(kind), p.accent);
    let trail_font = FontId::proportional(12.0);
    let trail_width = painter.layout_no_wrap(trail.to_owned(), trail_font.clone(), p.muted).size().x;
    row_text(&painter, rect, 36.0, trail_width + 20.0, label, p.ink);
    painter.text(rect.right_center() - vec2(10.0, 0.0), Align2::RIGHT_CENTER, trail, trail_font, p.muted);
    response.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text(label)
}

fn row_text(painter: &Painter, rect: Rect, left: f32, right: f32, label: &str, color: Color32) {
    let mut job = egui::text::LayoutJob::simple_singleline(label.to_owned(), FontId::proportional(14.0), color);
    job.wrap.max_width = (rect.width() - left - right).max(1.0);
    job.wrap.max_rows = 1;
    let galley = painter.layout_job(job);
    painter.galley(rect.left_center() + vec2(left, -galley.rect.height() / 2.0), galley, color);
}

pub fn panel(p: Palette) -> Frame {
    Frame::new().fill(p.white).stroke(Stroke::new(1.0_f32, p.line)).corner_radius(8).inner_margin(14)
}

pub fn status_box(ui: &mut Ui, id: impl std::hash::Hash, status: &mut Status) -> bool {
    let previous = *status;
    let fill = match (ui.visuals().dark_mode, previous) {
        (false, Status::Planned) => rgb(237, 240, 238),
        (false, Status::Active) => rgb(248, 234, 207),
        (false, Status::Done) => rgb(218, 240, 226),
        (true, Status::Planned) => rgb(51, 63, 58),
        (true, Status::Active) => rgb(78, 61, 32),
        (true, Status::Done) => rgb(38, 71, 56),
    };
    ui.scope(|ui| {
        ui.visuals_mut().widgets.inactive.weak_bg_fill = fill;
        ui.visuals_mut().widgets.inactive.bg_fill = fill;
        ComboBox::from_id_salt(id).width(110.0).selected_text(status.label()).show_ui(ui, |ui| {
            for choice in Status::ALL {
                ui.selectable_value(status, choice, choice.label());
            }
        });
    });
    previous != *status
}

pub fn editable<'a>(text: &'a mut String, hint: &str, font: FontId) -> TextEdit<'a> {
    TextEdit::singleline(text).frame(false).hint_text(hint).font(font).desired_width(f32::INFINITY)
}

pub fn right(ui: &mut Ui, show: impl FnOnce(&mut Ui)) {
    ui.with_layout(Layout::right_to_left(Align::Center), show);
}
