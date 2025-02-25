use std::sync::Arc;
use oxc_ast::ast::*;
use crate::types::*;

pub struct TypeChecker {
    errors: Vec<String>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            errors: Vec::new(),
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
                    if let Some(init) = &decl.init {
                        let init_type = self.check_expression(init);
                        if let Some(type_ann) = &decl.id.type_annotation {
                            let declared_type = self.check_type(&type_ann.type_annotation);
                            if !check_type_compatibility(&declared_type, &init_type) {
                                self.errors.push(format!(
                                    "Type '{}' is not assignable to type '{}'",
                                    init_type,
                                    declared_type
                                ));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn check_type(&self, ts_type: &TSType) -> Type {
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

    fn check_expression(&self, expr: &Expression) -> Type {
        match expr {
            Expression::NumericLiteral(_) => Type::Number,
            Expression::StringLiteral(_) => Type::String,
            Expression::BooleanLiteral(_) => Type::Boolean,
            Expression::NullLiteral(_) => Type::Null,
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
}
