<!DOCTYPE html>
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

        .data-indicator.ai {
            background-color: #8b5cf6;
            animation: pulse-ai 2s infinite;
        }

        .data-indicator.polyglot {
            background: linear-gradient(45deg, #3b82f6, #10b981, #f59e0b);
            animation: rainbow 3s infinite;
        }

        @keyframes pulse-ai {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.6; }
        }

        @keyframes rainbow {
            0% { filter: hue-rotate(0deg); }
            100% { filter: hue-rotate(360deg); }
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
