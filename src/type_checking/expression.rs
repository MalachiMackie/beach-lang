use std::collections::HashMap;

use crate::ast::node::{
    Expression, Function, FunctionCall, FunctionId, FunctionParameter, FunctionReturnType, Type,
    Value,
};

use super::{verify_type, TypeCheckingError};

impl Expression {
    pub fn get_type(
        &self,
        functions: &HashMap<FunctionId, Function>,
        local_variables: &HashMap<String, Expression>,
    ) -> Option<Type> {
        match self {
            Expression::ValueLiteral(value) => Some(value.get_type()),
            Expression::FunctionCall(function_call) => {
                let function = &functions[&function_call.function_id];

                match function.return_type() {
                    FunctionReturnType::Void => None,
                    FunctionReturnType::Type(return_type) => Some(return_type.clone()),
                }
            }
            Expression::Operation(operation) => Some(operation.get_type()),
            Expression::VariableAccess(var_name) => local_variables
                .get(var_name)
                .and_then(|x| x.get_type(functions, local_variables)),
        }
    }
}

fn type_check_variable_access(
    var_name: &str,
    local_variables: &HashMap<String, Expression>,
) -> Result<(), TypeCheckingError> {
    local_variables
        .get(var_name)
        .ok_or_else(|| TypeCheckingError {
            message: format!("Could not find variable with name {}", var_name),
        })
        .map(|_| ())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::ast::node::{
        BoolValue, Expression, Function, FunctionCall, FunctionId, FunctionReturnType, Operation,
        Type, UnaryOperation, Value,
    };

    use super::type_check_variable_access;

    #[test]
    fn expression_get_type_value_literal() {
        let result = Expression::ValueLiteral(Value::Boolean(BoolValue(true)))
            .get_type(&HashMap::new(), &HashMap::new());

        assert_eq!(result, Some(Type::Boolean));
    }

    #[test]
    fn expression_get_type_operation() {
        let expression = Expression::Operation(Operation::Unary {
            operation: UnaryOperation::Not,
            value: Box::new(Expression::ValueLiteral(Value::Boolean(BoolValue(true)))),
        });

        let result = expression.get_type(&HashMap::new(), &HashMap::new());

        assert_eq!(result, Some(Type::Boolean));
    }

    #[test]
    fn expression_get_type_function_call_return_value() {
        let expression = Expression::FunctionCall(FunctionCall {
            function_id: FunctionId("my_function".to_owned()),
            parameters: Vec::new(),
        });

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

        let result = expression.get_type(&functions, &HashMap::new());

        assert_eq!(result, Some(Type::UInt))
    }

    #[test]
    fn expression_get_type_function_call_void() {
        let expression = Expression::FunctionCall(FunctionCall {
            function_id: FunctionId("my_function".to_owned()),
            parameters: Vec::new(),
        });

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

        let result = expression.get_type(&functions, &HashMap::new());

        assert_eq!(result, None)
    }

    #[test]
    fn expression_get_type_variable_access() {
        let expression = Expression::VariableAccess("my_var".to_owned());

        let local_variables = HashMap::from_iter([(
            "my_var".to_owned(),
            Expression::ValueLiteral(Value::Boolean(BoolValue(true))),
        )]);

        let result = expression.get_type(&HashMap::new(), &local_variables);

        assert_eq!(result, Some(Type::Boolean));
    }

    #[test]
    fn expression_get_type_variable_access_missing_variable() {
        let expression = Expression::VariableAccess("my_var".to_owned());

        let result = expression.get_type(&HashMap::new(), &HashMap::new());

        assert_eq!(result, None);
    }

    #[test]
    fn test_type_check_variable_access_success() {
        let variables = HashMap::from_iter([(
            "my_var".to_owned(),
            Expression::ValueLiteral(Value::Boolean(BoolValue(true))),
        )]);

        let result = type_check_variable_access("my_var", &variables);

        assert!(matches!(result, Ok(_)));
    }

    #[test]
    fn test_type_check_variable_access_failure() {
        let variables = HashMap::new();
        let result = type_check_variable_access("my_var", &variables);

        assert!(
            matches!(result, Err(e) if e.message == "Could not find variable with name my_var")
        );
    }
}
