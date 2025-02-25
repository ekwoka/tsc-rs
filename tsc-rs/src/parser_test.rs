#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_typescript() {
        let source = r#"
            let x: number = 42;
            let y: string = "Hello";
        "#;

        let result = parse_typescript(source);
        assert!(result.is_ok(), "Failed to parse valid TypeScript: {:?}", result);
    }

    #[test]
    fn test_parse_invalid_typescript() {
        let source = r#"
            let x: number = "not a number";  // Type mismatch
        "#;

        let result = parse_typescript(source);
        // For now, this will pass because we're not doing type checking yet
        assert!(result.is_ok(), "Parser should accept invalid types for now");
    }
}
