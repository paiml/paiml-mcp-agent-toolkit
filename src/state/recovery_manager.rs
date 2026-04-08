// RecoveryManager: state recovery orchestrator with adaptive snapshot scheduling

impl<S: AgentState> RecoveryManager<S> {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn new(
        event_store_config: EventStoreConfig,
        snapshot_config: SnapshotConfig,
        snapshot_path: &str,
    ) -> Result<Self, RecoveryError> {
        let event_store = Arc::new(
            EventStore::new(event_store_config)
                .await
                .map_err(|e| RecoveryError::EventStoreError(e.to_string()))?,
        );

        let snapshot_store = Arc::new(
            SnapshotStore::new(snapshot_path, snapshot_config)
                .await
                .map_err(|e| RecoveryError::SnapshotError(e.to_string()))?,
        );

        let snapshot_scheduler = Arc::new(AdaptiveSnapshotScheduler::new(
            SnapshotSchedulerConfig::default(),
        ));

        Ok(Self {
            event_store,
            snapshot_store,
            snapshot_scheduler,
            _phantom: std::marker::PhantomData,
        })
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn recover_state(
        &self,
        initial_state: S,
        partition_key: Option<String>,
    ) -> Result<RestoredState<S>, RecoveryError> {
        let start_time = Instant::now();

        // Find latest snapshot
        let (mut state, starting_event_id) = if let Some(ref pk) = partition_key {
            self.recover_from_partition_snapshot(initial_state, pk).await?
        } else {
            self.recover_from_global_snapshot(initial_state).await?
        };

        // Replay events since snapshot
        let events = if let Some(pk) = partition_key {
            self.event_store
                .get_partition_events(&pk, Some(starting_event_id))
        } else {
            self.event_store.get_events_since(starting_event_id, None)
        };

        let events_replayed = events.len();
        for event in &events {
            state.apply_event(event);
        }

        let _recovery_time = start_time.elapsed();

        Ok(RestoredState {
            state,
            snapshot_id: Uuid::new_v4(), // Would get from actual snapshot
            events_to_replay: events_replayed,
        })
    }

    async fn recover_from_partition_snapshot(
        &self,
        initial_state: S,
        pk: &str,
    ) -> Result<(S, u64), RecoveryError> {
        if let Some(snapshot) = self
            .snapshot_store
            .find_partition_snapshots(pk)
            .into_iter()
            .max_by_key(|s| s.event_id)
        {
            let restored = self
                .snapshot_store
                .load_snapshot::<S>(&snapshot.id)
                .await
                .map_err(|e| RecoveryError::SnapshotError(e.to_string()))?;
            Ok((restored, snapshot.event_id))
        } else {
            Ok((initial_state, 0))
        }
    }

    async fn recover_from_global_snapshot(
        &self,
        initial_state: S,
    ) -> Result<(S, u64), RecoveryError> {
        if let Some(snapshot) = self.snapshot_store.find_latest_snapshot() {
            let restored = self
                .snapshot_store
                .load_snapshot::<S>(&snapshot.id)
                .await
                .map_err(|e| RecoveryError::SnapshotError(e.to_string()))?;
            Ok((restored, snapshot.event_id))
        } else {
            Ok((initial_state, 0))
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn save_snapshot(
        &self,
        state: &S,
        partition_key: Option<String>,
    ) -> Result<SnapshotId, RecoveryError> {
        let event_id = state.last_event_id();

        let snapshot_id = self
            .snapshot_store
            .save_snapshot(state, event_id, partition_key.clone())
            .await
            .map_err(|e| RecoveryError::SnapshotError(e.to_string()))?;

        // Update scheduler metrics
        self.snapshot_scheduler
            .record_snapshot(state.events_since_snapshot(), state.time_since_snapshot());

        Ok(snapshot_id)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn should_snapshot(&self, state: &S) -> bool {
        self.snapshot_scheduler
            .should_snapshot(state.events_since_snapshot(), state.time_since_snapshot())
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn compact_events(&self) -> Result<(), RecoveryError> {
        self.event_store
            .compact()
            .await
            .map_err(|e| RecoveryError::EventStoreError(e.to_string()))?;
        Ok(())
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn cleanup_old_snapshots(&self) -> Result<usize, RecoveryError> {
        self.snapshot_store
            .cleanup_orphaned_files()
            .await
            .map_err(|e| RecoveryError::SnapshotError(e.to_string()))
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Get recovery stats.
    pub fn get_recovery_stats(&self) -> RecoveryStats {
        let event_stats = self.event_store.get_statistics();
        let snapshot_stats = self.snapshot_store.get_statistics();
        let scheduler_config = self.snapshot_scheduler.get_config();

        RecoveryStats {
            total_events: event_stats.total_events,
            total_snapshots: snapshot_stats.total_snapshots,
            compression_ratio: snapshot_stats.compression_ratio,
            next_snapshot_in: scheduler_config.min_time_between_snapshots,
        }
    }
}
