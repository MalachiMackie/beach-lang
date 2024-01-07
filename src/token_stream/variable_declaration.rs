use std::collections::VecDeque;

use crate::ast::{
    builders::variable_declaration_builder::VariableDeclarationBuilder,
    node::{Node, VariableDeclarationType},
};

use super::{
    expression::take_expression,
    token::{Token, TokenStreamError},
};

pub(super) fn try_create_variable_declaration(
    var_decl_type: VariableDeclarationType,
    mut tokens: VecDeque<Token>,
) -> Result<impl FnOnce(VariableDeclarationBuilder) -> Node, Vec<TokenStreamError>> {
    let Some(Token::Identifier(name)) = tokens.pop_front() else {
        return Err(vec![TokenStreamError{message: "expected variable identifier".to_owned()}]);
    };

    if !matches!(tokens.pop_front(), Some(Token::AssignmentOperator)) {
        return Err(vec![TokenStreamError {
            message: "expected assignment operator \"=\"".to_owned(),
        }]);
    }

    let expression_fn = take_expression(&mut tokens)?;

    Ok(move |mut var_decl_builder: VariableDeclarationBuilder| {
        match var_decl_type {
            VariableDeclarationType::Infer => {
                var_decl_builder = var_decl_builder.infer_type();
            }
            VariableDeclarationType::Type(var_type) => {
                var_decl_builder = var_decl_builder.declare_type(var_type);
            }
        }
        var_decl_builder = var_decl_builder.name(&name);
        var_decl_builder.with_assignment(expression_fn)
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{builders::ast_builder::AstBuilder, node::Type},
        token_stream::token::Token,
    };

    /// infer my_var = true;
    #[test]
    fn infer_boolean_variable_declaration_from_token_stream() {
        let tokens = vec![
            Token::InferKeyword,
            Token::Identifier("my_var".to_owned()),
            Token::AssignmentOperator,
            Token::TrueKeyword,
            Token::SemiColon,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        let expected = AstBuilder::default().statement(|statement| {
            statement.var_declaration(|var_decl| {
                var_decl
                    .infer_type()
                    .name("my_var")
                    .with_assignment(|_| true.into())
            })
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }

    /// boolean my_var = my_other_var;
    #[test]
    fn type_decl_variable_declaration_assign_variable_name() {
        let tokens = vec![
            Token::TypeKeyword(Type::Boolean),
            Token::Identifier("my_var".to_owned()),
            Token::AssignmentOperator,
            Token::Identifier("my_other_var".to_owned()),
            Token::SemiColon,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        let expected = AstBuilder::default().statement(|statement| {
            statement.var_declaration(|var_decl| {
                var_decl
                    .name("my_var")
                    .declare_type(Type::Boolean)
                    .with_assignment(|value| value.variable("my_other_var"))
            })
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }

    #[test]
    fn var_declaration_greater_than_operatior_with_variable_name() {
        let tokens = vec![
            Token::InferKeyword,
            Token::Identifier("my_var".to_owned()),
            Token::AssignmentOperator,
            Token::Identifier("other_var".to_owned()), // breaks with identifier
            Token::RightAngle,
            Token::UIntValue(11),
            Token::SemiColon,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        let expected = AstBuilder::default().statement(|statement| {
            statement.var_declaration(|var_decl| {
                var_decl
                    .infer_type()
                    .name("my_var")
                    .with_assignment(|assignment| {
                        assignment.operation(|operation| {
                            operation.greater_than(
                                |expression| expression.variable("other_var"),
                                |_| 11.into(),
                            )
                        })
                    })
            })
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }

    #[test]
    fn var_declaration_greater_than_with_function_call() {
        let tokens = vec![
            Token::InferKeyword,
            Token::Identifier("my_var".to_owned()),
            Token::AssignmentOperator,
            Token::Identifier("my_function".to_owned()),
            Token::LeftParenthesis,
            Token::RightParenthesis,
            Token::RightAngle,
            Token::UIntValue(10),
            Token::SemiColon,
        ];

        let result = AstBuilder::from_token_stream(tokens);

        let expected = AstBuilder::default().statement(|statement| {
            statement.var_declaration(|var_declaration| {
                var_declaration
                    .infer_type()
                    .name("my_var")
                    .with_assignment(|value| {
                        value.operation(|operation| {
                            operation.greater_than(
                                |left| {
                                    left.function_call(|function_call| {
                                        function_call
                                            .function_id("my_function")
                                            .no_parameters()
                                            .build()
                                    })
                                },
                                |_| 10.into(),
                            )
                        })
                    })
            })
        });

        assert!(matches!(result, Ok(ast_builder) if ast_builder == expected));
    }
}
