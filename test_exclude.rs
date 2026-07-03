fn main() {
    let exclude_patterns = vec!["**/target/**", "**/node_modules/**", "**/.git/**", "**/build/**", "**/dist/**", "**/__pycache__/**"];
    let paths = vec![
        "./.git",
        "./.git/objects",
        "./.git/objects/04",
        "./.git/objects/04/e8c2c63f954263be466c5eca75bfdac15add8b"
    ];

    for path in paths {
        let mut excluded = false;
        for pattern in &exclude_patterns {
            if path.contains(pattern.trim_matches('*')) {
                excluded = true;
                break;
            }
        }
        println!("{} -> {}", path, excluded);
    }
}
