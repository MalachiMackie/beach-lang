use std::collections::HashMap;

use crate::ast::node::{
    BinaryOperation, BoolValue, Expression, FunctionId, Operation, UIntValue, UnaryOperation, Value,
};

use super::Functions;

impl Operation {
    pub fn evaluate(
        &self,
        functions: &Functions,
        local_variables: &HashMap<String, Value>,
        call_stack: &mut Vec<FunctionId>,
    ) -> Value {
        match self {
            Operation::Unary { operation, value } => {
                unary_operation(*operation, value, functions, local_variables, call_stack)
            }
            Operation::Binary {
                operation,
                left,
                right,
            } => binary_operation(
                *operation,
                left,
                right,
                functions,
                local_variables,
                call_stack,
            ),
        }
    }
}

fn unary_operation(
    operation: UnaryOperation,
    value: &Expression,
    functions: &Functions,
    local_variables: &HashMap<String, Value>,
    call_stack: &mut Vec<FunctionId>,
) -> Value {
    let value = value.evaluate(functions, local_variables, call_stack);
    match operation {
        UnaryOperation::Not => not(value),
    }
}

fn not(value: Value) -> Value {
    let BoolValue(bool) = value.expect_bool("not only operates on booleans");
    Value::Boolean(BoolValue(!bool))
}

fn binary_operation(
    operation: BinaryOperation,
    left: &Expression,
    right: &Expression,
    functions: &Functions,
    local_variables: &HashMap<String, Value>,
    call_stack: &mut Vec<FunctionId>,
) -> Value {
    let left_value = left.evaluate(functions, local_variables, call_stack);
    let right_value = right.evaluate(functions, local_variables, call_stack);
    match operation {
        BinaryOperation::Plus => plus(left_value, right_value),
        BinaryOperation::GreaterThan => greater_than(left_value, right_value),
    }
}

fn greater_than(left: Value, right: Value) -> Value {
    Value::Boolean(BoolValue(
        left.expect_uint("greater_than only operates on uint").0
            > right.expect_uint("greater_than only operates on uint").0,
    ))
}

fn plus(left: Value, right: Value) -> Value {
    Value::UInt(UIntValue(
        left.expect_uint("plus only operates on uint").0
            + right.expect_uint("plus only operates on uint").0,
    ))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        ast::node::{
            BinaryOperation, BoolValue, Expression, Operation, UIntValue, UnaryOperation, Value,
        },
        evaluation::operation::greater_than,
    };

    use super::{binary_operation, not, plus, unary_operation};

    #[test]
    fn test_plus() {
        let result = plus(Value::UInt(UIntValue(10)), Value::UInt(UIntValue(15)));

        assert_eq!(result, Value::UInt(UIntValue(25)));
    }

    #[test]
    #[should_panic]
    fn test_plus_incorrect_left_value() {
        plus(Value::Boolean(BoolValue(true)), Value::UInt(UIntValue(10)));
    }

    #[test]
    #[should_panic]
    fn test_plus_incorrect_right_value() {
        plus(Value::UInt(UIntValue(10)), Value::Boolean(BoolValue(true)));
    }

    #[test]
    fn test_greater_than_false() {
        let result = greater_than(Value::UInt(UIntValue(10)), Value::UInt(UIntValue(15)));

        assert_eq!(result, Value::Boolean(BoolValue(false)));
    }

    #[test]
    fn test_greater_than_false_equal() {
        let result = greater_than(Value::UInt(UIntValue(10)), Value::UInt(UIntValue(10)));

        assert_eq!(result, Value::Boolean(BoolValue(false)));
    }

    #[test]
    fn test_greater_than_true() {
        let result = greater_than(Value::UInt(UIntValue(15)), Value::UInt(UIntValue(10)));

        assert_eq!(result, Value::Boolean(BoolValue(true)));
    }

    #[test]
    #[should_panic]
    fn test_greater_than_incorrect_left_value() {
        greater_than(Value::Boolean(BoolValue(true)), Value::UInt(UIntValue(10)));
    }

    #[test]
    #[should_panic]
    fn test_greater_than_incorrect_right_value() {
        greater_than(Value::UInt(UIntValue(10)), Value::Boolean(BoolValue(true)));
    }

    #[test]
    fn test_binary_operation_plus() {
        let result = binary_operation(
            BinaryOperation::Plus,
            &Expression::ValueLiteral(Value::UInt(UIntValue(10))),
            &Expression::ValueLiteral(Value::UInt(UIntValue(10))),
            &HashMap::new(),
            &HashMap::new(),
            &mut Vec::new(),
        );

        assert_eq!(result, Value::UInt(UIntValue(20)));
    }

    #[test]
    fn test_binary_operation_greater_than() {
        let result = binary_operation(
            BinaryOperation::GreaterThan,
            &Expression::ValueLiteral(Value::UInt(UIntValue(10))),
            &Expression::ValueLiteral(Value::UInt(UIntValue(10))),
            &HashMap::new(),
            &HashMap::new(),
            &mut Vec::new(),
        );

        assert_eq!(result, Value::Boolean(BoolValue(false)));
    }

    #[test]
    fn test_not() {
        let result = not(Value::Boolean(BoolValue(true)));
        assert_eq!(result, Value::Boolean(BoolValue(false)));
    }

    #[test]
    #[should_panic]
    fn test_not_incorrect_value() {
        not(Value::UInt(UIntValue(10)));
    }

    #[test]
    fn unary_operation_not() {
        let result = unary_operation(
            UnaryOperation::Not,
            &Expression::ValueLiteral(Value::Boolean(BoolValue(true))),
            &HashMap::new(),
            &HashMap::new(),
            &mut Vec::new(),
        );

        assert_eq!(result, Value::Boolean(BoolValue(false)));
    }

    #[test]
    fn evaluate_unary() {
        let result = Operation::Unary {
            operation: UnaryOperation::Not,
            value: Box::new(Expression::ValueLiteral(Value::Boolean(BoolValue(true)))),
        }
        .evaluate(&HashMap::new(), &HashMap::new(), &mut Vec::new());

        assert_eq!(result, Value::Boolean(BoolValue(false)))
    }

    #[test]
    fn evaluate_binary() {
        let result = Operation::Binary {
            operation: BinaryOperation::Plus,
            left: Box::new(Expression::ValueLiteral(Value::UInt(UIntValue(10)))),
            right: Box::new(Expression::ValueLiteral(Value::UInt(UIntValue(10)))),
        }
        .evaluate(&HashMap::new(), &HashMap::new(), &mut Vec::new());

        assert_eq!(result, Value::UInt(UIntValue(20)));
    }
}
