use rowan::NodeOrToken;
use yaml_parser::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};

pub fn to_string(element: SyntaxElement, out: &mut String) {
    //let kind = element.kind();

    match element {
        NodeOrToken::Node(node) => {
            for child in node.children_with_tokens() {
                to_string(child, out);
            }
        }
        NodeOrToken::Token(token) => {
            out.push_str(token.text());
        }
    }
}

//use yaml_parser::ast::AstNode;
//let yaml_raw = std::fs::read_to_string("kube_config").unwrap();
//let parsed_yaml = yaml_parser::parse(&yaml_raw).unwrap();

//let mut output = string::new();
//yaml_writer::to_string(parsed_yaml.clone().into(), &mut output);

//println!("{output}");

//let ast = yaml_parser::ast::root::cast(parsed_yaml);

//if let some(ast) = ast {
//println!("{:#?}", ast.documents().next().unwrap().block().unwrap());
//}
