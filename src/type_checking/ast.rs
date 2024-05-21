use std::collections::HashMap;

use crate::{ast::node::Ast, evaluation::intrinsics::get_intrinsic_functions};

use super::{nodes::node::type_check_nodes, TypeCheckingError};

impl Ast {
    pub fn type_check(&self) -> Result<(), Vec<TypeCheckingError>> {
        let intrinsic_functions = get_intrinsic_functions();

        let functions: HashMap<_, _> = self
            .functions
            .iter()
            .map(|(id, function)| (id.clone(), function.clone()))
            .chain(intrinsic_functions)
            .collect();

        let function_errors = functions
            .values()
            .filter_map(|function| function.type_check(&functions).err())
            .flat_map(|x| x);

        let body_errors: Vec<_> = type_check_nodes(&self.nodes, &functions, &HashMap::new(), None)
            .err()
            .unwrap_or_default()
            .into_iter()
            .chain(function_errors)
            .collect();

        if body_errors.is_empty() {
            Ok(())
        } else {
            Err(body_errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            builders::ast_builder::AstBuilder,
            node::{FunctionParameter, Type},
        },
        token_stream::token::TokenSource,
    };

    #[test]
    fn type_check_ast_successful() {
        let ast = AstBuilder::default()
            .function_declaration(|fn_decl| {
                fn_decl
                    .parameters(vec![FunctionParameter::FunctionParameter {
                        param_type: Type::Boolean,
                        param_name: "my_param".to_owned(),
                    }])
                    .return_type(Type::Boolean)
                    .name("my_function")
                    .body(|body| {
                        body.statement(|statement| {
                            statement.return_value(|expression| expression.variable("my_param"))
                        })
                        .build()
                    })
            })
            .statement(|statement| {
                statement.function_call(|fn_call| {
                    fn_call
                        .function_id("print")
                        .parameter(|_| (10, TokenSource::dummy_uint(10)).into())
                        .build()
                })
            })
            .statement(|statement| {
                statement.function_call(|fn_call| {
                    fn_call
                        .function_id("my_function")
                        .parameter(|_| (true, TokenSource::dummy_true()).into())
                        .build()
                })
            })
            .build();

        let result = ast.type_check();

        assert!(matches!(result, Ok(())));
    }

    #[test]
    fn type_check_ast_errors() {
        let ast = AstBuilder::default()
            .function_declaration(|fn_decl| {
                fn_decl
                    .parameters(vec![FunctionParameter::FunctionParameter {
                        param_type: Type::Boolean,
                        param_name: "my_param".to_owned(),
                    }])
                    .return_type(Type::Boolean)
                    .name("my_function")
                    .body(|body| body.statement(|statement| statement.return_void()).build())
            })
            .statement(|statement| {
                statement.function_call(|fn_call| {
                    fn_call
                        .function_id("my_function")
                        .parameter(|_| (10, TokenSource::dummy_uint(10)).into())
                        .build()
                })
            })
            .build();

        let result = ast.type_check();

        assert!(matches!(result, Err(e) if e.len() == 2));
    }
}
