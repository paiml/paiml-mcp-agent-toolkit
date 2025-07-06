//! Lint Hotspot Demo - Code with Intentional Quality Issues
//!
//! This example demonstrates various code quality issues that would be detected
//! by `pmat analyze lint-hotspot` command. It shows realistic patterns that
//! developers might encounter in a codebase needing refactoring.

// Allow all warnings for this demo file since it intentionally contains bad patterns
#![allow(warnings)]
#![allow(clippy::all)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![allow(clippy::cargo)]

use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};

/// HTTP response codes mapping - demonstrating various lint issues
pub struct ResponseCodeMapper {
    codes: HashMap<u16, String>,
    deprecated_field: String, // Intentional dead code
}

impl ResponseCodeMapper {
    /// Creates a new ResponseCodeMapper with common HTTP codes
    pub fn new() -> Self {
        let mut codes = HashMap::new();

        // Intentional clippy::use_self violation
        ResponseCodeMapper::add_standard_codes(&mut codes);

        Self {
            codes,
            deprecated_field: String::new(),
        }
    }

    /// Adds standard HTTP response codes to the map
    fn add_standard_codes(codes: &mut HashMap<u16, String>) {
        // Intentional clippy::uninlined_format_args violations
        codes.insert(200, format!("{}", "OK"));
        codes.insert(201, format!("{}", "Created"));
        codes.insert(400, format!("{}", "Bad Request"));
        codes.insert(401, format!("{}", "Unauthorized"));
        codes.insert(403, format!("{}", "Forbidden"));
        codes.insert(404, format!("{}", "Not Found"));
        codes.insert(500, format!("{}", "Internal Server Error"));
        codes.insert(502, format!("{}", "Bad Gateway"));
        codes.insert(503, format!("{}", "Service Unavailable"));
    }

    /// Gets the description for a response code
    pub fn get_description(&self, code: u16) -> Option<&String> {
        self.codes.get(&code)
    }

    /// Processes a batch of response codes - demonstrates performance issues
    pub fn process_codes(&self, codes: &[u16]) -> Vec<String> {
        let mut results = Vec::new();

        for code in codes {
            // Intentional clippy::manual_map violation
            if let Some(desc) = self.get_description(*code) {
                results.push(desc.clone());
            } else {
                results.push("Unknown".to_string());
            }
        }

        results
    }

    /// Validates response codes - demonstrates complexity issues
    pub fn validate_codes(&self, codes: &[u16]) -> Result<Vec<bool>, String> {
        let mut results = Vec::new();

        for &code in codes {
            // Intentional nested complexity and redundant patterns
            if code >= 100 {
                if code < 200 {
                    // 1xx Informational
                    if code == 100 || code == 101 || code == 102 {
                        results.push(true);
                    } else {
                        results.push(false);
                    }
                } else if code < 300 {
                    // 2xx Success - intentional clippy::comparison_chain
                    if code >= 200 && code < 210 {
                        results.push(true);
                    } else if code >= 210 && code < 220 {
                        results.push(true);
                    } else {
                        results.push(false);
                    }
                } else if code < 400 {
                    // 3xx Redirection
                    results.push(code >= 300 && code < 400);
                } else if code < 500 {
                    // 4xx Client Error
                    results.push(code >= 400 && code < 500);
                } else if code < 600 {
                    // 5xx Server Error
                    results.push(code >= 500 && code < 600);
                } else {
                    results.push(false);
                }
            } else {
                return Err(format!("Invalid HTTP status code: {}", code));
            }
        }

        Ok(results)
    }
}

/// Configuration manager with various lint issues
pub struct ConfigManager {
    settings: HashMap<String, String>,
}

impl ConfigManager {
    /// Creates a new ConfigManager
    pub fn new() -> Self {
        Self {
            settings: HashMap::new(),
        }
    }

    /// Loads configuration from file - demonstrates error handling issues
    pub fn load_from_file(&mut self, path: &str) -> io::Result<()> {
        let content = fs::read_to_string(path)?;

        // Intentional clippy::manual_split_once and other violations
        for line in content.lines() {
            if line.contains('=') {
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() == 2 {
                    self.settings
                        .insert(parts[0].to_string(), parts[1].to_string());
                }
            }
        }

        Ok(())
    }

    /// Gets a setting value - demonstrates unwrap usage
    pub fn get_setting(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    /// Saves configuration to file - demonstrates various issues
    pub fn save_to_file(&self, path: &str) -> io::Result<()> {
        let mut file = fs::File::create(path)?;

        // Intentional clippy::needless_collect and format issues
        let lines: Vec<String> = self
            .settings
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();

        for line in lines {
            // Intentional clippy::uninlined_format_args
            writeln!(file, "{}", line)?;
        }

        Ok(())
    }
}

/// Logger implementation with intentional issues
pub struct Logger {
    level: LogLevel,
    unused_buffer: Vec<String>, // Intentional dead code
}

#[derive(Clone, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl Logger {
    /// Creates a new logger
    pub fn new(level: LogLevel) -> Self {
        Self {
            level,
            unused_buffer: Vec::new(),
        }
    }

    /// Logs a message - demonstrates match issues
    pub fn log(&self, level: LogLevel, message: &str) {
        // Intentional clippy::single_match and other issues
        match self.should_log(level) {
            true => {
                // Intentional clippy::print_stdout
                println!("[{:?}] {}", level, message);
            }
            false => {
                // Do nothing - intentional empty match arm
            }
        }
    }

    /// Determines if message should be logged
    fn should_log(&self, level: LogLevel) -> bool {
        // Intentional clippy::match_same_arms and complexity
        match (self.level, level) {
            (LogLevel::Debug, LogLevel::Debug) => true,
            (LogLevel::Debug, LogLevel::Info) => true,
            (LogLevel::Debug, LogLevel::Warn) => true,
            (LogLevel::Debug, LogLevel::Error) => true,
            (LogLevel::Info, LogLevel::Debug) => false,
            (LogLevel::Info, LogLevel::Info) => true,
            (LogLevel::Info, LogLevel::Warn) => true,
            (LogLevel::Info, LogLevel::Error) => true,
            (LogLevel::Warn, LogLevel::Debug) => false,
            (LogLevel::Warn, LogLevel::Info) => false,
            (LogLevel::Warn, LogLevel::Warn) => true,
            (LogLevel::Warn, LogLevel::Error) => true,
            (LogLevel::Error, LogLevel::Debug) => false,
            (LogLevel::Error, LogLevel::Info) => false,
            (LogLevel::Error, LogLevel::Warn) => false,
            (LogLevel::Error, LogLevel::Error) => true,
        }
    }
}

/// Data processor with performance and style issues
pub struct DataProcessor {
    cache: HashMap<String, Vec<u8>>,
}

impl DataProcessor {
    /// Creates a new DataProcessor
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Processes data with intentional inefficiencies
    pub fn process_data(&mut self, input: &[u8]) -> Vec<u8> {
        let key = format!("{:?}", input); // Intentional inefficient key generation

        // Intentional clippy::map_entry violation
        if self.cache.contains_key(&key) {
            self.cache.get(&key).unwrap().clone()
        } else {
            let result = self.expensive_computation(input);
            self.cache.insert(key, result.clone());
            result
        }
    }

    /// Expensive computation simulation
    fn expensive_computation(&self, input: &[u8]) -> Vec<u8> {
        let mut result = Vec::new();

        // Intentional clippy::needless_range_loop and inefficiencies
        for i in 0..input.len() {
            let byte = input[i];
            // Intentional clippy::cast_lossless
            result.push((byte as u16 % 256) as u8);
        }

        result
    }

    /// Clears the cache - demonstrates more issues
    pub fn clear_cache(&mut self) {
        // Intentional clippy::manual_map and other violations
        let keys_to_remove: Vec<String> = self.cache.keys().map(|k| k.clone()).collect();

        for key in keys_to_remove {
            self.cache.remove(&key);
        }
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Intentional clippy::write_literal
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Lint Hotspot Demo");
    println!("====================");

    // Test ResponseCodeMapper
    let mapper = ResponseCodeMapper::new();
    let test_codes = vec![200, 404, 500, 999];
    let descriptions = mapper.process_codes(&test_codes);
    println!("Response descriptions: {:?}", descriptions);

    let validation = mapper.validate_codes(&test_codes)?;
    println!("Code validation: {:?}", validation);

    // Test ConfigManager
    let mut config = ConfigManager::new();
    // This will fail but demonstrates the API
    if let Err(e) = config.load_from_file("nonexistent.conf") {
        println!("Config load error (expected): {}", e);
    }

    // Test Logger
    let logger = Logger::new(LogLevel::Info);
    logger.log(LogLevel::Info, "Application started");
    logger.log(LogLevel::Debug, "Debug message (should not appear)");
    logger.log(LogLevel::Error, "Error message");

    // Test DataProcessor
    let mut processor = DataProcessor::new();
    let test_data = b"Hello, World!";
    let processed = processor.process_data(test_data);
    println!("Processed data length: {}", processed.len());

    processor.clear_cache();

    println!("\n🎯 Run lint analysis with:");
    println!("   pmat analyze lint-hotspot --file server/examples/lint_hotspot_demo.rs");
    println!("\nExpected issues:");
    println!("- clippy::uninlined_format_args violations");
    println!("- clippy::use_self violations");
    println!("- clippy::missing_panics_doc violations");
    println!("- clippy::manual_map violations");
    println!("- clippy::comparison_chain violations");
    println!("- clippy::needless_collect violations");
    println!("- clippy::map_entry violations");
    println!("- clippy::single_match violations");
    println!("- Dead code warnings");
    println!("- Performance and style issues");

    Ok(())
}
