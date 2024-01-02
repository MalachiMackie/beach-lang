use std::collections::HashMap;

use crate::ast::node::{
    BinaryOperation, Function, FunctionId, Operation, Type, UnaryOperation, Value,
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
        local_variables: &HashMap<String, Value>,
    ) -> Result<(), Vec<TypeCheckingError>> {
        match self {
            Operation::Unary {
                operation: UnaryOperation::Not,
                value,
            } => {
                let value_type = value.get_type(functions, local_variables);
                verify_type(value_type, Type::Boolean).map_err(|err| vec![err])
            }
            Operation::Binary {
                operation: BinaryOperation::Plus | BinaryOperation::GreaterThan,
                left,
                right,
            } => {
                let left_type = left.get_type(functions, local_variables);
                let right_type = right.get_type(functions, local_variables);

                let errors: Vec<_> = vec![
                    verify_type(left_type, Type::UInt).err(),
                    verify_type(right_type, Type::UInt).err(),
                ]
                .into_iter()
                .filter_map(|x| x)
                .collect();

                if errors.len() == 0 {
                    Ok(())
                } else {
                    Err(errors)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::ast::node::{
        BinaryOperation, BoolValue, Expression, Operation, Type, UIntValue, UnaryOperation, Value,
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
}
