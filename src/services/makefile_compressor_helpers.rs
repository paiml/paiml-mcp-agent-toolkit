fn extract_package_name(line: &str, after: &str) -> Option<String> {
    debug_assert!(!line.is_empty(), "line must not be empty");
    debug_assert!(!after.is_empty(), "after must not be empty");
    let parts: Vec<&str> = line.split_whitespace().collect();

    let install_pos = find_install_position(&parts, after)?;
    find_package_at_position(&parts, install_pos)
}

fn find_install_position(parts: &[&str], after: &str) -> Option<usize> {
    debug_assert!(!after.is_empty(), "after must not be empty");
    match after {
        "cargo install" => find_cargo_install_position(parts),
        "npm install" => find_npm_install_position(parts),
        "install" => find_simple_install_position(parts),
        _ => None,
    }
}

fn find_cargo_install_position(parts: &[&str]) -> Option<usize> {
    parts.iter().position(|&p| p == "cargo")?;
    parts
        .iter()
        .position(|&p| p == "install")
        .map(|pos| pos + 1)
}

fn find_npm_install_position(parts: &[&str]) -> Option<usize> {
    parts.iter().position(|&p| p == "npm")?;
    parts
        .iter()
        .position(|&p| p == "install")
        .map(|pos| pos + 1)
}

fn find_simple_install_position(parts: &[&str]) -> Option<usize> {
    parts
        .iter()
        .position(|&p| p == "install")
        .map(|pos| pos + 1)
}

fn find_package_at_position(parts: &[&str], position: usize) -> Option<String> {
    // Check primary position
    if let Some(pkg) = get_valid_package(parts, position) {
        return Some(pkg);
    }

    // Check next position if primary was a flag
    get_valid_package(parts, position + 1)
}

fn get_valid_package(parts: &[&str], position: usize) -> Option<String> {
    parts
        .get(position)
        .filter(|pkg| !pkg.starts_with('-') && !pkg.is_empty())
        .map(|pkg| (*pkg).to_string())
}
