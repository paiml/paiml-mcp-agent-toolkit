// Roadmap type definitions: structs, enums, constants, serde helpers
//
// Included by roadmap.rs — shares parent scope, no `use` imports needed.

/// Roadmap YAML schema version
pub const ROADMAP_VERSION: &str = "1.0";

/// Main roadmap structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Roadmap {
    /// Schema version
    pub roadmap_version: String,

    /// GitHub integration enabled
    #[serde(
        default = "default_github_enabled",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub github_enabled: bool,

    /// GitHub repository (owner/repo)
    pub github_repo: Option<String>,

    /// List of roadmap items (tickets)
    #[serde(default)]
    pub roadmap: Vec<RoadmapItem>,
}

fn default_github_enabled() -> bool {
    true
}

/// Lenient boolean deserializer: accepts both native YAML booleans and quoted strings "true"/"false"
fn deserialize_bool_lenient<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;
    struct BoolVisitor;
    impl<'de> de::Visitor<'de> for BoolVisitor {
        type Value = bool;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a boolean or string \"true\"/\"false\"")
        }
        fn visit_bool<E: de::Error>(self, v: bool) -> Result<bool, E> {
            Ok(v)
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<bool, E> {
            match v {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err(E::custom(format!("expected true/false, got '{}'", v))),
            }
        }
    }
    deserializer.deserialize_any(BoolVisitor)
}

fn default_timestamp() -> String {
    "1970-01-01T00:00:00Z".to_string()
}

impl Default for Roadmap {
    fn default() -> Self {
        Self {
            roadmap_version: ROADMAP_VERSION.to_string(),
            github_enabled: true,
            github_repo: None,
            roadmap: Vec::new(),
        }
    }
}

/// Individual roadmap item (ticket/issue)
///
/// Note: Extra fields in YAML (like description, implementation, references)
/// are silently ignored to support backward compatibility with older roadmap formats.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoadmapItem {
    /// Unique ID (e.g., "GH-8", "PERF-001", "EPIC-001")
    pub id: String,

    /// GitHub issue number (null if YAML-only)
    pub github_issue: Option<u64>,

    /// Item type (task, epic, bug, etc.)
    #[serde(default = "default_item_type")]
    pub item_type: ItemType,

    /// Title
    pub title: String,

    /// Current status
    pub status: ItemStatus,

    /// Priority level
    #[serde(default)]
    pub priority: Priority,

    /// Assigned to (GitHub username with @)
    pub assigned_to: Option<String>,

    /// Created timestamp (ISO 8601)
    #[serde(default = "default_timestamp")]
    pub created: String,

    /// Last updated timestamp (ISO 8601)
    #[serde(default = "default_timestamp")]
    pub updated: String,

    /// Path to specification file
    pub spec: Option<PathBuf>,

    /// Acceptance criteria (checklist)
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,

    /// Phases (for multi-phase work)
    #[serde(default, deserialize_with = "deserialize_phases")]
    pub phases: Vec<Phase>,

    /// Subtasks (for epic items)
    #[serde(default)]
    pub subtasks: Vec<Subtask>,

    /// Estimated effort (human-readable)
    pub estimated_effort: Option<String>,

    /// Labels/tags
    #[serde(default)]
    pub labels: Vec<String>,

    /// Additional notes/documentation (markdown)
    #[serde(default)]
    pub notes: Option<String>,
}

fn default_item_type() -> ItemType {
    ItemType::Task
}

/// Item type enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ItemType {
    Task,
    Epic,
    Bug,
    Feature,
    Enhancement,
    Documentation,
    Refactor,
}

/// Priority enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

/// Phase within a roadmap item
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Phase {
    /// Phase name
    pub name: String,

    /// Phase status
    pub status: ItemStatus,

    /// Estimated effort
    pub estimated_effort: Option<String>,

    /// Completion percentage (0-100)
    #[serde(default)]
    pub completion: u8,
}

/// Subtask within an epic
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Subtask {
    /// Subtask ID
    pub id: String,

    /// GitHub issue number (if synced)
    pub github_issue: Option<u64>,

    /// Subtask title
    pub title: String,

    /// Subtask status
    pub status: ItemStatus,

    /// Completion percentage (0-100)
    #[serde(default)]
    pub completion: u8,
}

/// Custom deserializer for phases that provides helpful error messages (issue #130)
fn deserialize_phases<'de, D>(deserializer: D) -> Result<Vec<Phase>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, SeqAccess, Visitor};
    use std::fmt;

    struct PhasesVisitor;

    impl<'de> Visitor<'de> for PhasesVisitor {
        type Value = Vec<Phase>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence of Phase structs")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut phases = Vec::new();
            let mut index = 0;

            while let Some(value) = seq.next_element::<serde_yaml_ng::Value>()? {
                match value {
                    serde_yaml_ng::Value::String(s) => {
                        return Err(de::Error::custom(format!(
                            "phases[{}]: invalid type. \
                            Phases must be structs with 'name' and 'status' fields.\n\n\
                            Example:\n  \
                            phases:\n    \
                            - name: \"{}\"\n      \
                            status: planned\n\n\
                            Found string: \"{}\"",
                            index, s, s
                        )));
                    }
                    serde_yaml_ng::Value::Mapping(_) => {
                        let phase: Phase =
                            serde_yaml_ng::from_value(value).map_err(de::Error::custom)?;
                        phases.push(phase);
                    }
                    _ => {
                        return Err(de::Error::custom(format!(
                            "phases[{}]: expected a Phase struct, found {:?}",
                            index, value
                        )));
                    }
                }
                index += 1;
            }

            Ok(phases)
        }
    }

    deserializer.deserialize_seq(PhasesVisitor)
}
