use std::collections::HashMap;

use crate::ast::node::{Expression, Function, FunctionCall, FunctionId, FunctionParameter, Type};

use super::{verify_type, TypeCheckingError};

impl FunctionCall {
    pub(super) fn type_check(
        &self,
        functions: &HashMap<FunctionId, Function>,
        local_variables: &HashMap<String, Expression>,
    ) -> Result<(), Vec<TypeCheckingError>> {
        let function = functions.get(&self.function_id);

        let Some(function) = function else {
            return Err(vec![TypeCheckingError{message: format!("Could not find function with name {}", self.function_id)}])
        };

        let function_params = function.parameters();

        if self.parameters.len() != function_params.len() {
            return Err(vec![TypeCheckingError {
                message: format!(
                    "{} expects {} parameters, but you provided {}",
                    function.name(),
                    function_params.len(),
                    self.parameters.len()
                ),
            }]);
        }

        let errors: Vec<_> = self
            .parameters
            .iter()
            .enumerate()
            .map(|(i, param_expression)| {
                (
                    &function_params[i],
                    param_expression.get_type(functions, local_variables),
                )
            })
            .map(|(function_param, param_type)| function_param.verify_type(param_type))
            .filter_map(|result| result.err())
            .collect();

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
                    message: format!("Expecte parameter {} to be present", self.name()),
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

    use crate::ast::node::{
        BoolValue, Expression, Function, FunctionCall, FunctionId, FunctionParameter,
        FunctionReturnType, Type, UIntValue, Value,
    };

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

        let function_call = FunctionCall {
            function_id: FunctionId("my_function".to_owned()),
            parameters: vec![Expression::ValueLiteral(Value::Boolean(BoolValue(true)))],
        };

        let result = function_call.type_check(&functions, &HashMap::new());

        assert!(matches!(result, Ok(())));
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

        let function_call = FunctionCall {
            function_id: FunctionId("my_function".to_owned()),
            parameters: vec![
                Expression::ValueLiteral(Value::UInt(UIntValue(10))),
                Expression::ValueLiteral(Value::Boolean(BoolValue(true))),
            ],
        };

        let result = function_call.type_check(&functions, &HashMap::new());

        assert!(matches!(result, Err(e) if e.len() == 2));
    }

    #[test]
    fn function_call_type_check_missing_function() {
        let function_call = FunctionCall {
            function_id: FunctionId("my_function".to_owned()),
            parameters: vec![Expression::ValueLiteral(Value::UInt(UIntValue(10)))],
        };

        let result = function_call.type_check(&HashMap::new(), &HashMap::new());

        assert!(matches!(result, Err(_)))
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

        let function_call = FunctionCall {
            function_id: FunctionId("my_function".to_owned()),
            parameters: vec![
                Expression::ValueLiteral(Value::UInt(UIntValue(10))),
                Expression::ValueLiteral(Value::Boolean(BoolValue(true))),
            ],
        };

        let result = function_call.type_check(&functions, &HashMap::new());

        assert!(matches!(result, Err(_)));
    }
}
