use crate::ast::node::{Expression, FunctionCall, FunctionId};

use super::expression_builder::ExpressionBuilder;

pub struct FunctionCallBuilder {
    pub function_id: Option<FunctionId>,
    pub parameters: Option<Vec<Expression>>,
}

impl FunctionCallBuilder {
    pub fn new() -> Self {
        Self {
            function_id: None,
            parameters: None,
        }
    }

    pub fn function_id(mut self, function_id: &str) -> Self {
        self.function_id = Some(FunctionId(function_id.to_owned()));
        self
    }

    pub fn parameter(mut self, expression_fn: impl Fn(ExpressionBuilder) -> Expression) -> Self {
        let expression = expression_fn(ExpressionBuilder {});

        let Some(parameters) = &mut self.parameters else {
            self.parameters = Some(vec![expression]);
            return self;
        };

        parameters.push(expression);
        self
    }

    pub fn no_parameters(mut self) -> Self {
        self.parameters = Some(Vec::new());
        self
    }

    pub fn build(self) -> FunctionCall {
        FunctionCall {
            function_id: self.function_id.expect("function id to be set"),
            parameters: self.parameters.expect("parameters to be set"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::node::{BoolValue, FunctionId, UIntValue, Value};

    use super::*;

    #[test]
    fn function_call_no_parameters() {
        let result = FunctionCallBuilder::new()
            .function_id("my_function")
            .no_parameters()
            .build();

        let expected = FunctionCall {
            function_id: FunctionId("my_function".to_owned()),
            parameters: Vec::new(),
        };

        assert_eq!(result, expected)
    }

    #[test]
    fn function_call_parameter() {
        let actual = FunctionCallBuilder::new()
            .function_id("my_function")
            .parameter(|param| param.value_literal(Value::Boolean(BoolValue(true))))
            .parameter(|param| param.value_literal(Value::UInt(UIntValue(10))))
            .build();

        let expected = FunctionCall {
            function_id: FunctionId("my_function".to_owned()),
            parameters: vec![
                Expression::ValueLiteral(Value::Boolean(BoolValue(true))),
                Expression::ValueLiteral(Value::UInt(UIntValue(10))),
            ],
        };

        assert_eq!(actual, expected);
    }
}
