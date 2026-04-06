// ComparisonView - Rendering and variable diff

/// Compare two variable maps and return diff status for each variable
fn diff_both_maps(
    vars_a: &HashMap<String, serde_json::Value>,
    vars_b: &HashMap<String, serde_json::Value>,
) -> HashMap<String, DiffStatus> {
    let mut diff = HashMap::new();
    for (name, value_a) in vars_a {
        let status = match vars_b.get(name) {
            Some(value_b) if value_a == value_b => DiffStatus::Same,
            Some(_) => DiffStatus::Modified,
            None => DiffStatus::Removed,
        };
        diff.insert(name.clone(), status);
    }
    for name in vars_b.keys() {
        if !vars_a.contains_key(name) {
            diff.insert(name.clone(), DiffStatus::Added);
        }
    }
    diff
}

fn mark_all_as(vars: &HashMap<String, serde_json::Value>, status: DiffStatus) -> HashMap<String, DiffStatus> {
    vars.keys().map(|name| (name.clone(), status.clone())).collect()
}

/// Compute variable diff between two optional variable maps
fn compute_variable_diff(
    vars_a: Option<&HashMap<String, serde_json::Value>>,
    vars_b: Option<&HashMap<String, serde_json::Value>>,
) -> HashMap<String, DiffStatus> {
    match (vars_a, vars_b) {
        (Some(a), Some(b)) => diff_both_maps(a, b),
        (Some(a), None) => mark_all_as(a, DiffStatus::Removed),
        (None, Some(b)) => mark_all_as(b, DiffStatus::Added),
        (None, None) => HashMap::new(),
    }
}

impl ComparisonView {
    /// Render split view with both recordings side-by-side
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn render_split(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "Recording A: {}    |    Recording B: {}\n",
            self.name_a, self.name_b
        ));
        output.push_str(&format!(
            "Frame {}/{}          |    Frame {}/{}\n",
            self.current_frame_a(),
            self.total_frames_a(),
            self.current_frame_b(),
            self.total_frames_b()
        ));

        if self.recording_a_exhausted() {
            output.push_str("END                 |    \n");
        }
        if self.recording_b_exhausted() {
            output.push_str("                    |    END\n");
        }

        output
    }

    /// Calculate variable diff between current snapshots
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn variable_diff(&self) -> HashMap<String, DiffStatus> {
        let snapshot_a = if !self.recording_a_exhausted() {
            Some(self.player_a.current_snapshot())
        } else {
            None
        };

        let snapshot_b = if !self.recording_b_exhausted() {
            Some(self.player_b.current_snapshot())
        } else {
            None
        };

        compute_variable_diff(
            snapshot_a.as_ref().map(|s| &s.variables),
            snapshot_b.as_ref().map(|s| &s.variables),
        )
    }
}
