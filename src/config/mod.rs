use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub models_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub db_path: PathBuf,
    pub max_loaded_models: usize,
    pub default_context_size: usize,
    pub gpu_layers: i32,
    pub server_host: String,
    pub server_port: u16,
    pub log_level: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: usize,
    pub repeat_penalty: f32,
    pub stream_mode: bool,
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
            gpu_layers: 999,
            server_host: "127.0.0.1".to_string(),
            server_port: 11434,
            log_level: "info".to_string(),
            max_tokens: 2048,
            temperature: 0.8,
            top_p: 0.95,
            top_k: 40,
            repeat_penalty: 1.1,
            stream_mode: true,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        // Load .env file if it exists (ignore errors if not found)
        let _ = dotenvy::dotenv();
        
        let home = dirs::home_dir().expect("Could not find home directory");
        let default_base = home.join(".rust-llm-runner");
        
        let config = Config {
            models_dir: Self::get_path_env("RUST_LLM_MODELS_DIR", default_base.join("models")),
            cache_dir: Self::get_path_env("RUST_LLM_CACHE_DIR", default_base.join("cache")),
            db_path: Self::get_path_env("RUST_LLM_DB_PATH", default_base.join("db")),
            max_loaded_models: Self::get_env("RUST_LLM_MAX_LOADED_MODELS", 3),
            default_context_size: Self::get_env("RUST_LLM_CONTEXT_SIZE", 2048),
            gpu_layers: Self::get_env("RUST_LLM_GPU_LAYERS", 999),
            server_host: Self::get_env_string("RUST_LLM_HOST", "127.0.0.1"),
            server_port: Self::get_env("RUST_LLM_PORT", 11434),
            log_level: Self::get_env_string("RUST_LLM_LOG_LEVEL", "info"),
            max_tokens: Self::get_env("RUST_LLM_MAX_TOKENS", 2048),
            temperature: Self::get_env("RUST_LLM_TEMPERATURE", 0.8),
            top_p: Self::get_env("RUST_LLM_TOP_P", 0.95),
            top_k: Self::get_env("RUST_LLM_TOP_K", 40),
            repeat_penalty: Self::get_env("RUST_LLM_REPEAT_PENALTY", 1.1),
            stream_mode: Self::get_env_bool("RUST_LLM_STREAM", true),
        };
        
        std::fs::create_dir_all(&config.models_dir)?;
        std::fs::create_dir_all(&config.cache_dir)?;
        std::fs::create_dir_all(&config.db_path)?;
        
        Ok(config)
    }
    
    fn get_env<T: std::str::FromStr>(key: &str, default: T) -> T {
        env::var(key)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }
    
    fn get_env_string(key: &str, default: &str) -> String {
        env::var(key).unwrap_or_else(|_| default.to_string())
    }
    
    fn get_env_bool(key: &str, default: bool) -> bool {
        env::var(key)
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(default)
    }
    
    fn get_path_env(key: &str, default: PathBuf) -> PathBuf {
        env::var(key)
            .map(PathBuf::from)
            .unwrap_or(default)
    }
    
    pub fn get_model_path(&self, model_name: &str) -> PathBuf {
        self.models_dir.join(model_name)
    }
}
