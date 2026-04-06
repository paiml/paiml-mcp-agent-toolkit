// ComparisonView - Divergence detection

impl ComparisonView {
    /// Find the first frame where recordings diverge
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn find_divergence_point(&self) -> Option<usize> {
        let max_frames = self.total_frames_min();

        for frame in 0..max_frames {
            let snapshot_a = &self.player_a.recording().snapshots()[frame];
            let snapshot_b = &self.player_b.recording().snapshots()[frame];

            if snapshot_a.variables != snapshot_b.variables {
                return Some(frame);
            }

            if snapshot_a.instruction_pointer != snapshot_b.instruction_pointer {
                return Some(frame);
            }

            if snapshot_a.stack_frames.len() != snapshot_b.stack_frames.len() {
                return Some(frame);
            }

            for (frame_a, frame_b) in snapshot_a
                .stack_frames
                .iter()
                .zip(snapshot_b.stack_frames.iter())
            {
                if frame_a.name != frame_b.name {
                    return Some(frame);
                }
            }
        }

        None
    }
}
