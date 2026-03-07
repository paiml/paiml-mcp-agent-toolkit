#![cfg_attr(coverage_nightly, coverage(off))]
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

        // AI-Powered Repository Recommendations
        async function loadRecommendations() {
            try {
                const response = await fetch('/api/recommendations');
                if (response.ok) {
                    const recommendations = await response.json();
                    displayRecommendations(recommendations);
                }
            } catch (error) {
                console.warn('Recommendations not available:', error);
            }
        }

        function displayRecommendations(recommendations) {
            const grid = document.getElementById('recommendations-grid');
            if (!grid || !recommendations || recommendations.length === 0) return;

            grid.innerHTML = recommendations.map(rec => `
                <div class="recommendation-card" onclick="showRecommendationDetails('${rec.repository}')">
                    <div class="rec-header">
                        <h4>${rec.repository}</h4>
                        <span class="confidence-badge">${(rec.confidence * 100).toFixed(0)}%</span>
                    </div>
                    <p class="rec-reason">${rec.reason}</p>
                    <div class="rec-tags">
                        ${rec.framework_detected ? `<span class="framework-tag">${rec.framework_detected}</span>` : ''}
                        <span class="complexity-tag complexity-${rec.complexity_tier.toLowerCase()}">${rec.complexity_tier}</span>
                    </div>
                    <div class="learning-focus">
                        ${rec.learning_focus.slice(0, 2).map(focus => `<span class="focus-tag">${focus}</span>`).join('')}
                    </div>
                </div>
            `).join('');

            // Add CSS for recommendation cards
            if (!document.getElementById('rec-styles')) {
                const style = document.createElement('style');
                style.id = 'rec-styles';
                style.textContent = `
                    .recommendation-grid {
                        display: grid;
                        grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
                        gap: 1rem;
                        margin-top: 1rem;
                    }
                    .recommendation-card {
                        border: 1px solid var(--border);
                        border-radius: 8px;
                        padding: 1rem;
                        background: var(--surface);
                        cursor: pointer;
                        transition: all 0.2s ease;
                    }
                    .recommendation-card:hover {
                        border-color: var(--primary);
                        transform: translateY(-2px);
                        box-shadow: 0 4px 12px rgba(0,0,0,0.1);
                    }
                    .rec-header {
                        display: flex;
                        justify-content: space-between;
                        align-items: center;
                        margin-bottom: 0.5rem;
                    }
                    .rec-header h4 {
                        margin: 0;
                        color: var(--primary);
                    }
                    .confidence-badge {
                        background: var(--success);
                        color: white;
                        padding: 0.125rem 0.5rem;
                        border-radius: 12px;
                        font-size: 0.75rem;
                        font-weight: bold;
                    }
                    .rec-reason {
                        color: var(--text-light);
                        margin: 0.5rem 0;
                        font-size: 0.9rem;
                    }
                    .rec-tags, .learning-focus {
                        display: flex;
                        gap: 0.5rem;
                        flex-wrap: wrap;
                        margin: 0.5rem 0;
                    }
                    .framework-tag, .complexity-tag, .focus-tag {
                        padding: 0.25rem 0.5rem;
                        border-radius: 4px;
                        font-size: 0.75rem;
                        font-weight: 500;
                    }
                    .framework-tag {
                        background: #e0f2fe;
                        color: #0277bd;
                    }
                    .complexity-beginner { background: #e8f5e8; color: #2e7d32; }
                    .complexity-intermediate { background: #fff3e0; color: #f57c00; }
                    .complexity-advanced { background: #ffebee; color: #d32f2f; }
                    .focus-tag {
                        background: var(--background);
                        color: var(--text);
                        border: 1px solid var(--border);
                    }
                `;
                document.head.appendChild(style);
            }
        }

        // Multi-Language Project Intelligence
        async function loadPolyglotAnalysis() {
            try {
                const response = await fetch('/api/polyglot');
                if (response.ok) {
                    const analysis = await response.json();
                    displayPolyglotAnalysis(analysis);
                }
            } catch (error) {
                console.warn('Polyglot analysis not available:', error);
            }
        }

        function displayPolyglotAnalysis(analysis) {
            if (!analysis || analysis.languages.length <= 1) return;

            const section = document.getElementById('polyglot-section');
            const breakdown = document.getElementById('language-breakdown');
            const integrations = document.getElementById('integration-points');
            const architecture = document.getElementById('architecture-pattern');

            if (section) section.style.display = 'block';

            // Language breakdown
            if (breakdown) {
                breakdown.innerHTML = `
                    <h3>🔤 Language Breakdown</h3>
                    <div class="polyglot-languages">
                        ${analysis.languages.map(lang => `
                            <div class="lang-card">
                                <div class="lang-header">
                                    <span class="lang-name">${getLanguageEmoji(lang.language)} ${lang.language}</span>
                                    <span class="lang-share">${((lang.line_count / analysis.languages.reduce((sum, l) => sum + l.line_count, 0)) * 100).toFixed(1)}%</span>
                                </div>
                                <div class="lang-stats">
                                    <span>${lang.file_count} files</span>
                                    <span>${lang.line_count} lines</span>
                                    <span>Complexity: ${lang.complexity_score.toFixed(1)}</span>
                                </div>
                            </div>
                        `).join('')}
                    </div>
                `;
            }

            // Integration points
            if (integrations && analysis.integration_points.length > 0) {
                integrations.innerHTML = `
                    <h3>🔗 Integration Points</h3>
                    <div class="integration-list">
                        ${analysis.integration_points.map(point => `
                            <div class="integration-item risk-${point.risk_level.toLowerCase()}">
                                <div class="integration-name">${point.name}</div>
                                <div class="integration-type">${point.integration_type}</div>
                                <div class="risk-badge">${point.risk_level} Risk</div>
                            </div>
                        `).join('')}
                    </div>
                `;
            }

            // Architecture pattern
            if (architecture && analysis.architecture_pattern) {
                architecture.innerHTML = `
                    <h3>🏗️ Architecture Pattern</h3>
                    <div class="architecture-info">
                        <span class="pattern-badge">${analysis.architecture_pattern}</span>
                        <span class="recommendation-score">Score: ${(analysis.recommendation_score * 100).toFixed(0)}%</span>
                    </div>
                `;
            }

            // Add polyglot-specific styles
            if (!document.getElementById('polyglot-styles')) {
                const style = document.createElement('style');
                style.id = 'polyglot-styles';
                style.textContent = `
                    .polyglot-languages {
                        display: grid;
                        grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
                        gap: 1rem;
                        margin-top: 1rem;
                    }
                    .lang-card {
                        border: 1px solid var(--border);
                        border-radius: 8px;
                        padding: 1rem;
                        background: var(--surface);
                    }
                    .lang-header {
                        display: flex;
                        justify-content: space-between;
                        align-items: center;
                        margin-bottom: 0.5rem;
                    }
                    .lang-name {
                        font-weight: 600;
                        color: var(--text);
                    }
                    .lang-share {
                        background: var(--primary);
                        color: white;
                        padding: 0.125rem 0.5rem;
                        border-radius: 12px;
                        font-size: 0.75rem;
                    }
                    .lang-stats {
                        display: flex;
                        gap: 1rem;
                        font-size: 0.875rem;
                        color: var(--text-light);
                    }
                    .integration-list {
                        display: flex;
                        flex-direction: column;
                        gap: 0.5rem;
                        margin-top: 1rem;
                    }
                    .integration-item {
                        display: flex;
                        justify-content: space-between;
                        align-items: center;
                        padding: 0.75rem;
                        border-radius: 6px;
                        border-left: 4px solid;
                    }
                    .risk-low { border-left-color: var(--success); background: rgba(16, 185, 129, 0.1); }
                    .risk-medium { border-left-color: var(--warning); background: rgba(245, 158, 11, 0.1); }
                    .risk-high { border-left-color: var(--danger); background: rgba(239, 68, 68, 0.1); }
                    .architecture-info {
                        display: flex;
                        gap: 1rem;
                        align-items: center;
                        margin-top: 1rem;
                    }
                    .pattern-badge {
                        background: var(--primary);
                        color: white;
                        padding: 0.5rem 1rem;
                        border-radius: 6px;
                        font-weight: 600;
                    }
                `;
                document.head.appendChild(style);
            }
        }

        function getLanguageEmoji(language) {
            const emojis = {
                'rust': '🦀',
                'python': '🐍',
                'typescript': '🟦',
                'javascript': '💛',
                'go': '🐹',
                'java': '☕',
                'c++': '⚡',
                'c': '🔧'
            };
            return emojis[language.toLowerCase()] || '📄';
        }

        function showRecommendationDetails(repository) {
            showModal(`Repository: ${repository}`, `
                <p>Detailed analysis and learning resources for ${repository}</p>
                <div style="margin-top: 1rem;">
                    <a href="https://github.com/${repository}" target="_blank" class="button">View on GitHub</a>
                </div>
            `);
        }
"#;
