use std::fmt;

/// Error type for SQL type operations, mirroring .NET SqlTypes exception semantics.
#[derive(Debug, Clone, PartialEq)]
pub enum SqlTypeError {
    /// Attempted to access the value of a NULL SqlType.
    NullValue,
    /// Arithmetic overflow occurred.
    Overflow,
    /// Division by zero attempted.
    DivideByZero,
    /// Failed to parse a string into a SqlType.
    ParseError(String),
    /// Value is outside the valid range for the target SqlType.
    OutOfRange(String),
}

impl fmt::Display for SqlTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqlTypeError::NullValue => write!(
                f,
                "Data is Null. This method or property cannot be called on Null values."
            ),
            SqlTypeError::Overflow => write!(f, "Arithmetic overflow error."),
            SqlTypeError::DivideByZero => write!(f, "Divide by zero error."),
            SqlTypeError::ParseError(msg) => write!(f, "Failed to parse: {msg}"),
            SqlTypeError::OutOfRange(msg) => write!(f, "Value out of range: {msg}"),
        }
    }
}

impl std::error::Error for SqlTypeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_null_value() {
        let err = SqlTypeError::NullValue;
        assert_eq!(
            err.to_string(),
            "Data is Null. This method or property cannot be called on Null values."
        );
    }

    #[test]
    fn display_overflow() {
        let err = SqlTypeError::Overflow;
        assert_eq!(err.to_string(), "Arithmetic overflow error.");
    }

    #[test]
    fn display_divide_by_zero() {
        let err = SqlTypeError::DivideByZero;
        assert_eq!(err.to_string(), "Divide by zero error.");
    }

    #[test]
    fn display_parse_error() {
        let err = SqlTypeError::ParseError("invalid input".to_string());
        assert_eq!(err.to_string(), "Failed to parse: invalid input");
    }

    #[test]
    fn display_out_of_range() {
        let err = SqlTypeError::OutOfRange("value too large".to_string());
        assert_eq!(err.to_string(), "Value out of range: value too large");
    }

    #[test]
    fn error_is_clone() {
        let err = SqlTypeError::ParseError("test".to_string());
        let cloned = err.clone();
        assert_eq!(err, cloned);
    }

    #[test]
    fn error_is_debug() {
        let err = SqlTypeError::NullValue;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("NullValue"));
    }

    #[test]
    fn error_implements_std_error() {
        let err: Box<dyn std::error::Error> = Box::new(SqlTypeError::Overflow);
        assert_eq!(err.to_string(), "Arithmetic overflow error.");
    }
}
