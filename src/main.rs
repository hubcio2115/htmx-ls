use dashmap::DashMap;
use std::ops::Deref;
use std::sync::Mutex;
use tokio::io::{stdin, stdout};
use tower_lsp::jsonrpc::{Error, Result};
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tree_sitter::Parser;
mod constants;
mod utils;

use utils::{get_docs_for_attribute, get_node_on_position, node_to_text, Document};

struct Backend {
    client: Client,
    html_parser: Mutex<Parser>,
    documents: DashMap<Url, Document>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec!["hx-".to_string()]),
                    ..Default::default()
                }),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;

        let uri = params.text_document.uri;
        let text = &params.text_document.text;

        let cst = self.html_parser.lock().unwrap().parse(text, None).unwrap();

        self.documents
            .insert(uri, Document::new(cst, text.to_owned()));
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file changed!")
            .await;

        let uri = params.text_document.uri;
        let text = &params.content_changes[0].text;

        let cst = self.html_parser.lock().unwrap().parse(text, None).unwrap();

        self.documents
            .insert(uri, Document::new(cst, text.to_owned()));
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        self.client
            .log_message(MessageType::INFO, "hovering!")
            .await;

        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "position: line: {}, character: {}",
                    position.line, position.character
                ),
            )
            .await;

        let Some(document) = self.documents.get(&uri) else {
            return Err(Error::parse_error());
        };

        let node = get_node_on_position(document.deref(), position).unwrap();
        let node_as_text = node_to_text(&node, document.deref().text.as_str());

        match get_docs_for_attribute(node_as_text) {
            Some(docs) => Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: docs.desc.to_string(),
                }),
                range: None,
            })),
            None => Ok(None),
        }
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let stdin = stdin();
    let stdout = stdout();

    let mut html_parser = Parser::new();
    html_parser
        .set_language(tree_sitter_html::language())
        .expect("Error loading html grammar.");

    let documents = DashMap::new();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        html_parser: Mutex::new(html_parser),
        documents,
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
