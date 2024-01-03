use std::collections::HashMap;

use crate::ast::node::{Function, FunctionId, FunctionParameter};

use super::{nodes::node::type_check_nodes, TypeCheckingError};

impl Function {
    pub fn type_check(
        &self,
        functions: &HashMap<FunctionId, Function>,
    ) -> Result<(), Vec<TypeCheckingError>> {
        let Function::CustomFunction { id, parameters, body, .. } = self else {
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

        type_check_nodes(&body, functions, &local_variables, Some(&id))
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
            parameters: vec![FunctionParameter::FunctionParameter {
                param_type: Type::Boolean,
                param_name: "my_var".to_owned(),
            }],
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
}
