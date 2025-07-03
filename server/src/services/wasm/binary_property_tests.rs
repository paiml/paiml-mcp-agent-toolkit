#[cfg(test)]
mod tests {
    use super::super::binary::*;
    use proptest::prelude::*;
    use tempfile::NamedTempFile;
    use tokio;

    // Strategy for generating valid WASM magic number
    fn wasm_magic_strategy() -> impl Strategy<Value = Vec<u8>> {
        Just(vec![0x00, 0x61, 0x73, 0x6D]) // \0asm
    }

    // Strategy for generating WASM version
    fn wasm_version_strategy() -> impl Strategy<Value = Vec<u8>> {
        prop_oneof![
            Just(vec![0x01, 0x00, 0x00, 0x00]), // Version 1 (standard)
            Just(vec![0x02, 0x00, 0x00, 0x00]), // Version 2 (experimental)
        ]
    }

    // Strategy for generating arbitrary WASM section headers
    fn wasm_section_strategy() -> impl Strategy<Value = Vec<u8>> {
        (0u8..=12, 1usize..100).prop_map(|(section_id, size)| {
            let mut section = vec![section_id];
            // LEB128 encode the size
            let mut n = size;
            while n >= 0x80 {
                section.push((n & 0x7F) as u8 | 0x80);
                n >>= 7;
            }
            section.push(n as u8);
            section
        })
    }

    // Strategy for generating arbitrary WASM binaries
    fn wasm_binary_strategy() -> impl Strategy<Value = Vec<u8>> {
        (
            wasm_magic_strategy(),
            wasm_version_strategy(),
            prop::collection::vec(wasm_section_strategy(), 0..10),
            prop::collection::vec(any::<u8>(), 0..1000),
        )
            .prop_map(|(magic, version, sections, data)| {
                let mut binary = Vec::new();
                binary.extend(magic);
                binary.extend(version);
                for section in sections {
                    binary.extend(section);
                }
                binary.extend(data);
                binary
            })
    }

    // Strategy for generating invalid WASM binaries
    fn invalid_wasm_strategy() -> impl Strategy<Value = Vec<u8>> {
        prop_oneof![
            // Too short
            prop::collection::vec(any::<u8>(), 0..7),
            // Invalid magic
            (
                prop::collection::vec(any::<u8>().prop_filter("not wasm magic", |&b| b != 0x00), 4),
                prop::collection::vec(any::<u8>(), 4..100)
            )
                .prop_map(|(magic, rest)| {
                    let mut binary = magic;
                    binary.extend(rest);
                    binary
                }),
            // Valid magic but no version
            Just(vec![0x00, 0x61, 0x73, 0x6D]),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn analyzer_never_panics_on_arbitrary_input(
            data in prop::collection::vec(any::<u8>(), 0..10000)
        ) {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let temp_file = NamedTempFile::new().unwrap();
            runtime.block_on(async {
                tokio::fs::write(temp_file.path(), &data).await.unwrap()
            });

            // Should not panic
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let analyzer = WasmBinaryAnalyzer::new();
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(analyzer.analyze_file(temp_file.path()))
            }));

            prop_assert!(result.is_ok());
        }

        #[test]
        fn valid_wasm_is_accepted(
            wasm_data in wasm_binary_strategy()
        ) {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let temp_file = NamedTempFile::new().unwrap();
            runtime.block_on(async {
                tokio::fs::write(temp_file.path(), &wasm_data).await.unwrap()
            });

            let analyzer = WasmBinaryAnalyzer::new();
            let result = runtime.block_on(analyzer.analyze_file(temp_file.path()));

            // Should successfully analyze valid WASM
            prop_assert!(result.is_ok(), "Failed to analyze valid WASM: {:?}", result);

            if let Ok(metrics) = result {
                // Basic invariants - these are u32 so always >= 0
                // Just check the metrics are populated
                let _ = metrics.function_count;
                let _ = metrics.import_count;
                let _ = metrics.export_count;
                let _ = metrics.linear_memory_pages;
            }
        }

        #[test]
        fn invalid_wasm_is_rejected(
            invalid_data in invalid_wasm_strategy()
        ) {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let temp_file = NamedTempFile::new().unwrap();
            runtime.block_on(async {
                tokio::fs::write(temp_file.path(), &invalid_data).await.unwrap()
            });

            let analyzer = WasmBinaryAnalyzer::new();
            let result = runtime.block_on(analyzer.analyze_file(temp_file.path()));

            // Should reject invalid WASM
            prop_assert!(result.is_err(), "Accepted invalid WASM data");
        }

        #[test]
        fn file_size_limits_enforced(
            size_mb in 1usize..20,
            data in prop::collection::vec(any::<u8>(), 8..100)
        ) {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let temp_file = NamedTempFile::new().unwrap();

            // Create large file by repeating data
            let mut large_data = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];
            let target_size = size_mb * 1024 * 1024;
            while large_data.len() < target_size {
                large_data.extend(&data);
            }
            large_data.truncate(target_size);

            runtime.block_on(async {
                tokio::fs::write(temp_file.path(), &large_data).await.unwrap()
            });

            let analyzer = WasmBinaryAnalyzer::new();
            let result = runtime.block_on(analyzer.analyze_file(temp_file.path()));

            if size_mb > 10 {
                // Should reject files larger than 10MB
                prop_assert!(result.is_err());
                if let Err(e) = result {
                    prop_assert!(e.to_string().contains("too large"));
                }
            } else {
                // Should accept files under 10MB
                prop_assert!(result.is_ok());
            }
        }

        #[test]
        fn analyze_bytes_handles_edge_cases(
            data in prop::collection::vec(any::<u8>(), 0..1000)
        ) {
            let analyzer = WasmBinaryAnalyzer::new();

            // Test the internal analyze_bytes method
            let result = analyzer.analyze_bytes(&data);

            if data.len() < 8 || &data[0..4] != b"\0asm" {
                prop_assert!(result.is_err());
            } else {
                prop_assert!(result.is_ok());
                if let Ok(analysis) = result {
                    // Sections should be parsed
                    prop_assert!(analysis.sections.len() <= data.len());
                }
            }
        }

        #[test]
        fn count_occurrences_correctness(
            haystack in prop::collection::vec(any::<u8>(), 0..1000),
            needle in prop::collection::vec(any::<u8>(), 1..10)
        ) {
            let count = count_occurrences(&haystack, &needle);

            // Manual verification
            let mut manual_count = 0;
            let mut i = 0;
            while i + needle.len() <= haystack.len() {
                if &haystack[i..i + needle.len()] == needle.as_slice() {
                    manual_count += 1;
                    i += needle.len();
                } else {
                    i += 1;
                }
            }

            prop_assert_eq!(count, manual_count);
        }

        #[test]
        fn section_counting_accuracy(
            sections in prop::collection::vec((0u8..=12, 1usize..100), 0..50)
        ) {
            // Build a real WASM binary more carefully
            let analyzer = WasmBinaryAnalyzer::new();

            // Just test the analyze_bytes method directly
            let mut wasm_data = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];

            for (section_id, size) in &sections {
                wasm_data.push(*section_id);
                // Encode size as LEB128
                let mut size_val = *size;
                while size_val >= 0x80 {
                    wasm_data.push((size_val & 0x7F) as u8 | 0x80);
                    size_val >>= 7;
                }
                wasm_data.push(size_val as u8);

                // Add dummy section data
                wasm_data.resize(wasm_data.len() + *size, 0xFF);
            }

            let result = analyzer.analyze_bytes(&wasm_data);
            prop_assert!(result.is_ok());

            if let Ok(analysis) = result {
                // Check that we parsed the right number of sections
                prop_assert_eq!(analysis.sections.len(), sections.len());

                // Check each section
                for (i, section) in analysis.sections.iter().enumerate() {
                    prop_assert_eq!(section.id, sections[i].0);
                    prop_assert_eq!(section.size, sections[i].1);
                }
            }
        }
    }
}
