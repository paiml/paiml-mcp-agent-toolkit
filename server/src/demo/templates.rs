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

        #mermaid-container, #system-diagram-container {
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
        <div class="stats-grid">
            <div class="stat-card">
                <div class="stat-label">Files Analyzed</div>
                <div class="stat-value">
                    <span id="files-count">{files_analyzed}</span>
                    <span class="stat-unit">files</span>
                </div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Average Complexity</div>
                <div class="stat-value">
                    <span id="avg-complexity">{avg_complexity:.2}</span>
                    <span class="stat-unit">cyclomatic</span>
                </div>
            </div>
            <div class="stat-card">
                <div class="stat-label">90th Percentile</div>
                <div class="stat-value">
                    <span id="p90-complexity">{p90_complexity}</span>
                    <span class="stat-unit">cyclomatic</span>
                </div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Technical Debt</div>
                <div class="stat-value">
                    <span id="tech-debt">{tech_debt_hours}</span>
                    <span class="stat-unit">hours</span>
                </div>
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
                📊 Dependency Graph
            </h2>
            <div class="section-meta">
                <a href="/api/dag" target="_blank" class="endpoint-url">/api/dag</a>
                <div class="data-source">
                    <div class="data-indicator default"></div>
                    <span>Default</span>
                </div>
            </div>
            <div id="mermaid-container">
                <div class="loading">Loading dependency graph...</div>
            </div>
        </section>

        <section class="section">
            <h2 class="section-title">
                🏗️ System Architecture
            </h2>
            <div class="section-meta">
                <a href="/api/system-diagram" target="_blank" class="endpoint-url">/api/system-diagram</a>
                <div class="data-source">
                    <div class="data-indicator dynamic"></div>
                    <span>Dynamic</span>
                </div>
            </div>
            <div id="system-diagram-container">
                <div class="loading">Loading system architecture...</div>
            </div>
        </section>
    </main>

    <script src="/vendor/gridjs.min.js"></script>
    <script type="module">
        import mermaid from 'https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.esm.min.mjs';
        
        // Initialize Mermaid
        mermaid.initialize({ 
            startOnLoad: false,
            theme: 'default',
            flowchart: {
                useMaxWidth: true,
                htmlLabels: true,
                curve: 'basis'
            }
        });

        // Fetch and display demo data
        async function loadDemoData() {
            try {
                // Fetch summary data
                const summaryResponse = await fetch('/api/summary');
                const summary = await summaryResponse.json();

                // Update analysis time
                const totalTime = summary.time_context + summary.time_complexity + summary.time_dag + summary.time_churn;
                document.getElementById('analysis-time').textContent = `Analysis completed in ${totalTime}ms`;

                // Load hotspots table
                const hotspotsResponse = await fetch('/api/hotspots');
                const hotspots = await hotspotsResponse.json();

                new gridjs.Grid({
                    columns: [
                        { 
                            name: 'Function',
                            data: (row) => row.function
                        },
                        { 
                            name: 'Complexity',
                            data: (row) => row.complexity,
                            formatter: (cell) => {
                                const colorClass = cell > 10 ? 'danger' : cell > 5 ? 'warning' : 'success';
                                return gridjs.html(`<span style="color: var(--${colorClass}); font-weight: 600">${cell}</span>`);
                            }
                        },
                        { 
                            name: 'Lines of Code',
                            data: (row) => row.loc
                        },
                        { 
                            name: 'Path',
                            data: (row) => row.path,
                            formatter: (cell) => gridjs.html(`<code style="font-size: 0.75rem">${cell}</code>`)
                        }
                    ],
                    data: hotspots,
                    sort: true,
                    search: true,
                    pagination: {
                        limit: 10
                    },
                    style: {
                        table: {
                            'border-radius': '0.5rem'
                        },
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
                    }
                }).render(document.getElementById('hotspots-table'));

                // Load and render DAG
                const dagResponse = await fetch('/api/dag');
                const dagText = await dagResponse.text();
                
                if (dagText && dagText.trim()) {
                    const container = document.getElementById('mermaid-container');
                    container.innerHTML = `<pre class="mermaid">${dagText}</pre>`;
                    await mermaid.run();
                } else {
                    document.getElementById('mermaid-container').innerHTML = '<div class="error">No dependency graph available</div>';
                }

                // Load and render System Architecture
                const systemResponse = await fetch('/api/system-diagram');
                const systemText = await systemResponse.text();
                
                if (systemText && systemText.trim()) {
                    const systemContainer = document.getElementById('system-diagram-container');
                    systemContainer.innerHTML = `<pre class="mermaid">${systemText}</pre>`;
                    await mermaid.run();
                    
                    // Update data source indicator based on whether we have dynamic content
                    // Check if it's the hardcoded fallback diagram
                    const isHardcoded = systemText.includes('AST Context Analysis') && 
                                       systemText.includes('File Parser') && 
                                       systemText.includes('Handlebars');
                    
                    const indicator = document.querySelector('.section:last-of-type .data-indicator');
                    const statusText = document.querySelector('.section:last-of-type .data-source span');
                    
                    if (isHardcoded) {
                        indicator.className = 'data-indicator default';
                        statusText.textContent = 'Default';
                    } else {
                        indicator.className = 'data-indicator dynamic';
                        statusText.textContent = 'Dynamic';
                    }
                } else {
                    document.getElementById('system-diagram-container').innerHTML = '<div class="error">No system architecture available</div>';
                }

            } catch (error) {
                console.error('Error loading demo data:', error);
                document.getElementById('mermaid-container').innerHTML = `<div class="error">Error: ${error.message}</div>`;
                document.getElementById('system-diagram-container').innerHTML = `<div class="error">Error: ${error.message}</div>`;
            }
        }

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
