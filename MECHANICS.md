# Comportamento do Margem

## Evidências do aplicativo de referência

- O ponto de entrada nativo do executável Windows está em `0x1431cd920`. A triagem ev_b54cc60c167414eb92e3013b3addfad78b63361aea875ef3ae567b27a0107551 identifica um PE x86-64 com símbolos removidos. A solicitação de pseudocódigo desse endereço atingiu o limite de 300 segundos. O pseudocódigo de `sub_140511020`, identificado pela referência nativa em `0x140512131`, atingiu o mesmo limite. Nenhuma constante de edição ou interface foi atribuída a esses procedimentos.
- A busca nativa localizou o nome da API de configuração do corretor ortográfico em `0x14be567d8`. A referência em `0x14be567f8` aponta para dados, sem função identificada. Isso confirma o nome exportado pelo motor Electron. A correção ortográfica é uma capacidade do original que permanece fora desta implementação.
- A captura estática ev_69f2b6caf17757366aa0290fa6458d941ce479c9db343e5c4e437072e43954fb identifica `D.attachTab`, `D.closeActiveTab`, `D.handleAppCommandNavigation` e a ponte `__electronApi.onOpenSearch`. O Margem mantém abas de páginas, histórico de navegação e busca por título e conteúdo. As posições JavaScript são módulo 10019 de `.webpack/main/index.js` para os métodos `D.*` e as entradas de ponte no mapa do ASAR. Esses métodos não têm endereços de procedimento nativo estabelecidos.
- A documentação oficial descreve o menu `/`, transformação e reorganização de blocos. No Margem, a tecla abre opções em um bloco vazio e a ordem muda pelo menu de cada bloco. [Blocos](https://www.notion.com/help/what-is-a-block)
- A documentação oficial descreve itens de coleção que abrem como páginas, propriedades, visualizações, filtros e ordenação. No Margem, a mesma coleção tem tabela e quadro, com propriedades de status, etiqueta e data. [Bancos de dados](https://www.notion.com/help/intro-to-databases)

## Valores estimados

A busca de strings e o mapa estático localizaram a infraestrutura do desktop. Os valores originais de tema e zoom ficam sem confirmação porque a captura não contém as preferências executadas. O Margem começa com tema claro e fator de escala 1,0, ambos configuráveis. Cores, tamanhos de janela, dados de exemplo e rótulos fazem parte do design próprio.

## Alcance

O plano escolhido é uso pessoal local. Contas, permissões compartilhadas, sincronização, integrações, automações, serviços de IA e outros comportamentos do serviço online permanecem sem paridade. Importação e exportação lidam com os blocos Markdown suportados pelo editor. O pacote instalado não estabeleceu fórmulas de banco de dados, regras de autorização nem comportamento de sincronização.
