use eframe::egui::{
    self, pos2, vec2, Align, Align2, Button, Color32, ComboBox, Context, FontFamily, FontId,
    Frame, Layout, Rect, Response, RichText, Sense, Stroke, StrokeKind, TextEdit, Ui,
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
                paper: rgb(23, 29, 28), side: rgb(30, 39, 37), white: rgb(33, 42, 40),
                ink: rgb(237, 244, 239), muted: rgb(168, 187, 179), line: rgb(61, 77, 71),
                control_outline: rgb(112, 136, 125),
                accent: rgb(144, 218, 186), tint: rgb(40, 66, 56), on_accent: rgb(20, 44, 35),
            }
        } else {
            Self {
                paper: Color32::WHITE, side: rgb(239, 245, 255), white: Color32::WHITE,
                ink: rgb(25, 40, 65), muted: rgb(74, 93, 121), line: rgb(210, 223, 242),
                control_outline: rgb(119, 140, 171),
                accent: rgb(31, 87, 183), tint: rgb(224, 237, 255), on_accent: Color32::WHITE,
            }
        };
        if high_contrast {
            palette.muted = palette.ink;
            palette.control_outline = palette.ink;
            palette.line = if dark { rgb(143, 159, 151) } else { rgb(88, 111, 146) };
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
        style.visuals.extreme_bg_color = self.paper;
        style.visuals.hyperlink_color = self.accent;
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
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, self.control_outline);
        style.visuals.widgets.hovered.weak_bg_fill = self.tint;
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, self.accent);
        style.visuals.widgets.hovered.expansion = 0.0;
        style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, self.accent);
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
}

pub fn install_fonts(ctx: &Context) {
    let mut fonts = egui::FontDefinitions::default();
    if let Ok(data) = std::fs::read("C:/Windows/Fonts/segoeui.ttf") {
        fonts.font_data.insert("segoe".into(), egui::FontData::from_owned(data).into());
        fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "segoe".into());
    }
    ctx.set_fonts(fonts);
}

pub fn action(ui: &mut Ui, p: Palette, label: &str, primary: bool) -> Response {
    let text = if primary { p.on_accent } else { p.ink };
    let fill = if primary { p.accent } else { p.white };
    ui.add(Button::new(RichText::new(label).size(13.0).strong().color(text))
        .fill(fill).stroke(Stroke::new(1.0, if primary { p.accent } else { p.line }))
        .corner_radius(8).min_size(vec2(0.0, 31.0)))
}

pub fn nav(ui: &mut Ui, p: Palette, label: &str, selected: bool, height: f32) -> Response {
    let width = ui.available_width();
    let response = ui.with_layout(Layout::top_down(Align::Min), |ui| {
        let label = RichText::new(label).size(14.0);
        let label = if selected { label.strong().color(p.accent) } else { label.color(p.ink) };
        let mut button = Button::new(label)
            .stroke(Stroke::NONE).corner_radius(7).truncate().min_size(vec2(width, height));
        if selected {
            button = button.fill(p.tint);
        }
        ui.visuals_mut().widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
        ui.add(button)
    }).inner;
    if selected {
        let rail = Rect::from_center_size(response.rect.left_center() + vec2(2.0, 0.0), vec2(2.0, 14.0));
        ui.painter().rect_filled(rail, 1, p.accent);
    }
    response
}

pub fn search_button(ui: &mut Ui, p: Palette) -> Response {
    let width = ui.available_width();
    let response = ui.with_layout(Layout::top_down(Align::Min), |ui| {
        ui.add(Button::new(RichText::new("Buscar").size(13.0).color(p.ink))
            .stroke(Stroke::new(1.0, p.line)).corner_radius(8).min_size(vec2(width, 33.0)))
    }).inner;
    let shortcut = Rect::from_center_size(response.rect.right_center() - vec2(33.0, 0.0), vec2(49.0, 21.0));
    ui.painter().rect_filled(shortcut, 4, p.side);
    ui.painter().text(shortcut.center(), Align2::CENTER_CENTER, "Ctrl+K", FontId::proportional(11.0), p.muted);
    response.on_hover_text("Buscar (Ctrl+K)")
}

pub fn section(ui: &mut Ui, p: Palette, name: &str) {
    ui.label(RichText::new(name).size(11.0).strong().color(p.muted));
}

pub fn logo(ui: &mut Ui, p: Palette) {
    let (rect, _) = ui.allocate_exact_size(vec2(30.0, 30.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 8, p.accent);
    for (x, width) in [(7.0, 3.0), (13.5, 3.0), (20.0, 3.0)] {
        let top = rect.top() + 8.0 + (x - 7.0) * 0.29;
        painter.rect_filled(Rect::from_min_size(pos2(rect.left() + x, top), vec2(width, rect.bottom() - top - 6.0)), 1, p.on_accent);
    }
}

pub fn page_row(ui: &mut Ui, p: Palette, label: &str, kind: PageKind, trail: &str) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 41.0), Sense::click());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label));
    let painter = ui.painter().with_clip_rect(rect);
    if response.hovered() || response.has_focus() {
        painter.rect_filled(rect, 7, p.tint);
    }
    let badge = Rect::from_center_size(rect.left_center() + vec2(19.0, 0.0), vec2(28.0, 28.0));
    painter.rect_filled(badge, 7, if kind == PageKind::Collection { p.side } else { p.tint });
    let icon = Rect::from_center_size(badge.center(), vec2(13.0, 15.0));
    painter.rect_stroke(icon, 2, Stroke::new(1.2, p.accent), StrokeKind::Inside);
    let offsets: &[f32] = if kind == PageKind::Collection { &[4.0, 7.5, 11.0] } else { &[5.0, 9.0] };
    for y in offsets {
        painter.line_segment([icon.min + vec2(3.0, *y), icon.min + vec2(10.0, *y)], Stroke::new(1.0, p.accent));
    }
    let mut job = egui::text::LayoutJob::simple_singleline(label.to_owned(), FontId::proportional(14.0), p.ink);
    job.wrap.max_width = (rect.width() - 65.0).max(1.0);
    job.wrap.max_rows = 1;
    let galley = painter.layout_job(job);
    painter.galley(rect.left_center() + vec2(43.0, -galley.rect.height() / 2.0), galley, p.ink);
    painter.text(rect.right_center() - vec2(11.0, 0.0), Align2::RIGHT_CENTER, trail, FontId::proportional(15.0), p.accent);
    response.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text(label)
}

pub fn panel(p: Palette) -> Frame {
    Frame::new().fill(p.white).stroke(Stroke::new(1.0, p.line)).corner_radius(12).inner_margin(14)
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
