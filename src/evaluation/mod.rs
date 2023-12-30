use core::panic;
use std::collections::HashMap;

use crate::ast::node::{
    Ast, Expression, FunctionDeclaration, FunctionId, FunctionReturnType, Node, Operation,
    UnaryOperation, Value, BoolValue,
};

type Functions = HashMap<FunctionId, FunctionDeclaration>;

impl Ast {
    pub fn evaluate(
        &self,
        mut local_variables: HashMap<String, Value>,
        mut functions: HashMap<FunctionId, FunctionDeclaration>,
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
                local_variables.insert(var_name.to_owned(), value.evaluate(functions));
                None
            }
            Node::FunctionReturn { return_value } => {
                let return_value = if let Some(expression) = return_value {
                    let value = expression.evaluate(functions);
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
                function.evaluate(parameters.clone(), functions)
            }
        }
    }
}

impl Expression {
    pub fn evaluate(&self, functions: &Functions) -> Value {
        match self {
            Expression::ValueLiteral(value) => {
                println!("expression->ValueLiteral {:?}", value);
                value.clone()
            }
            Expression::FunctionCall(function_id, parameters) => {
                println!(
                    "expression->FunctionCall function_id: {:?}, parameters: {:?}",
                    function_id, parameters
                );
                let function = &functions[function_id];
                if matches!(function.return_type, FunctionReturnType::Void) {
                    panic!("Function expected to be value, but is void");
                };

                function
                    .evaluate(parameters.clone(), functions)
                    .expect("function has a non void return type")
            }
            Expression::Operation(operation) => {
                println!("expression->Operation {:?}", operation);
                operation.evaluate(functions)
            }
        }
    }
}

impl FunctionDeclaration {
    pub fn evaluate(
        &self,
        parameter_expressions: Vec<Expression>,
        functions: &Functions,
    ) -> Option<Value> {
        if parameter_expressions.len() != self.parameters.len() {
            panic!(
                "Expected {} parameters, but found {}",
                self.parameters.len(),
                parameter_expressions.len()
            );
        }

        let parameters: Vec<Value> = parameter_expressions
            .into_iter()
            .map(|expression| expression.evaluate(functions))
            .collect();

        let local_variables = self
            .parameters
            .iter()
            .enumerate()
            .map(|(i, function_parameter)| {
                (function_parameter.param_name.clone(), parameters[i].clone())
            })
            .collect();

        println!(
            "evaluate functionDeclaration {:?} {:?}",
            self.name, local_variables
        );

        self.body.evaluate(local_variables, functions.clone())
    }
}

impl Operation {
    pub fn evaluate(&self, functions: &Functions) -> Value {
        match self {
            Operation::Unary(UnaryOperation::Not { value }) => {
                let bool_value = value.evaluate(functions);
                let Value::Boolean(BoolValue(val)) =bool_value else {
                    panic!("Expected not argument to be boolean")
                };
                Value::Boolean(BoolValue(!val))
            },
        }
    }
}
