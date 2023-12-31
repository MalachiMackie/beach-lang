mod ast;
pub mod evaluation;

use std::collections::HashMap;

use ast::builders::ast_builder::AstBuilder;
use evaluation::intrinsics::get_intrinsic_functions;

use crate::ast::node::{BoolValue, Expression, Type, UIntValue, Value};

fn main() {
    let ast = AstBuilder::new()
        .function_declaration(|function_declaration_builder| {
            function_declaration_builder
                .name("my_function")
                .parameters(vec![(Type::UInt, "my_uint".to_owned()).into()])
                .return_type(Type::Boolean)
                .body(|builder| {
                    builder
                        .var_declaration(|var_declaration_builder| {
                            var_declaration_builder
                                .infer_type()
                                .name("my_uint")
                                .with_assignment(|expression_builder| {
                                    expression_builder.value_literal(Value::UInt(UIntValue(15)))
                                })
                        })
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
                        .var_declaration(|variable_declaration_builder| {
                            variable_declaration_builder
                                .declare_type(Type::Boolean)
                                .name("my_bool")
                                .with_assignment(|expression_builder| {
                                    expression_builder
                                        .value_literal(Value::Boolean(BoolValue(true)))
                                })
                        })
                        .function_call(|function_call_builder| {
                            function_call_builder
                                .function_id("print")
                                .parameter(|param_builder| param_builder.variable("my_bool"))
                        })
                        .function_call(|function_call_builder| {
                            function_call_builder
                                .function_id("print")
                                .parameter(|param_builder| param_builder.variable("my_value"))
                        })
                        .function_call(|function_call_builder| {
                            function_call_builder
                                .function_id("print")
                                .parameter(|param_builder| param_builder.variable("my_uint"))
                        })
                        .return_value(|_| Expression::ValueLiteral(Value::Boolean(BoolValue(true))))
                })
        })
        .function_declaration(|function_declaration_builder| {
            function_declaration_builder
                .name("other_function")
                .no_parameters()
                .return_type(Type::Boolean)
                .body(|builder| {
                    builder.return_value(|expression_builder| {
                        expression_builder.function_call(|function_call_builder| {
                            function_call_builder.function_id("my_function").parameter(
                                |expression_builder| {
                                    expression_builder.value_literal(Value::UInt(UIntValue(10)))
                                },
                            )
                        })
                    })
                })
        })
        .function_call(|function_call_builder| {
            function_call_builder
                .function_id("other_function")
                .no_parameters()
        })
        .build();

    ast.evaluate(HashMap::new(), get_intrinsic_functions());
}
