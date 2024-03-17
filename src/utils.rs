use tower_lsp::lsp_types::Position;
use tree_sitter::{Node, Point, Tree};
use crate::constants::{HxCompletion, HX_TAGS};

pub struct Document {
    pub cst: Tree,
    pub text: String,
}

impl Document {
    pub fn new(cst: Tree, text: String) -> Self {
        Self { cst, text }
    }
}

pub fn get_node_on_position(document: &Document, position: Position) -> Option<Node> {
    let point = Point::new(position.line as usize, position.character as usize);

    return document
        .cst
        .root_node()
        .named_descendant_for_point_range(point, point);
}

pub fn node_to_text<'a>(node: &Node<'_>, source: &'a str) -> &'a str {
    return node
        .utf8_text(source.as_bytes())
        .expect("getting text should never fail");
}

pub fn get_docs_for_attribute(attribute: &str) -> Option<&'static HxCompletion> {
    HX_TAGS.iter().find(|x| x.name == attribute)
}

#[cfg(test)]
mod test {
    use super::*;
    use tree_sitter::Parser;

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

    #[test]
    fn get_node_on_position_works() {
        let mut html_parser = Parser::new();
        html_parser
            .set_language(tree_sitter_html::language())
            .expect("Error loading html grammar.");

        let cst = html_parser.parse(HTML_FILE, None).unwrap();

        let document = Document::new(cst, HTML_FILE.to_string());

        let node = get_node_on_position(
            &document,
            Position {
                line: 14,
                character: 17,
            },
        )
        .unwrap();

        assert_eq!(node_to_text(&node, HTML_FILE), "hx-post");
    }
}
