use crate::ast::node::{Expression, Function, FunctionId, Operation, Value};

use super::{operation_builder::OperationBuilder, statement_builder::FunctionCallBuilder};

pub struct ExpressionBuilder {}

impl ExpressionBuilder {
    pub fn function_call(
        self,
        function_call_fn: impl Fn(FunctionCallBuilder) -> FunctionCallBuilder,
    ) -> Expression {
        let function_call_builder = function_call_fn(FunctionCallBuilder {
            function_id: None,
            parameters: None,
        });

        Expression::FunctionCall(
            function_call_builder
                .function_id
                .expect("Expected function call to be set"),
            function_call_builder
                .parameters
                .expect("Expected function parameters to be set"),
        )
    }

    pub fn variable(self, variable_name: &str) -> Expression {
        Expression::VariableAccess(variable_name.to_owned())
    }

    pub fn value_literal(self, value: Value) -> Expression {
        Expression::ValueLiteral(value)
    }

    pub fn operation(self, operation_fn: impl Fn(OperationBuilder) -> Operation) -> Expression {
        let operation = operation_fn(OperationBuilder {});

        Expression::Operation(operation)
    }
}
