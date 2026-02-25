// ConsensusStorage: inherent methods + RaftStorage trait implementation

impl<S: AgentState> ConsensusStorage<S> {
    pub fn new(node_id: NodeId, initial_state: S) -> Self {
        Self {
            node_id,
            log: Arc::new(RwLock::new(BTreeMap::new())),
            state_machine: Arc::new(RwLock::new(initial_state)),
            current_term: Arc::new(RwLock::new(0)),
            voted_for: Arc::new(RwLock::new(None)),
            membership: Arc::new(RwLock::new(MembershipConfig::new_initial(node_id))),
            snapshot: Arc::new(RwLock::new(None)),
        }
    }

    async fn apply_entry(&self, entry: &Entry<ClientRequest>) -> ClientResponse {
        match &entry.payload {
            EntryPayload::Normal(request) => {
                match &request.operation {
                    StateOperation::Apply(event) => {
                        let mut state = self.state_machine.write();
                        state.apply_event(event);
                        ClientResponse {
                            success: true,
                            result: Some(serde_json::json!({"applied": true})),
                        }
                    }
                    StateOperation::Snapshot(data) => {
                        if let Ok(new_state) = bincode::deserialize::<S>(data) {
                            *self.state_machine.write() = new_state;
                            ClientResponse {
                                success: true,
                                result: Some(serde_json::json!({"snapshot_applied": true})),
                            }
                        } else {
                            ClientResponse {
                                success: false,
                                result: Some(
                                    serde_json::json!({"error": "Failed to deserialize snapshot"}),
                                ),
                            }
                        }
                    }
                    StateOperation::Query(query) => {
                        // Read-only query
                        let state = self.state_machine.read();
                        ClientResponse {
                            success: true,
                            result: Some(serde_json::json!({
                                "query": query,
                                "last_event_id": state.last_event_id(),
                            })),
                        }
                    }
                }
            }
            EntryPayload::ConfigChange(membership) => {
                *self.membership.write() = membership.clone();
                ClientResponse {
                    success: true,
                    result: Some(serde_json::json!({"membership_updated": true})),
                }
            }
            _ => ClientResponse {
                success: false,
                result: None,
            },
        }
    }
}

#[async_trait]
impl<S: AgentState> RaftStorage<ClientRequest, ClientResponse> for ConsensusStorage<S> {
    type Snapshot = Vec<u8>;
    type ShutdownError = std::io::Error;

    async fn get_membership_config(&self) -> Result<MembershipConfig, std::io::Error> {
        Ok(self.membership.read().clone())
    }

    async fn get_initial_state(&self) -> Result<async_raft::storage::InitialState, std::io::Error> {
        let membership = self.membership.read().clone();
        let mut last_log_index = 0;
        let mut last_log_term = 0;

        if let Some(last_entry) = self.log.read().iter().rev().next() {
            last_log_index = *last_entry.0;
            last_log_term = last_entry.1.term;
        }

        let last_applied_log = if let Some(snapshot) = &*self.snapshot.read() {
            snapshot.index
        } else {
            last_log_index
        };

        Ok(async_raft::storage::InitialState {
            last_log_index,
            last_log_term,
            last_applied_log,
            hard_state: async_raft::storage::HardState {
                current_term: *self.current_term.read(),
                voted_for: *self.voted_for.read(),
            },
            membership,
        })
    }

    async fn save_hard_state(
        &self,
        hs: &async_raft::storage::HardState,
    ) -> Result<(), std::io::Error> {
        *self.current_term.write() = hs.current_term;
        *self.voted_for.write() = hs.voted_for;
        Ok(())
    }

    async fn get_log_entries(
        &self,
        start: u64,
        stop: u64,
    ) -> Result<Vec<Entry<ClientRequest>>, std::io::Error> {
        let log = self.log.read();
        let entries: Vec<_> = log
            .range(start..stop)
            .map(|(_, entry)| entry.clone())
            .collect();
        Ok(entries)
    }

    async fn delete_logs_from(&self, start: u64, stop: Option<u64>) -> Result<(), std::io::Error> {
        let mut log = self.log.write();
        let keys_to_remove: Vec<_> = if let Some(stop) = stop {
            log.range(start..stop).map(|(k, _)| *k).collect()
        } else {
            log.range(start..).map(|(k, _)| *k).collect()
        };

        for key in keys_to_remove {
            log.remove(&key);
        }
        Ok(())
    }

    async fn append_entry_to_log(
        &self,
        entry: &Entry<ClientRequest>,
    ) -> Result<(), std::io::Error> {
        self.log.write().insert(entry.index, entry.clone());
        Ok(())
    }

    async fn replicate_to_log(
        &self,
        entries: &[Entry<ClientRequest>],
    ) -> Result<(), std::io::Error> {
        let mut log = self.log.write();
        for entry in entries {
            log.insert(entry.index, entry.clone());
        }
        Ok(())
    }

    async fn apply_entry_to_state_machine(
        &self,
        index: &u64,
        data: &ClientRequest,
    ) -> Result<ClientResponse, std::io::Error> {
        let log = self.log.read();
        if let Some(entry) = log.get(index) {
            Ok(self.apply_entry(entry).await)
        } else {
            Ok(ClientResponse {
                success: false,
                result: Some(serde_json::json!({"error": "Entry not found"})),
            })
        }
    }

    async fn replicate_to_state_machine(
        &self,
        entries: &[(&u64, &ClientRequest)],
    ) -> Result<(), std::io::Error> {
        for (index, _data) in entries {
            if let Some(entry) = self.log.read().get(index) {
                self.apply_entry(entry).await;
            }
        }
        Ok(())
    }

    async fn do_log_compaction(&self) -> Result<Vec<u8>, std::io::Error> {
        let state = self.state_machine.read();
        let snapshot_data = bincode::serialize(&*state)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let last_log_index = self
            .log
            .read()
            .iter()
            .rev()
            .next()
            .map(|(idx, _)| *idx)
            .unwrap_or(0);

        let snapshot = RaftSnapshot {
            index: last_log_index,
            term: *self.current_term.read(),
            membership: self.membership.read().clone(),
            state: snapshot_data.clone(),
        };

        *self.snapshot.write() = Some(snapshot);

        Ok(snapshot_data)
    }

    async fn create_snapshot(
        &self,
    ) -> Result<
        (
            async_raft::storage::CurrentSnapshotData<Self::Snapshot>,
            MembershipConfig,
        ),
        std::io::Error,
    > {
        let snapshot_bytes = self.do_log_compaction().await?;
        let last_applied_log = self
            .log
            .read()
            .iter()
            .rev()
            .next()
            .map(|(idx, _)| *idx)
            .unwrap_or(0);

        let snapshot_data = async_raft::storage::CurrentSnapshotData {
            index: last_applied_log,
            term: *self.current_term.read(),
            membership: self.membership.read().clone(),
            snapshot: snapshot_bytes,
        };

        Ok((snapshot_data, self.membership.read().clone()))
    }

    async fn finalize_snapshot_installation(
        &self,
        index: u64,
        term: u64,
        delete_through: Option<u64>,
        id: String,
        snapshot: Self::Snapshot,
    ) -> Result<(), std::io::Error> {
        // Apply snapshot to state machine
        if let Ok(new_state) = bincode::deserialize::<S>(&snapshot) {
            *self.state_machine.write() = new_state;
        }

        // Delete old log entries
        if let Some(through) = delete_through {
            self.delete_logs_from(0, Some(through + 1)).await?;
        }

        // Save snapshot metadata
        let snapshot = RaftSnapshot {
            index,
            term,
            membership: self.membership.read().clone(),
            state: snapshot,
        };
        *self.snapshot.write() = Some(snapshot);

        Ok(())
    }

    async fn get_current_snapshot(
        &self,
    ) -> Result<Option<async_raft::storage::CurrentSnapshotData<Vec<u8>>>, std::io::Error> {
        if let Some(snapshot) = &*self.snapshot.read() {
            Ok(Some(async_raft::storage::CurrentSnapshotData {
                index: snapshot.index,
                term: snapshot.term,
                membership: snapshot.membership.clone(),
                snapshot: snapshot.state.clone(),
            }))
        } else {
            Ok(None)
        }
    }
}
