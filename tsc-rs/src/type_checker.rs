use crate::types::*;
use oxc_ast::ast::*;
use std::collections::HashMap;
use std::sync::Arc;

pub struct TypeChecker {
    errors: Vec<String>,
    symbol_table: HashMap<String, Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            errors: Vec::new(),
            symbol_table: HashMap::new(),
        }
    }

    pub fn check_program(&mut self, program: &Program) {
        for item in &program.body {
            self.check_statement(item);
        }
    }

    fn check_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                for decl in &var_decl.declarations {
                    if let BindingPatternKind::BindingIdentifier(ident) = &decl.id.kind {
                        let var_type = if let Some(type_ann) = &decl.id.type_annotation {
                            self.check_type(&type_ann.type_annotation)
                        } else if let Some(init) = &decl.init {
                            self.check_expression(init)
                        } else {
                            Type::Any
                        };
                        self.symbol_table
                            .insert(ident.name.to_string(), var_type.clone());

                        if let Some(init) = &decl.init {
                            let init_type = self.check_expression(init);
                            if !check_type_compatibility(&var_type, &init_type) {
                                self.errors.push(format!(
                                    "Type '{}' is not assignable to type '{}'",
                                    init_type, var_type
                                ));
                            }
                        }
                    }
                }
            }
            Statement::FunctionDeclaration(func_decl) => {
                // Add function to symbol table
                if let Some(ident) = &func_decl.id {
                    let mut param_types = Vec::new();
                    for param in &func_decl.params.items {
                        let param_type = if let Some(type_ann) = &param.pattern.type_annotation {
                            self.check_type(&type_ann.type_annotation)
                        } else {
                            Type::Any
                        };
                        if let BindingPatternKind::BindingIdentifier(ident) = &param.pattern.kind {
                            self.symbol_table
                                .insert(ident.name.to_string(), param_type.clone());
                        }
                        param_types.push(param_type);
                    }
                    let return_type = if let Some(return_type) = &func_decl.return_type {
                        self.check_type(&return_type.type_annotation)
                    } else {
                        Type::Any
                    };

                    self.symbol_table.insert(
                        ident.name.to_string(),
                        Type::Function {
                            params: param_types.clone(),
                            return_type: Arc::new(return_type.clone()),
                        },
                    );

                    // Check function body
                    if let Some(body) = &func_decl.body {
                        for stmt in &body.statements {
                            match stmt {
                                Statement::ReturnStatement(ret_stmt) => {
                                    if let Some(arg) = &ret_stmt.argument {
                                        let actual_return_type = self.check_expression(arg);
                                        if !check_type_compatibility(
                                            &return_type,
                                            &actual_return_type,
                                        ) {
                                            self.errors.push(format!(
                                                "Type '{}' is not assignable to type '{}'",
                                                actual_return_type, return_type
                                            ));
                                        }
                                    }
                                }
                                _ => self.check_statement(stmt),
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub fn check_type(&self, ts_type: &TSType) -> Type {
        match ts_type {
            TSType::TSAnyKeyword(_) => Type::Any,
            TSType::TSNumberKeyword(_) => Type::Number,
            TSType::TSStringKeyword(_) => Type::String,
            TSType::TSBooleanKeyword(_) => Type::Boolean,
            TSType::TSNullKeyword(_) => Type::Null,
            TSType::TSUndefinedKeyword(_) => Type::Undefined,
            TSType::TSNeverKeyword(_) => Type::Never,
            TSType::TSBigIntKeyword(_) => Type::BigInt,
            TSType::TSSymbolKeyword(_) => Type::Symbol,
            TSType::TSObjectKeyword(_) => Type::Object,
            TSType::TSUnknownKeyword(_) => Type::Unknown,
            TSType::TSVoidKeyword(_) => Type::Void,
            TSType::TSArrayType(array_type) => {
                let elem_type = self.check_type(&array_type.element_type);
                Type::Array(Arc::new(elem_type))
            }
            TSType::TSTupleType(tuple_type) => {
                let types: Vec<Type> = tuple_type
                    .element_types
                    .iter()
                    .map(|t| {
                        if let Some(ts_type) = t.as_ts_type() {
                            self.check_type(ts_type)
                        } else {
                            Type::Any // Default to Any if not a TSType
                        }
                    })
                    .collect();
                Type::Tuple(types)
            }
            TSType::TSUnionType(union_type) => {
                let types: Vec<Type> = union_type
                    .types
                    .iter()
                    .map(|t| self.check_type(t))
                    .collect();
                Type::Union(types)
            }
            TSType::TSFunctionType(func_type) => {
                let params: Vec<Type> = func_type
                    .params
                    .items
                    .iter()
                    .filter_map(|t| {
                        t.pattern
                            .type_annotation
                            .as_ref()
                            .map(|ann| self.check_type(&ann.type_annotation))
                    })
                    .collect();
                let return_type = Arc::new(self.check_type(&func_type.return_type.type_annotation));
                Type::Function {
                    params,
                    return_type,
                }
            }
            _ => Type::Any,
        }
    }

    pub fn check_expression(&mut self, expr: &Expression) -> Type {
        match expr {
            Expression::NumericLiteral(_) => Type::Number,
            Expression::BigIntLiteral(_) => Type::BigInt,
            Expression::StringLiteral(_) => Type::String,
            Expression::BooleanLiteral(_) => Type::Boolean,
            Expression::NullLiteral(_) => Type::Null,
            Expression::Identifier(ident) => match ident.name.as_str() {
                "number" => Type::Number,
                "string" => Type::String,
                "boolean" => Type::Boolean,
                "bigint" => Type::BigInt,
                "symbol" => Type::Symbol,
                "null" => Type::Null,
                "never" => Type::Never,
                "void" => Type::Void,
                "unknown" => Type::Unknown,
                "any" => Type::Any,
                _ => self
                    .symbol_table
                    .get(ident.name.as_str())
                    .cloned()
                    .unwrap_or(Type::Any),
            },
            Expression::ArrayExpression(array_expr) => {
                if let Some(first) = array_expr.elements.first() {
                    if let Some(expr) = first.as_expression() {
                        let elem_type = self.check_expression(expr);
                        Type::Array(Arc::new(elem_type))
                    } else {
                        Type::Array(Arc::new(Type::Any))
                    }
                } else {
                    Type::Array(Arc::new(Type::Any))
                }
            }
            Expression::BinaryExpression(bin_expr) => {
                let left_type = self.check_expression(&bin_expr.left);
                let right_type = self.check_expression(&bin_expr.right);

                match bin_expr.operator {
                    BinaryOperator::Addition => {
                        if matches!(left_type, Type::String) || matches!(right_type, Type::String) {
                            Type::String
                        } else {
                            match (left_type.clone(), right_type.clone()) {
                                (Type::BigInt, Type::BigInt) => Type::BigInt,
                                (Type::Number, Type::Number) => Type::Number,
                                (Type::BigInt, _) | (_, Type::BigInt) => {
                                    self.errors.push(format!(
                                        "The binary operation between '{}' and '{}' is not allowed",
                                        left_type, right_type
                                    ));
                                    Type::Number
                                }
                                _ => Type::Number, // Default to number for other numeric operations
                            }
                        }
                    }
                    BinaryOperator::Subtraction
                    | BinaryOperator::Multiplication
                    | BinaryOperator::Division
                    | BinaryOperator::Remainder
                    | BinaryOperator::Exponential => {
                        match (left_type.clone(), right_type.clone()) {
                            (Type::BigInt, Type::BigInt) => Type::BigInt,
                            (Type::Number, Type::Number) => Type::Number,
                            (Type::BigInt, _) | (_, Type::BigInt) => {
                                self.errors.push(format!(
                                    "The binary operation between '{}' and '{}' is not allowed",
                                    left_type, right_type
                                ));
                                Type::Number
                            }
                            _ => Type::Any,
                        }
                    }
                    BinaryOperator::LessThan
                    | BinaryOperator::LessEqualThan
                    | BinaryOperator::GreaterThan
                    | BinaryOperator::GreaterEqualThan
                    | BinaryOperator::Equality
                    | BinaryOperator::Inequality
                    | BinaryOperator::StrictEquality
                    | BinaryOperator::StrictInequality
                    | BinaryOperator::In
                    | BinaryOperator::Instanceof => Type::Boolean,

                    BinaryOperator::BitwiseAnd
                    | BinaryOperator::BitwiseOR
                    | BinaryOperator::BitwiseXOR
                    | BinaryOperator::ShiftLeft
                    | BinaryOperator::ShiftRight
                    | BinaryOperator::ShiftRightZeroFill => {
                        match (left_type.clone(), right_type.clone()) {
                            (Type::BigInt, Type::BigInt) => Type::BigInt,
                            (Type::Number, Type::Number) => Type::Number,
                            (Type::BigInt, _) | (_, Type::BigInt) => {
                                self.errors.push(format!(
                                    "The binary operation between '{}' and '{}' is not allowed",
                                    left_type, right_type
                                ));
                                Type::Number
                            }
                            _ => Type::Number, // Default to Number for bitwise operations
                        }
                    }
                    _ => Type::Any,
                }
            }
            _ => Type::Any,
        }
    }

    pub fn get_errors(&self) -> &[String] {
        &self.errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_typescript;

    #[test]
    fn test_type_checker() {
        let source = r#"
            let x: number = 42;
            let y: string = "hello";
            let z: number = "world"; // This should cause a type error
        "#;

        let ts_program = parse_typescript(source).unwrap();
        let mut checker = TypeChecker::new();
        checker.check_program(&ts_program.program);

        let errors = checker.get_errors();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("not assignable"));
    }

    #[test]
    fn test_function_type_checking() {
        // Test 1: Basic function with explicit return type
        let source1 = r#"
            function add(x: number, y: number): number {
                return x + y;
            }
        "#;
        let ts_program = parse_typescript(source1).unwrap();
        let mut checker = TypeChecker::new();
        checker.check_program(&ts_program.program);
        assert_eq!(
            checker.get_errors().len(),
            0,
            "Basic function should have no errors"
        );

        // Test 2: Function with inferred return type
        let source2 = r#"
            function greet(name: string) {
                return "Hello, " + name;
            }
        "#;
        let ts_program = parse_typescript(source2).unwrap();
        let mut checker = TypeChecker::new();
        checker.check_program(&ts_program.program);
        assert_eq!(
            checker.get_errors().len(),
            0,
            "String concatenation with name should have no errors"
        );

        // Test 3: Function with type mismatch
        let source3 = r#"
            function broken(x: number): string {
                return x;  // Should error: number is not assignable to string
            }
        "#;
        let ts_program = parse_typescript(source3).unwrap();
        let mut checker = TypeChecker::new();
        checker.check_program(&ts_program.program);
        let errors = checker.get_errors();
        println!("Test 3 errors: {:?}", errors);
        assert_eq!(
            errors.len(),
            1,
            "Should have exactly one error for type mismatch"
        );
        assert_eq!(
            errors[0],
            "Type 'number' is not assignable to type 'string'"
        );

        // Test 4: Function with string + number concatenation
        let source4 = r#"
            function concat(a: string): string {
                return a + 42;  // Valid: string + number returns string
            }
        "#;
        let ts_program = parse_typescript(source4).unwrap();
        let mut checker = TypeChecker::new();
        checker.check_program(&ts_program.program);
        let errors = checker.get_errors();
        println!("Test 4 errors: {:?}", errors);
        assert_eq!(
            errors.len(),
            0,
            "String + number concatenation should have no errors"
        );
    }

    #[test]
    fn test_binary_expression_types() {
        let source = r#"
            // Arithmetic operators
            let a1 = 5 + 3;          // number
            let a2 = 10 - 4;         // number
            let a3 = 6 * 2;          // number
            let a4 = 15 / 3;         // number
            let a5 = 10 % 3;         // number
            let a6 = 2 ** 3;         // number

            // String concatenation
            let s1 = "hello" + "world";  // string
            let s2 = "count: " + 42;     // string
            let s3 = 42 + "items";       // string

            // Comparison operators
            let c1 = 5 > 3;          // boolean
            let c2 = 10 <= 4;        // boolean
            let c3 = "a" < "b";      // boolean
            let c4 = 42 >= 42;       // boolean
            let c5 = "x" == "y";     // boolean
            let c6 = 5 != 3;         // boolean

            // Bitwise operators
            let b1 = 5 & 3;          // number
            let b2 = 10 | 4;         // number
            let b3 = 6 ^ 2;          // number
            let b4 = 8 << 2;         // number
            let b5 = 16 >> 2;        // number
            let b6 = -8 >>> 2;       // number
        "#;

        let ts_program = parse_typescript(source).unwrap();
        let mut checker = TypeChecker::new();
        checker.check_program(&ts_program.program);

        // Helper function to get the type of a variable declaration
        let program = parse_typescript(source).unwrap();
        let mut get_var_type = |var_name: &str| -> Type {
            for stmt in &program.program.body {
                if let Statement::VariableDeclaration(var_decl) = stmt {
                    for decl in &var_decl.declarations {
                        if let BindingPatternKind::BindingIdentifier(ident) = &decl.id.kind {
                            if ident.name == var_name {
                                if let Some(type_annotation) = &decl.id.type_annotation {
                                    return checker.check_type(&type_annotation.type_annotation);
                                } else if let Some(init) = &decl.init {
                                    return checker.check_expression(init);
                                }
                            }
                        }
                    }
                }
            }
            Type::Any
        };

        // Test arithmetic operators
        assert!(matches!(get_var_type("a1"), Type::Number));
        assert!(matches!(get_var_type("a2"), Type::Number));
        assert!(matches!(get_var_type("a3"), Type::Number));
        assert!(matches!(get_var_type("a4"), Type::Number));
        assert!(matches!(get_var_type("a5"), Type::Number));
        assert!(matches!(get_var_type("a6"), Type::Number));

        // Test string concatenation
        assert!(matches!(get_var_type("s1"), Type::String));
        assert!(matches!(get_var_type("s2"), Type::String));
        assert!(matches!(get_var_type("s3"), Type::String));

        // Test comparison operators
        assert!(matches!(get_var_type("c1"), Type::Boolean));
        assert!(matches!(get_var_type("c2"), Type::Boolean));
        assert!(matches!(get_var_type("c3"), Type::Boolean));
        assert!(matches!(get_var_type("c4"), Type::Boolean));
        assert!(matches!(get_var_type("c5"), Type::Boolean));
        assert!(matches!(get_var_type("c6"), Type::Boolean));

        // Test bitwise operators
        assert!(matches!(get_var_type("b1"), Type::Number));
        assert!(matches!(get_var_type("b2"), Type::Number));
        assert!(matches!(get_var_type("b3"), Type::Number));
        assert!(matches!(get_var_type("b4"), Type::Number));
        assert!(matches!(get_var_type("b5"), Type::Number));
        assert!(matches!(get_var_type("b6"), Type::Number));
    }

    #[test]
    fn test_bigint_binary_expression_types() {
        let mut checker = TypeChecker::new();
        let ts_program = r#"
            let a: bigint = 1n;
            let b: bigint = 2n;
            let c: number = 3;

            // BigInt arithmetic
            let d = a + b;  // Should be bigint
            let e = a - b;  // Should be bigint
            let f = a * b;  // Should be bigint
            let g = a / b;  // Should be bigint
            let h = a % b;  // Should be bigint

            // Mixed BigInt and Number (should produce errors)
            let i = a + c;  // Should produce error
            let j = c - a;  // Should produce error

            // BigInt bitwise operations
            let k = a & b;  // Should be bigint
            let l = a | b;  // Should be bigint
            let m = a ^ b;  // Should be bigint
            let n = a << b; // Should be bigint
            let o = a >> b; // Should be bigint

            // Mixed BigInt and Number bitwise (should produce errors)
            let p = a & c;  // Should produce error
            let q = c | a;  // Should produce error
        "#;

        let program = parse_typescript(ts_program).unwrap();
        checker.check_program(&program.program);
        let mut get_var_type = |var_name: &str| -> Type {
            for stmt in &program.program.body {
                if let Statement::VariableDeclaration(var_decl) = stmt {
                    for decl in &var_decl.declarations {
                        if let BindingPatternKind::BindingIdentifier(ident) = &decl.id.kind {
                            if ident.name == var_name {
                                if let Some(type_annotation) = &decl.id.type_annotation {
                                    return checker.check_type(&type_annotation.type_annotation);
                                } else if let Some(init) = &decl.init {
                                    return checker.check_expression(init);
                                }
                            }
                        }
                    }
                }
            }
            Type::Any
        };

        // Test initial numbers
        assert_eq!(get_var_type("a"), Type::BigInt);
        assert_eq!(get_var_type("b"), Type::BigInt);
        assert_eq!(get_var_type("c"), Type::Number);

        // Test BigInt arithmetic results
        assert_eq!(get_var_type("d"), Type::BigInt);
        assert_eq!(get_var_type("e"), Type::BigInt);
        assert_eq!(get_var_type("f"), Type::BigInt);
        assert_eq!(get_var_type("g"), Type::BigInt);
        assert_eq!(get_var_type("h"), Type::BigInt);

        // Test mixed BigInt and Number operations (should be Any due to errors)
        assert_eq!(get_var_type("i"), Type::Number);
        assert_eq!(get_var_type("j"), Type::Number);

        // Test BigInt bitwise operation results
        assert_eq!(get_var_type("k"), Type::BigInt);
        assert_eq!(get_var_type("l"), Type::BigInt);
        assert_eq!(get_var_type("m"), Type::BigInt);
        assert_eq!(get_var_type("n"), Type::BigInt);
        assert_eq!(get_var_type("o"), Type::BigInt);

        // Test mixed BigInt and Number bitwise operations (should be Any due to errors)
        assert_eq!(get_var_type("p"), Type::Number);
        assert_eq!(get_var_type("q"), Type::Number);

        // Verify that appropriate error messages were generated
        assert!(
            checker
                .errors
                .iter()
                .any(|e| e.contains("The binary operation between"))
        );
    }
}
