use pmat::services::file_discovery::ProjectFileDiscovery;
use std::path::PathBuf;

fn main() {
    let discovery = ProjectFileDiscovery::new(PathBuf::from("."));
    let files = discovery.discover_files().unwrap();
    let mut count_claude = 0;
    for f in &files {
        let s = f.to_string_lossy();
        if s.contains(".claude") {
            count_claude += 1;
            println!("Found in .claude: {}", s);
            if count_claude > 10 {
                println!("... and more");
                break;
            }
        }
    }
    println!("Total files found: {}", files.len());
    println!("Total in .claude: {}", count_claude);
}
