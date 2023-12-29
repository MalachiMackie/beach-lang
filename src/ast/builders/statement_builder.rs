use crate::ast::node::{Expression, Node};

use super::{ast_builder::AstBuilder, variable_declaration_builder::VariableDeclarationBuilder};

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

    pub fn return_value(mut self, expression: Expression) -> AstBuilder {
        self.builder.nodes.push(Node::FunctionReturn {
            return_value: Some(expression),
        });
        self.builder
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::node::{BoolValue, FunctionId, Type, Value, VariableDeclarationType};

    use super::*;

    #[test]
    fn function_call() {
        let result = AstBuilder::new()
            .function_declaration()
            .name("my_function")
            .no_parameters()
            .return_type(Type::Boolean)
            .body(|body| {
                body.statement()
                    .return_value(Expression::ValueLiteral(Value::Boolean(BoolValue(true))))
            })
            .statement()
            .var_declaration()
            .infer_type()
            .name("my_var")
            .with_assignment(Expression::FunctionCall(FunctionId(
                "my_function".to_owned(),
            )));

        let expected = AstBuilder {
            nodes: vec![
                Node::FunctionDeclaration {
                    id: FunctionId("my_function".to_owned()),
                    name: "my_function".to_owned(),
                    parameters: Vec::new(),
                    return_type: Type::Boolean,
                    body: vec![Node::FunctionReturn {
                        return_value: Some(Expression::ValueLiteral(Value::Boolean(BoolValue(
                            true,
                        )))),
                    }],
                },
                Node::VariableDeclaration {
                    var_type: VariableDeclarationType::Infer,
                    var_name: "my_var".to_owned(),
                    value: Expression::FunctionCall(FunctionId("my_function".to_owned())),
                },
            ],
        };

        assert_eq!(result, expected)
    }
}
