use std::{collections::HashMap, fmt::Display};

use crate::token_stream::token::TokenSource;

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    VariableDeclaration {
        var_type: VariableDeclarationType,
        var_name: String,
        value: Expression,
    },
    FunctionReturn {
        return_value: Option<Expression>,
    },
    FunctionCall(FunctionCall),
    IfStatement(IfStatement),
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfStatement {
    pub check_expression: Expression,
    pub if_block: Vec<Node>,
    pub else_if_blocks: Vec<ElseIfBlock>,
    pub else_block: Option<ElseBlock>,
    pub if_token: TokenSource,
    pub left_parenthesis_token: TokenSource,
    pub right_parenthesis_token: TokenSource,
    pub left_curley_brace_token: TokenSource,
    pub right_curley_brace_token: TokenSource,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ElseBlock {
    pub nodes: Vec<Node>,
    pub left_curley_brace_token: TokenSource,
    pub right_curley_brace_token: TokenSource
}

#[derive(Clone, Debug, PartialEq)]
pub struct ElseIfBlock {
    pub check: Expression,
    pub block: Vec<Node>,
    pub else_token: TokenSource,
    pub if_token: TokenSource,
    pub left_parenthesis_token: TokenSource,
    pub right_parenthesis_token: TokenSource,
    pub left_curley_brace_token: TokenSource,
    pub right_curley_brace_token: TokenSource,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDeclaration {
    pub id: FunctionId,
    pub name: String,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: FunctionReturnType,
    pub body: Vec<Node>,
    pub function_keyword_token: TokenSource,
    pub function_identifier_token: TokenSource,
    pub left_parenthesis_token: TokenSource,
    pub comma_tokens: Vec<TokenSource>,
    pub right_parenthesis_token: TokenSource,
    pub left_curley_brace_token: TokenSource,
    pub right_curley_brace_token: TokenSource,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FunctionReturnType {
    Type {
        return_type: Type,
        function_signiture_separator_token: TokenSource,
        type_token: TokenSource,
    },
    Void,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FunctionParameter {
    FunctionParameter {
        param_type: Type,
        type_token: TokenSource,
        param_name: String,
        param_name_token: TokenSource,
    },
    IntrinsicAny {
        param_name: String,
    },
}

impl FunctionParameter {
    pub fn name(&self) -> &str {
        match self {
            FunctionParameter::IntrinsicAny { param_name }
            | FunctionParameter::FunctionParameter { param_name, .. } => &param_name,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    ValueLiteral(Value, TokenSource),
    FunctionCall(FunctionCall),
    Operation(Operation),
    VariableAccess(String, TokenSource),
}

impl From<(bool, TokenSource)> for Expression {
    fn from((value, source): (bool, TokenSource)) -> Self {
        Expression::ValueLiteral(value.into(), source)
    }
}

impl From<(u32, TokenSource)> for Expression {
    fn from((value, source): (u32, TokenSource)) -> Self {
        Expression::ValueLiteral(value.into(), source)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionCall {
    pub function_id: FunctionId,
    pub parameters: Vec<Expression>,
    pub left_parenthesis_token: TokenSource,
    pub right_parenthesis_token: TokenSource,
    pub comma_tokens: Vec<TokenSource>,
    pub function_id_token: TokenSource,
}

#[derive(Debug, PartialEq, Hash, Clone, Eq)]
pub struct FunctionId(pub String);

impl Display for FunctionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    UInt(UIntValue),
    Boolean(BoolValue),
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value.into())
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Value::UInt(value.into())
    }
}

impl Value {
    pub fn expect_uint(self, expect_message: &str) -> UIntValue {
        if let Self::UInt(uint_value) = self {
            Some(uint_value)
        } else {
            None
        }
        .expect(expect_message)
    }

    pub fn expect_bool(self, expect_message: &str) -> BoolValue {
        if let Self::Boolean(bool_value) = self {
            Some(bool_value)
        } else {
            None
        }
        .expect(expect_message)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum VariableDeclarationType {
    Infer,
    Type(Type),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    UInt,
    Boolean,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::UInt => f.write_str("UInt"),
            Type::Boolean => f.write_str("Boolean"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UIntValue(pub u32);

impl From<u32> for UIntValue {
    fn from(value: u32) -> Self {
        UIntValue(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BoolValue(pub bool);

impl From<bool> for BoolValue {
    fn from(value: bool) -> Self {
        BoolValue(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operation {
    Unary {
        operation: UnaryOperation,
        value: Box<Expression>,
        operator_token: TokenSource,
    },
    Binary {
        operation: BinaryOperation,
        left: Box<Expression>,
        right: Box<Expression>,
        operator_token: TokenSource,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UnaryOperation {
    Not,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BinaryOperation {
    Plus,
    GreaterThan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ast {
    pub functions: HashMap<FunctionId, Function>,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Function {
    CustomFunction {
        id: FunctionId,
        name: String,
        parameters: Vec<FunctionParameter>,
        return_type: FunctionReturnType,
        body: Vec<Node>,
    },
    Intrinsic {
        id: FunctionId,
        name: String,
        parameters: Vec<FunctionParameter>,
        return_type: FunctionReturnType,
    },
}

impl Function {
    pub fn id(&self) -> &FunctionId {
        match self {
            Function::CustomFunction { id, .. } | Function::Intrinsic { id, .. } => id,
        }
    }

    pub fn name(&self) -> &String {
        match self {
            Function::CustomFunction { name, .. } | Function::Intrinsic { name, .. } => name,
        }
    }

    pub fn parameters(&self) -> &[FunctionParameter] {
        match self {
            Function::CustomFunction { parameters, .. }
            | Function::Intrinsic { parameters, .. } => parameters,
        }
    }

    pub fn return_type(&self) -> &FunctionReturnType {
        match self {
            Function::CustomFunction { return_type, .. }
            | Function::Intrinsic { return_type, .. } => return_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Function, FunctionId, FunctionParameter, FunctionReturnType, Node, Type};

    #[test]
    fn custom_function_getters() {
        let function = Function::CustomFunction {
            id: FunctionId("my_function".to_owned()),
            name: "my_function".to_owned(),
            parameters: vec![FunctionParameter::FunctionParameter {
                param_type: Type::Boolean,
                param_name: "my_param".to_owned(),
            }],
            return_type: FunctionReturnType::Void,
            body: vec![Node::FunctionReturn { return_value: None }],
        };

        assert_eq!(function.id(), &FunctionId("my_function".to_owned()));
        assert_eq!(function.name(), "my_function");
        assert_eq!(
            function.parameters(),
            &[FunctionParameter::FunctionParameter {
                param_type: Type::Boolean,
                param_name: "my_param".to_owned()
            }]
        );
        assert_eq!(function.return_type(), &FunctionReturnType::Void);
    }

    #[test]
    fn intrinsic_function_getters() {
        let function = Function::Intrinsic {
            id: FunctionId("my_function".to_owned()),
            name: "my_function".to_owned(),
            parameters: vec![FunctionParameter::FunctionParameter {
                param_type: Type::Boolean,
                param_name: "my_param".to_owned(),
            }],
            return_type: FunctionReturnType::Void,
        };

        assert_eq!(function.id(), &FunctionId("my_function".to_owned()));
        assert_eq!(function.name(), "my_function");
        assert_eq!(
            function.parameters(),
            &[FunctionParameter::FunctionParameter {
                param_type: Type::Boolean,
                param_name: "my_param".to_owned()
            }]
        );
        assert_eq!(function.return_type(), &FunctionReturnType::Void);
    }
}
