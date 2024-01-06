use crate::ast::node::{
    Ast, FunctionDeclaration, FunctionId, FunctionParameter, FunctionReturnType, Node, Type,
};

use super::ast_builder::AstBuilder;

#[derive(Debug, PartialEq, Default)]
pub struct FunctionDeclarationBuilder {
    pub(super) id: Option<FunctionId>,
    pub(super) name: Option<String>,
    pub(super) parameters: Option<Vec<FunctionParameter>>,
    pub(super) return_type: Option<FunctionReturnType>,
    pub(super) body: Option<Vec<Node>>,
    // todo: local functions
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

    pub fn return_type(mut self, return_type: Type) -> Self {
        self.return_type = Some(FunctionReturnType::Type(return_type));
        self
    }

    pub fn void(mut self) -> Self {
        self.return_type = Some(FunctionReturnType::Void);
        self
    }

    pub fn body(mut self, builder: impl FnOnce(AstBuilder) -> Ast) -> FunctionDeclaration {
        self.body = Some(builder(AstBuilder::default()).nodes);
        FunctionDeclaration {
            id: self.id.expect("Function id should be set"),
            name: self.name.expect("function name should be set"),
            parameters: self.parameters.expect("function parameters should be set"),
            return_type: self
                .return_type
                .expect("function return type should be set"),
            body: self.body.expect("function body should be set"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::node::VariableDeclarationType;

    use super::*;

    #[test]
    fn function_declaration_parameters() {
        let result = FunctionDeclarationBuilder::default()
            .name("my_function")
            .parameters(vec![(Type::Boolean, "param1".to_owned()).into()])
            .return_type(Type::UInt)
            .body(|builder: AstBuilder| {
                builder
                    .statement(|statement| {
                        statement.var_declaration(|var_declaration_builder| {
                            var_declaration_builder
                                .declare_type(Type::Boolean)
                                .name("my_var_name")
                                .with_assignment(|expression_builder| {
                                    expression_builder.value_literal(true.into())
                                })
                        })
                    })
                    .build()
            });

        let expected = FunctionDeclaration {
            id: FunctionId("my_function".to_owned()),
            name: "my_function".to_owned(),
            parameters: vec![FunctionParameter::FunctionParameter {
                param_type: Type::Boolean,
                param_name: "param1".to_owned(),
            }],
            return_type: FunctionReturnType::Type(Type::UInt),
            body: vec![Node::VariableDeclaration {
                var_type: VariableDeclarationType::Type(Type::Boolean),
                var_name: "my_var_name".to_owned(),
                value: true.into(),
            }],
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn function_declaration_no_parameters() {
        let actual = FunctionDeclarationBuilder::default()
            .name("my_function")
            .parameters(Vec::new())
            .return_type(Type::UInt)
            .body(|body| {
                body.statement(|statement| {
                    statement.return_value(|return_value| return_value.value_literal(10.into()))
                })
                .build()
            });

        let expected = FunctionDeclaration {
            id: FunctionId("my_function".to_owned()),
            name: "my_function".to_owned(),
            parameters: Vec::new(),
            return_type: FunctionReturnType::Type(Type::UInt),
            body: vec![Node::FunctionReturn {
                return_value: Some(10.into()),
            }],
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn function_declaration_void() {
        let actual = FunctionDeclarationBuilder::default()
            .name("my_function")
            .parameters(Vec::new())
            .void()
            .body(|body| body.statement(|statement| statement.return_void()).build());

        let expected = FunctionDeclaration {
            id: FunctionId("my_function".to_owned()),
            name: "my_function".to_owned(),
            parameters: Vec::new(),
            return_type: FunctionReturnType::Void,
            body: vec![Node::FunctionReturn { return_value: None }],
        };

        assert_eq!(actual, expected);
    }
}
