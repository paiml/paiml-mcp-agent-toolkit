/// Generate test code with specified lines of code
fn generate_test_code(lines: usize) -> String {
    let mut code = String::with_capacity(lines * 50);
    code.push_str("// Generated test code for performance testing\n");
    code.push_str("use std::collections::HashMap;\n\n");
    code.push_str("pub struct TestStruct {\n");
    code.push_str("    data: HashMap<String, i32>,\n");
    code.push_str("}\n\n");

    for i in 0..lines.saturating_sub(10) {
        code.push_str(&format!("pub fn test_function_{i}() -> i32 {{\n"));
        code.push_str("    let mut sum = 0;\n");
        code.push_str(&format!("    for j in 0..{i} {{\n"));
        code.push_str(&format!("        sum += j * {i};\n"));
        code.push_str("    }\n");
        code.push_str("    sum\n");
        code.push_str("}\n\n");
    }

    code
}
