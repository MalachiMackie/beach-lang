mod expression;
mod function;
mod if_statement;
pub mod intrinsics;
mod node;
mod operation;

use std::collections::HashMap;

use crate::ast::node::{Ast, Function, FunctionId, Node, Value};

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
