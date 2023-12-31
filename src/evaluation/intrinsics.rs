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
