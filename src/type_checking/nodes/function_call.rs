use std::collections::HashMap;

use crate::{
    ast::node::{Expression, Function, FunctionCall, FunctionId, FunctionParameter, Type},
    type_checking::{verify_type, TypeCheckingError},
};

impl FunctionCall {
    pub fn type_check(
        &self,
        functions: &HashMap<FunctionId, Function>,
        local_variables: &HashMap<String, Type>,
    ) -> Result<(), Vec<TypeCheckingError>> {
        let mut errors = Vec::new();
        let function = functions.get(&self.function_id);

        // type check each of the parameter expressions (regardless of if they are the correct parameters for the function)
        errors.extend(
            self.parameters
                .iter()
                .filter_map(|param| param.type_check(functions, local_variables).err())
                .flat_map(|x| x),
        );

        // if the found a valid function
        if let Some(function) = function {
            let function_params = function.parameters();

            // check we hav ethe correct number of parameters
            if self.parameters.len() != function_params.len() {
                errors.push(TypeCheckingError {
                    message: format!(
                        "{} expects {} parameter(s), but you provided {}",
                        function.name(),
                        function_params.len(),
                        self.parameters.len()
                    ),
                });
            }

            // check that all of our parameters expressions have the correct type for it's corresponding parameter
            errors.extend(
                self.parameters
                    .iter()
                    .enumerate()
                    .filter_map(|(i, param_expression)| {
                        function_params.get(i).map(|function_param| {
                            (
                                function_param,
                                param_expression.get_type(functions, local_variables),
                            )
                        })
                    })
                    .map(|(function_param, param_type)| function_param.verify_type(param_type))
                    .filter_map(|result| result.err()),
            );
        } else {
            // we didn't find a valid function, add an error
            errors.push(TypeCheckingError {
                message: format!("Could not find function with name {}", self.function_id),
            });
        };

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl FunctionParameter {
    pub fn verify_type(&self, found_type: Option<Type>) -> Result<(), TypeCheckingError> {
        match self {
            FunctionParameter::IntrinsicAny { .. } if found_type.is_none() => {
                Err(TypeCheckingError {
                    message: format!("Expected parameter {} to be present", self.name()),
                })
            }
            FunctionParameter::IntrinsicAny { .. } => Ok(()),
            FunctionParameter::FunctionParameter { param_type, .. } => {
                verify_type(found_type, *param_type)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{ast::node::{
        BinaryOperation, BoolValue, Expression, Function, FunctionCall, FunctionId,
        FunctionParameter, FunctionReturnType, Node, Operation, Type, UIntValue, Value,
    }, type_checking::nodes::node::NodeTypeCheckResult};

    #[test]
    fn function_parameter_verify_success() {
        let function_parameter = FunctionParameter::FunctionParameter {
            param_type: Type::Boolean,
            param_name: "hi".to_owned(),
        };

        let result = function_parameter.verify_type(Some(Type::Boolean));

        assert!(matches!(result, Ok(_)));
    }

    #[test]
    fn function_parameter_verify_intrinsic_any_success() {
        let function_parameter = FunctionParameter::IntrinsicAny {
            param_name: "hi".to_owned(),
        };

        let result = function_parameter.verify_type(Some(Type::UInt));

        assert!(matches!(result, Ok(())));
    }

    #[test]
    fn function_parameter_verify_failure() {
        let function_parameter = FunctionParameter::FunctionParameter {
            param_type: Type::Boolean,
            param_name: "hi".to_owned(),
        };

        let result = function_parameter.verify_type(Some(Type::UInt));

        assert!(matches!(result, Err(_)));
    }

    #[test]
    fn function_parameter_verify_intrinsic_any_failure() {
        let function_parameter = FunctionParameter::IntrinsicAny {
            param_name: "hi".to_owned(),
        };

        let result = function_parameter.verify_type(None);

        assert!(matches!(result, Err(_)));
    }

    #[test]
    fn function_call_type_check_success() {
        let functions = HashMap::from_iter([(
            FunctionId("my_function".to_owned()),
            Function::CustomFunction {
                id: FunctionId("my_function".to_owned()),
                name: "my_function".to_owned(),
                parameters: vec![FunctionParameter::FunctionParameter {
                    param_type: Type::Boolean,
                    param_name: "hi".to_owned(),
                }],
                return_type: FunctionReturnType::Void,
                body: Vec::new(),
            },
        )]);

        let function_call = Node::FunctionCall(FunctionCall {
            function_id: FunctionId("my_function".to_owned()),
            parameters: vec![true.into()],
        });

        let result = function_call.type_check(&functions, &mut HashMap::new(), None);

        assert!(matches!(result, NodeTypeCheckResult::DidNotReturnedFromFunction(Ok(()))));
    }

    #[test]
    fn function_call_type_check_failure() {
        let functions = HashMap::from_iter([(
            FunctionId("my_function".to_owned()),
            Function::CustomFunction {
                id: FunctionId("my_function".to_owned()),
                name: "my_function".to_owned(),
                parameters: vec![
                    FunctionParameter::FunctionParameter {
                        param_type: Type::Boolean,
                        param_name: "hi".to_owned(),
                    },
                    FunctionParameter::FunctionParameter {
                        param_type: Type::UInt,
                        param_name: "hi_2".to_owned(),
                    },
                ],
                return_type: FunctionReturnType::Void,
                body: Vec::new(),
            },
        )]);

        let function_call = Node::FunctionCall(FunctionCall {
            function_id: FunctionId("my_function".to_owned()),
            parameters: vec![10.into(), true.into()],
        });

        let result = function_call.type_check(&functions, &mut HashMap::new(), None);

        assert!(matches!(result, NodeTypeCheckResult::DidNotReturnedFromFunction(Err(e)) if e.len() == 2));
    }

    #[test]
    fn function_call_type_check_missing_function() {
        let function_call = Node::FunctionCall(FunctionCall {
            function_id: FunctionId("my_function".to_owned()),
            parameters: vec![10.into()],
        });

        let result = function_call.type_check(&HashMap::new(), &mut HashMap::new(), None);

        assert!(matches!(result, NodeTypeCheckResult::DidNotReturnedFromFunction(Err(_))))
    }

    #[test]
    fn function_call_type_check_incorrect_parameters_number() {
        let functions = HashMap::from_iter([(
            FunctionId("my_function".to_owned()),
            Function::CustomFunction {
                id: FunctionId("my_function".to_owned()),
                name: "my_function".to_owned(),
                parameters: vec![FunctionParameter::FunctionParameter {
                    param_type: Type::Boolean,
                    param_name: "hi".to_owned(),
                }],
                return_type: FunctionReturnType::Void,
                body: Vec::new(),
            },
        )]);

        let function_call = Node::FunctionCall(FunctionCall {
            function_id: FunctionId("my_function".to_owned()),
            parameters: vec![10.into(), true.into()],
        });

        let result = function_call.type_check(&functions, &mut HashMap::new(), None);

        assert!(matches!(result, NodeTypeCheckResult::DidNotReturnedFromFunction(Err(_))));
    }

    #[test]
    fn function_call_type_check_parameter_expressions() {
        let functions = HashMap::from_iter([(
            FunctionId("my_function".to_owned()),
            Function::CustomFunction {
                id: FunctionId("my_function".to_owned()),
                name: "my_function".to_owned(),
                parameters: vec![FunctionParameter::FunctionParameter {
                    param_type: Type::UInt,
                    param_name: "hi".to_owned(),
                }],
                return_type: FunctionReturnType::Void,
                body: Vec::new(),
            },
        )]);

        let function_call = Node::FunctionCall(FunctionCall {
            function_id: FunctionId("my_function".to_owned()),
            parameters: vec![Expression::Operation(Operation::Binary {
                operation: BinaryOperation::Plus,
                left: Box::new(true.into()),
                right: Box::new(10.into()),
            })],
        });

        let result = function_call.type_check(&functions, &mut HashMap::new(), None);

        assert!(matches!(result, NodeTypeCheckResult::DidNotReturnedFromFunction(Err(_))));
    }
}
