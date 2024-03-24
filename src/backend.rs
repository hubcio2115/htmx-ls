use std::collections::HashMap;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::{Error, Result};
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use tree_sitter::Parser;

use crate::utils::{get_docs_for_attribute, get_node_on_position, node_to_text, Document};

pub struct Backend {
    client: Client,
    html_parser: Mutex<Parser>,
    documents: Mutex<HashMap<Url, Document>>,
}

impl Backend {
    pub fn new(
        client: Client,
        html_parser: Mutex<Parser>,
        documents: Mutex<HashMap<Url, Document>>,
    ) -> Self {
        Self {
            client,
            html_parser,
            documents,
        }
    }
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

#[cfg(test)]
mod test {
    use std::path::Path;

    use super::*;
    use tower_lsp::LspService;
    use tree_sitter::Parser;

    fn initialize_lsp_service() -> (LspService<Backend>, tower_lsp::ClientSocket) {
        let mut html_parser = Parser::new();
        html_parser
            .set_language(tree_sitter_html::language())
            .expect("Error loading html grammar.");

        let documents = HashMap::new();

        // This sets up a `Client` and `ClientSocket` pair that allow for
        // manual testing of the server.
        //
        // The returned tuple is of type `(Arc<Mock>, ClientSocket)`.
        LspService::new(|client| {
            Backend::new(client, Mutex::new(html_parser), Mutex::new(documents))
        })
    }

    #[tokio::test(flavor = "current_thread")]
    async fn server_initializes_correctly() {
        let (service, _) = initialize_lsp_service();

        // Call `LanguageServer` methods directly, examine internal state, etc.
        assert!(service
            .inner()
            .initialize(InitializeParams::default())
            .await
            .is_ok());
    }

    const HTML_FILE: &str = r#"
        <!doctype html>
        <html lang="en">

        <head>
          <meta charset="UTF-8" />
          <meta name="viewport" content="width=device-width, initial-scale=1" />
          <script src="https://unpkg.com/htmx.org@1.9.10"
            integrity="sha384-D1Kt99CQMDuVetoL1lrYwg5t+9QdHe7NLX/SoJYkXDFfX37iInKRy5xLSi8nO7UC"
            crossorigin="anonymous"></script>
          <title>Htmx test</title>
        </head>

        <body>
            <div hx-post="test"></div>
        </body>

        </html>
    "#;

    #[tokio::test(flavor = "current_thread")]
    async fn server_returns_correct_hover_information() {
        let (service, _) = initialize_lsp_service();

        let document_url =
            Url::from_file_path(Path::new("/home/hkowalski/Documents/Projects/htmx-ls")).unwrap();

        let _ = service
            .inner()
            .did_open(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    language_id: "html".to_string(),
                    uri: document_url.clone(),
                    version: 0,
                    text: HTML_FILE.to_string(),
                },
            })
            .await;

        let result = service
            .inner()
            .hover(HoverParams {
                text_document_position_params: TextDocumentPositionParams {
                    position: Position {
                        line: 14,
                        character: 17,
                    },
                    text_document: TextDocumentIdentifier {
                        uri: document_url.clone(),
                    },
                },
                work_done_progress_params: WorkDoneProgressParams::default(),
            })
            .await
            .ok()
            .unwrap()
            .unwrap();

        let test_contents = HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: include_str!("./htmx/attributes/hx-post.md").to_string(),
        });

        assert_eq!(result.contents, test_contents);
    }
}
