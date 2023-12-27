use super::node::{BoolValue, Node, NodeInfo, Operation, UnaryOperation, Value};

#[derive(Debug, PartialEq)]
pub struct Builder {
    nodes: Vec<Node>,
}

impl Builder {
    pub fn new() -> Self {
        Builder { nodes: Vec::new() }
    }

    pub fn literal(mut self, info: NodeInfo, value: Value) -> Self {
        self.nodes.push(Node::Literal { info, value });
        self
    }

    pub fn operation(self, info: NodeInfo) -> impl OperationTraitStarter {
        OperationBuilder {
            builder: self,
            node_info: info,
        }
    }
}

pub trait OperationTraitStarter {
    fn not(self) -> NotOperationBuilder;
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

impl OperationTraitStarter for OperationBuilder {
    fn not(self) -> NotOperationBuilder {
        NotOperationBuilder {
            operation_builder: self,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::node::{BoolValue, NodeInfo, Operation, UIntValue, UnaryOperation};

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
}
