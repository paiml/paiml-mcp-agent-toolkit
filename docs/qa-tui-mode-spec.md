```markdown
# TUI Mode Quality Assurance Specification

## Abstract

This specification defines the implementation of a fully-functioning Terminal User Interface (TUI) mode for the PAIML MCP Agent Toolkit, integrating ratatui as the fourth protocol variant in the unified demo architecture. The TUI provides real-time, interactive analysis with sub-100ms update latency and memory-mapped state management.

## Architecture Integration

### 1. Protocol Enum Extension

```rust
// server/src/cli/mod.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Protocol {
    #[value(name = "cli")]
    Cli,
    #[value(name = "http")]
    Http,
    #[value(name = "mcp")]
    Mcp,
    #[value(name = "tui")]
    Tui,  // NEW: Terminal UI mode
}
```

### 2. TUI Adapter Implementation

```rust
// server/src/demo/adapters/tui.rs
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    widgets::{Block, Borders, List, ListItem, Gauge},
    layout::{Layout, Constraint, Direction},
    style::{Color, Modifier, Style},
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

pub struct TuiDemoAdapter {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    state: Arc<RwLock<TuiState>>,
    update_rx: mpsc::Receiver<AnalysisUpdate>,
}

#[derive(Default)]
struct TuiState {
    selected_panel: PanelId,
    analysis_results: AnalysisResults,
    dag_viewport: ViewPort,
    scroll_offset: usize,
    filter: String,
}

impl DemoProtocol for TuiDemoAdapter {
    async fn initialize(&mut self) -> Result<(), DemoError> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        
        self.terminal.clear()?;
        Ok(())
    }
    
    async fn handle_request(&mut self, request: DemoRequest) -> Result<DemoResponse, DemoError> {
        match request {
            DemoRequest::Analyze { path, options } => {
                // Spawn analysis in background, stream updates to UI
                let (tx, rx) = mpsc::channel(64);
                self.spawn_analysis_pipeline(path, options, tx).await?;
                self.update_rx = rx;
                Ok(DemoResponse::Streaming)
            }
            _ => self.handle_interactive_command(request).await,
        }
    }
}
```

### 3. Event Loop Architecture

```rust
impl TuiDemoAdapter {
    pub async fn run_event_loop(&mut self) -> Result<(), DemoError> {
        let tick_rate = Duration::from_millis(50);  // 20 FPS
        let mut last_tick = Instant::now();
        
        loop {
            // Non-blocking event polling with 10ms timeout
            if event::poll(Duration::from_millis(10))? {
                match event::read()? {
                    Event::Key(key) => {
                        if self.handle_key_event(key).await? == ControlFlow::Exit {
                            break;
                        }
                    }
                    Event::Mouse(mouse) => self.handle_mouse_event(mouse).await?,
                    Event::Resize(width, height) => self.handle_resize(width, height)?,
                }
            }
            
            // Process analysis updates without blocking UI
            while let Ok(update) = self.update_rx.try_recv() {
                self.apply_analysis_update(update).await?;
            }
            
            // Render at consistent framerate
            if last_tick.elapsed() >= tick_rate {
                self.render_frame().await?;
                last_tick = Instant::now();
            }
        }
        
        self.cleanup()?;
        Ok(())
    }
    
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<ControlFlow, DemoError> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Ok(ControlFlow::Exit),
            KeyCode::Tab => {
                self.state.write().unwrap().cycle_panel();
                Ok(ControlFlow::Continue)
            }
            KeyCode::Char('/') => {
                self.enter_search_mode().await?;
                Ok(ControlFlow::Continue)
            }
            KeyCode::F(5) => {
                self.refresh_analysis().await?;
                Ok(ControlFlow::Continue)
            }
            _ => Ok(ControlFlow::Continue),
        }
    }
}
```

### 4. Rendering Pipeline

```rust
impl TuiDemoAdapter {
    async fn render_frame(&mut self) -> Result<(), DemoError> {
        let state = self.state.read().unwrap();
        
        self.terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),   // Header
                    Constraint::Min(10),     // Main content
                    Constraint::Length(3),   // Status bar
                ])
                .split(f.size());
            
            // Header with project info
            self.render_header(f, chunks[0], &state);
            
            // Main content area with panels
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(30),  // Left panel (file tree)
                    Constraint::Percentage(40),  // Center panel (analysis)
                    Constraint::Percentage(30),  // Right panel (DAG/metrics)
                ])
                .split(chunks[1]);
            
            self.render_file_tree(f, main_chunks[0], &state);
            self.render_analysis_panel(f, main_chunks[1], &state);
            self.render_dag_panel(f, main_chunks[2], &state);
            
            // Status bar with shortcuts
            self.render_status_bar(f, chunks[2], &state);
        })?;
        
        Ok(())
    }
    
    fn render_analysis_panel(&self, f: &mut Frame, area: Rect, state: &TuiState) {
        let items: Vec<ListItem> = state.analysis_results.hotspots
            .iter()
            .map(|h| {
                let style = match h.severity {
                    Severity::Critical => Style::default().fg(Color::Red),
                    Severity::Warning => Style::default().fg(Color::Yellow),
                    Severity::Info => Style::default().fg(Color::Green),
                };
                
                let content = format!("{}: {} ({})", 
                    h.file_path.display(), 
                    h.description,
                    h.metric_value
                );
                
                ListItem::new(content).style(style)
            })
            .collect();
        
        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Analysis Results"))
            .highlight_style(Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD));
        
        f.render_stateful_widget(list, area, &mut state.list_state);
    }
}
```

### 5. State Management

```rust
#[derive(Debug, Clone)]
struct AnalysisUpdate {
    update_type: UpdateType,
    data: AnalysisData,
    progress: f32,
}

#[derive(Debug, Clone)]
enum UpdateType {
    FileDiscovered(PathBuf),
    ComplexityComputed(FileComplexity),
    ChurnAnalyzed(FileChurn),
    DagNodeAdded(NodeInfo),
    AnalysisComplete,
}

impl TuiDemoAdapter {
    async fn apply_analysis_update(&mut self, update: AnalysisUpdate) -> Result<(), DemoError> {
        let mut state = self.state.write().unwrap();
        
        match update.update_type {
            UpdateType::FileDiscovered(path) => {
                state.file_tree.insert(path);
            }
            UpdateType::ComplexityComputed(complexity) => {
                state.analysis_results.complexity_map.insert(
                    complexity.file_path.clone(),
                    complexity
                );
                state.recalculate_hotspots();
            }
            UpdateType::DagNodeAdded(node) => {
                state.dag_nodes.insert(node.id.clone(), node);
                state.dag_needs_layout = true;
            }
            _ => {}
        }
        
        state.progress = update.progress;
        Ok(())
    }
}
```

### 6. Performance Optimizations

```rust
// Virtualized rendering for large file trees
struct VirtualizedTree {
    visible_range: Range<usize>,
    total_items: usize,
    item_height: u16,
    viewport_height: u16,
}

impl VirtualizedTree {
    fn update_viewport(&mut self, scroll_offset: usize) {
        let visible_count = (self.viewport_height / self.item_height) as usize;
        self.visible_range = scroll_offset..scroll_offset.saturating_add(visible_count);
    }
    
    fn render_items<'a>(&self, items: &'a [TreeItem]) -> impl Iterator<Item = ListItem<'a>> {
        items[self.visible_range.clone()]
            .iter()
            .map(|item| self.render_tree_item(item))
    }
}

// Incremental DAG layout with spatial indexing
struct DagLayoutEngine {
    spatial_index: RTree<NodeId>,
    layout_cache: HashMap<NodeId, Point2D>,
    dirty_nodes: HashSet<NodeId>,
}

impl DagLayoutEngine {
    fn incremental_layout(&mut self, new_nodes: &[NodeInfo]) -> LayoutResult {
        // Only recompute positions for affected subgraph
        let affected = self.compute_affected_region(new_nodes);
        
        // Use force-directed layout with Barnes-Hut optimization
        let positions = self.barnes_hut_layout(&affected, 0.001);
        
        // Update spatial index
        for (id, pos) in positions {
            self.spatial_index.insert(id, pos);
            self.layout_cache.insert(id, pos);
        }
        
        LayoutResult { 
            positions: self.layout_cache.clone(),
            computation_time: start.elapsed(),
        }
    }
}
```

### 7. Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    
    #[tokio::test]
    async fn test_tui_initialization() {
        let backend = TestBackend::new(80, 25);
        let terminal = Terminal::new(backend).unwrap();
        let adapter = TuiDemoAdapter::new(terminal);
        
        assert!(adapter.initialize().await.is_ok());
    }
    
    #[tokio::test]
    async fn test_render_performance() {
        let mut adapter = create_test_adapter();
        
        // Generate synthetic analysis data
        let analysis = generate_large_analysis_result(1000);
        adapter.state.write().unwrap().analysis_results = analysis;
        
        // Measure render time
        let start = Instant::now();
        for _ in 0..60 {  // 60 frames
            adapter.render_frame().await.unwrap();
        }
        let elapsed = start.elapsed();
        
        // Assert 60 FPS capability
        assert!(elapsed < Duration::from_secs(1));
    }
    
    #[tokio::test] 
    async fn test_event_handling_latency() {
        let mut adapter = create_test_adapter();
        let key_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::empty());
        
        let start = Instant::now();
        adapter.handle_key_event(key_event).await.unwrap();
        let latency = start.elapsed();
        
        // Sub-millisecond response time
        assert!(latency < Duration::from_millis(1));
    }
}
```

### 8. Integration Points

```rust
// server/src/demo/mod.rs
pub async fn run_demo(args: DemoArgs) -> Result<()> {
    match args.protocol {
        Protocol::Cli => run_cli_demo(args).await,
        Protocol::Http => run_http_demo(args).await,
        Protocol::Mcp => run_mcp_demo(args).await,
        Protocol::Tui => {
            // NEW: TUI mode entry point
            let mut adapter = TuiDemoAdapter::new()?;
            adapter.initialize().await?;
            
            if let Some(repo) = args.repo {
                adapter.analyze_repository(repo).await?;
            } else {
                adapter.analyze_current_directory().await?;
            }
            
            adapter.run_event_loop().await?;
            Ok(())
        }
    }
}
```

## Performance Requirements

- **Startup Time**: <50ms to first frame
- **Frame Rate**: Consistent 20 FPS (50ms frame budget)
- **Event Latency**: <1ms for keyboard input processing
- **Memory Usage**: <50MB for 10,000 file projects
- **Incremental Updates**: <10ms for single file analysis update

## Success Criteria

1. **Zero-flicker rendering** via double buffering
2. **Responsive during analysis** via async update streaming
3. **Keyboard-only navigation** with vim-style bindings
4. **Graceful degradation** on terminal resize
5. **Cross-platform support** (Windows Terminal, iTerm2, Linux TTY)

## Implementation Timeline

- **Week 1**: Protocol integration and basic event loop
- **Week 2**: Rendering pipeline and state management
- **Week 3**: Performance optimizations and virtualization
- **Week 4**: Testing and cross-platform validation
```