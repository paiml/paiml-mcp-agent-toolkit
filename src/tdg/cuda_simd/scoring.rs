impl CudaSimdAnalyzer {

    /// Extract destination register from PTX instruction
    /// Example: "ld.shared.u32 %r1, [%rd1]" -> Some("%r1")
    fn extract_ptx_dest_register(line: &str) -> Option<String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let dest = parts[1].trim_end_matches(',');
            if dest.starts_with('%') {
                return Some(dest.to_string());
            }
        }
        None
    }

    /// Comprehensive WGPU/WGSL bug detection based on trueno research
    ///
    /// Detects (from trueno-explain/src/wgpu.rs and common WGSL bugs):
    ///
    /// ## P1 High (Performance)
    /// - WGPU_SMALL_WORKGROUP: Workgroup size too small (<64 threads)
    /// - WGPU_LARGE_WORKGROUP: Workgroup size too large (>1024 threads)
    /// - WGPU_NON_WARP_ALIGNED: Workgroup not multiple of 32 (warp waste)
    /// - WGPU_MISSING_WORKGROUP: No @workgroup_size attribute found
    /// - WGPU_NO_BOUNDS_CHECK: Global invocation without bounds check
    ///
    /// ## P2 Medium (Efficiency)
    /// - WGPU_EXCESSIVE_BARRIERS: Too many workgroupBarrier() calls
    /// - WGPU_UNIFORM_DIVERGENCE: Non-uniform control flow in workgroup
    fn detect_wgpu_memory_patterns(&self, content: &str, path: &Path, analysis: &mut FileAnalysis) {
        let lines: Vec<&str> = content.lines().collect();

        // Parse workgroup size from @workgroup_size(x, y, z)
        let mut workgroup_x = 1u32;
        let mut workgroup_y = 1u32;
        let mut workgroup_z = 1u32;
        let mut has_workgroup_size = false;

        // Count various patterns
        let mut barrier_count = 0u32;
        let mut has_bounds_check = false;
        let mut has_global_invocation = false;

        // Regex for workgroup_size
        let workgroup_regex = regex::Regex::new(
            r"@workgroup_size\s*\(\s*(\d+)(?:\s*,\s*(\d+))?(?:\s*,\s*(\d+))?\s*\)",
        )
        .ok();

        for (_line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Parse @workgroup_size
            if let Some(ref re) = workgroup_regex {
                if let Some(caps) = re.captures(trimmed) {
                    has_workgroup_size = true;
                    workgroup_x = caps.get(1).map_or(1, |m| m.as_str().parse().unwrap_or(1));
                    workgroup_y = caps.get(2).map_or(1, |m| m.as_str().parse().unwrap_or(1));
                    workgroup_z = caps.get(3).map_or(1, |m| m.as_str().parse().unwrap_or(1));
                }
            }

            // Count barriers
            if trimmed.contains("workgroupBarrier") || trimmed.contains("storageBarrier") {
                barrier_count += 1;
                analysis.barrier_safety.total_barriers += 1;
                analysis.barrier_safety.safe_barriers += 1;
            }

            // Detect global invocation usage
            if trimmed.contains("global_invocation_id") {
                has_global_invocation = true;
            }

            // Detect bounds checks
            if (trimmed.contains("if") || trimmed.contains("select"))
                && (trimmed.contains("<") || trimmed.contains(">="))
                && (trimmed.contains("size")
                    || trimmed.contains("len")
                    || trimmed.contains("count"))
            {
                has_bounds_check = true;
            }

            // Detect storage buffer accesses
            if trimmed.contains("storage")
                && (trimmed.contains("read") || trimmed.contains("write"))
            {
                analysis.coalescing.total_operations += 1;
                analysis.coalescing.coalesced_operations += 1;
            }

            // Detect array indexing
            if trimmed.contains('[') && trimmed.contains(']') {
                analysis.coalescing.total_operations += 1;
                analysis.coalescing.coalesced_operations += 1;
            }
        }

        let total_threads = workgroup_x * workgroup_y * workgroup_z;

        // ─────────────────────────────────────────────────────────────────
        // P1 HIGH: Missing workgroup_size
        // ─────────────────────────────────────────────────────────────────
        if !has_workgroup_size && content.contains("@compute") {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "WGPU_MISSING_WORKGROUP".to_string(),
                    description: "Compute shader missing @workgroup_size attribute".to_string(),
                    severity: DefectSeverity::P1Performance,
                    detection_method: "WGSL pattern analysis".to_string(),
                    resolved: false,
                    root_cause: Some(
                        "Default workgroup size (1,1,1) is extremely inefficient".to_string(),
                    ),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: None,
                suggestion: Some(
                    "Add @workgroup_size(256) or @workgroup_size(8, 8, 1)".to_string(),
                ),
            });
        }

        // ─────────────────────────────────────────────────────────────────
        // P1 HIGH: Small workgroup size (<64 threads)
        // ─────────────────────────────────────────────────────────────────
        if has_workgroup_size && total_threads < 64 {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "WGPU_SMALL_WORKGROUP".to_string(),
                    description: format!("Small workgroup size: {} threads (minimum: 64)", total_threads),
                    severity: DefectSeverity::P1Performance,
                    detection_method: "WGSL pattern analysis".to_string(),
                    resolved: false,
                    root_cause: Some("Low GPU occupancy, underutilization".to_string()),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: Some(format!("@workgroup_size({}, {}, {})", workgroup_x, workgroup_y, workgroup_z)),
                suggestion: Some(format!("Increase to at least 64 threads (e.g., @workgroup_size(64) or @workgroup_size(8, 8))")),
            });
        }

        // ─────────────────────────────────────────────────────────────────
        // P1 HIGH: Large workgroup size (>1024 threads)
        // ─────────────────────────────────────────────────────────────────
        if has_workgroup_size && total_threads > 1024 {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "WGPU_LARGE_WORKGROUP".to_string(),
                    description: format!(
                        "Large workgroup size: {} threads (max: 1024)",
                        total_threads
                    ),
                    severity: DefectSeverity::P1Performance,
                    detection_method: "WGSL pattern analysis".to_string(),
                    resolved: false,
                    root_cause: Some(
                        "May exceed hardware limits or cause register pressure".to_string(),
                    ),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: Some(format!(
                    "@workgroup_size({}, {}, {})",
                    workgroup_x, workgroup_y, workgroup_z
                )),
                suggestion: Some("Reduce to at most 1024 threads".to_string()),
            });
        }

        // ─────────────────────────────────────────────────────────────────
        // P1 HIGH: Non-warp-aligned workgroup
        // ─────────────────────────────────────────────────────────────────
        if has_workgroup_size && total_threads > 1 && total_threads % 32 != 0 {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "WGPU_NON_WARP_ALIGNED".to_string(),
                    description: format!(
                        "Workgroup size {} not multiple of 32 (warp size)",
                        total_threads
                    ),
                    severity: DefectSeverity::P1Performance,
                    detection_method: "WGSL pattern analysis".to_string(),
                    resolved: false,
                    root_cause: Some("Partial warp execution wastes GPU cycles".to_string()),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: Some(format!(
                    "@workgroup_size({}, {}, {})",
                    workgroup_x, workgroup_y, workgroup_z
                )),
                suggestion: Some("Align to multiple of 32 (e.g., 64, 128, 256)".to_string()),
            });
        }

        // ─────────────────────────────────────────────────────────────────
        // P1 HIGH: Missing bounds check
        // ─────────────────────────────────────────────────────────────────
        if has_global_invocation && !has_bounds_check {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "WGPU_NO_BOUNDS_CHECK".to_string(),
                    description: "Compute shader uses global_invocation_id without bounds check"
                        .to_string(),
                    severity: DefectSeverity::P1Performance,
                    detection_method: "WGSL pattern analysis".to_string(),
                    resolved: false,
                    root_cause: Some("Excess threads may access out-of-bounds memory".to_string()),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: None,
                suggestion: Some("Add: if (gid.x < params.size) { ... }".to_string()),
            });
        }

        // ─────────────────────────────────────────────────────────────────
        // P2 MEDIUM: Excessive barriers
        // ─────────────────────────────────────────────────────────────────
        if barrier_count > 5 {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "WGPU_EXCESSIVE_BARRIERS".to_string(),
                    description: format!(
                        "{} barrier calls may indicate inefficient algorithm",
                        barrier_count
                    ),
                    severity: DefectSeverity::P2Efficiency,
                    detection_method: "WGSL pattern analysis".to_string(),
                    resolved: false,
                    root_cause: Some("Each barrier synchronizes entire workgroup".to_string()),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: Some(format!(
                    "{} workgroupBarrier/storageBarrier calls",
                    barrier_count
                )),
                suggestion: Some(
                    "Consider restructuring algorithm to reduce synchronization".to_string(),
                ),
            });
        }

        if analysis.coalescing.total_operations > 0 {
            analysis.coalescing.efficiency = analysis.coalescing.coalesced_operations as f64
                / analysis.coalescing.total_operations as f64;
        }

        if analysis.barrier_safety.total_barriers > 0 {
            analysis.barrier_safety.safety_score = analysis.barrier_safety.safe_barriers as f64
                / analysis.barrier_safety.total_barriers as f64;
        }
    }

    fn check_flash_attention_tile_size(
        &self,
        content: &str,
        path: &Path,
        analysis: &mut FileAnalysis,
    ) {
        let has_flash = content.contains("FlashAttention")
            || content.contains("flash_attention")
            || content.contains("tiled_attention");
        if !has_flash {
            return;
        }
        let tile_kv = match self.extract_value(content, "tile_kv") {
            Some(v) => v,
            None => return,
        };
        let head_dim = match self.extract_value(content, "head_dim") {
            Some(v) => v,
            None => return,
        };
        if tile_kv >= head_dim {
            return;
        }
        if let Some(defect_class) = self.taxonomy.get("PAR-041") {
            analysis.defects.push(DetectedDefect {
                defect_class: defect_class.clone(),
                file_path: path.to_path_buf(),
                line: None,
                snippet: Some(format!("tile_kv ({}) < head_dim ({})", tile_kv, head_dim)),
                suggestion: Some(format!("Set tile_kv >= head_dim (at least {})", head_dim)),
            });
        }
    }

    fn check_missing_tensor_core(
        &self,
        content: &str,
        path: &Path,
        analysis: &mut FileAnalysis,
    ) {
        let has_matmul = content.contains("matmul") || content.contains("gemm");
        let has_tensor = content.contains("wmma")
            || content.contains("mma")
            || content.contains("tensor_core");
        if !has_matmul || has_tensor {
            return;
        }
        if let Some(defect_class) = self.taxonomy.get("PAR-034") {
            analysis.defects.push(DetectedDefect {
                defect_class: defect_class.clone(),
                file_path: path.to_path_buf(),
                line: None,
                snippet: Some("Matrix multiplication without Tensor Core".to_string()),
                suggestion: Some(
                    "Consider using wmma or mma instructions for better performance".to_string(),
                ),
            });
        }
    }

    fn detect_known_patterns(&self, content: &str, path: &Path, analysis: &mut FileAnalysis) {
        self.check_flash_attention_tile_size(content, path, analysis);
        self.check_missing_tensor_core(content, path, analysis);
    }

    fn extract_value(&self, content: &str, name: &str) -> Option<usize> {
        // Simple pattern matching for variable assignments
        let patterns = [
            format!("{} = ", name),
            format!("{}=", name),
            format!("const {} = ", name),
            format!("let {} = ", name),
        ];

        for pattern in &patterns {
            if let Some(pos) = content.find(pattern) {
                let after = content.get(pos + pattern.len()..).unwrap_or_default();
                let value_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(value) = value_str.parse() {
                    return Some(value);
                }
            }
        }
        None
    }

    /// Check if any .rs file in src/backends contains SAFETY comments
    fn has_safety_comments(path: &Path) -> bool {
        let backends = path.join("src/backends");
        let entries = match std::fs::read_dir(&backends) {
            Ok(e) => e,
            Err(_) => return false,
        };
        entries.filter_map(Result::ok).any(|entry| {
            entry.path().extension().is_some_and(|e| e == "rs")
                && std::fs::read_to_string(entry.path())
                    .map(|c| c.contains("// SAFETY:") || c.contains("/// SAFETY:"))
                    .unwrap_or(false)
        })
    }

    /// Detect Rust project quality patterns for enhanced scoring
    fn detect_rust_patterns(&self, path: &Path) -> RustProjectPatterns {
        let mut patterns = RustProjectPatterns::default();

        patterns.has_cargo_lock = path.join("Cargo.lock").exists();
        patterns.has_rust_toolchain =
            path.join("rust-toolchain.toml").exists() || path.join("rust-toolchain").exists();
        patterns.has_criterion_benches = path.join("benches").exists()
            && std::fs::read_dir(path.join("benches"))
                .map(|d| {
                    d.filter_map(Result::ok)
                        .any(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
                })
                .unwrap_or(false);
        patterns.has_github_ci = path.join(".github/workflows").exists();
        patterns.has_proptest_regressions = path.join("proptest-regressions").exists();
        patterns.has_changelog =
            path.join("CHANGELOG.md").exists() || path.join("CHANGELOG").exists();
        patterns.has_golden_traces = path.join("golden_traces").exists();
        patterns.has_safety_comments =
            path.join("src/backends").exists() && Self::has_safety_comments(path);
        patterns.has_miri_config = path.join(".cargo/config.toml").exists()
            && std::fs::read_to_string(path.join(".cargo/config.toml"))
                .map(|c| c.contains("miri"))
                .unwrap_or(false);

        patterns
    }

    fn score_falsifiability(
        defects: &[DetectedDefect],
        barrier_safety: &BarrierSafetyResult,
        patterns: &RustProjectPatterns,
    ) -> FalsifiabilityScore {
        let p0_defects = defects
            .iter()
            .filter(|d| d.defect_class.severity == DefectSeverity::P0Critical)
            .count();

        FalsifiabilityScore {
            barrier_safety: if barrier_safety.unsafe_barriers.is_empty() {
                5.0
            } else {
                5.0 * barrier_safety.safety_score
            },
            bounds_verification: if p0_defects == 0 {
                if patterns.has_safety_comments { 5.0 } else { 4.0 }
            } else {
                2.5
            },
            divergence_testing: if patterns.has_proptest_regressions { 5.0 } else { 2.5 },
            memory_race_detection: if patterns.has_miri_config { 5.0 } else { 2.5 },
            occupancy_bounds: 5.0,
        }
    }

    fn score_reproducibility(patterns: &RustProjectPatterns) -> ReproducibilityScore {
        ReproducibilityScore {
            deterministic_output: if patterns.has_golden_traces { 8.0 } else { 4.0 },
            version_pinning: if patterns.has_cargo_lock && patterns.has_rust_toolchain {
                5.0
            } else if patterns.has_cargo_lock {
                3.5
            } else {
                2.5
            },
            hardware_specification: 2.5,
            benchmark_harness: if patterns.has_criterion_benches { 4.0 } else { 2.0 },
            ci_cd_integration: if patterns.has_github_ci { 3.0 } else { 1.5 },
        }
    }

    fn score_statistical_rigor(patterns: &RustProjectPatterns) -> StatisticalRigorScore {
        if patterns.has_criterion_benches {
            StatisticalRigorScore {
                warmup_iterations: 4.0,
                sample_count: 4.0,
                outlier_analysis: 4.0,
                confidence_intervals: 3.0,
            }
        } else {
            StatisticalRigorScore {
                warmup_iterations: 2.0,
                sample_count: 2.0,
                outlier_analysis: 2.0,
                confidence_intervals: 1.5,
            }
        }
    }

    fn score_historical_integrity(
        defects: &[DetectedDefect],
        patterns: &RustProjectPatterns,
    ) -> HistoricalIntegrityScore {
        HistoricalIntegrityScore {
            fault_lineage: if patterns.has_changelog {
                4.0
            } else if !defects.is_empty() {
                3.0
            } else {
                2.0
            },
            regression_tests: if patterns.has_proptest_regressions { 3.0 } else { 1.5 },
            root_cause_documentation: if patterns.has_changelog { 3.0 } else { 1.5 },
        }
    }

    fn calculate_score(
        &self,
        defects: &[DetectedDefect],
        barrier_safety: &BarrierSafetyResult,
        coalescing: &CoalescingResult,
        path: &Path,
    ) -> PopperScore {
        let patterns = self.detect_rust_patterns(path);

        let falsifiability = Self::score_falsifiability(defects, barrier_safety, &patterns);
        let reproducibility = Self::score_reproducibility(&patterns);
        let transparency = TransparencyScore {
            ptx_inspection: 3.0,
            register_allocation: 2.5,
            occupancy_calculation: 2.5,
            memory_layout: 2.0,
        };
        let statistical_rigor = Self::score_statistical_rigor(&patterns);
        let historical_integrity = Self::score_historical_integrity(defects, &patterns);
        let gpu_simd_specific = GpuSimdSpecificScore {
            warp_efficiency: 1.0,
            memory_throughput: coalescing.efficiency * 2.0,
            instruction_mix: 0.5,
        };

        PopperScore::calculate(
            falsifiability,
            reproducibility,
            transparency,
            statistical_rigor,
            historical_integrity,
            gpu_simd_specific,
        )
    }

    fn build_kaizen_metrics(&self, defects: &[DetectedDefect]) -> KaizenMetrics {
        let ticket_references: Vec<String> = defects
            .iter()
            .map(|d| d.defect_class.ticket_id.clone())
            .collect();

        let resolved_count = defects.iter().filter(|d| d.defect_class.resolved).count() as u32;

        KaizenMetrics {
            tickets_resolved: resolved_count,
            mttd: 24.0,            // Default estimate
            mttf: 48.0,            // Default estimate
            escape_rate: 0.05,     // 5% default
            regression_rate: 0.02, // 2% default
            ticket_references,
        }
    }

    /// Check if quality gate passes
    #[must_use]
    pub fn passes_quality_gate(&self, result: &CudaSimdTdgResult) -> bool {
        if !result.score.gateway_passed {
            return false;
        }

        if self.config.fail_on_p0 {
            let has_p0 = result
                .defects
                .iter()
                .any(|d| d.defect_class.severity == DefectSeverity::P0Critical);
            if has_p0 {
                return false;
            }
        }

        result.score.total >= self.config.min_score
    }
}
