// This module will contain our type system implementation
use oxc_span::Span;
use std::fmt;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // Basic types
    Any,
    Number,
    String,
    Boolean,
    Null,
    Undefined,
    Never,
    BigInt,
    Symbol,
    Object,
    Unknown,
    Void,
    // Literal types
    StringLiteral(String),
    NumberLiteral(f64),
    BooleanLiteral(bool),
    // Compound types
    Union(Vec<Type>),
    Array(Arc<Type>),
    Tuple(Vec<Type>),
    Function {
        params: Vec<Type>,
        return_type: Arc<Type>,
    },
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Any => write!(f, "any"),
            Type::Number => write!(f, "number"),
            Type::String => write!(f, "string"),
            Type::Boolean => write!(f, "boolean"),
            Type::Null => write!(f, "null"),
            Type::Undefined => write!(f, "undefined"),
            Type::Never => write!(f, "never"),
            Type::BigInt => write!(f, "bigint"),
            Type::Symbol => write!(f, "symbol"),
            Type::Object => write!(f, "object"),
            Type::Unknown => write!(f, "unknown"),
            Type::Void => write!(f, "void"),
            Type::StringLiteral(s) => write!(f, "\"{}\"", s),
            Type::NumberLiteral(n) => write!(f, "{}", n),
            Type::BooleanLiteral(b) => write!(f, "{}", b),
            Type::Union(types) => {
                let types_str: Vec<String> = types.iter().map(|t| t.to_string()).collect();
                write!(f, "{}", types_str.join(" | "))
            }
            Type::Array(elem_type) => write!(f, "{}[]", elem_type),
            Type::Tuple(types) => {
                let types_str: Vec<String> = types.iter().map(|t| t.to_string()).collect();
                write!(f, "[{}]", types_str.join(", "))
            }
            Type::Function {
                params,
                return_type,
            } => {
                let params_str: Vec<String> = params.iter().map(|t| t.to_string()).collect();
                write!(f, "({}) => {}", params_str.join(", "), return_type)
            }
        }
    }
}

#[derive(Debug)]
pub struct TypeError {
    pub message: String,
    pub span: Option<Span>,
}

impl TypeError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
        }
    }

    pub fn with_span(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span: Some(span),
        }
    }
}

pub fn infer_type_from_literal(value: &str) -> Type {
    // Remove quotes if present
    let value = value.trim_matches('"').trim_matches('\'');

    // Basic type inference from string literals
    if value == "null" {
        Type::Null
    } else if let Ok(num) = value.parse::<f64>() {
        Type::NumberLiteral(num)
    } else if value == "true" {
        Type::BooleanLiteral(true)
    } else if value == "false" {
        Type::BooleanLiteral(false)
    } else {
        Type::StringLiteral(value.to_string())
    }
}

pub fn check_type_compatibility(expected: &Type, actual: &Type) -> bool {
    match (expected, actual) {
        // Any type can be assigned to any
        (Type::Any, _) => true,
        (Type::Number, Type::Number) => true,
        (Type::String, Type::String) => true,
        (Type::Boolean, Type::Boolean) => true,
        (Type::Null, Type::Null) => true,
        (Type::Undefined, Type::Undefined) => true,
        (Type::Never, Type::Never) => true,
        (Type::BigInt, Type::BigInt) => true,
        (Type::Symbol, Type::Symbol) => true,
        (Type::Object, Type::Object) => true,
        (Type::Unknown, Type::Unknown) => true,
        (Type::Void, Type::Void) => true,
        // Literal types can be assigned to their corresponding base types
        (Type::Number, Type::NumberLiteral(_)) => true,
        (Type::String, Type::StringLiteral(_)) => true,
        (Type::Boolean, Type::BooleanLiteral(_)) => true,
        // Literal types must match exactly
        (Type::NumberLiteral(n1), Type::NumberLiteral(n2)) => n1 == n2,
        (Type::StringLiteral(s1), Type::StringLiteral(s2)) => s1 == s2,
        (Type::BooleanLiteral(b1), Type::BooleanLiteral(b2)) => b1 == b2,
        (Type::Union(types), actual) => types.iter().any(|t| check_type_compatibility(t, actual)),
        (Type::Array(expected_elem), Type::Array(actual_elem)) => {
            check_type_compatibility(expected_elem, actual_elem)
        }
        (Type::Tuple(expected_types), Type::Tuple(actual_types)) => {
            expected_types.len() == actual_types.len()
                && expected_types
                    .iter()
                    .zip(actual_types.iter())
                    .all(|(expected, actual)| check_type_compatibility(expected, actual))
        }
        (
            Type::Function {
                params: params1,
                return_type: return1,
            },
            Type::Function {
                params: params2,
                return_type: return2,
            },
        ) => {
            params1.len() == params2.len()
                && params1
                    .iter()
                    .zip(params2.iter())
                    .all(|(p1, p2)| check_type_compatibility(p1, p2))
                && check_type_compatibility(return1, return2)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_inference() {
        assert_eq!(infer_type_from_literal("null"), Type::Null);
        assert_eq!(infer_type_from_literal("42"), Type::NumberLiteral(42.0));
        assert_eq!(infer_type_from_literal("true"), Type::BooleanLiteral(true));
        assert_eq!(
            infer_type_from_literal("false"),
            Type::BooleanLiteral(false)
        );
        assert_eq!(
            infer_type_from_literal("hello"),
            Type::StringLiteral("hello".to_string())
        );
    }

    #[test]
    fn test_type_compatibility() {
        // Test basic type compatibility
        assert!(check_type_compatibility(&Type::Any, &Type::Number));
        assert!(check_type_compatibility(&Type::Number, &Type::Number));
        assert!(!check_type_compatibility(&Type::String, &Type::Number));

        // Test array type compatibility
        let number_array = Type::Array(Arc::new(Type::Number));
        let string_array = Type::Array(Arc::new(Type::String));
        let any_array = Type::Array(Arc::new(Type::Any));

        assert!(check_type_compatibility(&any_array, &number_array));
        assert!(check_type_compatibility(&number_array, &number_array));
        assert!(!check_type_compatibility(&string_array, &number_array));

        // Test function type compatibility
        let func1 = Type::Function {
            params: vec![Type::Number],
            return_type: Arc::new(Type::Boolean),
        };
        let func2 = Type::Function {
            params: vec![Type::Number],
            return_type: Arc::new(Type::Boolean),
        };
        let func3 = Type::Function {
            params: vec![Type::String],
            return_type: Arc::new(Type::Boolean),
        };

        assert!(check_type_compatibility(&func1, &func2));
        assert!(!check_type_compatibility(&func1, &func3));
    }

    #[test]
    fn test_literal_types() {
        // Test string literal types
        let hello_type = Type::StringLiteral("hello".to_string());
        let world_type = Type::StringLiteral("world".to_string());
        let string_type = Type::String;

        assert!(check_type_compatibility(&string_type, &hello_type));
        assert!(!check_type_compatibility(&hello_type, &world_type));
        assert!(check_type_compatibility(&hello_type, &hello_type));

        // Test number literal types
        let num_42 = Type::NumberLiteral(42.0);
        let num_43 = Type::NumberLiteral(43.0);
        let number_type = Type::Number;

        assert!(check_type_compatibility(&number_type, &num_42));
        assert!(!check_type_compatibility(&num_42, &num_43));
        assert!(check_type_compatibility(&num_42, &num_42));

        // Test boolean literal types
        let true_type = Type::BooleanLiteral(true);
        let false_type = Type::BooleanLiteral(false);
        let boolean_type = Type::Boolean;

        assert!(check_type_compatibility(&boolean_type, &true_type));
        assert!(!check_type_compatibility(&true_type, &false_type));
        assert!(check_type_compatibility(&true_type, &true_type));

        // Test literal type display
        assert_eq!(hello_type.to_string(), "\"hello\"");
        assert_eq!(num_42.to_string(), "42");
        assert_eq!(true_type.to_string(), "true");
    }
}
