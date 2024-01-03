use std::collections::HashMap;

use crate::ast::node::{Ast, Function, FunctionId, Node, Value};

use super::{Functions, NodeResult};

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

pub(super) fn evaluate_nodes(
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        ast::node::{
            Ast, BoolValue, Expression, Function, FunctionCall, FunctionId, FunctionParameter,
            FunctionReturnType, IfStatement, Node, Type, Value, VariableDeclarationType,
        },
        evaluation::NodeResult,
    };

    use super::evaluate_nodes;

    #[test]
    fn test_evaluate_nodes_no_return() {
        let nodes = vec![
            Node::VariableDeclaration {
                var_type: VariableDeclarationType::Infer,
                var_name: "my_var".to_owned(),
                value: true.into(),
            },
            Node::FunctionCall(FunctionCall {
                function_id: FunctionId("my_function".to_owned()),
                parameters: vec![Expression::VariableAccess("my_var".to_owned())],
            }),
        ];

        let functions = HashMap::from_iter([(
            FunctionId("my_function".to_owned()),
            Function::CustomFunction {
                id: FunctionId("my_function".to_owned()),
                name: "my_function".to_owned(),
                parameters: vec![FunctionParameter::FunctionParameter {
                    param_type: Type::Boolean,
                    param_name: "param".to_owned(),
                }],
                return_type: FunctionReturnType::Void,
                body: Vec::new(),
            },
        )]);

        let result = evaluate_nodes(&nodes, &HashMap::new(), &mut Vec::new(), &functions);

        assert_eq!(result, NodeResult::None)
    }

    #[test]
    fn test_evaluate_nodes_return_value() {
        let nodes = vec![
            Node::VariableDeclaration {
                var_type: VariableDeclarationType::Infer,
                var_name: "my_var".to_owned(),
                value: true.into(),
            },
            Node::IfStatement(IfStatement {
                check_expression: true.into(),
                if_block: vec![Node::FunctionReturn {
                    return_value: Some(Expression::VariableAccess("my_var".to_owned())),
                }],
                else_if_blocks: Vec::new(),
                else_block: None,
            }),
            // extra return to check we return out early if we get a return value
            Node::FunctionReturn {
                return_value: Some(false.into()),
            },
        ];

        let result = evaluate_nodes(&nodes, &HashMap::new(), &mut Vec::new(), &HashMap::new());

        assert_eq!(
            result,
            NodeResult::FunctionReturn {
                value: Some(true.into())
            }
        );
    }

    #[test]
    fn test_ast_evaluate() {
        let ast_functions = HashMap::from_iter([(
            FunctionId("function_1".to_owned()),
            Function::CustomFunction {
                id: FunctionId("function_1".to_owned()),
                name: "function_1".to_owned(),
                parameters: Vec::new(),
                return_type: FunctionReturnType::Void,
                body: Vec::new(),
            },
        )]);

        let outer_functions = HashMap::from_iter([(
            FunctionId("function_2".to_owned()),
            Function::CustomFunction {
                id: FunctionId("function_2".to_owned()),
                name: "function_2".to_owned(),
                parameters: Vec::new(),
                return_type: FunctionReturnType::Void,
                body: Vec::new(),
            },
        )]);

        let nodes = vec![
            Node::FunctionCall(FunctionCall {
                function_id: FunctionId("function_1".to_owned()),
                parameters: Vec::new(),
            }),
            Node::FunctionCall(FunctionCall {
                function_id: FunctionId("function_2".to_owned()),
                parameters: Vec::new(),
            }),
        ];

        let ast = Ast {
            functions: ast_functions,
            nodes,
        };

        let result = ast.evaluate(HashMap::new(), outer_functions);

        assert_eq!(result, NodeResult::None)
    }
}
