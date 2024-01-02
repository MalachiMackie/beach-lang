use std::collections::HashMap;

use crate::ast::node::{
    BinaryOperation, Expression, Function, FunctionId, Operation, Type, UnaryOperation,
};

use super::{verify_type, TypeCheckingError};

impl Operation {
    pub fn get_type(&self) -> Type {
        match self {
            Operation::Unary {
                operation: UnaryOperation::Not,
                ..
            } => Type::Boolean,
            Operation::Binary {
                operation: BinaryOperation::GreaterThan,
                ..
            } => Type::UInt,
            Operation::Binary {
                operation: BinaryOperation::Plus,
                ..
            } => Type::UInt,
        }
    }

    pub fn type_check(
        &self,
        functions: &HashMap<FunctionId, Function>,
        local_variables: &HashMap<String, Expression>,
    ) -> Result<(), Vec<TypeCheckingError>> {
        match self {
            Operation::Unary { operation, value } => {
                operation.type_check(value, functions, local_variables)
            }
            Operation::Binary {
                operation,
                left,
                right,
            } => operation.type_check(left, right, functions, local_variables),
        }
    }
}

impl BinaryOperation {
    fn type_check(
        &self,
        left: &Expression,
        right: &Expression,
        functions: &HashMap<FunctionId, Function>,
        local_variables: &HashMap<String, Expression>,
    ) -> Result<(), Vec<TypeCheckingError>> {
        let mut errors = Vec::new();

        // get types for the left and right expressions
        let left_type = left.get_type(functions, local_variables);
        let right_type = right.get_type(functions, local_variables);

        // type check the actual left and right expressions
        if let Err(left_errors) = left.type_check(functions, local_variables) {
            errors.extend(left_errors);
        };
        if let Err(right_errors) = right.type_check(functions, local_variables) {
            errors.extend(right_errors);
        };

        // verify that the expression types are correct
        match self {
            BinaryOperation::GreaterThan | BinaryOperation::Plus => {
                if let Err(left_type_error) = verify_type(left_type, Type::UInt) {
                    errors.push(left_type_error);
                };
                if let Err(right_type_error) = verify_type(right_type, Type::UInt) {
                    errors.push(right_type_error);
                };
            }
        }

        // return result
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl UnaryOperation {
    fn type_check(
        &self,
        value: &Expression,
        functions: &HashMap<FunctionId, Function>,
        local_variables: &HashMap<String, Expression>,
    ) -> Result<(), Vec<TypeCheckingError>> {
        let mut errors = Vec::new();

        // get type for the expression
        let value_type = value.get_type(functions, local_variables);

        // type check the actual expression
        if let Err(expression_errors) = value.type_check(functions, local_variables) {
            errors.extend(expression_errors);
        };

        // verify that the expression type is correct
        match self {
            UnaryOperation::Not => {
                if let Err(type_error) = verify_type(value_type, Type::Boolean) {
                    errors.push(type_error);
                };
            }
        }

        // return result
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

    use crate::ast::node::{
        BinaryOperation, BoolValue, Expression, Function, FunctionCall, FunctionId,
        FunctionParameter, FunctionReturnType, Operation, Type, UIntValue, UnaryOperation, Value,
    };

    #[test]
    fn operation_get_type_not() {
        let operation = Operation::Unary {
            operation: UnaryOperation::Not,
            value: Box::new(Expression::ValueLiteral(Value::Boolean(BoolValue(true)))),
        };

        let result = operation.get_type();

        assert_eq!(result, Type::Boolean)
    }

    #[test]
    fn operation_get_type_plus() {
        let operation = Operation::Binary {
            operation: BinaryOperation::Plus,
            left: Box::new(Expression::ValueLiteral(Value::UInt(UIntValue(10)))),
            right: Box::new(Expression::ValueLiteral(Value::UInt(UIntValue(10)))),
        };

        let result = operation.get_type();

        assert_eq!(result, Type::UInt)
    }

    #[test]
    fn operation_get_type_greater_than() {
        let operation = Operation::Binary {
            operation: BinaryOperation::GreaterThan,
            left: Box::new(Expression::ValueLiteral(Value::UInt(UIntValue(10)))),
            right: Box::new(Expression::ValueLiteral(Value::UInt(UIntValue(10)))),
        };

        let result = operation.get_type();

        assert_eq!(result, Type::UInt)
    }

    #[test]
    fn operation_not_type_check_successful() {
        let operation = Operation::Unary {
            operation: UnaryOperation::Not,
            value: Box::new(Expression::ValueLiteral(Value::Boolean(BoolValue(true)))),
        };

        let result = operation.type_check(&HashMap::new(), &HashMap::new());

        assert!(matches!(result, Ok(_)));
    }

    #[test]
    fn operation_not_type_check_failure() {
        let operation = Operation::Unary {
            operation: UnaryOperation::Not,
            value: Box::new(Expression::ValueLiteral(Value::UInt(UIntValue(10)))),
        };

        let result = operation.type_check(&HashMap::new(), &HashMap::new());

        assert!(matches!(result, Err(errors) if errors.len() == 1));
    }

    #[test]
    fn operation_plus_type_check_success() {
        let operation = Operation::Binary {
            operation: BinaryOperation::GreaterThan,
            left: Box::new(Expression::ValueLiteral(Value::UInt(UIntValue(10)))),
            right: Box::new(Expression::ValueLiteral(Value::UInt(UIntValue(10)))),
        };

        let result = operation.type_check(&HashMap::new(), &HashMap::new());

        assert!(matches!(result, Ok(_)));
    }

    #[test]
    fn operation_plus_type_check_failure() {
        let operation = Operation::Binary {
            operation: BinaryOperation::GreaterThan,
            left: Box::new(Expression::ValueLiteral(Value::Boolean(BoolValue(true)))),
            right: Box::new(Expression::ValueLiteral(Value::Boolean(BoolValue(true)))),
        };

        let result = operation.type_check(&HashMap::new(), &HashMap::new());

        assert!(matches!(result, Err(errors) if errors.len() == 2));
    }

    #[test]
    fn operation_unary_type_check_expressions() {
        let operation = Operation::Unary {
            operation: UnaryOperation::Not,
            value: Box::new(Expression::FunctionCall(FunctionCall {
                function_id: FunctionId("my_function".to_owned()),
                parameters: Vec::new(),
            })),
        };

        let functions = HashMap::from_iter([(
            FunctionId("my_function".to_owned()),
            Function::CustomFunction {
                id: FunctionId("my_function".to_owned()),
                name: "my_function".to_owned(),
                parameters: vec![FunctionParameter::FunctionParameter {
                    param_type: Type::Boolean,
                    param_name: "my_param".to_owned(),
                }],
                return_type: FunctionReturnType::Type(Type::Boolean),
                body: vec![],
            },
        )]);

        let result = operation.type_check(&functions, &HashMap::new());

        assert!(
            matches!(result, Err(e) if e.len() == 1 && e[0].message == "my_function expects 1 parameter(s), but you provided 0")
        )
    }

    #[test]
    fn operation_binary_type_check_expressions() {
        let operation = Operation::Binary {
            operation: BinaryOperation::GreaterThan,
            left: Box::new(Expression::FunctionCall(FunctionCall {
                function_id: FunctionId("my_function".to_owned()),
                parameters: Vec::new(),
            })),
            right: Box::new(Expression::FunctionCall(FunctionCall {
                function_id: FunctionId("my_function".to_owned()),
                parameters: Vec::new(),
            })),
        };

        let functions = HashMap::from_iter([(
            FunctionId("my_function".to_owned()),
            Function::CustomFunction {
                id: FunctionId("my_function".to_owned()),
                name: "my_function".to_owned(),
                parameters: vec![FunctionParameter::FunctionParameter {
                    param_type: Type::Boolean,
                    param_name: "my_param".to_owned(),
                }],
                return_type: FunctionReturnType::Type(Type::UInt),
                body: vec![],
            },
        )]);

        let result = operation.type_check(&functions, &HashMap::new());

        assert!(
            matches!(result, Err(e) if e.len() == 2 && e[0].message == "my_function expects 1 parameter(s), but you provided 0" && e[1].message == e[1].message)
        )
    }
}
