<body>
    <header>
        <div class="container">
            <div class="header-content">
                <div class="logo">
                    🚀 PAIML MCP Agent Toolkit Demo
                    <span class="version">v{version}</span>
                </div>
                <div id="analysis-time" class="stat-unit"></div>
            </div>
        </div>
    </header>

    <main class="container">
        <!-- Progressive Loading Indicator -->
        <section class="section" id="progress-section">
            <h2 class="section-title">🚀 Analysis Progress</h2>
            <div class="progress-container" id="progress-container">
                <div class="progress-step">
                    <div class="progress-indicator pending" id="step-clone">1</div>
                    <div>
                        <div>Repository Cloning</div>
                        <small>Fetching source code from repository</small>
                    </div>
                </div>
                <div class="progress-step">
                    <div class="progress-indicator pending" id="step-language">2</div>
                    <div>
                        <div>Language Detection</div>
                        <small>Identifying programming languages and frameworks</small>
                    </div>
                </div>
                <div class="progress-step">
                    <div class="progress-indicator pending" id="step-ast">3</div>
                    <div>
                        <div>AST Parsing</div>
                        <small>Analyzing code structure and syntax</small>
                    </div>
                </div>
                <div class="progress-step">
                    <div class="progress-indicator pending" id="step-complexity">4</div>
                    <div>
                        <div>Complexity Analysis</div>
                        <small>Computing cyclomatic and cognitive complexity</small>
                    </div>
                </div>
                <div class="progress-step">
                    <div class="progress-indicator pending" id="step-dependencies">5</div>
                    <div>
                        <div>Dependency Graph</div>
                        <small>Building module and function relationships</small>
                    </div>
                </div>
            </div>
        </section>

        <!-- Language Ecosystem Overview -->
        <section class="section" id="languages-section" style="display: none;">
            <h2 class="section-title">🌐 Language Ecosystem</h2>
            <div id="language-badges"></div>
            <div class="stats-grid" id="language-stats"></div>
        </section>

        <!-- Enhanced Statistics with Interactivity -->
        <div class="stats-grid">
            <div class="stat-card interactive-card" onclick="showFunctionDetails()">
                <div class="stat-label">Total Functions</div>
                <div class="stat-value">
                    <span id="functions-count">{functions_analyzed}</span>
                    <span class="stat-unit">functions</span>
                </div>
                <button class="drill-down-button" style="margin-top: 0.5rem;">View Details</button>
            </div>
            <div class="stat-card interactive-card" onclick="showComplexityHeatmap()">
                <div class="stat-label">Average Complexity</div>
                <div class="stat-value">
                    <span id="avg-complexity">{avg_complexity:.2}</span>
                    <span class="stat-unit">cyclomatic</span>
                </div>
                <button class="drill-down-button" style="margin-top: 0.5rem;">View Heatmap</button>
            </div>
            <div class="stat-card interactive-card" onclick="showHotspotDetails()">
                <div class="stat-label">High Complexity</div>
                <div class="stat-value">
                    <span id="hotspot-count">{hotspot_functions}</span>
                    <span class="stat-unit">functions >20</span>
                </div>
                <button class="drill-down-button" style="margin-top: 0.5rem;">View Hotspots</button>
            </div>
            <div class="stat-card interactive-card" onclick="showQualityGates()">
                <div class="stat-label">Quality Score</div>
                <div class="stat-value">
                    <span id="quality-score">{quality_score:.1}</span>
                    <span class="stat-unit">/ 100</span>
                </div>
                <button class="drill-down-button" style="margin-top: 0.5rem;">View Gates</button>
            </div>
        </div>

        <!-- AI-Powered Repository Recommendations -->
        <section class="section">
            <h2 class="section-title">
                🤖 AI-Powered Repository Recommendations
            </h2>
            <div class="section-meta">
                <a href="/api/recommendations" target="_blank" class="endpoint-url">/api/recommendations</a>
                <div class="data-source">
                    <div class="data-indicator ai"></div>
                    <span>AI-Enhanced</span>
                </div>
            </div>
            <div id="recommendations-container">
                <div class="recommendation-grid" id="recommendations-grid">
                    <!-- Recommendations will be populated by JavaScript -->
                </div>
                <div class="learning-path-section" id="learning-path" style="display: none;">
                    <h3>🎯 Recommended Learning Path</h3>
                    <div class="learning-steps" id="learning-steps"></div>
                </div>
            </div>
        </section>

        <!-- Multi-Language Intelligence -->
        <section class="section" id="polyglot-section" style="display: none;">
            <h2 class="section-title">
                🌐 Multi-Language Project Intelligence
            </h2>
            <div class="section-meta">
                <a href="/api/polyglot" target="_blank" class="endpoint-url">/api/polyglot</a>
                <div class="data-source">
                    <div class="data-indicator polyglot"></div>
                    <span>Cross-Language</span>
                </div>
            </div>
            <div id="polyglot-container">
                <div class="language-breakdown" id="language-breakdown"></div>
                <div class="integration-points" id="integration-points"></div>
                <div class="architecture-pattern" id="architecture-pattern"></div>
            </div>
        </section>

        <section class="section">
            <h2 class="section-title">
                ⚡ Performance Breakdown
            </h2>
            <div class="section-meta">
                <a href="/api/summary" target="_blank" class="endpoint-url">/api/summary</a>
                <div class="data-source">
                    <div class="data-indicator dynamic"></div>
                    <span>Dynamic</span>
                </div>
            </div>
            <div class="timing-chart">
                <div class="timing-bar">
                    <div class="timing-label">Context Analysis</div>
                    <div class="timing-value"><span id="time-context">{time_context}</span>ms</div>
                    <div class="timing-progress">
                        <div class="timing-fill" id="bar-context" style="width: {context_percent}%"></div>
                    </div>
                </div>
                <div class="timing-bar">
                    <div class="timing-label">Complexity Analysis</div>
                    <div class="timing-value"><span id="time-complexity">{time_complexity}</span>ms</div>
                    <div class="timing-progress">
                        <div class="timing-fill" id="bar-complexity" style="width: {complexity_percent}%"></div>
                    </div>
                </div>
                <div class="timing-bar">
                    <div class="timing-label">DAG Generation</div>
                    <div class="timing-value"><span id="time-dag">{time_dag}</span>ms</div>
                    <div class="timing-progress">
                        <div class="timing-fill" id="bar-dag" style="width: {dag_percent}%"></div>
                    </div>
                </div>
                <div class="timing-bar">
                    <div class="timing-label">Churn Analysis</div>
                    <div class="timing-value"><span id="time-churn">{time_churn}</span>ms</div>
                    <div class="timing-progress">
                        <div class="timing-fill" id="bar-churn" style="width: {churn_percent}%"></div>
                    </div>
                </div>
            </div>
        </section>

        <section class="section">
            <h2 class="section-title">
                🔥 Complexity Hotspots
            </h2>
            <div class="section-meta">
                <a href="/api/hotspots" target="_blank" class="endpoint-url">/api/hotspots</a>
                <div class="data-source">
                    <div class="data-indicator default"></div>
                    <span>Default</span>
                </div>
            </div>
            <div id="hotspots-table"></div>
        </section>

        <section class="section">
            <h2 class="section-title">
                📊 Interactive Dependency Graph
            </h2>
            <div class="section-meta">
                <a href="/api/dag" target="_blank" class="endpoint-url">/api/dag</a>
                <div class="data-source">
                    <div class="data-indicator dynamic"></div>
                    <span>Interactive</span>
                </div>
            </div>
            
            <!-- Graph Controls -->
            <div class="graph-controls">
                <button class="filter-button active" data-filter="all">All Modules</button>
                <button class="filter-button" data-filter="rust">Rust Only</button>
                <button class="filter-button" data-filter="python">Python Only</button>
                <button class="filter-button" data-filter="javascript">JavaScript Only</button>
                <button class="filter-button" data-filter="high-complexity">High Complexity</button>
                <button class="filter-button" data-filter="circular">Circular Dependencies</button>
            </div>
            
            <div id="mermaid-container">
                <div class="loading">Loading interactive dependency graph...</div>
            </div>
        </section>

        <section class="section">
            <h2 class="section-title">
                📊 File Defect Report
            </h2>
            <div class="section-meta">
                <a href="/api/analysis" target="_blank" class="endpoint-url">/api/analysis</a>
                <div class="data-source">
                    <div class="data-indicator dynamic"></div>
                    <span>Dynamic</span>
                </div>
            </div>
            <div id="file-grid-container">
                <div id="file-grid"></div>
            </div>
        </section>

        <!-- Complexity Heatmap Section -->
        <section class="section" id="heatmap-section" style="display: none;">
            <h2 class="section-title">🔥 Complexity Heatmap</h2>
            <div class="heatmap-container" id="complexity-heatmap">
                <!-- Heatmap cells will be generated by JavaScript -->
            </div>
        </section>
    </main>

    <!-- Function Details Modal -->
    <div class="modal" id="function-modal">
        <div class="modal-content">
            <div class="modal-header">
                <h2 class="modal-title" id="modal-title">Function Details</h2>
                <button class="modal-close" onclick="closeModal()">&times;</button>
            </div>
            <div id="modal-body">
                <!-- Modal content will be populated by JavaScript -->
            </div>
        </div>
    </div>

    <!-- Quality Gates Modal -->
    <div class="modal" id="quality-modal">
        <div class="modal-content">
            <div class="modal-header">
                <h2 class="modal-title">Quality Gates Report</h2>
                <button class="modal-close" onclick="closeModal()">&times;</button>
            </div>
            <div id="quality-modal-body">
                <!-- Quality gates content will be populated by JavaScript -->
            </div>
        </div>
    </div>

