mod ast;
mod evaluation;
mod token_stream;
mod type_checking;

use ast::{
    builders::ast_builder::AstBuilder,
    node::{Ast, FunctionParameter},
};

use crate::ast::node::Type;

fn main() {
    let ast = fibonacci(AstBuilder::default());

    if let Err(errors) = ast.type_check() {
        for error in errors {
            println!("{}", error.message);
        }

        return;
    }

    ast.evaluate();
}

/// function fibonnacci(uint lower, uint higher, uint limit) -> uint
/// {
///     infer next = lower + higher;
///     if (next > limit)
///     {
///         return;
///     }
///
///     print(next);
///
///     fibonnacci(higher, next, limit);
/// }
///
/// print(0);
/// print(1);
/// fibonnacci(0, 1, 10000);
fn fibonacci(ast_builder: AstBuilder) -> Ast {
    ast_builder
        .function_declaration(|function_declaration| {
            function_declaration
                .name("fibonacci")
                .parameters(vec![
                    FunctionParameter::FunctionParameter {
                        param_type: Type::UInt,
                        param_name: "lower".to_owned(),
                    },
                    FunctionParameter::FunctionParameter {
                        param_type: Type::UInt,
                        param_name: "higher".to_owned(),
                    },
                    FunctionParameter::FunctionParameter {
                        param_type: Type::UInt,
                        param_name: "limit".to_owned(),
                    },
                ])
                .void()
                .body(|body| {
                    body.statement(|statement| {
                        statement.var_declaration(|var_declaration| {
                            var_declaration
                                .infer_type()
                                .name("next")
                                .with_assignment(|value| {
                                    value.operation(|operation| {
                                        operation.plus(
                                            |left| left.variable("lower"),
                                            |right| right.variable("higher"),
                                        )
                                    })
                                })
                        })
                    })
                    .statement(|statement| {
                        statement.if_statement(|if_statement| {
                            if_statement
                                .check_expression(|check| {
                                    check.operation(|operation| {
                                        operation.greater_than(
                                            |left| left.variable("next"),
                                            |right| right.variable("limit"),
                                        )
                                    })
                                })
                                .body(|if_body| {
                                    if_body
                                        .statement(|statement| statement.return_void())
                                        .build()
                                })
                                .build()
                        })
                    })
                    .statement(|statement| {
                        statement.function_call(|function_call| {
                            function_call
                                .function_id("print")
                                .parameter(|param| param.variable("next"))
                                .build()
                        })
                    })
                    .statement(|statement| {
                        statement.function_call(|function_call| {
                            function_call
                                .function_id("fibonacci")
                                .parameter(|param| param.variable("higher"))
                                .parameter(|param| param.variable("next"))
                                .parameter(|param| param.variable("limit"))
                                .build()
                        })
                    })
                })
        })
        .statement(|statement| {
            statement.function_call(|function_call| {
                function_call
                    .function_id("print")
                    .parameter(|param| param.value_literal(0.into()))
                    .build()
            })
        })
        .statement(|statement| {
            statement.function_call(|function_call| {
                function_call
                    .function_id("print")
                    .parameter(|param| param.value_literal(1.into()))
                    .build()
            })
        })
        .statement(|statement| {
            statement.function_call(|function_call| {
                function_call
                    .function_id("fibonacci")
                    .parameter(|param| param.value_literal(0.into()))
                    .parameter(|param| param.value_literal(1.into()))
                    .parameter(|param| param.value_literal(10000.into()))
                    .build()
            })
        })
        .build()
}
