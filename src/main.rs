mod ast;
pub mod evaluation;

use std::collections::HashMap;

use ast::{
    builders::ast_builder::AstBuilder,
    node::{Ast, FunctionParameter},
};
use evaluation::intrinsics::get_intrinsic_functions;

use crate::ast::node::{Type, UIntValue, Value};

fn main() {
    let ast = fibonacci(AstBuilder::new());

    ast.evaluate(HashMap::new(), get_intrinsic_functions());
}

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
                    body.var_declaration(|var_declaration| {
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
                    .if_statement(|if_statement| {
                        if_statement
                            .check_expression(|check| {
                                check.operation(|operation| {
                                    operation.greater_than(
                                        |left| left.variable("next"),
                                        |right| right.variable("limit"),
                                    )
                                })
                            })
                            .body(|if_body| if_body.return_void().build())
                            .build()
                    })
                    .function_call(|function_call| {
                        function_call
                            .function_id("print")
                            .parameter(|param| param.variable("next"))
                    })
                    .function_call(|function_call| {
                        function_call
                            .function_id("fibonacci")
                            .parameter(|param| param.variable("higher"))
                            .parameter(|param| param.variable("next"))
                            .parameter(|param| param.variable("limit"))
                    })
                })
        })
        .function_call(|function_call| {
            function_call
                .function_id("print")
                .parameter(|param| param.value_literal(Value::UInt(UIntValue(0))))
        })
        .function_call(|function_call| {
            function_call
                .function_id("print")
                .parameter(|param| param.value_literal(Value::UInt(UIntValue(1))))
        })
        .function_call(|function_call| {
            function_call
                .function_id("fibonacci")
                .parameter(|param| param.value_literal(Value::UInt(UIntValue(0))))
                .parameter(|param| param.value_literal(Value::UInt(UIntValue(1))))
                .parameter(|param| param.value_literal(Value::UInt(UIntValue(10000))))
        })
        .build()
}
