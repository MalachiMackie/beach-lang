use crate::ast::node::Type;

mod expression;
mod operation;
mod value;
mod function_call;

#[derive(Debug)]
pub struct TypeCheckingError {
    message: String,
}

fn verify_type(actual_type: Option<Type>, expected_type: Type) -> Result<(), TypeCheckingError> {
    match actual_type {
        None => Err(TypeCheckingError {
            message: format!("Expected type to be {}, but none was found", expected_type),
        }),
        Some(found_type) if found_type != expected_type => Err(TypeCheckingError {
            message: format!(
                "Expected type to be {}, but found {}",
                expected_type, found_type
            ),
        }),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::node::Type;

    use super::verify_type;

    #[test]
    fn verify_type_success() {
        let result = verify_type(Some(Type::Boolean), Type::Boolean);

        assert!(matches!(result, Ok(_)));
    }

    #[test]
    fn verify_type_failure_none() {
        let result = verify_type(None, Type::Boolean);

        assert!(
            matches!(result, Err(e) if e.message == "Expected type to be Boolean, but none was found")
        );
    }

    #[test]
    fn verify_type_failure_incorrect_type() {
        let result = verify_type(Some(Type::UInt), Type::Boolean);

        assert!(
            matches!(result, Err(e) if e.message == "Expected type to be Boolean, but found UInt")
        );
    }
}
