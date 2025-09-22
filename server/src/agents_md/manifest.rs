//! Agent Manifest and Capabilities
//!
//! Manages agent capabilities and registration.

use super::QualityRequirements;
use serde_json::Value as JsonValue;

/// Agent manifest manager
pub struct ManifestManager {
    /// Registered manifests
    manifests: Vec<AgentManifest>,
}

/// Agent manifest
#[derive(Debug, Clone)]
pub struct AgentManifest {
    /// Agent name
    pub name: String,

    /// Version
    pub version: String,

    /// Capabilities
    pub capabilities: Vec<Capability>,

    /// Supported protocols
    pub supported_protocols: Vec<Protocol>,

    /// Quality requirements
    pub quality_requirements: QualityRequirements,
}

/// Agent capability
#[derive(Debug, Clone)]
pub struct Capability {
    /// Capability name
    pub name: String,

    /// Description
    pub description: String,

    /// Input schema
    pub input_schema: JsonValue,

    /// Output schema
    pub output_schema: JsonValue,
}

/// Supported protocols
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Protocol {
    AgentsMd,
    Mcp,
    Http,
    WebSocket,
}

impl Default for ManifestManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ManifestManager {
    /// Create new manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            manifests: Vec::new(),
        }
    }

    /// Register manifest
    pub fn register(&mut self, manifest: AgentManifest) {
        self.manifests.push(manifest);
    }

    /// Get all manifests
    #[must_use]
    pub fn get_manifests(&self) -> &[AgentManifest] {
        &self.manifests
    }

    /// Find manifest by name
    #[must_use]
    pub fn find_by_name(&self, name: &str) -> Option<&AgentManifest> {
        self.manifests.iter().find(|m| m.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use serde_json::json;

    #[test]
    fn test_manifest_manager() {
        let mut manager = ManifestManager::new();

        let manifest = AgentManifest {
            name: "test_agent".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec![],
            supported_protocols: vec![Protocol::AgentsMd],
            quality_requirements: QualityRequirements::default(),
        };

        manager.register(manifest);
        assert_eq!(manager.get_manifests().len(), 1);
        assert!(manager.find_by_name("test_agent").is_some());
    }
}
