mod expression;
mod function;
mod if_statement;
pub mod intrinsics;
mod operation;

use core::panic;
use std::collections::HashMap;

use crate::ast::node::{
    Ast, Expression, Function, FunctionCall, FunctionId, FunctionReturnType, Node, Value,
};

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

#[derive(Debug, PartialEq)]
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
                    value.evaluate(functions, &local_variables, call_stack),
                );
            }
            Node::FunctionReturn { return_value } => {
                let return_value = if let Some(expression) = return_value {
                    let value = expression.evaluate(functions, &local_variables, call_stack);
                    Some(value)
                } else {
                    None
                };

                call_stack.pop();

                return NodeResult::FunctionReturn {
                    value: return_value,
                };
            }
            Node::FunctionCall(FunctionCall {
                function_id,
                parameters,
            }) => {
                let function = &functions[function_id];
                function.evaluate(parameters.clone(), &local_variables, functions, call_stack);
            }
            Node::IfStatement(if_statement) => {
                return if_statement.evaluate(functions, local_variables, call_stack);
            }
        };

        NodeResult::None
    }
}
