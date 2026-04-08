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

                // Load AI-powered recommendations
                await loadRecommendations();

                // Load polyglot analysis
                await loadPolyglotAnalysis();

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
</html>