use std::borrow::Cow;
use std::time::Instant;

use completion::check_completion;
use dashmap::DashMap;
use diagnostics::run_diagnostics;
use document::DocumentData;
use goto::definition::check_goto_definition;
use goto::references::check_goto_references;
use log::info;
use serde::{Deserialize, Serialize};
use tower_lsp::jsonrpc::{Error, ErrorCode, Result};
use tower_lsp::lsp_types::notification::Notification;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use tree_sitter::{Node, Parser};

mod completion;
mod diagnostics;
mod document;
mod goto;
mod semantics;
mod documentation;
mod hover;
mod utils;
mod signature_help;

#[cfg(test)]
mod test_utils;

fn print_tree(node: Node,source:&[u8], depth: usize) {
    let node_type = node.kind();
    let node_text = node.utf8_text(source).unwrap();
    info!("{}{}: {}", "  ".repeat(depth), node_type, node_text);

    for i in 0..node.child_count() {
        let child = node.child(i).unwrap();
        print_tree(child, source, depth + 1);
    }
}

#[derive(Debug)]
struct Backend {
    client: Client,
    document_map: DashMap<String, DocumentData>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec!["#".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".to_string()]),
                    retrigger_characters: Some(vec![",".to_string()]),
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                }),
                hover_provider: Some(
                    HoverProviderCapability::Simple(true),
                ),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "initialized!")
            .await;

    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_change_workspace_folders(&self, _: DidChangeWorkspaceFoldersParams) {
        self.client
            .log_message(MessageType::INFO, "workspace folders changed!")
            .await;
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        self.client
            .log_message(MessageType::INFO, "configuration changed!")
            .await;
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        self.client
            .log_message(MessageType::INFO, "watched files have changed!")
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let time = Instant::now();

        // Use rope for an efficient way to access byte offsets and string slices
        let rope = ropey::Rope::from_str(&params.text_document.text);

        // Parse the document and save the parse tree in a hashmap
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_clingo::language())
            .expect("Error loading clingo grammar");

        let tree = parser
            .parse(params.text_document.text.clone(), None)
            .unwrap();

        print_tree(tree.root_node(), params.text_document.text.clone().as_bytes(),0);

        let mut doc = DocumentData::new(
            params.text_document.uri.clone(),
            tree,
            rope,
            params.text_document.version,
        );

        let duration = time.elapsed();
        info!(
            "Time needed for first time generating the document: {:?}",
            duration
        );
        doc.generate_semantics(None);
        self.document_map
            .insert(params.text_document.uri.to_string(), doc.clone());

        

        // Run diagnostics for that file
        let time = Instant::now();
        let diagnostics = run_diagnostics(doc, 100);
        self.client
            .publish_diagnostics(params.text_document.uri.clone(), diagnostics, Some(1))
            .await;
        let duration = time.elapsed();
        info!("Time needed for diagnostics: {:?}", duration);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let client_copy = self.client.clone();
        let uri = params.text_document.uri.clone().to_string();

        if !self.document_map.contains_key(&uri) {
            self.client
                .log_message(
                    MessageType::ERROR,
                    format!("Document {} changed before opening!", uri),
                )
                .await;
            return;
        }

        let mut document = self.document_map.get(&uri).unwrap().clone();

        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_clingo::language())
            .expect("Error loading clingo grammar");

        document.update_document(params.content_changes, &mut parser);
        let doc = document.clone();

        self.document_map.insert(uri, document);

        let time = Instant::now();
        let diagnostics = run_diagnostics(doc, 100);
        client_copy
            .publish_diagnostics(params.text_document.uri.clone(), diagnostics, Some(1))
            .await;
        let duration = time.elapsed();
        info!("Time needed for diagnostics: {:?}", duration);
    }

    async fn did_save(&self, _: DidSaveTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file saved!")
            .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_string();

        self.client
            .log_message(MessageType::INFO, "file closed!")
            .await;

        if !self.document_map.contains_key(&uri) {
            self.client
                .log_message(
                    MessageType::ERROR,
                    format!("Document {} closed before opening!", uri),
                )
                .await;
            return;
        }

        // Remove our information for this file
        self.document_map.remove(&uri);
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let completions = || -> Option<Vec<CompletionItem>> {
            let document = self.document_map.get(&uri.to_string())?;

            if let Some(context) = params.context {
                let mut trigger_character = "".to_string();
                if let Some(trigger) = context.trigger_character.clone() {
                    trigger_character = trigger;
                }

                return check_completion(document.value(), context, trigger_character, position);
            }

            Some(vec![])
        }();

        Ok(completions.map(CompletionResponse::Array))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        if let Some(document) = self.document_map.get(&uri.to_string()) {
            return Ok(Some(GotoDefinitionResponse::Array(
                check_goto_definition(document.value(), position).unwrap(),
            )));
        }

        Result::Err(tower_lsp::jsonrpc::Error::new(
            tower_lsp::jsonrpc::ErrorCode::InternalError,
        ))
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        if let Some(document) = self.document_map.get(&uri.to_string()) {
            return Ok(check_goto_references(document.value(), position));
        }

        Result::Err(tower_lsp::jsonrpc::Error::new(
            tower_lsp::jsonrpc::ErrorCode::InternalError,
        ))
    }

    async fn signature_help(&self, params: SignatureHelpParams)-> Result<Option<SignatureHelp>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let document = match self.document_map.get(&uri.to_string()) {
            Some(document) => document,
            None => return Err(Error {
                code:ErrorCode::InternalError,
                message:Cow::Owned("Document not found".to_string()),
                data:None,
            }),
        };
        
        Ok(signature_help::handle(&document, &params))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let document = match self.document_map.get(&uri.to_string()) {
            Some(document) => document,
            None => return Err(Error {
                code:ErrorCode::InternalError,
                message: Cow::Owned("Document not found".to_string()),
                data:None,
            }),
        };
        
        Ok(hover::handle(&document, &params))
    }
}
#[derive(Debug, Deserialize, Serialize)]
struct InlayHintParams {
    path: String,
}

enum CustomNotification {}
impl Notification for CustomNotification {
    type Params = InlayHintParams;
    const METHOD: &'static str = "custom/notification";
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend {
        client: client.clone(),
        document_map: DashMap::new(),
    })
    .finish();
    Server::new(stdin, stdout, socket).serve(service).await;
}
