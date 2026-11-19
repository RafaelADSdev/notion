use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub type Id = u64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    #[default]
    Text,
    Heading,
    Subheading,
    Task,
    Bullet,
    Numbered,
    Quote,
    Code,
    Divider,
    Link,
}

impl Kind {
    pub const ALL: [Self; 10] = [
        Self::Text, Self::Heading, Self::Subheading, Self::Task, Self::Bullet,
        Self::Numbered, Self::Quote, Self::Code, Self::Divider, Self::Link,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Text => "Texto",
            Self::Heading => "Título",
            Self::Subheading => "Subtítulo",
            Self::Task => "Tarefa",
            Self::Bullet => "Lista",
            Self::Numbered => "Lista numerada",
            Self::Quote => "Citação",
            Self::Code => "Código",
            Self::Divider => "Divisor",
            Self::Link => "Link",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: Id,
    pub kind: Kind,
    pub text: String,
    pub checked: bool,
}

impl Block {
    pub fn new(id: Id, kind: Kind, text: impl Into<String>) -> Self {
        Self { id, kind, text: text.into(), checked: false }
    }

    fn markdown(&self) -> String {
        let text = &self.text;
        match self.kind {
            Kind::Heading => format!("## {text}"),
            Kind::Subheading => format!("### {text}"),
            Kind::Task => format!("- [{}] {text}", if self.checked { "x" } else { " " }),
            Kind::Bullet => format!("- {text}"),
            Kind::Numbered => format!("1. {text}"),
            Kind::Quote => format!("> {text}"),
            Kind::Code => format!("```\n{text}\n```"),
            Kind::Divider => "---".to_owned(),
            Kind::Text | Kind::Link => text.to_owned(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Status {
    #[default]
    Planned,
    Active,
    Done,
}

impl Status {
    pub const ALL: [Self; 3] = [Self::Planned, Self::Active, Self::Done];

    pub fn label(self) -> &'static str {
        match self {
            Self::Planned => "A fazer",
            Self::Active => "Em andamento",
            Self::Done => "Concluído",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageKind {
    #[default]
    Document,
    Collection,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollectionView {
    #[default]
    Table,
    Board,
    Category,
    Calendar,
    Gallery,
}

impl CollectionView {
    pub fn label(self) -> &'static str {
        match self {
            Self::Table => "Tabela",
            Self::Board => "Quadro",
            Self::Category => "Por categoria",
            Self::Calendar => "Calendário",
            Self::Gallery => "Cartões",
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CollectionOptions {
    pub view: CollectionView,
    pub status_filter: Option<Status>,
    pub sort_name: bool,
}

impl CollectionOptions {
    fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: Id,
    pub parent: Option<Id>,
    pub title: String,
    pub kind: PageKind,
    pub blocks: Vec<Block>,
    pub favorite: bool,
    pub trashed: bool,
    pub touched: u64,
    pub status: Status,
    pub tag: String,
    pub date: String,
    #[serde(default, skip_serializing_if = "CollectionOptions::is_default")]
    pub collection_options: CollectionOptions,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub collection_template: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub voted: bool,
}

impl Page {
    pub fn title(&self) -> &str {
        if self.title.trim().is_empty() { "Sem título" } else { self.title.trim() }
    }

    pub fn markdown(&self) -> String {
        let body = self.blocks.iter().map(Block::markdown).collect::<Vec<_>>().join("\n\n");
        format!("# {}\n\n{body}\n", self.title())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    #[default]
    Light,
    Dark,
    System,
}

impl Theme {
    pub fn label(self) -> &'static str {
        match self {
            Self::Light => "Claro",
            Self::Dark => "Escuro",
            Self::System => "Seguir o sistema",
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub name: String,
    pub theme: Theme,
    pub zoom: f32,
    pub compact: bool,
    pub restore_last: bool,
    pub show_sidebar: bool,
    pub show_favorites: bool,
    pub high_contrast: bool,
    pub wide_documents: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            name: "Meu espaço".to_owned(),
            theme: Theme::Light,
            zoom: 0.85,
            compact: false,
            restore_last: true,
            show_sidebar: true,
            show_favorites: true,
            high_contrast: false,
            wide_documents: false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub settings: Settings,
    pub pages: Vec<Page>,
    pub last_page: Option<Id>,
    next_id: Id,
    clock: u64,
}

impl Workspace {
    pub fn seeded() -> Self {
        let mut space = Self {
            settings: Settings::default(),
            pages: Vec::new(),
            last_page: None,
            next_id: 1,
            clock: 0,
        };
        let weekly = space.create(None, "Planejamento semanal", PageKind::Document);
        space.add_block(weekly, Kind::Text, "Organize as prioridades da semana e registre o que ficou para depois.");
        space.add_block(weekly, Kind::Heading, "Prioridades");
        space.add_block(weekly, Kind::Task, "Escolher três objetivos");
        space.add_block(weekly, Kind::Task, "Reservar tempo para revisar");
        space.page_mut(weekly).unwrap().favorite = true;
        let notes = space.create(None, "Caderno de ideias", PageKind::Document);
        space.add_block(notes, Kind::Heading, "Ideias em aberto");
        space.add_block(notes, Kind::Bullet, "Montar uma lista de leituras");
        let projects = space.create(None, "Projetos pessoais", PageKind::Collection);
        space.page_mut(projects).unwrap().favorite = true;
        for (title, tag, status) in [
            ("Organizar referências", "Pesquisa", Status::Active),
            ("Escrever o primeiro rascunho", "Escrita", Status::Planned),
            ("Definir o tema do mês", "Planejamento", Status::Done),
        ] {
            let id = space.create(Some(projects), title, PageKind::Document);
            let row = space.page_mut(id).unwrap();
            row.status = status;
            row.tag = tag.to_owned();
        }
        space.last_page = None;
        space
    }

    pub fn alloc(&mut self) -> Id {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn page(&self, id: Id) -> Option<&Page> {
        self.pages.iter().find(|page| page.id == id)
    }

    pub fn page_mut(&mut self, id: Id) -> Option<&mut Page> {
        self.pages.iter_mut().find(|page| page.id == id)
    }

    pub fn is_live(&self, id: Id) -> bool {
        let mut current = Some(id);
        for _ in 0..self.pages.len() {
            let Some(candidate) = current else { return true };
            let Some(page) = self.page(candidate) else { return false };
            if page.trashed { return false; }
            current = page.parent;
        }
        current.is_none()
    }

    pub fn touch(&mut self, id: Id) {
        self.clock += 1;
        let clock = self.clock;
        if let Some(page) = self.page_mut(id) { page.touched = clock; }
    }

    pub fn create(&mut self, parent: Option<Id>, title: &str, kind: PageKind) -> Id {
        let id = self.alloc();
        self.pages.push(Page {
            id, parent, title: title.to_owned(), kind, blocks: Vec::new(),
            favorite: false, trashed: false, touched: 0, status: Status::Planned,
            tag: String::new(), date: String::new(),
            collection_options: CollectionOptions::default(),
            collection_template: String::new(),
            properties: BTreeMap::new(),
            voted: false,
        });
        self.touch(id);
        id
    }

    pub fn add_block(&mut self, page: Id, kind: Kind, text: &str) -> Id {
        let id = self.alloc();
        self.page_mut(page).unwrap().blocks.push(Block::new(id, kind, text));
        self.touch(page);
        id
    }

    pub fn move_page(&mut self, id: Id, parent: Option<Id>) -> bool {
        if !self.is_live(id) || parent.is_some_and(|parent| !self.is_live(parent)) {
            return false;
        }
        let mut candidate = parent;
        while let Some(item) = candidate {
            if item == id { return false; }
            candidate = self.page(item).and_then(|page| page.parent);
        }
        self.page_mut(id).unwrap().parent = parent;
        self.touch(id);
        true
    }

    pub fn import_markdown(&mut self, title: &str, source: &str) -> Id {
        let id = self.create(None, title, PageKind::Document);
        let mut lines = source.lines().peekable();
        if let Some(first) = lines.peek().and_then(|line| line.strip_prefix("# ")) {
            self.page_mut(id).unwrap().title = first.to_owned();
            lines.next();
        }
        while let Some(line) = lines.next() {
            if line.trim().is_empty() { continue; }
            if line.starts_with("```") {
                let code = lines.by_ref().take_while(|next| !next.starts_with("```")).collect::<Vec<_>>().join("\n");
                self.add_block(id, Kind::Code, &code);
                continue;
            }
            if line.trim() == "---" {
                self.add_block(id, Kind::Divider, "");
                continue;
            }
            let variants = [
                ("### ", Kind::Subheading), ("## ", Kind::Heading), ("# ", Kind::Heading),
                ("- [x] ", Kind::Task), ("- [X] ", Kind::Task), ("- [ ] ", Kind::Task),
                ("- ", Kind::Bullet), ("1. ", Kind::Numbered), ("> ", Kind::Quote),
            ];
            let (kind, text) = variants.iter().find_map(|(prefix, kind)| line.strip_prefix(prefix).map(|text| (*kind, text)))
                .unwrap_or((Kind::Text, line));
            self.add_block(id, kind, text);
            if line.starts_with("- [x] ") || line.starts_with("- [X] ") {
                self.page_mut(id).unwrap().blocks.last_mut().unwrap().checked = true;
            }
        }
        id
    }
}

pub fn search_key(text: &str) -> String {
    text.to_lowercase().chars().map(|c| match c {
        'á' | 'à' | 'ã' | 'â' | 'ä' => 'a',
        'é' | 'ê' | 'è' | 'ë' => 'e',
        'í' | 'ì' | 'î' | 'ï' => 'i',
        'ó' | 'ò' | 'õ' | 'ô' | 'ö' => 'o',
        'ú' | 'ù' | 'û' | 'ü' => 'u',
        'ç' => 'c',
        _ => c,
    }).collect()
}
