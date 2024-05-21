use std::collections::HashMap;

use crate::ast::node::{Expression, FunctionCall, FunctionId, FunctionReturnType, Value};

use super::Functions;

impl Expression {
    pub fn evaluate(
        &self,
        functions: &Functions,
        local_variables: &HashMap<String, Value>,
        call_stack: &mut Vec<FunctionId>,
    ) -> Value {
        match self {
            Expression::ValueLiteral(value, _) => value.clone(),
            Expression::FunctionCall(function_call) => {
                evaluate_function_call(function_call, functions, local_variables, call_stack)
            }
            Expression::Operation(operation) => {
                operation.evaluate(functions, local_variables, call_stack)
            }
            Expression::VariableAccess(variable_name, _) => local_variables
                .get(variable_name)
                .expect("variable should exist")
                .clone(),
        }
    }
}

fn evaluate_function_call(
    function_call: &FunctionCall,
    functions: &Functions,
    local_variables: &HashMap<String, Value>,
    call_stack: &mut Vec<FunctionId>,
) -> Value {
    let function = &functions[&function_call.function_id];
    if matches!(function.return_type(), FunctionReturnType::Void) {
        panic!("Function expected to be value, but is void");
    };

    function
        .evaluate(
            function_call.parameters.clone(),
            local_variables,
            functions,
            call_stack,
        )
        .expect("function has a non void return type")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        ast::node::{
            Expression, Function, FunctionCall, FunctionId, FunctionParameter, FunctionReturnType,
            Node, Operation, Type, UnaryOperation,
        },
        token_stream::token::{Token, TokenSource},
    };

    use super::evaluate_function_call;

    #[test]
    fn test_evaluate_function_call() {
        let functions = HashMap::from_iter([(
            FunctionId("my_function".to_owned()),
            Function::CustomFunction {
                id: FunctionId("my_function".to_owned()),
                name: "my_function".to_owned(),
                parameters: vec![FunctionParameter::FunctionParameter {
                    param_type: Type::Boolean,
                    param_name: "bool_param".to_owned(),
                }],
                return_type: FunctionReturnType::Type {
                    return_type: Type::UInt,
                    function_signiture_separator_token: TokenSource::dummy(
                        Token::FunctionSignitureSplitter,
                    ),
                    type_token: TokenSource::dummy(Token::TypeKeyword(Type::UInt)),
                },
                body: vec![Node::FunctionReturn {
                    return_value: Some((10, TokenSource::dummy_uint(10)).into()),
                }],
            },
        )]);

        let function_call = FunctionCall {
            function_id: FunctionId("my_function".to_owned()),
            parameters: vec![(true, TokenSource::dummy_true()).into()],
            comma_tokens: Vec::new(),
            function_id_token: TokenSource::dummy(Token::Identifier("my_function".to_owned())),
            left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

            right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
        };

        let result =
            evaluate_function_call(&function_call, &functions, &HashMap::new(), &mut Vec::new());

        assert_eq!(result, 10.into())
    }

    #[test]
    #[should_panic]
    fn test_evaluate_function_call_void() {
        let functions = HashMap::from_iter([(
            FunctionId("my_function".to_owned()),
            Function::CustomFunction {
                id: FunctionId("my_function".to_owned()),
                name: "my_function".to_owned(),
                parameters: vec![FunctionParameter::FunctionParameter {
                    param_type: Type::Boolean,
                    param_name: "bool_param".to_owned(),
                }],
                return_type: FunctionReturnType::Void,
                body: vec![Node::FunctionReturn { return_value: None }],
            },
        )]);

        let function_call = FunctionCall {
            function_id: FunctionId("my_function".to_owned()),
            parameters: vec![(true, TokenSource::dummy_true()).into()],
            comma_tokens: Vec::new(),
            function_id_token: TokenSource::dummy(Token::Identifier("my_function".to_owned())),
            left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

            right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
        };

        evaluate_function_call(&function_call, &functions, &HashMap::new(), &mut Vec::new());
    }

    #[test]
    fn expression_value_literal() {
        let result = Expression::ValueLiteral(true.into(), TokenSource::dummy_true()).evaluate(
            &HashMap::new(),
            &HashMap::new(),
            &mut Vec::new(),
        );

        assert_eq!(result, true.into())
    }

    #[test]
    fn expression_function_call() {
        let functions = HashMap::from_iter([(
            FunctionId("my_function".to_owned()),
            Function::CustomFunction {
                id: FunctionId("my_function".to_owned()),
                name: "my_function".to_owned(),
                parameters: vec![FunctionParameter::FunctionParameter {
                    param_type: Type::Boolean,
                    param_name: "bool_param".to_owned(),
                }],
                return_type: FunctionReturnType::Type {
                    return_type: Type::UInt,
                    function_signiture_separator_token: TokenSource::dummy(
                        Token::FunctionSignitureSplitter,
                    ),
                    type_token: TokenSource::dummy(Token::TypeKeyword(Type::UInt)),
                },
                body: vec![Node::FunctionReturn {
                    return_value: Some((10, TokenSource::dummy_uint(10)).into()),
                }],
            },
        )]);

        let function_call = Expression::FunctionCall(FunctionCall {
            function_id: FunctionId("my_function".to_owned()),
            parameters: vec![(true, TokenSource::dummy_true()).into()],
            comma_tokens: Vec::new(),
            function_id_token: TokenSource::dummy(Token::Identifier("my_function".to_owned())),
            left_parenthesis_token: TokenSource::dummy_left_parenthesis(),

            right_parenthesis_token: TokenSource::dummy_right_parenthesis(),
        });

        let result = function_call.evaluate(&functions, &HashMap::new(), &mut Vec::new());

        assert_eq!(result, 10.into());
    }

    #[test]
    fn expression_operation() {
        let expression = Expression::Operation(Operation::Unary {
            operation: UnaryOperation::Not,
            value: Box::new((true, TokenSource::dummy_true()).into()),
            operator_token: TokenSource::dummy(Token::NotOperator),
        });

        let result = expression.evaluate(&HashMap::new(), &HashMap::new(), &mut Vec::new());

        assert_eq!(result, false.into());
    }

    #[test]
    fn expression_variable_access() {
        let expression = Expression::VariableAccess(
            "my_var".to_owned(),
            TokenSource::dummy(Token::Identifier("my_var".to_owned())),
        );

        let local_variables = HashMap::from_iter([("my_var".to_owned(), true.into())]);

        let result = expression.evaluate(&HashMap::new(), &local_variables, &mut Vec::new());

        assert_eq!(result, true.into());
    }
}
