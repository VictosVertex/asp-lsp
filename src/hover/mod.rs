use tower_lsp::lsp_types::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind};
use crate::{document::DocumentData, documentation, utils};

pub fn handle(document:&DocumentData, params:&HoverParams) -> Option<Hover> {
    let position = params.text_document_position_params.position;
    let target_node = utils::node::from_position(&document, position)?;
    let atom_node = utils::node::get_atom(target_node)?;

    let arguments = documentation::Documentation::get_atom_arguments(&atom_node, &document.source.to_string())?;
    let argument_position = utils::node::get_argument_position(target_node);
    let arity = arguments.len();
    let identifier = document.get_source_for_range(atom_node.child(0).unwrap().range());

    let documentation = document.documentation.predicates.get(&(identifier.clone(), arity))?;

    let doc_string = if let Some(arg_position) = argument_position {
        let index = arity - 1 - arg_position;
        let argument = &documentation.arguments[index];
        format!("`{}` - {}",argument.identifier, argument.description)
    } else {
        let parameters: String = documentation.arguments.iter()
        .map(|arg| format!(" - `{}` - {}", arg.identifier, arg.description))
        .collect::<Vec<String>>()
        .join("\n");


        format!("```\n{}\n```\n\n{}\n\n### Parameters\n\n{}", 
            documentation.signature,
            documentation.description, 
            parameters)
    };

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: doc_string
        }),
        range: None
    })
}