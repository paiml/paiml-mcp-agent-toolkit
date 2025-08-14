# TUI (Terminal User Interface) Documentation

The PMAT TUI provides an interactive terminal-based interface for exploring code analysis results. It offers a keyboard-driven experience for navigating complex codebases and understanding analysis metrics without leaving the terminal.

## Table of Contents

1. [Overview](#overview)
2. [Installation](#installation)
3. [Getting Started](#getting-started)
4. [Interface Layout](#interface-layout)
5. [Navigation](#navigation)
6. [Features](#features)
7. [Keyboard Shortcuts](#keyboard-shortcuts)
8. [Panels and Views](#panels-and-views)
9. [Configuration](#configuration)
10. [Examples](#examples)
11. [Troubleshooting](#troubleshooting)

## Overview

The TUI (Terminal User Interface) provides:

- **Interactive Navigation**: Browse files, functions, and metrics
- **Real-time Analysis**: Live updates as you explore
- **Keyboard-driven**: Efficient navigation without mouse
- **Multiple Views**: Different perspectives on your code
- **Search & Filter**: Find issues quickly
- **Cross-platform**: Works in any terminal

## Installation

The TUI interface requires the `tui` feature to be enabled:

```bash
# Install with TUI support
cargo install pmat --features tui

# Or build from source
cargo build --release --features tui
```

### Requirements

- Terminal with 256 color support
- Minimum terminal size: 80x24
- UTF-8 encoding support

## Getting Started

### Launch TUI

```bash
# Start TUI for current directory
pmat demo --mode tui

# Analyze specific path
pmat demo --mode tui --path ./src

# Analyze repository
pmat demo --mode tui --repo owner/repo
```

### First Steps

1. Wait for initial analysis to complete
2. Use arrow keys to navigate file tree
3. Press Enter to view file details
4. Press Tab to switch between panels
5. Press 'h' for help

## Interface Layout

```
┌─ PMAT Analysis - /path/to/project ─────────────────────────────┐
│ [Files] [Functions] [Metrics] [Issues] [Dependencies]          │
├─────────────────────────────────────────────────────────────────┤
│ 📁 src/                    │ 🔍 Function Details              │
│   📄 main.rs      [5/2]   │ Name: process_data               │
│   📄 lib.rs       [3/1]   │ Complexity: 15 (High)            │
│ ▶ 📁 modules/             │ Cognitive: 22                    │
│   📄 parser.rs    [12/4]  │ Lines: 87                        │
│   📄 analyzer.rs  [8/3]   │ Parameters: 4                    │
│ ▶ 📁 tests/               │ Nesting: 3                       │
├───────────────────────────┼──────────────────────────────────┤
│ Summary:                  │ Hotspots:                        │
│ Files: 42                 │ 1. parser::parse_expr (15)       │
│ Functions: 186            │ 2. analyzer::check (12)          │
│ Avg Complexity: 3.4       │ 3. main::process (10)            │
│ Tech Debt: 18.5h          │ 4. lib::validate (9)             │
└─────────────────────────────────────────────────────────────────┘
[Tab] Switch Panel  [↑↓] Navigate  [Enter] Select  [q] Quit  [h] Help
```

## Navigation

### Basic Movement

- **↑/↓**: Move up/down in current panel
- **←/→**: Expand/collapse tree nodes
- **Page Up/Down**: Scroll full page
- **Home/End**: Jump to top/bottom
- **Tab**: Switch between panels
- **Shift+Tab**: Previous panel

### Selection

- **Enter**: Select item/drill down
- **Space**: Toggle selection
- **Ctrl+A**: Select all
- **Esc**: Cancel/go back

## Features

### 1. File Browser

Navigate project structure with complexity indicators:

```
📁 src/                  
  📄 main.rs      [5/2]   # [complexity/issues]
  📄 lib.rs       [3/1]
  📁 modules/
    📄 parser.rs  [12/4]
```

### 2. Function Explorer

Browse functions with metrics:

```
Functions in parser.rs:
├─ parse_expression()    Complexity: 15  Lines: 120
├─ tokenize()           Complexity: 8   Lines: 65
├─ validate_syntax()    Complexity: 6   Lines: 45
└─ build_ast()          Complexity: 10  Lines: 85
```

### 3. Metrics Dashboard

Real-time metrics display:

```
━━━ Metrics Overview ━━━
Cyclomatic Complexity: 15 ⚠️
Cognitive Complexity:  22 🔴
Lines of Code:        450
Test Coverage:        78% ✓
Technical Debt:      2.5h
Maintainability:      B+
```

### 4. Issue Finder

Categorized issues and warnings:

```
━━━ Issues (12) ━━━
🔴 High (3)
  - Complex function: parse_expr (cc: 15)
  - Long method: process_data (lines: 150)
  - Deep nesting: validate_input (depth: 5)

⚠️ Medium (5)
  - TODO: Implement error handling
  - Duplicate code block (lines 45-67)
  ...
```

### 5. Dependency Viewer

Interactive dependency graph:

```
━━━ Dependencies ━━━
main.rs
├─→ lib.rs
│   ├─→ parser.rs
│   └─→ analyzer.rs
└─→ config.rs
    └─→ utils.rs
```

## Keyboard Shortcuts

### Global

| Key | Action |
|-----|--------|
| `q` | Quit application |
| `h` or `?` | Show help |
| `Tab` | Next panel |
| `Shift+Tab` | Previous panel |
| `/` | Search mode |
| `f` | Filter mode |
| `r` | Refresh analysis |
| `Ctrl+C` | Force quit |

### Navigation

| Key | Action |
|-----|--------|
| `↑` `k` | Move up |
| `↓` `j` | Move down |
| `←` `h` | Collapse/Back |
| `→` `l` | Expand/Forward |
| `g` | Go to top |
| `G` | Go to bottom |
| `Ctrl+U` | Page up |
| `Ctrl+D` | Page down |

### View Controls

| Key | Action |
|-----|--------|
| `1-5` | Switch to panel 1-5 |
| `m` | Toggle metrics view |
| `t` | Toggle tree view |
| `d` | Show dependencies |
| `i` | Show issues |
| `s` | Sort options |

### Search & Filter

| Key | Action |
|-----|--------|
| `/` | Start search |
| `n` | Next match |
| `N` | Previous match |
| `f` | Filter menu |
| `Ctrl+F` | Quick filter |
| `Esc` | Clear search/filter |

## Panels and Views

### Files Panel

Shows project structure with visual indicators:

- 📁 Directories (expandable)
- 📄 Files with [complexity/issues] badges
- 🔴 High complexity (>10)
- ⚠️ Medium complexity (5-10)
- ✅ Low complexity (<5)

### Functions Panel

Lists all functions with:

- Function signature
- Complexity metrics
- Line count
- Issue indicators
- Test coverage (if available)

### Metrics Panel

Displays detailed metrics:

- Cyclomatic complexity
- Cognitive complexity
- Lines of code
- Comment ratio
- Test coverage
- Maintainability index
- Technical debt estimation

### Issues Panel

Categorized list of problems:

- **Critical**: Security issues, bugs
- **High**: Complex functions, long methods
- **Medium**: TODOs, code smells
- **Low**: Style issues, minor warnings

### Dependencies Panel

Shows module relationships:

- Import/export relationships
- Circular dependencies (highlighted)
- External dependencies
- Dependency depth

## Configuration

### TUI Settings

Configure TUI behavior via environment variables:

```bash
# Color scheme
export PMAT_TUI_THEME=dark  # dark|light|auto

# Update interval (ms)
export PMAT_TUI_REFRESH=1000

# Initial panel
export PMAT_TUI_START_PANEL=files  # files|functions|metrics

# Show hidden files
export PMAT_TUI_SHOW_HIDDEN=true
```

### Display Options

```bash
# Terminal colors
export COLORTERM=truecolor

# Unicode support
export LANG=en_US.UTF-8

# Terminal type
export TERM=xterm-256color
```

## Examples

### Example 1: Quick Project Overview

```bash
# Start TUI and get overview
pmat demo --mode tui

# Press 'm' for metrics view
# See project health at a glance
```

### Example 2: Find Complex Functions

```bash
# Start TUI
pmat demo --mode tui

# Press '2' for functions panel
# Press 's' then 'c' to sort by complexity
# Navigate to highest complexity functions
```

### Example 3: Explore Dependencies

```bash
# Start TUI with focus on dependencies
pmat demo --mode tui

# Press 'd' for dependency view
# Use arrows to explore relationships
# Press 'c' to highlight circular deps
```

### Example 4: Search for Issues

```bash
# Start TUI
pmat demo --mode tui

# Press '/' to search
# Type "TODO" to find all TODOs
# Press 'n' to cycle through results
```

### Example 5: Filter by Complexity

```bash
# Start TUI
pmat demo --mode tui

# Press 'f' for filter menu
# Select "High complexity only"
# View only problematic files
```

## Troubleshooting

### Terminal Issues

1. **Colors not displaying correctly**
   ```bash
   export TERM=xterm-256color
   export COLORTERM=truecolor
   ```

2. **Unicode characters showing as boxes**
   ```bash
   export LANG=en_US.UTF-8
   export LC_ALL=en_US.UTF-8
   ```

3. **Layout broken**
   - Resize terminal to at least 80x24
   - Check terminal font supports Unicode

### Performance Issues

1. **Slow navigation**
   - Increase refresh interval
   - Disable real-time analysis
   - Use filter to reduce items

2. **High CPU usage**
   ```bash
   export PMAT_TUI_REFRESH=2000  # Slower updates
   ```

### Common Problems

1. **TUI won't start**
   - Ensure `tui` feature is enabled
   - Check terminal compatibility
   - Try different terminal emulator

2. **Keys not working**
   - Check terminal key bindings
   - Disable terminal shortcuts that conflict
   - Try vi-style keys (hjkl)

3. **Analysis not updating**
   - Press 'r' to force refresh
   - Check file system permissions
   - Verify git repository status

## Advanced Usage

### Custom Key Bindings

Create `~/.config/pmat/tui-keys.toml`:

```toml
[keys]
up = ["k", "Up"]
down = ["j", "Down"]
left = ["h", "Left"]
right = ["l", "Right"]
quit = ["q", "Ctrl+C"]
```

### Scripting

Use TUI in scripts with automatic navigation:

```bash
# Auto-navigate to issues
echo -e "i\n/TODO\n" | pmat demo --mode tui

# Export current view
pmat demo --mode tui --export-view metrics.txt
```

## Best Practices

1. **Learn Keyboard Shortcuts**: Efficiency comes from keyboard mastery
2. **Use Filters**: Focus on what matters
3. **Regular Refresh**: Press 'r' after code changes
4. **Panel Workflow**: Develop a consistent panel navigation pattern
5. **Search Effectively**: Use regex patterns for complex searches

## See Also

- [Demo Interface](/docs/features/demo-interface.md) - General demo documentation
- [CLI Reference](/docs/cli-reference.md) - Command-line options
- [Keyboard Shortcuts Reference Card](#keyboard-shortcuts) - Quick reference