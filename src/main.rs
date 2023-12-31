mod ast;
pub mod evaluation;

use std::collections::HashMap;

use ast::builders::ast_builder::AstBuilder;
use evaluation::intrinsics::get_intrinsic_functions;

use crate::ast::node::{BoolValue, Expression, Type, UIntValue, Value};

fn main() {
    let ast = AstBuilder::new()
        .function_declaration()
        .name("my_function")
        .parameters(vec![(Type::UInt, "my_uint".to_owned()).into()])
        .return_type(Type::Boolean)
        .body(|builder| {
            builder
                .statement()
                .var_declaration(|var_declaration_builder| {
                    var_declaration_builder
                        .infer_type()
                        .name("my_uint")
                        .with_assignment(|expression_builder| {
                            expression_builder.value_literal(Value::UInt(UIntValue(15)))
                        })
                })
                .statement()
                .var_declaration(|variable_declaration_builder| {
                    variable_declaration_builder
                        .infer_type()
                        .name("my_value")
                        .with_assignment(|expression_builder| {
                            expression_builder.operation(|operation_builder| {
                                operation_builder.not(|expression_builder| {
                                    expression_builder
                                        .value_literal(Value::Boolean(BoolValue(true)))
                                })
                            })
                        })
                })
                .statement()
                .var_declaration(|variable_declaration_builder| {
                    variable_declaration_builder
                        .declare_type(Type::Boolean)
                        .name("my_bool")
                        .with_assignment(|expression_builder| {
                            expression_builder.value_literal(Value::Boolean(BoolValue(true)))
                        })
                })
                .statement()
                .function_call(|function_call_builder| {
                    function_call_builder
                        .function_id("print")
                        .parameter(|param_builder| param_builder.variable("my_bool"))
                })
                .statement()
                .function_call(|function_call_builder| {
                    function_call_builder
                        .function_id("print")
                        .parameter(|param_builder| param_builder.variable("my_value"))
                })
                .statement()
                .function_call(|function_call_builder| {
                    function_call_builder
                        .function_id("print")
                        .parameter(|param_builder| param_builder.variable("my_uint"))
                })
                .statement()
                .return_value(|_| Expression::ValueLiteral(Value::Boolean(BoolValue(true))))
        })
        .function_declaration()
        .name("other_function")
        .no_parameters()
        .return_type(Type::Boolean)
        .body(|builder| {
            builder.statement().return_value(|expression_builder| {
                expression_builder.function_call(|function_call_builder| {
                    function_call_builder.function_id("my_function").parameter(
                        |expression_builder| {
                            expression_builder.value_literal(Value::UInt(UIntValue(10)))
                        },
                    )
                })
            })
        })
        .statement()
        .function_call(|function_call_builder| {
            function_call_builder
                .function_id("other_function")
                .no_parameters()
        })
        .build();

    ast.evaluate(HashMap::new(), get_intrinsic_functions());
}
