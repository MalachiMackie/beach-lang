use crate::ast::node::{BinaryOperation, Operation, Type, UnaryOperation};

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
}

#[cfg(test)]
mod tests {
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
}
