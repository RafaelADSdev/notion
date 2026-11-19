use crate::model::{CollectionView, Id, Kind, PageKind, Workspace};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Status,
    Tag,
    Date,
    Text,
    Choice(&'static [&'static str]),
    Vote,
}

#[derive(Clone, Copy)]
pub struct Property {
    pub label: &'static str,
    pub field: Field,
}

const fn property(label: &'static str, field: Field) -> Property {
    Property { label, field }
}

const fn text(label: &'static str) -> Property { property(label, Field::Text) }
const fn tag(label: &'static str) -> Property { property(label, Field::Tag) }
const fn date(label: &'static str) -> Property { property(label, Field::Date) }
const fn choice(label: &'static str, values: &'static [&'static str]) -> Property {
    property(label, Field::Choice(values))
}

const STATUS: Property = property("Status", Field::Status);
const PRIORITY: Property = choice("Prioridade", &["Alta", "Média", "Baixa"]);
const CATEGORY: Property = tag("Categoria");

pub struct Template {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub group: &'static str,
    pub view: CollectionView,
    pub title: &'static str,
    pub item: &'static str,
    pub properties: &'static [Property],
    pub sections: &'static [&'static str],
}

pub const DEFAULT_PROPERTIES: &[Property] = &[STATUS, tag("Etiqueta"), date("Data")];

pub static CATALOG: &[Template] = &[
    Template {
        id: "tarefas", name: "Controle de tarefas", group: "Dia a dia",
        description: "Planeje entregas, prazos e responsabilidades.",
        view: CollectionView::Table, title: "Tarefa", item: "tarefa",
        properties: &[STATUS, text("Responsável"), date("Prazo"), PRIORITY, text("Tipo de tarefa"), text("Descrição"), choice("Esforço", &["Pequeno", "Médio", "Grande"])],
        sections: &["Contexto", "Próximos passos"],
    },
    Template {
        id: "projetos", name: "Projetos", group: "Dia a dia",
        description: "Acompanhe cada projeto da ideia à entrega.",
        view: CollectionView::Board, title: "Projeto", item: "projeto",
        properties: &[STATUS, text("Responsável"), date("Entrega"), PRIORITY, text("Objetivo")],
        sections: &["Objetivo", "Etapas", "Riscos e dependências"],
    },
    Template {
        id: "reunioes", name: "Anotações de reunião", group: "Dia a dia",
        description: "Guarde pautas, decisões e próximos passos.",
        view: CollectionView::Table, title: "Reunião", item: "reunião",
        properties: &[date("Data"), text("Participantes")],
        sections: &["Pauta", "Decisões", "Próximos passos"],
    },
    Template {
        id: "documentos", name: "Central de documentos", group: "Dia a dia",
        description: "Reúna documentos e responsáveis em um só lugar.",
        view: CollectionView::Table, title: "Documento", item: "documento",
        properties: &[text("Autor"), date("Atualizado em"), tag("Categoria"), STATUS],
        sections: &["Resumo", "Conteúdo"],
    },
    Template {
        id: "ideias", name: "Sessão de brainstorming", group: "Dia a dia",
        description: "Registre ideias, agrupe por tema e marque suas favoritas.",
        view: CollectionView::Table, title: "Ideia", item: "ideia",
        properties: &[text("Criado por"), PRIORITY, CATEGORY, property("Apoio", Field::Vote)],
        sections: &["Ideia em detalhes", "Como testar"],
    },
    Template {
        id: "metas", name: "Controle de metas", group: "Dia a dia",
        description: "Defina objetivos e acompanhe o progresso.",
        view: CollectionView::Table, title: "Meta", item: "meta",
        properties: &[text("Responsável"), STATUS, date("Prazo"), text("Progresso"), text("Indicador")],
        sections: &["Resultado esperado", "Como medir"],
    },
    Template {
        id: "campanhas", name: "Gerenciamento de campanhas", group: "Marketing",
        description: "Organize campanhas, canais e datas de lançamento.",
        view: CollectionView::Board, title: "Campanha", item: "campanha",
        properties: &[STATUS, tag("Canal"), text("Responsável"), date("Lançamento"), text("Objetivo")],
        sections: &["Público e mensagem", "Entregas"],
    },
    Template {
        id: "conteudo", name: "Calendário de conteúdo", group: "Marketing",
        description: "Planeje conteúdos por data, canal e etapa de produção.",
        view: CollectionView::Calendar, title: "Conteúdo", item: "conteúdo",
        properties: &[date("Publicação"), tag("Canal"), STATUS, text("Formato"), text("Responsável")],
        sections: &["Pauta", "Rascunho"],
    },
    Template {
        id: "pipeline", name: "Acompanhamento do pipeline", group: "Operações",
        description: "Organize oportunidades e registre o próximo contato.",
        view: CollectionView::Board, title: "Oportunidade", item: "oportunidade",
        properties: &[STATUS, text("Empresa"), text("Valor estimado"), date("Próximo contato"), text("Responsável")],
        sections: &["Contexto", "Próximo contato"],
    },
    Template {
        id: "contas", name: "Gerenciamento de contas", group: "Operações",
        description: "Acompanhe clientes, contatos e renovações.",
        view: CollectionView::Board, title: "Conta", item: "conta",
        properties: &[STATUS, text("Contato"), text("Responsável"), date("Renovação"), tag("Segmento")],
        sections: &["Histórico", "Próximos passos"],
    },
    Template {
        id: "criativos", name: "Projetos criativos", group: "Marketing",
        description: "Organize briefs, produção, revisões e entregas.",
        view: CollectionView::Board, title: "Projeto criativo", item: "projeto criativo",
        properties: &[STATUS, text("Responsável"), tag("Formato"), date("Entrega")],
        sections: &["Briefing", "Referências", "Feedback"],
    },
    Template {
        id: "inventario", name: "Gerenciamento de inventário", group: "Operações",
        description: "Registre produtos, localização e quantidades.",
        view: CollectionView::Table, title: "Produto", item: "produto",
        properties: &[text("Código"), tag("Categoria"), text("Quantidade"), text("Localização"), date("Última contagem")],
        sections: &["Observações de estoque"],
    },
    Template {
        id: "roadmap", name: "Roadmap do produto", group: "Produto e tecnologia",
        description: "Organize iniciativas e acompanhe a previsão de entrega.",
        view: CollectionView::Board, title: "Iniciativa", item: "iniciativa",
        properties: &[STATUS, PRIORITY, date("Previsão"), text("Responsável"), tag("Área")],
        sections: &["Problema", "Resultado desejado", "Escopo"],
    },
    Template {
        id: "recursos", name: "Biblioteca de recursos", group: "Produto e tecnologia",
        description: "Encontre referências, arquivos e materiais de apoio.",
        view: CollectionView::Gallery, title: "Recurso", item: "recurso",
        properties: &[tag("Tipo"), text("Link"), text("Autor"), text("Descrição")],
        sections: &["Quando usar", "Referências"],
    },
    Template {
        id: "eventos", name: "Gerenciamento de eventos", group: "Operações",
        description: "Organize datas, locais e preparativos de cada evento.",
        view: CollectionView::Table, title: "Evento", item: "evento",
        properties: &[STATUS, date("Data do evento"), text("Local"), text("Responsável")],
        sections: &["Programação", "Logística"],
    },
    Template {
        id: "redes", name: "Planejamento de redes sociais", group: "Marketing",
        description: "Prepare publicações e distribua-as pelo calendário.",
        view: CollectionView::Calendar, title: "Publicação", item: "publicação",
        properties: &[date("Publicação"), tag("Rede social"), STATUS, text("Formato"), text("Link")],
        sections: &["Legenda", "Material de apoio"],
    },
    Template {
        id: "problemas", name: "Acompanhamento de problemas", group: "Produto e tecnologia",
        description: "Registre problemas, impacto e andamento da resolução.",
        view: CollectionView::Board, title: "Problema", item: "problema",
        properties: &[STATUS, PRIORITY, text("Responsável"), date("Prazo"), text("Impacto")],
        sections: &["Descrição", "Como reproduzir", "Resolução"],
    },
    Template {
        id: "solicitacoes", name: "Solicitações de recursos", group: "Produto e tecnologia",
        description: "Organize pedidos, avaliação e prazo de atendimento.",
        view: CollectionView::Board, title: "Solicitação", item: "solicitação",
        properties: &[STATUS, text("Solicitante"), tag("Tipo de recurso"), PRIORITY, date("Prazo")],
        sections: &["Necessidade", "Resultado esperado"],
    },
    Template {
        id: "engenharia", name: "Documentos de engenharia", group: "Produto e tecnologia",
        description: "Reúna propostas técnicas, revisões e decisões.",
        view: CollectionView::Table, title: "Documento técnico", item: "documento técnico",
        properties: &[text("Autor"), STATUS, text("Revisor"), date("Revisão"), tag("Área")],
        sections: &["Contexto", "Proposta", "Decisão"],
    },
    Template {
        id: "captacao", name: "Controle de captação de recursos", group: "Operações",
        description: "Acompanhe investidores, conversas e propostas.",
        view: CollectionView::Board, title: "Investidor", item: "investidor",
        properties: &[STATUS, text("Contato"), text("Valor potencial"), date("Próximo contato"), text("Responsável")],
        sections: &["Histórico de contato", "Próxima ação"],
    },
];

pub fn find(id: &str) -> Option<&'static Template> {
    CATALOG.iter().find(|template| template.id == id)
}

impl Template {
    pub fn create(&self, space: &mut Workspace) -> Id {
        let id = space.create(None, self.name, PageKind::Collection);
        let page = space.page_mut(id).unwrap();
        page.collection_template = self.id.to_owned();
        page.collection_options.view = self.view;
        id
    }

    pub fn prepare_item(&self, space: &mut Workspace, id: Id) {
        for section in self.sections {
            space.add_block(id, Kind::Heading, section);
            space.add_block(id, Kind::Text, "");
        }
    }

    pub fn has(&self, field: Field) -> bool {
        self.properties.iter().any(|property| property.field == field)
    }
}
