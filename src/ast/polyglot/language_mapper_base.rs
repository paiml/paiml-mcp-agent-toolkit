// BaseLanguageMapper trait implementation
// Included from language_mapper.rs - shares parent module scope

#[async_trait]
impl LanguageMapper for BaseLanguageMapper {
    fn language(&self) -> Language {
        self.language
    }

    async fn map_file(&self, path: &Path) -> Result<Vec<UnifiedNode>> {
        PolyglotPathValidator::validate_file_path(path)?;

        let content = fs::read_to_string(path).await?;
        self.map_source(&content, path).await
    }

    async fn map_directory(&self, path: &Path, recursive: bool) -> Result<Vec<UnifiedNode>> {
        PolyglotPathValidator::validate_directory_path(path)?;

        let mut nodes = Vec::new();
        let extensions: Vec<String> = self
            .language()
            .file_extensions()
            .iter()
            .map(|ext| ext.to_string())
            .collect();

        let read_dir = tokio::fs::read_dir(path).await?;
        let mut entries = Vec::new();

        tokio::pin!(read_dir);
        while let Some(entry) = read_dir.next_entry().await? {
            entries.push(entry);
        }

        for entry in entries {
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if let Some(ext_str) = ext.to_str() {
                        if extensions.contains(&ext_str.to_string()) {
                            let file_nodes = self.map_file(&path).await?;
                            nodes.extend(file_nodes);
                        }
                    }
                }
            } else if recursive && path.is_dir() {
                let dir_nodes = self.map_directory(&path, recursive).await?;
                nodes.extend(dir_nodes);
            }
        }

        Ok(nodes)
    }

    async fn map_source(&self, _source: &str, _path: &Path) -> Result<Vec<UnifiedNode>> {
        // Base implementation doesn't know how to map source
        // Subclasses should override this
        Err(anyhow!("Source mapping not implemented for this language"))
    }

    fn convert_ast_items(&self, items: &[AstItem], path: &Path) -> Vec<UnifiedNode> {
        items
            .iter()
            .map(|item| UnifiedNode::from_ast_item(item, self.language(), path, None))
            .collect()
    }

    fn clone_box(&self) -> Box<dyn LanguageMapper> {
        Box::new(self.clone())
    }
}
