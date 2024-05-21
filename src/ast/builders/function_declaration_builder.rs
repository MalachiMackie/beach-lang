use crate::{
    ast::node::{
        Ast, FunctionDeclaration, FunctionId, FunctionParameter, FunctionReturnType, Node, Type,
    },
    token_stream::token::TokenSource,
};

use super::ast_builder::AstBuilder;

#[derive(Debug, PartialEq, Default)]
pub struct FunctionDeclarationBuilder {
    pub(super) id: Option<FunctionId>,
    pub(super) name: Option<String>,
    pub(super) parameters: Option<Vec<FunctionParameter>>,
    pub(super) return_type: Option<FunctionReturnType>,
    pub(super) body: Option<Vec<Node>>,
    pub(super) function_keyword_token: Option<TokenSource>,
    pub(super) function_identifier_token: Option<TokenSource>,
    pub(super) left_parenthesis_token: Option<TokenSource>,
    pub(super) right_parenthesis_token: Option<TokenSource>,
    pub(super) left_curley_brace_token: Option<TokenSource>,
    pub(super) right_curley_brace_token: Option<TokenSource>,
    pub(super) comma_tokens: Vec<TokenSource>, // todo: local functions
}

impl FunctionDeclarationBuilder {
    pub fn name(
        mut self,
        name: &str,
        function_keyword_token: TokenSource,
        name_token: TokenSource,
        left_parenthesis_token: TokenSource,
        right_parenthesis_token: TokenSource,
        left_curley_brace_token: TokenSource,
        right_curley_brace_token: TokenSource,
    ) -> Self {
        self.id = Some(FunctionId(name.to_owned()));
        self.name = Some(name.to_owned());
        self.function_keyword_token = Some(function_keyword_token);
        self.function_identifier_token = Some(name_token);
        self.left_parenthesis_token = Some(left_parenthesis_token);
        self.right_parenthesis_token = Some(right_parenthesis_token);
        self.left_curley_brace_token = Some(left_curley_brace_token);
        self.right_curley_brace_token = Some(right_curley_brace_token);
        self
    }

    pub fn first_parameter(
        mut self,
        parameter_type: Type,
        parameter_name: &str,
        type_token: TokenSource,
        param_name_token: TokenSource,
    ) -> Self {
        if self.parameters.is_some() {
            panic!("first_parameter can only be set once");
        }

        self.parameters = Some(vec![FunctionParameter::FunctionParameter {
            param_type: parameter_type,
            type_token: type_token,
            param_name: parameter_name.to_owned(),
            param_name_token,
        }]);
        self
    }

    pub fn parameter(
        mut self,
        parameter_type: Type,
        parameter_name: &str,
        type_token: TokenSource,
        param_name_token: TokenSource,
        comma_token: TokenSource,
    ) -> Self {
        let Some(mut parameters) = self.parameters else {
            panic!("must set first_parameter first");
        };
        parameters.push(FunctionParameter::FunctionParameter {
            param_type: parameter_type,
            type_token: type_token,
            param_name: parameter_name.to_owned(),
            param_name_token,
        });
        self.comma_tokens.push(comma_token);
        self
    }

    pub fn no_parameters(mut self) -> Self {
        self.parameters = Some(Vec::new());
        self
    }

    pub fn return_type(
        mut self,
        return_type: Type,
        function_signiture_splitter_token: TokenSource,
        type_token: TokenSource,
    ) -> Self {
        self.return_type = Some(FunctionReturnType::Type {
            return_type,
            function_signiture_separator_token: function_signiture_splitter_token,
            type_token,
        });
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
            comma_tokens: self.comma_tokens,
            function_identifier_token: self
                .function_identifier_token
                .expect("function_identifiec_token to be set"),
            function_keyword_token: self
                .function_keyword_token
                .expect("function_keyword_token to be set"),
            left_parenthesis_token: self
                .left_parenthesis_token
                .expect("left_parenthesis_token to be set"),
            right_parenthesis_token: self
                .right_parenthesis_token
                .expect("right_parenthesis_token to be set"),
            left_curley_brace_token: self
                .left_curley_brace_token
                .expect("left_curley_brace_token to be set"),
            right_curley_brace_token: self
                .right_curley_brace_token
                .expect("right_curley_brace_token to be set"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::node::VariableDeclarationType,
        token_stream::token::{Token, TokenSource},
    };

    use super::*;

    #[test]
    fn function_declaration_parameters() {
        let result = FunctionDeclarationBuilder::default()
            .name(
                "my_function",
                TokenSource::dummy_function(),
                TokenSource::dummy(Token::Identifier("my_function".to_owned())),
                TokenSource::dummy_left_parenthesis(),
                TokenSource::dummy_right_parenthesis(),
                TokenSource::dummy_left_curley_brace(),
                TokenSource::dummy_right_curley_brace(),
            )
            .first_parameter(
                Type::Boolean,
                "param1",
                TokenSource::dummy(Token::TypeKeyword(Type::Boolean)),
                TokenSource::dummy(Token::Identifier("param1".to_owned())),
            )
            .return_type(
                Type::UInt,
                TokenSource::dummy(Token::FunctionSignitureSplitter),
                TokenSource::dummy(Token::TypeKeyword(Type::UInt)),
            )
            .body(|builder: AstBuilder| {
                builder
                    .statement(|statement| {
                        statement.var_declaration(|var_declaration_builder| {
                            var_declaration_builder
                                .declare_type(Type::Boolean)
                                .name("my_var_name")
                                .with_assignment(|expression_builder| {
                                    expression_builder
                                        .value_literal(true.into(), TokenSource::dummy_true())
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
                param_name_token: TokenSource::dummy(Token::Identifier("param1".to_owned())),
                type_token: TokenSource::dummy(Token::TypeKeyword(Type::Boolean)),
            }],
            return_type: FunctionReturnType::Type {
                return_type: Type::UInt,
                function_signiture_separator_token: TokenSource::dummy(
                    Token::FunctionSignitureSplitter,
                ),
                type_token: TokenSource::dummy(Token::TypeKeyword(Type::UInt)),
            },
            body: vec![Node::VariableDeclaration {
                var_type: VariableDeclarationType::Type(Type::Boolean),
                var_name: "my_var_name".to_owned(),
                value: (true, TokenSource::dummy_true()).into(),
            }],
            function_keyword_token: TokenSource::dummy_function(),
            function_identifier_token: TokenSource::dummy(Token::Identifier(
                "my_function".to_owned(),
            )),
            left_parenthesis_token: TokenSource::dummy_left_parenthesis(),
            right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
            comma_tokens: vec![TokenSource::dummy(Token::Comma)],
            left_curley_brace_token: TokenSource::dummy_left_curley_brace(),
            right_curley_brace_token: TokenSource::dummy_right_curley_brace(),
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn function_declaration_no_parameters() {
        let actual = FunctionDeclarationBuilder::default()
            .name(
                "my_function",
                TokenSource::dummy_function(),
                TokenSource::dummy(Token::Identifier("my_function".to_owned())),
                TokenSource::dummy_left_parenthesis(),
                TokenSource::dummy_right_parenthesis(),
                TokenSource::dummy_left_curley_brace(),
                TokenSource::dummy_right_curley_brace(),
            )
            .no_parameters()
            .return_type(
                Type::UInt,
                TokenSource::dummy(Token::FunctionSignitureSplitter),
                TokenSource::dummy(Token::TypeKeyword(Type::UInt)),
            )
            .body(|body| {
                body.statement(|statement| {
                    statement.return_value(|return_value| {
                        return_value.value_literal(10.into(), TokenSource::dummy_uint(10))
                    })
                })
                .build()
            });

        let expected = FunctionDeclaration {
            id: FunctionId("my_function".to_owned()),
            name: "my_function".to_owned(),
            parameters: Vec::new(),
            return_type: FunctionReturnType::Type {
                return_type: Type::UInt,
                function_signiture_separator_token: TokenSource::dummy(
                    Token::FunctionSignitureSplitter,
                ),
                type_token: TokenSource::dummy(Token::TypeKeyword(Type::UInt)),
            },
            function_keyword_token: TokenSource::dummy_function(),
            left_parenthesis_token: TokenSource::dummy_left_parenthesis(),
            right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
            comma_tokens: Vec::new(),
            body: vec![Node::FunctionReturn {
                return_value: Some((10, TokenSource::dummy_uint(10)).into()),
            }],
            function_identifier_token: TokenSource::dummy(Token::Identifier(
                "my_function".to_owned(),
            )),
            left_curley_brace_token: TokenSource::dummy_left_curley_brace(),
            right_curley_brace_token: TokenSource::dummy_right_curley_brace(),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn function_declaration_void() {
        let actual = FunctionDeclarationBuilder::default()
            .name(
                "my_function",
                TokenSource::dummy_function(),
                TokenSource::dummy(Token::Identifier("my_function".to_owned())),
                TokenSource::dummy_left_parenthesis(),
                TokenSource::dummy_right_parenthesis(),
                TokenSource::dummy_left_curley_brace(),
                TokenSource::dummy_right_curley_brace(),
            )
            .no_parameters()
            .void()
            .body(|body| body.statement(|statement| statement.return_void()).build());

        let expected = FunctionDeclaration {
            id: FunctionId("my_function".to_owned()),
            name: "my_function".to_owned(),
            parameters: Vec::new(),
            return_type: FunctionReturnType::Void,
            body: vec![Node::FunctionReturn { return_value: None }],
            comma_tokens: Vec::new(),
            function_identifier_token: TokenSource::dummy(Token::Identifier(
                "my_function".to_owned(),
            )),
            function_keyword_token: TokenSource::dummy_function(),
            left_parenthesis_token: TokenSource::dummy_left_parenthesis(),
            right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
            left_curley_brace_token: TokenSource::dummy_left_curley_brace(),
            right_curley_brace_token: TokenSource::dummy_right_curley_brace()
        };

        assert_eq!(actual, expected);
    }
}
