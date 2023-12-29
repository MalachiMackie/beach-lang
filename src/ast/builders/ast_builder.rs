use crate::ast::node::{Node, Value};

use super::{
    function_declaration_builder::FunctionDeclarationBuilder, operation_builder::OperationBuilder,
    statement_builder::StatementBuilder,
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

    pub fn literal(mut self, value: Value) -> Self {
        // todo: this should be in an `ExpressionBuilder`
        self.nodes.push(Node::Literal { value });
        self
    }

    pub fn operation(self) -> OperationBuilder {
        // todo: this should be in an `ExpressionBuilder`
        OperationBuilder { builder: self }
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

#[cfg(test)]
mod tests {
    use crate::ast::node::UIntValue;

    use super::*;

    #[test]
    fn add_literal() {
        let result = AstBuilder::new().literal(Value::UInt(UIntValue(13)));

        let expected = AstBuilder {
            nodes: vec![Node::Literal {
                value: Value::UInt(UIntValue(13)),
            }],
        };

        assert_eq!(result, expected);
    }
}
