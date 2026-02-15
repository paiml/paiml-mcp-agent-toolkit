//! Extract Demo - Demonstrates `pmat extract --list` for function boundary extraction
//!
//! This example shows how `pmat extract --list` uses tree-sitter to parse a single file
//! and dump function/struct/enum/trait boundaries as JSON — no index required.
//!
//! # Usage
//! ```bash
//! cargo run --example extract_demo
//! ```

use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn main() {
    println!("🔍 PMAT Extract Demo");
    println!("====================\n");

    println!("1. Extract from Rust file");
    test_extract_rust();

    println!("\n2. Extract from Python file");
    test_extract_python();

    println!("\n3. Extract from TypeScript file");
    test_extract_typescript();

    println!("\n4. Pipe extract output to jq for analysis");
    test_extract_pipeline();

    println!("\n✅ Extract Demo Completed!");
}

fn find_pmat_binary() -> String {
    // Try cargo-built binary first
    let cargo_bin = std::env::current_dir()
        .ok()
        .map(|p| {
            p.join("target")
                .join("release")
                .join("pmat")
                .display()
                .to_string()
        })
        .unwrap_or_default();
    if std::path::Path::new(&cargo_bin).exists() {
        return cargo_bin;
    }
    // Fall back to PATH
    "pmat".to_string()
}

fn test_extract_rust() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let rust_file = temp_dir.path().join("example.rs");

    fs::write(
        &rust_file,
        r#"
use std::collections::HashMap;

/// A cache with TTL-based eviction
pub struct Cache<K, V> {
    entries: HashMap<K, (V, std::time::Instant)>,
    ttl: std::time::Duration,
}

impl<K: std::hash::Hash + Eq, V> Cache<K, V> {
    pub fn new(ttl: std::time::Duration) -> Self {
        Self {
            entries: HashMap::new(),
            ttl,
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.entries.get(key).and_then(|(v, t)| {
            if t.elapsed() < self.ttl { Some(v) } else { None }
        })
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.entries.insert(key, (value, std::time::Instant::now()));
    }
}

pub enum CachePolicy {
    Lru,
    Ttl,
    None,
}

pub trait Evictable {
    fn should_evict(&self) -> bool;
}
"#,
    )
    .expect("Failed to write Rust file");

    let pmat = find_pmat_binary();
    let output = Command::new(&pmat)
        .args(["extract", "--list", rust_file.to_str().unwrap()])
        .output()
        .expect("Failed to run pmat extract");

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("   Output:\n{stdout}");

    // Parse and validate
    let items: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap_or_default();
    println!("   Found {} items", items.len());
    assert!(items.len() >= 5, "Expected at least 5 items (struct, impl, 3 fns, enum, trait)");
}

fn test_extract_python() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let py_file = temp_dir.path().join("server.py");

    fs::write(
        &py_file,
        r#"
import asyncio
from dataclasses import dataclass

@dataclass
class Config:
    host: str = "0.0.0.0"
    port: int = 8080

class Server:
    def __init__(self, config: Config):
        self.config = config
        self._running = False

    async def start(self):
        self._running = True
        print(f"Listening on {self.config.host}:{self.config.port}")

    async def stop(self):
        self._running = False

    def is_running(self) -> bool:
        return self._running

def create_server(host: str, port: int) -> Server:
    return Server(Config(host=host, port=port))
"#,
    )
    .expect("Failed to write Python file");

    let pmat = find_pmat_binary();
    let output = Command::new(&pmat)
        .args(["extract", "--list", py_file.to_str().unwrap()])
        .output()
        .expect("Failed to run pmat extract");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let items: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap_or_default();
    println!("   Found {} Python items", items.len());

    for item in &items {
        let name = item["name"].as_str().unwrap_or("?");
        let ty = item["type"].as_str().unwrap_or("?");
        let start = item["start_line"].as_u64().unwrap_or(0);
        let end = item["end_line"].as_u64().unwrap_or(0);
        println!("   {ty:>10} {name} (lines {start}-{end})");
    }
}

fn test_extract_typescript() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ts_file = temp_dir.path().join("api.ts");

    fs::write(
        &ts_file,
        r#"
interface ApiResponse<T> {
    data: T;
    status: number;
    error?: string;
}

class HttpClient {
    constructor(private baseUrl: string) {}

    async get<T>(path: string): Promise<ApiResponse<T>> {
        const resp = await fetch(`${this.baseUrl}${path}`);
        return resp.json();
    }

    async post<T>(path: string, body: unknown): Promise<ApiResponse<T>> {
        const resp = await fetch(`${this.baseUrl}${path}`, {
            method: "POST",
            body: JSON.stringify(body),
        });
        return resp.json();
    }
}

export function createClient(baseUrl: string): HttpClient {
    return new HttpClient(baseUrl);
}
"#,
    )
    .expect("Failed to write TypeScript file");

    let pmat = find_pmat_binary();
    let output = Command::new(&pmat)
        .args(["extract", "--list", ts_file.to_str().unwrap()])
        .output()
        .expect("Failed to run pmat extract");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let items: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap_or_default();
    println!("   Found {} TypeScript items", items.len());

    for item in &items {
        let name = item["name"].as_str().unwrap_or("?");
        let ty = item["type"].as_str().unwrap_or("?");
        let lines = item["lines"].as_u64().unwrap_or(0);
        println!("   {ty:>10} {name} ({lines} lines)");
    }
}

fn test_extract_pipeline() {
    // Demonstrate using extract on pmat's own source
    let pmat = find_pmat_binary();
    let target = "src/cli/handlers/extract_handler.rs";

    if !std::path::Path::new(target).exists() {
        println!("   Skipped (not in project root)");
        return;
    }

    let output = Command::new(&pmat)
        .args(["extract", "--list", target])
        .output()
        .expect("Failed to run pmat extract");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let items: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap_or_default();

    let functions: Vec<_> = items
        .iter()
        .filter(|i| i["type"].as_str() == Some("function"))
        .collect();
    let structs: Vec<_> = items
        .iter()
        .filter(|i| i["type"].as_str() == Some("struct"))
        .collect();

    println!("   extract_handler.rs: {} functions, {} structs", functions.len(), structs.len());
    println!("   Total lines covered: {}", items.iter()
        .map(|i| i["lines"].as_u64().unwrap_or(0))
        .sum::<u64>());
}
