use std::collections::HashMap;

use crate::ast::node::{
    BoolValue, Function, FunctionId, FunctionParameter, FunctionReturnType, UIntValue, Value,
};

pub(super) fn evaluate_intrinsic_function(
    id: &FunctionId,
    parameters: &HashMap<String, Value>,
) -> Option<Value> {
    match id.0.as_str() {
        "print" => {
            intrinsic_print(&parameters["value"]);
            None
        }
        _ => panic!("unknown intrinsic function"),
    }
}

pub fn get_intrinsic_functions() -> HashMap<FunctionId, Function> {
    [Function::Intrinsic {
        id: FunctionId("print".to_owned()),
        name: "print".to_owned(),
        parameters: vec![FunctionParameter::IntrinsicAny {
            param_name: "value".to_owned(),
        }],
        return_type: FunctionReturnType::Void,
    }]
    .into_iter()
    .map(|function| (function.id().clone(), function))
    .collect()
}

fn intrinsic_print(value: &Value) {
    match value {
        Value::Boolean(BoolValue(bool_value)) => println!("{}", bool_value),
        Value::UInt(UIntValue(uint_value)) => println!("{}", uint_value),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::ast::node::{BoolValue, Function, FunctionId, Value, UIntValue};

    use super::{evaluate_intrinsic_function, get_intrinsic_functions};

    #[test]
    fn test_get_intrinsic_functions() {
        let functions = get_intrinsic_functions();
        let keys: Vec<_> = functions.keys().into_iter().collect();

        assert_eq!(keys, vec![&FunctionId("print".to_owned())]);
    }

    #[test]
    fn evaluate_print_bool() {
        let result = evaluate_intrinsic_function(
            &FunctionId("print".to_owned()),
            &[("value".to_owned(), Value::Boolean(BoolValue(true)))]
                .into_iter()
                .collect(),
        );

        assert!(matches!(result, None));
    }

    #[test]
    fn evaluate_print_uint() {
        let result = evaluate_intrinsic_function(
            &FunctionId("print".to_owned()),
            &[("value".to_owned(), Value::UInt(UIntValue(10)))]
                .into_iter()
                .collect(),
        );

        assert!(matches!(result, None));
    }

    #[test]
    #[should_panic]
    fn evaluate_missing_intrinsic() {
        evaluate_intrinsic_function(&FunctionId("unknown".to_owned()), &HashMap::new());
    }
}
