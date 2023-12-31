pub mod intrinsics;

use core::panic;
use std::collections::HashMap;

use crate::ast::node::{
    Ast, BoolValue, Expression, Function, FunctionDeclaration, FunctionId, FunctionParameter,
    FunctionReturnType, Node, Operation, UnaryOperation, Value,
};

use self::intrinsics::evaluate_intrinsic_function;

type Functions = HashMap<FunctionId, Function>;

impl Ast {
    pub fn evaluate(
        &self,
        mut local_variables: HashMap<String, Value>,
        mut functions: HashMap<FunctionId, Function>,
    ) -> Option<Value> {
        let mut call_stack: Vec<FunctionId> = Vec::new();
        let mut return_value = None;

        functions.extend(self.functions.clone());

        for node in &self.nodes {
            return_value = node.evaluate(&mut local_variables, &mut call_stack, &functions)
        }

        return_value
    }
}

impl Node {
    pub fn evaluate(
        &self,
        local_variables: &mut HashMap<String, Value>,
        call_stack: &mut Vec<FunctionId>,
        functions: &Functions,
    ) -> Option<Value> {
        match self {
            Node::VariableDeclaration {
                var_name, value, ..
            } => {
                local_variables.insert(
                    var_name.to_owned(),
                    value.evaluate(functions, &local_variables),
                );
                None
            }
            Node::FunctionReturn { return_value } => {
                let return_value = if let Some(expression) = return_value {
                    let value = expression.evaluate(functions, &local_variables);
                    Some(value)
                } else {
                    None
                };

                call_stack.pop();

                return_value
            }
            Node::FunctionDeclaration(_) => None,
            Node::FunctionCall {
                function_id,
                parameters,
            } => {
                let function = &functions[function_id];
                function.evaluate(parameters.clone(), &local_variables, functions)
            }
        }
    }
}

impl Expression {
    pub fn evaluate(
        &self,
        functions: &Functions,
        local_variables: &HashMap<String, Value>,
    ) -> Value {
        match self {
            Expression::ValueLiteral(value) => value.clone(),
            Expression::FunctionCall(function_id, parameters) => {
                let function = &functions[function_id];
                if matches!(function.return_type(), FunctionReturnType::Void) {
                    panic!("Function expected to be value, but is void");
                };

                function
                    .evaluate(parameters.clone(), local_variables, functions)
                    .expect("function has a non void return type")
            }
            Expression::Operation(operation) => operation.evaluate(functions, local_variables),
            Expression::VariableAccess(variable_name) => local_variables
                .get(variable_name)
                .expect("variable should exist")
                .clone(),
        }
    }
}

fn evaluate_custom_function(
    body: &Ast,
    function_name: &str,
    parameters: HashMap<String, Value>,
    functions: &Functions,
) -> Option<Value> {
    body.evaluate(parameters, functions.clone())
}

impl Function {
    pub fn evaluate(
        &self,
        parameter_expressions: Vec<Expression>,
        local_variables: &HashMap<String, Value>,
        functions: &Functions,
    ) -> Option<Value> {
        if parameter_expressions.len() != self.parameters().len() {
            panic!(
                "Expected {} parameters, but found {}",
                self.parameters().len(),
                parameter_expressions.len()
            );
        }

        let parameter_values: Vec<Value> = parameter_expressions
            .into_iter()
            .map(|expression| expression.evaluate(functions, local_variables))
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
            Function::CustomFunction { name, body, .. } => {
                evaluate_custom_function(body, &name, local_variables, functions)
            }
            Function::Intrinsic { id, .. } => {
                evaluate_intrinsic_function(id, &local_variables, functions)
            }
        }
    }
}

impl Operation {
    pub fn evaluate(
        &self,
        functions: &Functions,
        local_variables: &HashMap<String, Value>,
    ) -> Value {
        match self {
            Operation::Unary(UnaryOperation::Not { value }) => {
                let bool_value = value.evaluate(functions, local_variables);
                let Value::Boolean(BoolValue(val)) =bool_value else {
                    panic!("Expected not argument to be boolean")
                };
                Value::Boolean(BoolValue(!val))
            }
        }
    }
}
