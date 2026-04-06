fn analyze_python_dead_code(
    lines: &[&str],
    dead_functions: &mut usize,
    dead_classes: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    debug_assert!(!lines.is_empty(), "lines must not be empty");
    analyze_python_dead_functions(lines, dead_functions, dead_items);
    analyze_python_dead_classes(lines, dead_classes, dead_items);
}

/// Analyze dead functions in Python code
#[allow(clippy::cast_possible_truncation)]
fn analyze_python_dead_functions(
    lines: &[&str],
    dead_functions: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    debug_assert!(!lines.is_empty(), "lines must not be empty");
    use crate::models::dead_code::{DeadCodeItem, DeadCodeType};

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("def _") {
            if let Some(function_name) = extract_python_function_name_if_unused(lines, trimmed) {
                *dead_functions += 1;
                dead_items.push(DeadCodeItem {
                    item_type: DeadCodeType::Function,
                    name: function_name,
                    line: (line_num + 1) as u32,
                    reason: "Private function with no apparent callers".to_string(),
                });
            }
        }
    }
}

/// Analyze dead classes in Python code
#[allow(clippy::cast_possible_truncation)]
fn analyze_python_dead_classes(
    lines: &[&str],
    dead_classes: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    debug_assert!(!lines.is_empty(), "lines must not be empty");
    use crate::models::dead_code::{DeadCodeItem, DeadCodeType};

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("class _") {
            if let Some(class_name) = extract_python_class_name_if_unused(lines, trimmed) {
                *dead_classes += 1;
                dead_items.push(DeadCodeItem {
                    item_type: DeadCodeType::Class,
                    name: class_name,
                    line: (line_num + 1) as u32,
                    reason: "Private class with no apparent usage".to_string(),
                });
            }
        }
    }
}

/// Extract Python function name if unused
fn extract_python_function_name_if_unused(lines: &[&str], trimmed: &str) -> Option<String> {
    debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
    debug_assert!(!lines.is_empty(), "lines must not be empty");
    let function_name = extract_python_function_name(trimmed);
    if !function_name.is_empty() && !is_function_called_in_file(lines, &function_name) {
        Some(function_name)
    } else {
        None
    }
}

/// Extract Python class name if unused
fn extract_python_class_name_if_unused(lines: &[&str], trimmed: &str) -> Option<String> {
    debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
    debug_assert!(!lines.is_empty(), "lines must not be empty");
    let class_name = extract_python_class_name(trimmed);
    if !class_name.is_empty() && !is_type_used_in_file(lines, &class_name) {
        Some(class_name)
    } else {
        None
    }
}

fn extract_function_name(line: &str) -> String {
    debug_assert!(!line.is_empty(), "line must not be empty");
    if let Some(start) = line.find("fn ") {
        let after_fn = line.get(start + 3..).unwrap_or_default();
        if let Some(paren_pos) = after_fn.find('(') {
            after_fn
                .get(..paren_pos)
                .unwrap_or_default()
                .trim()
                .to_string()
        } else {
            String::with_capacity(1024)
        }
    } else {
        String::with_capacity(1024)
    }
}

fn extract_struct_name(line: &str) -> String {
    debug_assert!(!line.is_empty(), "line must not be empty");
    if let Some(start) = line.find("struct ") {
        let after_struct = line.get(start + 7..).unwrap_or_default();
        after_struct
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string()
    } else {
        String::with_capacity(1024)
    }
}

fn extract_js_function_name(line: &str) -> String {
    debug_assert!(!line.is_empty(), "line must not be empty");
    if let Some(start) = line.find("function ") {
        let after_fn = line.get(start + 9..).unwrap_or_default();
        if let Some(paren_pos) = after_fn.find('(') {
            after_fn
                .get(..paren_pos)
                .unwrap_or_default()
                .trim()
                .to_string()
        } else {
            String::with_capacity(1024)
        }
    } else {
        String::with_capacity(1024)
    }
}

fn extract_class_name(line: &str) -> String {
    debug_assert!(!line.is_empty(), "line must not be empty");
    if let Some(start) = line.find("class ") {
        let after_class = line.get(start + 6..).unwrap_or_default();
        after_class
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string()
    } else {
        String::with_capacity(1024)
    }
}

fn extract_python_function_name(line: &str) -> String {
    debug_assert!(!line.is_empty(), "line must not be empty");
    if let Some(start) = line.find("def ") {
        let after_def = line.get(start + 4..).unwrap_or_default();
        if let Some(paren_pos) = after_def.find('(') {
            after_def
                .get(..paren_pos)
                .unwrap_or_default()
                .trim()
                .to_string()
        } else {
            String::with_capacity(1024)
        }
    } else {
        String::with_capacity(1024)
    }
}

fn extract_python_class_name(line: &str) -> String {
    debug_assert!(!line.is_empty(), "line must not be empty");
    if let Some(start) = line.find("class ") {
        let after_class = line.get(start + 6..).unwrap_or_default();
        if let Some(colon_pos) = after_class.find(':') {
            after_class
                .get(..colon_pos)
                .unwrap_or_default()
                .trim()
                .split('(')
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        } else {
            after_class
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string()
        }
    } else {
        String::with_capacity(1024)
    }
}

fn is_function_called_in_file(lines: &[&str], function_name: &str) -> bool {
    debug_assert!(!function_name.is_empty(), "function_name must not be empty");
    debug_assert!(!lines.is_empty(), "lines must not be empty");
    let call_pattern = format!("{function_name}(");
    lines.iter().any(|line| line.contains(&call_pattern))
}

fn is_type_used_in_file(lines: &[&str], type_name: &str) -> bool {
    debug_assert!(!type_name.is_empty(), "type_name must not be empty");
    debug_assert!(!lines.is_empty(), "lines must not be empty");
    lines.iter().any(|line| {
        line.contains(type_name)
            && (line.contains(&format!("new {type_name}"))
                || line.contains(&format!(": {type_name}"))
                || line.contains(&format!("<{type_name}>")))
    })
}

