use crate::ast::node::{Expression, Node, Type, VariableDeclarationType};

use super::expression_builder::ExpressionBuilder;

#[derive(Default)]
pub struct VariableDeclarationBuilder {
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

    pub fn with_assignment<TExpressionFn: FnOnce(ExpressionBuilder) -> Expression>(
        self,
        value_fn: TExpressionFn,
    ) -> Node {
        let var_name = self.var_name.expect(
            "variable declaration name is None, builder should not be able to get to this point",
        );
        let var_type = self.var_type.expect(
            "Variable declaration type is None, builder should not be able to get to this point",
        );

        Node::VariableDeclaration {
            var_type,
            var_name,
            value: value_fn(ExpressionBuilder {}),
        }
    }
}

#[cfg(test)]
mod tests {
    

    use super::*;
    #[test]
    pub fn variable_declaration() {
        let result = VariableDeclarationBuilder::default()
            .declare_type(Type::Boolean)
            .name("my_var_name")
            .with_assignment(|expression_builder| expression_builder.value_literal(true.into()));

        let expected = Node::VariableDeclaration {
            var_type: VariableDeclarationType::Type(Type::Boolean),
            var_name: "my_var_name".to_owned(),
            value: true.into(),
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn variable_declaration_infer_type() {
        let result = VariableDeclarationBuilder::default()
            .infer_type()
            .name("my_var_name")
            .with_assignment(|expression_builder| expression_builder.value_literal(true.into()));

        let expected = Node::VariableDeclaration {
            var_type: VariableDeclarationType::Infer,
            var_name: "my_var_name".to_owned(),
            value: true.into(),
        };

        assert_eq!(result, expected);
    }
}
