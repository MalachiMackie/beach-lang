use crate::ast::node::{Expression, FunctionCall, Node};

use super::{
    expression_builder::ExpressionBuilder, function_call_builder::FunctionCallBuilder,
    if_statement_builder::IfStatementBuilder,
    variable_declaration_builder::VariableDeclarationBuilder,
};

#[derive(Default)]
pub struct StatementBuilder {}

impl StatementBuilder {
    pub fn return_void(self) -> Node {
        Node::FunctionReturn { return_value: None }
    }

    pub fn var_declaration(
        self,
        var_declaration_fn: impl FnOnce(VariableDeclarationBuilder) -> Node,
    ) -> Node {
        var_declaration_fn(VariableDeclarationBuilder {
            var_name: None,
            var_type: None,
        })
    }

    pub fn if_statement(self, if_statement_fn: impl Fn(IfStatementBuilder) -> Node) -> Node {
        if_statement_fn(IfStatementBuilder::new())
    }

    pub fn function_call(
        self,
        function_call_fn: impl Fn(FunctionCallBuilder) -> FunctionCall,
    ) -> Node {
        Node::FunctionCall(function_call_fn(FunctionCallBuilder {
            function_id: None,
            parameters: None,
        }))
    }

    pub fn return_value(self, expression: impl Fn(ExpressionBuilder) -> Expression) -> Node {
        Node::FunctionReturn {
            return_value: Some(expression(ExpressionBuilder {})),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{
        builders::statement_builder::StatementBuilder,
        node::{FunctionCall, FunctionId, IfStatement, Node},
    };

    #[test]
    fn var_declaration() {
        let actual = StatementBuilder::default().var_declaration(|var_declaration| {
            var_declaration
                .infer_type()
                .name("my_var")
                .with_assignment(|assignment| assignment.value_literal(true.into()))
        });

        let expected = Node::VariableDeclaration {
            var_type: crate::ast::node::VariableDeclarationType::Infer,
            var_name: "my_var".to_owned(),
            value: true.into(),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn if_statement() {
        let actual = StatementBuilder::default().if_statement(|if_statement| {
            if_statement
                .check_expression(|check| check.value_literal(true.into()))
                .body(|body| body.build())
                .build()
        });

        let expected = Node::IfStatement(IfStatement {
            check_expression: true.into(),
            if_block: Vec::new(),
            else_if_blocks: Vec::new(),
            else_block: None,
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn function_call() {
        let actual = StatementBuilder::default().function_call(|function_call| {
            function_call
                .function_id("my_function")
                .no_parameters()
                .build()
        });

        let expected = Node::FunctionCall(FunctionCall {
            function_id: FunctionId("my_function".to_owned()),
            parameters: Vec::new(),
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn return_value() {
        let actual = StatementBuilder::default()
            .return_value(|return_value| return_value.value_literal(true.into()));

        let expected = Node::FunctionReturn {
            return_value: Some(true.into()),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn return_void() {
        let actual = StatementBuilder::default().return_void();

        let expected = Node::FunctionReturn { return_value: None };

        assert_eq!(actual, expected);
    }
}
