use std::collections::HashMap;

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
    FunctionDeclaration(FunctionDeclaration),
    FunctionCall {
        function_id: FunctionId,
        parameters: Vec<Expression>
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDeclaration {
    pub id: FunctionId,
    pub name: String,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: FunctionReturnType,
    pub body: Ast,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FunctionReturnType {
    Type(Type),
    Void,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionParameter {
    pub param_type: Type,
    pub param_name: String,
}

impl From<(Type, String)> for FunctionParameter {
    fn from((param_type, name): (Type, String)) -> Self {
        FunctionParameter {
            param_name: name,
            param_type,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    ValueLiteral(Value),
    FunctionCall(FunctionId, Vec<Expression>),
    Operation(Operation),
}

#[derive(Debug, PartialEq, Hash, Clone, Eq)]
pub struct FunctionId(pub String);

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    UInt(UIntValue),
    Boolean(BoolValue),
}

#[derive(Clone, Debug, PartialEq)]
pub enum VariableDeclarationType {
    Infer,
    Type(Type),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    UInt,
    Boolean,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UIntValue(pub u32);

#[derive(Clone, Debug, PartialEq)]
pub struct BoolValue(pub bool);

#[derive(Clone, Debug, PartialEq)]
pub enum Operation {
    Unary(UnaryOperation),
}

#[derive(Clone, Debug, PartialEq)]
pub enum UnaryOperation {
    Not { value: Box<Expression> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ast {
    pub functions: HashMap<FunctionId, FunctionDeclaration>,
    pub nodes: Vec<Node>,
}
