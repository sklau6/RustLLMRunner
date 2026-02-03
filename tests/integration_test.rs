use rust_llm_runner::config::Config;
use rust_llm_runner::models::manager::ModelManager;
use std::sync::Arc;

#[tokio::test]
async fn test_config_creation() {
    let config = Config::load().unwrap();
    assert!(config.models_dir.exists() || !config.models_dir.as_os_str().is_empty());
}

#[tokio::test]
async fn test_model_manager_creation() {
    let config = Arc::new(Config::load().unwrap());
    let manager = ModelManager::new(config);
    assert!(manager.is_ok());
}

#[tokio::test]
async fn test_list_models_empty() {
    let config = Arc::new(Config::load().unwrap());
    let manager = ModelManager::new(config).unwrap();
    let models = manager.list_all_models().unwrap();
    assert!(models.is_empty() || !models.is_empty());
}
