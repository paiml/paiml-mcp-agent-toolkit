//! Workflow repository for persistence
//!
//! Provides storage and retrieval of workflow definitions.

use super::*;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// In-memory workflow repository implementation
pub struct InMemoryWorkflowRepository {
    workflows: Arc<RwLock<HashMap<Uuid, Workflow>>>,
    name_index: Arc<RwLock<HashMap<String, Uuid>>>,
}

impl Default for InMemoryWorkflowRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryWorkflowRepository {
    pub fn new() -> Self {
        Self {
            workflows: Arc::new(RwLock::new(HashMap::new())),
            name_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn count(&self) -> usize {
        self.workflows.read().len()
    }

    pub fn clear(&self) {
        self.workflows.write().clear();
        self.name_index.write().clear();
    }
}

#[async_trait]
impl WorkflowRepository for InMemoryWorkflowRepository {
    async fn save(&self, workflow: &Workflow) -> Result<(), WorkflowError> {
        self.workflows.write().insert(workflow.id, workflow.clone());
        self.name_index.write().insert(workflow.name.clone(), workflow.id);
        Ok(())
    }

    async fn get(&self, id: Uuid) -> Result<Option<Workflow>, WorkflowError> {
        Ok(self.workflows.read().get(&id).cloned())
    }

    async fn list(&self) -> Result<Vec<Workflow>, WorkflowError> {
        Ok(self.workflows.read().values().cloned().collect())
    }

    async fn delete(&self, id: Uuid) -> Result<(), WorkflowError> {
        if let Some(workflow) = self.workflows.write().remove(&id) {
            self.name_index.write().remove(&workflow.name);
        }
        Ok(())
    }

    async fn get_by_name(&self, name: &str) -> Result<Option<Workflow>, WorkflowError> {
        let id = self.name_index.read().get(name).copied();
        if let Some(id) = id {
            self.get(id).await
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_workflow(name: &str) -> Workflow {
        WorkflowBuilder::new(name)
            .description("Test workflow")
            .version("1.0.0")
            .build()
    }

    #[tokio::test]
    async fn test_repository_creation() {
        let repo = InMemoryWorkflowRepository::new();
        assert_eq!(repo.count(), 0);
    }

    #[tokio::test]
    async fn test_save_workflow() {
        let repo = InMemoryWorkflowRepository::new();
        let workflow = create_test_workflow("test_workflow");

        repo.save(&workflow).await.unwrap();
        assert_eq!(repo.count(), 1);
    }

    #[tokio::test]
    async fn test_get_workflow() {
        let repo = InMemoryWorkflowRepository::new();
        let workflow = create_test_workflow("test_workflow");
        let id = workflow.id;

        repo.save(&workflow).await.unwrap();

        let retrieved = repo.get(id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test_workflow");
    }

    #[tokio::test]
    async fn test_get_nonexistent_workflow() {
        let repo = InMemoryWorkflowRepository::new();
        let id = Uuid::new_v4();

        let retrieved = repo.get(id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_list_workflows() {
        let repo = InMemoryWorkflowRepository::new();

        repo.save(&create_test_workflow("workflow1")).await.unwrap();
        repo.save(&create_test_workflow("workflow2")).await.unwrap();
        repo.save(&create_test_workflow("workflow3")).await.unwrap();

        let workflows = repo.list().await.unwrap();
        assert_eq!(workflows.len(), 3);
    }

    #[tokio::test]
    async fn test_delete_workflow() {
        let repo = InMemoryWorkflowRepository::new();
        let workflow = create_test_workflow("test_workflow");
        let id = workflow.id;

        repo.save(&workflow).await.unwrap();
        assert_eq!(repo.count(), 1);

        repo.delete(id).await.unwrap();
        assert_eq!(repo.count(), 0);
    }

    #[tokio::test]
    async fn test_get_by_name() {
        let repo = InMemoryWorkflowRepository::new();
        let workflow = create_test_workflow("my_workflow");

        repo.save(&workflow).await.unwrap();

        let retrieved = repo.get_by_name("my_workflow").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "my_workflow");
    }

    #[tokio::test]
    async fn test_get_by_name_nonexistent() {
        let repo = InMemoryWorkflowRepository::new();

        let retrieved = repo.get_by_name("nonexistent").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_update_workflow() {
        let repo = InMemoryWorkflowRepository::new();
        let mut workflow = create_test_workflow("test_workflow");
        let id = workflow.id;

        repo.save(&workflow).await.unwrap();

        // Update workflow
        workflow.version = "2.0.0".to_string();
        repo.save(&workflow).await.unwrap();

        let retrieved = repo.get(id).await.unwrap().unwrap();
        assert_eq!(retrieved.version, "2.0.0");
        assert_eq!(repo.count(), 1); // Should still be 1, not 2
    }

    #[tokio::test]
    async fn test_clear_repository() {
        let repo = InMemoryWorkflowRepository::new();

        repo.save(&create_test_workflow("workflow1")).await.unwrap();
        repo.save(&create_test_workflow("workflow2")).await.unwrap();
        assert_eq!(repo.count(), 2);

        repo.clear();
        assert_eq!(repo.count(), 0);
    }
}
