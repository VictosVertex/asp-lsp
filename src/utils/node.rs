use tower_lsp::lsp_types::Position;
use tree_sitter::{Node, Point};

use crate::document::DocumentData;

pub fn from_position(document:&DocumentData, position: Position) -> Option<Node> {
    document.tree.root_node().descendant_for_point_range(
        Point {
            row: position.line as usize,
            column: (position.character) as usize,
        },
        Point {
            row: position.line as usize,
            column: (position.character) as usize,
        },
    )
}

pub fn get_atom(mut node:Node) -> Option<Node> {
    while let Some(parent) = node.parent() {
        if parent.kind() == "atom" {
            return Some(node.parent().unwrap())
        }

        node = parent
    }
    None
}

pub fn get_argument_position(mut node:Node) -> Option<usize> {
    let mut count = 0;
    while let Some(parent) = node.parent() {
        if parent.kind() == "atom" {
            if count < 3 {
                return None
            }
            count -= 3;
            break;
        }

        if parent.kind() == "term" {
            count = 0;
        }

        node = parent;
        count += 1;
    }

    Some(count) 
}