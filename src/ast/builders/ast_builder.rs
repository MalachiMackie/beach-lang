use std::collections::HashMap;

use crate::ast::node::{Ast, Expression, Function, FunctionId, Node};

use super::{
    expression_builder::ExpressionBuilder,
    function_declaration_builder::FunctionDeclarationBuilder,
    if_statement_builder::IfStatementBuilder, statement_builder::FunctionCallBuilder,
    variable_declaration_builder::VariableDeclarationBuilder,
};

#[derive(Debug, PartialEq)]
pub struct AstBuilder {
    pub(super) nodes: Vec<Node>,
}

impl AstBuilder {
    pub fn new() -> Self {
        AstBuilder { nodes: Vec::new() }
    }

    pub fn var_declaration(
        mut self,
        var_declaration_fn: impl Fn(VariableDeclarationBuilder) -> Node,
    ) -> AstBuilder {
        let var_declaration_node = var_declaration_fn(VariableDeclarationBuilder {
            var_name: None,
            var_type: None,
        });

        self.nodes.push(var_declaration_node);

        self
    }

    pub fn if_statement(mut self, if_statement_fn: impl Fn(IfStatementBuilder) -> Node) -> Self {
        let if_statement = if_statement_fn(IfStatementBuilder::new());

        self.nodes.push(if_statement);

        self
    }

    pub fn function_call(
        mut self,
        function_call_fn: impl Fn(FunctionCallBuilder) -> FunctionCallBuilder,
    ) -> AstBuilder {
        let function_call_builder = function_call_fn(FunctionCallBuilder {
            function_id: None,
            parameters: None,
        });

        self.nodes.push(Node::FunctionCall {
            function_id: function_call_builder
                .function_id
                .expect("function id to be set"),
            parameters: function_call_builder
                .parameters
                .expect("parameters to be set"),
        });

        self
    }

    pub fn return_value(
        mut self,
        expression: impl Fn(ExpressionBuilder) -> Expression,
    ) -> AstBuilder {
        self.nodes.push(Node::FunctionReturn {
            return_value: Some(expression(ExpressionBuilder {})),
        });
        self
    }

    pub fn function_declaration(
        mut self,
        function_declaration_fn: impl Fn(FunctionDeclarationBuilder) -> Node,
    ) -> AstBuilder {
        let function_declaration = function_declaration_fn(FunctionDeclarationBuilder {
            id: None,
            body: None,
            name: None,
            parameters: None,
            return_type: None,
        });

        self.nodes.push(function_declaration);

        self
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
