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
        parameters: Vec<Expression>,
    },
    IfStatement(IfStatement),
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfStatement {
    pub check_expression: Expression,
        pub if_block: Vec<Node>,
        pub else_if_blocks: Vec<ElseIfBlock>,
        pub else_block: Option<Vec<Node>>
}

#[derive(Clone, Debug, PartialEq)]
pub struct ElseIfBlock {
    pub check: Expression,
    pub block: Vec<Node>,
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
pub enum FunctionParameter {
    FunctionParameter {
        param_type: Type,
        param_name: String,
    },
    IntrinsicAny {
        param_name: String,
    },
}

impl From<(Type, String)> for FunctionParameter {
    fn from((param_type, name): (Type, String)) -> Self {
        FunctionParameter::FunctionParameter {
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
    VariableAccess(String),
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
        body: Ast,
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
