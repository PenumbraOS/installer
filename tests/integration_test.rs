//! Integration tests for the penumbra installer

use penumbra_installer::ConfigLoader;
use std::path::Path;

#[tokio::test]
async fn test_builtin_config_loading() {
    let config = ConfigLoader::load_builtin("penumbra").expect("Failed to load builtin config");
    
    assert_eq!(config.name, "PenumbraOS");
    assert_eq!(config.repositories.len(), 3);
    
    // Check repositories exist
    let repo_names: Vec<_> = config.repositories.iter().map(|r| r.name.as_str()).collect();
    assert!(repo_names.contains(&"pinitd"));
    assert!(repo_names.contains(&"sdk"));
    assert!(repo_names.contains(&"mabl"));
}

#[tokio::test]
async fn test_custom_config_loading() {
    let config_path = Path::new("configs/examples/custom.yml");
    if config_path.exists() {
        let config = ConfigLoader::load_from_file(config_path).await
            .expect("Failed to load custom config");
        
        assert_eq!(config.name, "My Custom Android Project");
        assert!(!config.repositories.is_empty());
    }
}

#[test]
fn test_config_validation() {
    // Test empty repositories
    let invalid_config = r#"
name: "Test"
repositories: []
"#;
    
    let result = ConfigLoader::load_from_str(invalid_config);
    assert!(result.is_err());
    
    // Test valid minimal config
    let valid_config = r#"
name: "Test"
repositories:
  - name: "test"
    owner: "testorg"
    repo: "testrepo"
    version: "latest"
    assets: ["*.apk"]
    installation:
      - type: "InstallApks"
        priority_order: ["*"]
"#;
    
    let result = ConfigLoader::load_from_str(valid_config);
    assert!(result.is_ok());
}

#[test]
fn test_repository_filtering() {
    let config = ConfigLoader::load_builtin("penumbra").expect("Failed to load config");
    
    // Test valid filter
    let filtered = config.filter_repositories(&["pinitd".to_string()]).unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "pinitd");
    
    // Test invalid filter
    let result = config.filter_repositories(&["nonexistent".to_string()]);
    assert!(result.is_err());
    
    // Test multiple filters
    let filtered = config.filter_repositories(&["pinitd".to_string(), "sdk".to_string()]).unwrap();
    assert_eq!(filtered.len(), 2);
}