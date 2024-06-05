use log::info;
use tower_lsp::lsp_types::{SignatureHelp, SignatureHelpParams};

use crate::{document::DocumentData, documentation, utils};

pub fn handle(document:&DocumentData, params: &SignatureHelpParams) -> Option<SignatureHelp> {
    let position = params.text_document_position_params.position;
    info!("{:?}", position);
    let target_node = utils::node::from_position(&document, position)?;
    let atom_node = utils::node::get_atom(target_node)?;

    let arguments = documentation::Documentation::get_atom_arguments(&atom_node, &document.source.to_string());
    
    info!("args {:?}", arguments.is_none());
    let argument_position = utils::node::get_argument_position(target_node)?;
    info!("pos {:?}", argument_position);
    let arity = arguments.unwrap().len();

    None

    /*Ok(Some(SignatureHelp {
            signatures: vec![SignatureInformation {
                label: identifier.clone(),
                documentation: Some(Documentation::String(format!("DOCU: {}", identifier))),
                parameters: Some(parameters),
                active_parameter: Some(1),
            }],
            active_signature: Some(0),
            active_parameter: Some(0),
        })) */
}