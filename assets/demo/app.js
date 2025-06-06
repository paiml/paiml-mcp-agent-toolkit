// Demo app JavaScript - loaded after Grid.js and Mermaid.js

// Global configuration
const APP_CONFIG = {
    refreshInterval: 30000, // 30 seconds
    maxHotspots: 20,
    complexityThresholds: {
        critical: 20,
        high: 10
    }
};

// Utility functions
function formatFileSize(bytes) {
    const sizes = ['B', 'KB', 'MB', 'GB'];
    if (bytes === 0) return '0 B';
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i];
}

function formatDuration(ms) {
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(2)}s`;
}

// Interactive features
document.addEventListener('DOMContentLoaded', () => {
    // Add keyboard shortcuts
    document.addEventListener('keydown', (e) => {
        if (e.key === 'r' && (e.ctrlKey || e.metaKey)) {
            e.preventDefault();
            refreshData();
        }
        if (e.key === 'e' && (e.ctrlKey || e.metaKey)) {
            e.preventDefault();
            exportReport();
        }
    });

    // Add export button to toolbar
    const toolbar = document.createElement('div');
    toolbar.className = 'toolbar';
    toolbar.innerHTML = `
        <button onclick="exportReport()" title="Export Report (Ctrl+E)">
            📄 Export
        </button>
        <button onclick="refreshData()" title="Refresh Data (Ctrl+R)">
            🔄 Refresh
        </button>
    `;
    document.querySelector('.main')?.prepend(toolbar);
});

// Data refresh functionality
async function refreshData() {
    try {
        // Refresh hotspots
        if (window.hotspotsGrid) {
            window.hotspotsGrid.updateConfig({
                data: () => fetch('/api/hotspots').then(res => res.json())
            }).forceRender();
        }

        // Refresh metrics
        const metricsResponse = await fetch('/api/metrics');
        if (metricsResponse.ok) {
            const metrics = await metricsResponse.json();
            updateMetricsDisplay(metrics);
        }

        // Refresh DAG
        const dagResponse = await fetch('/api/dag');
        if (dagResponse.ok) {
            const mermaidCode = await dagResponse.text();
            const container = document.getElementById('mermaid-container');
            if (container) {
                container.innerHTML = `<div class="mermaid">${mermaidCode}</div>`;
                if (window.mermaid) {
                    window.mermaid.init(undefined, container.querySelector('.mermaid'));
                }
            }
        }
    } catch (error) {
        console.error('Failed to refresh data:', error);
    }
}

function updateMetricsDisplay(metrics) {
    const metricElements = document.querySelectorAll('.metric-value');
    if (metricElements[0]) metricElements[0].textContent = metrics.files_analyzed;
    if (metricElements[1]) metricElements[1].textContent = metrics.avg_complexity.toFixed(1);
    if (metricElements[2]) metricElements[2].textContent = metrics.tech_debt_hours + ' hrs';
}

// Export functionality
async function exportReport() {
    try {
        const [metrics, hotspots, dag] = await Promise.all([
            fetch('/api/metrics').then(r => r.json()),
            fetch('/api/hotspots').then(r => r.json()),
            fetch('/api/dag').then(r => r.text())
        ]);

        const report = {
            generated_at: new Date().toISOString(),
            metrics,
            hotspots,
            dag_mermaid: dag
        };

        const blob = new Blob([JSON.stringify(report, null, 2)], {
            type: 'application/json'
        });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `paiml-analysis-${new Date().toISOString().split('T')[0]}.json`;
        a.click();
        URL.revokeObjectURL(url);
    } catch (error) {
        console.error('Failed to export report:', error);
        alert('Failed to export report. Check console for details.');
    }
}

// Add custom styles
const customStyles = document.createElement('style');
customStyles.textContent = `
.toolbar {
    display: flex;
    gap: 1rem;
    margin-bottom: 2rem;
    padding: 1rem;
    background: var(--bg-secondary);
    border-radius: 6px;
    border: 1px solid #30363d;
}
.toolbar button {
    padding: 0.5rem 1rem;
    background: var(--accent);
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.875rem;
    font-weight: 500;
    transition: opacity 0.2s;
}
.toolbar button:hover {
    opacity: 0.8;
}
.toolbar button:active {
    transform: translateY(1px);
}
.performance-metrics {
    display: grid;
    grid-template-columns: 1fr;
    gap: 0.5rem;
    padding: 1rem;
    background: var(--bg-primary);
    border-radius: 6px;
    border: 1px solid #30363d;
    font-size: 0.875rem;
    color: #8b949e;
}
.performance-metrics div {
    display: flex;
    justify-content: space-between;
}
code {
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', monospace;
    font-size: 0.875em;
    background: var(--bg-primary);
    padding: 0.125rem 0.25rem;
    border-radius: 3px;
}
`;
document.head.appendChild(customStyles);