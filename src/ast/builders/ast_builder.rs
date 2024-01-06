use std::collections::HashMap;

use crate::ast::node::{Ast, Function, FunctionDeclaration, FunctionId, Node};

use super::{
    function_declaration_builder::FunctionDeclarationBuilder, statement_builder::StatementBuilder,
};

#[derive(Default, Debug, PartialEq)]
pub struct AstBuilder {
    pub(super) functions: Vec<FunctionDeclaration>,
    pub(super) nodes: Vec<Node>,
}

impl AstBuilder {
    pub fn statement(mut self, statement_fn: impl FnOnce(StatementBuilder) -> Node) -> Self {
        self.nodes.push(statement_fn(StatementBuilder::default()));
        self
    }

    pub fn function_declaration(
        mut self,
        function_declaration_fn: impl FnOnce(FunctionDeclarationBuilder) -> FunctionDeclaration,
    ) -> AstBuilder {
        let function_declaration = function_declaration_fn(FunctionDeclarationBuilder::default());

        self.functions.push(function_declaration);

        self
    }

    pub fn build(self) -> Ast {
        let functions: HashMap<FunctionId, Function> = self
            .functions
            .iter()
            .cloned()
            .map(|function_declaration| {
                (
                    function_declaration.id.clone(),
                    Function::CustomFunction {
                        id: function_declaration.id,
                        name: function_declaration.name,
                        parameters: function_declaration.parameters,
                        return_type: function_declaration.return_type,
                        body: function_declaration.body,
                    },
                )
            })
            .collect();

        Ast {
            functions,
            nodes: self.nodes,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::ast::node::{Ast, Function, FunctionId, FunctionReturnType, Node};

    use super::AstBuilder;

    #[test]
    fn statement() {
        let actual = AstBuilder::default()
            .statement(|statement| statement.return_void())
            .build();

        let expected = Ast {
            functions: HashMap::new(),
            nodes: vec![Node::FunctionReturn { return_value: None }],
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn function_declaration() {
        let actual = AstBuilder::default()
            .function_declaration(|function_declaration| {
                function_declaration
                    .name("my_function")
                    .parameters(Vec::new())
                    .void()
                    .body(|body| body.build())
            })
            .build();

        let expected = Ast {
            functions: HashMap::from_iter([(
                FunctionId("my_function".to_owned()),
                Function::CustomFunction {
                    id: FunctionId("my_function".to_owned()),
                    name: "my_function".to_owned(),
                    parameters: Vec::new(),
                    return_type: FunctionReturnType::Void,
                    body: Vec::new(),
                },
            )]),
            nodes: Vec::new(),
        };

        assert_eq!(actual, expected);
    }
}
