use std::collections::HashMap;

use crate::{
    ast::node::{Expression, Function, FunctionId, FunctionReturnType, Type},
    type_checking::TypeCheckingError,
};

pub(super) fn type_check_return_value(
    return_value: Option<&Expression>,
    functions: &HashMap<FunctionId, Function>,
    local_variables: &mut HashMap<String, Type>,
    current_function: Option<&FunctionId>,
) -> Result<(), Vec<TypeCheckingError>> {
    let mut errors = Vec::new();

    let return_value_type = return_value.and_then(|x| x.get_type(functions, local_variables));

    if let Some(Err(expression_errors)) =
        return_value.map(|return_value| return_value.type_check(functions, local_variables))
    {
        errors.extend(expression_errors)
    }

    if let Some(current_function_id) = current_function {
        let function = functions
            .get(current_function_id)
            .expect("current_function should only be set with valid functions");

        match (function.return_type(), return_value_type) {
            // void and some return value
            (FunctionReturnType::Void, Some(return_value_type)) => {
                errors.push(TypeCheckingError {
                    message: format!(
                        "{} is a void function, but you returned a {} value",
                        function.name(),
                        return_value_type
                    ),
                });
            }
            // non void and no return value
            (FunctionReturnType::Type(expected_return_type), None) => {
                errors.push(TypeCheckingError {
                    message: format!(
                        "{} expects a return type of {}, but you returned void",
                        function.name(),
                        expected_return_type
                    ),
                });
            }
            // non void and incorrect return value
            (FunctionReturnType::Type(expected_return_type), Some(return_value_type))
                if *expected_return_type != return_value_type =>
            {
                errors.push(TypeCheckingError {
                    message: format!(
                        "{} expects a return type of {}, but you returned a {} value",
                        function.name(),
                        expected_return_type,
                        return_value_type
                    ),
                })
            }
            _ => {
                // all is ok here
            }
        }
    } else {
        // no current function, top level statements
        match return_value_type {
            None => {}
            // can return uint from top level statements. It's the exit code
            Some(Type::UInt) => {}
            Some(return_value_type) => errors.push(TypeCheckingError {
                message: format!("Cannot return a {return_value_type} from a top level statement"),
            }),
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        ast::node::{
            BinaryOperation, BoolValue, Expression, Function, FunctionId, FunctionReturnType, Node,
            Operation, Type, UIntValue, UnaryOperation, Value,
        },
        type_checking::nodes::node::NodeTypeCheckResult,
    };

    #[test]
    fn type_check_return_value_successful_empty_call_stack() {
        let node = Node::FunctionReturn {
            return_value: Some(10.into()),
        };

        let result = node.type_check(&HashMap::new(), &mut HashMap::new(), None);

        assert!(matches!(
            result,
            NodeTypeCheckResult::ReturnedFromFunction(Ok(()))
        ))
    }

    #[test]
    fn type_check_return_void_successful_empty_call_stack() {
        let node = Node::FunctionReturn { return_value: None };

        let result = node.type_check(&HashMap::new(), &mut HashMap::new(), None);

        assert!(matches!(
            result,
            NodeTypeCheckResult::ReturnedFromFunction(Ok(()))
        ))
    }

    #[test]
    fn type_check_return_value_successful_in_function() {
        let node = Node::FunctionReturn {
            return_value: Some(true.into()),
        };

        let functions = HashMap::from_iter([(
            FunctionId("my_function".to_owned()),
            Function::CustomFunction {
                id: FunctionId("my_function".to_owned()),
                name: "my_function".to_owned(),
                parameters: Vec::new(),
                return_type: FunctionReturnType::Type(Type::Boolean),
                body: Vec::new(),
            },
        )]);

        let current_function = FunctionId("my_function".to_owned());

        let result = node.type_check(&functions, &mut HashMap::new(), Some(&current_function));

        assert!(matches!(
            result,
            NodeTypeCheckResult::ReturnedFromFunction(Ok(()))
        ))
    }

    #[test]
    fn type_check_return_void_successfull_in_function() {
        let node = Node::FunctionReturn { return_value: None };

        let functions = HashMap::from_iter([(
            FunctionId("my_function".to_owned()),
            Function::CustomFunction {
                id: FunctionId("my_function".to_owned()),
                name: "my_function".to_owned(),
                parameters: Vec::new(),
                return_type: FunctionReturnType::Void,
                body: Vec::new(),
            },
        )]);

        let current_function = FunctionId("my_function".to_owned());

        let result = node.type_check(&functions, &mut HashMap::new(), Some(&current_function));

        assert!(matches!(
            result,
            NodeTypeCheckResult::ReturnedFromFunction(Ok(()))
        ));
    }

    #[test]
    fn type_check_return_expression_failure() {
        let node = Node::FunctionReturn {
            return_value: Some(Expression::Operation(Operation::Binary {
                operation: BinaryOperation::Plus,
                left: Box::new(10.into()),
                right: Box::new(true.into()),
            })),
        };

        let result = node.type_check(&HashMap::new(), &mut HashMap::new(), None);

        assert!(
            matches!(result, NodeTypeCheckResult::ReturnedFromFunction(Err(e)) if e.len() == 1 && e[0].message == "Expected type to be UInt, but found Boolean")
        );
    }

    #[test]
    fn type_check_return_top_level_incorrect_type() {
        let node = Node::FunctionReturn {
            return_value: Some(true.into()),
        };

        let result = node.type_check(&HashMap::new(), &mut HashMap::new(), None);

        assert!(
            matches!(result, NodeTypeCheckResult::ReturnedFromFunction(Err(e)) if e.len() == 1 && e[0].message == "Cannot return a Boolean from a top level statement")
        );
    }

    #[test]
    fn type_check_return_incorrect_value_from_function() {
        let node = Node::FunctionReturn {
            return_value: Some(true.into()),
        };

        let functions = HashMap::from_iter([(
            FunctionId("my_function".to_owned()),
            Function::CustomFunction {
                id: FunctionId("my_function".to_owned()),
                name: "my_function".to_owned(),
                parameters: Vec::new(),
                return_type: FunctionReturnType::Type(Type::UInt),
                body: Vec::new(),
            },
        )]);

        let result = node.type_check(
            &functions,
            &mut HashMap::new(),
            Some(&FunctionId("my_function".to_owned())),
        );

        assert!(
            matches!(result, NodeTypeCheckResult::ReturnedFromFunction(Err(e)) if e.len() == 1 && e[0].message == "my_function expects a return type of UInt, but you returned a Boolean value")
        )
    }

    #[test]
    fn type_check_return_void_from_function_that_expects_value() {
        let node = Node::FunctionReturn { return_value: None };

        let functions = HashMap::from_iter([(
            FunctionId("my_function".to_owned()),
            Function::CustomFunction {
                id: FunctionId("my_function".to_owned()),
                name: "my_function".to_owned(),
                parameters: Vec::new(),
                return_type: FunctionReturnType::Type(Type::UInt),
                body: Vec::new(),
            },
        )]);

        let result = node.type_check(
            &functions,
            &mut HashMap::new(),
            Some(&FunctionId("my_function".to_owned())),
        );

        assert!(
            matches!(result, NodeTypeCheckResult::ReturnedFromFunction(Err(e)) if e.len() == 1 && e[0].message == "my_function expects a return type of UInt, but you returned void")
        );
    }

    #[test]
    fn type_check_return_value_from_void_function() {
        let node = Node::FunctionReturn {
            return_value: Some(true.into()),
        };

        let functions = HashMap::from_iter([(
            FunctionId("my_function".to_owned()),
            Function::CustomFunction {
                id: FunctionId("my_function".to_owned()),
                name: "my_function".to_owned(),
                parameters: Vec::new(),
                return_type: FunctionReturnType::Void,
                body: Vec::new(),
            },
        )]);

        let result = node.type_check(
            &functions,
            &mut HashMap::new(),
            Some(&FunctionId("my_function".to_owned())),
        );

        assert!(
            matches!(result, NodeTypeCheckResult::ReturnedFromFunction(Err(e)) if e.len() == 1 && e[0].message == "my_function is a void function, but you returned a Boolean value")
        );
    }
}
