// ComparisonView - JSON export of diff reports

impl ComparisonView {
    /// Export diff report as JSON
    pub fn export_diff_json(&self) -> Result<String> {
        let mut report = serde_json::json!({
            "metadata": {
                "recording_a_name": self.name_a,
                "recording_b_name": self.name_b,
                "recording_a_frames": self.total_frames_a(),
                "recording_b_frames": self.total_frames_b(),
                "sync_mode": format!("{:?}", self.sync_mode),
                "divergence_point": self.find_divergence_point(),
            },
            "frame_diffs": []
        });

        let frame_diffs = self.build_frame_diffs();
        report["frame_diffs"] = serde_json::json!(frame_diffs);

        Ok(serde_json::to_string_pretty(&report)?)
    }

    fn build_frame_diffs(&self) -> Vec<serde_json::Value> {
        let max_frames = self.total_frames_max();
        let snapshots_a = self.player_a.recording().snapshots();
        let snapshots_b = self.player_b.recording().snapshots();

        (0..max_frames)
            .map(|frame| {
                let vars_a = snapshots_a.get(frame).map(|s| &s.variables);
                let vars_b = snapshots_b.get(frame).map(|s| &s.variables);
                let diff = compute_variable_diff(vars_a, vars_b);

                serde_json::json!({
                    "frame": frame,
                    "variable_diff": diff,
                })
            })
            .collect()
    }
}
