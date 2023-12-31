mod ast;
pub mod evaluation;

use std::collections::HashMap;

use ast::{builders::ast_builder::AstBuilder, node::FunctionParameter};
use evaluation::intrinsics::get_intrinsic_functions;

use crate::ast::node::{BoolValue, Expression, Type, UIntValue, Value};

fn main() {
    let ast = AstBuilder::new()
        .function_declaration(|function_declaration| {
            function_declaration
                .name("my_function")
                .return_type(Type::UInt)
                .parameters(vec![FunctionParameter::FunctionParameter {
                    param_name: "param".to_owned(),
                    param_type: Type::Boolean,
                }])
                .body(|body| {
                    body.if_statement(|if_statement| {
                        if_statement
                            .check_expression(|check| check.variable("param"))
                            .body(|body| {
                                body.return_value(|return_value| {
                                    return_value.value_literal(Value::UInt(UIntValue(69)))
                                })
                                .build()
                            })
                            .else_block(|else_block| {
                                else_block
                                    .return_value(|return_value| {
                                        return_value.value_literal(Value::UInt(UIntValue(420)))
                                    })
                                    .build()
                            })
                            .build()
                    })
                })
        })
        .var_declaration(|var_declaration| {
            var_declaration
                .infer_type()
                .name("val_1")
                .with_assignment(|value| {
                    value.function_call(|function_call| {
                        function_call
                            .function_id("my_function")
                            .parameter(|param| param.value_literal(Value::Boolean(BoolValue(true))))
                    })
                })
        })
        .var_declaration(|var_declaration| {
            var_declaration
                .infer_type()
                .name("val_2")
                .with_assignment(|value| {
                    value.function_call(|function_call| {
                        function_call.function_id("my_function").parameter(|param| {
                            param.value_literal(Value::Boolean(BoolValue(false)))
                        })
                    })
                })
        })
        .function_call(|function_call| {
            function_call
                .function_id("print")
                .parameter(|param| param.variable("val_1"))
        })
        .function_call(|function_call| {
            function_call
                .function_id("print")
                .parameter(|param| param.variable("val_2"))
        })
        .build();

    ast.evaluate(HashMap::new(), get_intrinsic_functions());
}
