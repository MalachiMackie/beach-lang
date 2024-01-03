use crate::ast::node::{Type, Value};

impl Value {
    pub fn get_type(&self) -> Type {
        match self {
            Value::UInt(_) => Type::UInt,
            Value::Boolean(_) => Type::Boolean,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::node::{BoolValue, Type, UIntValue, Value};

    #[test]
    fn value_get_type_uint() {
        let result = Value::UInt(10.into()).get_type();

        assert_eq!(result, Type::UInt)
    }

    #[test]
    fn value_get_type_boolean() {
        let result = Value::Boolean(true.into()).get_type();

        assert_eq!(result, Type::Boolean)
    }
}
