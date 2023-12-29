use crate::ast::node::{Expression, Node, Operation, UnaryOperation};

use super::ast_builder::AstBuilder;

pub struct OperationBuilder {
    pub(super) builder: AstBuilder,
}

impl OperationBuilder {
    pub fn not(mut self, value: Expression) -> AstBuilder {
        self.builder.nodes.push(Node::Operation {
            operation: Operation::Unary(UnaryOperation::Not { value }),
        });

        self.builder
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::node::{BoolValue, Value};

    use super::*;

    #[test]
    fn add_operation() {
        let result = AstBuilder::new()
            .operation()
            .not(Expression::ValueLiteral(Value::Boolean(BoolValue(true))));

        let expected = AstBuilder {
            nodes: vec![Node::Operation {
                operation: Operation::Unary(UnaryOperation::Not {
                    value: Expression::ValueLiteral(Value::Boolean(BoolValue(true))),
                }),
            }],
        };

        assert_eq!(result, expected);
    }
}
