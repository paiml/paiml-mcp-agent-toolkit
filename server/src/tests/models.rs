use crate::models::template::{ParameterType, TemplateCategory, Toolchain};

#[test]
fn test_toolchain_priority() {
    let rust = Toolchain::RustCli {
        cargo_features: vec![],
    };
    let deno = Toolchain::DenoTypescript {
        deno_version: "1.39".to_string(),
    };
    let python = Toolchain::PythonUv {
        python_version: "3.12".to_string(),
    };

    assert_eq!(rust.priority(), 1);
    assert_eq!(deno.priority(), 2);
    assert_eq!(python.priority(), 3);
}

#[test]
fn test_toolchain_as_str() {
    let rust = Toolchain::RustCli {
        cargo_features: vec!["serde".to_string()],
    };
    let deno = Toolchain::DenoTypescript {
        deno_version: "1.39".to_string(),
    };
    let python = Toolchain::PythonUv {
        python_version: "3.12".to_string(),
    };

    assert_eq!(rust.as_str(), "rust");
    assert_eq!(deno.as_str(), "deno");
    assert_eq!(python.as_str(), "python-uv");
}

#[test]
fn test_template_category_serialization() {
    use serde_json;

    let makefile = TemplateCategory::Makefile;
    let readme = TemplateCategory::Readme;
    let gitignore = TemplateCategory::Gitignore;
    let context = TemplateCategory::Context;

    assert_eq!(serde_json::to_string(&makefile).unwrap(), "\"makefile\"");
    assert_eq!(serde_json::to_string(&readme).unwrap(), "\"readme\"");
    assert_eq!(serde_json::to_string(&gitignore).unwrap(), "\"gitignore\"");
    assert_eq!(serde_json::to_string(&context).unwrap(), "\"context\"");
}

#[test]
fn test_parameter_type_serialization() {
    use serde_json;

    let project_name = ParameterType::ProjectName;
    let semver = ParameterType::SemVer;
    let github = ParameterType::GitHubUsername;
    let license = ParameterType::LicenseIdentifier;
    let boolean = ParameterType::Boolean;
    let string = ParameterType::String;

    assert_eq!(
        serde_json::to_string(&project_name).unwrap(),
        "\"project_name\""
    );
    assert_eq!(serde_json::to_string(&semver).unwrap(), "\"sem_ver\"");
    assert_eq!(
        serde_json::to_string(&github).unwrap(),
        "\"git_hub_username\""
    );
    assert_eq!(
        serde_json::to_string(&license).unwrap(),
        "\"license_identifier\""
    );
    assert_eq!(serde_json::to_string(&boolean).unwrap(), "\"boolean\"");
    assert_eq!(serde_json::to_string(&string).unwrap(), "\"string\"");
}
