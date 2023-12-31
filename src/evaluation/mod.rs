pub mod intrinsics;

use core::panic;
use std::collections::HashMap;

use crate::ast::node::{
    Ast, BinaryOperation, BoolValue, Expression, Function, FunctionId, FunctionParameter,
    FunctionReturnType, IfStatement, Node, Operation, UIntValue, UnaryOperation, Value,
};

use self::intrinsics::evaluate_intrinsic_function;

type Functions = HashMap<FunctionId, Function>;

impl Ast {
    pub fn evaluate(
        &self,
        mut local_variables: HashMap<String, Value>,
        mut functions: HashMap<FunctionId, Function>,
    ) -> NodeResult {
        let mut call_stack: Vec<FunctionId> = Vec::new();

        functions.extend(self.functions.clone());

        evaluate_nodes(
            &self.nodes,
            &mut local_variables,
            &mut call_stack,
            &functions,
        )
    }
}

fn evaluate_nodes(
    nodes: &[Node],
    local_variables: &HashMap<String, Value>,
    call_stack: &mut Vec<FunctionId>,
    functions: &Functions,
) -> NodeResult {
    let mut local_variables = local_variables.clone();
    for node in nodes {
        let return_value = node.evaluate(&mut local_variables, call_stack, &functions);
        if return_value.is_return() {
            return return_value;
        }
    }

    NodeResult::None
}

pub enum NodeResult {
    None,
    FunctionReturn { value: Option<Value> },
}

impl NodeResult {
    fn is_return(&self) -> bool {
        matches!(self, NodeResult::FunctionReturn { .. })
    }
}

impl Node {
    pub fn evaluate(
        &self,
        local_variables: &mut HashMap<String, Value>,
        call_stack: &mut Vec<FunctionId>,
        functions: &Functions,
    ) -> NodeResult {
        match self {
            Node::VariableDeclaration {
                var_name, value, ..
            } => {
                local_variables.insert(
                    var_name.to_owned(),
                    value.evaluate(functions, &local_variables),
                );
            }
            Node::FunctionReturn { return_value } => {
                let return_value = if let Some(expression) = return_value {
                    let value = expression.evaluate(functions, &local_variables);
                    Some(value)
                } else {
                    None
                };

                call_stack.pop();

                return NodeResult::FunctionReturn {
                    value: return_value,
                };
            }
            Node::FunctionDeclaration(_) => {}
            Node::FunctionCall {
                function_id,
                parameters,
            } => {
                let function = &functions[function_id];
                function.evaluate(parameters.clone(), &local_variables, functions);
            }
            Node::IfStatement(if_statement) => {
                return if_statement.evaluate(functions, local_variables, call_stack);
            }
        };

        NodeResult::None
    }
}

impl IfStatement {
    pub fn evaluate(
        &self,
        functions: &Functions,
        local_variables: &HashMap<String, Value>,
        call_stack: &mut Vec<FunctionId>,
    ) -> NodeResult {
        let check_value = self.check_expression.evaluate(functions, local_variables);
        let Value::Boolean(BoolValue(bool_value)) = check_value else {
            panic!("Expected if statement check value to be boolean, but found {:?}", check_value)
        };

        if bool_value {
            return evaluate_nodes(&self.if_block, local_variables, call_stack, functions);
        }

        for else_if_block in &self.else_if_blocks {
            let check_value = else_if_block.check.evaluate(functions, local_variables);
            let Value::Boolean(BoolValue(bool_value)) = check_value else {
                panic!("Expected if statement check value to be boolean, but found {:?}", check_value)
            };

            if bool_value {
                return evaluate_nodes(
                    &else_if_block.block,
                    local_variables,
                    call_stack,
                    functions,
                );
            }
        }

        if let Some(else_block) = &self.else_block {
            return evaluate_nodes(&else_block, local_variables, call_stack, functions);
        }

        NodeResult::None
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
            Expression::FunctionCall(function_call) => {
                let function = &functions[&function_call.function_id];
                if matches!(function.return_type(), FunctionReturnType::Void) {
                    panic!("Function expected to be value, but is void");
                };

                function
                    .evaluate(function_call.parameters.clone(), local_variables, functions)
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
    parameters: HashMap<String, Value>,
    functions: &Functions,
) -> Option<Value> {
    if let NodeResult::FunctionReturn { value } = body.evaluate(parameters, functions.clone()) {
        value
    } else {
        None
    }
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
                "Expected {} parameters, but found {} for {}",
                self.parameters().len(),
                parameter_expressions.len(),
                self.name()
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
            Function::CustomFunction { body, .. } => {
                evaluate_custom_function(body, local_variables, functions)
            }
            Function::Intrinsic { id, .. } => evaluate_intrinsic_function(id, &local_variables),
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
            Operation::Unary {
                operation: UnaryOperation::Not,
                value,
            } => {
                let bool_value = value.evaluate(functions, local_variables);
                let Value::Boolean(BoolValue(val)) =bool_value else {
                    panic!("Expected not argument to be boolean")
                };
                Value::Boolean(BoolValue(!val))
            }
            Operation::Binary {
                operation,
                left,
                right,
            } => {
                let left_value = left.evaluate(functions, local_variables);
                let right_value = right.evaluate(functions, local_variables);

                match operation {
                    BinaryOperation::Plus => {
                        let Value::UInt(UIntValue(left_uint)) = left_value else {
                            panic!("Expected a uint, but found {:?}", left_value);
                        };

                        let Value::UInt(UIntValue(right_uint)) = right_value else {
                            panic!("Expected a uint, but found {:?}", right_value);
                        };

                        Value::UInt(UIntValue(left_uint + right_uint))
                    }
                    BinaryOperation::GreaterThan => {
                        let Value::UInt(UIntValue(left_uint)) = left_value else {
                            panic!("Expected a uint, but found {:?}", left_value);
                        };

                        let Value::UInt(UIntValue(right_uint)) = right_value else {
                            panic!("Expected a uint, but found {:?}", right_value);
                        };

                        Value::Boolean(BoolValue(left_uint > right_uint))
                    }
                }
            }
        }
    }
}
