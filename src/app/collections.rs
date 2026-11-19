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
        let properties = template.map_or(templates::DEFAULT_PROPERTIES, |template| template.properties);
        let title = template.map_or("Título", |template| template.title);
        let add_label = template.map_or_else(|| "+ Adicionar item".to_owned(), |template| format!("+ Adicionar {}", template.item));
        let has_status = properties.iter().any(|property| property.field == Field::Status);
        let has_date = template.is_some_and(|template| template.has(Field::Date));
        let views = [
            (CollectionView::Table, true),
            (CollectionView::Board, has_status),
            (CollectionView::Category, template.is_some_and(|template| template.has(Field::Tag))),
            (CollectionView::Calendar, has_date),
            (CollectionView::Gallery, template.is_some_and(|template| template.view == CollectionView::Gallery)),
        ];
        if let Some(template) = template {
            ui.label(p.label(template.description));
            ui.add_space(6.0);
        }

        let original = self.space.page(id).unwrap().collection_options;
        let mut options = original;
        let add_item = Frame::new().fill(p.white).stroke(egui::Stroke::new(1.0, p.line))
            .corner_radius(9).inner_margin(8).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                for (view, available) in views {
                    if available { ui.selectable_value(&mut options.view, view, view.label()); }
                }
                if has_status {
                    ComboBox::from_id_salt(("status-filter", id))
                        .selected_text(options.status_filter.map_or("Todos os status", Status::label))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut options.status_filter, None, "Todos os status");
                            for status in Status::ALL {
                                ui.selectable_value(&mut options.status_filter, Some(status), status.label());
                            }
                        });
                }
                ui.checkbox(&mut options.sort_name, "Ordenar por nome");
                if ui.available_width() >= 110.0 {
                    right_action(ui, p, template)
                } else {
                    new_item_action(ui, p, template)
                }
            }).inner
        }).inner;
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

        let mut rows = self.space.pages.iter().filter(|page| page.parent == Some(id) && !page.trashed
            && filter.is_none_or(|status| page.status == status))
            .map(|page| page.id).collect::<Vec<_>>();
        if options.sort_name { rows.sort_by_key(|id| search_key(self.space.page(*id).unwrap().title())); }
        ui.add_space(9.0);
        match options.view {
            CollectionView::Table => {
                self.table(ctx, ui, p, id, &rows, title, properties);
                if ui.add(Button::new(p.label(add_label)).frame(false)).clicked() {
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

    pub(super) fn item_properties(&mut self, ctx: &Context, ui: &mut Ui, p: Palette, id: Id, template: &Template) {
        let page = self.space.page_mut(id).unwrap();
        let mut changed = false;
        panel(p).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                for &property in template.properties {
                    let width = ui.available_size_before_wrap().x.min(168.0);
                    ui.push_id(("detail", property.label), |ui| {
                        ui.allocate_ui_with_layout(vec2(width.max(124.0), 0.0), Layout::top_down(Align::Min), |ui| {
                            ui.label(p.label(property.label));
                            changed |= property_edit(ui, p, id, page, property, width.clamp(112.0, 158.0));
                        });
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

    fn table(&mut self, ctx: &Context, ui: &mut Ui, p: Palette, id: Id, rows: &[Id], title: &str, properties: &[Property]) {
        let mut open = None;
        let mut changed = Vec::new();
        let margins = egui::Margin::symmetric(12, 1);
        let cell = |ui: &mut Ui, width: f32, content: &mut dyn FnMut(&mut Ui)| {
            let (rect, _) = ui.allocate_exact_size(vec2(width, 36.0), egui::Sense::hover());
            let mut child = ui.new_child(egui::UiBuilder::new()
                .max_rect(rect).layout(Layout::left_to_right(Align::Center)));
            child.set_clip_rect(child.clip_rect().intersect(rect));
            content(&mut child);
        };
        ScrollArea::horizontal().id_salt(("table-scroll", id)).auto_shrink([false, true]).show(ui, |ui| {
            let fixed = properties.iter().map(|property| match property.field {
                Field::Status => 130.0,
                Field::Date => 118.0,
                Field::Vote => 110.0,
                _ => 145.0,
            }).collect::<Vec<_>>();
            let gaps = fixed.len() as f32 * ui.spacing().item_spacing.x;
            let other_width = fixed.iter().sum::<f32>() + gaps;
            let title_width = (ui.available_width() - other_width - 26.0).max(220.0);
            let content_width = title_width + other_width;
            Frame::new().fill(p.white).stroke(egui::Stroke::new(1.0, p.line)).corner_radius(10)
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    Frame::new().fill(p.side).inner_margin(margins)
                        .corner_radius(egui::CornerRadius { nw: 9, ne: 9, sw: 0, se: 0 })
                        .show(ui, |ui| {
                            ui.set_width(content_width);
                            ui.horizontal(|ui| {
                                cell(ui, title_width, &mut |ui| { ui.label(p.label(title).strong()); });
                                for (property, &width) in properties.iter().zip(&fixed) {
                                    cell(ui, width, &mut |ui| { ui.label(p.label(property.label).strong()); });
                                }
                            });
                        });
                    if rows.is_empty() {
                        Frame::new().inner_margin(margins).show(ui, |ui| {
                            ui.label(p.label("Nenhum item nesta visualização."));
                        });
                    }
                    for (index, row) in rows.iter().enumerate() {
                        let page = self.space.page_mut(*row).unwrap();
                        let mut edited = false;
                        ui.separator();
                        ui.push_id(row, |ui| {
                            Frame::new().fill(if index % 2 == 0 { p.white } else { p.paper })
                                .inner_margin(margins).show(ui, |ui| {
                                ui.set_width(content_width);
                                ui.horizontal(|ui| {
                                    cell(ui, title_width, &mut |ui| {
                                        let width = ui.available_width() - 68.0;
                                        edited |= ui.add(editable(&mut page.title, "Sem título", FontId::proportional(14.0))
                                            .desired_width(width)).changed();
                                        if ui.small_button("Abrir").clicked() { open = Some(*row); }
                                    });
                                    for (&property, &width) in properties.iter().zip(&fixed) {
                                        cell(ui, width, &mut |ui| {
                                            edited |= property_edit(ui, p, *row, page, property, width - 10.0);
                                        });
                                    }
                                });
                            });
                        });
                        if edited { changed.push(*row); }
                    }
                });
        });
        for item in changed {
            self.space.touch(item);
            self.dirty(ctx);
        }
        if let Some(item) = open { self.select(ctx, View::Page(item)); }
    }

    fn board(&mut self, ctx: &Context, ui: &mut Ui, p: Palette, id: Id, rows: &[Id], filter: Option<Status>) {
        let statuses = if filter.is_some() { filter.as_slice() } else { &Status::ALL };
        let mut open = None;
        let mut added = None;
        let mut changed = Vec::new();
        ScrollArea::horizontal().id_salt(("board-scroll", id)).show(ui, |ui| {
            let width = ((ui.available_width() - 54.0) / statuses.len() as f32).max(160.0);
            ui.horizontal_top(|ui| {
                for &status in statuses {
                    let items = rows.iter().copied().filter(|row| self.space.page(*row).unwrap().status == status).collect::<Vec<_>>();
                    panel(p).show(ui, |ui| {
                        ui.set_width(width);
                        ui.set_min_height(104.0);
                        ui.label(p.heading(format!("{}  ·  {}", status.label(), items.len()), 14.0));
                        ui.separator();
                        for child in items {
                            let page = self.space.page_mut(child).unwrap();
                            Frame::new().fill(p.paper).stroke(egui::Stroke::new(1.0, p.line))
                                .corner_radius(8).inner_margin(10).show(ui, |ui| {
                                if ui.link(page.title()).clicked() { open = Some(child); }
                                if !page.tag.trim().is_empty() { ui.label(p.label(&page.tag)); }
                                if !page.date.trim().is_empty() { ui.label(p.label(&page.date)); }
                                if design::status_box(ui, ("board-status", child), &mut page.status) { changed.push(child); }
                            });
                        }
                        if ui.button("+ Adicionar").clicked() { added = Some(status); }
                    });
                }
            });
        });
        for child in changed {
            self.space.touch(child);
            self.dirty(ctx);
        }
        if let Some(child) = open { self.select(ctx, View::Page(child)); }
        if let Some(status) = added {
            self.add_collection_item(ctx, id, status);
        }
    }

    fn category_view(&mut self, ctx: &Context, ui: &mut Ui, p: Palette, id: Id, rows: &[Id], status: Status) {
        let mut groups = std::collections::BTreeMap::<&str, Vec<Id>>::new();
        for &row in rows {
            let tag = self.space.page(row).unwrap().tag.trim();
            groups.entry(tag).or_default().push(row);
        }
        let mut open = None;
        let mut add = None;
        if groups.is_empty() { ui.label(p.label("Nenhum item para agrupar.")); }
        for (tag, rows) in groups {
            panel(p).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(p.heading(if tag.is_empty() { "Sem categoria" } else { tag }, 16.0));
                for row in rows {
                    let page = self.space.page(row).unwrap();
                    if page_row(ui, p, page.title(), page.kind, "›").clicked() { open = Some(row); }
                }
                if ui.small_button("+ Adicionar neste grupo").clicked() { add = Some(tag.to_owned()); }
            });
        }
        if let Some(row) = open { self.select(ctx, View::Page(row)); }
        if let Some(tag) = add {
            let item = self.add_collection_item(ctx, id, status);
            self.space.page_mut(item).unwrap().tag = tag;
        }
    }

    fn gallery_view(&mut self, ctx: &Context, ui: &mut Ui, p: Palette, rows: &[Id]) {
        let mut open = None;
        if rows.is_empty() { ui.label(p.label("Nenhum recurso cadastrado.")); }
        ui.horizontal_wrapped(|ui| {
            for &row in rows {
                let page = self.space.page(row).unwrap();
                let width = (ui.available_width() - 30.0).clamp(120.0, 260.0);
                panel(p).show(ui, |ui| {
                    ui.set_width(width);
                    if ui.add(Button::new(p.heading(page.title(), 16.0)).frame(false).wrap()).clicked() { open = Some(row); }
                    if !page.tag.is_empty() { ui.label(p.label(&page.tag)); }
                    if let Some(description) = page.properties.get("Descrição") { ui.label(p.label(description)); }
                });
            }
        });
        if let Some(row) = open { self.select(ctx, View::Page(row)); }
    }
}

fn new_item_action(ui: &mut Ui, p: Palette, template: Option<&Template>) -> bool {
    let label = template.map_or_else(|| "+ Novo item".to_owned(), |template| format!("+ {}", template.title));
    action(ui, p, &label, true).clicked()
}

fn right_action(ui: &mut Ui, p: Palette, template: Option<&Template>) -> bool {
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| new_item_action(ui, p, template)).inner
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
