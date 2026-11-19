use super::*;

impl Margem {
    pub(super) fn template_library(&mut self, ctx: &Context, ui: &mut Ui, p: Palette) {
        ui.label(p.heading("Modelos", 40.0));
        ui.label(p.label("Escolha uma estrutura pronta para organizar seu trabalho."));
        ui.add_space(10.0);
        let blank = ui.horizontal_wrapped(|ui| {
            let search = TextEdit::singleline(&mut self.template_query)
                .hint_text("Buscar modelos")
                .desired_width((ui.available_width() - 155.0).clamp(166.0, 340.0));
            let response = ui.add(search);
            if std::mem::take(&mut self.focus_templates) { response.request_focus(); }
            action(ui, p, "+ Coleção vazia", false).clicked()
        }).inner;
        if blank {
            self.create(ctx, None, PageKind::Collection, true);
            return;
        }
        ui.add_space(10.0);

        let words = search_key(self.template_query.trim());
        let matches = templates::CATALOG.iter().filter(|template| {
            search_key(&format!("{} {} {}", template.name, template.group, template.description))
                .contains(&words)
        }).collect::<Vec<_>>();
        if matches.is_empty() {
            ui.label(p.label("Nenhum modelo encontrado. Tente outro termo."));
            return;
        }

        let mut selected = None;
        let columns = if ui.available_width() >= 560.0 { 2 } else { 1 };
        let width = (ui.available_width() - (columns - 1) as f32 * ui.spacing().item_spacing.x)
            / columns as f32;
        for row in matches.chunks(columns) {
            ui.horizontal_top(|ui| {
                for &template in row {
                    ui.push_id(template.id, |ui| {
                        ui.allocate_ui_with_layout(vec2(width, 0.0), Layout::top_down(Align::Min), |ui| {
                            ui.set_width(width);
                            if template_card(ui, p, template) { selected = Some(template); }
                        });
                    });
                }
            });
        }
        if let Some(template) = selected {
            let id = template.create(&mut self.space);
            self.template_query.clear();
            self.select(ctx, View::Page(id));
        }
    }
}

fn template_card(ui: &mut Ui, p: Palette, template: &Template) -> bool {
    Frame::new().fill(p.side).stroke(egui::Stroke::new(1.0, p.line))
        .corner_radius(12).inner_margin(13).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(p.label(template.group).color(p.accent).strong());
            ui.label(p.heading(template.name, 17.0));
            ui.allocate_ui_with_layout(vec2(ui.available_width(), 34.0), Layout::top_down(Align::Min), |ui| {
                ui.label(p.label(template.description));
            });
            Frame::new().fill(p.white).stroke(egui::Stroke::new(1.0, p.line))
                .corner_radius(7).inner_margin(8).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(p.heading(template.view.label(), 12.0));
                    ui.separator();
                    let preview = std::iter::once(template.title)
                        .chain(template.properties.iter().take(2).map(|property| property.label))
                        .collect::<Vec<_>>().join("  ·  ");
                    ui.add(egui::Label::new(p.label(&preview)).truncate()).on_hover_text(preview);
                });
            action(ui, p, "Usar modelo", true).clicked()
        }).inner
}
