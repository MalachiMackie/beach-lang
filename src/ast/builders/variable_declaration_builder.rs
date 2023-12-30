use crate::ast::node::{Expression, Node, Type, VariableDeclarationType};

use super::{
    ast_builder::AstBuilder, expression_builder::ExpressionBuilder,
    statement_builder::StatementBuilder,
};

pub struct VariableDeclarationBuilder {
    pub(super) builder: StatementBuilder,
    pub(super) var_name: Option<String>,
    pub(super) var_type: Option<VariableDeclarationType>,
}

impl VariableDeclarationBuilder {
    pub fn declare_type(mut self, var_type: Type) -> VariableDeclarationBuilder {
        self.var_type = Some(VariableDeclarationType::Type(var_type));
        self
    }

    pub fn infer_type(mut self) -> VariableDeclarationBuilder {
        self.var_type = Some(VariableDeclarationType::Infer);
        self
    }

    pub fn name(mut self, name: &str) -> VariableDeclarationBuilder {
        self.var_name = Some(name.to_owned());
        self
    }

    pub fn with_assignment<TExpressionFn: Fn(ExpressionBuilder) -> Expression>(
        mut self,
        value_fn: TExpressionFn,
    ) -> AstBuilder {
        let var_name = self.var_name.expect(
            "variable declaration name is None, builder should not be able to get to this point",
        );
        let var_type = self.var_type.expect(
            "Variable declaration type is None, builder should not be able to get to this point",
        );

        self.builder
            .builder
            .nodes
            .extend([Node::VariableDeclaration {
                var_name: var_name,
                var_type: var_type,
                value: value_fn(ExpressionBuilder {}),
            }]);

        self.builder.builder
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::node::{BoolValue, Value};

    use super::*;
    #[test]
    pub fn variable_declaration() {
        let result = AstBuilder::new()
            .statement()
            .var_declaration()
            .declare_type(Type::Boolean)
            .name("my_var_name")
            .with_assignment(|expression_builder| {
                expression_builder.value_literal(Value::Boolean(BoolValue(true)))
            });

        let expected = AstBuilder {
            nodes: vec![Node::VariableDeclaration {
                var_type: VariableDeclarationType::Type(Type::Boolean),
                var_name: "my_var_name".to_owned(),
                value: Expression::ValueLiteral(Value::Boolean(BoolValue(true))),
            }],
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn variable_declaration_with_assignment() {
        let result = AstBuilder::new()
            .statement()
            .var_declaration()
            .declare_type(Type::Boolean)
            .name("my_var_name")
            .with_assignment(|expression_builder| {
                expression_builder.value_literal(Value::Boolean(BoolValue(true)))
            });

        let expected = AstBuilder {
            nodes: vec![Node::VariableDeclaration {
                var_type: VariableDeclarationType::Type(Type::Boolean),
                var_name: "my_var_name".to_owned(),
                value: Expression::ValueLiteral(Value::Boolean(BoolValue(true))),
            }],
        };

        assert_eq!(result, expected)
    }
}
