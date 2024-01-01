use std::collections::HashMap;

use crate::ast::node::{FunctionCall, FunctionId, Node, Value};

use super::{Functions, NodeResult};

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
                let return_value = return_value
                    .as_ref()
                    .map(|expression| expression.evaluate(functions, local_variables, call_stack));

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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        ast::node::{
            BoolValue, Expression, Function, FunctionCall, FunctionId, FunctionReturnType,
            IfStatement, Node, UIntValue, Value, VariableDeclarationType,
        },
        evaluation::NodeResult,
    };

    #[test]
    fn test_variable_declaration() {
        let node = Node::VariableDeclaration {
            var_type: VariableDeclarationType::Infer,
            var_name: "my_var".to_owned(),
            value: Expression::ValueLiteral(Value::Boolean(BoolValue(true))),
        };

        let mut local_variables = HashMap::new();

        let result = node.evaluate(&mut local_variables, &mut Vec::new(), &HashMap::new());

        assert_eq!(result, NodeResult::None);

        assert_eq!(
            local_variables,
            HashMap::from_iter([("my_var".to_owned(), Value::Boolean(BoolValue(true)))])
        );
    }

    #[test]
    fn test_function_return_with_value() {
        let node = Node::FunctionReturn {
            return_value: Some(Expression::ValueLiteral(Value::Boolean(BoolValue(true)))),
        };

        let result = node.evaluate(&mut HashMap::new(), &mut Vec::new(), &HashMap::new());

        assert_eq!(
            result,
            NodeResult::FunctionReturn {
                value: Some(Value::Boolean(BoolValue(true)))
            }
        );
    }

    #[test]
    fn test_function_return_void() {
        let node = Node::FunctionReturn { return_value: None };

        let result = node.evaluate(&mut HashMap::new(), &mut Vec::new(), &HashMap::new());

        assert_eq!(result, NodeResult::FunctionReturn { value: None });
    }

    #[test]
    fn test_function_call() {
        let node = Node::FunctionCall(FunctionCall {
            function_id: FunctionId("my_function".to_owned()),
            parameters: Vec::new(),
        });

        let functions = HashMap::from_iter([(
            FunctionId("my_function".to_owned()),
            Function::CustomFunction {
                id: FunctionId("my_function".to_owned()),
                name: "my_function".to_owned(),
                parameters: Vec::new(),
                return_type: FunctionReturnType::Void,
                body: Vec::new(),
            },
        )]);

        let result = node.evaluate(&mut HashMap::new(), &mut Vec::new(), &functions);

        assert_eq!(result, NodeResult::None);
    }

    #[test]
    fn test_if_statement_return() {
        let node = Node::IfStatement(IfStatement {
            check_expression: Expression::ValueLiteral(Value::Boolean(BoolValue(true))),
            if_block: vec![Node::FunctionReturn {
                return_value: Some(Expression::ValueLiteral(Value::UInt(UIntValue(10)))),
            }],
            else_if_blocks: Vec::new(),
            else_block: None,
        });

        let result = node.evaluate(&mut HashMap::new(), &mut Vec::new(), &HashMap::new());

        assert_eq!(
            result,
            NodeResult::FunctionReturn {
                value: Some(Value::UInt(UIntValue(10)))
            }
        );
    }

    #[test]
    fn test_if_statement_no_return() {
        let node = Node::IfStatement(IfStatement {
            check_expression: Expression::ValueLiteral(Value::Boolean(BoolValue(true))),
            if_block: Vec::new(),
            else_if_blocks: Vec::new(),
            else_block: None,
        });

        let result = node.evaluate(&mut HashMap::new(), &mut Vec::new(), &HashMap::new());

        assert_eq!(result, NodeResult::None);
    }
}
