use std::collections::HashMap;

use crate::ast::node::{Ast, Function, FunctionDeclaration, FunctionId, Node};

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
        let functions: HashMap<FunctionId, Function> = self
            .nodes
            .iter()
            .filter_map(|node| {
                if let Node::FunctionDeclaration(function_declaration) = node {
                    let function_declaration = function_declaration.clone();
                    Some((
                        function_declaration.id.clone(),
                        Function::CustomFunction {
                            id: function_declaration.id,
                            name: function_declaration.name,
                            parameters: function_declaration.parameters,
                            return_type: function_declaration.return_type,
                            body: function_declaration.body,
                        },
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
