#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn test_consensus_storage() {
        let storage = ConsensusStorage::<ExampleState>::new(1, ExampleState::default());

        let entry = Entry {
            term: 1,
            index: 1,
            payload: EntryPayload::Normal(ClientRequest {
                id: Uuid::new_v4(),
                operation: StateOperation::Query("test".to_string()),
            }),
        };

        storage.append_entry_to_log(&entry).await.unwrap();

        let entries = storage.get_log_entries(1, 2).await.unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[actix_rt::test]
    async fn test_consensus_manager() {
        let config = RaftConfig {
            heartbeat_interval: 200,
            election_timeout_min: 300,
            election_timeout_max: 600,
            ..Default::default()
        };

        let manager = ConsensusManager::<ExampleState>::new(1, ExampleState::default(), config)
            .await
            .unwrap();

        assert_eq!(manager.node_id, 1);
    }

    #[test]
    fn test_network_peer_management() {
        let network = ConsensusNetwork::new(1);
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

        network.add_peer(2, addr);
        assert_eq!(network.peers.read().len(), 1);

        network.remove_peer(2);
        assert_eq!(network.peers.read().len(), 0);
    }
}
