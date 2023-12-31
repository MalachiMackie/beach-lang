use crate::ast::node::{Expression, FunctionId};

use super::expression_builder::ExpressionBuilder;

pub struct FunctionCallBuilder {
    pub function_id: Option<FunctionId>,
    pub parameters: Option<Vec<Expression>>,
}

impl FunctionCallBuilder {
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
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::ast::{
        builders::ast_builder::AstBuilder,
        node::{
            Ast, BoolValue, FunctionDeclaration, FunctionId, FunctionReturnType, Node, Type, Value,
            VariableDeclarationType,
        },
    };

    use super::*;

    #[test]
    fn function_call() {
        let result = AstBuilder::new()
            .function_declaration(|function_declaration_builder| {
                function_declaration_builder
                    .name("my_function")
                    .no_parameters()
                    .return_type(Type::Boolean)
                    .body(|builder| {
                        builder.return_value(|expression_builder| {
                            expression_builder.value_literal(Value::Boolean(BoolValue(true)))
                        })
                    })
            })
            .var_declaration(|var_declaration_builder| {
                var_declaration_builder
                    .infer_type()
                    .name("my_var")
                    .with_assignment(|expression_builder| {
                        expression_builder.function_call(|builder| {
                            builder.function_id("my_function").no_parameters()
                        })
                    })
            });

        let expected = AstBuilder {
            nodes: vec![
                Node::FunctionDeclaration(FunctionDeclaration {
                    id: FunctionId("my_function".to_owned()),
                    name: "my_function".to_owned(),
                    parameters: Vec::new(),
                    return_type: FunctionReturnType::Type(Type::Boolean),
                    body: Ast {
                        nodes: vec![Node::FunctionReturn {
                            return_value: Some(Expression::ValueLiteral(Value::Boolean(
                                BoolValue(true),
                            ))),
                        }],
                        functions: HashMap::new(),
                    },
                }),
                Node::VariableDeclaration {
                    var_type: VariableDeclarationType::Infer,
                    var_name: "my_var".to_owned(),
                    value: Expression::FunctionCall(
                        FunctionId("my_function".to_owned()),
                        Vec::new(),
                    ),
                },
            ],
        };

        assert_eq!(result, expected)
    }
}
