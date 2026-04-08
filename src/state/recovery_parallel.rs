// Parallel recovery for partitioned state

/// Parallel recovery.
pub struct ParallelRecovery<S: AgentState> {
    managers: Vec<Arc<RecoveryManager<S>>>,
}

impl<S: AgentState> ParallelRecovery<S> {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn new(
        num_partitions: usize,
        event_config: EventStoreConfig,
        snapshot_config: SnapshotConfig,
        base_path: &str,
    ) -> Result<Self, RecoveryError> {
        let mut managers = Vec::with_capacity(num_partitions);

        for i in 0..num_partitions {
            let snapshot_path = format!("{}/partition_{}", base_path, i);
            let manager = Arc::new(
                RecoveryManager::new(
                    event_config.clone(),
                    snapshot_config.clone(),
                    &snapshot_path,
                )
                .await?,
            );
            managers.push(manager);
        }

        Ok(Self { managers })
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn recover_all_partitions(
        &self,
        initial_state_factory: impl Fn() -> S + Send + Sync,
    ) -> Result<Vec<RestoredState<S>>, RecoveryError> {
        use futures::future::try_join_all;

        let futures: Vec<_> = self
            .managers
            .iter()
            .enumerate()
            .map(|(i, manager)| {
                let initial = initial_state_factory();
                let partition_key = format!("partition_{}", i);
                async move { manager.recover_state(initial, Some(partition_key)).await }
            })
            .collect();

        try_join_all(futures).await
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn merge_partitions(
        &self,
        states: Vec<RestoredState<S>>,
    ) -> Result<S, RecoveryError> {
        if states.is_empty() {
            return Err(RecoveryError::RecoveryFailed(
                "No partitions to merge".to_string(),
            ));
        }

        let mut states_iter = states.into_iter();
        let mut merged = states_iter.next().expect("internal error").state;

        for restored in states_iter {
            merged.merge_partition(restored.state);
        }

        Ok(merged)
    }
}
