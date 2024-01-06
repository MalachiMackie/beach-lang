use crate::ast::node::{Ast, ElseIfBlock, Expression, IfStatement, Node};

use super::{ast_builder::AstBuilder, expression_builder::ExpressionBuilder};

pub struct IfStatementBuilder {
    check_expression: Option<Expression>,
    body: Option<Ast>,
    else_if_blocks: Vec<(Expression, Ast)>,
    else_block: Option<Ast>,
}

impl IfStatementBuilder {
    pub fn new() -> Self {
        Self {
            body: None,
            check_expression: None,
            else_if_blocks: Vec::new(),
            else_block: None,
        }
    }

    pub fn check_expression(
        mut self,
        expression_fn: impl FnOnce(ExpressionBuilder) -> Expression,
    ) -> Self {
        let expression = expression_fn(ExpressionBuilder {});
        self.check_expression = Some(expression);

        self
    }

    pub fn body(mut self, body_fn: impl FnOnce(AstBuilder) -> Ast) -> Self {
        let body = body_fn(AstBuilder::default());
        self.body = Some(body);

        self
    }

    pub fn else_if(
        mut self,
        check_fn: impl Fn(ExpressionBuilder) -> Expression,
        body_fn: impl Fn(AstBuilder) -> Ast,
    ) -> Self {
        let check = check_fn(ExpressionBuilder {});
        let body = body_fn(AstBuilder::default());

        self.else_if_blocks.push((check, body));

        self
    }

    pub fn else_block(mut self, body_fn: impl Fn(AstBuilder) -> Ast) -> Self {
        let body = body_fn(AstBuilder::default());

        self.else_block = Some(body);

        self
    }

    pub fn build(self) -> Node {
        Node::IfStatement(IfStatement {
            check_expression: self.check_expression.expect("check expression to be set"),
            if_block: self.body.expect("body to be set").nodes,
            else_if_blocks: self
                .else_if_blocks
                .into_iter()
                .map(|(check, ast)| ElseIfBlock {
                    check,
                    block: ast.nodes,
                })
                .collect(),
            else_block: self.else_block.map(|ast| ast.nodes),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{
        builders::if_statement_builder::IfStatementBuilder,
        node::{ElseIfBlock, FunctionCall, FunctionId, IfStatement, Node},
    };

    #[test]
    fn if_statement() {
        let actual = IfStatementBuilder::new()
            .check_expression(|check| check.value_literal(true.into()))
            .body(|body| {
                body.statement(|statement| {
                    statement.function_call(|function_call| {
                        function_call
                            .function_id("my_function")
                            .no_parameters()
                            .build()
                    })
                })
                .build()
            })
            .build();

        let expected = Node::IfStatement(IfStatement {
            check_expression: true.into(),
            if_block: vec![Node::FunctionCall(FunctionCall {
                function_id: FunctionId("my_function".to_owned()),
                parameters: Vec::new(),
            })],
            else_block: None,
            else_if_blocks: Vec::new(),
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn else_if_statement() {
        let actual = IfStatementBuilder::new()
            .check_expression(|check| check.value_literal(true.into()))
            .body(|body| {
                body.statement(|statement| {
                    statement.function_call(|function_call| {
                        function_call
                            .function_id("my_function")
                            .no_parameters()
                            .build()
                    })
                })
                .build()
            })
            .else_if(
                |check| check.value_literal(true.into()),
                |body| {
                    body.statement(|statement| {
                        statement.function_call(|function_call| {
                            function_call
                                .function_id("my_function")
                                .no_parameters()
                                .build()
                        })
                    })
                    .build()
                },
            )
            .build();

        let expected = Node::IfStatement(IfStatement {
            check_expression: true.into(),
            if_block: vec![Node::FunctionCall(FunctionCall {
                function_id: FunctionId("my_function".to_owned()),
                parameters: Vec::new(),
            })],
            else_block: None,
            else_if_blocks: vec![ElseIfBlock {
                check: true.into(),
                block: vec![Node::FunctionCall(FunctionCall {
                    function_id: FunctionId("my_function".to_owned()),
                    parameters: Vec::new(),
                })],
            }],
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn else_statement() {
        let actual = IfStatementBuilder::new()
            .check_expression(|check| check.value_literal(true.into()))
            .body(|body| {
                body.statement(|statement| {
                    statement.function_call(|function_call| {
                        function_call
                            .function_id("my_function")
                            .no_parameters()
                            .build()
                    })
                })
                .build()
            })
            .else_block(|body| {
                body.statement(|statement| {
                    statement.function_call(|function_call| {
                        function_call
                            .function_id("my_function")
                            .no_parameters()
                            .build()
                    })
                })
                .build()
            })
            .build();

        let expected = Node::IfStatement(IfStatement {
            check_expression: true.into(),
            if_block: vec![Node::FunctionCall(FunctionCall {
                function_id: FunctionId("my_function".to_owned()),
                parameters: Vec::new(),
            })],
            else_block: Some(vec![Node::FunctionCall(FunctionCall {
                function_id: FunctionId("my_function".to_owned()),
                parameters: Vec::new(),
            })]),
            else_if_blocks: Vec::new(),
        });

        assert_eq!(actual, expected);
    }
}
