mod ast;

use ast::builders::ast_builder::AstBuilder;

use crate::ast::node::{BoolValue, Expression, FunctionId, Type, UIntValue, Value};

fn main() {
    let builder = AstBuilder::new()
        .function_declaration()
        .name("my_function")
        .parameters(vec![(Type::UInt, "my_uint".to_owned()).into()])
        .return_type(Type::Boolean)
        .body(|builder| {
            builder
                .statement()
                .var_declaration()
                .infer_type()
                .name("my_uint")
                .with_assignment(Expression::ValueLiteral(Value::UInt(UIntValue(15))))
                .literal(Value::Boolean(BoolValue(true)))
                .operation()
                .not(Expression::ValueLiteral(Value::Boolean(BoolValue(true))))
                .statement()
                .var_declaration()
                .declare_type(Type::Boolean)
                .name("my_bool")
                .with_assignment(Expression::ValueLiteral(Value::Boolean(BoolValue(true))))
                .statement()
                .return_value(Expression::ValueLiteral(Value::Boolean(BoolValue(true))))
        })
        .function_declaration()
        .name("other_function")
        .no_parameters()
        .return_type(Type::Boolean)
        .body(|builder| {
            builder
                .statement()
                .return_value(Expression::FunctionCall(FunctionId(
                    "my_function".to_owned(),
                )))
        });

    println!("{:#?}", builder);
}
