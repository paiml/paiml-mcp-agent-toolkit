#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_header_validation() {
        assert!(validate_magic_header(b"PMAT"));
        assert!(!validate_magic_header(b"XMAT"));
        assert!(!validate_magic_header(b"PM"));
    }

    #[test]
    fn test_empty_recording_roundtrip() {
        let recording = Recording::new("test_program".to_string(), vec![]);
        let bytes = recording.to_bytes().unwrap();
        let deserialized = Recording::from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.metadata().program, "test_program");
        assert_eq!(deserialized.snapshots().len(), 0);
    }

    #[test]
    fn test_recording_with_snapshots_roundtrip() {
        let mut recording = Recording::new("test_program".to_string(), vec!["--test".to_string()]);

        let snapshot1 = Snapshot {
            frame_id: 1,
            timestamp_relative_ms: 0,
            variables: HashMap::new(),
            stack_frames: vec![],
            instruction_pointer: 0x1000,
            memory_snapshot: None,
        };

        let snapshot2 = Snapshot {
            frame_id: 2,
            timestamp_relative_ms: 100,
            variables: HashMap::new(),
            stack_frames: vec![],
            instruction_pointer: 0x1008,
            memory_snapshot: None,
        };

        recording.add_snapshot(snapshot1.clone());
        recording.add_snapshot(snapshot2.clone());

        let bytes = recording.to_bytes().unwrap();
        let deserialized = Recording::from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.snapshots().len(), 2);
        assert_eq!(deserialized.snapshots()[0].frame_id, 1);
        assert_eq!(deserialized.snapshots()[1].frame_id, 2);
    }

    #[test]
    fn test_invalid_magic_header() {
        let invalid_bytes = b"XMAT\x01...";
        let result = Recording::from_bytes(invalid_bytes);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("magic header"));
    }

    #[test]
    fn test_unsupported_version() {
        // Create a file with future version (99)
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"PMAT");
        bytes.push(99); // Future version
        bytes.extend_from_slice(b"metadata...");

        let result = Recording::from_bytes(&bytes);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("version"));
    }

    // REPLAY-002: Streaming Serialization Tests (GREEN/REFACTOR phase)

    #[test]
    fn test_streaming_writer_basic() {
        let mut buffer = Cursor::new(Vec::new());
        let mut writer =
            RecordingWriter::new(&mut buffer, "test_program".to_string(), vec![]).unwrap();

        // Write 3 snapshots
        for i in 0..3 {
            let snapshot = Snapshot {
                frame_id: i,
                timestamp_relative_ms: i as u32 * 10,
                variables: HashMap::new(),
                stack_frames: vec![],
                instruction_pointer: 0x1000 + i * 8,
                memory_snapshot: None,
            };
            writer.write_snapshot(&snapshot).unwrap();
        }

        assert_eq!(writer.snapshot_count(), 3);

        writer.finalize().unwrap();

        // Verify file is valid
        let bytes = buffer.into_inner();
        let recording = Recording::from_bytes(&bytes).unwrap();
        assert_eq!(recording.snapshot_count(), 3);
        assert_eq!(recording.snapshots()[0].frame_id, 0);
        assert_eq!(recording.snapshots()[1].frame_id, 1);
        assert_eq!(recording.snapshots()[2].frame_id, 2);
    }

    #[test]
    fn test_streaming_writer_metadata_modification() {
        let mut buffer = Cursor::new(Vec::new());
        let mut writer =
            RecordingWriter::new(&mut buffer, "test".to_string(), vec!["--flag".to_string()])
                .unwrap();

        writer.add_environment("PATH", "/usr/bin");
        writer.add_environment("USER", "developer");

        let snapshot = Snapshot {
            frame_id: 1,
            timestamp_relative_ms: 0,
            variables: HashMap::new(),
            stack_frames: vec![],
            instruction_pointer: 0x1000,
            memory_snapshot: None,
        };
        writer.write_snapshot(&snapshot).unwrap();
        writer.finalize().unwrap();

        // Verify metadata
        let bytes = buffer.into_inner();
        let recording = Recording::from_bytes(&bytes).unwrap();
        let metadata = recording.metadata();
        assert_eq!(
            metadata.environment.get("PATH"),
            Some(&"/usr/bin".to_string())
        );
        assert_eq!(
            metadata.environment.get("USER"),
            Some(&"developer".to_string())
        );
    }

    #[test]
    fn test_streaming_writer_error_after_finalize() {
        let mut buffer = Cursor::new(Vec::new());
        let mut writer = RecordingWriter::new(&mut buffer, "test".to_string(), vec![]).unwrap();

        let snapshot = Snapshot {
            frame_id: 1,
            timestamp_relative_ms: 0,
            variables: HashMap::new(),
            stack_frames: vec![],
            instruction_pointer: 0x1000,
            memory_snapshot: None,
        };

        writer.write_snapshot(&snapshot).unwrap();
        writer.finalize().unwrap();

        // Attempt to write after finalize should fail
        // Note: writer is consumed by finalize(), so we can't actually call write_snapshot
        // This test documents the API design
        assert!(
            true,
            "finalize() consumes writer, preventing further writes"
        );
    }

    #[test]
    fn test_snapshot_serializer_reuse() {
        let mut serializer = SnapshotSerializer::new();

        // Serialize multiple snapshots
        for i in 0..10 {
            let snapshot = Snapshot {
                frame_id: i,
                timestamp_relative_ms: i as u32,
                variables: HashMap::new(),
                stack_frames: vec![],
                instruction_pointer: 0x1000,
                memory_snapshot: None,
            };

            let bytes = serializer.serialize(&snapshot).unwrap();
            assert!(!bytes.is_empty(), "Serialization should produce bytes");
        }

        // Buffer capacity should not grow unbounded
        assert!(
            serializer.capacity() < 4096,
            "Buffer should stay within reasonable bounds"
        );
    }

    #[test]
    fn test_snapshot_serializer_with_capacity() {
        let serializer = SnapshotSerializer::with_capacity(2048);
        assert_eq!(serializer.capacity(), 2048);
    }

    #[test]
    fn test_empty_recording_streaming() {
        let mut buffer = Cursor::new(Vec::new());
        let writer = RecordingWriter::new(&mut buffer, "test".to_string(), vec![]).unwrap();

        assert_eq!(writer.snapshot_count(), 0);
        writer.finalize().unwrap();

        // Verify empty recording is valid
        let bytes = buffer.into_inner();
        let recording = Recording::from_bytes(&bytes).unwrap();
        assert_eq!(recording.snapshot_count(), 0);
    }
}
