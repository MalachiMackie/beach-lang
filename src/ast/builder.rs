use super::node::{Node, NodeInfo, Value};

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
}

#[cfg(test)]
mod tests {
    use crate::ast::node::NodeInfo;

    use super::*;

    #[test]
    fn add_literal() {
        let result = Builder::new().literal(
            NodeInfo {
                line: 3,
                character: 64,
            },
            Value::UInt(13),
        );

        let expected = Builder {
            nodes: vec![Node::Literal {
                info: NodeInfo {
                    line: 3,
                    character: 64,
                },
                value: Value::UInt(13),
            }],
        };

        assert_eq!(result, expected);
    }
}
