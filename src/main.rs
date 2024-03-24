use htmx_ls::backend::Backend;
use std::collections::HashMap;
use tokio::{
    io::{stdin, stdout},
    sync::Mutex,
};
use tower_lsp::{LspService, Server};
use tree_sitter::Parser;

#[tokio::main]
async fn main() {
    let stdin = stdin();
    let stdout = stdout();

    let mut html_parser = Parser::new();
    html_parser
        .set_language(tree_sitter_html::language())
        .expect("Error loading html grammar.");

    let documents = HashMap::new();

    let (service, socket) = LspService::new(|client| {
        Backend::new(client, Mutex::new(html_parser), Mutex::new(documents))
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}
