// ItemStatus enum with alias support and Levenshtein-based typo suggestions
//
// Included by roadmap.rs — shares parent scope, no `use` imports needed.

/// Item status enumeration with alias support (Part A: YAML Parsing Resilience)
///
/// Supports multiple aliases for user-friendly YAML input:
/// - completed: "done", "finished", "closed"
/// - inprogress: "in_progress", "in-progress", "wip", "active", "started"
/// - planned: "todo", "open", "pending", "new"
/// - blocked: "stuck", "waiting", "on-hold"
/// - review: "reviewing", "pr", "pending-review"
/// - cancelled: "canceled", "dropped", "wontfix"
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ItemStatus {
    Planned,
    InProgress,
    Blocked,
    Review,
    Completed,
    Cancelled,
}

impl<'de> serde::Deserialize<'de> for ItemStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ItemStatus::from_string(&s).map_err(serde::de::Error::custom)
    }
}

impl ItemStatus {
    /// Parse status from string with alias support
    ///
    /// Returns helpful error messages with suggestions for typos
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn from_string(s: &str) -> Result<Self, String> {
        debug_assert!(!s.is_empty(), "s must not be empty");
        // Normalize: lowercase, remove hyphens/underscores, trim whitespace
        let normalized = s.to_lowercase().replace(['-', '_'], "").trim().to_string();

        match normalized.as_str() {
            // Completed aliases
            "completed" | "done" | "finished" | "closed" => Ok(Self::Completed),

            // InProgress aliases
            "inprogress" | "wip" | "active" | "started" | "working" => Ok(Self::InProgress),

            // Planned aliases
            "planned" | "todo" | "open" | "pending" | "new" => Ok(Self::Planned),

            // Blocked aliases
            "blocked" | "stuck" | "waiting" | "onhold" => Ok(Self::Blocked),

            // Review aliases
            "review" | "reviewing" | "pr" | "pendingreview" => Ok(Self::Review),

            // Cancelled aliases
            "cancelled" | "canceled" | "dropped" | "wontfix" => Ok(Self::Cancelled),

            _ => {
                // Provide helpful suggestion using Levenshtein distance
                let valid_statuses = [
                    "completed",
                    "done",
                    "inprogress",
                    "wip",
                    "planned",
                    "todo",
                    "blocked",
                    "stuck",
                    "review",
                    "cancelled",
                ];
                let suggestion = valid_statuses
                    .iter()
                    .min_by_key(|v| levenshtein_distance(&normalized, v))
                    .map(|s| format!(" (did you mean '{}'?)", s))
                    .unwrap_or_default();

                Err(format!(
                    "unknown status '{}'{}\n\nValid values: completed, done, inprogress, wip, planned, todo, blocked, review, cancelled",
                    s, suggestion
                ))
            }
        }
    }

    /// Validate a state transition against the work-dbc-v1 adjacency matrix.
    ///
    /// Valid transitions (contract §work_lifecycle):
    ///   Planned    → InProgress, Cancelled
    ///   InProgress → Blocked, Review, Completed
    ///   Blocked    → InProgress
    ///   Review     → InProgress, Completed
    ///   Completed  → (terminal, no outgoing)
    ///   Cancelled  → (terminal, no outgoing)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn can_transition_to(self, target: Self) -> bool {
        matches!(
            (self, target),
            (Self::Planned, Self::InProgress)
                | (Self::Planned, Self::Cancelled)
                | (Self::InProgress, Self::Blocked)
                | (Self::InProgress, Self::Review)
                | (Self::InProgress, Self::Completed)
                | (Self::Blocked, Self::InProgress)
                | (Self::Review, Self::InProgress)
                | (Self::Review, Self::Completed)
        )
    }

    /// Human-readable name for display
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Planned => "Planned",
            Self::InProgress => "InProgress",
            Self::Blocked => "Blocked",
            Self::Review => "Review",
            Self::Completed => "Completed",
            Self::Cancelled => "Cancelled",
        }
    }

    /// Get all valid status strings for help text
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn valid_values() -> &'static [&'static str] {
        &[
            "completed",
            "done",
            "finished",
            "closed",
            "inprogress",
            "in_progress",
            "wip",
            "active",
            "planned",
            "todo",
            "open",
            "pending",
            "blocked",
            "stuck",
            "waiting",
            "review",
            "reviewing",
            "pr",
            "cancelled",
            "canceled",
            "dropped",
        ]
    }
}

/// Simple Levenshtein distance for typo suggestions
fn levenshtein_distance(a: &str, b: &str) -> usize {
    debug_assert!(!a.is_empty(), "a must not be empty");
    debug_assert!(!b.is_empty(), "b must not be empty");
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut matrix = vec![vec![0usize; b_len + 1]; a_len + 1];

    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                matrix[i - 1][j - 1] + cost,
            );
        }
    }

    matrix[a_len][b_len]
}
