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
            Operation::Unary { operation, value, .. } => {
                unary_operation(*operation, value, functions, local_variables, call_stack)
            }
            Operation::Binary {
                operation,
                left,
                right,
                ..
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
        ast::node::{BinaryOperation, Operation, UnaryOperation},
        evaluation::operation::greater_than,
        token_stream::token::{Token, TokenSource},
    };

    use super::{binary_operation, not, plus, unary_operation};

    #[test]
    fn test_plus() {
        let result = plus(10.into(), 15.into());

        assert_eq!(result, 25.into());
    }

    #[test]
    #[should_panic]
    fn test_plus_incorrect_left_value() {
        plus(true.into(), 10.into());
    }

    #[test]
    #[should_panic]
    fn test_plus_incorrect_right_value() {
        plus(10.into(), true.into());
    }

    #[test]
    fn test_greater_than_false() {
        let result = greater_than(10.into(), 15.into());

        assert_eq!(result, false.into());
    }

    #[test]
    fn test_greater_than_false_equal() {
        let result = greater_than(10.into(), 10.into());

        assert_eq!(result, false.into());
    }

    #[test]
    fn test_greater_than_true() {
        let result = greater_than(15.into(), 10.into());

        assert_eq!(result, true.into());
    }

    #[test]
    #[should_panic]
    fn test_greater_than_incorrect_left_value() {
        greater_than(true.into(), 10.into());
    }

    #[test]
    #[should_panic]
    fn test_greater_than_incorrect_right_value() {
        greater_than(10.into(), true.into());
    }

    #[test]
    fn test_binary_operation_plus() {
        let result = binary_operation(
            BinaryOperation::Plus,
            &(10, TokenSource::dummy_uint(10)).into(),
            &(10, TokenSource::dummy_uint(10)).into(),
            &HashMap::new(),
            &HashMap::new(),
            &mut Vec::new(),
        );

        assert_eq!(result, 20.into());
    }

    #[test]
    fn test_binary_operation_greater_than() {
        let result = binary_operation(
            BinaryOperation::GreaterThan,
            &(10, TokenSource::dummy_uint(10)).into(),
            &(10, TokenSource::dummy_uint(10)).into(),
            &HashMap::new(),
            &HashMap::new(),
            &mut Vec::new(),
        );

        assert_eq!(result, false.into());
    }

    #[test]
    fn test_not() {
        let result = not(true.into());
        assert_eq!(result, false.into());
    }

    #[test]
    #[should_panic]
    fn test_not_incorrect_value() {
        not(10.into());
    }

    #[test]
    fn unary_operation_not() {
        let result = unary_operation(
            UnaryOperation::Not,
            &(true, TokenSource::dummy_true()).into(),
            &HashMap::new(),
            &HashMap::new(),
            &mut Vec::new(),
        );

        assert_eq!(result, false.into());
    }

    #[test]
    fn evaluate_unary() {
        let result = Operation::Unary {
            operation: UnaryOperation::Not,
            value: Box::new((true, TokenSource::dummy_true()).into()),
            operator_token: TokenSource::dummy(Token::NotOperator),
        }
        .evaluate(&HashMap::new(), &HashMap::new(), &mut Vec::new());

        assert_eq!(result, false.into())
    }

    #[test]
    fn evaluate_binary() {
        let result = Operation::Binary {
            operation: BinaryOperation::Plus,
            left: Box::new((10, TokenSource::dummy_uint(10)).into()),
            right: Box::new((10, TokenSource::dummy_uint(10)).into()),
            operator_token: TokenSource::dummy(Token::PlusOperator),
        }
        .evaluate(&HashMap::new(), &HashMap::new(), &mut Vec::new());

        assert_eq!(result, 20.into());
    }
}
