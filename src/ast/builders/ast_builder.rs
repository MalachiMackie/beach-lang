use std::collections::HashMap;

use crate::ast::node::{Ast, FunctionDeclaration, FunctionId, Node};

use super::{
    function_declaration_builder::FunctionDeclarationBuilder, statement_builder::StatementBuilder,
};

#[derive(Debug, PartialEq)]
pub struct AstBuilder {
    pub(super) nodes: Vec<Node>,
}

impl AstBuilder {
    pub fn new() -> Self {
        AstBuilder { nodes: Vec::new() }
    }

    pub fn statement(self) -> StatementBuilder {
        StatementBuilder { builder: self }
    }

    pub fn function_declaration(self) -> FunctionDeclarationBuilder {
        FunctionDeclarationBuilder {
            id: None,
            builder: self,
            body: None,
            name: None,
            parameters: None,
            return_type: None,
        }
    }

    pub fn build(self) -> Ast {
        let functions: HashMap<FunctionId, FunctionDeclaration> = self
            .nodes
            .iter()
            .filter_map(|node| {
                if let Node::FunctionDeclaration(function_declaration) = node {
                    Some((
                        function_declaration.id.clone(),
                        function_declaration.to_owned(),
                    ))
                } else {
                    None
                }
            })
            .collect();

        Ast {
            functions,
            nodes: self.nodes,
        }
    }
}
