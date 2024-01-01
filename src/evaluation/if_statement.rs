use std::collections::HashMap;

use crate::ast::node::{BoolValue, FunctionId, IfStatement, Value};

use super::{evaluate_nodes, Functions, NodeResult};

impl IfStatement {
    pub fn evaluate(
        &self,
        functions: &Functions,
        local_variables: &HashMap<String, Value>,
        call_stack: &mut Vec<FunctionId>,
    ) -> NodeResult {
        let check_value = self
            .check_expression
            .evaluate(functions, local_variables, call_stack);
        let Value::Boolean(BoolValue(bool_value)) = check_value else {
            panic!("Expected if statement check value to be boolean, but found {:?}", check_value)
        };

        if bool_value {
            return evaluate_nodes(&self.if_block, local_variables, call_stack, functions);
        }

        for else_if_block in &self.else_if_blocks {
            let check_value = else_if_block
                .check
                .evaluate(functions, local_variables, call_stack);
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        ast::node::{BoolValue, ElseIfBlock, Expression, IfStatement, Node, UIntValue, Value},
        evaluation::NodeResult,
    };

    #[test]
    fn test_if_statement_true() {
        let if_statement = IfStatement {
            check_expression: Expression::ValueLiteral(Value::Boolean(BoolValue(true))),
            if_block: vec![Node::FunctionReturn {
                return_value: Some(Expression::ValueLiteral(Value::UInt(UIntValue(1)))),
            }],
            else_if_blocks: Vec::new(),
            else_block: None,
        };

        let result = if_statement.evaluate(&HashMap::new(), &HashMap::new(), &mut Vec::new());

        assert_eq!(
            result,
            NodeResult::FunctionReturn {
                value: Some(Value::UInt(UIntValue(1)))
            }
        );
    }

    #[test]
    fn test_if_statement_false_no_else() {
        let if_statement = IfStatement {
            check_expression: Expression::ValueLiteral(Value::Boolean(BoolValue(false))),
            if_block: vec![Node::FunctionReturn {
                return_value: Some(Expression::ValueLiteral(Value::UInt(UIntValue(1)))),
            }],
            else_if_blocks: Vec::new(),
            else_block: None,
        };

        let result = if_statement.evaluate(&HashMap::new(), &HashMap::new(), &mut Vec::new());

        assert_eq!(result, NodeResult::None);
    }

    #[test]
    fn test_if_statement_else() {
        let if_statement = IfStatement {
            check_expression: Expression::ValueLiteral(Value::Boolean(BoolValue(false))),
            if_block: vec![Node::FunctionReturn {
                return_value: Some(Expression::ValueLiteral(Value::UInt(UIntValue(1)))),
            }],
            else_if_blocks: Vec::new(),
            else_block: Some(vec![Node::FunctionReturn {
                return_value: Some(Expression::ValueLiteral(Value::UInt(UIntValue(2)))),
            }]),
        };

        let result = if_statement.evaluate(&HashMap::new(), &HashMap::new(), &mut Vec::new());

        assert_eq!(
            result,
            NodeResult::FunctionReturn {
                value: Some(Value::UInt(UIntValue(2)))
            }
        )
    }

    #[test]
    fn test_else_if_statement() {
        let if_statement = IfStatement {
            check_expression: Expression::ValueLiteral(Value::Boolean(BoolValue(false))),
            if_block: vec![Node::FunctionReturn {
                return_value: Some(Expression::ValueLiteral(Value::UInt(UIntValue(1)))),
            }],
            else_if_blocks: vec![ElseIfBlock {
                check: Expression::ValueLiteral(Value::Boolean(BoolValue(true))),
                block: vec![Node::FunctionReturn {
                    return_value: Some(Expression::ValueLiteral(Value::UInt(UIntValue(3)))),
                }],
            }],
            else_block: Some(vec![Node::FunctionReturn {
                return_value: Some(Expression::ValueLiteral(Value::UInt(UIntValue(2)))),
            }]),
        };

        let result = if_statement.evaluate(&HashMap::new(), &HashMap::new(), &mut Vec::new());

        assert_eq!(
            result,
            NodeResult::FunctionReturn {
                value: Some(Value::UInt(UIntValue(3)))
            }
        )
    }

    #[test]
    #[should_panic]
    fn test_if_statement_incorrect_check() {
        let if_statement = IfStatement {
            check_expression: Expression::ValueLiteral(Value::UInt(UIntValue(10))),
            if_block: vec![Node::FunctionReturn {
                return_value: Some(Expression::ValueLiteral(Value::UInt(UIntValue(1)))),
            }],
            else_if_blocks: Vec::new(),
            else_block: None,
        };

        if_statement.evaluate(&HashMap::new(), &HashMap::new(), &mut Vec::new());
    }

    #[test]
    #[should_panic]
    fn test_else_if_statement_incorrect_check() {
        let if_statement = IfStatement {
            check_expression: Expression::ValueLiteral(Value::Boolean(BoolValue(false))),
            if_block: vec![Node::FunctionReturn {
                return_value: Some(Expression::ValueLiteral(Value::UInt(UIntValue(1)))),
            }],
            else_if_blocks: vec![ElseIfBlock {
                check: Expression::ValueLiteral(Value::UInt(UIntValue(10))),
                block: vec![Node::FunctionReturn {
                    return_value: Some(Expression::ValueLiteral(Value::UInt(UIntValue(3)))),
                }],
            }],
            else_block: None,
        };

        if_statement.evaluate(&HashMap::new(), &HashMap::new(), &mut Vec::new());
    }
}