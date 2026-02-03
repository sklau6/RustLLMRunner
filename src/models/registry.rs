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
        let name = parts[0];
        let name_lower = name.to_lowercase();
        let tag = parts.get(1).unwrap_or(&"latest");
        
        let url = if name.contains('/') {
            // Direct HuggingFace path: user/repo-name
            // Try to find a GGUF file - common patterns
            let repo = name;
            if name_lower.contains("gguf") {
                // Repo name contains GGUF, likely has GGUF files
                format!("https://huggingface.co/{}/resolve/main/model.gguf", repo)
            } else {
                format!("https://huggingface.co/{}-GGUF/resolve/main/model.gguf", repo)
            }
        } else if name_lower.contains("llama4") || name_lower.contains("llama-4") || name_lower.contains("llama3") {
            "https://huggingface.co/QuantFactory/Meta-Llama-3.1-8B-Instruct-GGUF/resolve/main/Meta-Llama-3.1-8B-Instruct.Q4_K_M.gguf".to_string()
        } else if name_lower.contains("qwen3") || name_lower.contains("qwen-3") || name_lower.contains("qwen2") {
            "https://huggingface.co/Qwen/Qwen2.5-7B-Instruct-GGUF/resolve/main/qwen2.5-7b-instruct-q4_k_m.gguf".to_string()
        } else if name_lower.contains("gemma3") || name_lower.contains("gemma-3") || name_lower.contains("gemma2") {
            if tag.contains("27b") {
                "https://huggingface.co/bartowski/gemma-2-27b-it-GGUF/resolve/main/gemma-2-27b-it-Q4_K_M.gguf".to_string()
            } else {
                "https://huggingface.co/bartowski/gemma-2-9b-it-GGUF/resolve/main/gemma-2-9b-it-Q4_K_M.gguf".to_string()
            }
        } else if name_lower.contains("mistral") {
            "https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.2-GGUF/resolve/main/mistral-7b-instruct-v0.2.Q4_K_M.gguf".to_string()
        } else if name_lower.contains("phi") {
            "https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4.gguf".to_string()
        } else {
            return Err(anyhow!("Unknown model: {}. Try using a direct HuggingFace path like 'username/model-GGUF'", model_name));
        };
        
        Ok(url)
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}
