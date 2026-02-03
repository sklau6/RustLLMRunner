use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ModelRegistry {
    registry_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistryModel {
    pub name: String,
    pub tags: Vec<String>,
    pub description: String,
    pub downloads: HashMap<String, ModelDownloadInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelDownloadInfo {
    pub url: String,
    pub size: u64,
    pub sha256: String,
    pub format: String,
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self {
            registry_url: "https://registry.ollama.ai".to_string(),
        }
    }
    
    pub async fn get_model_info(&self, model_name: &str) -> Result<RegistryModel> {
        let parts: Vec<&str> = model_name.split(':').collect();
        let name = parts[0];
        let tag = parts.get(1).unwrap_or(&"latest");
        
        let url = match name.to_lowercase().as_str() {
            name if name.contains("llama") => {
                format!("https://huggingface.co/meta-llama/{}/resolve/main/model.gguf", name)
            },
            name if name.contains("qwen") => {
                format!("https://huggingface.co/Qwen/{}/resolve/main/model.gguf", name)
            },
            name if name.contains("gemma") => {
                format!("https://huggingface.co/google/{}/resolve/main/model.gguf", name)
            },
            _ => {
                format!("{}/v2/library/{}/manifests/{}", self.registry_url, name, tag)
            }
        };
        
        Ok(RegistryModel {
            name: name.to_string(),
            tags: vec![tag.to_string()],
            description: format!("{} model", name),
            downloads: HashMap::from([
                (tag.to_string(), ModelDownloadInfo {
                    url,
                    size: 0,
                    sha256: String::new(),
                    format: "gguf".to_string(),
                })
            ]),
        })
    }
    
    pub fn get_huggingface_url(&self, model_name: &str) -> Result<String> {
        let parts: Vec<&str> = model_name.split(':').collect();
        let name = parts[0].to_lowercase();
        let tag = parts.get(1).unwrap_or(&"latest");
        
        let url = if name.contains("llama4") || name.contains("llama-4") {
            format!("https://huggingface.co/meta-llama/Llama-3.2-1B-Instruct-GGUF/resolve/main/Llama-3.2-1B-Instruct-Q4_K_M.gguf")
        } else if name.contains("qwen3") || name.contains("qwen-3") {
            format!("https://huggingface.co/Qwen/Qwen2.5-7B-Instruct-GGUF/resolve/main/qwen2.5-7b-instruct-q4_k_m.gguf")
        } else if name.contains("gemma3") || name.contains("gemma-3") {
            if tag.contains("27b") {
                format!("https://huggingface.co/google/gemma-2-27b-it-GGUF/resolve/main/gemma-2-27b-it-Q4_K_M.gguf")
            } else {
                format!("https://huggingface.co/google/gemma-2-9b-it-GGUF/resolve/main/gemma-2-9b-it-Q4_K_M.gguf")
            }
        } else {
            return Err(anyhow!("Unknown model: {}", model_name));
        };
        
        Ok(url)
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}
