use crate::ast::node::{Expression, FunctionCall, Operation, Value};

use super::{function_call_builder::FunctionCallBuilder, operation_builder::OperationBuilder};

#[derive(Default)]
pub struct ExpressionBuilder {}

impl ExpressionBuilder {
    pub fn function_call(
        self,
        function_call_fn: impl FnOnce(FunctionCallBuilder) -> FunctionCall,
    ) -> Expression {
        let function_call = function_call_fn(FunctionCallBuilder::default());

        Expression::FunctionCall(function_call)
    }

    pub fn variable(self, variable_name: &str) -> Expression {
        Expression::VariableAccess(variable_name.to_owned())
    }

    pub fn value_literal(self, value: Value) -> Expression {
        Expression::ValueLiteral(value)
    }

    pub fn operation(self, operation_fn: impl FnOnce(OperationBuilder) -> Operation) -> Expression {
        let operation = operation_fn(OperationBuilder {});

        Expression::Operation(operation)
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::node::{
        Expression, FunctionCall, FunctionId, Operation, UnaryOperation,
    };

    use super::ExpressionBuilder;

    #[test]
    fn function_call() {
        let actual = ExpressionBuilder::default().function_call(|function_call| {
            function_call
                .function_id("my_function")
                .no_parameters()
                .build()
        });

        let expected = Expression::FunctionCall(FunctionCall {
            function_id: FunctionId("my_function".to_owned()),
            parameters: Vec::new(),
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn variable() {
        let actual = ExpressionBuilder::default().variable("var_name");

        let expected = Expression::VariableAccess("var_name".to_owned());

        assert_eq!(actual, expected);
    }

    #[test]
    fn value_literal() {
        let actual = ExpressionBuilder::default().value_literal(true.into());

        let expected = true.into();

        assert_eq!(actual, expected);
    }

    #[test]
    fn operation() {
        let actual = ExpressionBuilder::default().operation(|operation| {
            operation.not(|not| not.value_literal(true.into()))
        });

        let expected = Expression::Operation(Operation::Unary {
            operation: UnaryOperation::Not,
            value: Box::new(true.into()),
        });

        assert_eq!(actual, expected);
    }
}
