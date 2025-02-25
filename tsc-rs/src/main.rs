mod parser;
mod types;

fn main() {
    let source = r#"
        let x: number = 42;
        let y: string = "Hello";
    "#;

    match parser::parse_typescript(source) {
        Ok(_) => println!("Successfully parsed TypeScript code"),
        Err(e) => eprintln!("Error parsing TypeScript: {}", e),
    }
}
