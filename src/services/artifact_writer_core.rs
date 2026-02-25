impl ArtifactWriter {
    /// Create a new artifact writer for the given root directory
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::artifact_writer::ArtifactWriter;
    /// use std::path::PathBuf;
    ///
    /// let writer = ArtifactWriter::new(PathBuf::from("/tmp/artifacts")).expect("internal error");
    /// // Writer is ready to store artifacts
    /// ```
    pub fn new(root: PathBuf) -> Result<Self, TemplateError> {
        // Ensure root directory exists
        fs::create_dir_all(&root).map_err(TemplateError::Io)?;

        // Load existing manifest if it exists
        let manifest_path = root.join("artifacts.json");
        let manifest = if manifest_path.exists() {
            let content = fs::read_to_string(&manifest_path).map_err(TemplateError::Io)?;
            serde_json::from_str(&content).map_err(|e| TemplateError::InvalidUtf8(e.to_string()))?
        } else {
            BTreeMap::new()
        };

        Ok(Self { root, manifest })
    }

    /// Write complete artifact tree to storage
    pub fn write_artifacts(&mut self, tree: &ArtifactTree) -> Result<(), TemplateError> {
        // Ensure directory structure exists
        self.create_directory_structure()?;

        // Write dogfooding artifacts
        for (name, content) in &tree.dogfooding {
            let artifact_type = if name.ends_with(".md") {
                ArtifactType::DogfoodingMarkdown
            } else {
                ArtifactType::DogfoodingJson
            };

            let path = self.root.join("dogfooding").join(name);
            let hash = self.write_with_hash(&path, content, artifact_type.clone())?;

            self.manifest.insert(
                name.clone(),
                ArtifactMetadata {
                    path: path.clone(),
                    hash: format!("{hash}"),
                    size: content.len(),
                    generated_at: Utc::now(),
                    artifact_type,
                },
            );
        }

        // Write Mermaid diagrams with directory structure
        self.write_mermaid_artifacts(&tree.mermaid)?;

        // Write templates
        self.write_template_artifacts(&tree.templates)?;

        // Write manifest for verification
        self.write_manifest()?;

        Ok(())
    }

    /// Create the canonical directory structure
    fn create_directory_structure(&self) -> Result<(), TemplateError> {
        let directories = [
            "dogfooding",
            "mermaid",
            "mermaid/ast-generated",
            "mermaid/ast-generated/simple",
            "mermaid/ast-generated/styled",
            "mermaid/non-code",
            "mermaid/non-code/simple",
            "mermaid/non-code/styled",
            "mermaid/fixtures",
            "templates",
        ];

        for dir in &directories {
            let path = self.root.join(dir);
            fs::create_dir_all(&path).map_err(TemplateError::Io)?;
        }

        Ok(())
    }

    /// Write Mermaid artifacts with proper directory organization
    fn write_mermaid_artifacts(
        &mut self,
        artifacts: &MermaidArtifacts,
    ) -> Result<(), TemplateError> {
        // Write AST-generated diagrams
        for (name, content) in &artifacts.ast_generated {
            let subdir = if name.contains("styled") {
                "styled"
            } else {
                "simple"
            };
            let path = self
                .root
                .join("mermaid")
                .join("ast-generated")
                .join(subdir)
                .join(name);

            let hash = self.write_with_hash(&path, content, ArtifactType::MermaidDiagram)?;

            self.manifest.insert(
                format!("mermaid/ast-generated/{subdir}/{name}"),
                ArtifactMetadata {
                    path: path.clone(),
                    hash: format!("{hash}"),
                    size: content.len(),
                    generated_at: Utc::now(),
                    artifact_type: ArtifactType::MermaidDiagram,
                },
            );
        }

        // Write non-code diagrams
        for (name, content) in &artifacts.non_code {
            let subdir = if name.contains("styled") {
                "styled"
            } else {
                "simple"
            };
            let path = self
                .root
                .join("mermaid")
                .join("non-code")
                .join(subdir)
                .join(name);

            let hash = self.write_with_hash(&path, content, ArtifactType::MermaidDiagram)?;

            self.manifest.insert(
                format!("mermaid/non-code/{subdir}/{name}"),
                ArtifactMetadata {
                    path: path.clone(),
                    hash: format!("{hash}"),
                    size: content.len(),
                    generated_at: Utc::now(),
                    artifact_type: ArtifactType::MermaidDiagram,
                },
            );
        }

        Ok(())
    }

    /// Write template artifacts
    fn write_template_artifacts(&mut self, templates: &[Template]) -> Result<(), TemplateError> {
        for template in templates {
            let filename = format!("{}.hbs", template.name);
            let path = self.root.join("templates").join(&filename);

            let hash = self.write_with_hash(&path, &template.content, ArtifactType::Template)?;

            self.manifest.insert(
                format!("templates/{filename}"),
                ArtifactMetadata {
                    path: path.clone(),
                    hash: format!("{hash}"),
                    size: template.content.len(),
                    generated_at: Utc::now(),
                    artifact_type: ArtifactType::Template,
                },
            );
        }

        Ok(())
    }

    /// Write content with atomic operation and return hash
    fn write_with_hash(
        &self,
        path: &Path,
        content: &str,
        _artifact_type: ArtifactType,
    ) -> Result<Hash, TemplateError> {
        // Compute hash first
        let hash = blake3::hash(content.as_bytes());

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(TemplateError::Io)?;
        }

        // Use two-phase write: create with .tmp extension, then rename
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, content).map_err(TemplateError::Io)?;
        fs::rename(temp_path, path).map_err(TemplateError::Io)?;

        Ok(hash)
    }

    /// Write the manifest file
    fn write_manifest(&mut self) -> Result<(), TemplateError> {
        let manifest_path = self.root.join("artifacts.json");
        let manifest_content = serde_json::to_string_pretty(&self.manifest)
            .map_err(|e| TemplateError::InvalidUtf8(e.to_string()))?;

        // Compute hash and add manifest to itself
        let hash = blake3::hash(manifest_content.as_bytes());
        self.manifest.insert(
            "artifacts.json".to_string(),
            ArtifactMetadata {
                path: manifest_path.clone(),
                hash: format!("{hash}"),
                size: manifest_content.len(),
                generated_at: Utc::now(),
                artifact_type: ArtifactType::Manifest,
            },
        );

        // Re-serialize with updated manifest
        let final_content = serde_json::to_string_pretty(&self.manifest)
            .map_err(|e| TemplateError::InvalidUtf8(e.to_string()))?;

        // Atomic write
        let temp_path = manifest_path.with_extension("tmp");
        {
            let file = File::create(&temp_path).map_err(TemplateError::Io)?;
            let mut writer = BufWriter::new(file);
            writer
                .write_all(final_content.as_bytes())
                .map_err(TemplateError::Io)?;
            writer.flush().map_err(TemplateError::Io)?;
        }
        fs::rename(temp_path, manifest_path).map_err(TemplateError::Io)?;

        Ok(())
    }

    /// Verify artifact integrity using stored hashes
    pub fn verify_integrity(&self) -> Result<VerificationReport, TemplateError> {
        let mut report = VerificationReport {
            total_artifacts: self.manifest.len(),
            verified: 0,
            failed: Vec::new(),
            missing: Vec::new(),
        };

        for (name, metadata) in &self.manifest {
            if !metadata.path.exists() {
                report.missing.push(name.clone());
                continue;
            }

            // Read file and compute hash
            let content = fs::read_to_string(&metadata.path).map_err(TemplateError::Io)?;
            let computed_hash = blake3::hash(content.as_bytes());

            if format!("{computed_hash}") == metadata.hash {
                report.verified += 1;
            } else {
                report.failed.push(IntegrityFailure {
                    artifact: name.clone(),
                    expected_hash: metadata.hash.clone(),
                    actual_hash: format!("{computed_hash}"),
                });
            }
        }

        Ok(report)
    }

    /// Get artifact statistics
    #[must_use]
    pub fn get_statistics(&self) -> ArtifactStatistics {
        let mut stats = ArtifactStatistics {
            total_artifacts: self.manifest.len(),
            total_size: 0,
            by_type: BTreeMap::new(),
            oldest: None,
            newest: None,
        };

        for metadata in self.manifest.values() {
            stats.total_size += metadata.size;

            let type_stats = stats
                .by_type
                .entry(format!("{:?}", metadata.artifact_type))
                .or_insert(TypeStatistics { count: 0, size: 0 });
            type_stats.count += 1;
            type_stats.size += metadata.size;

            if stats.oldest.is_none()
                || stats.oldest.as_ref().expect("internal error") > &metadata.generated_at
            {
                stats.oldest = Some(metadata.generated_at);
            }

            if stats.newest.is_none()
                || stats.newest.as_ref().expect("internal error") < &metadata.generated_at
            {
                stats.newest = Some(metadata.generated_at);
            }
        }

        stats
    }

    /// Clean up artifacts older than specified duration
    pub fn cleanup_old_artifacts(
        &mut self,
        max_age_days: u32,
    ) -> Result<CleanupReport, TemplateError> {
        let cutoff = Utc::now() - chrono::Duration::days(i64::from(max_age_days));
        let mut removed = Vec::new();
        let mut failed = Vec::new();

        let old_artifacts: Vec<_> = self
            .manifest
            .iter()
            .filter(|(_, metadata)| metadata.generated_at < cutoff)
            .map(|(name, _)| name.clone())
            .collect();

        for name in old_artifacts {
            if let Some(metadata) = self.manifest.remove(&name) {
                match fs::remove_file(&metadata.path) {
                    Ok(()) => removed.push(name),
                    Err(e) => {
                        failed.push((name.clone(), e.to_string()));
                        // Re-add to manifest if removal failed
                        self.manifest.insert(name, metadata);
                    }
                }
            }
        }

        // Update manifest if any files were removed
        if !removed.is_empty() {
            self.write_manifest()?;
        }

        Ok(CleanupReport { removed, failed })
    }
}
