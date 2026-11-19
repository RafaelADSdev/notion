use super::*;
use egui::{Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, Shape};
use serde_json::{json, Value};

struct Interface {
    ctx: Context,
    app: Margem,
    shapes: Vec<egui::epaint::ClippedShape>,
    size: Vec2,
}

impl Interface {
    fn new() -> Self {
        let ctx = Context::default();
        // Capture mode starts with sample pages and never writes the user's workspace.
        let app = Margem::new(&ctx, Some(PathBuf::from("ui-test.png")));
        let mut interface = Self { ctx, app, shapes: Vec::new(), size: vec2(1160.0, 740.0) };
        interface.frame(Vec::new(), Modifiers::NONE);
        interface.frame(Vec::new(), Modifiers::NONE);
        interface
    }

    fn frame(&mut self, events: Vec<Event>, modifiers: Modifiers) {
        self.shapes = self.ctx.run(RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, self.size)),
            events,
            modifiers,
            ..Default::default()
        }, |ctx| self.app.render(ctx)).shapes;
    }

    fn labels(&self, label: &str) -> Vec<Pos2> {
        self.shapes.iter().filter_map(|clipped| {
            let Shape::Text(text) = &clipped.shape else { return None };
            let point = text.pos + text.galley.rect.center().to_vec2();
            (text.galley.text() == label && clipped.clip_rect.contains(point)).then_some(point)
        }).collect()
    }

    fn click(&mut self, label: &str, occurrence: usize) {
        let point = *self.labels(label).get(occurrence)
            .unwrap_or_else(|| {
                let visible = self.shapes.iter().filter_map(|item| {
                    let Shape::Text(text) = &item.shape else { return None };
                    Some(text.galley.text())
                }).collect::<Vec<_>>();
                panic!("Controle não encontrado: {label} ({occurrence}); textos: {visible:?}");
            });
        self.frame(vec![Event::PointerMoved(point)], Modifiers::NONE);
        for pressed in [true, false] {
            self.frame(vec![Event::PointerButton {
                pos: point,
                button: PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            }], Modifiers::NONE);
        }
        self.frame(Vec::new(), Modifiers::NONE);
    }

    fn key(&mut self, key: Key, modifiers: Modifiers) {
        self.frame(vec![Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }], modifiers);
        self.frame(Vec::new(), Modifiers::NONE);
    }

    fn type_text(&mut self, text: &str) {
        self.frame(vec![Event::Text(text.to_owned())], Modifiers::NONE);
        self.frame(Vec::new(), Modifiers::NONE);
    }

    fn current_title(&self) -> &str {
        match self.app.view {
            View::Page(id) => self.app.space.page(id).unwrap().title(),
            _ => "",
        }
    }
}

type Measurement = (&'static str, Value);

fn check(name: &'static str, measured: Value) -> Measurement {
    let sheet: Value = serde_json::from_str(include_str!("../../mechanics.json")).unwrap();
    let entry = sheet["values"].as_array().unwrap().iter()
        .find(|item| item["name"] == name)
        .unwrap_or_else(|| panic!("Mecânica ausente da planilha: {name}"));
    assert_eq!(measured, entry["value"], "{name}");
    (name, measured)
}

fn navigation_tabs_history_and_search() -> [Measurement; 3] {
    let mut ui = Interface::new();
    ui.click("Planejamento semanal", 0);
    ui.click("Caderno de ideias", 0);

    ui.click("←", 0);
    let went_back = ui.current_title() == "Planejamento semanal";
    ui.click("→", 0);
    let history = check("Navegação de volta e avanço", json!(went_back && ui.current_title() == "Caderno de ideias"));

    let tab_visible = ui.labels("×").len() == 2;
    ui.click("×", 1);
    let tabs = check("Abas para páginas", json!(tab_visible && ui.labels("×").len() == 1
        && ui.current_title() == "Planejamento semanal"));

    ui.key(Key::K, Modifiers::COMMAND);
    ui.type_text("lista de leituras");
    let found_by_content = ui.labels("Conteúdo").len() == 1;
    ui.key(Key::Enter, Modifiers::NONE);
    let search = check("Busca rápida", json!(found_by_content && !ui.app.search
        && ui.current_title() == "Caderno de ideias"));
    [tabs, history, search]
}

fn editor_transforms_and_moves_blocks() -> [Measurement; 3] {
    let mut ui = Interface::new();
    ui.click("Planejamento semanal", 0);
    ui.click("+ Texto", 0);
    ui.type_text("/");
    assert_eq!(ui.labels("Citação").len(), 1);
    let menu = check("Menu de blocos", json!(ui.labels("/").first().map(|_| "/")));

    ui.click("Citação", 0);
    ui.type_text("Um bloco de teste");

    let View::Page(page_id) = ui.app.view else { panic!("A página do editor precisa estar aberta"); };
    let quote = ui.app.space.page(page_id).unwrap().blocks.last().unwrap();
    let transform = check("Transformar blocos", json!(quote.kind == Kind::Quote && quote.text == "Um bloco de teste"));
    let quote_id = quote.id;
    let index = ui.app.space.page(page_id).unwrap().blocks.len() - 1;
    ui.click("⋮", index);
    ui.click("Subir", 0);
    let reorder = check("Reordenar blocos", json!(ui.app.space.page(page_id).unwrap().blocks[index - 1].id == quote_id));
    [menu, transform, reorder]
}

fn collection_views_and_default_appearance() -> [Measurement; 4] {
    let mut ui = Interface::new();
    let theme = if ui.ctx.style().visuals.dark_mode { "Escuro" } else { "Claro" };
    let theme = check("Tema ao abrir pela primeira vez", json!(theme));
    let zoom = ui.ctx.zoom_factor();
    let measured_zoom = if zoom.fract() == 0.0 { json!(zoom as u32) } else { json!(zoom) };
    let zoom = check("Zoom inicial", measured_zoom);

    ui.click("Projetos pessoais", 0);
    assert!(!ui.labels("Etiqueta").is_empty(), "A tabela precisa renderizar suas colunas");
    ui.click("Abrir", 0);
    let opened_from_table = ui.current_title() == "Organizar referências";
    assert!(opened_from_table, "Abrir deveria navegar para a primeira linha, abriu {:?}", ui.current_title());
    ui.click("←", 0);
    ui.click("Quadro", 0);
    assert_eq!(ui.labels("Em andamento  ·  1").len(), 1);
    assert_eq!(ui.labels("Abrir").len(), 0);
    let views = check("Visualizações implementadas", json!("Tabela e quadro"));
    let View::Page(collection) = ui.app.view else { panic!("A coleção precisa estar aberta"); };
    ui.click("Ordenar por nome", 0);
    ui.click("Todos os status", 0);
    let status_labels = ui.labels("Em andamento").len();
    ui.click("Em andamento", status_labels - 1);
    assert!(ui.labels("Concluído  ·  1").is_empty());

    let cards = ui.labels("Organizar referências");
    assert!(cards.len() >= 2, "O quadro precisa mostrar o item além do menu lateral");
    ui.click("Organizar referências", cards.len() - 1);
    let items = check("Itens da coleção abrem como páginas", json!(opened_from_table
        && ui.current_title() == "Organizar referências"));
    ui.click("←", 0);
    assert_eq!(ui.labels("Em andamento  ·  1").len(), 1);
    assert!(ui.labels("Concluído  ·  1").is_empty());

    ui.app.space.create(None, "Outra coleção", PageKind::Collection);
    let saved = serde_json::to_vec(&ui.app.space).unwrap();
    ui.app.space = serde_json::from_slice(&saved).unwrap();
    ui.frame(Vec::new(), Modifiers::NONE);
    ui.click("Outra coleção", 0);
    assert!(!ui.labels("Etiqueta").is_empty());
    assert_eq!(ui.labels("Todos os status").len(), 1);
    ui.click("Projetos pessoais", 0);
    assert_eq!(ui.labels("Em andamento  ·  1").len(), 1);
    assert!(ui.labels("Concluído  ·  1").is_empty());

    let options = ui.app.space.page(collection).unwrap().collection_options;
    assert!(options.sort_name);
    [items, views, theme, zoom]
}

fn settings_and_empty_editor() {
    let mut ui = Interface::new();
    ui.click("Planejamento semanal", 0);
    ui.click("Configurações", 0);
    assert!(ui.app.settings_tab == Some(SettingsTab::Appearance), "O botão abre configurações");
    ui.click("Ativar alto contraste", 0);
    assert!(ui.app.space.settings.high_contrast);
    ui.click("Claro", 0);
    ui.click("Escuro", 0);
    assert!(ui.ctx.style().visuals.dark_mode);
    ui.click("Editor", 0);
    ui.click("Usar largura ampla", 0);
    assert!(ui.app.space.settings.wide_documents);
    ui.click("Barra lateral", 0);
    assert_eq!(ui.app.settings_tab, Some(SettingsTab::Sidebar));
    ui.click("Exibir favoritos", 0);
    ui.click("Exibir barra lateral", 0);
    assert!(!ui.app.space.settings.show_favorites && !ui.app.space.settings.show_sidebar);
    ui.key(Key::Escape, Modifiers::NONE);
    assert!(ui.app.settings_tab.is_none());
    assert_eq!(ui.current_title(), "Planejamento semanal");

    let saved = serde_json::to_vec(&ui.app.space).unwrap();
    let restored: Workspace = serde_json::from_slice(&saved).unwrap();
    assert!(restored.settings == ui.app.space.settings);
    ui.click("Configurações", 0);
    ui.click("Fechar", 0);
    ui.click("+ Nova página", 0);
    ui.click("Escreva aqui", 0);
    ui.type_text("Primeira anotação");
    let View::Page(id) = ui.app.view else { panic!("O editor precisa estar aberto"); };
    assert_eq!(ui.app.space.page(id).unwrap().blocks[0].text, "Primeira anotação");
}

fn template_creation_and_persistence() {
    let mut ui = Interface::new();
    ui.click("Ver modelos", 0);
    assert!(!ui.labels("Controle de tarefas").is_empty());
    ui.type_text("brainstorming");
    assert!(ui.labels("Controle de tarefas").is_empty());
    ui.click("Usar modelo", 0);
    assert_eq!(ui.current_title(), "Sessão de brainstorming");
    assert!(!ui.labels("Categoria").is_empty());
    let View::Page(collection) = ui.app.view else { panic!("A coleção do modelo deve abrir"); };
    ui.click("+ Ideia", 0);
    ui.type_text("Revisar as sugestões");
    let View::Page(item) = ui.app.view else { panic!("O item deve abrir como página"); };
    let page = ui.app.space.page(item).unwrap();
    assert_eq!(page.title(), "Revisar as sugestões");
    assert_eq!(page.parent, Some(collection));
    assert_eq!(page.blocks[0].text, "Ideia em detalhes");
    ui.click("Apoiar · 0", 0);
    assert!(ui.app.space.page(item).unwrap().voted);
    ui.app.space.page_mut(item).unwrap().tag = "Melhorias".to_owned();
    ui.click("←", 0);
    ui.click("Por categoria", 0);
    assert_eq!(ui.labels("Melhorias").len(), 1);
    let saved = serde_json::to_vec(&ui.app.space).unwrap();
    let restored: Workspace = serde_json::from_slice(&saved).unwrap();
    assert!(restored.page(item).unwrap().voted);
    assert_eq!(restored.page(collection).unwrap().collection_template, "ideias");
    assert!(restored.page(collection).unwrap().collection_options.view == CollectionView::Category);

    ui.click("Modelos", 0);
    ui.type_text("calendario de conteudo");
    ui.click("Usar modelo", 0);
    assert_eq!(ui.current_title(), "Calendário de conteúdo");
    ui.click("+ Adicionar neste dia", 0);
    let View::Page(item) = ui.app.view else { panic!("O item do calendário deve abrir"); };
    assert!(!ui.app.space.page(item).unwrap().date.is_empty());
    ui.type_text("Publicar a pauta");
    ui.click("←", 0);
    let titles = ui.labels("Publicar a pauta");
    assert!(titles.len() >= 2, "O item deve aparecer no calendário e na árvore");
}

/// Exercise the same egui controls as the desktop window, using an isolated sample workspace.
pub fn verify_mechanics() -> Value {
    settings_and_empty_editor();
    template_creation_and_persistence();
    let observed = navigation_tabs_history_and_search().into_iter()
        .chain(editor_transforms_and_moves_blocks())
        .chain(collection_views_and_default_appearance())
        .collect::<std::collections::BTreeMap<_, _>>();
    json!({ "observed": observed, "local_checks": ["Preferências por coleção", "Configurações persistentes", "Escrita em página vazia", "Busca e aplicação de modelos", "Votos locais e calendário por data"] })
}

#[cfg(test)]
#[test]
fn desktop_interface_mechanics() {
    verify_mechanics();
}

#[cfg(test)]
#[test]
fn narrow_dark_workspace_keeps_navigation_and_editor_reachable() {
    let mut ui = Interface::new();
    ui.size = vec2(680.0, 490.0);
    ui.app.space.settings.theme = Theme::Dark;
    ui.frame(Vec::new(), Modifiers::NONE);
    ui.frame(Vec::new(), Modifiers::NONE);
    assert!(ui.ctx.style().visuals.dark_mode);
    ui.click("Configurações", 0);
    assert_eq!(ui.app.settings_tab, Some(SettingsTab::Appearance));
    // The same window offers fewer logical points at high interface zoom.
    ui.size = vec2(530.0, 390.0);
    ui.frame(Vec::new(), Modifiers::NONE);
    ui.frame(Vec::new(), Modifiers::NONE);
    ui.click("Editor", 0);
    assert_eq!(ui.app.settings_tab, Some(SettingsTab::Editor));
    assert_eq!(ui.labels("Usar largura ampla").len(), 1);
    ui.key(Key::Escape, Modifiers::NONE);
    ui.click("+ Criar página", 0);
    ui.click("Escreva aqui", 0);
    ui.type_text("Anotação");
    let View::Page(id) = ui.app.view else { panic!("O editor precisa estar aberto"); };
    assert_eq!(ui.app.space.page(id).unwrap().blocks[0].text, "Anotação");
}
