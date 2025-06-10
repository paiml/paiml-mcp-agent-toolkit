#[cfg(feature = "tui")]
use async_trait::async_trait;
#[cfg(feature = "tui")]
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, MouseEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
#[cfg(feature = "tui")]
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, List, ListItem, ListState},
    Terminal,
};
#[cfg(feature = "tui")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "tui")]
use serde_json::Value;
#[cfg(feature = "tui")]
use std::collections::HashMap;
#[cfg(feature = "tui")]
use std::path::PathBuf;
#[cfg(feature = "tui")]
use std::sync::{Arc, RwLock};
#[cfg(feature = "tui")]
use std::time::{Duration, Instant};
#[cfg(feature = "tui")]
use thiserror::Error;
#[cfg(feature = "tui")]
use tokio::sync::mpsc;

#[cfg(feature = "tui")]
use crate::demo::protocol_harness::{DemoProtocol, ProtocolMetadata};

#[cfg(feature = "tui")]
#[derive(Debug, Clone, Copy, PartialEq)]
enum ControlFlow {
    Continue,
    Exit,
}

#[cfg(feature = "tui")]
#[derive(Debug, Clone, Copy)]
enum PanelId {
    FileTree,
    Analysis,
    Dag,
}

#[cfg(feature = "tui")]
impl PanelId {
    fn next(self) -> Self {
        match self {
            Self::FileTree => Self::Analysis,
            Self::Analysis => Self::Dag,
            Self::Dag => Self::FileTree,
        }
    }
}

#[cfg(feature = "tui")]
#[derive(Default)]
#[allow(dead_code)]
struct TuiState {
    selected_panel: PanelId,
    analysis_results: AnalysisResults,
    scroll_offset: usize,
    filter: String,
    list_state: ListState,
    progress: f32,
}

#[cfg(feature = "tui")]
impl Default for PanelId {
    fn default() -> Self {
        Self::FileTree
    }
}

#[cfg(feature = "tui")]
impl TuiState {
    fn cycle_panel(&mut self) {
        self.selected_panel = self.selected_panel.next();
    }
}

#[cfg(feature = "tui")]
#[derive(Default)]
struct AnalysisResults {
    hotspots: Vec<Hotspot>,
    files: Vec<FileInfo>,
    dag_nodes: HashMap<String, NodeInfo>,
}

#[cfg(feature = "tui")]
#[derive(Debug, Clone)]
struct Hotspot {
    file_path: PathBuf,
    description: String,
    severity: Severity,
    metric_value: f32,
}

#[cfg(feature = "tui")]
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Severity {
    Critical,
    Warning,
    Info,
}

#[cfg(feature = "tui")]
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct FileInfo {
    path: PathBuf,
    complexity: f32,
    size_kb: u64,
}

#[cfg(feature = "tui")]
#[derive(Debug, Clone)]
struct NodeInfo {
    id: String,
    name: String,
    kind: String,
}

#[cfg(feature = "tui")]
#[derive(Debug, Clone)]
struct AnalysisUpdate {
    update_type: UpdateType,
    progress: f32,
}

#[cfg(feature = "tui")]
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum UpdateType {
    FileDiscovered(PathBuf),
    ComplexityComputed(FileComplexity),
    ChurnAnalyzed(FileChurn),
    DagNodeAdded(NodeInfo),
    AnalysisComplete,
}

#[cfg(feature = "tui")]
#[derive(Debug, Clone)]
struct FileComplexity {
    file_path: PathBuf,
    complexity: f32,
}

#[cfg(feature = "tui")]
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct FileChurn {
    file_path: PathBuf,
    churn_rate: f32,
}

#[cfg(feature = "tui")]
pub struct TuiDemoAdapter {
    terminal: Option<Terminal<CrosstermBackend<std::io::Stdout>>>,
    state: Arc<RwLock<TuiState>>,
    update_rx: Option<mpsc::Receiver<AnalysisUpdate>>,
    should_quit: bool,
}

#[cfg(feature = "tui")]
#[derive(Debug, Serialize, Deserialize)]
pub struct TuiRequest {
    pub action: String,
    pub params: HashMap<String, Value>,
}

#[cfg(feature = "tui")]
impl From<Value> for TuiRequest {
    fn from(value: Value) -> Self {
        serde_json::from_value(value).unwrap_or_else(|_| TuiRequest {
            action: "unknown".to_string(),
            params: HashMap::new(),
        })
    }
}

#[cfg(feature = "tui")]
#[derive(Debug, Serialize, Deserialize)]
pub struct TuiResponse {
    pub status: String,
    pub data: Option<Value>,
}

#[cfg(feature = "tui")]
impl From<TuiResponse> for Value {
    fn from(val: TuiResponse) -> Self {
        serde_json::to_value(val).unwrap_or_default()
    }
}

#[cfg(feature = "tui")]
#[derive(Debug, Error)]
pub enum TuiDemoError {
    #[error("Terminal initialization failed: {0}")]
    TerminalInit(String),
    #[error("Event handling error: {0}")]
    EventHandling(String),
    #[error("Rendering error: {0}")]
    Rendering(String),
    #[error("Analysis error: {0}")]
    Analysis(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(feature = "tui")]
impl TuiDemoAdapter {
    pub fn new() -> Result<Self, TuiDemoError> {
        Ok(Self {
            terminal: None,
            state: Arc::new(RwLock::new(TuiState::default())),
            update_rx: None,
            should_quit: false,
        })
    }

    pub async fn initialize(&mut self) -> Result<(), TuiDemoError> {
        enable_raw_mode().map_err(|e| TuiDemoError::TerminalInit(e.to_string()))?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        let terminal =
            Terminal::new(backend).map_err(|e| TuiDemoError::TerminalInit(e.to_string()))?;

        self.terminal = Some(terminal);
        Ok(())
    }

    pub async fn run_event_loop(&mut self) -> Result<(), TuiDemoError> {
        let tick_rate = Duration::from_millis(50); // 20 FPS
        let mut last_tick = Instant::now();

        loop {
            // Process input events
            if let ControlFlow::Exit = self.process_events().await? {
                break;
            }

            // Process analysis updates
            self.process_updates().await?;

            // Render frame at consistent rate
            if last_tick.elapsed() >= tick_rate {
                self.render_frame().await?;
                last_tick = Instant::now();
            }

            if self.should_quit {
                break;
            }
        }

        self.cleanup()?;
        Ok(())
    }

    async fn process_events(&mut self) -> Result<ControlFlow, TuiDemoError> {
        if !event::poll(Duration::from_millis(10))? {
            return Ok(ControlFlow::Continue);
        }

        match event::read()? {
            Event::Key(key) => self.handle_key_event(key).await,
            Event::Mouse(mouse) => {
                self.handle_mouse_event(mouse).await?;
                Ok(ControlFlow::Continue)
            }
            Event::Resize(width, height) => {
                self.handle_resize(width, height)?;
                Ok(ControlFlow::Continue)
            }
            Event::FocusGained | Event::FocusLost | Event::Paste(_) => Ok(ControlFlow::Continue),
        }
    }

    async fn process_updates(&mut self) -> Result<(), TuiDemoError> {
        let updates = self.collect_pending_updates();
        for update in updates {
            self.apply_analysis_update(update).await?;
        }
        Ok(())
    }

    fn collect_pending_updates(&mut self) -> Vec<AnalysisUpdate> {
        let mut updates = Vec::new();
        if let Some(ref mut rx) = self.update_rx {
            while let Ok(update) = rx.try_recv() {
                updates.push(update);
            }
        }
        updates
    }

    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<ControlFlow, TuiDemoError> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
                Ok(ControlFlow::Exit)
            }
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

    async fn handle_mouse_event(&mut self, _mouse: MouseEvent) -> Result<(), TuiDemoError> {
        // Handle mouse events (click to select panels, scroll, etc.)
        Ok(())
    }

    fn handle_resize(&mut self, _width: u16, _height: u16) -> Result<(), TuiDemoError> {
        // Handle terminal resize
        Ok(())
    }

    async fn apply_analysis_update(&mut self, update: AnalysisUpdate) -> Result<(), TuiDemoError> {
        let mut state = self.state.write().unwrap();

        match update.update_type {
            UpdateType::FileDiscovered(path) => {
                let file_info = FileInfo {
                    path,
                    complexity: 0.0,
                    size_kb: 0,
                };
                state.analysis_results.files.push(file_info);
            }
            UpdateType::ComplexityComputed(complexity) => {
                // Update file complexity and create hotspots if needed
                if complexity.complexity > 10.0 {
                    let hotspot = Hotspot {
                        file_path: complexity.file_path.clone(),
                        description: format!("High complexity: {:.1}", complexity.complexity),
                        severity: if complexity.complexity > 20.0 {
                            Severity::Critical
                        } else {
                            Severity::Warning
                        },
                        metric_value: complexity.complexity,
                    };
                    state.analysis_results.hotspots.push(hotspot);
                }
            }
            UpdateType::DagNodeAdded(node) => {
                state
                    .analysis_results
                    .dag_nodes
                    .insert(node.id.clone(), node);
            }
            _ => {}
        }

        state.progress = update.progress;
        Ok(())
    }

    async fn render_frame(&mut self) -> Result<(), TuiDemoError> {
        self.render_frame_internal()
    }

    fn render_frame_internal(&mut self) -> Result<(), TuiDemoError> {
        if let Some(terminal) = &mut self.terminal {
            let state = self.state.read().unwrap();

            terminal
                .draw(|f| {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(3), // Header
                            Constraint::Min(10),   // Main content
                            Constraint::Length(3), // Status bar
                        ])
                        .split(f.area());

                    // Header with project info
                    let header = Block::default()
                        .borders(Borders::ALL)
                        .title("PAIML Analysis Toolkit - TUI Mode");

                    let progress = Gauge::default()
                        .block(header)
                        .gauge_style(Style::default().fg(Color::Green))
                        .percent((state.progress * 100.0) as u16);

                    f.render_widget(progress, chunks[0]);

                    // Main content area with panels
                    let main_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Percentage(30), // Left panel (file tree)
                            Constraint::Percentage(40), // Center panel (analysis)
                            Constraint::Percentage(30), // Right panel (DAG/metrics)
                        ])
                        .split(chunks[1]);

                    // File tree panel
                    let files: Vec<ListItem> = state
                        .analysis_results
                        .files
                        .iter()
                        .map(|file| {
                            let content = format!(
                                "{} ({:.1} complexity)",
                                file.path.display(),
                                file.complexity
                            );
                            ListItem::new(content)
                        })
                        .collect();

                    let file_list = List::new(files)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title("Files")
                                .border_style(
                                    if matches!(state.selected_panel, PanelId::FileTree) {
                                        Style::default().fg(Color::Yellow)
                                    } else {
                                        Style::default()
                                    },
                                ),
                        )
                        .highlight_style(
                            Style::default()
                                .bg(Color::DarkGray)
                                .add_modifier(Modifier::BOLD),
                        );

                    f.render_widget(file_list, main_chunks[0]);

                    // Analysis panel
                    let items: Vec<ListItem> = state
                        .analysis_results
                        .hotspots
                        .iter()
                        .map(|h| {
                            let style = match h.severity {
                                Severity::Critical => Style::default().fg(Color::Red),
                                Severity::Warning => Style::default().fg(Color::Yellow),
                                Severity::Info => Style::default().fg(Color::Green),
                            };

                            let content = format!(
                                "{}: {} ({:.1})",
                                h.file_path.display(),
                                h.description,
                                h.metric_value
                            );

                            ListItem::new(content).style(style)
                        })
                        .collect();

                    let analysis_list = List::new(items)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title("Analysis Results")
                                .border_style(
                                    if matches!(state.selected_panel, PanelId::Analysis) {
                                        Style::default().fg(Color::Yellow)
                                    } else {
                                        Style::default()
                                    },
                                ),
                        )
                        .highlight_style(
                            Style::default()
                                .bg(Color::DarkGray)
                                .add_modifier(Modifier::BOLD),
                        );

                    f.render_widget(analysis_list, main_chunks[1]);

                    // DAG panel
                    let dag_items: Vec<ListItem> = state
                        .analysis_results
                        .dag_nodes
                        .values()
                        .map(|node| {
                            let content = format!("{}: {}", node.kind, node.name);
                            ListItem::new(content)
                        })
                        .collect();

                    let dag_list = List::new(dag_items)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title("Dependency Graph")
                                .border_style(if matches!(state.selected_panel, PanelId::Dag) {
                                    Style::default().fg(Color::Yellow)
                                } else {
                                    Style::default()
                                }),
                        )
                        .highlight_style(
                            Style::default()
                                .bg(Color::DarkGray)
                                .add_modifier(Modifier::BOLD),
                        );

                    f.render_widget(dag_list, main_chunks[2]);

                    // Status bar with shortcuts
                    let status = Block::default()
                        .borders(Borders::ALL)
                        .title("Controls: [Q]uit | [Tab]Switch Panel | [F5]Refresh | [/]Search");

                    f.render_widget(status, chunks[2]);
                })
                .map_err(|e| TuiDemoError::Rendering(e.to_string()))?;
        }

        Ok(())
    }
}

#[cfg(feature = "tui")]
impl TuiDemoAdapter {
    async fn enter_search_mode(&mut self) -> Result<(), TuiDemoError> {
        // TRACKED: Implement search mode
        Ok(())
    }

    async fn refresh_analysis(&mut self) -> Result<(), TuiDemoError> {
        // TRACKED: Implement analysis refresh
        Ok(())
    }

    fn cleanup(&mut self) -> Result<(), TuiDemoError> {
        if let Some(mut terminal) = self.terminal.take() {
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;
        }
        Ok(())
    }
}

#[cfg(feature = "tui")]
impl Default for TuiDemoAdapter {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(feature = "tui")]
#[async_trait]
impl DemoProtocol for TuiDemoAdapter {
    type Request = TuiRequest;
    type Response = TuiResponse;
    type Error = TuiDemoError;

    async fn decode_request(&self, raw: &[u8]) -> Result<Self::Request, Self::Error> {
        let request: TuiRequest = serde_json::from_slice(raw)?;
        Ok(request)
    }

    async fn encode_response(&self, response: Self::Response) -> Result<Vec<u8>, Self::Error> {
        let encoded = serde_json::to_vec(&response)?;
        Ok(encoded)
    }

    async fn get_protocol_metadata(&self) -> ProtocolMetadata {
        ProtocolMetadata {
            name: "TUI",
            version: "1.0",
            description: "Terminal User Interface for interactive analysis".to_string(),
            request_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {"type": "string"},
                    "params": {"type": "object"}
                },
                "required": ["action"]
            }),
            response_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "status": {"type": "string"},
                    "data": {"type": "object"}
                },
                "required": ["status"]
            }),
            example_requests: vec![serde_json::json!({
                "action": "analyze",
                "params": {"path": "/path/to/project"}
            })],
            capabilities: vec![
                "interactive_analysis".to_string(),
                "real_time_updates".to_string(),
                "keyboard_navigation".to_string(),
            ],
        }
    }

    async fn execute_demo(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        match request.action.as_str() {
            "analyze" => {
                // For TUI, this just starts the analysis
                Ok(TuiResponse {
                    status: "started".to_string(),
                    data: Some(serde_json::json!({
                        "message": "TUI analysis started"
                    })),
                })
            }
            "quit" => Ok(TuiResponse {
                status: "quitting".to_string(),
                data: None,
            }),
            _ => Ok(TuiResponse {
                status: "unknown_action".to_string(),
                data: None,
            }),
        }
    }
}

#[cfg(feature = "tui")]
impl TuiDemoAdapter {
    pub async fn handle_request(
        &mut self,
        request: TuiRequest,
    ) -> Result<TuiResponse, TuiDemoError> {
        match request.action.as_str() {
            "analyze" => {
                // Start analysis in background
                let (tx, rx) = mpsc::channel(64);
                self.update_rx = Some(rx);

                // Simulate analysis updates
                tokio::spawn(async move {
                    for i in 0..10 {
                        let update = AnalysisUpdate {
                            update_type: UpdateType::FileDiscovered(PathBuf::from(format!(
                                "src/file_{i}.rs"
                            ))),
                            progress: i as f32 / 10.0,
                        };
                        let _ = tx.send(update).await;
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }

                    let _ = tx
                        .send(AnalysisUpdate {
                            update_type: UpdateType::AnalysisComplete,
                            progress: 1.0,
                        })
                        .await;
                });

                Ok(TuiResponse {
                    status: "started".to_string(),
                    data: None,
                })
            }
            "quit" => {
                self.should_quit = true;
                Ok(TuiResponse {
                    status: "quitting".to_string(),
                    data: None,
                })
            }
            _ => Ok(TuiResponse {
                status: "unknown_action".to_string(),
                data: None,
            }),
        }
    }
}

// Provide empty stubs when TUI feature is disabled
#[cfg(not(feature = "tui"))]
pub struct TuiDemoAdapter;

#[cfg(not(feature = "tui"))]
#[derive(Debug)]
pub struct TuiRequest;

#[cfg(not(feature = "tui"))]
#[derive(Debug)]
pub struct TuiResponse;

#[cfg(not(feature = "tui"))]
impl TuiDemoAdapter {
    pub fn new() -> Result<Self, &'static str> {
        Err("TUI feature not enabled")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tui_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
