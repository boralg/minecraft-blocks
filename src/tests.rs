use std::fs;

use tree_sitter::{Node, Parser};

use crate::registered_blocks;

#[test]
fn captures_all_blocks() {
    let path = "mc_src/Blocks.java";

    let src = fs::read_to_string(&path).unwrap();
    let src_bytes = src.as_bytes();

    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_java::LANGUAGE.into())
        .unwrap();
    let tree = parser.parse(&src, None).unwrap();
    let root = tree.root_node();

    let blocks = registered_blocks(root, src_bytes);

    let mut block_decls = block_declarations(root, src_bytes);
    block_decls.iter_mut().for_each(|s| *s = s.to_lowercase());

    let missing: Vec<_> = block_decls
        .iter()
        .filter(|decl| !blocks.contains(decl))
        .collect();

    assert!(missing.is_empty(), "Missing blocks: {:?}", missing);
}

fn block_declarations(node: Node, src: &[u8]) -> Vec<String> {
    let mut blocks = Vec::new();
    extract_block_declarations(node, src, &mut blocks);

    blocks
}

fn extract_block_declarations(node: Node, src: &[u8], out: &mut Vec<String>) {
    let mut cur = node.walk();
    //println!("{}", node.kind());

    if node.kind() == "field_declaration" {
        let mut is_block = false;
        let mut name = "";

        for child in node.named_children(&mut cur) {
            match child.kind() {
                "type_identifier" => {
                    if let Ok(typ) = child.utf8_text(src) {
                        if typ == "Block" {
                            is_block = true;
                        }
                    }
                }
                "variable_declarator" => {
                    if let Some(idn) = child
                        .named_children(&mut child.walk())
                        .find(|n| n.kind() == "identifier")
                    {
                        name = idn.utf8_text(src).unwrap();
                    }
                }
                _ => {}
            }
        }

        if is_block {
            out.push(name.to_owned());
        }
    }

    let mut nxt = node.walk();
    for child in node.named_children(&mut nxt) {
        extract_block_declarations(child, src, out);
    }
}
