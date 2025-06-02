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
    templates[Template Engine]
    mcp_server[MCP Server]
    cache[Cache Layer]
    handlers[Protocol Handlers]
    analyzer[Code Analyzer]

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
    cache_check[Cache Check]
    response[Send Response]
    validate[Validate Input]
    analyze[Analyze Code]
    request[Client Request]
    generate[Generate Output]

    request --> validate
    validate --> cache_check
    cache_check --> analyze
    cache_check --> response
    analyze --> generate
    generate --> response

    style cache_check fill:#90EE90,stroke:#333,stroke-dasharray: 5 5,stroke-width:2px
    style response fill:#90EE90,stroke:#333,stroke-dasharray: 5 5,stroke-width:2px
    style validate fill:#FFD700,stroke:#333,stroke-dasharray: 5 5,stroke-width:2px
    style analyze fill:#FF6347,stroke:#333,stroke-dasharray: 5 5,stroke-width:2px
    style request fill:#90EE90,stroke:#333,stroke-dasharray: 5 5,stroke-width:2px
    style generate fill:#FFA500,stroke:#333,stroke-dasharray: 5 5,stroke-width:2px
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
    utils[utils]
    handlers[handlers]
    cli[cli]
    services[services]
    models[models]
    lib[lib]

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
    dag_builder[DagBuilder]
    complexity[ComplexityAnalyzer]
    ast_rust[RustAST]
    template_service[TemplateService]
    mermaid_generator[MermaidGenerator]
    code_intelligence[CodeIntelligence]

    code_intelligence --> dag_builder
    dag_builder --> ast_rust
    dag_builder --> mermaid_generator
    code_intelligence --> complexity
    template_service --- ast_rust

    style dag_builder fill:#FF6347,stroke-width:2px
    style complexity fill:#FF6347,stroke-width:2px
    style ast_rust fill:#FF6347,stroke-width:2px
    style template_service fill:#FFA500,stroke-width:2px
    style mermaid_generator fill:#FFA500,stroke-width:2px
    style code_intelligence fill:#FF6347,stroke-width:2px
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

Last validated: 2025-06-01 20:48:05 UTC
