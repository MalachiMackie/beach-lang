use crate::ast::node::{
    Ast, FunctionDeclaration, FunctionId, FunctionParameter, FunctionReturnType, Node, Type,
};

use super::ast_builder::AstBuilder;

#[derive(Debug, PartialEq)]
pub struct FunctionDeclarationBuilder {
    pub(super) builder: AstBuilder,
    pub(super) id: Option<FunctionId>,
    pub(super) name: Option<String>,
    pub(super) parameters: Option<Vec<FunctionParameter>>,
    pub(super) return_type: Option<FunctionReturnType>,
    pub(super) body: Option<Ast>,
}

impl FunctionDeclarationBuilder {
    pub fn name(mut self, name: &str) -> Self {
        self.id = Some(FunctionId(name.to_owned()));
        self.name = Some(name.to_owned());
        self
    }

    pub fn parameters(mut self, parameters: Vec<FunctionParameter>) -> Self {
        self.parameters = Some(parameters);
        self
    }

    pub fn no_parameters(mut self) -> Self {
        self.parameters = Some(Vec::new());
        self
    }

    pub fn return_type(mut self, return_type: Type) -> Self {
        self.return_type = Some(FunctionReturnType::Type(return_type));
        self
    }

    pub fn void(mut self) -> Self {
        self.return_type = Some(FunctionReturnType::Void);
        self
    }

    pub fn body(mut self, builder: impl FnOnce(AstBuilder) -> AstBuilder) -> AstBuilder {
        self.body = Some(builder(AstBuilder::new()).build());
        self.builder
            .nodes
            .push(Node::FunctionDeclaration(FunctionDeclaration {
                id: self.id.expect("Function id should be set"),
                name: self.name.expect("function name should be set"),
                parameters: self.parameters.expect("function parameters should be set"),
                return_type: self
                    .return_type
                    .expect("function return type should be set"),
                body: self.body.expect("function body should be set"),
            }));
        self.builder
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::ast::node::{BoolValue, Expression, Value, VariableDeclarationType};

    use super::*;

    #[test]
    fn function_declaration() {
        let result = AstBuilder::new()
            .function_declaration()
            .name("my_function")
            .parameters(vec![(Type::Boolean, "param1".to_owned()).into()])
            .return_type(Type::UInt)
            .body(|body: AstBuilder| {
                body.statement()
                    .var_declaration()
                    .declare_type(Type::Boolean)
                    .name("my_var_name")
                    .with_assignment(|expression_builder| {
                        expression_builder.value_literal(Value::Boolean(BoolValue(true)))
                    })
            });

        let expected = AstBuilder {
            nodes: vec![Node::FunctionDeclaration(FunctionDeclaration {
                id: FunctionId("my_function".to_owned()),
                name: "my_function".to_owned(),
                parameters: vec![FunctionParameter::FunctionParameter {
                    param_type: Type::Boolean,
                    param_name: "param1".to_owned(),
                }],
                return_type: FunctionReturnType::Type(Type::UInt),
                body: Ast {
                    nodes: vec![Node::VariableDeclaration {
                        var_type: VariableDeclarationType::Type(Type::Boolean),
                        var_name: "my_var_name".to_owned(),
                        value: Expression::ValueLiteral(Value::Boolean(BoolValue(true))),
                    }],
                    functions: HashMap::new(),
                },
            })],
        };

        assert_eq!(result, expected);
    }
}
