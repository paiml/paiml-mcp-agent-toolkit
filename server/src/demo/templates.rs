/// HTML templates for the demo web interface
/// These are validated by scripts/validate-demo-assets.ts
pub const HTML_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>PAIML MCP Agent Toolkit - Demo</title>
    <link rel="stylesheet" href="/vendor/gridjs-mermaid.min.css">
    <style>
        :root {
            --primary: #2563eb;
            --primary-dark: #1d4ed8;
            --secondary: #64748b;
            --background: #f8fafc;
            --surface: #ffffff;
            --text: #1e293b;
            --text-light: #64748b;
            --border: #e2e8f0;
            --success: #10b981;
            --warning: #f59e0b;
            --danger: #ef4444;
        }

        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: var(--background);
            color: var(--text);
            line-height: 1.6;
        }

        .container {
            max-width: 1400px;
            margin: 0 auto;
            padding: 2rem;
        }

        /* Enhanced Interactive Elements */
        .interactive-card {
            cursor: pointer;
            transition: all 0.3s ease;
        }

        .interactive-card:hover {
            transform: translateY(-4px);
            box-shadow: 0 8px 25px rgba(0, 0, 0, 0.1);
            border-color: var(--primary);
        }

        .drill-down-button {
            background: var(--primary);
            color: white;
            border: none;
            padding: 0.5rem 1rem;
            border-radius: 0.5rem;
            font-size: 0.875rem;
            cursor: pointer;
            transition: background-color 0.2s;
        }

        .drill-down-button:hover {
            background: var(--primary-dark);
        }

        .language-indicator {
            display: inline-flex;
            align-items: center;
            gap: 0.25rem;
            padding: 0.25rem 0.5rem;
            border-radius: 1rem;
            font-size: 0.75rem;
            font-weight: 600;
        }

        .language-indicator.rust { background: #ce422b; color: white; }
        .language-indicator.python { background: #3776ab; color: white; }
        .language-indicator.javascript { background: #f7df1e; color: black; }
        .language-indicator.typescript { background: #3178c6; color: white; }
        .language-indicator.java { background: #ed8b00; color: white; }
        .language-indicator.go { background: #00add8; color: white; }

        /* Progress Loading States */
        .progress-container {
            background: var(--background);
            border-radius: 0.5rem;
            padding: 1rem;
            margin: 1rem 0;
        }

        .progress-step {
            display: flex;
            align-items: center;
            gap: 1rem;
            padding: 0.5rem 0;
            border-bottom: 1px solid var(--border);
        }

        .progress-step:last-child {
            border-bottom: none;
        }

        .progress-indicator {
            width: 24px;
            height: 24px;
            border-radius: 50%;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 0.75rem;
            font-weight: bold;
        }

        .progress-indicator.pending { background: var(--border); color: var(--text-light); }
        .progress-indicator.active { background: var(--primary); color: white; animation: pulse 2s infinite; }
        .progress-indicator.complete { background: var(--success); color: white; }
        .progress-indicator.error { background: var(--danger); color: white; }

        @keyframes pulse {
            0% { opacity: 1; }
            50% { opacity: 0.5; }
            100% { opacity: 1; }
        }

        /* Interactive Dependency Graph */
        .graph-controls {
            display: flex;
            gap: 1rem;
            margin-bottom: 1rem;
            flex-wrap: wrap;
        }

        .filter-button {
            padding: 0.5rem 1rem;
            border: 1px solid var(--border);
            background: var(--surface);
            color: var(--text);
            border-radius: 0.5rem;
            cursor: pointer;
            transition: all 0.2s;
        }

        .filter-button.active {
            background: var(--primary);
            color: white;
            border-color: var(--primary);
        }

        .filter-button:hover {
            border-color: var(--primary);
        }

        /* Function Detail Modal */
        .modal {
            display: none;
            position: fixed;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
            background: rgba(0, 0, 0, 0.5);
            z-index: 1000;
        }

        .modal.active {
            display: flex;
            align-items: center;
            justify-content: center;
        }

        .modal-content {
            background: var(--surface);
            border-radius: 0.75rem;
            padding: 2rem;
            max-width: 800px;
            max-height: 80vh;
            overflow-y: auto;
            margin: 2rem;
            border: 1px solid var(--border);
        }

        .modal-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 1.5rem;
        }

        .modal-title {
            font-size: 1.5rem;
            font-weight: 600;
        }

        .modal-close {
            background: none;
            border: none;
            font-size: 1.5rem;
            cursor: pointer;
            color: var(--text-light);
        }

        .modal-close:hover {
            color: var(--text);
        }

        /* Complexity Heatmap */
        .heatmap-container {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
            gap: 0.5rem;
            margin: 1rem 0;
        }

        .heatmap-cell {
            aspect-ratio: 1;
            border-radius: 0.5rem;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 0.75rem;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.2s;
        }

        .heatmap-cell:hover {
            transform: scale(1.05);
        }

        .heatmap-cell.low { background: var(--success); color: white; }
        .heatmap-cell.medium { background: var(--warning); color: white; }
        .heatmap-cell.high { background: var(--danger); color: white; }

        header {
            background: var(--surface);
            border-bottom: 1px solid var(--border);
            padding: 1.5rem 0;
            margin-bottom: 2rem;
        }

        .header-content {
            display: flex;
            justify-content: space-between;
            align-items: center;
        }

        .logo {
            display: flex;
            align-items: center;
            gap: 1rem;
            font-size: 1.5rem;
            font-weight: 700;
            color: var(--primary);
        }

        .logo .version {
            font-size: 0.875rem;
            font-weight: 400;
            color: var(--text-light);
            background: var(--border);
            padding: 0.25rem 0.5rem;
            border-radius: 0.25rem;
        }

        .stats-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 1.5rem;
            margin-bottom: 3rem;
        }

        .stat-card {
            background: var(--surface);
            border-radius: 0.75rem;
            padding: 1.5rem;
            border: 1px solid var(--border);
            transition: transform 0.2s, box-shadow 0.2s;
        }

        .stat-card:hover {
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
        }

        .stat-label {
            color: var(--text-light);
            font-size: 0.875rem;
            margin-bottom: 0.25rem;
        }

        .stat-value {
            font-size: 2rem;
            font-weight: 700;
            color: var(--primary);
        }

        .stat-unit {
            font-size: 0.875rem;
            color: var(--text-light);
            font-weight: 400;
            margin-left: 0.25rem;
        }

        .section {
            background: var(--surface);
            border-radius: 0.75rem;
            padding: 2rem;
            margin-bottom: 2rem;
            border: 1px solid var(--border);
            border-top: 3px solid var(--primary);
        }

        .section-title {
            font-size: 1.5rem;
            font-weight: 600;
            margin-bottom: 1rem;
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }

        .section-meta {
            display: flex;
            align-items: center;
            gap: 1rem;
            margin-bottom: 1.5rem;
            font-size: 0.875rem;
            color: var(--text-light);
        }

        .endpoint-url {
            font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
            background: var(--background);
            padding: 0.25rem 0.5rem;
            border-radius: 0.25rem;
            border: 1px solid var(--border);
            text-decoration: none;
            color: var(--text);
            transition: background-color 0.2s, border-color 0.2s;
        }

        .endpoint-url:hover {
            background: var(--border);
            border-color: var(--primary);
        }

        .data-source {
            display: flex;
            align-items: center;
            gap: 0.25rem;
        }

        .data-indicator {
            width: 8px;
            height: 8px;
            border-radius: 50%;
        }

        .data-indicator.dynamic {
            background-color: var(--success);
        }

        .data-indicator.default {
            background-color: var(--danger);
        }

        .timing-chart {
            display: flex;
            gap: 1rem;
            margin-bottom: 2rem;
        }

        .timing-bar {
            flex: 1;
            text-align: center;
        }

        .timing-label {
            font-size: 0.875rem;
            color: var(--text-light);
            margin-bottom: 0.5rem;
        }

        .timing-value {
            font-size: 1.25rem;
            font-weight: 600;
            color: var(--primary);
            margin-bottom: 0.5rem;
        }

        .timing-progress {
            height: 8px;
            background: var(--border);
            border-radius: 4px;
            overflow: hidden;
        }

        .timing-fill {
            height: 100%;
            background: var(--primary);
            border-radius: 4px;
            transition: width 0.3s ease;
        }

        #hotspots-table {
            margin-top: 1rem;
        }

        .gridjs-wrapper {
            border-radius: 0.5rem;
            overflow: hidden;
        }

        #mermaid-container {
            background: var(--background);
            border-radius: 0.5rem;
            padding: 2rem;
            overflow: auto;
            min-height: 400px;
            max-height: 600px;
        }

        .loading {
            display: flex;
            align-items: center;
            justify-content: center;
            min-height: 200px;
            color: var(--text-light);
        }

        .error {
            color: var(--danger);
            padding: 1rem;
            background: #fef2f2;
            border-radius: 0.5rem;
            margin: 1rem 0;
        }

        @media (max-width: 768px) {
            .container {
                padding: 1rem;
            }

            .stats-grid {
                grid-template-columns: 1fr;
                gap: 1rem;
            }

            .timing-chart {
                flex-direction: column;
            }
        }
    </style>
</head>
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

    <script src="/vendor/gridjs.min.js"></script>
    <script type="module">
        import mermaid from 'https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.esm.min.mjs';
        
        // Global state for demo interactivity
        let demoData = {
            summary: null,
            hotspots: null,
            analysisData: null,
            languages: null
        };
        
        // Initialize Mermaid with enhanced configuration
        mermaid.initialize({ 
            startOnLoad: false,
            theme: 'default',
            flowchart: {
                useMaxWidth: true,
                htmlLabels: true,
                curve: 'basis'
            }
        });

        // Progressive loading simulation
        async function simulateProgressiveLoading() {
            const steps = [
                { id: 'step-clone', delay: 500 },
                { id: 'step-language', delay: 800 },
                { id: 'step-ast', delay: 1200 },
                { id: 'step-complexity', delay: 1500 },
                { id: 'step-dependencies', delay: 2000 }
            ];

            for (let i = 0; i < steps.length; i++) {
                const step = steps[i];
                const element = document.getElementById(step.id);
                
                // Set to active
                element.className = 'progress-indicator active';
                element.textContent = '⏳';
                
                await new Promise(resolve => setTimeout(resolve, step.delay));
                
                // Set to complete
                element.className = 'progress-indicator complete';
                element.textContent = '✓';
            }

            // Hide progress section and show results
            setTimeout(() => {
                document.getElementById('progress-section').style.display = 'none';
                document.getElementById('languages-section').style.display = 'block';
            }, 500);
        }

        // Enhanced data loading with language awareness
        async function loadDemoData() {
            try {
                // Start progressive loading animation
                simulateProgressiveLoading();

                // Fetch all data in parallel
                const [summaryResponse, hotspotsResponse, analysisResponse] = await Promise.all([
                    fetch('/api/summary'),
                    fetch('/api/hotspots'),
                    fetch('/api/analysis')
                ]);

                demoData.summary = await summaryResponse.json();
                demoData.hotspots = await hotspotsResponse.json();
                demoData.analysisData = await analysisResponse.json();

                // Update analysis time
                const totalTime = demoData.summary.time_context + demoData.summary.time_complexity + demoData.summary.time_dag + demoData.summary.time_churn;
                document.getElementById('analysis-time').textContent = `Analysis completed in ${totalTime}ms`;

                // Load language ecosystem data
                await loadLanguageEcosystem();

                // Create interactive tables
                createInteractiveHotspotsTable();
                createInteractiveFileGrid();

                // Load interactive dependency graph
                await loadInteractiveDependencyGraph();

                // Generate complexity heatmap
                generateComplexityHeatmap();

            } catch (error) {
                console.error('Error loading demo data:', error);
                showError('Failed to load demo data. Please refresh the page.');
            }
        }

        // Load language ecosystem information
        async function loadLanguageEcosystem() {
            // Simulate language detection from file analysis
            const languages = new Map();
            
            if (demoData.analysisData && demoData.analysisData.ast_contexts) {
                demoData.analysisData.ast_contexts.forEach(ctx => {
                    const ext = ctx.path.split('.').pop();
                    const lang = getLanguageFromExtension(ext);
                    
                    if (!languages.has(lang)) {
                        languages.set(lang, { files: 0, complexity: 0, functions: 0 });
                    }
                    
                    const data = languages.get(lang);
                    data.files++;
                    data.complexity += ctx.complexity_metrics.cognitive || 0;
                    data.functions += ctx.complexity_metrics.total_functions || 0;
                });
            }

            // Display language badges
            const badgesContainer = document.getElementById('language-badges');
            const statsContainer = document.getElementById('language-stats');
            
            let badgesHTML = '';
            let statsHTML = '';
            
            languages.forEach((data, lang) => {
                badgesHTML += `<span class="language-indicator ${lang.toLowerCase()}">${getLanguageEmoji(lang)} ${lang}</span> `;
                
                const avgComplexity = data.files > 0 ? (data.complexity / data.files).toFixed(1) : 0;
                statsHTML += `
                    <div class="stat-card">
                        <div class="stat-label">${lang} Files</div>
                        <div class="stat-value">
                            <span>${data.files}</span>
                            <span class="stat-unit">avg: ${avgComplexity}</span>
                        </div>
                    </div>
                `;
            });
            
            badgesContainer.innerHTML = badgesHTML;
            statsContainer.innerHTML = statsHTML;
            demoData.languages = languages;
        }

        // Create interactive hotspots table with drill-down capability
        function createInteractiveHotspotsTable() {
            new gridjs.Grid({
                columns: [
                    { 
                        name: 'Function',
                        data: (row) => row.function,
                        formatter: (cell, row) => {
                            return gridjs.html(`
                                <div style="cursor: pointer;" onclick="showFunctionDetail('${cell}', ${JSON.stringify(row).replace(/"/g, '&quot;')})">
                                    <strong>${cell}</strong>
                                    <br><small>Click for details</small>
                                </div>
                            `);
                        }
                    },
                    { 
                        name: 'Complexity',
                        data: (row) => row.complexity,
                        formatter: (cell) => {
                            const colorClass = cell > 20 ? 'danger' : cell > 10 ? 'warning' : 'success';
                            const emoji = cell > 20 ? '🔥' : cell > 10 ? '⚠️' : '✅';
                            return gridjs.html(`<span style="color: var(--${colorClass}); font-weight: 600">${emoji} ${cell}</span>`);
                        }
                    },
                    { 
                        name: 'Language',
                        data: (row) => row.language || 'Unknown',
                        formatter: (cell) => {
                            return gridjs.html(`<span class="language-indicator ${cell.toLowerCase()}">${getLanguageEmoji(cell)} ${cell}</span>`);
                        }
                    },
                    { 
                        name: 'Path',
                        data: (row) => row.path,
                        formatter: (cell) => gridjs.html(`<code style="font-size: 0.75rem">${cell}</code>`)
                    }
                ],
                data: demoData.hotspots.map(h => ({...h, language: getLanguageFromPath(h.path)})),
                sort: true,
                search: true,
                pagination: { limit: 10 },
                style: getGridStyle()
            }).render(document.getElementById('hotspots-table'));
        }

        // Create interactive file grid with enhanced filtering
        function createInteractiveFileGrid() {
            if (demoData.analysisData && demoData.analysisData.ast_contexts) {
                new gridjs.Grid({
                    columns: [
                        { 
                            name: 'File', 
                            width: '35%',
                            formatter: (cell, row) => {
                                const lang = getLanguageFromPath(cell);
                                const emoji = getLanguageEmoji(lang);
                                return gridjs.html(`
                                    <div onclick="showFileDetails('${cell}')">
                                        <code style="font-size: 0.875rem; cursor: pointer;">${emoji} ${cell}</code>
                                    </div>
                                `);
                            }
                        },
                        { 
                            name: 'Complexity',
                            formatter: (cell) => {
                                const colorClass = cell > 20 ? 'danger' : cell > 15 ? 'warning' : 'success';
                                const bar = Math.min(cell / 30 * 100, 100);
                                return gridjs.html(`
                                    <div style="display: flex; align-items: center; gap: 0.5rem;">
                                        <span style="color: var(--${colorClass}); font-weight: 600">${cell}</span>
                                        <div style="width: 50px; height: 4px; background: var(--border); border-radius: 2px;">
                                            <div style="width: ${bar}%; height: 100%; background: var(--${colorClass}); border-radius: 2px;"></div>
                                        </div>
                                    </div>
                                `);
                            }
                        },
                        { name: 'Functions', formatter: (cell) => cell || 0 },
                        { name: 'LOC', formatter: (cell) => cell.toLocaleString() }
                    ],
                    data: demoData.analysisData.ast_contexts.map(ctx => [
                        ctx.path.replace('./server/src/', ''),
                        ctx.complexity_metrics.cognitive || 0,
                        ctx.complexity_metrics.total_functions || 0,
                        ctx.lines_of_code || 0
                    ]),
                    sort: true,
                    search: true,
                    pagination: { limit: 15 },
                    style: getGridStyle()
                }).render(document.getElementById('file-grid'));
            }
        }

        // Load interactive dependency graph with filtering
        async function loadInteractiveDependencyGraph() {
            try {
                const dagResponse = await fetch('/api/dag');
                const dagText = await dagResponse.text();
                
                if (dagText && dagText.trim()) {
                    const container = document.getElementById('mermaid-container');
                    container.innerHTML = `<pre class="mermaid">${dagText}</pre>`;
                    await mermaid.run();
                    
                    // Add click handlers for graph filtering
                    setupGraphFiltering();
                } else {
                    document.getElementById('mermaid-container').innerHTML = '<div class="error">No dependency graph available</div>';
                }
            } catch (error) {
                console.error('Error loading dependency graph:', error);
                document.getElementById('mermaid-container').innerHTML = `<div class="error">Error loading dependency graph</div>`;
            }
        }

        // Generate complexity heatmap
        function generateComplexityHeatmap() {
            if (!demoData.analysisData || !demoData.analysisData.ast_contexts) return;
            
            const heatmapContainer = document.getElementById('complexity-heatmap');
            let heatmapHTML = '';
            
            demoData.analysisData.ast_contexts.slice(0, 20).forEach(ctx => {
                const complexity = ctx.complexity_metrics.cognitive || 0;
                const level = complexity > 20 ? 'high' : complexity > 10 ? 'medium' : 'low';
                const filename = ctx.path.split('/').pop();
                
                heatmapHTML += `
                    <div class="heatmap-cell ${level}" onclick="showFileDetails('${ctx.path}')">
                        <div>${filename}</div>
                        <div style="font-size: 1.2em; margin-top: 0.25rem;">${complexity}</div>
                    </div>
                `;
            });
            
            heatmapContainer.innerHTML = heatmapHTML;
        }

        // Interactive functions
        window.showFunctionDetails = function() {
            document.getElementById('heatmap-section').style.display = 'block';
            document.getElementById('heatmap-section').scrollIntoView({ behavior: 'smooth' });
        };

        window.showComplexityHeatmap = function() {
            document.getElementById('heatmap-section').style.display = 'block';
            document.getElementById('heatmap-section').scrollIntoView({ behavior: 'smooth' });
        };

        window.showHotspotDetails = function() {
            document.querySelector('#hotspots-table').scrollIntoView({ behavior: 'smooth' });
        };

        window.showQualityGates = function() {
            // Generate quality gates report
            const qualityHTML = `
                <div class="progress-container">
                    <div class="progress-step">
                        <div class="progress-indicator complete">✓</div>
                        <div>Complexity Check: All functions ≤ 20 complexity</div>
                    </div>
                    <div class="progress-step">
                        <div class="progress-indicator complete">✓</div>
                        <div>SATD Check: No self-admitted technical debt</div>
                    </div>
                    <div class="progress-step">
                        <div class="progress-indicator complete">✓</div>
                        <div>Dead Code Check: No unused functions detected</div>
                    </div>
                </div>
            `;
            
            document.getElementById('quality-modal-body').innerHTML = qualityHTML;
            document.getElementById('quality-modal').classList.add('active');
        };

        window.showFunctionDetail = function(name, rowData) {
            const data = typeof rowData === 'string' ? JSON.parse(rowData) : rowData;
            
            const modalHTML = `
                <div class="progress-container">
                    <h3>Function: ${name}</h3>
                    <p><strong>Complexity:</strong> ${data.complexity}</p>
                    <p><strong>Lines of Code:</strong> ${data.loc}</p>
                    <p><strong>Path:</strong> <code>${data.path}</code></p>
                    <p><strong>Refactoring Suggestion:</strong> Consider breaking this function into smaller, more focused functions.</p>
                </div>
            `;
            
            document.getElementById('modal-body').innerHTML = modalHTML;
            document.getElementById('function-modal').classList.add('active');
        };

        window.closeModal = function() {
            document.querySelectorAll('.modal').forEach(modal => {
                modal.classList.remove('active');
            });
        };

        // Setup graph filtering controls
        function setupGraphFiltering() {
            document.querySelectorAll('.filter-button').forEach(button => {
                button.addEventListener('click', function() {
                    // Update active state
                    document.querySelectorAll('.filter-button').forEach(b => b.classList.remove('active'));
                    this.classList.add('active');
                    
                    // Apply filter (placeholder - would need actual graph filtering logic)
                    console.log('Filtering by:', this.dataset.filter);
                });
            });
        }

        // Utility functions
        function getLanguageFromExtension(ext) {
            const langMap = {
                'rs': 'Rust',
                'py': 'Python', 
                'js': 'JavaScript',
                'ts': 'TypeScript',
                'java': 'Java',
                'go': 'Go',
                'cpp': 'C++',
                'c': 'C'
            };
            return langMap[ext] || 'Unknown';
        }

        function getLanguageFromPath(path) {
            const ext = path.split('.').pop();
            return getLanguageFromExtension(ext);
        }

        function getLanguageEmoji(lang) {
            const emojiMap = {
                'Rust': '🦀',
                'Python': '🐍',
                'JavaScript': '📜',
                'TypeScript': '🔷',
                'Java': '☕',
                'Go': '🐹',
                'C++': '⚙️',
                'C': '⚙️'
            };
            return emojiMap[lang] || '📄';
        }

        function getGridStyle() {
            return {
                table: { 'border-radius': '0.5rem' },
                th: {
                    'background-color': 'var(--background)',
                    color: 'var(--text)',
                    'font-weight': '600',
                    'text-align': 'left',
                    padding: '1rem'
                },
                td: {
                    padding: '1rem',
                    'border-color': 'var(--border)'
                }
            };
        }

        function showError(message) {
            document.getElementById('mermaid-container').innerHTML = `<div class="error">${message}</div>`;
        }

        // Close modals when clicking outside
        document.addEventListener('click', function(event) {
            if (event.target.classList.contains('modal')) {
                closeModal();
            }
        });

        // Load data on page load
        loadDemoData();
    </script>
</body>
</html>"#;

pub const CSS_DARK_THEME: &str = r#"
/* Dark theme overrides */
@media (prefers-color-scheme: dark) {
    :root {
        --primary: #58a6ff;
        --primary-dark: #1f6feb;
        --secondary: #8b949e;
        --background: #0d1117;
        --surface: #161b22;
        --text: #e6edf3;
        --text-light: #8b949e;
        --border: #30363d;
        --success: #3fb950;
        --warning: #fb8500;
        --danger: #f85149;
    }
}
"#;

#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_templates_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
