use crate::ast::node::{Expression, FunctionCall, FunctionId};

use super::expression_builder::ExpressionBuilder;

#[derive(Default)]
pub struct FunctionCallBuilder {
    pub function_id: Option<FunctionId>,
    pub parameters: Option<Vec<Expression>>,
}

impl FunctionCallBuilder {
    pub fn function_id(mut self, function_id: &str) -> Self {
        self.function_id = Some(FunctionId(function_id.to_owned()));
        self
    }

    pub fn parameter(
        mut self,
        expression_fn: impl FnOnce(ExpressionBuilder) -> Expression,
    ) -> Self {
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
    use crate::ast::node::FunctionId;

    use super::*;

    #[test]
    fn function_call_no_parameters() {
        let result = FunctionCallBuilder::default()
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
        let actual = FunctionCallBuilder::default()
            .function_id("my_function")
            .parameter(|param| param.value_literal(true.into()))
            .parameter(|param| param.value_literal(10.into()))
            .build();

        let expected = FunctionCall {
            function_id: FunctionId("my_function".to_owned()),
            parameters: vec![true.into(), 10.into()],
        };

        assert_eq!(actual, expected);
    }
}
