# Ratatui Interactive Mode Specification v2.0

## Executive Summary

This specification defines the fourth interface paradigm for PAIML MCP Agent Toolkit: an interactive TUI mode implemented as a `ProtocolAdapter` within the unified protocol architecture. The design leverages existing analysis engines, caching infrastructure, and SIMD-optimized operations while providing Excel-like defect grid visualization, real-time analysis updates, and integrated REPL capabilities.

## 1. Architecture Overview

### 1.1 Protocol Adapter Integration

```rust
pub struct TuiAdapter {
    terminal: Arc<Mutex<Terminal<CrosstermBackend<Stdout>>>>,
    event_rx: mpsc::Receiver<Event>,
    state: Arc<RwLock<TuiState>>,
    // Reuse existing infrastructure
    cache_manager: Arc<SessionCacheManager>,
    persistent_cache: Arc<PersistentCacheManager>,
}

impl ProtocolAdapter for TuiAdapter {
    type Output = TuiOutput;
    
    fn protocol(&self) -> Protocol {
        Protocol::Tui
    }
    
    async fn decode(&self, input: Self::Input) -> Result<UnifiedRequest, ProtocolError> {
        match input {
            TuiInput::KeyEvent(key) => self.decode_key_event(key),
            TuiInput::Command(cmd) => self.decode_repl_command(cmd),
            TuiInput::FileSystemEvent(event) => self.decode_fs_event(event),
        }
    }
    
    async fn encode(&self, response: UnifiedResponse) -> Result<Self::Output, ProtocolError> {
        // Transform UnifiedResponse into TUI render commands
        TuiOutput::RenderCommand(self.build_render_tree(response)?)
    }
}
```

### 1.2 State Machine with Cache Persistence

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiState {
    current_view: ViewState,
    analysis_cache: AnalysisCache,
    grid_state: DefectGridState,
    navigation_stack: VecDeque<ViewState>,
    repl_history: VecDeque<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViewState {
    DefectGrid(DefectGridState),
    DagExplorer(DagExplorerState),
    ReplMode(ReplState),
    MermaidViewer(MermaidViewerState),
    AnalysisToggle(AnalysisToggleState),
}

impl CacheStrategy<TuiState> for TuiCacheStrategy {
    type Value = TuiState;
    
    fn cache_key(&self, _key: &str) -> String {
        format!("tui_state_{}", self.session_id)
    }
    
    fn validate(&self, entry: &TuiState) -> bool {
        // Validate against current codebase hash
        entry.analysis_cache.codebase_hash == self.current_hash
    }
    
    fn ttl(&self) -> Option<Duration> {
        Some(Duration::from_secs(3600)) // 1 hour
    }
}
```

## 2. Defect Grid Implementation

### 2.1 Excel-like Grid with SIMD Sorting

```rust
pub struct DefectGridState {
    // Columnar storage for SIMD operations
    rankings: ColumnStore<DefectRanking>,
    // View state
    viewport: ViewportState,
    sort_state: SortState,
    filter_state: FilterState,
    // Selection
    selected_cells: BitVec,
    cursor: GridCursor,
}

#[derive(Debug, Clone)]
pub struct DefectRanking {
    pub file: CompactString,
    pub rank: u32,
    pub complexity_score: f32,
    pub churn_score: f32,
    pub dead_code_score: f32,
    pub satd_score: f32,
    pub duplication_score: f32,
    pub composite_score: f32,
    pub defect_probability: f32,
}

impl DefectGridState {
    pub fn sort_by_column_simd(&mut self, column: DefectColumn) {
        let indices = match column {
            DefectColumn::CompositeScore => {
                // SIMD-accelerated sorting using AVX2
                self.rankings.sort_indices_f32_simd(
                    |ranking| ranking.composite_score,
                    self.sort_state.direction
                )
            }
            DefectColumn::DefectProbability => {
                self.rankings.sort_indices_f32_simd(
                    |ranking| ranking.defect_probability,
                    self.sort_state.direction
                )
            }
            DefectColumn::File => {
                // Radix sort for strings
                self.rankings.sort_indices_string_radix(
                    |ranking| &ranking.file,
                    self.sort_state.direction
                )
            }
            _ => {
                // Generic sort for other columns
                self.rankings.sort_indices_generic(column, self.sort_state.direction)
            }
        };
        
        self.rankings.reorder_by_indices(&indices);
        self.invalidate_render_cache();
    }
    
    pub fn apply_filter_vectorized(&mut self, filter: &FilterExpression) {
        // Compile filter to SIMD operations
        let filter_mask = match filter {
            FilterExpression::Threshold { column, operator, value } => {
                self.rankings.compute_mask_simd(column, operator, *value)
            }
            FilterExpression::Pattern { column, regex } => {
                self.rankings.compute_regex_mask_parallel(column, regex)
            }
            FilterExpression::And(left, right) => {
                let left_mask = self.apply_filter_vectorized(left);
                let right_mask = self.apply_filter_vectorized(right);
                left_mask.bitand(&right_mask)
            }
        };
        
        self.filter_state.active_mask = Some(filter_mask);
    }
}
```

### 2.2 Grid Rendering with Cell Formatting

```rust
impl View for DefectGridView {
    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState) {
        let AppState::DefectGrid(grid_state) = state else { return };
        
        // Calculate layout with fixed header and scrollable body
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(5),     // Grid body
                Constraint::Length(2),  // Status bar
                Constraint::Length(3),  // REPL input
            ])
            .split(area);
        
        // Render sortable headers with indicators
        self.render_headers(frame, chunks[0], grid_state);
        
        // Render grid with conditional formatting
        let visible_rows = self.calculate_visible_rows(&grid_state.viewport, chunks[1].height);
        
        for (row_idx, ranking) in visible_rows {
            let cells = vec![
                Cell::from(format!("{:>4}", ranking.rank)),
                Cell::from(truncate_path(&ranking.file, 40)),
                Cell::from(format!("{:>6.2}", ranking.complexity_score))
                    .style(self.score_style(ranking.complexity_score)),
                Cell::from(format!("{:>6.2}", ranking.churn_score))
                    .style(self.score_style(ranking.churn_score)),
                Cell::from(format!("{:>6.2}", ranking.dead_code_score))
                    .style(self.score_style(ranking.dead_code_score)),
                Cell::from(format!("{:>6.2}", ranking.satd_score))
                    .style(self.score_style(ranking.satd_score)),
                Cell::from(format!("{:>6.2}", ranking.duplication_score))
                    .style(self.score_style(ranking.duplication_score)),
                Cell::from(format!("{:>7.3}", ranking.composite_score))
                    .style(self.composite_score_style(ranking.composite_score)),
                Cell::from(format!("{:>5.1}%", ranking.defect_probability * 100.0))
                    .style(self.probability_style(ranking.defect_probability)),
            ];
            
            let row = Row::new(cells).style(
                if grid_state.is_row_selected(row_idx) {
                    Style::default().bg(Color::Rgb(40, 40, 40))
                } else {
                    Style::default()
                }
            );
            
            frame.render_widget(row, self.calculate_row_area(chunks[1], row_idx));
        }
        
        // Render status bar with statistics
        self.render_status_bar(frame, chunks[2], grid_state);
        
        // Render REPL input area
        self.render_repl_input(frame, chunks[3], &state.repl_state);
    }
    
    fn score_style(&self, score: f32) -> Style {
        match score {
            s if s >= 0.8 => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            s if s >= 0.6 => Style::default().fg(Color::LightRed),
            s if s >= 0.4 => Style::default().fg(Color::Yellow),
            s if s >= 0.2 => Style::default().fg(Color::LightYellow),
            _ => Style::default().fg(Color::Green),
        }
    }
}
```

## 3. Real-time Analysis Integration

### 3.1 File Watcher with Incremental Updates

```rust
pub struct AnalysisWatcher {
    watcher: RecommendedWatcher,
    git_service: Arc<GitAnalysisService>,
    analysis_tx: mpsc::Sender<AnalysisUpdate>,
    debouncer: Arc<Mutex<HashMap<PathBuf, Instant>>>,
}

impl AnalysisWatcher {
    pub async fn watch(&mut self, project_path: PathBuf) -> Result<()> {
        let (tx, mut rx) = mpsc::channel(1024);
        
        let mut watcher = notify::recommended_watcher(move |res| {
            futures::executor::block_on(async {
                if let Ok(event) = res {
                    tx.send(event).await.ok();
                }
            });
        })?;
        
        watcher.watch(&project_path, RecursiveMode::Recursive)?;
        
        while let Some(event) = rx.recv().await {
            if self.should_analyze(&event) {
                self.trigger_analysis(event).await?;
            }
        }
        
        Ok(())
    }
    
    async fn trigger_analysis(&mut self, event: notify::Event) -> Result<()> {
        // Debounce rapid changes
        let path = &event.paths[0];
        let now = Instant::now();
        
        if let Some(last) = self.debouncer.lock().unwrap().get(path) {
            if now.duration_since(*last) < Duration::from_millis(500) {
                return Ok(());
            }
        }
        
        self.debouncer.lock().unwrap().insert(path.clone(), now);
        
        // Determine analysis type based on file
        let analysis_types = self.determine_analysis_types(path);
        
        // Trigger parallel analysis
        let mut join_set = JoinSet::new();
        
        for analysis_type in analysis_types {
            let path = path.clone();
            let service = self.get_analysis_service(analysis_type);
            
            join_set.spawn(async move {
                service.analyze_file(&path).await
            });
        }
        
        // Collect results and send update
        while let Some(result) = join_set.join_next().await {
            if let Ok(Ok(analysis)) = result {
                self.analysis_tx.send(AnalysisUpdate::Incremental(analysis)).await?;
            }
        }
        
        // Update git churn if needed
        if event.kind.is_modify() {
            let churn = self.git_service.analyze_file_churn(path).await?;
            self.analysis_tx.send(AnalysisUpdate::ChurnUpdate(churn)).await?;
        }
        
        Ok(())
    }
}
```

### 3.2 Analysis Type Toggle Interface

```rust
pub struct AnalysisToggleView {
    active_analyses: EnumSet<AnalysisType>,
    analysis_stats: HashMap<AnalysisType, AnalysisStats>,
}

impl AnalysisToggleView {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let analyses = [
            (AnalysisType::AST, "AST Context", self.active_analyses.contains(AnalysisType::AST)),
            (AnalysisType::Complexity, "Complexity", self.active_analyses.contains(AnalysisType::Complexity)),
            (AnalysisType::Churn, "Git Churn", self.active_analyses.contains(AnalysisType::Churn)),
            (AnalysisType::DeadCode, "Dead Code", self.active_analyses.contains(AnalysisType::DeadCode)),
            (AnalysisType::SATD, "Technical Debt", self.active_analyses.contains(AnalysisType::SATD)),
        ];
        
        let list_items: Vec<ListItem> = analyses
            .iter()
            .map(|(typ, name, active)| {
                let stats = &self.analysis_stats[typ];
                let content = if *active {
                    format!("✓ {} ({} items, {:.1}ms)", name, stats.item_count, stats.last_duration_ms)
                } else {
                    format!("  {} (disabled)", name)
                };
                
                ListItem::new(content).style(if *active {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::DarkGray)
                })
            })
            .collect();
        
        let list = List::new(list_items)
            .block(Block::default().title("Analysis Types [Space to toggle]").borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        
        frame.render_widget(list, area);
    }
    
    pub fn toggle_current(&mut self) -> Vec<AnalysisType> {
        let current = self.get_current_type();
        self.active_analyses.toggle(current);
        self.active_analyses.iter().collect()
    }
}
```

## 4. DAG Navigation

### 4.1 Relationship Explorer

```rust
pub struct DagExplorerState {
    current_node: NodeKey,
    dag: Arc<AstDag>,
    view_mode: DagViewMode,
    relationship_filter: EdgeType,
    navigation_history: VecDeque<NodeKey>,
    // Precomputed for performance
    pagerank_scores: HashMap<NodeKey, f32>,
    strongly_connected: Vec<HashSet<NodeKey>>,
}

#[derive(Debug, Clone)]
pub enum DagViewMode {
    Tree,           // Hierarchical tree view
    Graph,          // Force-directed graph
    Matrix,         // Adjacency matrix
    Sunburst,       // Circular hierarchy
}

impl DagExplorerState {
    pub fn navigate_to(&mut self, target: NodeKey) {
        self.navigation_history.push_back(self.current_node);
        if self.navigation_history.len() > 100 {
            self.navigation_history.pop_front();
        }
        self.current_node = target;
    }
    
    pub fn find_related(&self, edge_type: EdgeType) -> Vec<(NodeKey, RelationshipInfo)> {
        let mut related = Vec::new();
        
        // Outgoing edges
        for edge in self.dag.edges_from(self.current_node) {
            if edge.edge_type == edge_type {
                related.push((edge.target, RelationshipInfo {
                    direction: Direction::Outgoing,
                    edge_type,
                    weight: edge.weight,
                }));
            }
        }
        
        // Incoming edges
        for edge in self.dag.edges_to(self.current_node) {
            if edge.edge_type == edge_type {
                related.push((edge.source, RelationshipInfo {
                    direction: Direction::Incoming,
                    edge_type,
                    weight: edge.weight,
                }));
            }
        }
        
        // Sort by PageRank score for relevance
        related.sort_by(|a, b| {
            let score_a = self.pagerank_scores.get(&a.0).unwrap_or(&0.0);
            let score_b = self.pagerank_scores.get(&b.0).unwrap_or(&0.0);
            score_b.partial_cmp(score_a).unwrap()
        });
        
        related
    }
    
    pub fn jump_to_definition(&mut self) -> Option<FileLocation> {
        if let Some(node) = self.dag.get_node(self.current_node) {
            Some(FileLocation {
                path: node.file_path.clone(),
                line: node.span.start.line,
                column: node.span.start.column,
            })
        } else {
            None
        }
    }
}
```

## 5. Mermaid Diagram Integration

### 5.1 Inline Mermaid Renderer

```rust
pub struct MermaidViewerState {
    current_diagram: String,
    diagram_type: MermaidDiagramType,
    options: MermaidOptions,
    // Cache rendered ASCII art
    ascii_cache: HashMap<u64, String>,
}

impl MermaidViewerState {
    pub fn render_ascii(&mut self, width: u16, height: u16) -> String {
        let cache_key = self.calculate_cache_key(width, height);
        
        if let Some(cached) = self.ascii_cache.get(&cache_key) {
            return cached.clone();
        }
        
        // Parse Mermaid and convert to ASCII art
        let ascii = match self.diagram_type {
            MermaidDiagramType::Graph => self.render_graph_ascii(width, height),
            MermaidDiagramType::Sequence => self.render_sequence_ascii(width, height),
            MermaidDiagramType::Gantt => self.render_gantt_ascii(width, height),
        };
        
        self.ascii_cache.insert(cache_key, ascii.clone());
        ascii
    }
    
    fn render_graph_ascii(&self, width: u16, height: u16) -> String {
        // Use graph layout algorithms to position nodes
        let layout = self.calculate_sugiyama_layout(width, height);
        let mut canvas = AsciiCanvas::new(width as usize, height as usize);
        
        // Draw nodes
        for (node_id, position) in &layout.nodes {
            canvas.draw_box(
                position.x,
                position.y,
                position.width,
                position.height,
                &self.get_node_label(node_id),
            );
        }
        
        // Draw edges with ASCII art
        for edge in &layout.edges {
            canvas.draw_edge(
                edge.from_pos,
                edge.to_pos,
                edge.edge_type,
            );
        }
        
        canvas.to_string()
    }
}
```

## 6. REPL Integration

### 6.1 Embedded Analysis REPL

```rust
pub struct ReplState {
    input_buffer: String,
    history: VecDeque<String>,
    history_index: usize,
    completer: ReplCompleter,
    context: ReplContext,
}

pub struct ReplCompleter {
    commands: HashMap<String, CommandSpec>,
    file_cache: Arc<RwLock<Vec<PathBuf>>>,
}

impl ReplCompleter {
    pub fn complete(&self, input: &str) -> Vec<CompletionCandidate> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        
        match parts.len() {
            0 => self.complete_commands(""),
            1 => self.complete_commands(parts[0]),
            _ => self.complete_arguments(&parts[0], &parts[1..]),
        }
    }
    
    fn complete_arguments(&self, command: &str, args: &[&str]) -> Vec<CompletionCandidate> {
        match command {
            "analyze" => self.complete_analysis_args(args),
            "export" => self.complete_export_args(args),
            "navigate" => self.complete_navigation_args(args),
            "filter" => self.complete_filter_args(args),
            _ => vec![],
        }
    }
}

pub async fn execute_repl_command(command: &str, state: &mut TuiState) -> Result<ReplOutput> {
    let parsed = parse_command(command)?;
    
    match parsed.command.as_str() {
        "analyze" => {
            let analysis_type = AnalysisType::from_str(&parsed.args[0])?;
            let request = AnalysisRequest {
                project_path: state.project_path.clone(),
                analysis_type,
                options: parsed.options,
            };
            
            let result = state.unified_service.analyze(request).await?;
            Ok(ReplOutput::Analysis(result))
        }
        
        "export" => {
            let format = ExportFormat::from_str(&parsed.args[0])?;
            let data = state.export_current_view(format)?;
            Ok(ReplOutput::Export(data))
        }
        
        "filter" => {
            let expr = FilterExpression::parse(&parsed.args.join(" "))?;
            state.apply_global_filter(expr);
            Ok(ReplOutput::Message("Filter applied".to_string()))
        }
        
        "template" => {
            let file_path = &state.get_selected_file()?;
            let suggestions = state.template_service.suggest_fixes(file_path).await?;
            Ok(ReplOutput::TemplateSuggestions(suggestions))
        }
        
        _ => Err(anyhow!("Unknown command: {}", parsed.command)),
    }
}
```

## 7. Performance Optimizations

### 7.1 SIMD-Accelerated Operations

```rust
pub struct SimdOperations;

impl SimdOperations {
    #[target_feature(enable = "avx2")]
    pub unsafe fn rank_files_vectorized(scores: &[f32]) -> Vec<usize> {
        let n = scores.len();
        let mut indices: Vec<usize> = (0..n).collect();
        
        // Use AVX2 for parallel comparison
        let chunks = scores.chunks_exact(8);
        let remainder = chunks.remainder();
        
        for (chunk_idx, chunk) in chunks.enumerate() {
            let vec = _mm256_loadu_ps(chunk.as_ptr());
            // Vectorized sorting network
            self.sort_8_elements_avx2(vec, &mut indices[chunk_idx * 8..]);
        }
        
        // Handle remainder with scalar code
        if !remainder.is_empty() {
            let start = n - remainder.len();
            indices[start..].sort_by(|&a, &b| {
                scores[b].partial_cmp(&scores[a]).unwrap()
            });
        }
        
        indices
    }
    
    #[target_feature(enable = "avx2")]
    unsafe fn sort_8_elements_avx2(&self, values: __m256, indices: &mut [usize]) {
        // Bitonic sorting network for 8 elements
        // ... (implementation details)
    }
}
```

### 7.2 Render Caching with Differential Updates

```rust
pub struct RenderCache {
    grid_cache: HashMap<ViewportKey, RenderedGrid>,
    mermaid_cache: HashMap<DiagramKey, RenderedDiagram>,
    generation: AtomicU64,
    diff_tracker: DifferentialTracker,
}

pub struct DifferentialTracker {
    previous_state: Option<TuiState>,
    dirty_regions: Vec<Rect>,
}

impl DifferentialTracker {
    pub fn compute_diff(&mut self, old: &TuiState, new: &TuiState) -> Vec<RenderCommand> {
        let mut commands = Vec::new();
        
        // Compare grid data
        if old.grid_state.rankings != new.grid_state.rankings {
            let changed_rows = self.find_changed_rows(
                &old.grid_state.rankings,
                &new.grid_state.rankings,
            );
            
            for row in changed_rows {
                commands.push(RenderCommand::UpdateRow(row));
            }
        }
        
        // Compare viewport
        if old.grid_state.viewport != new.grid_state.viewport {
            commands.push(RenderCommand::ScrollTo(new.grid_state.viewport.offset));
        }
        
        commands
    }
}
```

## 8. Thread Pool Integration

```rust
pub struct TuiThreadPool {
    analysis_pool: Arc<rayon::ThreadPool>,
    render_pool: Arc<rayon::ThreadPool>,
    io_pool: Arc<tokio::runtime::Runtime>,
}

impl TuiThreadPool {
    pub fn new() -> Self {
        Self {
            analysis_pool: Arc::new(
                rayon::ThreadPoolBuilder::new()
                    .num_threads(num_cpus::get())
                    .thread_name(|i| format!("tui-analysis-{}", i))
                    .build()
                    .unwrap()
            ),
            render_pool: Arc::new(
                rayon::ThreadPoolBuilder::new()
                    .num_threads(2)
                    .thread_name(|i| format!("tui-render-{}", i))
                    .build()
                    .unwrap()
            ),
            io_pool: Arc::new(
                tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .thread_name("tui-io")
                    .enable_all()
                    .build()
                    .unwrap()
            ),
        }
    }
    
    pub fn spawn_analysis<F, R>(&self, f: F) -> JoinHandle<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        self.analysis_pool.spawn(move || {
            let result = f();
            tx.send(result).ok();
        });
        
        tokio::spawn(async move {
            rx.await.expect("Analysis task panicked")
        })
    }
}
```

## 9. Export Integration

```rust
pub enum ExportFormat {
    Sarif,
    Json,
    Csv,
    Markdown,
    Mermaid,
    Excel,
}

impl TuiState {
    pub async fn export_current_view(&self, format: ExportFormat) -> Result<ExportedData> {
        match &self.current_view {
            ViewState::DefectGrid(grid) => {
                match format {
                    ExportFormat::Excel => self.export_grid_to_excel(grid).await,
                    ExportFormat::Csv => self.export_grid_to_csv(grid),
                    ExportFormat::Sarif => self.export_grid_to_sarif(grid),
                    _ => self.export_grid_generic(grid, format),
                }
            }
            ViewState::DagExplorer(dag) => {
                match format {
                    ExportFormat::Mermaid => self.export_dag_to_mermaid(dag),
                    ExportFormat::Json => self.export_dag_to_json(dag),
                    _ => Err(anyhow!("Unsupported export format for DAG")),
                }
            }
            _ => Err(anyhow!("Current view does not support export")),
        }
    }
    
    async fn export_grid_to_excel(&self, grid: &DefectGridState) -> Result<ExportedData> {
        use xlsxwriter::{Workbook, Format};
        
        let mut workbook = Workbook::new("defect_analysis.xlsx")?;
        let mut worksheet = workbook.add_worksheet(Some("Defect Rankings"))?;
        
        // Header format
        let header_format = workbook.add_format()
            .set_bold()
            .set_bg_color(xlsxwriter::FormatColor::Custom(0x366092))
            .set_font_color(xlsxwriter::FormatColor::White);
        
        // Conditional formats for scores
        let high_risk_format = workbook.add_format()
            .set_bg_color(xlsxwriter::FormatColor::Custom(0xFFCCCC));
        
        // Write headers
        let headers = ["Rank", "File", "Complexity", "Churn", "Dead Code", 
                      "SATD", "Duplication", "Composite", "Defect %"];
        
        for (col, header) in headers.iter().enumerate() {
            worksheet.write_string(0, col as u16, header, Some(&header_format))?;
        }
        
        // Write data with formatting
        for (row_idx, ranking) in grid.rankings.iter().enumerate() {
            let row = row_idx as u32 + 1;
            
            worksheet.write_number(row, 0, ranking.rank as f64, None)?;
            worksheet.write_string(row, 1, &ranking.file, None)?;
            
            // Apply conditional formatting to score columns
            for (col, score) in [
                (2, ranking.complexity_score),
                (3, ranking.churn_score),
                (4, ranking.dead_code_score),
                (5, ranking.satd_score),
                (6, ranking.duplication_score),
            ].iter() {
                let format = if *score > 0.7 { Some(&high_risk_format) } else { None };
                worksheet.write_number(row, *col, *score as f64, format)?;
            }
            
            worksheet.write_number(row, 7, ranking.composite_score as f64, None)?;
            worksheet.write_number(row, 8, ranking.defect_probability as f64 * 100.0, None)?;
        }
        
        // Auto-fit columns
        worksheet.autofit()?;
        
        let buffer = workbook.save_to_buffer()?;
        Ok(ExportedData::Binary(buffer))
    }
}
```

## 10. Template Quick Fix Integration

```rust
pub struct TemplateQuickFix {
    template_service: Arc<UnifiedService>,
    quick_fix_cache: HashMap<PathBuf, Vec<QuickFix>>,
}

#[derive(Debug, Clone)]
pub struct QuickFix {
    pub title: String,
    pub description: String,
    pub template_uri: String,
    pub parameters: HashMap<String, Value>,
    pub preview: String,
}

impl TemplateQuickFix {
    pub async fn suggest_fixes(&mut self, file: &Path, issues: &[Issue]) -> Result<Vec<QuickFix>> {
        let mut fixes = Vec::new();
        
        for issue in issues {
            match issue.category {
                IssueCategory::MissingDocumentation => {
                    fixes.push(self.create_doc_template(file, issue).await?);
                }
                IssueCategory::ComplexFunction => {
                    fixes.push(self.create_refactor_template(file, issue).await?);
                }
                IssueCategory::MissingTests => {
                    fixes.push(self.create_test_template(file, issue).await?);
                }
                _ => {}
            }
        }
        
        self.quick_fix_cache.insert(file.to_path_buf(), fixes.clone());
        Ok(fixes)
    }
    
    pub async fn apply_fix(&self, fix: &QuickFix) -> Result<()> {
        let request = GenerateTemplateRequest {
            template_uri: fix.template_uri.clone(),
            parameters: fix.parameters.clone(),
        };
        
        let response = self.template_service.generate_template(request).await?;
        
        // Apply the generated content
        let target_path = fix.parameters.get("target_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing target path"))?;
        
        tokio::fs::write(target_path, response.content).await?;
        Ok(())
    }
}
```

## 11. Keybinding System

```rust
pub struct KeybindingRegistry {
    global_bindings: HashMap<KeyEvent, Action>,
    mode_bindings: HashMap<ViewState, HashMap<KeyEvent, Action>>,
    chord_state: Option<ChordState>,
}

#[derive(Debug, Clone)]
pub enum Action {
    // Navigation
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    PageUp,
    PageDown,
    Home,
    End,
    
    // Grid operations
    SortColumn(DefectColumn),
    ToggleSort,
    Filter,
    ClearFilter,
    SelectRow,
    SelectRange,
    ExportSelection,
    
    // View switching
    SwitchToGrid,
    SwitchToDag,
    SwitchToRepl,
    ToggleAnalysis,
    
    // Analysis
    RefreshAnalysis,
    AnalyzeFile,
    AnalyzeSelection,
    
    // REPL
    ExecuteCommand,
    HistoryPrevious,
    HistoryNext,
    Autocomplete,
    
    // Quick actions
    QuickFix,
    JumpToDefinition,
    FindReferences,
    ShowHelp,
}

impl KeybindingRegistry {
    pub fn default_bindings() -> Self {
        let mut registry = Self {
            global_bindings: HashMap::new(),
            mode_bindings: HashMap::new(),
            chord_state: None,
        };
        
        // Global bindings
        registry.global_bindings.extend([
            (key!(Ctrl-'c'), Action::Quit),
            (key!(Ctrl-'s'), Action::ExportSelection),
            (key!(F1), Action::ShowHelp),
            (key!(F5), Action::RefreshAnalysis),
        ]);
        
        // Grid mode bindings
        let mut grid_bindings = HashMap::new();
        grid_bindings.extend([
            (key!(Up), Action::NavigateUp),
            (key!(Down), Action::NavigateDown),
            (key!('j'), Action::NavigateDown),
            (key!('k'), Action::NavigateUp),
            (key!(PageDown), Action::PageDown),
            (key!(PageUp), Action::PageUp),
            (key!(Home), Action::Home),
            (key!(End), Action::End),
            (key!('1'), Action::SortColumn(DefectColumn::Rank)),
            (key!('2'), Action::SortColumn(DefectColumn::File)),
            (key!('3'), Action::SortColumn(DefectColumn::Complexity)),
            (key!('4'), Action::SortColumn(DefectColumn::Churn)),
            (key!('5'), Action::SortColumn(DefectColumn::DeadCode)),
            (key!('6'), Action::SortColumn(DefectColumn::SATD)),
            (key!('7'), Action::SortColumn(DefectColumn::Duplication)),
            (key!('8'), Action::SortColumn(DefectColumn::Composite)),
            (key!('9'), Action::SortColumn(DefectColumn::DefectProbability)),
            (key!('/'), Action::Filter),
            (key!(Esc), Action::ClearFilter),
            (key!(Enter), Action::JumpToDefinition),
            (key!(' '), Action::SelectRow),
            (key!(Shift-' '), Action::SelectRange),
        ]);
        
        registry.mode_bindings.insert(
            ViewState::DefectGrid(Default::default()),
            grid_bindings,
        );
        
        registry
    }
}
```

## 12. Performance Metrics

| Operation | Target | Implementation |
|-----------|--------|----------------|
| Grid Render (10K rows) | <16ms | Viewport virtualization + diff tracking |
| Sort Operation (10K rows) | <5ms | SIMD-accelerated sorting |
| Filter Application | <2ms | Vectorized boolean operations |
| Analysis Update | <100ms | Incremental + parallel processing |
| Mermaid ASCII Render | <10ms | Cached layout computation |
| Export to Excel | <500ms | Streaming with xlsxwriter |
| REPL Command | <50ms | Pre-parsed command cache |
| DAG Navigation | <1ms | Pre-computed adjacency lists |

## 13. Implementation Phases

### Phase 1: Core Infrastructure (Week 1-2)
- Implement `TuiAdapter` as `ProtocolAdapter`
- Integrate with `UnifiedService` and caching layers
- Basic grid rendering with viewport virtualization
- State persistence via `CacheStrategy`

### Phase 2: Defect Grid (Week 3-4)
- SIMD-accelerated sorting
- Excel-like filtering
- Multi-column selection
- Conditional formatting
- Export to Excel/CSV/SARIF

### Phase 3: Real-time Analysis (Week 5-6)
- File watcher integration
- Incremental analysis updates
- Analysis type toggling
- Performance monitoring

### Phase 4: Advanced Features (Week 7-8)
- DAG navigation with relationship filtering
- Inline Mermaid rendering
- REPL with autocomplete
- Template quick fixes
- Comprehensive keybindings

### Phase 5: Polish & Optimization (Week 9-10)
- Differential rendering
- Thread pool tuning
- Memory optimization
- Comprehensive testing
- Documentation

This specification leverages your existing infrastructure while providing a powerful interactive interface that maintains consistency with your unified protocol architecture.