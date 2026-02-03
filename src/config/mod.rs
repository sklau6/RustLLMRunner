use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub models_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub db_path: PathBuf,
    pub max_loaded_models: usize,
    pub default_context_size: usize,
    pub gpu_layers: Option<i32>,
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs::home_dir().expect("Could not find home directory");
        let base_dir = home.join(".rust-llm-runner");
        
        Self {
            models_dir: base_dir.join("models"),
            cache_dir: base_dir.join("cache"),
            db_path: base_dir.join("db"),
            max_loaded_models: 3,
            default_context_size: 2048,
            gpu_layers: Some(-1),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config = Config::default();
        
        std::fs::create_dir_all(&config.models_dir)?;
        std::fs::create_dir_all(&config.cache_dir)?;
        std::fs::create_dir_all(&config.db_path)?;
        
        Ok(config)
    }
    
    pub fn get_model_path(&self, model_name: &str) -> PathBuf {
        self.models_dir.join(model_name)
    }
}
