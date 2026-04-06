impl<W: Write> RecordingWriter<W> {
    /// Create a new streaming recording writer
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(mut writer: W, program: String, args: Vec<String>) -> Result<Self> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let metadata = RecordingMetadata {
            timestamp,
            program,
            args,
            environment: HashMap::new(),
        };

        // Write magic header and version
        writer.write_all(MAGIC_HEADER)?;
        writer.write_all(&[FORMAT_VERSION])?;

        Ok(Self {
            writer,
            metadata,
            snapshot_count: 0,
            snapshots_buffer: Vec::with_capacity(4096), // 4KB initial buffer
            finalized: false,
            compression: CompressionLevel::None,
        })
    }

    /// Create writer with specific compression level
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_compression(
        writer: W,
        program: String,
        args: Vec<String>,
        compression: CompressionLevel,
    ) -> Result<Self> {
        let mut recorder = Self::new(writer, program, args)?;
        recorder.compression = compression;
        Ok(recorder)
    }

    /// Add environment variable to metadata
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_environment(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.environment.insert(key.into(), value.into());
    }

    /// Write a snapshot to the recording
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn write_snapshot(&mut self, snapshot: &Snapshot) -> Result<()> {
        if self.finalized {
            anyhow::bail!("Cannot write snapshot: recording has been finalized");
        }

        // Serialize snapshot to MessagePack
        let snapshot_bytes = rmp_serde::to_vec(snapshot).context("Failed to serialize snapshot")?;

        // Accumulate in buffer
        self.snapshots_buffer.extend_from_slice(&snapshot_bytes);
        self.snapshot_count += 1;

        Ok(())
    }

    /// Get current snapshot count
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn snapshot_count(&self) -> u32 {
        self.snapshot_count
    }

    /// Finalize the recording (write metadata, count, and snapshots)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn finalize(mut self) -> Result<()> {
        if self.finalized {
            anyhow::bail!("Recording has already been finalized");
        }

        // Serialize metadata
        let metadata_bytes =
            rmp_serde::to_vec(&self.metadata).context("Failed to serialize metadata")?;
        self.writer.write_all(&metadata_bytes)?;

        // Write snapshot count
        self.writer.write_all(&self.snapshot_count.to_le_bytes())?;

        // Write snapshots as MessagePack array
        // We need to wrap accumulated snapshots in array format
        let mut snapshots_vec = Vec::new();
        let mut cursor = Cursor::new(&self.snapshots_buffer[..]);

        for _ in 0..self.snapshot_count {
            let snapshot: Snapshot = rmp_serde::from_read(&mut cursor)
                .context("Failed to deserialize accumulated snapshot")?;
            snapshots_vec.push(snapshot);
        }

        let snapshots_array_bytes =
            rmp_serde::to_vec(&snapshots_vec).context("Failed to serialize snapshots array")?;

        self.writer.write_all(&snapshots_array_bytes)?;

        self.finalized = true;
        Ok(())
    }
}

impl SnapshotSerializer {
    /// Create a new serializer with default capacity
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self::with_capacity(1024)
    }

    /// Create a serializer with specific initial capacity
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_capacity(capacity: usize) -> Self {
        debug_assert!(capacity > 0, "capacity must be positive");
        Self {
            buffer: Vec::with_capacity(capacity),
            compression: CompressionLevel::None,
        }
    }

    /// Create serializer with compression
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_compression(compression: CompressionLevel) -> Self {
        let mut serializer = Self::new();
        serializer.compression = compression;
        serializer
    }

    /// Serialize a snapshot (reuses internal buffer)
    pub fn serialize(&mut self, snapshot: &Snapshot) -> Result<&[u8]> {
        // Clear buffer but retain capacity
        self.buffer.clear();

        // Serialize to buffer
        rmp_serde::encode::write(&mut self.buffer, snapshot)
            .context("Failed to serialize snapshot")?;

        Ok(&self.buffer)
    }

    /// Get current buffer capacity
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }
}

impl Default for SnapshotSerializer {
    fn default() -> Self {
        Self::new()
    }
}
