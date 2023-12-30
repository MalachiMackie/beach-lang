use crate::ast::node::{Expression, FunctionId, Operation, Value};

use super::operation_builder::OperationBuilder;

pub struct ExpressionBuilder {}

impl ExpressionBuilder {
    pub fn function_call(self, function_id: &str, parameters: Vec<Expression>) -> Expression {
        Expression::FunctionCall(FunctionId(function_id.to_owned()), parameters)
    }

    pub fn value_literal(self, value: Value) -> Expression {
        Expression::ValueLiteral(value)
    }

    pub fn operation(self, operation_fn: impl Fn(OperationBuilder) -> Operation) -> Expression {
        let operation = operation_fn(OperationBuilder {});

        Expression::Operation(operation)
    }
}
