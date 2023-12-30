use crate::ast::node::{Expression, FunctionId, Node};

use super::{
    ast_builder::AstBuilder, expression_builder::ExpressionBuilder,
    variable_declaration_builder::VariableDeclarationBuilder,
};

pub struct StatementBuilder {
    pub(super) builder: AstBuilder,
}

impl StatementBuilder {
    pub fn var_declaration(self) -> VariableDeclarationBuilder {
        VariableDeclarationBuilder {
            builder: self,
            var_name: None,
            var_type: None,
        }
    }

    pub fn function_call(mut self, function_id: &str, parameters: Vec<Expression>) -> AstBuilder {
        self.builder.nodes.push(Node::FunctionCall {
            function_id: FunctionId(function_id.to_owned()),
            parameters,
        });

        self.builder
    }

    pub fn return_value(
        mut self,
        expression: impl Fn(ExpressionBuilder) -> Expression,
    ) -> AstBuilder {
        self.builder.nodes.push(Node::FunctionReturn {
            return_value: Some(expression(ExpressionBuilder {})),
        });
        self.builder
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::ast::node::{
        Ast, BoolValue, FunctionDeclaration, FunctionId, FunctionReturnType, Type, Value,
        VariableDeclarationType,
    };

    use super::*;

    #[test]
    fn function_call() {
        let result = AstBuilder::new()
            .function_declaration()
            .name("my_function")
            .no_parameters()
            .return_type(Type::Boolean)
            .body(|body| {
                body.statement().return_value(|expression_builder| {
                    expression_builder.value_literal(Value::Boolean(BoolValue(true)))
                })
            })
            .statement()
            .var_declaration()
            .infer_type()
            .name("my_var")
            .with_assignment(|expression_builder| {
                expression_builder.function_call("my_function", Vec::new())
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
