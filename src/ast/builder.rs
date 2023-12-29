use super::node::{
    BoolValue, Expression, FunctionParameter, Node, Operation, Type, UnaryOperation, Value,
    VariableDeclarationType,
};

#[derive(Debug, PartialEq)]
pub struct Builder {
    nodes: Vec<Node>,
}

pub struct StatementBuilder {
    builder: Builder,
}

pub struct VariableDeclarationBuilder {
    builder: StatementBuilder,
    var_name: Option<String>,
    var_type: Option<VariableDeclarationType>,
}

impl StatementBuilder {
    pub fn var_declaration(self) -> VariableDeclarationBuilder {
        VariableDeclarationBuilder {
            builder: self,
            var_name: None,
            var_type: None,
        }
    }
}

impl VariableDeclarationBuilder {
    fn declare_type(mut self, var_type: Type) -> VariableDeclarationBuilder {
        self.var_type = Some((VariableDeclarationType::Type(var_type)));
        self
    }

    fn name(mut self, name: &str) -> VariableDeclarationBuilder {
        self.var_name = Some((name.to_owned()));
        self
    }

    fn with_assignment(mut self, value: Expression) -> Builder {
        let var_name = self
            .var_name
            .expect("builder should not be able to get to this point");
        let var_type = self
            .var_type
            .expect("builder should not be able to get to this point");

        self.builder.builder.nodes.extend([
            Node::VariableDeclaration {
                var_name: var_name,
                var_type: var_type,
            },
            Node::VariableAssignment { value },
        ]);

        self.builder.builder
    }

    fn without_assignment(mut self) -> Builder {
        let var_name = self
            .var_name
            .expect("builder should not be able to get to this point");
        let var_type = self
            .var_type
            .expect("builder should not be able to get to this point");

        self.builder.builder.nodes.push(Node::VariableDeclaration {
            var_name: var_name,
            var_type: var_type,
        });
        self.builder.builder
    }
}

impl Builder {
    pub fn new() -> Self {
        Builder { nodes: Vec::new() }
    }

    pub fn statement(self) -> StatementBuilder {
        StatementBuilder { builder: self }
    }

    pub fn literal(mut self, value: Value) -> Self {
        self.nodes.push(Node::Literal { value });
        self
    }

    pub fn operation(self) -> OperationBuilder {
        OperationBuilder { builder: self }
    }

    pub fn function_declaration(self) -> FunctionDeclarationBuilder {
        FunctionDeclarationBuilder {
            builder: self,
            body: None,
            name: None,
            parameters: None,
            return_type: None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionDeclarationBuilder {
    builder: Builder,
    name: Option<String>,
    parameters: Option<Vec<FunctionParameter>>,
    return_type: Option<Type>,
    body: Option<Vec<Node>>,
}

impl FunctionDeclarationBuilder {
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_owned());
        self
    }

    pub fn parameters(mut self, parameters: Vec<FunctionParameter>) -> Self {
        self.parameters = Some(parameters);
        self
    }

    pub fn return_type(mut self, return_type: Type) -> Self {
        self.return_type = Some(return_type);
        self
    }

    pub fn body(mut self, builder: impl FnOnce(Builder) -> Builder) -> Builder {
        self.body = Some(builder(Builder::new()).nodes);
        self.builder.nodes.push(Node::FunctionDeclaration {
            name: self.name.expect("function name should be set"),
            parameters: self.parameters.expect("function parameters should be set"),
            return_type: self
                .return_type
                .expect("function return type should be set"),
            body: self.body.expect("function body should be set"),
        });
        self.builder
    }
}

pub struct NotOperationBuilder {
    operation_builder: OperationBuilder,
}

impl NotOperationBuilder {
    pub fn value(mut self, value: bool) -> Builder {
        self.operation_builder.builder.nodes.push(Node::Operation {
            operation: Operation::Unary(UnaryOperation::Not {
                value: BoolValue(value),
            }),
        });

        self.operation_builder.builder
    }
}

pub struct OperationBuilder {
    builder: Builder,
}

impl OperationBuilder {
    fn not(self) -> NotOperationBuilder {
        NotOperationBuilder {
            operation_builder: self,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::node::{
        BoolValue, Expression, FunctionParameter, Operation, Type, UIntValue, UnaryOperation,
    };

    use super::*;

    #[test]
    fn add_literal() {
        let result = Builder::new().literal(Value::UInt(UIntValue(13)));

        let expected = Builder {
            nodes: vec![Node::Literal {
                value: Value::UInt(UIntValue(13)),
            }],
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn add_operation() {
        let result = Builder::new().operation().not().value(true);

        let expected = Builder {
            nodes: vec![Node::Operation {
                operation: Operation::Unary(UnaryOperation::Not {
                    value: BoolValue(true),
                }),
            }],
        };
    }

    #[test]
    pub fn variable_declaration() {
        let result = Builder::new()
            .statement()
            .var_declaration()
            .declare_type(Type::Boolean)
            .name("my_var_name")
            .without_assignment();

        let expected = Builder {
            nodes: vec![Node::VariableDeclaration {
                var_type: VariableDeclarationType::Type(Type::Boolean),
                var_name: "my_var_name".to_owned(),
            }],
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn variable_declaration_with_assignment() {
        let result = Builder::new()
            .statement()
            .var_declaration()
            .declare_type(Type::Boolean)
            .name("my_var_name")
            .with_assignment(Expression::ValueLiteral(Value::Boolean(BoolValue(true))));

        let expected = Builder {
            nodes: vec![
                Node::VariableDeclaration {
                    var_type: VariableDeclarationType::Type(Type::Boolean),
                    var_name: "my_var_name".to_owned(),
                },
                Node::VariableAssignment {
                    value: Expression::ValueLiteral(Value::Boolean(BoolValue(true))),
                },
            ],
        };

        assert_eq!(result, expected)
    }

    #[test]
    fn test_something() {
        let result = Builder::new()
            .function_declaration()
            .name("my_function")
            .parameters(vec![(Type::Boolean, "param1".to_owned()).into()])
            .return_type(Type::UInt)
            .body(|body: Builder| {
                body.statement()
                    .var_declaration()
                    .declare_type(Type::Boolean)
                    .name("my_var_name")
                    .with_assignment(Expression::ValueLiteral(Value::Boolean(BoolValue(true))))
            });

        let expected = Builder {
            nodes: vec![Node::FunctionDeclaration {
                name: "my_function".to_owned(),
                parameters: vec![FunctionParameter {
                    param_type: Type::Boolean,
                    param_name: "param1".to_owned(),
                }],
                return_type: Type::UInt,
                body: vec![
                    Node::VariableDeclaration {
                        var_type: VariableDeclarationType::Type(Type::Boolean),
                        var_name: "my_var_name".to_owned(),
                    },
                    Node::VariableAssignment {
                        value: Expression::ValueLiteral(Value::Boolean(BoolValue(true))),
                    },
                ],
            }],
        };

        assert_eq!(result, expected);
    }
}
