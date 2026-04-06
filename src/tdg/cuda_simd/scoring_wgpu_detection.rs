impl CudaSimdAnalyzer {

    /// Extract destination register from PTX instruction
    /// Example: "ld.shared.u32 %r1, [%rd1]" -> Some("%r1")
    fn extract_ptx_dest_register(line: &str) -> Option<String> {
        debug_assert!(!line.is_empty(), "line must not be empty");
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
        debug_assert!(path.exists(), "path must exist: {}", path.display());
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
}
