use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_ast::ast::Program;
use oxc_span::SourceType;

pub struct TypeScriptProgram {
    pub program: Program<'static>,
    _allocator: Allocator, // Keep allocator alive as long as program
}

impl std::fmt::Debug for TypeScriptProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypeScriptProgram")
            .field("program", &self.program)
            .finish_non_exhaustive()
    }
}

pub fn parse_typescript(source_code: &str) -> Result<TypeScriptProgram, String> {
    let allocator = Allocator::default();
    let source_type = match SourceType::from_path("test.ts") {
        Ok(st) => st.with_typescript(true).with_module(true),
        Err(e) => return Err(format!("Unknown extension: {e:?}")),
    };

    let ret = Parser::new(&allocator, source_code, source_type).parse();
    
    // ParserReturn is not a Result, but contains diagnostics if there were errors
    if ret.errors.is_empty() {
        Ok(TypeScriptProgram {
            program: unsafe { std::mem::transmute(ret.program) },
            _allocator: allocator,
        })
    } else {
        Err(ret.errors.first().unwrap().to_string())
    }
}

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
