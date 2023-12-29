#[derive(Debug, PartialEq)]
pub enum Node {
    Literal {
        value: Value,
    },
    Operation {
        operation: Operation,
    },
    VariableDeclaration {
        var_type: VariableDeclarationType,
        var_name: String,
        value: Expression,
    },
    FunctionReturn {
        return_value: Option<Expression>,
    },
    FunctionDeclaration {
        id: FunctionId,
        name: String,
        parameters: Vec<FunctionParameter>,
        return_type: Type,
        body: Vec<Node>,
    },
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum Expression {
    ValueLiteral(Value),
    FunctionCall(FunctionId),
}

#[derive(Debug, PartialEq)]
pub struct FunctionId(pub String);

#[derive(Debug, PartialEq)]
pub enum Value {
    UInt(UIntValue),
    Boolean(BoolValue),
}

#[derive(Debug, PartialEq)]
pub enum VariableDeclarationType {
    Infer,
    Type(Type),
}

#[derive(Debug, PartialEq)]
pub enum Type {
    UInt,
    Boolean,
}

#[derive(Debug, PartialEq)]
pub struct UIntValue(pub u32);
#[derive(Debug, PartialEq)]
pub struct BoolValue(pub bool);

#[derive(Debug, PartialEq)]
pub enum Operation {
    Unary(UnaryOperation),
}

#[derive(Debug, PartialEq)]
pub enum UnaryOperation {
    Not { value: Expression },
}
