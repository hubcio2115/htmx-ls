use std::collections::HashMap;
use tokio::io::{stdin, stdout};
use tokio::sync::Mutex;
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
    documents: Mutex<HashMap<Url, Document>>,
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

        let cst = self.html_parser.lock().await.parse(text, None).unwrap();

        self.documents
            .lock()
            .await
            .insert(uri, Document::new(cst, text.to_owned()));
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file changed!")
            .await;

        let uri = params.text_document.uri;
        let text_document = &params.content_changes[0];

        let cst = self
            .html_parser
            .lock()
            .await
            .parse(&text_document.text, None)
            .unwrap();

        self.documents
            .lock()
            .await
            .insert(uri, Document::new(cst, text_document.text.to_owned()));
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        self.client
            .log_message(MessageType::INFO, "hovering!")
            .await;

        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        match self.documents.lock().await.get(&uri) {
            Some(document) => {
                let node = get_node_on_position(document, position).unwrap();
                let node_as_text = node_to_text(&node, document.text.as_str());

                Ok(get_docs_for_attribute(node_as_text).map(|docs| Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: docs.desc.to_string(),
                    }),
                    range: None,
                }))
            }
            None => Err(Error::parse_error()),
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

    let documents = HashMap::new();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        html_parser: Mutex::new(html_parser),
        documents: Mutex::new(documents),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
