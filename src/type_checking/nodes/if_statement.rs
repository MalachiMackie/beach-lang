use std::collections::HashMap;

use crate::{
    ast::node::{ElseIfBlock, Function, FunctionId, IfStatement, Type},
    type_checking::{verify_type, TypeCheckingError},
};

use super::node::type_check_nodes;

impl IfStatement {
    pub fn type_check(
        &self,
        functions: &HashMap<FunctionId, Function>,
        local_variables: &HashMap<String, Type>,
        current_function: Option<&FunctionId>,
    ) -> Result<Option<Type>, Vec<TypeCheckingError>> {
        let mut errors = Vec::new();

        // get the if check expression type
        let check_type = self.check_expression.get_type(functions, local_variables);

        // verify the if check expression is a boolean
        if let Err(err) = verify_type(check_type, Type::Boolean) {
            errors.push(err);
        };

        // type check the actual if check expression
        if let Err(expression_errors) = self.check_expression.type_check(functions, local_variables)
        {
            errors.extend(expression_errors)
        }

        // type check the if block nodes
        let return_type =
            match type_check_nodes(&self.if_block, functions, local_variables, current_function) {
                Ok(return_type) => return_type,
                Err(if_errors) => {
                    errors.extend(if_errors);
                    None
                }
            };

        // type check any else if blocks
        errors.extend(
            self.else_if_blocks
                .iter()
                .filter_map(|else_if_block| {
                    else_if_block
                        .type_check(functions, local_variables, current_function)
                        .err()
                })
                .flat_map(|x| x),
        );

        // type check the else block nodes
        if let Some(Err(else_errors)) = self.else_block.as_ref().map(|else_block| {
            type_check_nodes(
                &else_block.nodes,
                functions,
                local_variables,
                current_function,
            )
        }) {
            errors.extend(else_errors);
        }

        if errors.is_empty() {
            Ok(return_type)
        } else {
            Err(errors)
        }
    }
}

impl ElseIfBlock {
    pub fn type_check(
        &self,
        functions: &HashMap<FunctionId, Function>,
        local_variables: &HashMap<String, Type>,
        current_function: Option<&FunctionId>,
    ) -> Result<(), Vec<TypeCheckingError>> {
        let mut errors = Vec::new();

        // get the check expression's type
        let check_type = self.check.get_type(functions, local_variables);

        // verify the check expression is a boolean
        if let Err(err) = verify_type(check_type, Type::Boolean) {
            errors.push(err);
        };

        // type check the actual check expression
        if let Err(expression_errors) = self.check.type_check(functions, local_variables) {
            errors.extend(expression_errors);
        }

        // type check the else block nodes
        if let Err(block_errors) =
            type_check_nodes(&self.block, functions, local_variables, current_function)
        {
            errors.extend(block_errors);
        };

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        ast::node::{
            BinaryOperation, ElseBlock, ElseIfBlock, Expression, Function, FunctionId,
            FunctionReturnType, IfStatement, Node, Operation, Type, VariableDeclarationType,
        },
        token_stream::token::{Token, TokenSource},
    };

    #[test]
    fn type_check_if_statement_successful() {
        let if_statement = IfStatement {
            check_expression: (true, TokenSource::dummy_true()).into(),
            if_block: vec![Node::VariableDeclaration {
                var_type: VariableDeclarationType::Infer,
                var_name: "my_var".to_owned(),
                value: (true, TokenSource::dummy_true()).into(),
            }],
            else_if_blocks: vec![ElseIfBlock {
                check: (true, TokenSource::dummy_true()).into(),
                block: vec![Node::VariableDeclaration {
                    var_type: VariableDeclarationType::Infer,
                    var_name: "second_var".to_owned(),
                    value: (false, TokenSource::dummy_false()).into(),
                }],
                else_token: TokenSource::dummy_else(),
                if_token: TokenSource::dummy_if(),
                left_curley_brace_token: TokenSource::dummy_right_curley_brace(),
                left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

                right_curley_brace_token: TokenSource::dummy_right_curley_brace(),
                right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
            }],
            else_block: Some(ElseBlock {
                nodes: vec![Node::VariableDeclaration {
                    var_type: VariableDeclarationType::Infer,
                    var_name: "third_var".to_owned(),
                    value: (true, TokenSource::dummy_true()).into(),
                }],
                left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

                right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
            }),
            if_token: TokenSource::dummy_if(),
            left_curley_brace_token: TokenSource::dummy_right_curley_brace(),
            left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

            right_curley_brace_token: TokenSource::dummy_right_curley_brace(),
            right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
        };

        let result = if_statement.type_check(&HashMap::new(), &HashMap::new(), None);

        assert!(matches!(result, Ok(None)));
    }

    #[test]
    fn type_check_if_statement_return_value() {
        let if_statement = Node::IfStatement(IfStatement {
            check_expression: (true, TokenSource::dummy_true()).into(),
            if_block: vec![Node::FunctionReturn {
                return_value: Some((true, TokenSource::dummy_true()).into()),
            }],
            else_if_blocks: Vec::new(),
            else_block: None,
            if_token: TokenSource::dummy_if(),
            left_curley_brace_token: TokenSource::dummy_right_curley_brace(),
            left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

            right_curley_brace_token: TokenSource::dummy_right_curley_brace(),
            right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
        });

        let function = Function::CustomFunction {
            id: FunctionId("my_function".to_owned()),
            name: "my_function".to_owned(),
            parameters: Vec::new(),
            return_type: FunctionReturnType::Type {
                return_type: Type::Boolean,
                function_signiture_separator_token: TokenSource::dummy(
                    Token::FunctionSignitureSplitter,
                ),
                type_token: TokenSource::dummy(Token::TypeKeyword(Type::Boolean)),
            },
            body: Vec::new(),
        };

        let functions = HashMap::from_iter([(function.id().clone(), function)]);

        let result = if_statement.type_check(
            &functions,
            &mut HashMap::new(),
            Some(&FunctionId("my_function".to_owned())),
        );

        assert!(matches!(result, Ok(Some(Type::Boolean))));
    }

    #[test]
    fn type_check_if_statement_incorrect_check_type() {
        let if_statement = IfStatement {
            check_expression: (22, TokenSource::dummy_uint(22)).into(),
            if_block: vec![Node::VariableDeclaration {
                var_type: VariableDeclarationType::Infer,
                var_name: "my_var".to_owned(),
                value: (true, TokenSource::dummy_true()).into(),
            }],
            else_if_blocks: vec![ElseIfBlock {
                check: (true, TokenSource::dummy_true()).into(),
                block: vec![Node::VariableDeclaration {
                    var_type: VariableDeclarationType::Infer,
                    var_name: "second_var".to_owned(),
                    value: (false, TokenSource::dummy_false()).into(),
                }],
                if_token: TokenSource::dummy_if(),
                left_curley_brace_token: TokenSource::dummy_right_curley_brace(),
                left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

                right_curley_brace_token: TokenSource::dummy_right_curley_brace(),
                right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
                else_token: TokenSource::dummy_else(),
            }],
            else_block: Some(ElseBlock {
                nodes: vec![Node::VariableDeclaration {
                    var_type: VariableDeclarationType::Infer,
                    var_name: "third_var".to_owned(),
                    value: (true, TokenSource::dummy_true()).into(),
                }],
                left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

                right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
            }),
            if_token: TokenSource::dummy_if(),
            left_curley_brace_token: TokenSource::dummy_right_curley_brace(),
            left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

            right_curley_brace_token: TokenSource::dummy_right_curley_brace(),
            right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
        };

        let result = if_statement.type_check(&HashMap::new(), &HashMap::new(), None);

        assert!(
            matches!(result, Err(e) if e.len() == 1 && e[0].message == "Expected type to be Boolean, but found UInt")
        );
    }

    #[test]
    fn type_check_if_statement_nodes_failure() {
        let if_statement = IfStatement {
            check_expression: (true, TokenSource::dummy_true()).into(),
            if_block: vec![Node::VariableDeclaration {
                var_type: VariableDeclarationType::Type(Type::UInt),
                var_name: "my_var".to_owned(),
                value: (true, TokenSource::dummy_true()).into(),
            }],
            else_if_blocks: vec![ElseIfBlock {
                check: (true, TokenSource::dummy_true()).into(),
                block: vec![Node::VariableDeclaration {
                    var_type: VariableDeclarationType::Infer,
                    var_name: "second_var".to_owned(),
                    value: (false, TokenSource::dummy_false()).into(),
                }],
                else_token: TokenSource::dummy_else(),
                if_token: TokenSource::dummy_if(),
                left_curley_brace_token: TokenSource::dummy_right_curley_brace(),
                left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

                right_curley_brace_token: TokenSource::dummy_right_curley_brace(),
                right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
            }],
            else_block: Some(ElseBlock {
                nodes: vec![Node::VariableDeclaration {
                    var_type: VariableDeclarationType::Infer,
                    var_name: "third_var".to_owned(),
                    value: (true, TokenSource::dummy_true()).into(),
                }],
                left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

                right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
            }),
            if_token: TokenSource::dummy_if(),
            left_curley_brace_token: TokenSource::dummy_right_curley_brace(),
            left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

            right_curley_brace_token: TokenSource::dummy_right_curley_brace(),
            right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
        };

        let result = if_statement.type_check(&HashMap::new(), &HashMap::new(), None);

        assert!(matches!(result, Err(_)));
    }

    #[test]
    fn type_check_else_if_incorrect_check_type() {
        let if_statement = IfStatement {
            check_expression: (true, TokenSource::dummy_true()).into(),
            if_block: vec![Node::VariableDeclaration {
                var_type: VariableDeclarationType::Infer,
                var_name: "my_var".to_owned(),
                value: (true, TokenSource::dummy_true()).into(),
            }],
            else_if_blocks: vec![ElseIfBlock {
                check: (32, TokenSource::dummy_uint(32)).into(),
                block: vec![Node::VariableDeclaration {
                    var_type: VariableDeclarationType::Infer,
                    var_name: "second_var".to_owned(),
                    value: (false, TokenSource::dummy_false()).into(),
                }],
                else_token: TokenSource::dummy_else(),
                if_token: TokenSource::dummy_if(),
                left_curley_brace_token: TokenSource::dummy_right_curley_brace(),
                left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

                right_curley_brace_token: TokenSource::dummy_right_curley_brace(),
                right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
            }],
            else_block: Some(ElseBlock {
                nodes: vec![Node::VariableDeclaration {
                    var_type: VariableDeclarationType::Infer,
                    var_name: "third_var".to_owned(),
                    value: (true, TokenSource::dummy_true()).into(),
                }],
                left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

                right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
            }),
            if_token: TokenSource::dummy_if(),
            left_curley_brace_token: TokenSource::dummy_right_curley_brace(),
            left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

            right_curley_brace_token: TokenSource::dummy_right_curley_brace(),
            right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
        };

        let result = if_statement.type_check(&HashMap::new(), &HashMap::new(), None);

        assert!(matches!(result, Err(_)));
    }

    #[test]
    fn type_check_else_if_node_failure() {
        let if_statement = IfStatement {
            check_expression: (true, TokenSource::dummy_true()).into(),
            if_block: vec![Node::VariableDeclaration {
                var_type: VariableDeclarationType::Infer,
                var_name: "my_var".to_owned(),
                value: (true, TokenSource::dummy_true()).into(),
            }],
            else_if_blocks: vec![ElseIfBlock {
                check: (true, TokenSource::dummy_true()).into(),
                block: vec![Node::VariableDeclaration {
                    var_type: VariableDeclarationType::Type(Type::UInt),
                    var_name: "second_var".to_owned(),
                    value: (false, TokenSource::dummy_false()).into(),
                }],
                else_token: TokenSource::dummy_else(),
                if_token: TokenSource::dummy_if(),
                left_curley_brace_token: TokenSource::dummy_right_curley_brace(),
                left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

                right_curley_brace_token: TokenSource::dummy_right_curley_brace(),
                right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
            }],
            else_block: Some(ElseBlock {
                nodes: vec![Node::VariableDeclaration {
                    var_type: VariableDeclarationType::Infer,
                    var_name: "third_var".to_owned(),
                    value: (true, TokenSource::dummy_true()).into(),
                }],
                left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

                right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
            }),
            if_token: TokenSource::dummy_if(),
            left_curley_brace_token: TokenSource::dummy_right_curley_brace(),
            left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

            right_curley_brace_token: TokenSource::dummy_right_curley_brace(),
            right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
        };

        let result = if_statement.type_check(&HashMap::new(), &HashMap::new(), None);

        assert!(matches!(result, Err(_)));
    }

    #[test]
    fn type_check_else_node_failure() {
        let if_statement = IfStatement {
            check_expression: (true, TokenSource::dummy_true()).into(),
            if_block: vec![Node::VariableDeclaration {
                var_type: VariableDeclarationType::Infer,
                var_name: "my_var".to_owned(),
                value: (true, TokenSource::dummy_true()).into(),
            }],
            else_if_blocks: vec![ElseIfBlock {
                check: (true, TokenSource::dummy_true()).into(),
                block: vec![Node::VariableDeclaration {
                    var_type: VariableDeclarationType::Infer,
                    var_name: "second_var".to_owned(),
                    value: (false, TokenSource::dummy_false()).into(),
                }],
                else_token: TokenSource::dummy_else(),
                if_token: TokenSource::dummy_if(),
                left_curley_brace_token: TokenSource::dummy_right_curley_brace(),
                left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

                right_curley_brace_token: TokenSource::dummy_right_curley_brace(),
                right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
            }],
            else_block: Some(ElseBlock {
                nodes: vec![Node::VariableDeclaration {
                    var_type: VariableDeclarationType::Type(Type::UInt),
                    var_name: "third_var".to_owned(),
                    value: (true, TokenSource::dummy_true()).into(),
                }],
                left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

                right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
            }),
            if_token: TokenSource::dummy_if(),
            left_curley_brace_token: TokenSource::dummy_right_curley_brace(),
            left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

            right_curley_brace_token: TokenSource::dummy_right_curley_brace(),
            right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
        };

        let result = if_statement.type_check(&HashMap::new(), &HashMap::new(), None);

        assert!(matches!(result, Err(_)));
    }

    #[test]
    fn type_check_if_statement_check_expression_errors() {
        let if_statement = IfStatement {
            check_expression: Expression::Operation(Operation::Binary {
                operation: BinaryOperation::GreaterThan,
                left: Box::new((true, TokenSource::dummy_true()).into()),
                right: Box::new((10, TokenSource::dummy_uint(10)).into()),
                operator_token: TokenSource::dummy(Token::RightAngle),
            }),
            if_block: vec![Node::VariableDeclaration {
                var_type: VariableDeclarationType::Infer,
                var_name: "my_var".to_owned(),
                value: (true, TokenSource::dummy_true()).into(),
            }],
            else_if_blocks: vec![ElseIfBlock {
                check: (true, TokenSource::dummy_true()).into(),
                block: vec![Node::VariableDeclaration {
                    var_type: VariableDeclarationType::Infer,
                    var_name: "second_var".to_owned(),
                    value: (false, TokenSource::dummy_false()).into(),
                }],
                else_token: TokenSource::dummy_else(),
                if_token: TokenSource::dummy_if(),
                left_curley_brace_token: TokenSource::dummy_right_curley_brace(),
                left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

                right_curley_brace_token: TokenSource::dummy_right_curley_brace(),
                right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
            }],
            else_block: Some(ElseBlock {
                nodes: vec![Node::VariableDeclaration {
                    var_type: VariableDeclarationType::Infer,
                    var_name: "third_var".to_owned(),
                    value: (true, TokenSource::dummy_true()).into(),
                }],
                left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

                right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
            }),
            if_token: TokenSource::dummy_if(),
            left_curley_brace_token: TokenSource::dummy_right_curley_brace(),
            left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

            right_curley_brace_token: TokenSource::dummy_right_curley_brace(),
            right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
        };

        let result = if_statement.type_check(&HashMap::new(), &HashMap::new(), None);

        assert!(matches!(result, Err(_)));
    }

    #[test]
    fn type_check_if_statement_else_if_check_expression_errors() {
        let if_statement = IfStatement {
            check_expression: (true, TokenSource::dummy_true()).into(),
            if_block: vec![Node::VariableDeclaration {
                var_type: VariableDeclarationType::Infer,
                var_name: "my_var".to_owned(),
                value: (true, TokenSource::dummy_true()).into(),
            }],
            else_if_blocks: vec![ElseIfBlock {
                check: Expression::Operation(Operation::Binary {
                    operation: BinaryOperation::GreaterThan,
                    left: Box::new((true, TokenSource::dummy_true()).into()),
                    right: Box::new((10, TokenSource::dummy_uint(10)).into()),
                    operator_token: TokenSource::dummy(Token::RightAngle),
                }),
                block: vec![Node::VariableDeclaration {
                    var_type: VariableDeclarationType::Infer,
                    var_name: "second_var".to_owned(),
                    value: (false, TokenSource::dummy_false()).into(),
                }],
                else_token: TokenSource::dummy_else(),
                if_token: TokenSource::dummy_if(),
                left_curley_brace_token: TokenSource::dummy_right_curley_brace(),
                left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

                right_curley_brace_token: TokenSource::dummy_right_curley_brace(),
                right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
            }],
            else_block: Some(ElseBlock {
                nodes: vec![Node::VariableDeclaration {
                    var_type: VariableDeclarationType::Infer,
                    var_name: "third_var".to_owned(),
                    value: (true, TokenSource::dummy_true()).into(),
                }],
                left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

                right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
            }),
            if_token: TokenSource::dummy_if(),
            left_curley_brace_token: TokenSource::dummy_right_curley_brace(),
            left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

            right_curley_brace_token: TokenSource::dummy_right_curley_brace(),
            right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
        };

        let result = if_statement.type_check(&HashMap::new(), &HashMap::new(), None);

        assert!(matches!(result, Err(_)));
    }
}
