// ── ExclusionSummary implementation ──────────────────────────────────────────
// Included from coverage_exclusion.rs — do NOT add `use` imports or `#!` attributes here.

impl ExclusionSummary {
    pub fn from_results(excluded: &[&QueryResult]) -> Self {
        debug_assert!(!excluded.is_empty(), "excluded must not be empty");
        let mut summary = Self::default();
        let mut cov_off_files: HashSet<&str> = HashSet::new();
        let mut dead_files: HashSet<&str> = HashSet::new();
        let mut make_files: HashSet<&str> = HashSet::new();

        for r in excluded {
            match r.coverage_exclusion {
                CoverageExclusion::CoverageOff => {
                    summary.coverage_off_count += 1;
                    cov_off_files.insert(&r.file_path);
                }
                CoverageExclusion::DeadCode => {
                    summary.dead_code_count += 1;
                    dead_files.insert(&r.file_path);
                }
                CoverageExclusion::MakefileExcluded => {
                    summary.makefile_count += 1;
                    make_files.insert(&r.file_path);
                }
                CoverageExclusion::None => {}
            }
        }
        summary.coverage_off_files = cov_off_files.len();
        summary.dead_code_files = dead_files.len();
        summary.makefile_files = make_files.len();
        summary
    }

    pub fn total(&self) -> usize {
        self.coverage_off_count + self.dead_code_count + self.makefile_count
    }

    pub fn is_empty(&self) -> bool {
        self.total() == 0
    }
}
