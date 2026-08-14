// Lean 4 heuristic analysis for TDG scoring
// Analyzes: theorems, lemmas, defs, sorry usage, tactic complexity, imports

impl TdgAnalyzerAst {
    #[allow(clippy::cast_possible_truncation)]
    fn analyze_lean_heuristic(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        score.confidence *= 0.9; // Pattern-based but well-defined

        let lines: Vec<&str> = source.lines().collect();
        let total_lines = lines.len().max(1);

        // ── Structural complexity: tactic nesting, pattern matches, branching ──
        let mut cyclomatic = 1u32;
        let mut max_nesting = 0u32;
        #[allow(unused_assignments)]
        let mut current_nesting = 0u32;
        let mut max_proof_length = 0usize;
        let mut current_proof_lines = 0usize;
        let mut in_proof = false;

        for line in &lines {
            let trimmed = line.trim();

            // Track proof blocks (started by := by, ended by next top-level decl)
            if trimmed.contains(":= by") || trimmed == "by" {
                in_proof = true;
                current_proof_lines = 0;
            } else if in_proof
                && !trimmed.is_empty()
                && !trimmed.starts_with("--")
                && (trimmed.starts_with("def ")
                    || trimmed.starts_with("theorem ")
                    || trimmed.starts_with("lemma ")
                    || trimmed.starts_with("structure ")
                    || trimmed.starts_with("class ")
                    || trimmed.starts_with("namespace ")
                    || trimmed.starts_with("end ")
                    || trimmed.starts_with("section "))
            {
                max_proof_length = max_proof_length.max(current_proof_lines);
                in_proof = false;
            }

            if in_proof {
                current_proof_lines += 1;
            }

            // Branching and control flow in tactics
            if trimmed.starts_with("if ")
                || trimmed.contains(" if ")
                || trimmed.starts_with("match ")
                || trimmed.contains(" match ")
                || trimmed.starts_with("| ")
                || trimmed.starts_with("case ")
            {
                cyclomatic += 1;
            }

            // Tactic complexity
            if trimmed.starts_with("by ")
                || trimmed.starts_with("calc")
                || trimmed.starts_with("have ")
                || trimmed.starts_with("show ")
                || trimmed.starts_with("suffices ")
                || trimmed.starts_with("obtain ")
            {
                cyclomatic += 1;
            }

            // Indentation-based nesting (Lean uses indentation)
            let indent = line.len() - line.trim_start().len();
            let indent_level = (indent / 2) as u32;
            current_nesting = indent_level;
            max_nesting = max_nesting.max(current_nesting);
        }
        max_proof_length = max_proof_length.max(current_proof_lines);

        score.structural_complexity = self.score_structural_complexity(
            cyclomatic,
            max_nesting.min(20),
            max_nesting as usize,
            max_proof_length,
            tracker,
        );

        // ── Semantic complexity: type universe levels, dependent types, tactics ──
        let mut universe_count = 0u32;
        let mut tactic_count = 0u32;

        for line in &lines {
            let trimmed = line.trim();
            if trimmed.starts_with("--") {
                continue;
            }
            if trimmed.contains("Type") || trimmed.contains("Prop") || trimmed.contains("Sort") {
                universe_count += 1;
            }
            if trimmed.starts_with("simp")
                || trimmed.starts_with("rw")
                || trimmed.starts_with("exact")
                || trimmed.starts_with("apply")
                || trimmed.starts_with("intro")
                || trimmed.starts_with("omega")
                || trimmed.starts_with("ring")
                || trimmed.starts_with("norm_num")
                || trimmed.starts_with("decide")
                || trimmed.starts_with("aesop")
            {
                tactic_count += 1;
            }
        }

        score.semantic_complexity = self.score_semantic_complexity(
            universe_count as usize,
            tactic_count,
            max_nesting.min(10),
            tracker,
        );

        // ── Duplication ──
        score.duplication_ratio = self.analyze_duplication_ast(source, Language::Lean, tracker);

        // ── Coupling: import count + open count ──
        let import_count = lines
            .iter()
            .filter(|l| {
                let t = l.trim();
                t.starts_with("import ") || t.starts_with("open ")
            })
            .count() as u32;

        let namespace_count = lines
            .iter()
            .filter(|l| l.trim().starts_with("namespace "))
            .count() as u32;

        score.coupling_score =
            self.score_coupling(import_count, namespace_count, 0, tracker);

        // ── Documentation: doc comments (/--, /-!, --) ──
        let doc_lines = lines
            .iter()
            .filter(|l| {
                let t = l.trim();
                t.starts_with("/--")
                    || t.starts_with("/-!")
                    || t.starts_with("--")
            })
            .count() as u32;

        let def_count = lines
            .iter()
            .filter(|l| {
                let t = l.trim();
                t.starts_with("def ")
                    || t.starts_with("theorem ")
                    || t.starts_with("lemma ")
                    || t.starts_with("structure ")
                    || t.starts_with("class ")
                    || t.starts_with("inductive ")
                    || t.starts_with("axiom ")
                    || t.starts_with("instance ")
            })
            .count() as u32;

        score.doc_coverage = self.score_documentation(
            doc_lines,
            def_count.max(1),
            doc_lines,
            total_lines as u32,
            tracker,
        );

        // ── Consistency: indentation style ──
        let mut spaces_2 = 0u32;
        let mut spaces_4 = 0u32;
        for line in &lines {
            if line.starts_with("  ") && !line.starts_with("    ") {
                spaces_2 += 1;
            }
            if line.starts_with("    ") {
                spaces_4 += 1;
            }
        }
        let total_indented = spaces_2 + spaces_4;
        if total_indented > 0 {
            let dominant = spaces_2.max(spaces_4) as f32 / total_indented as f32;
            score.consistency_score = dominant * self.config.weights.consistency;
        } else {
            score.consistency_score = self.config.weights.consistency;
        }

        // ── Entropy ──
        score.entropy_score = self.score_entropy_analysis(source, Language::Lean, tracker);

        Ok(())
    }
}
// `count_lean_sorry_ast` and its two helpers were a byte-for-byte copy of the
// counter in `analyzer_simple_helpers.rs`: one rule, two implementations, and
// only one of them reachable from the MCP surface. Both copies are gone; the
// counter is `crate::tdg::critical_defect_gate::count_lean_sorry`, called by the
// shared gate that both `analyze_source` implementations run.
