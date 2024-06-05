use std::collections::VecDeque;

use dashmap::DashMap;
use log::info;
use tree_sitter::{Node, Parser};

use crate::DocumentData;

#[derive(Debug, Clone)]
pub struct Documentation {
    pub predicates: DashMap<(String, usize), PredicateDocumentation>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PredicateDocumentation {
    pub signature: String,
    pub description: String,
    pub arguments: Vec<ArgumentDocumentation>
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ArgumentDocumentation {
    pub identifier: String,
    pub description: String,
}

impl Documentation {
    pub fn new() -> Documentation {
        Documentation {
            predicates: DashMap::new(),
        }
    }

    pub fn startup(document: &mut crate::document::DocumentData) {
        document.documentation = Documentation::new();
    }
    
    pub fn on_node(node: Node, document: &mut DocumentData) {

        match node.kind() {
            "multi_comment"  => {
                let mut comment = document.get_source_for_range(node.range());

                if comment.len() > 4 {
                    comment = comment[2..comment.len() - 2].trim().to_string()
                } else {
                    comment.clear()
                };


                // Doc comments have to be indicated by a leading '#'.
                if !comment.starts_with("#") {
                    return;
                }

                // The '#' has to be followed by the predicate signature this documentation is for
                // this includes the dot closing the statement.              
                let signature_end = match comment.find(".") {
                    Some(index) => index,
                    None => return
                };

                let signature = &comment[1..signature_end+1];
                let descriptions = &comment[signature_end+1..];

                //Parse signature for analysis
                let tree = {
                    let mut parser = Parser::new();
                    parser
                        .set_language(tree_sitter_clingo::language())
                        .expect("Error loading clingo grammar");
                    parser
                        .parse(signature, None)
                        .unwrap()
                };

                let mut atom_node = tree.root_node();

                while atom_node.kind() != "atom" {
                    if atom_node.child_count() == 0 {
                        break;
                    }
                    atom_node = atom_node.child(0).unwrap()
                }

                
                //print_tree(tree.root_node(), comment_sections.as_bytes(), 0);
                let identifier = Documentation::get_atom_identifier(&atom_node, signature);
                let arguments = Documentation::get_atom_arguments(&atom_node, signature);


                if identifier.is_none() || arguments.is_none() {
                    return;
                }
                
                if Documentation::has_error(&atom_node) {
                    return;
                }

                let parameters_begin = descriptions.find("#parameters");

                let description = match parameters_begin {
                    Some(index) => &descriptions[..index],
                    None => &descriptions
                }.trim();
                

                let argument_descriptions = Documentation::get_argument_descriptions(&descriptions);

                let sig = if arguments.clone().unwrap().len() > 0 {
                    format!("{}({}).", identifier.clone().unwrap(), arguments.clone().unwrap().join(","))
                } else {
                    format!("{}.", identifier.clone().unwrap())
                };

                Documentation::insert_predicate_documentation(&document.documentation,
                    &identifier.unwrap(),
                    &sig,
                    description,
                    &arguments.unwrap(),
                    &argument_descriptions);

            }
            _ => {}
        }
    }

    fn insert_predicate_documentation(
        documentation: &Documentation,
        identifier: &str,
        signature: &str,
        description: &str,
        arguments: &Vec<String>, 
        argument_descriptions: &DashMap<String, String>) {
        
        let arg_docu: Vec<ArgumentDocumentation> = arguments.iter().filter_map(|arg| {
            argument_descriptions.get(arg).map(|desc| {
                ArgumentDocumentation {
                    identifier: arg.clone(),
                    description: desc.clone(),
                }
            })
        }).collect();

        let predicate_documentation = PredicateDocumentation {
            signature: signature.to_string(),
            description: description.to_string(),
            arguments: arg_docu
        };

        let arity = arguments.len();

        documentation.predicates.insert((identifier.to_string(), arity), predicate_documentation);
    }

    fn get_atom_identifier(node:&Node, source:&str) -> Option<String> {
        if node.kind() != "atom" {
            return None;
        }

        node.child(0)?
            .utf8_text(source.as_bytes())
            .ok()
            .map(|text| text.to_string())
    }

    pub fn has_error(node:&Node) -> bool {
        if node.kind() != "atom" {
            return true;
        }

        let mut queue = VecDeque::<Node>::new();
        let mut cursor = node.walk();
        queue.push_back(node.clone());


        while queue.len() > 0 {
            let current = queue.pop_front().unwrap();

            if current.kind() == "ERROR" {
                return true;
            }

            current.children(&mut cursor)
                    .for_each(|child| {queue.push_back(child)});
        }

        false
    }

    pub fn get_atom_arguments(node:&Node, source:&str) -> Option<Vec<String>> {
        if node.kind() != "atom" {
            return None;
        }

        let mut arguments = Vec::new();

        if node.child_count() == 1 || node.child_count() == 3 {
            // Either only identifier or identifier with two parentheses
            // it therefore has 0 arguments.
            return Some(arguments);
        }

        let argvec = node.child(2)?;
        let mut termvec = argvec.child(0)?;
        

        while termvec.kind() == "termvec" {
            let child_index = if termvec.child_count() == 1 {0} else {2};

            if let Some(term_node) = termvec.child(child_index) {
                if let Ok(term_text) = term_node.utf8_text(source.as_bytes()) {
                    arguments.push(term_text.to_string());
                }
            }

            termvec = termvec.child(0)?;
        }

        arguments.reverse();

        Some(arguments)
    }

    fn get_argument_descriptions(input: &str) -> DashMap<String, String> {
        input.lines()
            .map(str::trim_start)
            .skip_while(|line| !line.starts_with("#parameters"))
            .skip(1)
            .filter_map(|line| {
                let parts = line.split_once(':');

                if parts.is_some() {
                    Some((parts.unwrap().0.trim().to_string(), parts.unwrap().1.trim().to_string()))
                } else {
                    None
                }
            })
            .collect()
    }
}