use std::collections::HashMap;

use crate::ast::node::{Expression, Function, FunctionId, FunctionParameter, Node, Value};

use super::{ast::evaluate_nodes, intrinsics::evaluate_intrinsic_function, Functions, NodeResult};

fn evaluate_custom_function(
    id: &FunctionId,
    body: &[Node],
    parameters: HashMap<String, Value>,
    call_stack: &mut Vec<FunctionId>,
    functions: &Functions,
) -> Option<Value> {
    call_stack.push(id.clone());
    if let NodeResult::FunctionReturn { value } =
        evaluate_nodes(body, &parameters, call_stack, functions)
    {
        call_stack.pop();
        value
    } else {
        call_stack.pop();
        None
    }
}

impl Function {
    pub fn evaluate(
        &self,
        parameter_expressions: Vec<Expression>,
        local_variables: &HashMap<String, Value>,
        functions: &Functions,
        call_stack: &mut Vec<FunctionId>,
    ) -> Option<Value> {
        if parameter_expressions.len() != self.parameters().len() {
            panic!(
                "Expected {} parameters, but found {} for {}",
                self.parameters().len(),
                parameter_expressions.len(),
                self.name()
            );
        }

        let parameter_values: Vec<Value> = parameter_expressions
            .into_iter()
            .map(|expression| expression.evaluate(functions, local_variables, call_stack))
            .collect();

        let local_variables = self
            .parameters()
            .iter()
            .enumerate()
            .map(|(i, function_parameter)| {
                let param_name = match function_parameter {
                    FunctionParameter::FunctionParameter { param_name, .. }
                    | FunctionParameter::IntrinsicAny { param_name } => param_name,
                };
                (param_name.clone(), parameter_values[i].clone())
            })
            .collect();

        match self {
            Function::CustomFunction { id, body, .. } => {
                evaluate_custom_function(id, body, local_variables, call_stack, functions)
            }
            Function::Intrinsic { id, .. } => evaluate_intrinsic_function(id, &local_variables),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::ast::node::{
        Function, FunctionId, FunctionParameter, FunctionReturnType, Node, Type,
    };

    use super::evaluate_custom_function;

    #[test]
    fn test_custom_function_evaluation_return_value() {
        let result = evaluate_custom_function(
            &FunctionId("my_function".to_owned()),
            &[Node::FunctionReturn {
                return_value: Some(1.into()),
            }],
            HashMap::new(),
            &mut Vec::new(),
            &HashMap::new(),
        );

        assert_eq!(result, Some(1.into()))
    }

    #[test]
    fn test_custom_function_evaluation_no_return_value() {
        let result = evaluate_custom_function(
            &FunctionId("my_function".to_owned()),
            &[],
            HashMap::new(),
            &mut Vec::new(),
            &HashMap::new(),
        );

        assert_eq!(result, None);
    }

    #[test]
    fn function_evaluate_custom_function() {
        let function = Function::CustomFunction {
            id: FunctionId("my_function".to_owned()),
            name: "my_function".to_owned(),
            parameters: vec![FunctionParameter::FunctionParameter {
                param_type: Type::UInt,
                param_name: "param".to_owned(),
            }],
            return_type: FunctionReturnType::Type(Type::Boolean),
            body: vec![Node::FunctionReturn {
                return_value: Some(true.into()),
            }],
        };

        let result = function.evaluate(
            vec![1.into()],
            &HashMap::new(),
            &HashMap::new(),
            &mut Vec::new(),
        );

        assert_eq!(result, Some(true.into()))
    }

    #[test]
    fn function_evaluate_intrinsic_function() {
        let function = Function::Intrinsic {
            id: FunctionId("print".to_owned()),
            name: "print".to_owned(),
            parameters: vec![FunctionParameter::IntrinsicAny {
                param_name: "value".to_owned(),
            }],
            return_type: FunctionReturnType::Void,
        };

        let result = function.evaluate(
            vec![1.into()],
            &HashMap::new(),
            &HashMap::new(),
            &mut Vec::new(),
        );

        assert_eq!(result, None);
    }

    #[test]
    #[should_panic]
    fn function_evaluate_incorrect_parameter_number() {
        let function = Function::CustomFunction {
            id: FunctionId("my_function".to_owned()),
            name: "my_function".to_owned(),
            parameters: Vec::new(),
            return_type: FunctionReturnType::Type(Type::Boolean),
            body: vec![Node::FunctionReturn {
                return_value: Some(true.into()),
            }],
        };

        let result = function.evaluate(
            vec![1.into()],
            &HashMap::new(),
            &HashMap::new(),
            &mut Vec::new(),
        );

        assert_eq!(result, Some(true.into()))
    }
}
