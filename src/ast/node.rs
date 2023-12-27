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
}

impl Node {
    pub fn info(&self) -> &NodeInfo {
        match self {
            Self::Literal { info, .. } | Self::Operation { info, .. } => info,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Value {
    UInt(UIntValue),
    Boolean(BoolValue),
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
