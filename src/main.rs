use std::{env, fs, process};
use tree_sitter::{Node, Parser};

mod tests;

fn main() {
    let path = "mc_src/Blocks.java";

    let src = fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("Failed to read {}: {}", path, e);
        process::exit(1);
    });
    let src_bytes = src.as_bytes();

    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_java::LANGUAGE.into())
        .unwrap();
    let tree = parser.parse(&src, None).unwrap();
    let root = tree.root_node();

    let blocks = registered_blocks(root, src_bytes);
    for id in &blocks {
        println!("{}", id);
    }
}

pub fn registered_blocks(node: Node, src: &[u8]) -> Vec<String> {
    let mut blocks = Vec::new();
    scan_for_registers(node, src, &mut blocks);

    blocks
}

pub fn scan_for_registers(node: Node, src: &[u8], out: &mut Vec<String>) {
    let mut cur = node.walk();
    if node.kind() == "method_invocation" {
        let children: Vec<_> = node.named_children(&mut cur).collect();
        let is_register = children
            .iter()
            .any(|n| n.kind() == "identifier" && n.utf8_text(src).unwrap() == "register");
        if is_register {
            if let Some(arg_list) = children.iter().find(|n| n.kind() == "argument_list") {
                let mut ac = arg_list.walk();
                if let Some(first_arg) = arg_list.named_children(&mut ac).next() {
                    if let Some(s) = extract_string_literal(first_arg, src) {
                        out.push(s);
                    }
                }
            }
        }
    }
    let mut nxt = node.walk();
    for child in node.named_children(&mut nxt) {
        scan_for_registers(child, src, out);
    }
}

pub fn extract_string_literal(node: Node, src: &[u8]) -> Option<String> {
    match node.kind() {
        "string_literal" => {
            let s = node.utf8_text(src).ok()?;
            Some(s.trim_matches('"').to_string())
        }
        "cast_expression" | "parenthesized_expression" => {
            let mut cur = node.walk();
            for c in node.named_children(&mut cur) {
                if let Some(found) = extract_string_literal(c, src) {
                    return Some(found);
                }
            }
            None
        }
        // "BlockKeys.PUMPKIN"
        "field_access" => {
            let mut cur = node.walk();
            let children: Vec<_> = node.named_children(&mut cur).collect();

            if children.len() >= 2 && children[1].kind() == "identifier" {
                let field_name = children[1].utf8_text(src).ok()?;
                return Some(field_name.to_lowercase());
            }
            None
        }
        _ => None,
    }
}
