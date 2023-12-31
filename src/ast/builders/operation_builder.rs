use crate::ast::node::{BinaryOperation, Expression, Operation, UnaryOperation};

use super::expression_builder::ExpressionBuilder;

pub struct OperationBuilder {}

impl OperationBuilder {
    pub fn not<TExpressionFn: Fn(ExpressionBuilder) -> Expression>(
        self,
        expression_fn: TExpressionFn,
    ) -> Operation {
        Operation::Unary {
            operation: UnaryOperation::Not,
            value: Box::new(expression_fn(ExpressionBuilder {})),
        }
    }

    pub fn greater_than(
        self,
        left_fn: impl Fn(ExpressionBuilder) -> Expression,
        right_fn: impl Fn(ExpressionBuilder) -> Expression,
    ) -> Operation {
        Operation::Binary {
            operation: BinaryOperation::GreaterThan,
            left: Box::new(left_fn(ExpressionBuilder {})),
            right: Box::new(right_fn(ExpressionBuilder {})),
        }
    }

    pub fn plus(
        self,
        left_fn: impl Fn(ExpressionBuilder) -> Expression,
        right_fn: impl Fn(ExpressionBuilder) -> Expression,
    ) -> Operation {
        Operation::Binary {
            operation: BinaryOperation::Plus,
            left: Box::new(left_fn(ExpressionBuilder {})),
            right: Box::new(right_fn(ExpressionBuilder {})),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{
        builders::ast_builder::AstBuilder,
        node::{BoolValue, Node, Value, VariableDeclarationType},
    };

    use super::*;

    #[test]
    fn add_operation() {
        let result = AstBuilder::new().var_declaration(|var_declaration_builder| {
            var_declaration_builder
                .infer_type()
                .name("my_var")
                .with_assignment(|expression_builder| {
                    expression_builder.operation(|operation_builder| {
                        operation_builder.not(|not_expression_builder| {
                            not_expression_builder.value_literal(Value::Boolean(BoolValue(true)))
                        })
                    })
                })
        });

        let expected = AstBuilder {
            nodes: vec![Node::VariableDeclaration {
                var_type: VariableDeclarationType::Infer,
                var_name: "my_var".to_owned(),
                value: Expression::Operation(Operation::Unary {
                    operation: UnaryOperation::Not,
                    value: Box::new(Expression::ValueLiteral(Value::Boolean(BoolValue(true)))),
                }),
            }],
        };

        assert_eq!(result, expected);
    }
}
