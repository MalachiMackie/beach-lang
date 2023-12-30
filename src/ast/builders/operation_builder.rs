use crate::ast::node::{Expression, Operation, UnaryOperation};

use super::expression_builder::ExpressionBuilder;

pub struct OperationBuilder {}

impl OperationBuilder {
    pub fn not<TExpressionFn: Fn(ExpressionBuilder) -> Expression>(
        self,
        expression_fn: TExpressionFn,
    ) -> Operation {
        Operation::Unary(UnaryOperation::Not {
            value: Box::new(expression_fn(ExpressionBuilder {})),
        })
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
        let result = AstBuilder::new()
            .statement()
            .var_declaration()
            .infer_type()
            .name("my_var")
            .with_assignment(|expression_builder| {
                expression_builder.operation(|operation_builder| {
                    operation_builder.not(|not_expression_builder| {
                        not_expression_builder.value_literal(Value::Boolean(BoolValue(true)))
                    })
                })
            });

        let expected = AstBuilder {
            nodes: vec![Node::VariableDeclaration {
                var_type: VariableDeclarationType::Infer,
                var_name: "my_var".to_owned(),
                value: Expression::Operation(Operation::Unary(UnaryOperation::Not {
                    value: Box::new(Expression::ValueLiteral(Value::Boolean(BoolValue(true)))),
                })),
            }],
        };

        assert_eq!(result, expected);
    }
}
