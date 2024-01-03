use std::collections::HashMap;

use crate::{
    ast::node::{Expression, Function, FunctionId, Node, Type, VariableDeclarationType},
    type_checking::{verify_type, TypeCheckingError},
};

use super::function_return::type_check_return_value;

pub fn type_check_nodes(
    nodes: &[Node],
    functions: &HashMap<FunctionId, Function>,
    local_variables: &HashMap<String, Type>,
) -> Result<(), Vec<TypeCheckingError>> {
    // todo
    Ok(())
}

#[derive(Debug)]
pub enum NodeTypeCheckResult {
    ReturnedFromFunction(Result<(), Vec<TypeCheckingError>>),
    // gross name
    DidNotReturnedFromFunction(Result<(), Vec<TypeCheckingError>>),
}

impl Node {
    pub fn type_check(
        &self,
        functions: &HashMap<FunctionId, Function>,
        local_variables: &mut HashMap<String, Type>,
        current_function: Option<&FunctionId>,
    ) -> NodeTypeCheckResult {
        match self {
            Node::VariableDeclaration {
                var_type,
                var_name,
                value,
            } => NodeTypeCheckResult::DidNotReturnedFromFunction(
                Self::type_check_variable_declaration(
                    var_name,
                    var_type,
                    value,
                    functions,
                    local_variables,
                ),
            ),
            Node::FunctionReturn { return_value } => {
                NodeTypeCheckResult::ReturnedFromFunction(type_check_return_value(
                    return_value.as_ref(),
                    functions,
                    local_variables,
                    current_function,
                ))
            }
            Node::FunctionCall(_) => todo!(),
            Node::IfStatement(_) => todo!(),
        }
    }

    fn type_check_variable_declaration(
        var_name: &str,
        var_type: &VariableDeclarationType,
        value: &Expression,
        functions: &HashMap<FunctionId, Function>,
        local_variables: &mut HashMap<String, Type>,
    ) -> Result<(), Vec<TypeCheckingError>> {
        let mut errors = Vec::new();

        let value_type = value.get_type(functions, local_variables);

        if let Err(value_expression_errors) = value.type_check(functions, local_variables) {
            errors.extend(value_expression_errors);
        }

        let variable_already_exists = if local_variables.get(var_name).is_some() {
            errors.push(TypeCheckingError {
                message: format!("Variable {var_name} is already defined"),
            });
            true
        } else {
            false
        };

        match var_type {
            VariableDeclarationType::Infer => {
                if let Some(value_type) = value_type {
                    // only insert variable if it isn't already declared as .insert overwrites the existing value. see https://github.com/rust-lang/rust/issues/82766
                    if !variable_already_exists {
                        local_variables.insert(var_name.to_owned(), value_type);
                    }
                } else {
                    errors.push(TypeCheckingError {
                        message: format!("cannot assign void to variable {}", var_name),
                    });
                }
            }
            VariableDeclarationType::Type(expected_type) => {
                if !variable_already_exists {
                    local_variables.insert(var_name.to_owned(), *expected_type);
                }

                if let Err(var_error) = verify_type(value_type, *expected_type) {
                    errors.push(var_error);
                }
            }
        };

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        ast::node::{
            BoolValue, Expression, Function, FunctionCall, FunctionId, FunctionReturnType, Node,
            Operation, Type, UIntValue, UnaryOperation, Value, VariableDeclarationType,
        },
        type_checking::nodes::node::NodeTypeCheckResult,
    };

    #[test]
    fn type_check_variable_declaration_success() {
        let node = Node::VariableDeclaration {
            var_type: VariableDeclarationType::Type(Type::Boolean),
            var_name: "my_var".to_owned(),
            value: true.into(),
        };

        let mut local_variables = HashMap::new();

        let result = node.type_check(&HashMap::new(), &mut local_variables, None);

        assert!(matches!(
            result,
            NodeTypeCheckResult::DidNotReturnedFromFunction(Ok(_))
        ));

        assert!(matches!(local_variables.get("my_var"), Some(Type::Boolean)));
    }

    #[test]
    fn type_check_variable_declaration_infer_success() {
        let node = Node::VariableDeclaration {
            var_type: VariableDeclarationType::Infer,
            var_name: "my_var".to_owned(),
            value: true.into(),
        };

        let mut local_variables = HashMap::new();

        let result = node.type_check(&HashMap::new(), &mut local_variables, None);

        assert!(matches!(
            result,
            NodeTypeCheckResult::DidNotReturnedFromFunction(Ok(_))
        ));

        assert!(matches!(local_variables.get("my_var"), Some(Type::Boolean)));
    }

    #[test]
    fn type_check_variable_declaration_failure_existing_variable() {
        let node = Node::VariableDeclaration {
            var_type: VariableDeclarationType::Type(Type::Boolean),
            var_name: "my_name".to_owned(),
            value: true.into(),
        };

        let mut local_variables = HashMap::from_iter([("my_name".to_owned(), Type::UInt)]);

        let result = node.type_check(&HashMap::new(), &mut local_variables, None);

        assert!(
            matches!(result, NodeTypeCheckResult::DidNotReturnedFromFunction(Err(e)) if e.len() == 1 && e[0].message == "Variable my_name is already defined".to_owned())
        );

        // verify variable type wasn't overwritten
        assert!(matches!(local_variables.get("my_name"), Some(Type::UInt)));
    }

    #[test]
    fn type_check_variable_declaration_failure_value_expression() {
        let node = Node::VariableDeclaration {
            var_type: VariableDeclarationType::Infer,
            var_name: "my_name".to_owned(),
            value: Expression::Operation(Operation::Unary {
                operation: UnaryOperation::Not,
                value: Box::new(10.into()),
            }),
        };

        let result = node.type_check(&HashMap::new(), &mut HashMap::new(), None);

        assert!(matches!(
            result,
            NodeTypeCheckResult::DidNotReturnedFromFunction(Err(_))
        ))
    }

    #[test]
    fn type_check_variable_declaration_failure_infer_void_function() {
        let node = Node::VariableDeclaration {
            var_type: VariableDeclarationType::Infer,
            var_name: "my_value".to_owned(),
            value: Expression::FunctionCall(FunctionCall {
                function_id: FunctionId("my_function".to_owned()),
                parameters: Vec::new(),
            }),
        };

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

        let result = node.type_check(&functions, &mut HashMap::new(), None);

        assert!(
            matches!(result, NodeTypeCheckResult::DidNotReturnedFromFunction(Err(e)) if e.len() == 1 && e[0].message == "cannot assign void to variable my_value".to_owned())
        )
    }

    #[test]
    fn type_check_variable_declaration_failure_incorrect_type() {
        let node = Node::VariableDeclaration {
            var_type: VariableDeclarationType::Type(Type::UInt),
            var_name: "my_value".to_owned(),
            value: true.into(),
        };

        let result = node.type_check(&HashMap::new(), &mut HashMap::new(), None);

        assert!(
            matches!(result, NodeTypeCheckResult::DidNotReturnedFromFunction(Err(e)) if e.len() == 1 && e[0].message == "Expected type to be UInt, but found Boolean".to_owned())
        )
    }
}
