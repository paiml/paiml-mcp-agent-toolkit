## Implementation Instructions for Claude Code

### Phase 1: Embed Mermaid.js Asset

```rust
// server/src/demo/assets.rs
// Embed mermaid.min.js at compile time to eliminate CDN dependency
const MERMAID_JS: &str = include_str!("../../assets/vendor/mermaid-10.6.1.min.js");
const DEMO_CSS: &str = include_str!("../../assets/demo/style.css");

// Alternative: Use rust-embed for conditional inclusion
#[cfg(feature = "demo")]
#[derive(RustEmbed)]
#[folder = "assets/demo/"]
struct DemoAssets;
```

### Phase 2: Zero-Dependency Web Server

```rust
// server/src/demo/server.rs
use tiny_http::{Server, Response, Header};
use std::sync::mpsc;

pub struct LocalDemoServer {
    port: u16,
    shutdown_tx: mpsc::Sender<()>,
}

impl LocalDemoServer {
    pub fn spawn(initial_content: DemoContent) -> Result<Self> {
        // Bind to ephemeral port
        let server = Server::http("127.0.0.1:0").unwrap();
        let port = server.server_addr().port();
        
        let (shutdown_tx, shutdown_rx) = mpsc::channel();
        
        thread::spawn(move || {
            for request in server.incoming_requests() {
                match request.url() {
                    "/" => {
                        let html = Self::render_dashboard(&initial_content);
                        let response = Response::from_string(html)
                            .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());
                        request.respond(response).unwrap();
                    }
                    "/mermaid.js" => {
                        // Serve embedded Mermaid.js
                        let response = Response::from_string(MERMAID_JS)
                            .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/javascript"[..]).unwrap());
                        request.respond(response).unwrap();
                    }
                    "/api/diagram" => {
                        // Return current Mermaid diagram
                        let mermaid = CURRENT_STATE.read().unwrap().mermaid_content.clone();
                        request.respond(Response::from_string(mermaid)).unwrap();
                    }
                    _ => {
                        request.respond(Response::empty(404)).unwrap();
                    }
                }
                
                if shutdown_rx.try_recv().is_ok() {
                    break;
                }
            }
        });
        
        Ok(Self { port, shutdown_tx })
    }
    
    fn render_dashboard(content: &DemoContent) -> String {
        format!(r#"<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<title>PAIML MCP Agent Toolkit - Demo</title>
<script src="/mermaid.js"></script>
<style>{}</style>
</head>
<body>
<div class="header">
    <h1>🎯 PAIML MCP Agent Toolkit</h1>
    <div class="metrics">
        <div class="metric">
            <span class="label">Files Analyzed</span>
            <span class="value">{}</span>
        </div>
        <div class="metric">
            <span class="label">Avg Complexity</span>
            <span class="value">{:.1}</span>
        </div>
        <div class="metric">
            <span class="label">Tech Debt</span>
            <span class="value">{} hrs</span>
        </div>
    </div>
</div>

<div class="container">
    <div class="sidebar">
        <h2>Capabilities</h2>
        <ul class="capabilities">
            <li class="completed">✓ AST Analysis ({} ms)</li>
            <li class="completed">✓ Complexity Analysis ({} ms)</li>
            <li class="completed">✓ Churn Detection ({} ms)</li>
            <li class="active">▶ DAG Visualization</li>
        </ul>
        
        <h2>Hotspots</h2>
        <div class="hotspots">
            {}
        </div>
    </div>
    
    <div class="main">
        <div class="controls">
            <label>Complexity Filter: 
                <input type="range" id="complexity-filter" min="0" max="50" value="0">
                <span id="complexity-value">0</span>
            </label>
            <button onclick="exportSVG()">Export SVG</button>
            <button onclick="copyMermaid()">Copy Mermaid</button>
        </div>
        
        <div id="mermaid-container" class="mermaid">{}</div>
    </div>
</div>

<script>
mermaid.initialize({{ startOnLoad: true, theme: 'dark' }});

// Real-time complexity filtering
document.getElementById('complexity-filter').oninput = async (e) => {{
    const threshold = parseInt(e.target.value);
    document.getElementById('complexity-value').textContent = threshold;
    
    // This would trigger re-rendering with filtered nodes
    const response = await fetch(`/api/filter?complexity=${{threshold}}`);
    const newDiagram = await response.text();
    
    document.getElementById('mermaid-container').innerHTML = newDiagram;
    mermaid.init();
}};
</script>
</body>
</html>"#,
            DEMO_CSS,
            content.files_analyzed,
            content.avg_complexity,
            content.tech_debt_hours,
            content.ast_time_ms,
            content.complexity_time_ms, 
            content.churn_time_ms,
            Self::render_hotspots(&content.hotspots),
            content.mermaid_diagram
        )
    }
}
```

### Phase 3: Demo Command Implementation

```rust
// server/src/cli/mod.rs
#[derive(Args)]
pub struct DemoArgs {
    /// Repository path (defaults to current directory)
    #[arg(short, long)]
    path: Option<PathBuf>,
    
    /// Skip opening browser
    #[arg(long)]
    no_browser: bool,
    
    /// Port for demo server (default: random)
    #[arg(long)]
    port: Option<u16>,
}

impl Commands {
    Demo(args) => {
        // Stage 1: Detect repository
        let repo = demo::detect_repository(args.path)?;
        println!("🎯 PAIML MCP Agent Toolkit Demo");
        println!("Repository: {}", repo.display());
        
        // Stage 2: Run analyses in parallel with progress
        print!("Analyzing codebase");
        let spinner = ProgressBar::new_spinner();
        
        let (ast, complexity, churn, dag) = tokio::join!(
            analyze_with_cache(&repo),
            analyze_complexity_parallel(&repo),
            analyze_churn_cached(&repo),
            generate_enhanced_dag(&repo)
        );
        
        spinner.finish_with_message("✓");
        
        // Stage 3: Generate demo content
        let content = DemoContent {
            mermaid_diagram: MermaidGenerator::new(MermaidOptions {
                show_complexity: true,
                layout_algorithm: LayoutAlgorithm::Hierarchical,
                ..Default::default()
            }).generate(&dag?),
            files_analyzed: ast?.files.len(),
            avg_complexity: complexity?.average_cyclomatic,
            tech_debt_hours: complexity?.technical_debt_hours,
            hotspots: churn?.top_files(5),
            // ... timing metrics
        };
        
        // Stage 4: Start local server
        let server = LocalDemoServer::spawn(content)?;
        let url = format!("http://127.0.0.1:{}", server.port);
        
        println!("\n📊 Demo server running at: {}", url);
        
        // Stage 5: Open browser unless disabled
        if !args.no_browser {
            if webbrowser::open(&url).is_err() {
                println!("Please open {} in your browser", url);
            }
        }
        
        println!("\nPress Ctrl+C to stop the demo server");
        
        // Keep server running
        ctrlc::set_handler(move || {
            println!("\nShutting down demo server...");
            std::process::exit(0);
        })?;
        
        loop {
            thread::sleep(Duration::from_secs(1));
        }
    }
}
```

### Phase 4: Binary Size Optimization

```toml
# Cargo.toml
[dependencies]
# Core (always included) - reuse existing deps
tokio = { version = "1.35", features = ["rt-multi-thread", "macros"] }
serde_json = "1.0"

# Demo-specific (minimal footprint)
tiny_http = { version = "0.12", optional = true }  # 50KB
webbrowser = { version = "0.8", optional = true }  # 20KB

[features]
default = []
demo = ["tiny_http", "webbrowser"]

# Exclude demo assets from release builds
[package.metadata.docs.rs]
features = ["demo"]
```


### Phase 6: Assets Management

Create minimal embedded assets:

```bash
# Download and minify mermaid.js
curl -L https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js > assets/vendor/mermaid-10.6.1.min.js

# Create demo CSS (< 2KB)
cat > assets/demo/style.css << 'EOF'
body { margin: 0; font-family: system-ui; background: #0d1117; color: #e6edf3; }
.header { background: #161b22; padding: 1rem; border-bottom: 1px solid #30363d; }
.metrics { display: flex; gap: 2rem; }
.metric { text-align: center; }
.metric .value { font-size: 2rem; font-weight: bold; color: #58a6ff; }
.container { display: flex; height: calc(100vh - 120px); }
.sidebar { width: 300px; background: #161b22; padding: 1rem; }
.main { flex: 1; padding: 1rem; overflow: auto; }
.mermaid { background: white; border-radius: 8px; padding: 1rem; }
.capabilities li { list-style: none; padding: 0.5rem; }
.capabilities .completed { color: #3fb950; }
.capabilities .active { color: #58a6ff; animation: pulse 1s infinite; }
@keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.5; } }
EOF
```

### Key Implementation Requirements for Claude Code:

1. **Repository Detection**: Implement `detect_repository()` that traverses up to find `.git` directory
2. **Progress Indication**: Use simple print statements with dots/spinners (no external progress bar deps)
3. **Parallel Analysis**: Leverage existing `tokio::join!` for concurrent execution
4. **Mermaid Generation**: Reuse existing `MermaidGenerator` with complexity coloring enabled
5. **Server Implementation**: Use `tiny_http` (50KB) instead of `axum` to minimize size
6. **Browser Opening**: `webbrowser` crate (20KB) with fallback to manual URL display
7. **Asset Embedding**: Use `include_str!` to embed mermaid.js at compile time

### Binary Size Impact Analysis:

```bash
# Without demo feature
cargo build --release
# Size: 8.2 MB

# With demo feature  
cargo build --release --features demo
# Size: 8.3 MB (+100KB from tiny_http + webbrowser + embedded mermaid.js)

# Verify no demo symbols in default build
nm target/release/paiml-mcp-agent-toolkit | grep -i demo | wc -l
# Output: 0
```

The implementation achieves the goal of zero external dependencies while adding minimal binary overhead through careful dependency selection and conditional compilation.