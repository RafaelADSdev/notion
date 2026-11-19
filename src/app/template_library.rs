use super::*;
use std::collections::BTreeMap;

impl Margem {
    pub(super) fn template_library(&mut self, ctx: &Context, ui: &mut Ui, p: Palette) {
        ui.label(p.display("Modelos", 38.0));
        ui.label(p.label("Comece com uma estrutura pronta para sua coleção."));
        ui.add_space(12.0);
        let mut blank = false;
        ui.horizontal_wrapped(|ui| {
            let response = ui.add(
                TextEdit::singleline(&mut self.template_query)
                    .hint_text("Buscar modelos")
                    .desired_width((ui.available_width() - 170.0).clamp(120.0, 300.0))
                    .margin(egui::Margin::symmetric(10, 7)),
            );
            if std::mem::take(&mut self.focus_templates) {
                response.request_focus();
            }
            blank = action(ui, p, "+ Coleção vazia", false).clicked();
        });
        if blank {
            self.create(ctx, None, PageKind::Collection, true);
            return;
        }
        ui.add_space(16.0);

        let words = search_key(self.template_query.trim());
        let mut groups: BTreeMap<&str, Vec<&Template>> = BTreeMap::new();
        for template in templates::CATALOG {
            let text = search_key(&format!(
                "{} {} {}",
                template.name, template.group, template.description
            ));
            if text.contains(&words) {
                groups.entry(template.group).or_default().push(template);
            }
        }
        if groups.is_empty() {
            ui.separator();
            ui.add_space(12.0);
            ui.label(p.heading("Nenhum modelo encontrado", 18.0));
            ui.label(p.label("Busque pelo nome de um modelo ou de uma categoria."));
            if action(ui, p, "Limpar busca", false).clicked() {
                self.template_query.clear();
                self.focus_templates = true;
            }
            return;
        }

        let mut selected = None;
        let columns = if ui.available_width() >= 648.0 { 2 } else { 1 };
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = vec2(12.0, 8.0);
            let width = (ui.available_width() - (columns - 1) as f32 * ui.spacing().item_spacing.x)
                / columns as f32;
            for (group, templates) in groups {
                ui.label(p.heading(group, 19.0));
                ui.separator();
                for row in templates.chunks(columns) {
                    ui.horizontal_top(|ui| {
                        for &template in row {
                            ui.push_id(template.id, |ui| {
                                ui.allocate_ui_with_layout(
                                    vec2(width, 0.0),
                                    Layout::top_down(Align::Min),
                                    |ui| {
                                        ui.set_width(width);
                                        if template_card(ui, p, template) {
                                            selected = Some(template);
                                        }
                                    },
                                );
                            });
                        }
                    });
                }
                ui.add_space(12.0);
            }
        });
        if let Some(template) = selected {
            let id = template.create(&mut self.space);
            self.template_query.clear();
            self.select(ctx, View::Page(id));
        }
    }
}

fn template_card(ui: &mut Ui, p: Palette, template: &Template) -> bool {
    Frame::new()
        .fill(p.white)
        .stroke(egui::Stroke::new(1.0_f32, p.line))
        .corner_radius(8)
        .inner_margin(12)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing = vec2(12.0, 5.0);
            ui.horizontal_top(|ui| {
                view_preview(ui, p, template.view);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    ui.add(egui::Label::new(p.heading(template.name, 16.0)).wrap());
                    ui.add(egui::Label::new(p.label(template.description)).wrap());
                });
            });
            ui.separator();
            let fields = std::iter::once(template.title)
                .chain(
                    template
                        .properties
                        .iter()
                        .take(2)
                        .map(|property| property.label),
                )
                .collect::<Vec<_>>()
                .join(" · ");
            let summary = format!("{}  ·  {fields}", template.view.label());
            let mut apply = false;
            if ui.available_width() >= 265.0 {
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        vec2(ui.available_width() - 120.0, 30.0),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.add(egui::Label::new(p.label(&summary)).truncate())
                                .on_hover_text(&summary);
                        },
                    );
                    right(ui, |ui| {
                        apply = action(ui, p, "Usar modelo", false).clicked()
                    });
                });
            } else {
                ui.add(egui::Label::new(p.label(&summary)).truncate())
                    .on_hover_text(summary);
                apply = action(ui, p, "Usar modelo", false).clicked();
            }
            apply
        })
        .inner
}

fn view_preview(ui: &mut Ui, p: Palette, view: CollectionView) {
    let (tile, _) = ui.allocate_exact_size(vec2(48.0, 48.0), egui::Sense::hover());
    let paint = ui.painter();
    paint.rect_filled(tile, 7, p.tint);
    let area = tile.shrink(10.0);
    let accent = egui::Stroke::new(1.2_f32, p.accent);
    let cell = |origin: Vec2, size: Vec2, outline| {
        let rect = egui::Rect::from_min_size(area.min + origin, size);
        if outline {
            paint.rect_stroke(rect, 2, accent, egui::StrokeKind::Inside);
        } else {
            paint.rect_filled(rect, 1, p.accent);
        }
    };
    match view {
        CollectionView::Table => {
            cell(vec2(0.0, 3.0), vec2(28.0, 22.0), true);
            for y in [10.0, 17.0] {
                paint.line_segment([area.min + vec2(1.0, y), area.min + vec2(27.0, y)], accent);
            }
            paint.line_segment(
                [area.min + vec2(17.0, 3.0), area.min + vec2(17.0, 25.0)],
                accent,
            );
        }
        CollectionView::Board | CollectionView::Category => {
            for (x, height) in [(0.0, 18.0), (10.0, 14.0), (20.0, 10.0)] {
                cell(vec2(x, 3.0), vec2(8.0, 2.0), false);
                cell(vec2(x, 8.0), vec2(8.0, height), true);
            }
        }
        CollectionView::Calendar => {
            cell(vec2(0.0, 3.0), vec2(28.0, 22.0), true);
            paint.line_segment(
                [area.min + vec2(1.0, 10.0), area.min + vec2(27.0, 10.0)],
                accent,
            );
            for x in [5.0, 12.0, 19.0] {
                for y in [14.0, 19.0] {
                    cell(vec2(x, y), vec2(3.0, 2.0), false);
                }
            }
        }
        CollectionView::Gallery => {
            for origin in [
                vec2(1.0, 1.0),
                vec2(16.0, 1.0),
                vec2(1.0, 16.0),
                vec2(16.0, 16.0),
            ] {
                cell(origin, vec2(11.0, 11.0), true);
                cell(origin + vec2(3.0, 7.0), vec2(5.0, 1.0), false);
            }
        }
    }
}
