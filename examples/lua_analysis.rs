//! Lua Language Analysis Example
//!
//! Demonstrates pmat's Lua analysis capabilities using tree-sitter-lua.
//!
//! Features demonstrated:
//! 1. Parsing Lua source files into unified AST
//! 2. Extracting functions, imports (require), and types (tables)
//! 3. Calculating cyclomatic and cognitive complexity
//! 4. Analyzing control flow patterns (if/for/while/repeat/and/or)
//! 5. TDG (Technical Debt Grading) with full 7-component scoring
//! 6. Dead code detection with module export awareness
//! 7. Defect detection (LUA-GLOBAL, LUA-NIL, LUA-PCALL, LUA-DANGER)
//!
//! # Usage
//!
//! ```bash
//! cargo run --example lua_analysis
//! ```

use anyhow::Result;
use pmat::ast::core::Language;
use pmat::ast::languages::LanguageStrategy;
use pmat::tdg::TdgAnalyzer;

fn main() -> Result<()> {
    println!("=== PMAT Lua Language Analysis ===\n");

    let strategy = pmat::ast::languages::lua::LuaStrategy::new();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    demo_function_parsing(&strategy, &rt)?;
    demo_imports(&strategy, &rt)?;
    demo_control_flow(&strategy, &rt)?;
    demo_oop(&strategy, &rt)?;
    demo_language_detection(&strategy);
    demo_tdg_scoring()?;
    demo_dead_code_detection()?;
    demo_defect_detection();

    println!("=== Lua Analysis Complete ===");
    println!();
    println!("Try analyzing your own Lua projects:");
    println!("  pmat analyze tdg --path /path/to/file.lua");
    println!("  pmat analyze complexity --path /path/to/lua/project");
    println!("  pmat analyze dead-code --path /path/to/lua/project");
    println!("  pmat comply check   # includes CB-600 Lua best practices");
    println!("  pmat query \"function_name\" --include-source");

    Ok(())
}

fn demo_function_parsing(
    strategy: &pmat::ast::languages::lua::LuaStrategy,
    rt: &tokio::runtime::Runtime,
) -> Result<()> {
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
    let ast = rt.block_on(strategy.parse_file(std::path::Path::new("demo.lua"), code))?;
    let functions = strategy.extract_functions(&ast);
    let (cyclomatic, cognitive) = strategy.calculate_complexity(&ast);
    println!("   Functions found: {}", functions.len());
    println!(
        "   Complexity: cyclomatic={}, cognitive={}",
        cyclomatic, cognitive
    );
    println!();
    Ok(())
}

fn demo_imports(
    strategy: &pmat::ast::languages::lua::LuaStrategy,
    rt: &tokio::runtime::Runtime,
) -> Result<()> {
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
    Ok(())
}

fn demo_control_flow(
    strategy: &pmat::ast::languages::lua::LuaStrategy,
    rt: &tokio::runtime::Runtime,
) -> Result<()> {
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
    let grade = match cyclomatic {
        0..=10 => "A (simple)",
        11..=20 => "B (moderate)",
        _ => "C (complex)",
    };
    println!("   Grade: {grade}");
    println!();
    Ok(())
}

fn demo_oop(
    strategy: &pmat::ast::languages::lua::LuaStrategy,
    rt: &tokio::runtime::Runtime,
) -> Result<()> {
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
    Ok(())
}

fn demo_language_detection(strategy: &pmat::ast::languages::lua::LuaStrategy) {
    println!("5. Language Detection");
    assert_eq!(strategy.language(), Language::Lua);
    assert!(strategy.can_parse(std::path::Path::new("init.lua")));
    assert!(strategy.can_parse(std::path::Path::new("scripts/game.lua")));
    assert!(!strategy.can_parse(std::path::Path::new("main.py")));
    println!("   Language: {:?}", strategy.language());
    println!("   Parses .lua files: true");
    println!("   Parses .py files: false");
    println!();
}

fn demo_tdg_scoring() -> Result<()> {
    println!("6. TDG Quality Scoring");
    let tdg_code = r#"
--- Game entity manager module
-- Handles creation, update, and querying of game entities.

local json = require("dkjson")

local M = {}

local entities = {}

--- Create a new entity with the given components
function M.create_entity(name, x, y, health)
    local entity = {
        name = name,
        x = x or 0,
        y = y or 0,
        health = health or 100,
        alive = true,
    }
    entities[#entities + 1] = entity
    return entity
end

--- Update all entities each frame
function M.update(dt)
    for i, entity in ipairs(entities) do
        if entity.alive then
            if entity.health <= 0 then
                entity.alive = false
            else
                entity.x = entity.x + dt
            end
        end
    end
end

--- Find entity by name
function M.find(name)
    for _, entity in ipairs(entities) do
        if entity.name == name then
            return entity
        end
    end
    return nil
end

return M
"#;
    let analyzer = TdgAnalyzer::new()?;
    let score = analyzer.analyze_source(tdg_code, pmat::tdg::Language::Lua, None)?;
    println!("   TDG Total:  {:.1}/100", score.total);
    println!("   Grade:      {:?}", score.grade);
    println!("   Confidence: {:.0}%", score.confidence * 100.0);
    println!("   Components:");
    println!("     Structural Complexity: {:.1}/25", score.structural_complexity);
    println!("     Semantic Complexity:   {:.1}/20", score.semantic_complexity);
    println!("     Duplication Ratio:     {:.1}/20", score.duplication_ratio);
    println!("     Coupling Score:        {:.1}/15", score.coupling_score);
    println!("     Doc Coverage:          {:.1}/10", score.doc_coverage);
    println!("     Consistency Score:     {:.1}/10", score.consistency_score);
    if !score.penalties_applied.is_empty() {
        println!("   Penalties:");
        for penalty in &score.penalties_applied {
            println!("     - {}: -{:.1}", penalty.issue, penalty.amount);
        }
    }
    println!();
    Ok(())
}

fn demo_dead_code_detection() -> Result<()> {
    println!("7. Dead Code Detection (Module-Export Aware)");

    // Create a temp directory with Lua files
    let temp = tempfile::TempDir::new()?;
    std::fs::write(
        temp.path().join("mylib.lua"),
        r#"local M = {}

function M.public_api()
    return M.helper()
end

function M.helper()
    return 42
end

local function truly_dead()
    return "nobody calls me"
end

return M
"#,
    )?;

    let result =
        pmat::services::dead_code_multi_language::analyze_dead_code_multi_language(temp.path())?;

    println!("   Language: {}", result.language);
    println!("   Total functions: {}", result.total_functions);
    println!("   Dead functions: {}", result.dead_functions.len());
    for dead_fn in &result.dead_functions {
        println!("     - {} (line {}): {}", dead_fn.name, dead_fn.line, dead_fn.reason);
    }
    println!(
        "   Dead code: {:.1}%",
        result.dead_code_percentage
    );
    println!(
        "   Module exports (M.*): correctly excluded from dead code"
    );
    println!();
    Ok(())
}

fn demo_defect_detection() {
    println!("8. Defect Detection (LuaDefectDetector)");
    use pmat::services::defect_detector::LuaDefectDetector;

    let detector = LuaDefectDetector::new();
    let code = r#"count = 0
result = compute(data)
get_player():set_health(100)
pcall(dangerous_function)
os.execute("rm -rf " .. user_input)
"#;
    let defects = detector.detect(code, std::path::Path::new("demo.lua"));

    println!("   Defects found: {}", defects.len());
    for defect in &defects {
        println!(
            "     [{:?}] {} ({} instance{})",
            defect.severity,
            defect.name,
            defect.instances.len(),
            if defect.instances.len() == 1 { "" } else { "s" }
        );
    }
    println!();
}
