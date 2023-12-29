use super::node::{
    BoolValue, Expression, Node, NodeInfo, Operation, Type, UnaryOperation, Value,
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
    var_name: Option<(String, NodeInfo)>,
    var_type: Option<(VariableDeclarationType, NodeInfo)>,
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
    fn declare_type(mut self, var_type: Type, info: NodeInfo) -> VariableDeclarationBuilder {
        self.var_type = Some((VariableDeclarationType::Type(var_type), info));
        self
    }

    fn name(mut self, name: &str, info: NodeInfo) -> VariableDeclarationBuilder {
        self.var_name = Some((name.to_owned(), info));
        self
    }

    fn with_assignment(mut self, value: Expression, info: NodeInfo) -> Builder {
        let (var_name, var_name_info) = self
            .var_name
            .expect("builder should not be able to get to this point");
        let (var_type, var_type_info) = self
            .var_type
            .expect("builder should not be able to get to this point");

        self.builder.builder.nodes.extend([
            Node::VariableDeclaration {
                name_info: var_name_info,
                var_name: var_name,
                type_info: var_type_info,
                var_type: var_type,
            },
            Node::VariableAssignment { info, value },
        ]);

        self.builder.builder
    }

    fn without_assignment(mut self) -> Builder {
        let (var_name, var_name_info) = self
            .var_name
            .expect("builder should not be able to get to this point");
        let (var_type, var_type_info) = self
            .var_type
            .expect("builder should not be able to get to this point");

        self.builder.builder.nodes.push(Node::VariableDeclaration {
            name_info: var_name_info,
            var_name: var_name,
            type_info: var_type_info,
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

    pub fn literal(mut self, info: NodeInfo, value: Value) -> Self {
        self.nodes.push(Node::Literal { info, value });
        self
    }

    pub fn operation(self, info: NodeInfo) -> OperationBuilder {
        OperationBuilder {
            builder: self,
            node_info: info,
        }
    }
}

pub struct NotOperationBuilder {
    operation_builder: OperationBuilder,
}

impl NotOperationBuilder {
    pub fn value(mut self, value: bool) -> Builder {
        self.operation_builder.builder.nodes.push(Node::Operation {
            info: self.operation_builder.node_info,
            operation: Operation::Unary(UnaryOperation::Not {
                value: BoolValue(value),
            }),
        });

        self.operation_builder.builder
    }
}

pub struct OperationBuilder {
    builder: Builder,
    node_info: NodeInfo,
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
        BoolValue, Expression, NodeInfo, Operation, Type, UIntValue, UnaryOperation,
    };

    use super::*;

    #[test]
    fn add_literal() {
        let result = Builder::new().literal(
            NodeInfo {
                line: 3,
                character: 64,
            },
            Value::UInt(UIntValue(13)),
        );

        let expected = Builder {
            nodes: vec![Node::Literal {
                info: NodeInfo {
                    line: 3,
                    character: 64,
                },
                value: Value::UInt(UIntValue(13)),
            }],
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn add_operation() {
        let result = Builder::new()
            .operation(NodeInfo {
                line: 3,
                character: 15,
            })
            .not()
            .value(true);

        let expected = Builder {
            nodes: vec![Node::Operation {
                info: NodeInfo {
                    line: 3,
                    character: 15,
                },
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
            .declare_type(
                Type::Boolean,
                NodeInfo {
                    line: 3,
                    character: 4,
                },
            )
            .name(
                "my_var_name",
                NodeInfo {
                    line: 3,
                    character: 5,
                },
            )
            .without_assignment();

        let expected = Builder {
            nodes: vec![Node::VariableDeclaration {
                type_info: NodeInfo {
                    line: 3,
                    character: 4,
                },
                var_type: VariableDeclarationType::Type(Type::Boolean),
                name_info: NodeInfo {
                    line: 3,
                    character: 5,
                },
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
            .declare_type(
                Type::Boolean,
                NodeInfo {
                    line: 3,
                    character: 4,
                },
            )
            .name(
                "my_var_name",
                NodeInfo {
                    line: 3,
                    character: 5,
                },
            )
            .with_assignment(
                Expression::ValueLiteral(Value::Boolean(BoolValue(true))),
                NodeInfo {
                    line: 3,
                    character: 6,
                },
            );

        let expected = Builder {
            nodes: vec![
                Node::VariableDeclaration {
                    type_info: NodeInfo {
                        line: 3,
                        character: 4,
                    },
                    var_type: VariableDeclarationType::Type(Type::Boolean),
                    name_info: NodeInfo {
                        line: 3,
                        character: 5,
                    },
                    var_name: "my_var_name".to_owned(),
                },
                Node::VariableAssignment {
                    info: NodeInfo {
                        line: 3,
                        character: 6,
                    },
                    value: Expression::ValueLiteral(Value::Boolean(BoolValue(true))),
                },
            ],
        };

        assert_eq!(result, expected)
    }
}
