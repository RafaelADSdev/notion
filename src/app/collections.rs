use super::*;

impl Margem {
    fn collection_template(&self, id: Id) -> Option<&'static Template> {
        templates::find(&self.space.page(id)?.collection_template)
    }

    pub(super) fn template_for_item(&self, id: Id) -> Option<&'static Template> {
        self.collection_template(self.space.page(id)?.parent?)
    }

    pub(super) fn collection(&mut self, ctx: &Context, ui: &mut Ui, p: Palette, id: Id) {
        let template = self.collection_template(id);
        let properties = template.map_or(templates::DEFAULT_PROPERTIES, |template| {
            template.properties
        });
        let title = template.map_or("Título", |template| template.title);
        let add_label = template.map_or_else(
            || "+ Adicionar item".to_owned(),
            |template| format!("+ Adicionar {}", template.item),
        );
        let has_status = properties
            .iter()
            .any(|property| property.field == Field::Status);
        let has_date = template.is_some_and(|template| template.has(Field::Date));
        let views = [
            (CollectionView::Table, true),
            (CollectionView::Board, has_status),
            (
                CollectionView::Category,
                template.is_some_and(|template| template.has(Field::Tag)),
            ),
            (CollectionView::Calendar, has_date),
            (
                CollectionView::Gallery,
                template.is_some_and(|template| template.view == CollectionView::Gallery),
            ),
        ];
        if let Some(template) = template {
            ui.label(p.label(template.description).size(14.0));
            ui.add_space(12.0);
        }

        let original = self.space.page(id).unwrap().collection_options;
        let mut options = original;
        ui.horizontal_wrapped(|ui| {
            for (view, available) in views {
                if available && view_tab(ui, p, view, options.view == view).clicked() {
                    options.view = view;
                }
            }
        });
        ui.separator();
        let add_item = ui
            .horizontal_wrapped(|ui| {
                if has_status {
                    ComboBox::from_id_salt(("status-filter", id))
                        .selected_text(
                            options
                                .status_filter
                                .map_or("Todos os status", Status::label),
                        )
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut options.status_filter,
                                None,
                                "Todos os status",
                            );
                            for status in Status::ALL {
                                ui.selectable_value(
                                    &mut options.status_filter,
                                    Some(status),
                                    status.label(),
                                );
                            }
                        });
                }
                ui.checkbox(&mut options.sort_name, "Ordenar por nome");
                if ui.available_width() >= 155.0 {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        new_item_action(ui, p, template)
                    })
                    .inner
                } else {
                    new_item_action(ui, p, template)
                }
            })
            .inner;
        if options != original {
            self.space.page_mut(id).unwrap().collection_options = options;
            self.dirty(ctx);
        }
        let filter = options.status_filter.filter(|_| has_status);
        let status = filter.unwrap_or_default();
        if add_item {
            self.add_collection_item(ctx, id, status);
            return;
        }

        let mut rows = self
            .space
            .pages
            .iter()
            .filter(|page| {
                page.parent == Some(id)
                    && !page.trashed
                    && filter.is_none_or(|status| page.status == status)
            })
            .map(|page| page.id)
            .collect::<Vec<_>>();
        if options.sort_name {
            rows.sort_by_key(|id| search_key(self.space.page(*id).unwrap().title()));
        }
        ui.add_space(5.0);
        match options.view {
            CollectionView::Table => {
                self.table(ctx, ui, p, id, &rows, title, properties);
                if ui
                    .add(Button::new(p.label(add_label).color(p.accent)).frame(false))
                    .clicked()
                {
                    self.add_collection_item(ctx, id, status);
                }
            }
            CollectionView::Board => self.board(ctx, ui, p, id, &rows, filter),
            CollectionView::Category => self.category_view(ctx, ui, p, id, &rows, status),
            CollectionView::Calendar => self.calendar_view(ctx, ui, p, id, &rows, status),
            CollectionView::Gallery => self.gallery_view(ctx, ui, p, &rows),
        }
    }

    pub(super) fn add_collection_item(&mut self, ctx: &Context, collection: Id, status: Status) -> Id {
        let item = self.create(ctx, Some(collection), PageKind::Document, true);
        self.space.page_mut(item).unwrap().status = status;
        if let Some(template) = self.collection_template(collection) {
            template.prepare_item(&mut self.space, item);
        }
        item
    }

    pub(super) fn item_properties(
        &mut self,
        ctx: &Context,
        ui: &mut Ui,
        p: Palette,
        id: Id,
        template: &Template,
    ) {
        let page = self.space.page_mut(id).unwrap();
        let mut changed = false;
        Frame::new()
            .fill(p.side)
            .corner_radius(6)
            .inner_margin(egui::Margin::symmetric(12, 9))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal_wrapped(|ui| {
                    for &property in template.properties {
                        ui.push_id(("detail", property.label), |ui| {
                            ui.allocate_ui_with_layout(
                                vec2(144.0, 0.0),
                                Layout::top_down(Align::Min),
                                |ui| {
                                    ui.label(p.label(property.label).size(12.0).strong());
                                    changed |= property_edit(ui, p, id, page, property, 133.0);
                                },
                            );
                        });
                    }
                });
            });
        if changed {
            self.space.touch(id);
            self.dirty(ctx);
        }
        ui.add_space(12.0);
    }

    fn table(
        &mut self,
        ctx: &Context,
        ui: &mut Ui,
        p: Palette,
        id: Id,
        rows: &[Id],
        title: &str,
        properties: &[Property],
    ) {
        let mut open = None;
        let mut changed = Vec::new();
        ScrollArea::horizontal()
            .id_salt(("table-scroll", id))
            .auto_shrink([false, true])
            .show(ui, |ui| {
                let widths: Vec<_> = properties
                    .iter()
                    .map(|property| match property.field {
                        Field::Status => 146.0,
                        Field::Date => 118.0,
                        Field::Vote => 112.0,
                        _ => 140.0,
                    })
                    .collect();
                let properties_width = widths.iter().sum::<f32>();
                let title_width = (ui.available_width() - properties_width - 2.0).max(220.0);
                let content_width = title_width + properties_width;
                let row_height = if self.space.settings.compact {
                    34.0
                } else {
                    42.0
                };
                Frame::new()
                    .stroke(egui::Stroke::new(1.0_f32, p.line))
                    .show(ui, |ui| {
                        ui.set_width(content_width);
                        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                        Frame::new().fill(p.side).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                table_cell(ui, p, title_width, 33.0, false, |ui| {
                                    ui.label(p.label(title).size(12.0).strong());
                                });
                                for (property, &width) in properties.iter().zip(&widths) {
                                    table_cell(ui, p, width, 33.0, true, |ui| {
                                        ui.label(p.label(property.label).size(12.0).strong());
                                    });
                                }
                            });
                        });
                        if rows.is_empty() {
                            table_rule(ui, p, content_width);
                            Frame::new().inner_margin(12).show(ui, |ui| {
                                ui.label(p.label("Nenhum item nesta visualização."));
                            });
                        }
                        for &row in rows {
                            let page = self.space.page_mut(row).unwrap();
                            let mut edited = false;
                            table_rule(ui, p, content_width);
                            ui.push_id(row, |ui| {
                                let row_rect = egui::Rect::from_min_size(
                                    ui.next_widget_position(),
                                    vec2(content_width, row_height),
                                );
                                let hovered = ui.rect_contains_pointer(row_rect);
                                ui.painter().rect_filled(
                                    row_rect,
                                    0,
                                    if hovered { p.side } else { p.white },
                                );
                                ui.horizontal(|ui| {
                                    table_cell(ui, p, title_width, row_height, false, |ui| {
                                        let width = ui.available_width() - 56.0;
                                        edited |= ui
                                            .add(
                                                editable(
                                                    &mut page.title,
                                                    "Sem título",
                                                    FontId::proportional(14.0),
                                                )
                                                .desired_width(width),
                                            )
                                            .changed();
                                        if ui
                                            .add(
                                                Button::new(p.label("Abrir").color(p.accent))
                                                    .frame(false),
                                            )
                                            .clicked()
                                        {
                                            open = Some(row);
                                        }
                                    });
                                    for (&property, &width) in properties.iter().zip(&widths) {
                                        table_cell(ui, p, width, row_height, true, |ui| {
                                            edited |= property_edit(
                                                ui,
                                                p,
                                                row,
                                                page,
                                                property,
                                                ui.available_width(),
                                            );
                                        });
                                    }
                                });
                            });
                            if edited {
                                changed.push(row);
                            }
                        }
                    });
            });
        for item in changed {
            self.space.touch(item);
            self.dirty(ctx);
        }
        if let Some(item) = open {
            self.select(ctx, View::Page(item));
        }
    }

    fn board(
        &mut self,
        ctx: &Context,
        ui: &mut Ui,
        p: Palette,
        id: Id,
        rows: &[Id],
        filter: Option<Status>,
    ) {
        let statuses = if filter.is_some() {
            filter.as_slice()
        } else {
            &Status::ALL
        };
        let mut open = None;
        let mut added = None;
        let mut changed = Vec::new();
        ScrollArea::horizontal()
            .id_salt(("board-scroll", id))
            .show(ui, |ui| {
                let count = statuses.len() as f32;
                let width = ((ui.available_width() - (count - 1.0) * ui.spacing().item_spacing.x)
                    / count
                    - 16.0)
                    .clamp(168.0, 350.0);
                ui.horizontal_top(|ui| {
                    for &status in statuses {
                        let items = rows
                            .iter()
                            .copied()
                            .filter(|row| self.space.page(*row).unwrap().status == status)
                            .collect::<Vec<_>>();
                        Frame::new()
                            .fill(p.side)
                            .corner_radius(8)
                            .inner_margin(8)
                            .show(ui, |ui| {
                                ui.set_width(width);
                                ui.set_min_height(100.0);
                                ui.horizontal(|ui| {
                                    let (marker, _) = ui.allocate_exact_size(
                                        vec2(12.0, 24.0),
                                        egui::Sense::hover(),
                                    );
                                    let color = match status {
                                        Status::Planned => p.muted,
                                        Status::Active => egui::Color32::from_rgb(174, 122, 54),
                                        Status::Done => p.accent,
                                    };
                                    ui.painter().circle_filled(marker.center(), 3.0, color);
                                    ui.label(p.heading(
                                        format!("{}  ·  {}", status.label(), items.len()),
                                        13.0,
                                    ));
                                });
                                for child in items {
                                    let page = self.space.page_mut(child).unwrap();
                                    Frame::new()
                                        .fill(p.white)
                                        .stroke(egui::Stroke::new(1.0_f32, p.line))
                                        .corner_radius(6)
                                        .inner_margin(8)
                                        .show(ui, |ui| {
                                            ui.set_width(ui.available_width());
                                            if page_row(ui, p, page.title(), page.kind, "›")
                                                .clicked()
                                            {
                                                open = Some(child);
                                            }
                                            ui.horizontal_wrapped(|ui| {
                                                if !page.tag.trim().is_empty() {
                                                    ui.label(p.label(&page.tag).size(12.0));
                                                }
                                                if !page.date.trim().is_empty() {
                                                    ui.label(p.label(&page.date).size(12.0));
                                                }
                                            });
                                            if design::status_box(
                                                ui,
                                                ("board-status", child),
                                                &mut page.status,
                                            ) {
                                                changed.push(child);
                                            }
                                        });
                                }
                                if ui
                                    .add(
                                        Button::new(p.label("+ Adicionar").color(p.accent))
                                            .frame(false),
                                    )
                                    .clicked()
                                {
                                    added = Some(status);
                                }
                            });
                    }
                });
            });
        for child in changed {
            self.space.touch(child);
            self.dirty(ctx);
        }
        if let Some(child) = open {
            self.select(ctx, View::Page(child));
        }
        if let Some(status) = added {
            self.add_collection_item(ctx, id, status);
        }
    }

    fn category_view(
        &mut self,
        ctx: &Context,
        ui: &mut Ui,
        p: Palette,
        id: Id,
        rows: &[Id],
        status: Status,
    ) {
        let mut groups = std::collections::BTreeMap::<&str, Vec<Id>>::new();
        for &row in rows {
            let tag = self.space.page(row).unwrap().tag.trim();
            groups.entry(tag).or_default().push(row);
        }
        let mut open = None;
        let mut add = None;
        if groups.is_empty() {
            ui.label(p.label("Nenhum item para agrupar."));
        }
        for (tag, rows) in groups {
            ui.horizontal(|ui| {
                ui.label(p.heading(if tag.is_empty() { "Sem categoria" } else { tag }, 15.0));
                ui.label(p.label(rows.len()));
            });
            ui.separator();
            Frame::new()
                .inner_margin(egui::Margin::symmetric(4, 2))
                .show(ui, |ui| {
                    for row in rows {
                        let page = self.space.page(row).unwrap();
                        if page_row(ui, p, page.title(), page.kind, "›").clicked() {
                            open = Some(row);
                        }
                    }
                    if ui
                        .add(
                            Button::new(p.label("+ Adicionar neste grupo").color(p.accent))
                                .frame(false),
                        )
                        .clicked()
                    {
                        add = Some(tag.to_owned());
                    }
                });
            ui.add_space(10.0);
        }
        if let Some(row) = open {
            self.select(ctx, View::Page(row));
        }
        if let Some(tag) = add {
            let item = self.add_collection_item(ctx, id, status);
            self.space.page_mut(item).unwrap().tag = tag;
        }
    }

    fn gallery_view(&mut self, ctx: &Context, ui: &mut Ui, p: Palette, rows: &[Id]) {
        let mut open = None;
        if rows.is_empty() {
            ui.label(p.label("Nenhum recurso cadastrado."));
        }
        let columns = ((ui.available_width() + 8.0) / 240.0)
            .floor()
            .clamp(1.0, 3.0) as usize;
        for chunk in rows.chunks(columns) {
            ui.columns(columns, |uis| {
                for (&row, ui) in chunk.iter().zip(uis) {
                    let page = self.space.page(row).unwrap();
                    Frame::new()
                        .fill(p.white)
                        .stroke(egui::Stroke::new(1.0_f32, p.line))
                        .corner_radius(7)
                        .inner_margin(10)
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.set_min_height(84.0);
                            if page_row(ui, p, page.title(), page.kind, "›").clicked() {
                                open = Some(row);
                            }
                            if !page.tag.is_empty() {
                                ui.label(p.label(&page.tag).color(p.accent).size(12.0));
                            }
                            if let Some(description) = page.properties.get("Descrição") {
                                ui.add(egui::Label::new(p.label(description)).truncate())
                                    .on_hover_text(description);
                            }
                        });
                }
            });
        }
        if let Some(row) = open {
            self.select(ctx, View::Page(row));
        }
    }
}

fn new_item_action(ui: &mut Ui, p: Palette, template: Option<&Template>) -> bool {
    let label = template.map_or_else(
        || "+ Novo item".to_owned(),
        |template| format!("+ {}", template.title),
    );
    action(ui, p, &label, true).clicked()
}

fn view_tab(ui: &mut Ui, p: Palette, view: CollectionView, selected: bool) -> egui::Response {
    let text =
        RichText::new(view.label())
            .size(13.0)
            .color(if selected { p.accent } else { p.muted });
    let text = if selected { text.strong() } else { text };
    let response = ui
        .scope(|ui| {
            let widgets = &mut ui.visuals_mut().widgets;
            widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
            widgets.inactive.bg_stroke = egui::Stroke::NONE;
            widgets.hovered.weak_bg_fill = p.tint;
            widgets.hovered.bg_stroke = egui::Stroke::NONE;
            widgets.active.weak_bg_fill = p.tint;
            widgets.active.bg_stroke = egui::Stroke::NONE;
            ui.add(Button::new(text).min_size(vec2(0.0, 29.0)))
        })
        .inner;
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::SelectableLabel,
            ui.is_enabled(),
            selected,
            view.label(),
        )
    });
    if selected {
        let rect = response.rect.shrink2(vec2(8.0, 0.0));
        ui.painter().line_segment(
            [rect.left_bottom(), rect.right_bottom()],
            egui::Stroke::new(2.0_f32, p.accent),
        );
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            response.rect,
            5,
            egui::Stroke::new(1.0_f32, p.accent),
            egui::StrokeKind::Inside,
        );
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn table_rule(ui: &mut Ui, p: Palette, width: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(width, 1.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0, p.line);
}

fn table_cell(
    ui: &mut Ui,
    p: Palette,
    width: f32,
    height: f32,
    divider: bool,
    content: impl FnOnce(&mut Ui),
) {
    let (rect, _) = ui.allocate_exact_size(vec2(width, height), egui::Sense::hover());
    if divider {
        ui.painter().line_segment(
            [rect.left_top(), rect.left_bottom()],
            egui::Stroke::new(1.0_f32, p.line),
        );
    }
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink2(vec2(8.0, 0.0)))
            .layout(Layout::left_to_right(Align::Center)),
    );
    child.set_clip_rect(child.clip_rect().intersect(rect));
    child.spacing_mut().item_spacing.x = 6.0;
    content(&mut child);
}

fn property_edit(ui: &mut Ui, p: Palette, id: Id, page: &mut Page, property: Property, width: f32) -> bool {
    let label = property.label;
    let edit_text = |ui: &mut Ui, value: &mut String, hint: &str| {
        ui.add(editable(value, hint, FontId::proportional(13.0))
            .id_salt(label).desired_width(width)).changed()
    };
    match property.field {
        Field::Status => design::status_box(ui, ("status", id), &mut page.status),
        Field::Tag => edit_text(ui, &mut page.tag, label),
        Field::Date => edit_text(ui, &mut page.date, "AAAA-MM-DD"),
        Field::Text => {
            let mut value = page.properties.get(label).cloned().unwrap_or_default();
            let changed = edit_text(ui, &mut value, label);
            if changed {
                if value.trim().is_empty() { page.properties.remove(label); }
                else { page.properties.insert(label.to_owned(), value); }
            }
            changed
        }
        Field::Choice(choices) => {
            let previous = page.properties.get(label).map_or("", String::as_str);
            let mut chosen = previous;
            ComboBox::from_id_salt(label).width(width - 10.0)
                .selected_text(if chosen.is_empty() { "Selecionar" } else { chosen })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut chosen, "", "Sem valor");
                    for &choice in choices { ui.selectable_value(&mut chosen, choice, choice); }
                });
            if chosen == previous { false } else {
                if chosen.is_empty() { page.properties.remove(label); }
                else {
                    let chosen = chosen.to_owned();
                    page.properties.insert(label.to_owned(), chosen);
                }
                true
            }
        }
        Field::Vote => {
            if action(ui, p, if page.voted { "Apoiado · 1" } else { "Apoiar · 0" }, page.voted).clicked() {
                page.voted = !page.voted;
                true
            } else { false }
        }
    }
}
