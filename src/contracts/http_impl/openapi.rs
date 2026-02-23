#![cfg_attr(coverage_nightly, coverage(off))]
//! OpenAPI specification generation for uniform contract endpoints

use serde_json::{json, Value};

/// Generate `OpenAPI` specification with uniform contracts
pub(super) fn generate_openapi_spec() -> Value {
    json!({
        "openapi": "3.0.0",
        "info": {
            "title": "PMAT API",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "Professional project quantitative analysis toolkit with uniform contracts"
        },
        "servers": [
            {
                "url": "http://localhost:8080",
                "description": "Local server"
            }
        ],
        "paths": {
            "/api/analyze/complexity": {
                "post": {
                    "summary": "Analyze code complexity",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/AnalyzeComplexityContract"
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Analysis results"
                        }
                    }
                }
            },
            "/api/analyze/satd": {
                "post": {
                    "summary": "Analyze SATD",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/AnalyzeSatdContract"
                                }
                            }
                        }
                    }
                }
            },
            "/api/analyze/dead-code": {
                "post": {
                    "summary": "Analyze dead code",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/AnalyzeDeadCodeContract"
                                }
                            }
                        }
                    }
                }
            },
            "/api/analyze/tdg": {
                "post": {
                    "summary": "Analyze TDG",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/AnalyzeTdgContract"
                                }
                            }
                        }
                    }
                }
            },
            "/api/analyze/lint-hotspot": {
                "post": {
                    "summary": "Analyze lint hotspots",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/AnalyzeLintHotspotContract"
                                }
                            }
                        }
                    }
                }
            },
            "/api/quality-gate": {
                "post": {
                    "summary": "Run quality gate",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/QualityGateContract"
                                }
                            }
                        }
                    }
                }
            },
            "/api/refactor/auto": {
                "post": {
                    "summary": "Auto refactor",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/RefactorAutoContract"
                                }
                            }
                        }
                    }
                }
            }
        },
        "components": {
            "schemas": {
                "BaseAnalysisContract": {
                    "type": "object",
                    "required": ["path"],
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to analyze"
                        },
                        "format": {
                            "type": "string",
                            "enum": ["table", "json", "yaml", "markdown", "csv", "summary"],
                            "default": "table"
                        },
                        "output": {
                            "type": "string",
                            "description": "Output file path"
                        },
                        "top_files": {
                            "type": "integer",
                            "description": "Number of top files",
                            "default": 10
                        },
                        "include_tests": {
                            "type": "boolean",
                            "default": false
                        },
                        "timeout": {
                            "type": "integer",
                            "default": 60
                        }
                    }
                },
                "AnalyzeComplexityContract": {
                    "allOf": [
                        { "$ref": "#/components/schemas/BaseAnalysisContract" },
                        {
                            "type": "object",
                            "properties": {
                                "max_cyclomatic": { "type": "integer" },
                                "max_cognitive": { "type": "integer" },
                                "max_halstead": { "type": "number" }
                            }
                        }
                    ]
                },
                "AnalyzeSatdContract": {
                    "allOf": [
                        { "$ref": "#/components/schemas/BaseAnalysisContract" },
                        {
                            "type": "object",
                            "properties": {
                                "severity": {
                                    "type": "string",
                                    "enum": ["low", "medium", "high", "critical"]
                                },
                                "critical_only": { "type": "boolean" },
                                "strict": { "type": "boolean" },
                                "fail_on_violation": { "type": "boolean" }
                            }
                        }
                    ]
                },
                "AnalyzeDeadCodeContract": {
                    "allOf": [
                        { "$ref": "#/components/schemas/BaseAnalysisContract" },
                        {
                            "type": "object",
                            "properties": {
                                "include_unreachable": { "type": "boolean" },
                                "min_dead_lines": { "type": "integer" },
                                "max_percentage": { "type": "number" },
                                "fail_on_violation": { "type": "boolean" }
                            }
                        }
                    ]
                },
                "AnalyzeTdgContract": {
                    "allOf": [
                        { "$ref": "#/components/schemas/BaseAnalysisContract" },
                        {
                            "type": "object",
                            "properties": {
                                "threshold": { "type": "number" },
                                "include_components": { "type": "boolean" },
                                "critical_only": { "type": "boolean" }
                            }
                        }
                    ]
                },
                "AnalyzeLintHotspotContract": {
                    "allOf": [
                        { "$ref": "#/components/schemas/BaseAnalysisContract" },
                        {
                            "type": "object",
                            "properties": {
                                "file": { "type": "string" },
                                "max_density": { "type": "number" },
                                "min_confidence": { "type": "number" },
                                "enforce": { "type": "boolean" },
                                "dry_run": { "type": "boolean" }
                            }
                        }
                    ]
                },
                "QualityGateContract": {
                    "allOf": [
                        { "$ref": "#/components/schemas/BaseAnalysisContract" },
                        {
                            "type": "object",
                            "properties": {
                                "profile": {
                                    "type": "string",
                                    "enum": ["standard", "strict", "extreme", "toyota"]
                                },
                                "file": { "type": "string" },
                                "fail_on_violation": { "type": "boolean" },
                                "verbose": { "type": "boolean" }
                            }
                        }
                    ]
                },
                "RefactorAutoContract": {
                    "type": "object",
                    "required": ["file"],
                    "properties": {
                        "file": { "type": "string" },
                        "format": {
                            "type": "string",
                            "enum": ["table", "json", "yaml", "markdown", "csv", "summary"]
                        },
                        "output": { "type": "string" },
                        "target_complexity": { "type": "integer" },
                        "dry_run": { "type": "boolean" },
                        "timeout": { "type": "integer" }
                    }
                }
            }
        }
    })
}
