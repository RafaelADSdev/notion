mod calendar;
mod collections;
mod settings;
mod template_library;
#[cfg(debug_assertions)]
mod tests;
#[cfg(debug_assertions)]
pub use tests::verify_mechanics;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use eframe::egui::{
    self, vec2, Align, Align2, Button, CentralPanel, ComboBox, Context, FontId, Frame,
    Layout, RichText, ScrollArea, SidePanel, TextEdit, TopBottomPanel, Ui, Vec2, ViewportCommand,
};

use crate::model::{search_key, CollectionView, Id, Kind, Page, PageKind, Status, Theme, Workspace};
use crate::templates::{self, Field, Property, Template};
use calendar::Calendar;
use settings::SettingsTab;

use crate::ui::{self as design, action, editable, nav_icon, page_row, panel, right, section, Icon, Palette};

#[derive(Clone, Copy, PartialEq, Eq)]
enum View {
    Home,
    Templates,
    Page(Id),
    Trash,
}

#[derive(Clone, Copy)]
enum Transfer {
    Import,
    Export(Id),
}

enum BlockChange {
    Add(Kind),
    Convert(usize, Kind),
    Duplicate(usize),
    Move(usize, usize),
    Remove(usize),
}

pub struct Margem {
    space: Workspace,
    path: PathBuf,
    storage_blocked: bool,
    dirty_since: Option<Instant>,
    notice: Option<(String, Instant)>,
    view: View,
    history: Vec<View>,
    history_at: usize,
    tabs: Vec<Id>,
    collapsed: HashSet<Id>,
    search: bool,
    query: String,
    focus_search: bool,
    template_query: String,
    focus_templates: bool,
    calendar: Calendar,
    focus_title: bool,
    focus_block: Option<Id>,
    move_page: Option<Id>,
    transfer: Option<Transfer>,
    transfer_path: String,
    settings_tab: Option<SettingsTab>,
    screenshot: Option<PathBuf>,
    frames: u32,
    shot_requested: bool,
    capture_stage: usize,
    applied_appearance: Option<(bool, bool, bool, f32)>,
}

impl Margem {
    pub fn new(ctx: &Context, screenshot: Option<PathBuf>) -> Self {
        Self::with_storage(ctx, screenshot, workspace_path)
    }

    pub(crate) fn with_storage(
        ctx: &Context,
        screenshot: Option<PathBuf>,
        storage: impl FnOnce() -> Result<PathBuf, String>,
    ) -> Self {
        design::install_fonts(ctx);
        let (path, read) = if screenshot.is_some() {
            (PathBuf::new(), Ok(None))
        } else {
            match storage() {
                Ok(path) => {
                    let read = load(&path);
                    (path, read)
                }
                Err(error) => (PathBuf::new(), Err(error)),
            }
        };
        let storage_blocked = read.is_err();
        let notice = read.as_ref().err().map(|error| {
            (format!("Não foi possível abrir as páginas salvas: {error}"), Instant::now())
        });
        let space = read.ok().flatten().unwrap_or_else(Workspace::seeded);
        let initial = space.last_page.filter(|id| space.settings.restore_last && space.is_live(*id));
        let view = initial.map_or(View::Home, View::Page);
        Self {
            space, path, storage_blocked, dirty_since: None, notice, view,
            history: vec![view], history_at: 0, tabs: initial.into_iter().collect(),
            collapsed: HashSet::new(), search: false, query: String::new(), focus_search: false,
            template_query: String::new(), focus_templates: false, calendar: Calendar::default(),
            focus_title: false, focus_block: None, move_page: None, transfer: None,
            transfer_path: String::new(), settings_tab: None, screenshot, frames: 0, shot_requested: false, capture_stage: 0,
            applied_appearance: None,
        }
    }

    fn dirty(&mut self, ctx: &Context) {
        self.dirty_since = Some(Instant::now());
        ctx.request_repaint_after(Duration::from_millis(550));
    }

    fn notify(&mut self, ctx: &Context, message: impl Into<String>) {
        self.notice = Some((message.into(), Instant::now()));
        ctx.request_repaint_after(Duration::from_secs(5));
    }

    fn save(&mut self) {
        if self.screenshot.is_some() || self.storage_blocked || self.dirty_since.is_none() {
            return;
        }
        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
            let data = serde_json::to_vec_pretty(&self.space)?;
            if let Some(folder) = self.path.parent() {
                std::fs::create_dir_all(folder)?;
            }
            let temporary = self.path.with_extension("json.tmp");
            std::fs::write(&temporary, data)?;
            std::fs::rename(temporary, &self.path)?;
            Ok(())
        })();
        match result {
            Ok(()) => self.dirty_since = None,
            Err(error) => {
                self.dirty_since = Some(Instant::now());
                self.notice = Some((format!("Erro ao salvar: {error}"), Instant::now()));
            }
        }
    }

    fn select(&mut self, ctx: &Context, target: View) {
        if cfg!(target_os = "android") && self.space.settings.show_sidebar {
            self.space.settings.show_sidebar = false;
            self.dirty(ctx);
        }
        if self.view == target { return; }
        self.history.truncate(self.history_at + 1);
        self.history.push(target);
        self.history_at = self.history.len() - 1;
        self.show(ctx, target);
    }

    fn show(&mut self, ctx: &Context, target: View) {
        self.view = target;
        self.focus_title = false;
        self.focus_block = None;
        if let View::Page(id) = target {
            if !self.tabs.contains(&id) { self.tabs.push(id); }
            self.space.touch(id);
            self.space.last_page = Some(id);
        } else if target == View::Home {
            self.space.last_page = None;
        } else if target == View::Templates {
            self.focus_templates = true;
        }
        self.dirty(ctx);
    }

    fn travel(&mut self, ctx: &Context, reverse: bool) {
        loop {
            if reverse && self.history_at > 0 { self.history_at -= 1; }
            else if !reverse && self.history_at + 1 < self.history.len() { self.history_at += 1; }
            else { return; }
            let target = self.history[self.history_at];
            if !matches!(target, View::Page(id) if !self.space.is_live(id)) {
                self.show(ctx, target);
                return;
            }
        }
    }

    fn create(&mut self, ctx: &Context, parent: Option<Id>, kind: PageKind, open: bool) -> Id {
        let title = if kind == PageKind::Collection { "Nova coleção" } else { "" };
        let id = self.space.create(parent, title, kind);
        if let Some(parent) = parent { self.collapsed.remove(&parent); }
        if open {
            self.select(ctx, View::Page(id));
            self.focus_title = true;
        }
        self.dirty(ctx);
        id
    }

    fn favorite(&mut self, ctx: &Context, id: Id) {
        if let Some(page) = self.space.page_mut(id) {
            page.favorite = !page.favorite;
            self.dirty(ctx);
        }
    }

    fn trash(&mut self, ctx: &Context, id: Id) {
        if let Some(page) = self.space.page_mut(id) { page.trashed = true; }
        self.tabs.retain(|tab| self.space.is_live(*tab));
        if matches!(self.view, View::Page(page) if !self.space.is_live(page)) {
            self.select(ctx, View::Home);
        }
        self.dirty(ctx);
        self.notify(ctx, "Página movida para a lixeira.");
    }

    fn start_search(&mut self) {
        self.search = true;
        self.focus_search = true;
        self.query.clear();
    }

    fn start_transfer(&mut self, transfer: Transfer) {
        self.transfer_path.clear();
        self.transfer = Some(transfer);
    }

    fn close_tab(&mut self, ctx: &Context, tab: Id) {
        let Some(index) = self.tabs.iter().position(|id| *id == tab) else { return };
        self.tabs.remove(index);
        if self.view == View::Page(tab) {
            let replacement = self.tabs.get(index).or_else(|| self.tabs.last()).copied();
            self.select(ctx, replacement.map_or(View::Home, View::Page));
        }
    }

    fn shortcuts(&mut self, ctx: &Context) {
        if self.settings_tab.is_some() { return; }
        if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.search = false;
            self.move_page = None;
            self.transfer = None;
        }
        let pressed = |key| ctx.input_mut(|input| input.consume_key(egui::Modifiers::COMMAND, key));
        if pressed(egui::Key::Comma) {
            self.settings_tab = Some(SettingsTab::Appearance);
            return;
        }
        if pressed(egui::Key::K) || pressed(egui::Key::P) { self.start_search(); }
        if pressed(egui::Key::N) { self.create(ctx, None, PageKind::Document, true); }
        if pressed(egui::Key::W) {
            if let View::Page(id) = self.view { self.close_tab(ctx, id); }
        }
        if pressed(egui::Key::E) {
            self.space.settings.show_sidebar = !self.space.settings.show_sidebar;
            self.dirty(ctx);
        }
    }

    fn toolbar(&mut self, ctx: &Context, p: Palette) {
        TopBottomPanel::top("toolbar").frame(Frame::new().fill(p.paper).inner_margin(egui::Margin::symmetric(14, 6)))
            .show(ctx, |ui| ui.horizontal(|ui| {
                if !self.space.settings.show_sidebar {
                    if action(ui, p, "Menu", false).clicked() {
                        self.space.settings.show_sidebar = true;
                        self.dirty(ctx);
                    }
                    if action(ui, p, "Configurações", false).clicked() {
                        self.settings_tab = Some(SettingsTab::Appearance);
                    }
                }
                if ui.add_enabled(self.history_at > 0, Button::new("←").frame(false)).on_hover_text("Voltar").clicked() {
                    self.travel(ctx, true);
                }
                if ui.add_enabled(self.history_at + 1 < self.history.len(), Button::new("→").frame(false)).on_hover_text("Avançar").clicked() {
                    self.travel(ctx, false);
                }
                ui.separator();
                let label = match self.view {
                    View::Home => "Início",
                    View::Templates => "Modelos",
                    View::Trash => "Lixeira",
                    View::Page(id) => self.space.page(id).map_or("Página", |page| page.title()),
                };
                let label_width = (ui.available_width() - 123.0).max(0.0);
                ui.allocate_ui_with_layout(vec2(label_width, 29.0), Layout::left_to_right(Align::Center), |ui| {
                    ui.add(egui::Label::new(p.label(label)).truncate()).on_hover_text(label);
                });
                right(ui, |ui| {
                    if action(ui, p, "+ Nova página", false).on_hover_text("Nova página (Ctrl+N)").clicked() {
                        self.create(ctx, None, PageKind::Document, true);
                    }
                });
            }));
        if self.tabs.is_empty() { return; }
        TopBottomPanel::top("tabs").frame(Frame::new().fill(p.side).inner_margin(egui::Margin::symmetric(12, 3)))
            .show(ctx, |ui| {
                ScrollArea::horizontal().id_salt("tab-scroll").show(ui, |ui| ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    for id in self.tabs.clone() {
                        let Some(page) = self.space.page(id) else { continue };
                        let label = page.title().to_owned();
                        let active = self.view == View::Page(id);
                        let tab = Frame::new().fill(if active { p.paper } else { p.side })
                            .corner_radius(4).inner_margin(egui::Margin::symmetric(5, 2))
                            .show(ui, |ui| ui.horizontal(|ui| {
                                ui.allocate_ui_with_layout(vec2(128.0, 23.0), Layout::left_to_right(Align::Center), |ui| {
                                    let title = if active { p.heading(&label, 13.0).color(p.accent) } else { p.label(&label) };
                                    if ui.add(Button::new(title).frame(false).truncate())
                                        .on_hover_text(&label).clicked() { self.select(ctx, View::Page(id)); }
                                });
                                if ui.add(Button::new("×").frame(false)).on_hover_text("Fechar aba").clicked() { self.close_tab(ctx, id); }
                            }));
                        if active {
                            let rect = tab.response.rect;
                            ui.painter().line_segment(
                                [rect.left_bottom() + vec2(4.0, 0.0), rect.right_bottom() - vec2(4.0, 0.0)],
                                egui::Stroke::new(2.0_f32, p.accent),
                            );
                        }
                    }
                }));
            });
    }

    fn sidebar(&mut self, ctx: &Context, p: Palette) {
        if !self.space.settings.show_sidebar { return; }
        let width = (ctx.screen_rect().width() * 0.31).clamp(154.0, 226.0);
        SidePanel::left("sidebar").resizable(false).exact_width(width)
            .frame(Frame::new().fill(p.side).inner_margin(egui::Margin::symmetric(12, 13)))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 3.0;
                ui.horizontal(|ui| {
                    design::logo(ui, p);
                    right(ui, |ui| {
                        if ui.add(Button::new("«").frame(false)).on_hover_text("Ocultar barra lateral (Ctrl+E)").clicked() {
                            self.space.settings.show_sidebar = false;
                            self.dirty(ctx);
                        }
                        let width = ui.available_width();
                        ui.allocate_ui_with_layout(vec2(width, 28.0), Layout::left_to_right(Align::Center), |ui| {
                            ui.add(egui::Label::new(p.display("Margem", 20.0)).truncate());
                        });
                    });
                });
                ui.add_space(13.0);
                if design::search_button(ui, p).clicked() { self.start_search(); }
                ui.add_space(9.0);
                let h = if self.space.settings.compact { 23.0 } else { 28.0 };
                if nav_icon(ui, p, "Início", Icon::Home, self.view == View::Home, h).clicked() { self.select(ctx, View::Home); }
                if nav_icon(ui, p, "Modelos", Icon::Models, self.view == View::Templates, h).clicked() { self.select(ctx, View::Templates); }
                ui.add_space(9.0);
                ui.separator();
                ui.add_space(6.0);
                let footer_height = h * 2.0 + 21.0;
                ScrollArea::vertical().id_salt("page-tree").max_height((ui.available_height() - footer_height).max(0.0))
                    .auto_shrink([false, false]).show(ui, |ui| {
                        if self.space.settings.show_favorites {
                            let favorites = self.space.pages.iter().filter(|page| page.favorite && self.space.is_live(page.id))
                                .map(|page| page.id).collect::<Vec<_>>();
                            if !favorites.is_empty() {
                                section(ui, p, "Favoritos");
                                for id in favorites {
                                    let title = self.space.page(id).unwrap().title().to_owned();
                                    if nav_icon(ui, p, &title, Icon::Star, self.view == View::Page(id), h).on_hover_text(&title).clicked() {
                                        self.select(ctx, View::Page(id));
                                    }
                                }
                                ui.add_space(12.0);
                            }
                        }
                        ui.horizontal(|ui| {
                            section(ui, p, "Minhas páginas");
                            right(ui, |ui| {
                                if ui.add(Button::new("+").frame(false)).on_hover_text("Nova página").clicked() {
                                    self.create(ctx, None, PageKind::Document, true);
                                }
                            });
                        });
                        self.tree(ctx, ui, p, None, 0, h);
                    });
                ui.separator();
                if nav_icon(ui, p, "Lixeira", Icon::Trash, self.view == View::Trash, h).clicked() {
                    self.select(ctx, View::Trash);
                }
                if nav_icon(ui, p, "Configurações", Icon::Settings, self.settings_tab.is_some(), h).on_hover_text("Configurações (Ctrl+,)").clicked() {
                    self.settings_tab = Some(SettingsTab::Appearance);
                }
            });
    }

    fn tree(&mut self, ctx: &Context, ui: &mut Ui, p: Palette, parent: Option<Id>, level: usize, row: f32) {
        let items = self.space.pages.iter().filter(|page| !page.trashed && page.parent == parent)
            .map(|page| (page.id, page.title().to_owned(), page.kind)).collect::<Vec<_>>();
        for (id, title, kind) in items {
            let has_children = self.space.pages.iter().any(|page| page.parent == Some(id) && !page.trashed);
            let collapsed = self.collapsed.contains(&id);
            ui.push_id(("tree", id), |ui| ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                ui.add_space((level.min(8) as f32) * 9.0);
                if has_children {
                    let icon = if collapsed { "›" } else { "−" };
                    if ui.add_sized([16.0, row], Button::new(p.label(icon)).frame(false))
                        .on_hover_text("Mostrar ou ocultar subpáginas").clicked() {
                        if collapsed { self.collapsed.remove(&id); } else { self.collapsed.insert(id); }
                    }
                } else {
                    ui.add_space(16.0);
                }
                let response = nav_icon(ui, p, &title, design::page_icon(kind), self.view == View::Page(id), row).on_hover_text(
                    if kind == PageKind::Collection { format!("{title} · Coleção") } else { title.clone() },
                );
                if response.clicked() { self.select(ctx, View::Page(id)); }
                response.context_menu(|ui| { self.page_menu(ctx, ui, id); });
            }));
            if has_children && !self.collapsed.contains(&id) { self.tree(ctx, ui, p, Some(id), level + 1, row); }
        }
    }

    fn page_menu(&mut self, ctx: &Context, ui: &mut Ui, id: Id) {
        let favorite = self.space.page(id).is_some_and(|page| page.favorite);
        for (label, operation) in [
            (if favorite { "Remover dos favoritos" } else { "Adicionar aos favoritos" }, 0),
            ("Nova subpágina", 1),
            ("Mover página", 2),
            ("Exportar Markdown", 3),
            ("Mover para a lixeira", 4),
        ] {
            if ui.button(label).clicked() {
                match operation {
                    0 => self.favorite(ctx, id),
                    1 if self.space.page(id).is_some_and(|page| page.kind == PageKind::Collection) => {
                        self.add_collection_item(ctx, id, Status::Planned);
                    }
                    1 => { self.create(ctx, Some(id), PageKind::Document, true); }
                    2 => self.move_page = Some(id),
                    3 => self.start_transfer(Transfer::Export(id)),
                    _ => self.trash(ctx, id),
                }
                ui.close_menu();
            }
        }
    }

    fn home(&mut self, ctx: &Context, ui: &mut Ui, p: Palette) {
        ui.add_space(25.0);
        design::icon_tile(ui, p, Icon::Home, 46.0);
        ui.add_space(6.0);
        ui.label(p.display(&self.space.settings.name, if ui.available_width() < 450.0 { 36.0 } else { 48.0 }));
        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            if action(ui, p, "+ Criar página", true).clicked() { self.create(ctx, None, PageKind::Document, true); }
            if action(ui, p, "+ Criar coleção", false).clicked() { self.create(ctx, None, PageKind::Collection, true); }
            if action(ui, p, "Ver modelos", false).clicked() { self.select(ctx, View::Templates); }
            if action(ui, p, "Importar Markdown", false).clicked() { self.start_transfer(Transfer::Import); }
        });
        ui.add_space(24.0);
        ui.separator();
        ui.add_space(14.0);
        let live = self.space.pages.iter().filter(|page| self.space.is_live(page.id)).collect::<Vec<_>>();
        let mut by_activity = live.clone();
        by_activity.sort_by_key(|page| std::cmp::Reverse(page.touched));
        let recent = by_activity.iter().take(5).map(|page| (page.id, page.title().to_owned(), page.kind)).collect::<Vec<_>>();
        let favorites = live.iter().filter(|page| page.favorite)
            .map(|page| (page.id, page.title().to_owned(), page.kind)).collect::<Vec<_>>();
        if ui.available_width() >= 615.0 {
            let gap = 32.0;
            let width = ui.available_width() - gap;
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.allocate_ui_with_layout(vec2(width * 0.61, 0.0), Layout::top_down(Align::Min), |ui| {
                    ui.set_width(width * 0.61);
                    self.home_group(ctx, ui, p, "Páginas recentes", &recent, "As páginas que você abrir aparecem aqui.");
                });
                ui.add_space(gap);
                ui.allocate_ui_with_layout(vec2(width * 0.39, 0.0), Layout::top_down(Align::Min), |ui| {
                    ui.set_width(width * 0.39);
                    self.home_group(ctx, ui, p, "Favoritos", &favorites, "Marque uma página para encontrá-la aqui.");
                });
            });
        } else {
            self.home_group(ctx, ui, p, "Páginas recentes", &recent, "As páginas que você abrir aparecem aqui.");
            ui.add_space(15.0);
            self.home_group(ctx, ui, p, "Favoritos", &favorites, "Marque uma página para encontrá-la aqui.");
        }
    }

    fn home_group(&mut self, ctx: &Context, ui: &mut Ui, p: Palette, title: &str, rows: &[(Id, String, PageKind)], empty: &str) {
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.y = 3.0;
            ui.label(p.heading(title, 17.0));
            ui.add_space(5.0);
            if rows.is_empty() { ui.label(p.label(empty)); }
            for (index, (id, name, kind)) in rows.iter().enumerate() {
                if index > 0 { ui.separator(); }
                if page_row(ui, p, name, *kind, "›").clicked() { self.select(ctx, View::Page(*id)); }
            }
        });
    }

    fn page(&mut self, ctx: &Context, ui: &mut Ui, p: Palette, id: Id) {
        let Some(page) = self.space.page(id) else { return };
        let kind = page.kind;
        let favorite = page.favorite;
        let parent = page.parent.and_then(|parent| self.space.page(parent).map(|page| (parent, page.title().to_owned())));
        let mut changed = false;
        right(ui, |ui| {
            ui.menu_button("Mais", |ui| { self.page_menu(ctx, ui, id); });
            if ui.add(Button::new(p.label(if favorite { "★ Favorito" } else { "☆ Favoritar" })).frame(false)).clicked() {
                self.favorite(ctx, id);
            }
            section(ui, p, if kind == PageKind::Collection { "Coleção" } else { "Página" });
            if let Some((parent_id, title)) = &parent {
                ui.label(p.label("›"));
                if ui.add(Button::new(p.label(title)).frame(false).truncate())
                    .on_hover_text(title).clicked() { self.select(ctx, View::Page(*parent_id)); }
            }
        });
        if self.view != View::Page(id) { return; }
        let page = self.space.page_mut(id).unwrap();
        ui.add_space(if kind == PageKind::Collection { 13.0 } else { 30.0 });
        design::icon_tile(ui, p, design::page_icon(kind), 47.0);
        ui.add_space(4.0);
        let size = if ui.available_width() < 450.0 { 33.0 } else { 44.0 };
        let title = ui.add(editable(&mut page.title, "Sem título", design::display_font(size))
            .hint_text(p.display("Sem título", size).color(p.muted))
            .margin(egui::Margin::ZERO).id_salt(("page-title", id)));
        if std::mem::take(&mut self.focus_title) { title.request_focus(); }
        changed |= title.changed();
        ui.add_space(11.0);
        if kind == PageKind::Collection {
            self.collection(ctx, ui, p, id);
        } else {
            if let Some(template) = self.template_for_item(id) {
                self.item_properties(ctx, ui, p, id, template);
            }
            self.editor(ctx, ui, p, id);
        }
        if changed {
            self.space.touch(id);
            self.dirty(ctx);
        }
    }

    fn block_menu(ui: &mut Ui, p: Palette, index: usize, count: usize) -> Option<BlockChange> {
        let mut choice = None;
        ui.menu_button(p.label("⋮"), |ui| {
            ui.menu_button("Transformar em", |ui| {
                for kind in Kind::ALL {
                    if ui.button(kind.label()).clicked() {
                        choice = Some(BlockChange::Convert(index, kind));
                        ui.close_menu();
                    }
                }
            });
            if ui.button("Duplicar").clicked() { choice = Some(BlockChange::Duplicate(index)); }
            if ui.add_enabled(index > 0, Button::new("Subir")).clicked() {
                choice = Some(BlockChange::Move(index, index - 1));
            }
            if ui.add_enabled(index + 1 < count, Button::new("Descer")).clicked() {
                choice = Some(BlockChange::Move(index, index + 1));
            }
            if ui.button("Remover bloco").clicked() { choice = Some(BlockChange::Remove(index)); }
            if choice.is_some() { ui.close_menu(); }
        }).response.on_hover_text("Opções do bloco");
        choice
    }

    fn editor(&mut self, ctx: &Context, ui: &mut Ui, p: Palette, id: Id) {
        let mut operation = None;
        let mut changed = false;
        let page = self.space.page_mut(id).unwrap();
        let count = page.blocks.len();
        let mut list_number = 0;
        for (i, block) in page.blocks.iter_mut().enumerate() {
            if block.kind == Kind::Numbered { list_number += 1; } else { list_number = 0; }
            ui.push_id(block.id, |ui| {
                ui.horizontal_top(|ui| {
                    ui.scope(|ui| {
                        ui.spacing_mut().button_padding = vec2(5.0, 3.0);
                        ui.visuals_mut().widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
                        ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::NONE;
                        if let Some(choice) = Self::block_menu(ui, p, i, count) { operation = Some(choice); }
                    });
                    match block.kind {
                        Kind::Divider => {
                            let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 22.0), egui::Sense::hover());
                            ui.painter().hline(rect.x_range(), rect.center().y, egui::Stroke::new(1.0_f32, p.line));
                            return;
                        }
                        Kind::Task => { changed |= ui.checkbox(&mut block.checked, "").changed(); }
                        Kind::Bullet => { ui.label("•"); }
                        Kind::Numbered => { ui.label(format!("{list_number}.")); }
                        Kind::Link => {
                            if (block.text.starts_with("https://") || block.text.starts_with("http://"))
                                && ui.small_button("Abrir").clicked() {
                                ctx.open_url(egui::OpenUrl::new_tab(block.text.trim()));
                            }
                        }
                        _ => {}
                    }
                    let font = match block.kind {
                        Kind::Heading => design::display_font(27.0),
                        Kind::Subheading => design::display_font(21.0),
                        Kind::Code => FontId::monospace(14.0),
                        _ => FontId::proportional(16.0),
                    };
                    let placeholder = if block.kind == Kind::Link { "https://…" } else { "Escreva aqui" };
                    let text_edit = TextEdit::multiline(&mut block.text)
                        .desired_rows(1).font(font)
                        .hint_text(p.label(placeholder)).desired_width(f32::INFINITY)
                        .id_salt("text").frame(false);
                    let response = match block.kind {
                        Kind::Quote | Kind::Code => {
                            let quote = block.kind == Kind::Quote;
                            let body = Frame::new().fill(if quote { p.tint } else { p.side })
                                .corner_radius(5).inner_margin(egui::Margin::symmetric(9, 5))
                                .show(ui, |ui| ui.add(text_edit));
                            if quote {
                                let rail = body.response.rect.left_center().x + 2.0;
                                ui.painter().vline(rail, body.response.rect.y_range().shrink(4.0), egui::Stroke::new(2.0_f32, p.accent));
                            }
                            body.inner
                        }
                        _ => ui.add(text_edit),
                    };
                    if self.focus_block == Some(block.id) {
                        response.request_focus();
                        self.focus_block = None;
                    }
                    changed |= response.changed();
                });
                if let Some(query) = block.text.strip_prefix('/') {
                    let query = search_key(query);
                    if !query.contains('\n') {
                        let hits = Kind::ALL.into_iter().filter(|kind| search_key(kind.label()).contains(&query)).collect::<Vec<_>>();
                        if !hits.is_empty() {
                            panel(p).show(ui, |ui| {
                                section(ui, p, "Tipo de bloco");
                                ui.horizontal_wrapped(|ui| {
                                    for kind in hits {
                                        if ui.button(kind.label()).clicked() {
                                            block.text.clear();
                                            operation = Some(BlockChange::Convert(i, kind));
                                        }
                                    }
                                });
                            });
                        }
                    }
                }
            });
        }
        if count == 0 {
            let start = ui.add(Button::new(p.label("Escreva aqui").size(16.0)).frame(false));
            if start.clicked() { operation = Some(BlockChange::Add(Kind::Text)); }
            ui.label(p.label("Digite / para escolher um tipo de bloco."));
            ui.add_space(8.0);
        }
        ui.add_space(5.0);
        ui.horizontal_wrapped(|ui| {
            if action(ui, p, "+ Texto", false).clicked() { operation = Some(BlockChange::Add(Kind::Text)); }
            ui.menu_button("+ Bloco", |ui| {
                for kind in Kind::ALL {
                    if ui.button(kind.label()).clicked() {
                        operation = Some(BlockChange::Add(kind));
                        ui.close_menu();
                    }
                }
            });
        });
        if let Some(operation) = operation {
            changed = true;
            match operation {
                BlockChange::Add(kind) => {
                    self.focus_block = Some(self.space.add_block(id, kind, ""));
                }
                BlockChange::Convert(index, kind) => {
                    let block = &mut self.space.page_mut(id).unwrap().blocks[index];
                    block.kind = kind;
                    self.focus_block = Some(block.id);
                }
                BlockChange::Duplicate(index) => {
                    let copy_id = self.space.alloc();
                    let blocks = &mut self.space.page_mut(id).unwrap().blocks;
                    let mut copy = blocks[index].clone();
                    copy.id = copy_id;
                    blocks.insert(index + 1, copy);
                    self.focus_block = Some(copy_id);
                }
                BlockChange::Move(i, j) => self.space.page_mut(id).unwrap().blocks.swap(i, j),
                BlockChange::Remove(index) => { self.space.page_mut(id).unwrap().blocks.remove(index); }
            }
        }
        if changed {
            self.space.touch(id);
            self.dirty(ctx);
        }
        let children = self.space.pages.iter().filter(|page| page.parent == Some(id) && !page.trashed)
            .map(|page| (page.id, page.title().to_owned(), page.kind)).collect::<Vec<_>>();
        if !children.is_empty() {
            ui.add_space(12.0);
            ui.separator();
            section(ui, p, "Subpáginas");
            for (child, title, kind) in children {
                if page_row(ui, p, &title, kind, "›").clicked() { self.select(ctx, View::Page(child)); }
            }
        }
    }

    fn trash_view(&mut self, ctx: &Context, ui: &mut Ui, p: Palette) {
        ui.add_space(16.0);
        design::icon_tile(ui, p, Icon::Trash, 46.0);
        ui.label(p.display("Lixeira", 42.0));
        ui.add_space(8.0);
        let rows = self.space.pages.iter().filter(|page| page.trashed)
            .map(|page| (page.id, page.title().to_owned(), page.kind, page.parent)).collect::<Vec<_>>();
        ui.separator();
        if rows.is_empty() { ui.label(p.label("A lixeira está vazia.")); }
        for (id, title, kind, parent) in rows {
            right(ui, |ui| {
                if action(ui, p, "Restaurar", false).clicked() {
                    let is_visible = parent.is_none_or(|item| self.space.is_live(item));
                    let page = self.space.page_mut(id).unwrap();
                    if !is_visible { page.parent = None; }
                    page.trashed = false;
                    self.space.touch(id);
                    self.dirty(ctx);
                    self.notify(ctx, "Página restaurada.");
                }
                let width = ui.available_width();
                ui.allocate_ui_with_layout(vec2(width, 34.0), Layout::left_to_right(Align::Center), |ui| {
                    design::icon_tile(ui, p, design::page_icon(kind), 24.0);
                    ui.add(egui::Label::new(p.heading(&title, 14.0)).truncate()).on_hover_text(title);
                });
            });
            ui.separator();
        }
    }
    fn search_window(&mut self, ctx: &Context, p: Palette) {
        if !self.search { return; }
        let mut open = self.search;
        let mut chosen = None;
        let width = (ctx.screen_rect().width() - 52.0).min(490.0);
        egui::Window::new("Buscar no espaço").id(egui::Id::new("search-window")).open(&mut open)
            .anchor(Align2::CENTER_TOP, vec2(0.0, 86.0)).collapsible(false)
            .resizable(false).default_width(width).show(ctx, |ui| {
                ui.set_width(width);
                let response = ui.add(TextEdit::singleline(&mut self.query).hint_text("Digite um título ou trecho…")
                    .margin(egui::Margin::symmetric(10, 7)).desired_width(f32::INFINITY).font(FontId::proportional(16.0)));
                if std::mem::take(&mut self.focus_search) { response.request_focus(); }
                let query = search_key(self.query.trim());
                let mut matches = self.space.pages.iter().filter(|page| self.space.is_live(page.id))
                    .filter_map(|page| {
                        if search_key(page.title()).contains(&query) {
                            Some((page.id, page.title().to_owned(), page.kind, 0, page.touched))
                        } else if page.blocks.iter().any(|block| search_key(&block.text).contains(&query))
                            || page.properties.values().any(|value| search_key(value).contains(&query))
                            || search_key(&page.tag).contains(&query) {
                            Some((page.id, page.title().to_owned(), page.kind, 1, page.touched))
                        } else { None }
                    }).collect::<Vec<_>>();
                matches.sort_by_key(|row| (row.3, std::cmp::Reverse(row.4)));
                if ctx.input(|input| input.key_pressed(egui::Key::Enter)) {
                    chosen = matches.first().map(|row| row.0);
                }
                ui.add_space(4.0);
                section(ui, p, if query.is_empty() { "Páginas recentes" } else { "Resultados" });
                let results_height = (ctx.screen_rect().height() - 280.0).clamp(80.0, 310.0);
                ScrollArea::vertical().max_height(results_height).show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    if matches.is_empty() { ui.label(p.label("Nenhuma página encontrada.")); }
                    for (id, title, kind, rank, _) in matches {
                        let trail = if rank == 1 { "Conteúdo" } else { "›" };
                        if page_row(ui, p, &title, kind, trail).clicked() { chosen = Some(id); }
                    }
                });
                ui.separator();
                ui.label(p.label("Enter abre o primeiro resultado · Esc fecha"));
            });
        self.search = open && chosen.is_none();
        if let Some(id) = chosen { self.select(ctx, View::Page(id)); }
    }

    fn move_window(&mut self, ctx: &Context, p: Palette) {
        let Some(id) = self.move_page else { return };
        let mut open = true;
        let mut target = None;
        egui::Window::new("Mover página").open(&mut open).anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .collapsible(false).resizable(false).default_width(370.0).show(ctx, |ui| {
                ui.label(p.label("Escolha onde a página ficará."));
                if action(ui, p, "Minhas páginas", false).clicked() { target = Some(None); }
                ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                    for page in &self.space.pages {
                        if page.id != id && self.space.is_live(page.id)
                            && page_row(ui, p, page.title(), page.kind, "›").clicked() {
                            target = Some(Some(page.id));
                        }
                    }
                });
            });
        if let Some(parent) = target {
            if self.space.move_page(id, parent) {
                if let Some(parent) = parent { self.collapsed.remove(&parent); }
                self.move_page = None;
                self.dirty(ctx);
                self.notify(ctx, "Página movida.");
            } else { self.notify(ctx, "Escolha um destino fora da própria página."); }
        }
        if !open { self.move_page = None; }
    }

    fn transfer_window(&mut self, ctx: &Context, p: Palette) {
        let Some(transfer) = self.transfer else { return };
        let (title, button) = match transfer {
            Transfer::Import => ("Importar Markdown", "Importar"),
            Transfer::Export(_) => ("Exportar Markdown", "Exportar"),
        };
        let mut open = true;
        let mut submitted = false;
        egui::Window::new(title).open(&mut open).anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .collapsible(false).resizable(false).show(ctx, |ui| {
                ui.label(p.label("Caminho completo do arquivo Markdown"));
                let width = (ctx.screen_rect().width() - 60.0).min(410.0);
                ui.add(TextEdit::singleline(&mut self.transfer_path).hint_text("C:\\Documentos\\minha-página.md")
                    .margin(egui::Margin::symmetric(8, 6)).desired_width(width));
                ui.add_space(4.0);
                right(ui, |ui| {
                    submitted = ui.add_enabled_ui(!self.transfer_path.trim().is_empty(), |ui| action(ui, p, button, true))
                        .inner.clicked();
                });
            });
        if !open {
            self.transfer = None;
            return;
        }
        if !submitted { return; }
        let path = PathBuf::from(self.transfer_path.trim().trim_matches('"'));
        let result = match transfer {
            Transfer::Import => std::fs::read_to_string(&path).map(|markdown| {
                let title = path.file_stem().and_then(|stem| stem.to_str()).unwrap_or("Página importada");
                let id = self.space.import_markdown(title, &markdown);
                self.select(ctx, View::Page(id));
                "Página importada."
            }),
            Transfer::Export(id) => {
                let page = self.space.page(id).unwrap();
                let mut markdown = page.markdown();
                if page.kind == PageKind::Collection {
                    markdown.push_str("\n| Título | Status | Etiqueta | Data |\n| --- | --- | --- | --- |\n");
                    let escape = |s: &str| s.replace('|', "\\|").replace('\n', " ");
                    for row in self.space.pages.iter().filter(|row| row.parent == Some(id) && !row.trashed) {
                        markdown.push_str(&format!("| {} | {} | {} | {} |\n", escape(row.title()), row.status.label(), escape(&row.tag), escape(&row.date)));
                    }
                }
                use std::io::Write;
                std::fs::OpenOptions::new().write(true).create_new(true).open(&path)
                    .and_then(|mut file| file.write_all(markdown.as_bytes())).map(|_| "Página exportada.")
            }
        };
        match result {
            Ok(message) => {
                self.transfer = None;
                self.notify(ctx, message);
            }
            Err(error) => self.notify(ctx, format!("Não foi possível concluir: {error}")),
        }
    }

    fn capture(&mut self, ctx: &Context) {
        let Some(base) = &self.screenshot else { return };
        let Some(suffix) = ["", "configuracoes", "editor", "colecao", "modelos", "modelo-tarefas"].get(self.capture_stage).copied() else { return };
        let path = if suffix.is_empty() {
            base.clone()
        } else {
            let stem = base.file_stem().unwrap_or_default().to_string_lossy();
            base.with_file_name(format!("{stem}-{suffix}.png"))
        };
        let screenshot = ctx.input(|input| input.events.iter().find_map(|event| match event {
            egui::Event::Screenshot { image, .. } => Some(image.clone()),
            _ => None,
        }));
        if let Some(screenshot) = screenshot {
            let result = (|| -> Result<(), Box<dyn std::error::Error>> {
                if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
                    std::fs::create_dir_all(parent)?;
                }
                let pixels: Vec<u8> = screenshot.pixels.iter().flat_map(|pixel| pixel.to_array()).collect();
                image::save_buffer_with_format(&path, &pixels, screenshot.width() as u32, screenshot.height() as u32,
                    image::ColorType::Rgba8, image::ImageFormat::Png)?;
                Ok(())
            })();
            if let Err(error) = result {
                eprintln!("Não foi possível salvar a captura: {error}");
                ctx.send_viewport_cmd(ViewportCommand::Close);
                return;
            }
            self.capture_stage += 1;
            self.frames = 0;
            self.shot_requested = false;
            match self.capture_stage {
                1 => self.settings_tab = Some(SettingsTab::Appearance),
                2 => {
                    self.settings_tab = None;
                    self.create(ctx, None, PageKind::Document, true);
                }
                3 => {
                    if let Some(id) = self.space.pages.iter().find(|page| page.kind == PageKind::Collection).map(|page| page.id) {
                        self.select(ctx, View::Page(id));
                    }
                }
                4 => self.select(ctx, View::Templates),
                5 => {
                    let id = templates::CATALOG[0].create(&mut self.space);
                    self.select(ctx, View::Page(id));
                }
                _ => {
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                    return;
                }
            }
            ctx.request_repaint();
            return;
        }
        self.frames += 1;
        if self.frames >= 36 && !self.shot_requested {
            self.shot_requested = true;
            ctx.send_viewport_cmd(ViewportCommand::Screenshot(Default::default()));
        }
        if self.frames > 300 {
            eprintln!("A captura não retornou uma imagem.");
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
        ctx.request_repaint_after(Duration::from_millis(12));
    }

    fn render(&mut self, ctx: &Context) {
        let settings = &self.space.settings;
        let dark = match settings.theme {
            Theme::Dark => true,
            Theme::Light => false,
            Theme::System => ctx.system_theme() == Some(egui::Theme::Dark),
        };
        let appearance = (dark, settings.high_contrast, settings.compact, settings.zoom);
        let p = Palette::new(dark, settings.high_contrast);
        if self.applied_appearance != Some(appearance) {
            p.apply(ctx, dark, settings.compact);
            ctx.set_zoom_factor(settings.zoom);
            self.applied_appearance = Some(appearance);
        }
        self.shortcuts(ctx);
        self.sidebar(ctx, p);
        self.toolbar(ctx, p);
        CentralPanel::default().frame(Frame::new().fill(p.paper).inner_margin(egui::Margin::symmetric(26, 20))).show(ctx, |ui| {
            ScrollArea::vertical().id_salt(match self.view {
                View::Home => ("home", 0),
                View::Templates => ("templates", 0),
                View::Page(id) => ("page", id),
                View::Trash => ("trash", 0),
            }).show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    let wide = match self.view {
                        View::Page(id) => self.space.settings.wide_documents
                            || self.space.page(id).is_some_and(|page| page.kind == PageKind::Collection),
                        _ => true,
                    };
                    let max_width = match self.view {
                        View::Home | View::Templates => 990.0,
                        _ if wide => 1160.0,
                        _ => 715.0,
                    };
                    let width = ui.available_width().min(max_width);
                    ui.allocate_ui_with_layout(vec2(width, 0.0), Layout::top_down(Align::Min), |ui| {
                        ui.set_width(width);
                        if self.storage_blocked {
                            ui.label(p.label("As páginas salvas não puderam ser abertas. Exporte as alterações que desejar guardar."));
                            ui.add_space(8.0);
                        }
                        match self.view {
                            View::Home => self.home(ctx, ui, p),
                            View::Templates => self.template_library(ctx, ui, p),
                            View::Page(id) => self.page(ctx, ui, p, id),
                            View::Trash => self.trash_view(ctx, ui, p),
                        }
                    });
                });
            });
        });
        self.search_window(ctx, p);
        self.move_window(ctx, p);
        self.transfer_window(ctx, p);
        self.settings_window(ctx, p);
        if let Some((message, since)) = &self.notice {
            if since.elapsed() < Duration::from_secs(5) {
                egui::Area::new("notice".into()).anchor(Align2::RIGHT_BOTTOM, vec2(-18.0, -16.0))
                    .show(ctx, |ui| { panel(p).show(ui, |ui| { ui.label(message); }); });
            }
        }
        if self.dirty_since.is_some_and(|since| cfg!(target_os = "android") || since.elapsed() >= Duration::from_millis(500))
            || ctx.input(|input| input.viewport().close_requested()) { self.save(); }
        self.capture(ctx);
    }
}

impl eframe::App for Margem {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.render(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save();
    }
}

fn workspace_path() -> Result<PathBuf, String> {
    if cfg!(target_os = "windows") && !cfg!(debug_assertions) {
        let local = std::env::var_os("LOCALAPPDATA")
            .filter(|folder| !folder.is_empty())
            .ok_or_else(|| "A pasta LOCALAPPDATA do Windows não está disponível.".to_owned())?;
        return Ok(PathBuf::from(local).join("Margem").join("workspace.json"));
    }
    if cfg!(target_os = "macos") && !cfg!(debug_assertions) {
        let home = std::env::var_os("HOME").filter(|path| !path.is_empty())
            .ok_or_else(|| "A pasta pessoal do macOS não está disponível.".to_owned())?;
        return Ok(PathBuf::from(home).join("Library/Application Support/Margem/workspace.json"));
    }

    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let folder = if manifest.join("Cargo.toml").exists() {
        manifest.to_path_buf()
    } else {
        std::env::current_exe().ok().and_then(|exe| exe.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| PathBuf::from("."))
    };
    Ok(folder.join("workspace.json"))
}

fn load(path: &Path) -> Result<Option<Workspace>, String> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    serde_json::from_slice(&bytes).map(Some).map_err(|error| error.to_string())
}
