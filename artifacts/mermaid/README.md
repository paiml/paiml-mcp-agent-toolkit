# Mermaid Diagram Artifacts

This directory contains test-maintained Mermaid diagram examples demonstrating the capabilities of the PAIML MCP Agent Toolkit.

## Directory Structure

```
mermaid/
├── non-code/          # Hand-crafted architectural diagrams
│   ├── simple/       # Without styling
│   └── styled/       # With complexity indicators
└── ast-generated/     # Generated from codebase analysis
    ├── simple/       # Basic structure
    └── styled/       # With metrics
```

## Non-Code Simple Diagrams

### architecture-overview.mmd

Simple 5-component system architecture

```mermaid
graph TD
    cache[Cache Layer]
    analyzer[Code Analyzer]
    mcp_server[MCP Server]
    handlers[Protocol Handlers]
    templates[Template Engine]

    mcp_server --- handlers
    handlers --> analyzer
    handlers --> templates
    analyzer --- cache
```

**Metrics:**
- Nodes: 5
- Edges: 4
- Max depth: 1

## Non-Code Styled Diagrams

### workflow-styled.mmd

Request processing workflow with complexity styling

```mermaid
graph TD
    analyze[Analyze Code]
    cache_check[Cache Check]
    request[Client Request]
    generate[Generate Output]
    response[Send Response]
    validate[Validate Input]

    request --> validate
    validate --> cache_check
    cache_check --> analyze
    cache_check --> response
    analyze --> generate
    generate --> response

    style analyze fill:#FF6347,stroke:#333,stroke-dasharray: 5 5,stroke-width:2px
    style cache_check fill:#90EE90,stroke:#333,stroke-dasharray: 5 5,stroke-width:2px
    style request fill:#90EE90,stroke:#333,stroke-dasharray: 5 5,stroke-width:2px
    style generate fill:#FFA500,stroke:#333,stroke-dasharray: 5 5,stroke-width:2px
    style response fill:#90EE90,stroke:#333,stroke-dasharray: 5 5,stroke-width:2px
    style validate fill:#FFD700,stroke:#333,stroke-dasharray: 5 5,stroke-width:2px
```

**Metrics:**
- Nodes: 6
- Edges: 6
- Max depth: 1
- Styling: ✓ Complexity indicators

## AST-Generated Simple Diagrams

### codebase-modules.mmd

Top-level module structure from AST analysis

```mermaid
graph TD
    cli[cli]
    handlers[handlers]
    lib[lib]
    models[models]
    services[services]
    utils[utils]

    lib --- handlers
    handlers --- services
    services --- models
    cli --- services
    services --- utils
```

**Metrics:**
- Nodes: 6
- Edges: 5
- Max depth: 1

## AST-Generated Styled Diagrams

### service-interactions.mmd

Core service interactions with complexity indicators

```mermaid
graph TD
    code_intelligence[CodeIntelligence]
    complexity[ComplexityAnalyzer]
    dag_builder[DagBuilder]
    mermaid_generator[MermaidGenerator]
    ast_rust[RustAST]
    template_service[TemplateService]

    code_intelligence --> dag_builder
    dag_builder --> ast_rust
    dag_builder --> mermaid_generator
    code_intelligence --> complexity
    template_service --- ast_rust

    style code_intelligence fill:#FF6347,stroke-width:2px
    style complexity fill:#FF6347,stroke-width:2px
    style dag_builder fill:#FF6347,stroke-width:2px
    style mermaid_generator fill:#FFA500,stroke-width:2px
    style ast_rust fill:#FF6347,stroke-width:2px
    style template_service fill:#FFA500,stroke-width:2px
```

**Metrics:**
- Nodes: 6
- Edges: 5
- Max depth: 1
- Styling: ✓ Complexity indicators

## Validation Status

All diagrams are automatically validated for:
- ✓ Correct Mermaid syntax
- ✓ Node count ≤ 15
- ✓ Proper labeling (no empty nodes)
- ✓ Category-appropriate styling

Last validated: 2025-06-09 03:55:31 UTC
