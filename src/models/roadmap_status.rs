// ItemStatus enum with alias support and Levenshtein-based typo suggestions
//
// Included by roadmap.rs — shares parent scope, no `use` imports needed.

/// Item status enumeration with alias support (Part A: YAML Parsing Resilience)
///
/// Aliases make YAML input forgiving; the accepted set is
/// [`ItemStatus::STATUS_TABLE`], which is also what `pmat work list-statuses`
/// prints and what `docs/roadmap-schema.md` documents. This doc comment
/// deliberately does not repeat the list: an earlier copy here and a third in
/// the `list-statuses` handler both drifted from `from_string`, which is the
/// defect GH #628 was about.
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
    /// The status vocabulary, as canonical value / aliases / description.
    ///
    /// Single source for everything user-facing: `pmat work list-statuses`
    /// renders it, and `test_status_table_matches_valid_values` pins it against
    /// [`Self::valid_values`] in both directions, so an alias can no longer be
    /// accepted by `from_string` while the command that documents the
    /// vocabulary omits it (`working` was missing for exactly that reason).
    pub const STATUS_TABLE: &'static [(&'static str, &'static str, &'static str)] = &[
        (
            "planned",
            "todo, open, pending, new",
            "Task not yet started",
        ),
        (
            "inprogress",
            "in-progress, wip, active, started, working",
            "Currently being worked on",
        ),
        (
            "blocked",
            "stuck, waiting, on-hold",
            "Cannot proceed (waiting on something)",
        ),
        (
            "review",
            "reviewing, pr, pending-review",
            "Ready for or in code review",
        ),
        (
            "completed",
            "done, finished, closed",
            "Work finished successfully",
        ),
        (
            "cancelled",
            "canceled, dropped, wontfix",
            "Work abandoned or not needed",
        ),
    ];

    /// Parse status from string with alias support
    ///
    /// Returns helpful error messages with suggestions for typos
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn from_string(s: &str) -> Result<Self, String> {
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
                // Rank over the *full* accepted set: this previously used a
                // hand-maintained list of 10 values while `from_string` accepted
                // 27, so a single-character typo of any of the other 17 aliases
                // was confidently pointed at the wrong word ("wontfixx" suggested
                // "done"). Ties break toward a canonical status, because the
                // widened pool otherwise lets an obscure alias win on list order
                // alone — 'obsolete' is distance 5 from both 'completed' and
                // 'on-hold', and should keep suggesting the canonical one.
                let suggestion = Self::valid_values()
                    .iter()
                    .min_by_key(|v| {
                        let distance = levenshtein_distance(&normalized, &v.replace('-', ""));
                        (distance, !Self::CANONICAL_VALUES.contains(v))
                    })
                    .map(|s| format!(" (did you mean '{}'?)", s))
                    .unwrap_or_default();

                Err(format!(
                    "unknown status '{}'{}\n\nValid values: {}\n\
                     (case-insensitive; '-' and '_' are ignored)",
                    s,
                    suggestion,
                    Self::valid_values().join(", ")
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

    /// The six canonical spellings, preferred when a typo suggestion ties.
    const CANONICAL_VALUES: [&'static str; 6] = [
        "planned",
        "inprogress",
        "blocked",
        "review",
        "completed",
        "cancelled",
    ];

    /// Get all valid status strings for help text
    ///
    /// Must stay in sync with [`ItemStatus::from_string`]; the round-trip is
    /// guarded by `test_valid_values_all_parse`.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn valid_values() -> &'static [&'static str] {
        &[
            "planned",
            "todo",
            "open",
            "pending",
            "new",
            "inprogress",
            "in-progress",
            "wip",
            "active",
            "started",
            "working",
            "blocked",
            "stuck",
            "waiting",
            "on-hold",
            "review",
            "reviewing",
            "pr",
            "pending-review",
            "completed",
            "done",
            "finished",
            "closed",
            "cancelled",
            "canceled",
            "dropped",
            "wontfix",
        ]
    }
}

/// Simple Levenshtein distance for typo suggestions
fn levenshtein_distance(a: &str, b: &str) -> usize {
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
