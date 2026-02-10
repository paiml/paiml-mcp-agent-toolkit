//! Lua Language Analysis Example
//!
//! Demonstrates pmat's Lua AST parsing capabilities using tree-sitter-lua.
//!
//! Features demonstrated:
//! 1. Parsing Lua source files into unified AST
//! 2. Extracting functions, imports (require), and types (tables)
//! 3. Calculating cyclomatic and cognitive complexity
//! 4. Analyzing control flow patterns (if/for/while/repeat/and/or)
//!
//! # Usage
//!
//! ```bash
//! cargo run --example lua_analysis
//! ```

use anyhow::Result;
use pmat::ast::core::Language;
use pmat::ast::languages::LanguageStrategy;

fn main() -> Result<()> {
    println!("=== PMAT Lua Language Analysis ===\n");

    let strategy = pmat::ast::languages::lua::LuaStrategy::new();

    // 1. Simple function parsing
    println!("1. Simple Function Parsing");
    let code = r#"
function greet(name)
    print("Hello, " .. name)
end

local function factorial(n)
    if n <= 1 then
        return 1
    end
    return n * factorial(n - 1)
end
"#;
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let ast = rt.block_on(strategy.parse_file(std::path::Path::new("demo.lua"), code))?;
    let functions = strategy.extract_functions(&ast);
    let (cyclomatic, cognitive) = strategy.calculate_complexity(&ast);
    println!("   Functions found: {}", functions.len());
    println!(
        "   Complexity: cyclomatic={}, cognitive={}",
        cyclomatic, cognitive
    );
    println!();

    // 2. Module with require() imports
    println!("2. Module Imports (require)");
    let code = r#"
local json = require("dkjson")
local socket = require("socket")
local lfs = require("lfs")

local M = {}

function M.encode(data)
    return json.encode(data)
end

function M.decode(str)
    return json.decode(str)
end

return M
"#;
    let ast = rt.block_on(strategy.parse_file(std::path::Path::new("module.lua"), code))?;
    let imports = strategy.extract_imports(&ast);
    let functions = strategy.extract_functions(&ast);
    println!("   Imports detected: {}", imports.len());
    println!("   Functions defined: {}", functions.len());
    println!();

    // 3. Complex control flow
    println!("3. Complex Control Flow Analysis");
    let code = r#"
function process_events(events)
    local results = {}

    for i, event in ipairs(events) do
        if event.type == "click" then
            if event.target and event.x and event.y then
                table.insert(results, handle_click(event))
            end
        elseif event.type == "key" then
            if event.key == "escape" or event.key == "q" then
                return results
            end
        elseif event.type == "resize" then
            while #results > 0 and results[#results].stale do
                table.remove(results)
            end
        end
    end

    repeat
        local pending = check_pending()
        if pending then
            table.insert(results, pending)
        end
    until not pending

    return results
end
"#;
    let ast = rt.block_on(strategy.parse_file(std::path::Path::new("complex.lua"), code))?;
    let (cyclomatic, cognitive) = strategy.calculate_complexity(&ast);
    println!(
        "   Complexity: cyclomatic={}, cognitive={}",
        cyclomatic, cognitive
    );
    println!(
        "   Grade: {}",
        if cyclomatic <= 10 {
            "A (simple)"
        } else if cyclomatic <= 20 {
            "B (moderate)"
        } else {
            "C (complex)"
        }
    );
    println!();

    // 4. Table constructors (OOP pattern)
    println!("4. Table Constructors (Lua OOP)");
    let code = r#"
local Player = {}
Player.__index = Player

function Player.new(name, health)
    local self = setmetatable({}, Player)
    self.name = name
    self.health = health or 100
    self.inventory = {}
    return self
end

function Player:take_damage(amount)
    self.health = self.health - amount
    if self.health <= 0 then
        self.health = 0
        return false
    end
    return true
end

local config = {
    width = 1920,
    height = 1080,
    fullscreen = true,
    audio = { volume = 0.8, muted = false },
}
"#;
    let ast = rt.block_on(strategy.parse_file(std::path::Path::new("oop.lua"), code))?;
    let types = strategy.extract_types(&ast);
    let functions = strategy.extract_functions(&ast);
    println!("   Table constructors: {}", types.len());
    println!("   Methods/functions: {}", functions.len());
    println!();

    // 5. Language detection
    println!("5. Language Detection");
    assert_eq!(strategy.language(), Language::Lua);
    assert!(strategy.can_parse(std::path::Path::new("init.lua")));
    assert!(strategy.can_parse(std::path::Path::new("scripts/game.lua")));
    assert!(!strategy.can_parse(std::path::Path::new("main.py")));
    println!("   Language: {:?}", strategy.language());
    println!("   Parses .lua files: true");
    println!("   Parses .py files: false");
    println!();

    println!("=== Lua Analysis Complete ===");
    println!();
    println!("Try analyzing your own Lua projects:");
    println!("  pmat context --project-path /path/to/lua/project");
    println!("  pmat analyze complexity /path/to/lua/project");
    println!("  pmat query \"function_name\" --include-source");

    Ok(())
}
