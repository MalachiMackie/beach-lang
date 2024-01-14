use std::collections::HashMap;

use crate::ast::node::{Function, FunctionId, FunctionParameter, FunctionReturnType};

use super::{nodes::node::type_check_nodes, TypeCheckingError};

impl Function {
    pub fn type_check(
        &self,
        functions: &HashMap<FunctionId, Function>,
    ) -> Result<(), Vec<TypeCheckingError>> {
        let Function::CustomFunction {
            id,
            parameters,
            body,
            return_type,
            ..
        } = self
        else {
            return Ok(());
        };

        let local_variables: HashMap<_, _> = parameters
            .iter()
            .filter_map(|param| match param {
                FunctionParameter::FunctionParameter {
                    param_type,
                    param_name,
                } => Some((param_name.clone(), *param_type)),
                FunctionParameter::IntrinsicAny { .. } => None,
            })
            .collect();

        let found_return_type =
            match type_check_nodes(&body, functions, &local_variables, Some(&id)) {
                Err(errors) => return Err(errors),
                Ok(return_type) => return_type,
            };

        match (found_return_type, return_type) {
            (None, FunctionReturnType::Type(_)) => Err(vec![TypeCheckingError {
                message: "expected return value, but void was returned".to_owned(),
            }]),
            (Some(_), FunctionReturnType::Void) => unreachable!("return statement type checking should validate that some can't be returned from void function"),
            // expect return type checking to happen at the return site
            (Some(_), FunctionReturnType::Type(_)) => Ok(()),
            (None, FunctionReturnType::Void) => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::ast::node::{
        Expression, Function, FunctionId, FunctionParameter, FunctionReturnType, Node, Type,
    };

    #[test]
    fn type_check_function_success() {
        let function = Function::CustomFunction {
            id: FunctionId("my_function".to_owned()),
            name: "my_function".to_owned(),
            parameters: vec![
                FunctionParameter::FunctionParameter {
                    param_type: Type::Boolean,
                    param_name: "my_var".to_owned(),
                },
                // only here to get the test coverage
                FunctionParameter::IntrinsicAny {
                    param_name: "intrinsic_any".to_owned(),
                },
            ],
            return_type: FunctionReturnType::Type(Type::Boolean),
            body: vec![Node::FunctionReturn {
                return_value: Some(Expression::VariableAccess("my_var".to_owned())),
            }],
        };

        let functions = HashMap::from_iter([(function.id().clone(), function.clone())]);

        let result = function.type_check(&functions);

        assert!(matches!(result, Ok(())));
    }

    #[test]
    fn type_check_intrinsic_function() {
        let function = Function::Intrinsic {
            id: FunctionId("my_function".to_owned()),
            name: "my_function".to_owned(),
            parameters: vec![],
            return_type: FunctionReturnType::Void,
        };

        let result = function.type_check(&HashMap::new());

        assert!(matches!(result, Ok(())));
    }

    #[test]
    fn type_check_function_failure() {
        let function = Function::CustomFunction {
            id: FunctionId("my_function".to_owned()),
            name: "my_function".to_owned(),
            parameters: vec![FunctionParameter::FunctionParameter {
                param_type: Type::Boolean,
                param_name: "my_var".to_owned(),
            }],
            return_type: FunctionReturnType::Type(Type::Boolean),
            body: vec![Node::FunctionReturn {
                return_value: Some(10.into()),
            }],
        };

        let functions = HashMap::from_iter([(function.id().clone(), function.clone())]);

        let result = function.type_check(&functions);

        assert!(matches!(result, Err(_)));
    }

    #[test]
    fn type_check_missing_return_value() {
        let function = Function::CustomFunction {
            id: FunctionId("my_function".to_owned()),
            name: "my_function".to_owned(),
            parameters: Vec::new(),
            return_type: FunctionReturnType::Type(Type::Boolean),
            body: Vec::new(),
        };

        let functions = HashMap::from_iter([(function.id().clone(), function.clone())]);

        let result = function.type_check(&functions);

        assert!(
            matches!(dbg!(result), Err(e) if e.len() == 1 && e[0].message == "expected return value, but void was returned")
        )
    }

    #[test]
    fn type_check_void_no_return() {
        let function = Function::CustomFunction {
            id: FunctionId("my_function".to_owned()),
            name: "my_function".to_owned(),
            parameters: Vec::new(),
            return_type: FunctionReturnType::Void,
            body: Vec::new(),
        };

        let functions = HashMap::from_iter([(function.id().clone(), function.clone())]);

        let result = function.type_check(&functions);


        assert!(matches!(result, Ok(())));
    }
}
