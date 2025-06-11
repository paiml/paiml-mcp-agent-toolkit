use super::*;
use tempfile::TempDir;

#[test]
fn test_artifact_writer_creation() {
    let temp_dir = TempDir::new().unwrap();
    let writer = ArtifactWriter::new(temp_dir.path().to_path_buf()).unwrap();
    assert!(writer.manifest.is_empty());
}

#[test]
fn test_artifact_writer_with_existing_manifest() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("artifacts.json");
    
    // Create a sample manifest
    let manifest_data = r#"{
        "test.md": {
            "path": "dogfooding/test.md",
            "hash": "abcdef",
            "size": 100,
            "generated_at": "2024-01-01T00:00:00Z",
            "artifact_type": "DogfoodingMarkdown"
        }
    }"#;
    
    fs::write(&manifest_path, manifest_data).unwrap();
    
    let writer = ArtifactWriter::new(temp_dir.path().to_path_buf()).unwrap();
    assert_eq!(writer.manifest.len(), 1);
    assert!(writer.manifest.contains_key("test.md"));
}

#[test]
fn test_create_directory_structure() {
    let temp_dir = TempDir::new().unwrap();
    let writer = ArtifactWriter::new(temp_dir.path().to_path_buf()).unwrap();
    
    writer.create_directory_structure().unwrap();
    
    // Check that all directories were created
    assert!(temp_dir.path().join("dogfooding").exists());
    assert!(temp_dir.path().join("mermaid").exists());
    assert!(temp_dir.path().join("mermaid/ast-generated/simple").exists());
    assert!(temp_dir.path().join("mermaid/ast-generated/styled").exists());
    assert!(temp_dir.path().join("mermaid/non-code/simple").exists());
    assert!(temp_dir.path().join("mermaid/non-code/styled").exists());
    assert!(temp_dir.path().join("mermaid/fixtures").exists());
    assert!(temp_dir.path().join("templates").exists());
}

#[test]
fn test_artifact_type_determination() {
    let temp_dir = TempDir::new().unwrap();
    let mut writer = ArtifactWriter::new(temp_dir.path().to_path_buf()).unwrap();
    
    // Create test artifact tree
    let mut tree = ArtifactTree {
        dogfooding: BTreeMap::new(),
        mermaid: MermaidArtifacts {
            ast_generated: BTreeMap::new(),
            non_code: BTreeMap::new(),
            fixtures: BTreeMap::new(),
        },
        templates: BTreeMap::new(),
    };
    
    tree.dogfooding.insert("readme.md".to_string(), "# Test".to_string());
    tree.dogfooding.insert("data.json".to_string(), "{}".to_string());
    
    writer.write_artifacts(&tree).unwrap();
    
    // Verify artifact types
    let readme_meta = writer.manifest.get("readme.md").unwrap();
    match &readme_meta.artifact_type {
        ArtifactType::DogfoodingMarkdown => {},
        _ => panic!("Expected DogfoodingMarkdown"),
    }
    
    let json_meta = writer.manifest.get("data.json").unwrap();
    match &json_meta.artifact_type {
        ArtifactType::DogfoodingJson => {},
        _ => panic!("Expected DogfoodingJson"),
    }
}

#[test]
fn test_write_mermaid_artifacts() {
    let temp_dir = TempDir::new().unwrap();
    let mut writer = ArtifactWriter::new(temp_dir.path().to_path_buf()).unwrap();
    
    let mut tree = ArtifactTree {
        dogfooding: BTreeMap::new(),
        mermaid: MermaidArtifacts {
            ast_generated: BTreeMap::new(),
            non_code: BTreeMap::new(),
            fixtures: BTreeMap::new(),
        },
        templates: BTreeMap::new(),
    };
    
    // Add test diagrams
    tree.mermaid.ast_generated.insert("simple-diagram.mmd".to_string(), "graph TD".to_string());
    tree.mermaid.ast_generated.insert("styled-diagram.mmd".to_string(), "graph LR".to_string());
    tree.mermaid.non_code.insert("workflow.mmd".to_string(), "flowchart TB".to_string());
    
    writer.write_artifacts(&tree).unwrap();
    
    // Verify files were written to correct locations
    assert!(temp_dir.path().join("mermaid/ast-generated/simple/simple-diagram.mmd").exists());
    assert!(temp_dir.path().join("mermaid/ast-generated/styled/styled-diagram.mmd").exists());
    assert!(temp_dir.path().join("mermaid/non-code/simple/workflow.mmd").exists());
}

#[test]
fn test_manifest_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let mut writer = ArtifactWriter::new(temp_dir.path().to_path_buf()).unwrap();
    
    let mut tree = ArtifactTree {
        dogfooding: BTreeMap::new(),
        mermaid: MermaidArtifacts {
            ast_generated: BTreeMap::new(),
            non_code: BTreeMap::new(),
            fixtures: BTreeMap::new(),
        },
        templates: BTreeMap::new(),
    };
    
    tree.dogfooding.insert("test.md".to_string(), "# Test Content".to_string());
    
    writer.write_artifacts(&tree).unwrap();
    
    // Verify manifest was written
    let manifest_path = temp_dir.path().join("artifacts.json");
    assert!(manifest_path.exists());
    
    // Load and verify manifest content
    let manifest_content = fs::read_to_string(&manifest_path).unwrap();
    let loaded_manifest: BTreeMap<String, ArtifactMetadata> = serde_json::from_str(&manifest_content).unwrap();
    
    assert!(loaded_manifest.contains_key("test.md"));
    assert_eq!(loaded_manifest.get("test.md").unwrap().size, 14);
}

#[test]
fn test_template_writing() {
    let temp_dir = TempDir::new().unwrap();
    let mut writer = ArtifactWriter::new(temp_dir.path().to_path_buf()).unwrap();
    
    let mut tree = ArtifactTree {
        dogfooding: BTreeMap::new(),
        mermaid: MermaidArtifacts {
            ast_generated: BTreeMap::new(),
            non_code: BTreeMap::new(),
            fixtures: BTreeMap::new(),
        },
        templates: BTreeMap::new(),
    };
    
    // Add template
    let template = Template {
        name: "test_template".to_string(),
        content: "Template content".to_string(),
        metadata: BTreeMap::new(),
    };
    
    tree.templates.insert("test_template".to_string(), template);
    
    writer.write_artifacts(&tree).unwrap();
    
    // Verify template was written
    let template_path = temp_dir.path().join("templates/test_template");
    assert!(template_path.exists());
    
    // Verify manifest entry
    assert!(writer.manifest.contains_key("templates/test_template"));
}