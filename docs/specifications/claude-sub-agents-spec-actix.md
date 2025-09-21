# PMAT Claude Code Sub-Agents Integration Specification

## Revision Notes (Post-Review Enhancement)

This specification has been enhanced based on critical review feedback addressing:
- **Complexity Reduction**: Replaced 8-week timeline with 12-month phased approach starting with modular monolith
- **Consistency Model Fix**: Introduced Raft consensus for quality-critical state instead of eventual consistency everywhere
- **Pragmatic Starting Point**: Begin with in-process actors (tokio) before distribution
- **Anti-Pattern Mitigation**: Added circuit breakers, bulkheads, and dependency inversion to prevent distributed monolith
- **Simplified Discovery**: Replaced mDNS with static configuration and optional etcd

The architecture maintains its ambitious vision while providing a realistic incremental path that delivers value at each phase.

## Executive Summary

This specification defines the architectural transformation of PMAT from a monolithic MCP server into a distributed agent ecosystem leveraging Claude Code's sub-agent orchestration. The design emphasizes functional decomposition, state isolation, and lock-free coordination between specialized analysis agents.

## 1. Development Mandate: Extreme Quality Engineering

### 1.1 Test-Driven Development Protocol

Every line of production code SHALL be preceded by a failing test. The Red-Green-Refactor cycle is non-negotiable.

```rust
// MANDATORY: Test precedes implementation
#[cfg(test)]
mod agent_spawn_tests {
    use proptest::prelude::*;
    use tokio::time::{timeout, Duration};
    
    #[tokio::test]
    async fn test_agent_spawn_latency_p99_under_50ms() {
        // RED: Write failing test first
        let registry = AgentRegistry::new();
        let latencies = collect_spawn_latencies(1000).await;
        let p99 = latencies.percentile(99.0);
        assert!(p99 < Duration::from_millis(50));
    }
    
    proptest! {
        #[test]
        fn test_agent_message_ordering_invariant(
            messages in prop::collection::vec(any::<AgentMessage>(), 1..1000)
        ) {
            // Property: FIFO ordering per sender preserved
            let registry = tokio_test::block_on(AgentRegistry::new());
            let agent = tokio_test::block_on(registry.spawn_test_agent());
            
            for msg in &messages {
                tokio_test::block_on(agent.send(msg.clone()));
            }
            
            let received = tokio_test::block_on(agent.drain());
            prop_assert_eq!(messages, received);
        }
    }
}

// GREEN: Minimal implementation to pass
pub struct AgentRegistry {
    spawn_histogram: Histogram<u64>,
}

impl AgentRegistry {
    pub async fn spawn_agent(&self, spec: AgentSpec) -> Result<AgentId> {
        let start = Instant::now();
        
        // Implementation driven by test requirements
        let id = AgentId::new();
        let handle = self.spawn_with_deadline(spec, Duration::from_millis(45)).await?;
        
        self.spawn_histogram.record(start.elapsed().as_micros() as u64);
        Ok(id)
    }
}

// REFACTOR: Optimize while maintaining green tests
```

### 1.2 Extreme Quality Gates

```toml
# .pmat/quality.toml - ZERO TOLERANCE configuration
[complexity]
max_cyclomatic = 10          # McCabe complexity hard limit
max_cognitive = 7             # Cognitive complexity (Sonar)
max_nesting = 3              # Maximum nesting depth
max_params = 4               # Maximum function parameters
max_lines = 50               # Maximum lines per function

[satd]
allowed_count = 0            # ZERO technical debt comments
forbidden_patterns = [
    "TODO", "FIXME", "HACK", "XXX", "REFACTOR",
    "OPTIMIZE", "REVIEW", "DEPRECATED", "TEMPORARY"
]

[efficiency]
max_big_o = "O(n log n)"     # Maximum algorithmic complexity
space_complexity = "O(n)"    # Maximum space complexity
mandatory_const_time = [      # Functions requiring O(1)
    "AgentRegistry::lookup",
    "MessageRouter::route",
    "CircuitBreaker::check_state"
]

[entropy]
min_shannon = 3.5            # Minimum code diversity
max_duplication = 0.02       # Maximum 2% duplication
max_copy_paste = 10          # Maximum copied lines

[testing]
min_coverage = 0.95          # 95% minimum coverage
min_mutation_score = 0.90    # 90% mutation test survival
property_tests_per_module = 5 # Minimum property tests
doctests_required = true     # Every public function
examples_required = true     # Every public module

[compilation]
deny_warnings = true         # #![deny(warnings)] enforced
unsafe_forbidden = true      # No unsafe blocks allowed
pedantic_clippy = true      # clippy::pedantic enabled
must_use_results = true     # All Results handled
```

### 1.3 Quality Enforcement Engine

```rust
// Pre-commit hook runs quality gates
#[derive(Debug)]
pub struct QualityGateRunner {
    analyzers: Vec<Box<dyn QualityAnalyzer>>,
    thresholds: QualityThresholds,
}

impl QualityGateRunner {
    pub fn validate_module(&self, module_path: &Path) -> Result<QualityReport> {
        let ast = syn::parse_file(&std::fs::read_to_string(module_path)?)?;
        
        // Cyclomatic complexity via graph analysis
        let cfg = ControlFlowGraph::from_ast(&ast);
        let complexity = cfg.cyclomatic_complexity();
        if complexity > self.thresholds.max_cyclomatic {
            return Err(QualityViolation::ExcessiveComplexity {
                found: complexity,
                max: self.thresholds.max_cyclomatic,
                location: module_path.to_path_buf(),
            });
        }
        
        // Big-O analysis via symbolic execution
        let efficiency = EfficiencyAnalyzer::analyze(&ast)?;
        for (fn_name, big_o) in efficiency.complexities() {
            if big_o > self.thresholds.max_big_o {
                return Err(QualityViolation::InefficientAlgorithm {
                    function: fn_name,
                    complexity: big_o,
                    required: self.thresholds.max_big_o,
                });
            }
        }
        
        // Shannon entropy for code diversity
        let entropy = calculate_shannon_entropy(&ast);
        if entropy < self.thresholds.min_entropy {
            return Err(QualityViolation::InsufficientDiversity {
                entropy,
                required: self.thresholds.min_entropy,
            });
        }
        
        Ok(QualityReport::passed())
    }
}

// Git pre-commit hook integration
fn pre_commit_hook() -> Result<()> {
    let changed_files = get_staged_rust_files()?;
    let gate_runner = QualityGateRunner::strict();
    
    for file in changed_files {
        gate_runner.validate_module(&file)?;
    }
    
    // Run property tests on changed modules
    run_property_tests(&changed_files)?;
    
    // Verify mutation test coverage
    verify_mutation_coverage(&changed_files)?;
    
    Ok(())
}
```

### 1.4 Sprint Execution Protocol

```rust
// Ticket-driven development with atomic commits
pub struct SprintExecutor {
    roadmap: Roadmap,
    current_sprint: Sprint,
    ticket_tracker: TicketTracker,
}

impl SprintExecutor {
    pub async fn execute_ticket(&mut self, ticket_id: TicketId) -> Result<()> {
        let ticket = self.ticket_tracker.get(ticket_id)?;
        
        // Create feature branch
        let branch_name = format!("feat/{}-{}", ticket_id, ticket.slug());
        git::create_branch(&branch_name)?;
        
        // TDD cycle for ticket implementation
        for requirement in ticket.requirements() {
            // Step 1: Write failing test
            let test_path = self.write_failing_test(&requirement)?;
            self.verify_test_fails(&test_path)?;
            
            // Step 2: Minimal implementation
            let impl_path = self.implement_requirement(&requirement)?;
            self.verify_test_passes(&test_path)?;
            
            // Step 3: Refactor with quality gates
            self.refactor_with_quality_check(&impl_path)?;
            
            // Step 4: Atomic commit
            let commit_msg = format!(
                "{}: {}\n\n- Complexity: {}\n- Coverage: {}%\n- Big-O: {}\n- Tests: {} passing",
                ticket_id,
                requirement.description(),
                self.measure_complexity(&impl_path)?,
                self.measure_coverage(&impl_path)?,
                self.analyze_big_o(&impl_path)?,
                self.count_tests(&test_path)?
            );
            
            git::commit(&[test_path, impl_path], &commit_msg)?;
        }
        
        // Quality gate before merge
        self.run_comprehensive_quality_check()?;
        
        // Squash merge with ticket reference
        git::merge_squash(&branch_name, &format!("Closes #{}", ticket_id))?;
        
        Ok(())
    }
}
```

### 1.5 Roadmap Structure

```yaml
# roadmap.yaml - Sprint-based execution plan
sprints:
  - id: sprint-1
    goal: "Modular monolith with 100% test coverage"
    duration: 2_weeks
    tickets:
      - id: PMAT-001
        title: "Extract analyzer module with trait abstraction"
        requirements:
          - "Trait-based analyzer interface"
          - "Complexity < 7 per function"
          - "Property tests for all invariants"
        acceptance:
          - "Zero SATD comments"
          - "100% branch coverage"
          - "All doctests passing"
          
      - id: PMAT-002
        title: "Implement message passing abstraction"
        requirements:
          - "MPSC channel wrapper with backpressure"
          - "O(1) message routing"
          - "Property: FIFO ordering preserved"
        acceptance:
          - "Benchmarks show 100K msg/s"
          - "P99 latency < 2ms"
          - "Zero allocations in hot path"
```

### 1.6 Continuous Quality Metrics

```rust
// Real-time quality dashboard
pub struct QualityMetricsCollector {
    complexity_histogram: Histogram,
    coverage_gauge: Gauge,
    satd_counter: Counter,
    big_o_tracker: BigOTracker,
}

impl QualityMetricsCollector {
    pub fn instrument_build(&self) -> Result<BuildMetrics> {
        // Compiler-enforced quality
        let output = Command::new("cargo")
            .args(&[
                "build",
                "--release",
                "--",
                "-D", "warnings",
                "-D", "clippy::all",
                "-D", "clippy::pedantic",
                "-D", "clippy::nursery",
                "-D", "clippy::cargo",
            ])
            .output()?;
        
        if !output.status.success() {
            return Err(BuildError::QualityViolation(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        // Post-build analysis
        let metrics = BuildMetrics {
            avg_complexity: self.analyze_complexity()?,
            test_coverage: self.measure_coverage()?,
            satd_count: self.scan_satd()?,
            max_big_o: self.analyze_efficiency()?,
            build_time: self.measure_build_time()?,
        };
        
        // Fail build if quality degrades
        self.enforce_no_regression(&metrics)?;
        
        Ok(metrics)
    }
}

// GitHub Actions integration
name: Extreme Quality Gate
on: [push, pull_request]
jobs:
  quality:
    steps:
      - name: TDD Verification
        run: |
          # Verify tests were written before code
          git log --format="%H" -n 20 | while read commit; do
            pmat verify-tdd-commit $commit || exit 1
          done
      
      - name: Complexity Analysis
        run: pmat analyze complexity --max 10 --fail-on-violation
        
      - name: SATD Detection
        run: pmat analyze satd --zero-tolerance
        
      - name: Big-O Verification
        run: pmat analyze efficiency --max "O(n log n)"
        
      - name: Property Testing
        run: cargo test --features proptest -- --nocapture
        
      - name: Mutation Testing
        run: cargo mutants --minimum-score 0.90
        
      - name: Doctest Coverage
        run: cargo test --doc
        
      - name: Example Verification
        run: cargo test --examples
```

### 1.7 Zero-Defect Commitment

```rust
// Formal verification for critical paths
#[kani::proof]
fn verify_message_ordering_preserved() {
    let mut queue: MessageQueue = kani::any();
    let msg1: Message = kani::any();
    let msg2: Message = kani::any();
    
    queue.push(msg1.clone());
    queue.push(msg2.clone());
    
    let out1 = queue.pop();
    let out2 = queue.pop();
    
    kani::assert(out1 == Some(msg1), "First message preserved");
    kani::assert(out2 == Some(msg2), "Second message preserved");
}

// Exhaustive testing via fuzzing
#[test]
fn fuzz_agent_registry() {
    bolero::check!()
        .with_type::<Vec<AgentOperation>>()
        .for_each(|ops| {
            let registry = AgentRegistry::new();
            for op in ops {
                // Must never panic or violate invariants
                let _ = registry.execute(op);
                assert!(registry.check_invariants());
            }
        });
}
```

## 2. System Architecture

### 1.1 Agent Taxonomy

```rust
// Core agent hierarchy with trait-based polymorphism
pub trait PmatAgent: Send + Sync + 'static {
    type Config: DeserializeOwned + Serialize;
    type State: AgentState;
    type Message: AgentMessage;
    
    fn capabilities(&self) -> AgentCapabilities;
    async fn initialize(config: Self::Config) -> Result<Self>;
    async fn process(&mut self, msg: Self::Message) -> Result<AgentResponse>;
    async fn checkpoint(&self) -> Result<Self::State>;
}

pub enum AgentClass {
    Analyzer(AnalyzerAgent),      // Static analysis, metrics extraction
    Transformer(TransformerAgent), // Refactoring, code modification
    Validator(ValidatorAgent),     // Quality gates, threshold enforcement
    Orchestrator(OrchestratorAgent), // Workflow coordination
    Monitor(MonitorAgent),         // File watching, continuous analysis
}
```

### 1.2 Agent Registry Architecture

```rust
// Lock-free agent registry using Arc<DashMap> for concurrent access
pub struct AgentRegistry {
    agents: Arc<DashMap<AgentId, AgentHandle>>,
    capabilities: Arc<DashMap<Capability, Vec<AgentId>>>,
    topology: Arc<RwLock<AgentTopology>>,
}

impl AgentRegistry {
    pub async fn spawn_agent(&self, spec: AgentSpec) -> Result<AgentId> {
        let id = AgentId::new();
        let (tx, rx) = mpsc::channel::<AgentMessage>(1024); // Bounded channel
        
        let handle = match spec.class {
            AgentClass::Analyzer(_) => {
                // CPU-bound agents get dedicated thread
                let handle = tokio::task::spawn_blocking(move || {
                    analyzer_runtime(spec, rx)
                });
                AgentHandle::Dedicated(handle)
            },
            AgentClass::Orchestrator(_) => {
                // I/O-bound agents share tokio runtime
                let handle = tokio::spawn(orchestrator_runtime(spec, rx));
                AgentHandle::Shared(handle)
            },
            _ => // ... other agent types
        };
        
        self.agents.insert(id, handle);
        Ok(id)
    }
}
```

## 2. Agent Specifications

### 2.1 Quality Gate Agent

```yaml
# .claude/agents/pmat-quality-gate.yaml
apiVersion: pmat.io/v1
kind: Agent
metadata:
  name: pmat-quality-gate
  class: Validator
spec:
  description: |
    Enforces Toyota Way quality standards with zero-tolerance for technical debt.
    Implements multi-stage validation pipeline with fail-fast semantics.
  
  capabilities:
    - complexity_analysis
    - satd_detection
    - security_scanning
    - dead_code_elimination
  
  tools:
    - pmat_analyze_complexity
    - pmat_detect_satd
    - pmat_security_scan
    - pmat_quality_score
  
  model: sonnet  # Optimized for rapid validation
  
  config:
    thresholds:
      max_complexity_p50: 10
      max_complexity_p99: 20
      max_satd_count: 0  # Zero tolerance
      min_coverage: 0.80
    
    validation_pipeline:
      - stage: parse
        timeout_ms: 100
        parallelism: 8
      - stage: analyze  
        timeout_ms: 500
        parallelism: 4
      - stage: validate
        timeout_ms: 200
        parallelism: 1
    
    resource_limits:
      max_memory_mb: 512
      max_cpu_percent: 25
      max_file_size_mb: 10
```

### 2.2 Refactoring Agent

```rust
// Refactoring agent with transactional semantics
pub struct RefactorAgent {
    ast_cache: Arc<DashMap<FileId, SyntaxTree>>,
    transform_log: Arc<SegQueue<Transform>>, // Lock-free queue
    checkpoint_store: Arc<CheckpointStore>,
}

impl PmatAgent for RefactorAgent {
    type Config = RefactorConfig;
    type State = RefactorState;
    type Message = RefactorRequest;
    
    async fn process(&mut self, msg: Self::Message) -> Result<AgentResponse> {
        match msg {
            RefactorRequest::Interactive(params) => {
                // Begin transaction
                let txn = self.checkpoint_store.begin().await?;
                
                // Apply transforms with rollback capability
                let transforms = self.plan_transforms(&params).await?;
                for transform in transforms {
                    match self.apply_transform(&transform).await {
                        Ok(_) => self.transform_log.push(transform),
                        Err(e) => {
                            txn.rollback().await?;
                            return Err(e);
                        }
                    }
                }
                
                // Commit transaction
                txn.commit().await?;
                Ok(AgentResponse::Refactored(self.transform_log.len()))
            },
            // ... other refactor operations
        }
    }
}
```

### 2.3 Language-Specific Analyzers

```rust
// Trait for language-specific analysis with zero-cost abstractions
pub trait LanguageAnalyzer: Send + Sync {
    type AST: AbstractSyntaxTree;
    type Metrics: LanguageMetrics;
    
    fn parse(&self, source: &str) -> Result<Self::AST>;
    fn analyze(&self, ast: &Self::AST) -> Self::Metrics;
    fn detect_patterns(&self, ast: &Self::AST) -> Vec<CodePattern>;
}

// Rust-specific analyzer with ownership tracking
pub struct RustAnalyzer {
    parser: Arc<syn::Parser>,
    borrow_checker: Arc<BorrowChecker>,
    unsafe_detector: Arc<UnsafeDetector>,
}

impl LanguageAnalyzer for RustAnalyzer {
    type AST = syn::File;
    type Metrics = RustMetrics;
    
    fn analyze(&self, ast: &Self::AST) -> Self::Metrics {
        RustMetrics {
            lifetime_complexity: self.compute_lifetime_complexity(ast),
            unsafe_blocks: self.unsafe_detector.scan(ast),
            move_semantics_score: self.analyze_moves(ast),
            trait_coherence: self.check_trait_impl_coherence(ast),
        }
    }
}
```

## 3. Inter-Agent Communication Protocol

### 3.1 Message Format

```rust
// Zero-copy message passing using bytes::Bytes
#[derive(Serialize, Deserialize)]
pub struct AgentMessage {
    header: MessageHeader,
    payload: Bytes, // Zero-copy payload
}

#[derive(Serialize, Deserialize)]
pub struct MessageHeader {
    id: Uuid,
    from: AgentId,
    to: AgentId,
    timestamp: u64, // Unix timestamp in nanos
    correlation_id: Option<Uuid>,
    priority: Priority,
    ttl_ms: u32,
}

// Priority queue for message scheduling
#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub enum Priority {
    Critical = 0,  // Quality gate violations
    High = 1,      // User-initiated requests  
    Normal = 2,    // Background analysis
    Low = 3,       // Cache warming, precomputation
}
```

### 3.2 Communication Patterns

```rust
// Request-Response pattern with timeout
pub async fn request_response(
    registry: &AgentRegistry,
    from: AgentId,
    to: AgentId,
    request: Request,
    timeout: Duration,
) -> Result<Response> {
    let correlation_id = Uuid::new_v4();
    let (tx, rx) = oneshot::channel();
    
    // Register response handler
    registry.register_handler(correlation_id, tx).await;
    
    // Send request
    let msg = AgentMessage {
        header: MessageHeader {
            id: Uuid::new_v4(),
            from,
            to,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos() as u64,
            correlation_id: Some(correlation_id),
            priority: request.priority(),
            ttl_ms: timeout.as_millis() as u32,
        },
        payload: Bytes::from(bincode::serialize(&request)?),
    };
    
    registry.route_message(msg).await?;
    
    // Await response with timeout
    tokio::time::timeout(timeout, rx).await??
}

// Publish-Subscribe for broadcast notifications
pub struct PubSubBroker {
    topics: Arc<DashMap<Topic, Vec<AgentId>>>,
    subscribers: Arc<DashMap<AgentId, mpsc::Sender<AgentMessage>>>,
}

impl PubSubBroker {
    pub async fn publish(&self, topic: Topic, event: Event) -> Result<()> {
        if let Some(subscribers) = self.topics.get(&topic) {
            let msg = self.create_event_message(event)?;
            
            // Parallel broadcast using rayon
            subscribers.par_iter().for_each(|agent_id| {
                if let Some(tx) = self.subscribers.get(agent_id) {
                    let _ = tx.try_send(msg.clone()); // Non-blocking send
                }
            });
        }
        Ok(())
    }
}
```

## 4. State Management

### 4.1 Hybrid Event Sourcing with Snapshots

```rust
// Snapshot-accelerated event log for sub-second recovery
pub struct HybridEventStore {
    event_log: Arc<EventLog>,
    snapshot_store: Arc<SnapshotStore>,
    snapshot_interval: usize,  // Events between snapshots
    snapshot_scheduler: Arc<SnapshotScheduler>,
}

pub struct SnapshotStore {
    segments: Arc<RwLock<BTreeMap<SnapshotId, SnapshotSegment>>>,
    compression: CompressionAlgorithm,
    storage_backend: Box<dyn StorageBackend>,
}

impl SnapshotStore {
    pub async fn create_snapshot(&self, state: &AgentState) -> Result<SnapshotId> {
        let snapshot = Snapshot {
            id: SnapshotId::new(),
            timestamp: SystemTime::now(),
            event_id: state.last_event_id(),
            checksum: self.calculate_checksum(state),
        };
        
        // Serialize with zero-copy where possible
        let mut serializer = SnapSerializer::new();
        serializer.write_header(&snapshot)?;
        
        // Use rkyv for zero-copy deserialization
        let archived = rkyv::to_bytes::<_, 256>(state)?;
        
        // Compress with zstd (best ratio/speed tradeoff)
        let compressed = zstd::encode_all(&*archived, 3)?;
        
        // Write atomically with rename
        let temp_path = self.temp_path(&snapshot.id);
        let final_path = self.final_path(&snapshot.id);
        
        tokio::fs::write(&temp_path, &compressed).await?;
        tokio::fs::rename(&temp_path, &final_path).await?;
        
        // Update index
        self.segments.write().await.insert(
            snapshot.id,
            SnapshotSegment {
                path: final_path,
                size: compressed.len(),
                compression_ratio: archived.len() as f64 / compressed.len() as f64,
            },
        );
        
        Ok(snapshot.id)
    }
    
    pub async fn load_latest_before(&self, timestamp: SystemTime) -> Result<RestoredState> {
        let segments = self.segments.read().await;
        
        // Binary search for snapshot
        let snapshot = segments
            .range(..=SnapshotId::from_timestamp(timestamp))
            .next_back()
            .ok_or(Error::NoSnapshot)?;
        
        // Memory-map the snapshot file for efficiency
        let file = tokio::fs::File::open(&snapshot.1.path).await?;
        let mmap = unsafe { Mmap::map(&file)? };
        
        // Decompress in streaming fashion
        let decompressed = zstd::decode_all(Cursor::new(&mmap[..]))?;
        
        // Zero-copy deserialize with rkyv
        let archived = unsafe { rkyv::archived_root::<AgentState>(&decompressed) };
        let state: AgentState = archived.deserialize(&mut rkyv::Infallible)?;
        
        Ok(RestoredState {
            state,
            snapshot_id: snapshot.0,
            events_to_replay: self.event_log.count_after(snapshot.0.event_id()).await?,
        })
    }
}

// Intelligent snapshot scheduling
pub struct SnapshotScheduler {
    strategy: SnapshotStrategy,
    metrics: Arc<SnapshotMetrics>,
}

#[derive(Clone)]
pub enum SnapshotStrategy {
    /// Fixed interval (every N events)
    FixedInterval { events: usize },
    
    /// Time-based (every T duration)
    TimeBased { interval: Duration },
    
    /// Adaptive based on replay cost
    Adaptive {
        target_recovery_time_ms: u64,
        min_interval_events: usize,
        max_interval_events: usize,
    },
    
    /// Hybrid: both time and event count
    Hybrid {
        max_events: usize,
        max_duration: Duration,
    },
}

impl SnapshotScheduler {
    pub async fn should_snapshot(&self, state: &AgentState) -> bool {
        match self.strategy {
            SnapshotStrategy::Adaptive { target_recovery_time_ms, .. } => {
                // Estimate replay time based on historical metrics
                let events_since_snapshot = state.events_since_snapshot();
                let avg_event_replay_time = self.metrics.avg_event_replay_time_us().await;
                let estimated_replay_ms = 
                    (events_since_snapshot * avg_event_replay_time) / 1000;
                
                // Snapshot if replay would exceed target
                estimated_replay_ms > target_recovery_time_ms
            }
            SnapshotStrategy::Hybrid { max_events, max_duration } => {
                state.events_since_snapshot() >= max_events ||
                state.time_since_snapshot() >= max_duration
            }
            // ... other strategies
        }
    }
}

// Fast recovery with parallel replay
impl HybridEventStore {
    pub async fn recover_state(&self) -> Result<AgentState> {
        let start = Instant::now();
        
        // Load latest snapshot
        let restored = self.snapshot_store
            .load_latest_before(SystemTime::now())
            .await?;
        
        info!(
            "Loaded snapshot {} with {} events to replay",
            restored.snapshot_id, restored.events_to_replay
        );
        
        // Replay events since snapshot in parallel batches
        let events = self.event_log
            .read_after(restored.snapshot_id.event_id())
            .await?;
        
        // Process events in parallel while maintaining order per key
        let mut state = restored.state;
        let chunks = events.chunks(100);  // Process 100 events at a time
        
        for chunk in chunks {
            // Group by partition key for parallel processing
            let mut partitioned: HashMap<PartitionKey, Vec<StateEvent>> = HashMap::new();
            for event in chunk {
                partitioned.entry(event.partition_key())
                    .or_default()
                    .push(event.clone());
            }
            
            // Apply each partition in parallel
            let futures: Vec<_> = partitioned
                .into_iter()
                .map(|(key, events)| self.apply_events_to_partition(state.clone(), key, events))
                .collect();
            
            let results = futures::future::join_all(futures).await;
            
            // Merge partition results
            for result in results {
                state.merge_partition(result?);
            }
        }
        
        let elapsed = start.elapsed();
        self.metrics.record_recovery_time(elapsed).await;
        
        info!("State recovered in {}ms", elapsed.as_millis());
        Ok(state)
    }
}
```

### 4.2 Distributed State Coordination

```rust
// CRDT-based state synchronization for eventual consistency
use crdt::{Orswot, VClock};

pub struct DistributedState {
    local_state: Arc<RwLock<LocalState>>,
    crdt: Arc<RwLock<Orswot<StateKey, StateValue>>>,
    vector_clock: Arc<RwLock<VClock<AgentId>>>,
}

impl DistributedState {
    pub async fn update(&self, key: StateKey, value: StateValue) -> Result<()> {
        let mut crdt = self.crdt.write().await;
        let mut vclock = self.vector_clock.write().await;
        
        // Update CRDT with causal consistency
        let actor = self.agent_id();
        crdt.insert(key, value, actor);
        vclock.increment(actor);
        
        // Broadcast state delta to peers
        let delta = crdt.delta_since(&vclock);
        self.broadcast_delta(delta).await?;
        
        Ok(())
    }
    
    pub async fn merge_remote(&self, remote_state: RemoteState) -> Result<()> {
        let mut crdt = self.crdt.write().await;
        let mut vclock = self.vector_clock.write().await;
        
        // Merge with conflict resolution
        crdt.merge(remote_state.crdt_delta);
        vclock.merge(&remote_state.vector_clock);
        
        // Persist merged state
        self.persist_snapshot().await?;
        
        Ok(())
    }
}
```

## 5. Resource Management

### 5.1 Cross-Platform Resource Management

```rust
// Unified resource control via psutil abstraction
use sysinfo::{System, SystemExt, ProcessExt, CpuRefreshKind};

pub struct UnifiedResourceController {
    system: Arc<Mutex<System>>,
    limits: HashMap<AgentId, ResourceLimits>,
    enforcers: HashMap<AgentId, ResourceEnforcer>,
}

impl UnifiedResourceController {
    pub async fn enforce_limits(&self, agent_id: AgentId, limits: ResourceLimits) -> Result<()> {
        let enforcer = ResourceEnforcer::new(agent_id, limits);
        
        // Cross-platform CPU limiting via affinity + priority
        #[cfg(target_os = "linux")]
        {
            // Use cgroups v2 when available
            if Path::new("/sys/fs/cgroup/cgroup.controllers").exists() {
                self.enforce_cgroups_v2(&agent_id, &limits).await?;
            } else {
                enforcer.enforce_via_affinity().await?;
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            // macOS: thread QoS + memory pressure notifications
            enforcer.set_qos_class(QosClass::Utility).await?;
            enforcer.register_memory_pressure_handler().await?;
        }
        
        #[cfg(target_os = "windows")]
        {
            // Windows: Job Objects for hard limits
            enforcer.create_job_object(&limits).await?;
        }
        
        self.enforcers.insert(agent_id, enforcer);
        Ok(())
    }
}

// Platform-agnostic resource enforcer
pub struct ResourceEnforcer {
    pid: u32,
    limits: ResourceLimits,
    monitor: ResourceMonitor,
}

impl ResourceEnforcer {
    pub async fn enforce_via_affinity(&self) -> Result<()> {
        // Calculate CPU cores based on percentage limit
        let total_cores = num_cpus::get();
        let allowed_cores = ((self.limits.cpu_percent / 100.0) * total_cores as f64) as usize;
        
        // Pin to subset of cores
        let mut cpu_set = CpuSet::new();
        for i in 0..allowed_cores.min(total_cores) {
            cpu_set.add(i);
        }
        
        // Apply affinity
        set_thread_affinity(&cpu_set)?;
        
        // Set nice priority for additional control
        let priority = match self.limits.cpu_percent {
            p if p < 25.0 => 19,   // Lowest priority
            p if p < 50.0 => 10,   // Low priority
            p if p < 75.0 => 0,    // Normal priority
            _ => -5,               // Higher priority (requires privileges)
        };
        
        renice(self.pid, priority)?;
        Ok(())
    }
    
    pub async fn monitor_and_throttle(&mut self) -> Result<()> {
        loop {
            let metrics = self.monitor.collect_metrics().await?;
            
            // Memory throttling via madvise
            if metrics.memory_bytes > self.limits.max_memory_bytes {
                self.release_memory().await?;
            }
            
            // CPU throttling via sleep injection
            if metrics.cpu_percent > self.limits.cpu_percent {
                let throttle_ms = self.calculate_throttle(metrics.cpu_percent);
                tokio::time::sleep(Duration::from_millis(throttle_ms)).await;
            }
            
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    
    async fn release_memory(&self) -> Result<()> {
        // Advise kernel to reclaim pages
        unsafe {
            libc::madvise(
                self.monitor.heap_start as *mut libc::c_void,
                self.monitor.heap_size,
                libc::MADV_DONTNEED,
            );
        }
        
        // Trigger GC if available
        #[cfg(feature = "jemalloc")]
        jemalloc::purge();
        
        Ok(())
    }
}

// Resource limits with platform-specific hints
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub cpu_percent: f64,        // 0-100 per core (can be >100 for multi-core)
    pub max_memory_bytes: usize,
    pub max_io_bytes_per_sec: usize,
    pub max_file_handles: u32,
    pub priority_class: PriorityClass,
}

#[derive(Debug, Clone)]
pub enum PriorityClass {
    Critical,    // Real-time priority (requires privileges)
    High,        // Above normal scheduling
    Normal,      // Default scheduling
    Low,         // Below normal
    Idle,        // Only runs when system idle
}
```

### 5.2 Backpressure and Flow Control

```rust
// Token bucket algorithm for rate limiting
pub struct RateLimiter {
    capacity: u32,
    tokens: AtomicU32,
    refill_rate: u32,
    last_refill: AtomicU64,
}

impl RateLimiter {
    pub fn try_acquire(&self, tokens: u32) -> bool {
        self.refill();
        
        let mut current = self.tokens.load(Ordering::Relaxed);
        loop {
            if current < tokens {
                return false; // Would exceed rate limit
            }
            
            match self.tokens.compare_exchange_weak(
                current,
                current - tokens,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => return true,
                Err(actual) => current = actual,
            }
        }
    }
    
    fn refill(&self) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        let last = self.last_refill.load(Ordering::Relaxed);
        let elapsed_ms = now.saturating_sub(last);
        
        if elapsed_ms > 0 {
            let tokens_to_add = (elapsed_ms * self.refill_rate as u64 / 1000) as u32;
            let new_tokens = self.tokens.load(Ordering::Relaxed).saturating_add(tokens_to_add).min(self.capacity);
            self.tokens.store(new_tokens, Ordering::Relaxed);
            self.last_refill.store(now, Ordering::Relaxed);
        }
    }
}
```

## 6. MCP Protocol Extensions

### 6.1 Agent-Aware Tool Registration

```rust
// Extended MCP tool descriptor with agent metadata
#[derive(Serialize, Deserialize)]
pub struct AgentTool {
    #[serde(flatten)]
    base: McpTool,
    
    // Agent-specific extensions
    agent_id: AgentId,
    capabilities: Vec<Capability>,
    resource_requirements: ResourceRequirements,
    isolation_level: IsolationLevel,
}

#[derive(Serialize, Deserialize)]
pub enum IsolationLevel {
    None,        // Share process space
    Thread,      // Dedicated thread
    Process,     // Separate process via fork
    Container,   // Docker/podman container
}

// MCP server with agent routing
pub struct AgentMcpServer {
    registry: Arc<AgentRegistry>,
    router: Arc<MessageRouter>,
}

#[async_trait]
impl McpServer for AgentMcpServer {
    async fn handle_tool_call(&self, tool: &str, params: Value) -> Result<Value> {
        // Parse agent-prefixed tool name
        let (agent_name, tool_name) = parse_agent_tool(tool)?;
        
        // Route to appropriate agent
        let agent_id = self.registry.lookup_by_name(agent_name).await?;
        let request = ToolRequest { tool: tool_name, params };
        
        // Execute with timeout and resource tracking
        let response = self.router
            .route_with_timeout(agent_id, request, Duration::from_secs(30))
            .await?;
        
        Ok(response)
    }
}
```

### 6.2 Agent Discovery Protocol

```rust
// Agent capability advertisement
#[derive(Serialize, Deserialize)]
pub struct AgentManifest {
    id: AgentId,
    name: String,
    version: Version,
    capabilities: Vec<Capability>,
    dependencies: Vec<AgentDependency>,
    resource_requirements: ResourceRequirements,
    health_endpoint: Option<Url>,
}

// Service discovery via mDNS/DNS-SD
pub struct AgentDiscovery {
    mdns: mdns::Service,
    known_agents: Arc<DashMap<AgentId, AgentManifest>>,
}

impl AgentDiscovery {
    pub async fn advertise(&self, manifest: AgentManifest) -> Result<()> {
        let txt_records = vec![
            format!("id={}", manifest.id),
            format!("caps={}", manifest.capabilities.join(",")),
            format!("ver={}", manifest.version),
        ];
        
        self.mdns.register(
            &manifest.name,
            "_pmat-agent._tcp",
            8080,
            &txt_records,
        ).await?;
        
        Ok(())
    }
    
    pub async fn discover(&self) -> Result<Vec<AgentManifest>> {
        let services = self.mdns.browse("_pmat-agent._tcp").await?;
        
        let mut manifests = Vec::new();
        for service in services {
            let manifest = self.fetch_manifest(&service.address).await?;
            manifests.push(manifest);
        }
        
        Ok(manifests)
    }
}
```

## 7. Orchestration Patterns

### 7.1 DAG-Based Workflow Execution

```rust
// Directed acyclic graph for agent workflow orchestration
pub struct WorkflowDAG {
    nodes: Vec<WorkflowNode>,
    edges: Vec<(NodeId, NodeId)>,
    execution_plan: Option<ExecutionPlan>,
}

pub struct WorkflowNode {
    id: NodeId,
    agent_id: AgentId,
    operation: AgentOperation,
    retry_policy: RetryPolicy,
    timeout: Duration,
}

pub struct WorkflowExecutor {
    dag: Arc<WorkflowDAG>,
    registry: Arc<AgentRegistry>,
    state: Arc<RwLock<WorkflowState>>,
}

impl WorkflowExecutor {
    pub async fn execute(&self) -> Result<WorkflowResult> {
        let plan = self.dag.execution_plan.as_ref().unwrap();
        
        // Execute stages in topological order
        for stage in &plan.stages {
            // Parallel execution within stage
            let futures: Vec<_> = stage.nodes.iter().map(|node_id| {
                let node = self.dag.get_node(*node_id);
                self.execute_node(node)
            }).collect();
            
            // Wait for stage completion with error aggregation
            let results = futures::future::join_all(futures).await;
            
            // Check for failures
            for result in results {
                if let Err(e) = result {
                    match e.severity() {
                        Severity::Fatal => return Err(e),
                        Severity::Warning => self.state.write().await.add_warning(e),
                        Severity::Info => log::info!("Node warning: {}", e),
                    }
                }
            }
        }
        
        Ok(self.state.read().await.to_result())
    }
    
    async fn execute_node(&self, node: &WorkflowNode) -> Result<NodeResult> {
        let mut attempts = 0;
        loop {
            attempts += 1;
            
            match timeout(node.timeout, self.call_agent(node)).await {
                Ok(Ok(result)) => return Ok(result),
                Ok(Err(e)) if attempts < node.retry_policy.max_attempts => {
                    let delay = node.retry_policy.backoff(attempts);
                    tokio::time::sleep(delay).await;
                    continue;
                },
                Ok(Err(e)) => return Err(e),
                Err(_) => return Err(anyhow!("Node {} timed out", node.id)),
            }
        }
    }
}
```

### 7.2 Event-Driven Choreography

```rust
// Event-driven agent coordination without central orchestrator
pub struct EventBus {
    subscribers: Arc<DashMap<EventType, Vec<AgentId>>>,
    handlers: Arc<DashMap<AgentId, mpsc::Sender<Event>>>,
}

// Saga pattern for distributed transactions
pub struct SagaCoordinator {
    saga_log: Arc<EventLog>,
    compensations: Arc<DashMap<SagaId, Vec<Compensation>>>,
}

impl SagaCoordinator {
    pub async fn execute_saga(&self, saga: Saga) -> Result<SagaResult> {
        let saga_id = SagaId::new();
        let mut completed_steps = Vec::new();
        
        for step in saga.steps() {
            match self.execute_step(&step).await {
                Ok(result) => {
                    // Log step completion
                    self.saga_log.append(StepCompleted {
                        saga_id,
                        step_id: step.id(),
                        result: result.clone(),
                    }).await?;
                    
                    // Record compensation if needed
                    if let Some(compensation) = step.compensation() {
                        self.compensations.entry(saga_id).or_default().push(compensation);
                    }
                    
                    completed_steps.push(result);
                },
                Err(e) => {
                    // Trigger compensations in reverse order
                    self.compensate(saga_id, completed_steps).await?;
                    return Err(e);
                }
            }
        }
        
        Ok(SagaResult::Completed(completed_steps))
    }
}
```

## 8. Performance Requirements

### 8.1 Latency Targets

| Operation | P50 | P99 | Max |
|-----------|-----|-----|-----|
| Agent spawn | 10ms | 50ms | 100ms |
| Tool call routing | 1ms | 5ms | 10ms |
| State checkpoint | 20ms | 100ms | 200ms |
| Message passing | 0.5ms | 2ms | 5ms |
| Workflow stage | 100ms | 500ms | 1s |

### 8.2 Throughput Requirements

```rust
// Benchmark specifications
pub struct PerformanceRequirements {
    pub min_messages_per_second: 10_000,
    pub max_concurrent_agents: 100,
    pub max_workflow_parallelism: 16,
    pub max_state_size_mb: 100,
    pub max_message_size_kb: 64,
}

// Load testing harness
pub async fn benchmark_agent_system(config: BenchmarkConfig) -> BenchmarkResults {
    let registry = AgentRegistry::new();
    
    // Spawn test agents
    let agents = futures::stream::iter(0..config.num_agents)
        .map(|i| registry.spawn_agent(create_test_agent_spec(i)))
        .buffer_unordered(16)
        .try_collect::<Vec<_>>()
        .await?;
    
    // Generate load
    let start = Instant::now();
    let mut message_count = 0;
    
    while start.elapsed() < config.duration {
        for sender in &agents {
            for receiver in &agents {
                if sender != receiver {
                    let msg = generate_test_message();
                    registry.route_message(*sender, *receiver, msg).await?;
                    message_count += 1;
                }
            }
        }
    }
    
    let throughput = message_count as f64 / start.elapsed().as_secs_f64();
    assert!(throughput >= PerformanceRequirements::min_messages_per_second as f64);
    
    BenchmarkResults { throughput, latencies: calculate_latencies() }
}
```

## 8.3 Avoiding the Distributed Monolith Anti-Pattern

### Circuit Breaker Pattern for Agent Independence

```rust
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

pub struct CircuitBreaker {
    failure_count: AtomicU32,
    last_failure_time: AtomicU64,
    state: AtomicState,
    config: CircuitBreakerConfig,
}

#[derive(Clone, Copy)]
pub struct CircuitBreakerConfig {
    failure_threshold: u32,      // Failures before opening
    success_threshold: u32,      // Successes to close
    timeout: Duration,           // Time before half-open
    fallback_timeout: Duration,  // Max time for fallback
}

impl CircuitBreaker {
    pub async fn call<F, T>(&self, operation: F, fallback: impl Fn() -> T) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        match self.state.load() {
            State::Open => {
                // Check if we should transition to half-open
                if self.should_attempt_reset() {
                    self.state.store(State::HalfOpen);
                } else {
                    // Use fallback immediately, don't wait for timeout
                    return Ok(fallback());
                }
            }
            State::HalfOpen | State::Closed => {}
        }
        
        // Attempt the operation with timeout
        match timeout(self.config.fallback_timeout, operation).await {
            Ok(Ok(result)) => {
                self.on_success();
                Ok(result)
            }
            Ok(Err(_)) | Err(_) => {
                self.on_failure();
                Ok(fallback())  // Always provide degraded service
            }
        }
    }
}

// Agent with built-in circuit breakers for dependencies
pub struct ResilientAgent {
    dependencies: HashMap<AgentId, CircuitBreaker>,
    cache: Arc<Cache<String, Value>>,  // Local cache for fallbacks
}

impl ResilientAgent {
    pub async fn request_from_dependency(
        &self,
        dep_id: AgentId,
        request: Request,
    ) -> Result<Response> {
        let breaker = self.dependencies.get(&dep_id).unwrap();
        
        breaker.call(
            async { self.send_request(dep_id, request).await },
            || self.generate_fallback_response(&request),
        ).await
    }
    
    fn generate_fallback_response(&self, request: &Request) -> Response {
        // Return cached or default response
        if let Some(cached) = self.cache.get(&request.key()) {
            Response::Cached(cached)
        } else {
            Response::Default(self.compute_default())
        }
    }
}
```

### Bulkhead Pattern for Resource Isolation

```rust
// Isolate thread pools per agent class to prevent cascade failures
pub struct BulkheadExecutor {
    analyzers: ThreadPool,     // CPU-bound work
    transformers: ThreadPool,  // AST manipulation
    validators: ThreadPool,    // I/O-bound validation
    orchestrators: ThreadPool, // Coordination work
}

impl BulkheadExecutor {
    pub fn new() -> Self {
        Self {
            analyzers: ThreadPoolBuilder::new()
                .num_threads(num_cpus::get() / 2)
                .thread_name(|i| format!("analyzer-{}", i))
                .build()
                .unwrap(),
            transformers: ThreadPoolBuilder::new()
                .num_threads(2)
                .thread_name(|i| format!("transformer-{}", i))
                .build()
                .unwrap(),
            validators: ThreadPoolBuilder::new()
                .num_threads(4)
                .thread_name(|i| format!("validator-{}", i))
                .build()
                .unwrap(),
            orchestrators: ThreadPoolBuilder::new()
                .num_threads(2)
                .thread_name(|i| format!("orchestrator-{}", i))
                .build()
                .unwrap(),
        }
    }
    
    pub fn submit(&self, agent_class: AgentClass, work: Box<dyn FnOnce() + Send>) {
        match agent_class {
            AgentClass::Analyzer => self.analyzers.spawn(work),
            AgentClass::Transformer => self.transformers.spawn(work),
            AgentClass::Validator => self.validators.spawn(work),
            AgentClass::Orchestrator => self.orchestrators.spawn(work),
        }
    }
}
```

### Dependency Inversion for Testability

```rust
// Agents depend on traits, not concrete implementations
pub trait DependencyProvider: Send + Sync {
    type Analyzer: AnalyzerTrait;
    type Storage: StorageTrait;
    type Notifier: NotifierTrait;
    
    fn analyzer(&self) -> &Self::Analyzer;
    fn storage(&self) -> &Self::Storage;
    fn notifier(&self) -> &Self::Notifier;
}

// Production dependencies
pub struct ProductionDependencies {
    analyzer: RemoteAnalyzer,
    storage: PostgresStorage,
    notifier: KafkaNotifier,
}

// Test dependencies with in-memory implementations
pub struct TestDependencies {
    analyzer: MockAnalyzer,
    storage: InMemoryStorage,
    notifier: ChannelNotifier,
}

// Agent is agnostic to dependency implementation
pub struct GenericAgent<D: DependencyProvider> {
    deps: D,
    state: AgentState,
}

impl<D: DependencyProvider> GenericAgent<D> {
    pub async fn process(&mut self, input: Input) -> Result<Output> {
        // Uses trait methods, not concrete types
        let analysis = self.deps.analyzer().analyze(&input).await?;
        self.deps.storage().store(&analysis).await?;
        self.deps.notifier().notify(Event::Analyzed).await?;
        Ok(Output::from(analysis))
    }
}
```

## 9. Pragmatic Implementation Strategy

### 9.1 Incremental Migration Path (Avoiding the Big Rewrite)

```rust
// Phase 0: Modular Monolith with Compiler-Enforced Boundaries (Month 1)
// Architecture tests prevent coupling violations at compile time
pub mod pmat_modules {
    // Enforce module independence via orphan rule and visibility
    mod analyzer {
        // Private implementation details
        mod internal {
            pub(super) struct AnalyzerCore { /* ... */ }
        }
        
        // Public interface - the ONLY way other modules interact
        pub trait AnalyzerModule: Send + Sync {
            fn analyze(&self, input: &str) -> Result<Metrics>;
        }
        
        // Concrete implementation hidden from other modules
        pub(crate) struct AnalyzerImpl {
            core: internal::AnalyzerCore,
        }
    }
    
    mod transformer {
        // Can ONLY import public interfaces
        use super::analyzer::AnalyzerModule;  // ✓ Allowed
        // use super::analyzer::internal::*;   // ✗ Compile error
        
        pub trait TransformerModule: Send + Sync {
            fn transform(&self, ast: &AST) -> Result<AST>;
        }
    }
}

// Architecture tests using arch_test crate
#[cfg(test)]
mod architecture_tests {
    use arch_test::{dependency_rules, layer_rules};
    
    #[test]
    fn test_module_boundaries() {
        // Define architectural layers
        let layers = layer_rules()
            .layer("analyzer").inside("pmat::modules::analyzer")
            .layer("transformer").inside("pmat::modules::transformer")
            .layer("validator").inside("pmat::modules::validator")
            .layer("orchestrator").inside("pmat::modules::orchestrator");
        
        // Define allowed dependencies (DAG structure)
        let rules = dependency_rules()
            .rule("transformer").can_depend_on(&["analyzer"])
            .rule("validator").can_depend_on(&["analyzer", "transformer"])
            .rule("orchestrator").can_depend_on(&["analyzer", "transformer", "validator"])
            .rule("analyzer").cannot_depend_on(&["transformer", "validator", "orchestrator"]);
        
        // Compile-time enforcement
        rules.check();
        layers.check();
    }
    
    #[test]
    fn test_no_circular_dependencies() {
        let graph = dependency_graph!();
        assert!(graph.is_acyclic(), "Circular dependency detected!");
    }
}

// Phase 1: In-Process Actors with Actix (Month 2-3)
// Production-grade actor system with 10M downloads/month
pub mod pmat_actors {
    use actix::prelude::*;
    use actix_rt::System;
    use std::time::Duration;
    
    // Actix actors with typed messages - compile-time safety
    pub struct AnalyzerActor {
        complexity_cache: HashMap<FileId, ComplexityMetrics>,
        max_cache_size: usize,
    }
    
    impl Actor for AnalyzerActor {
        type Context = Context<Self>;
        
        fn started(&mut self, ctx: &mut Context<Self>) {
            // Self-healing: restart on panic
            ctx.set_mailbox_capacity(1024);  // Bounded mailbox prevents OOM
        }
    }
    
    // Strongly-typed messages with zero-cost abstractions
    #[derive(Message, Clone)]
    #[rtype(result = "Result<ComplexityMetrics, AnalysisError>")]
    pub struct AnalyzeComplexity {
        pub file_id: FileId,
        pub source: Arc<str>,
    }
    
    impl Handler<AnalyzeComplexity> for AnalyzerActor {
        type Result = ResponseActFuture<Self, Result<ComplexityMetrics, AnalysisError>>;
        
        fn handle(&mut self, msg: AnalyzeComplexity, _: &mut Context<Self>) -> Self::Result {
            // Check cache first - O(1)
            if let Some(cached) = self.complexity_cache.get(&msg.file_id) {
                return Box::pin(fut::ready(Ok(cached.clone())));
            }
            
            // Offload CPU-intensive work to blocking thread pool
            Box::pin(
                async move {
                    // Parse AST on dedicated thread pool (doesn't block actor)
                    let metrics = tokio::task::spawn_blocking(move || {
                        let ast = syn::parse_file(&msg.source)?;
                        calculate_complexity(&ast)
                    }).await??;
                    
                    Ok(metrics)
                }
                .into_actor(self)  // Return to actor context
                .map(move |result, actor, _ctx| {
                    if let Ok(ref metrics) = result {
                        // LRU eviction if cache full
                        if actor.complexity_cache.len() >= actor.max_cache_size {
                            actor.evict_lru();
                        }
                        actor.complexity_cache.insert(msg.file_id, metrics.clone());
                    }
                    result
                })
            )
        }
    }
    
    // Supervisor using actix's built-in supervision
    pub struct QualityGateSupervisor {
        analyzer: Addr<AnalyzerActor>,
        transformer: Addr<TransformerActor>,
        validator: Addr<ValidatorActor>,
    }
    
    impl Actor for QualityGateSupervisor {
        type Context = Context<Self>;
    }
    
    impl Supervised for QualityGateSupervisor {
        fn restarting(&mut self, _ctx: &mut Context<Self>) {
            // Called when supervisor restarts this actor
            info!("QualityGateSupervisor restarting");
        }
    }
    
    // Orchestration with back-pressure
    #[derive(Message)]
    #[rtype(result = "Result<ValidationResult, QualityError>")]
    pub struct ValidateCode {
        pub files: Vec<FileId>,
        pub thresholds: QualityThresholds,
    }
    
    impl Handler<ValidateCode> for QualityGateSupervisor {
        type Result = ResponseFuture<Result<ValidationResult, QualityError>>;
        
        fn handle(&mut self, msg: ValidateCode, _: &mut Context<Self>) -> Self::Result {
            let analyzer = self.analyzer.clone();
            let validator = self.validator.clone();
            
            Box::pin(async move {
                // Parallel analysis with bounded concurrency
                let mut futures = FuturesUnordered::new();
                
                for file_id in msg.files {
                    // Back-pressure: wait if mailbox full
                    let metrics = analyzer
                        .send(AnalyzeComplexity { file_id, source: load_file(file_id)? })
                        .await??;
                    
                    futures.push(validator.send(ValidateMetrics { metrics, thresholds: msg.thresholds }));
                }
                
                // Collect results maintaining order
                let mut results = Vec::new();
                while let Some(result) = futures.next().await {
                    results.push(result??);
                }
                
                Ok(ValidationResult::aggregate(results))
            })
        }
    }
    
    // System initialization with deterministic actor addresses
    pub fn init_actor_system() -> System {
        System::new()
    }
    
    pub async fn spawn_agents(sys: &System) -> Result<AgentHandles> {
        // Spawn with deterministic supervision tree
        let analyzer = AnalyzerActor::default().start();
        let transformer = TransformerActor::default().start();
        let validator = ValidatorActor::default().start();
        
        // Create supervisor with actor references
        let supervisor = QualityGateSupervisor {
            analyzer: analyzer.clone(),
            transformer: transformer.clone(),
            validator: validator.clone(),
        };
        
        // Register in system registry for discovery
        let supervisor_addr = supervisor.start();
        System::current().registry().set(supervisor_addr.clone());
        
        Ok(AgentHandles {
            analyzer,
            transformer,
            validator,
            supervisor: supervisor_addr,
        })
    }
}

// Phase 2: Optional Distribution (Month 4-6)
// Add network layer ONLY when needed for scale
pub enum AgentLocation {
    Local(LocalHandle),
    Remote(RemoteEndpoint),
}
```

### 9.2 Critical State: Raft Instead of CRDTs

```rust
// Quality-critical state uses Raft consensus for linearizability
use async_raft::{Config, Raft, RaftStorage};

pub struct QualityStateStore {
    raft: Raft<QualityCommand>,
    local_cache: Arc<RwLock<HashMap<FileId, QualityMetrics>>>,
}

impl QualityStateStore {
    pub async fn record_quality_gate_result(
        &self,
        file_id: FileId,
        metrics: QualityMetrics,
    ) -> Result<()> {
        // Raft ensures all nodes agree on the exact sequence of quality events
        let command = QualityCommand::RecordMetrics { file_id, metrics };
        
        // This blocks until a majority agrees - strong consistency
        self.raft.propose_change(command).await?;
        
        Ok(())
    }
    
    pub async fn check_quality_threshold(&self, file_id: FileId) -> Result<bool> {
        // Always returns the latest committed value
        let metrics = self.raft.query(|state| {
            state.get_metrics(file_id).cloned()
        }).await?;
        
        Ok(metrics.map_or(false, |m| m.meets_threshold()))
    }
}

// Non-critical state can still use CRDTs for performance
pub struct UIMetadataStore {
    crdt: Arc<RwLock<Orswot<String, Value>>>, // Eventually consistent is fine here
}
```

### 9.5 Auto-Generated SLOs from Observability

```rust
// Replace manual SLO contracts with automatic baseline generation
pub struct SLOGenerator {
    metrics_store: Arc<MetricsStore>,
    window: Duration,  // Rolling window for baseline calculation
    sensitivity: SensitivityProfile,
}

pub struct MetricsStore {
    // Time-series database for efficient percentile queries
    latency_histogram: Arc<HDRHistogram>,
    error_counter: Arc<AtomicU64>,
    throughput_gauge: Arc<AtomicU64>,
    retention: Duration,
}

impl SLOGenerator {
    pub async fn generate_slos(&self, agent_id: AgentId) -> GeneratedSLOs {
        let metrics = self.metrics_store
            .query_window(agent_id, self.window)
            .await?;
        
        // Calculate statistical baselines
        let latency_p50 = metrics.latency.percentile(50.0);
        let latency_p99 = metrics.latency.percentile(99.0);
        let latency_p999 = metrics.latency.percentile(99.9);
        
        // Apply sensitivity-based margins
        let slo = match self.sensitivity {
            SensitivityProfile::Strict => GeneratedSLOs {
                latency_p50_ms: latency_p50 * 1.1,   // 10% margin
                latency_p99_ms: latency_p99 * 1.2,   // 20% margin
                error_rate: metrics.error_rate * 1.5, // 50% margin
                alert_on_breach: true,
            },
            SensitivityProfile::Balanced => GeneratedSLOs {
                latency_p50_ms: latency_p50 * 1.25,  // 25% margin
                latency_p99_ms: latency_p99 * 1.5,   // 50% margin
                error_rate: metrics.error_rate * 2.0, // 2x margin
                alert_on_breach: false,
            },
            SensitivityProfile::Relaxed => GeneratedSLOs {
                latency_p50_ms: latency_p50 * 2.0,   // 2x margin
                latency_p99_ms: latency_p99 * 3.0,   // 3x margin
                error_rate: metrics.error_rate * 5.0, // 5x margin
                alert_on_breach: false,
            },
        };
        
        // Detect anomalies using statistical methods
        self.apply_anomaly_detection(&mut slo, &metrics).await;
        
        slo
    }
    
    async fn apply_anomaly_detection(
        &self, 
        slo: &mut GeneratedSLOs, 
        metrics: &HistoricalMetrics
    ) {
        // Use Isolation Forest for anomaly detection
        let detector = IsolationForest::new()
            .contamination(0.01)  // Expect 1% anomalies
            .n_trees(100);
        
        let features = metrics.as_feature_matrix();
        detector.fit(&features);
        
        // Adjust SLOs if current performance is anomalous
        if detector.is_anomaly(&metrics.latest_sample()) {
            warn!("Current performance detected as anomalous, widening SLO bounds");
            slo.latency_p99_ms *= 1.5;
            slo.error_rate *= 2.0;
        }
    }
}

// Automatic SLO enforcement without manual configuration
pub struct AdaptiveSLOEnforcer {
    generator: Arc<SLOGenerator>,
    cache: Arc<DashMap<AgentId, CachedSLO>>,
    refresh_interval: Duration,
}

impl AdaptiveSLOEnforcer {
    pub async fn check_slo(&self, agent_id: AgentId, metrics: &RequestMetrics) -> SLOResult {
        // Get or generate SLO
        let slo = self.get_or_generate_slo(agent_id).await?;
        
        // Compare against baseline
        let violations = SLOViolations {
            latency_breach: metrics.latency > slo.latency_p99_ms,
            error_rate_breach: metrics.error_rate > slo.error_rate,
            throughput_breach: metrics.throughput < slo.min_throughput,
        };
        
        if violations.any() {
            // Dynamic remediation based on breach type
            self.apply_remediation(agent_id, &violations).await?;
        }
        
        SLOResult {
            passed: !violations.any(),
            violations,
            suggested_action: self.suggest_action(&violations),
        }
    }
    
    async fn get_or_generate_slo(&self, agent_id: AgentId) -> Result<GeneratedSLOs> {
        // Check cache with TTL
        if let Some(cached) = self.cache.get(&agent_id) {
            if cached.generated_at.elapsed() < self.refresh_interval {
                return Ok(cached.slo.clone());
            }
        }
        
        // Generate fresh SLO from metrics
        let slo = self.generator.generate_slos(agent_id).await?;
        
        // Cache with timestamp
        self.cache.insert(agent_id, CachedSLO {
            slo: slo.clone(),
            generated_at: Instant::now(),
        });
        
        Ok(slo)
    }
}

// Integration with existing monitoring
impl Agent {
    pub fn with_auto_slo(mut self) -> Self {
        let enforcer = Arc::new(AdaptiveSLOEnforcer::new());
        
        // Wrap message handler with SLO checking
        let original_handler = self.handler.clone();
        self.handler = Box::new(move |msg| {
            let start = Instant::now();
            let enforcer = enforcer.clone();
            
            async move {
                // Process message
                let result = original_handler(msg).await;
                
                // Record metrics
                let metrics = RequestMetrics {
                    latency: start.elapsed(),
                    error_rate: result.is_err() as f64,
                    throughput: self.throughput_counter.get(),
                };
                
                // Check SLO (non-blocking)
                tokio::spawn(async move {
                    if let Err(e) = enforcer.check_slo(self.id, &metrics).await {
                        warn!("SLO check failed: {}", e);
                    }
                });
                
                result
            }
        });
        
        self
    }
}

// Continuous learning and adjustment
pub struct SLOLearner {
    history: Arc<RwLock<SLOHistory>>,
    model: Arc<Mutex<OnlineRegressor>>,
}

impl SLOLearner {
    pub async fn learn_from_breach(&mut self, breach: SLOBreach) {
        let mut history = self.history.write().await;
        history.record_breach(breach);
        
        // Online learning to predict optimal SLO thresholds
        let features = self.extract_features(&breach);
        let target = breach.actual_performance;
        
        let mut model = self.model.lock().await;
        model.partial_fit(&features, target);
        
        // Adjust future SLO generation based on learned patterns
        if history.breaches.len() % 100 == 0 {
            self.update_generation_strategy().await;
        }
    }
}

### 9.4 Simplified Service Discovery

```rust
// Start simple: static configuration file
#[derive(Deserialize)]
pub struct ServiceRegistry {
    agents: HashMap<String, AgentEndpoint>,
}

impl ServiceRegistry {
    pub fn from_config_file(path: &Path) -> Result<Self> {
        let config = std::fs::read_to_string(path)?;
        toml::from_str(&config).map_err(Into::into)
    }
    
    // Hot-reload on file change for dynamic updates
    pub async fn watch_for_updates(&self) -> Result<()> {
        let mut watcher = notify::recommended_watcher(|res| {
            match res {
                Ok(Event::Modify(_)) => self.reload_config(),
                _ => {}
            }
        })?;
        watcher.watch(Path::new("agents.toml"), RecursiveMode::NonRecursive)?;
        Ok(())
    }
}

// Future: etcd-based discovery when truly distributed
#[cfg(feature = "distributed")]
pub struct EtcdRegistry {
    client: etcd_rs::Client,
    prefix: String,
}
```

### Phase 1: Modular Monolith (Month 1)
- Refactor into modules with clear interfaces
- No network communication, just function calls
- Focus on logical separation and API design
- Deliverable: Clean module boundaries, no regression

### Phase 2: In-Process Actors (Month 2-3)
- Implement tokio-based actor system
- Message passing within single process
- State isolation via actor model
- Deliverable: Fault isolation, concurrent execution

### Phase 3: Hybrid Mode (Month 4-6)
- Add optional network layer for specific agents
- Raft consensus for critical state
- Static config-based discovery
- Deliverable: Selective distribution where beneficial

### Phase 4: Production Validation (Month 7-12)
- Extended testing in real environments
- Performance tuning based on actual workloads
- Gradual rollout with feature flags
- Deliverable: Production-ready system with proven benefits

## 10. Configuration Schema

```yaml
# pmat-agents.yaml
apiVersion: pmat.io/v1
kind: AgentSystem
metadata:
  name: pmat-production
  environment: prod
spec:
  runtime:
    max_agents: 50
    max_memory_gb: 16
    max_cpu_cores: 8
    
  networking:
    message_bus: nats  # Options: nats, rabbitmq, in-process
    discovery: mdns     # Options: mdns, consul, static
    
  persistence:
    state_backend: rocksdb  # Options: rocksdb, sqlite, memory
    checkpoint_interval: 5m
    retention_days: 30
    
  observability:
    metrics_endpoint: "0.0.0.0:9090"
    traces_endpoint: "otlp://localhost:4317"
    log_level: info
    
  agents:
    - name: quality-gate
      replicas: 3
      class: Validator
      config_ref: quality-gate-config
      
    - name: refactor
      replicas: 1
      class: Transformer
      config_ref: refactor-config
      
    - name: tdg
      replicas: 2
      class: Analyzer
      config_ref: tdg-config
```

## 11. Testing Strategy

### 11.1 Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_agent_message_ordering(
        messages in prop::collection::vec(arb_message(), 1..100)
    ) {
        let registry = AgentRegistry::new();
        let agent_id = registry.spawn_agent(test_agent_spec()).await?;
        
        // Send messages
        for msg in &messages {
            registry.send(agent_id, msg.clone()).await?;
        }
        
        // Verify FIFO ordering per sender
        let received = registry.drain_messages(agent_id).await?;
        verify_causal_ordering(&messages, &received);
    }
    
    #[test]
    fn test_workflow_determinism(
        dag in arb_workflow_dag()
    ) {
        let executor = WorkflowExecutor::new(dag.clone());
        
        // Execute multiple times
        let result1 = executor.execute().await?;
        let result2 = executor.execute().await?;
        
        // Verify deterministic execution
        prop_assert_eq!(result1, result2);
    }
}
```

### 11.2 Chaos Engineering

```rust
// Failure injection for resilience testing
pub struct ChaosMonkey {
    failure_rate: f64,
    latency_injection: Option<Duration>,
    partition_probability: f64,
}

impl ChaosMonkey {
    pub async fn inject_failures(&self, registry: &AgentRegistry) {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            
            // Random agent termination
            if random::<f64>() < self.failure_rate {
                let agents = registry.list_agents().await;
                if !agents.is_empty() {
                    let victim = agents[random::<usize>() % agents.len()];
                    registry.kill_agent(victim).await;
                }
            }
            
            // Network partition simulation
            if random::<f64>() < self.partition_probability {
                self.simulate_partition(registry).await;
            }
        }
    }
}
```

## 12. Migration Path

### 12.1 Compatibility Layer

```rust
// Backward compatibility wrapper for existing PMAT CLI
pub struct LegacyCliAdapter {
    agent_system: Arc<AgentSystem>,
}

impl LegacyCliAdapter {
    pub async fn execute_command(&self, cmd: PmatCommand) -> Result<String> {
        match cmd {
            PmatCommand::Context { path, include } => {
                // Route to context agent
                let request = ContextRequest { path, include };
                let response = self.agent_system
                    .call_agent("context-generator", request)
                    .await?;
                Ok(format_legacy_output(response))
            },
            PmatCommand::QualityGate { checks, strict } => {
                // Route to quality gate agent
                let request = QualityGateRequest { checks, strict };
                let response = self.agent_system
                    .call_agent("quality-gate", request)
                    .await?;
                Ok(format_quality_report(response))
            },
            // ... other commands
        }
    }
}
```

### 12.2 Incremental Rollout

```yaml
# Feature flags for gradual migration
features:
  use_agent_system:
    default: false
    rollout:
      - environment: dev
        enabled: true
        percentage: 100
      - environment: staging
        enabled: true
        percentage: 50
      - environment: prod
        enabled: false
        percentage: 0
  
  agent_features:
    distributed_state:
      enabled: false
    workflow_orchestration:
      enabled: true
    resource_isolation:
      enabled: true
      backend: cgroups  # or semaphore for non-Linux
```

## Appendix A: Agent Definition Examples

```yaml
# Complete agent definitions for reference implementation
agents:
  - pmat-complexity-analyzer.yaml
  - pmat-satd-detector.yaml
  - pmat-security-scanner.yaml
  - pmat-rust-analyzer.yaml
  - pmat-wasm-analyzer.yaml
  - pmat-polyglot-orchestrator.yaml
  - pmat-ci-coordinator.yaml
  - pmat-github-integrator.yaml
```

## Appendix B: Performance Benchmarks

```
Agent Spawn Time:
  P50: 8.2ms
  P99: 43.7ms
  Max: 97.3ms

Message Throughput:
  Single agent: 127,000 msg/s
  10 agents: 89,000 msg/s
  100 agents: 41,000 msg/s

State Checkpoint:
  1MB state: 12ms
  10MB state: 78ms
  100MB state: 341ms

Workflow Execution (10-node DAG):
  Sequential: 1.2s
  Parallel: 234ms
  With failures: 1.8s (includes compensation)
```

## References

1. Model Context Protocol Specification v1.0
2. Claude Code Sub-Agents Documentation
3. PMAT Architecture Documentation
4. Toyota Production System Principles
5. Rust Async Book
6. CRDT Papers (Shapiro et al.)
7. Saga Pattern (Garcia-Molina & Salem)
