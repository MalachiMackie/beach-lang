use crate::ast::node::Node;

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
}
