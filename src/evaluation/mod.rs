mod ast;
mod expression;
mod function;
mod if_statement;
pub mod intrinsics;
mod node;
mod operation;

use std::collections::HashMap;

use crate::ast::node::{Function, FunctionId, Value};

type Functions = HashMap<FunctionId, Function>;

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
