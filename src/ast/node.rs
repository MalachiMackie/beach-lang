#[derive(Debug, PartialEq)]
pub enum Node {
    Literal {
        info: NodeInfo,
        value: Value,
    },
    Operation {
        info: NodeInfo,
        operation: Operation,
    },
    VariableDeclaration {
        type_info: NodeInfo,
        name_info: NodeInfo,
        var_type: VariableDeclarationType,
        var_name: String,
    },
    VariableAssignment {
        info: NodeInfo,
        value: Expression,
    },
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    ValueLiteral(Value),
}

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
    Not { value: BoolValue },
}

#[derive(Debug, PartialEq)]
pub struct NodeInfo {
    pub line: u32,
    pub character: u32,
}
