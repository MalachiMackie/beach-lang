use std::collections::HashMap;

use crate::ast::node::{Expression, Function, FunctionId, FunctionReturnType, Type, Value};

impl Expression {
    pub fn get_type(
        &self,
        functions: &HashMap<FunctionId, Function>,
        local_variables: &HashMap<String, Value>,
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
            Expression::VariableAccess(var_name) => {
                local_variables.get(var_name).map(|x| x.get_type())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::ast::node::{
        BoolValue, Expression, Function, FunctionCall, FunctionId, FunctionReturnType, Operation,
        Type, UnaryOperation, Value,
    };

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

        let local_variables =
            HashMap::from_iter([("my_var".to_owned(), Value::Boolean(BoolValue(true)))]);

        let result = expression.get_type(&HashMap::new(), &local_variables);

        assert_eq!(result, Some(Type::Boolean));
    }

    #[test]
    fn expression_get_type_variable_access_missing_variable() {
        let expression = Expression::VariableAccess("my_var".to_owned());

        let result = expression.get_type(&HashMap::new(), &HashMap::new());

        assert_eq!(result, None);
    }
}
