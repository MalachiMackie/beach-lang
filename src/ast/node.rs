#[derive(Debug, PartialEq)]
pub enum Node {
    Literal { info: NodeInfo, value: Value },
}

impl Node {
    pub fn info(&self) -> &NodeInfo {
        match self {
            Self::Literal { info, .. } => info,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Value {
    UInt(u32),
}

#[derive(Debug, PartialEq)]
pub struct NodeInfo {
    pub line: u32,
    pub character: u32,
}
