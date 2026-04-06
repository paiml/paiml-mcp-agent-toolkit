impl Recording {
    /// Create a new recording
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(program: String, args: Vec<String>) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            metadata: RecordingMetadata {
                timestamp,
                program,
                args,
                environment: HashMap::new(),
            },
            snapshots: Vec::new(),
        }
    }

    /// Add a snapshot to the recording
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_snapshot(&mut self, snapshot: Snapshot) {
        self.snapshots.push(snapshot);
    }

    /// Get metadata
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn metadata(&self) -> &RecordingMetadata {
        &self.metadata
    }

    /// Get snapshots
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn snapshots(&self) -> &[Snapshot] {
        &self.snapshots
    }

    /// Get snapshot count
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }

    /// Serialize recording to bytes
    ///
    /// Format:
    /// - Magic header (4 bytes: "PMAT")
    /// - Format version (1 byte)
    /// - Metadata (MessagePack)
    /// - Snapshot count (4 bytes, little-endian u32)
    /// - Snapshots array (MessagePack)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        // Write magic header
        buffer.write_all(MAGIC_HEADER)?;

        // Write format version
        buffer.write_all(&[FORMAT_VERSION])?;

        // Serialize metadata with MessagePack
        let metadata_bytes =
            rmp_serde::to_vec(&self.metadata).context("Failed to serialize metadata")?;
        buffer.write_all(&metadata_bytes)?;

        // Write snapshot count (u32 little-endian)
        let snapshot_count = self.snapshots.len() as u32;
        buffer.write_all(&snapshot_count.to_le_bytes())?;

        // Serialize snapshots with MessagePack
        let snapshots_bytes =
            rmp_serde::to_vec(&self.snapshots).context("Failed to serialize snapshots")?;
        buffer.write_all(&snapshots_bytes)?;

        Ok(buffer)
    }

    /// Deserialize recording from bytes
    ///
    /// Validates magic header, version, and snapshot count before parsing.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(bytes);

        // Validate magic header
        let mut magic = [0u8; 4];
        cursor
            .read_exact(&mut magic)
            .context("Failed to read magic header (file too short or corrupted)")?;

        if &magic != MAGIC_HEADER {
            anyhow::bail!(
                "Invalid magic header: expected {:?}, got {:?}",
                MAGIC_HEADER,
                magic
            );
        }

        // Read and validate version
        let mut version_buf = [0u8; 1];
        cursor
            .read_exact(&mut version_buf)
            .context("Failed to read format version")?;
        let version = version_buf[0];

        if version > FORMAT_VERSION {
            anyhow::bail!(
                "Unsupported format version: {} (current: {})",
                version,
                FORMAT_VERSION
            );
        }

        // Read remaining bytes for MessagePack parsing
        let mut remaining = Vec::new();
        cursor
            .read_to_end(&mut remaining)
            .context("Failed to read remaining data")?;

        // Parse metadata
        let mut mp_cursor = Cursor::new(&remaining);
        let metadata: RecordingMetadata =
            rmp_serde::from_read(&mut mp_cursor).context("Failed to deserialize metadata")?;

        // Calculate position after metadata
        let metadata_end = mp_cursor.position() as usize;
        let after_metadata = &remaining[metadata_end..];

        // Read snapshot count
        if after_metadata.len() < 4 {
            anyhow::bail!("File truncated: missing snapshot count");
        }

        let snapshot_count_bytes: [u8; 4] = after_metadata[0..4]
            .try_into()
            .context("Failed to read snapshot count")?;
        let snapshot_count = u32::from_le_bytes(snapshot_count_bytes);

        // Validate snapshot count (DoS protection)
        if snapshot_count > MAX_SNAPSHOT_COUNT {
            anyhow::bail!(
                "Unreasonable snapshot count: {} (max: {})",
                snapshot_count,
                MAX_SNAPSHOT_COUNT
            );
        }

        // Parse snapshots
        let snapshots_bytes = &after_metadata[4..];
        let snapshots: Vec<Snapshot> =
            rmp_serde::from_slice(snapshots_bytes).context("Failed to deserialize snapshots")?;

        // Verify snapshot count matches array length
        if snapshots.len() != snapshot_count as usize {
            anyhow::bail!(
                "Snapshot count mismatch: declared {}, actual {}",
                snapshot_count,
                snapshots.len()
            );
        }

        Ok(Self {
            metadata,
            snapshots,
        })
    }

    /// Write recording to file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let bytes = self.to_bytes()?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    /// Load recording from file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let bytes = std::fs::read(&path).with_context(|| {
            format!("Failed to read recording file: {}", path.as_ref().display())
        })?;
        Self::from_bytes(&bytes)
    }
}

/// Validate magic header
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn validate_magic_header(bytes: &[u8]) -> bool {
    bytes == MAGIC_HEADER
}
